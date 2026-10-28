//! Home grid: its rows and destinations, moving the selection between cards and opening a destination.

use super::*;

impl App {
  /// Lists the Home catalog for the current language and detected capabilities.
  pub fn home_rows(&self) -> Vec<HomeRow> {
    home_catalog_rows(self.lang, self.home_availability())
  }

  /// Destinations in the order the Home lists them, without headings.
  pub fn home_destinations(&self) -> Vec<HomeDestination> {
    self
      .home_rows()
      .iter()
      .filter_map(|row| match row {
        HomeRow::Item { destination, .. } | HomeRow::Standalone { destination, .. } => {
          Some(*destination)
        }
        HomeRow::Header(_) => None,
      })
      .collect()
  }

  /// Runtime facts that decide which Home destinations are listed.
  pub(super) fn home_availability(&self) -> HomeAvailability {
    HomeAvailability {
      hyprctl: self.capabilities.has_hyprctl,
      loginctl: self.capabilities.has_loginctl,
      sessionctl: self.capabilities.has_sessionctl,
      bluetooth: self.capabilities.has_bluetooth,
      systemd: self.capabilities.has_systemd,
      podman: self.capabilities.has_podman,
      docker: self.capabilities.has_docker,
    }
  }

  pub fn home_item_count(&self) -> usize {
    self.home_destinations().len()
  }

  pub fn move_home(&mut self, delta: isize) {
    let last = self.home_item_count().saturating_sub(1);
    self.home_selected = (self.home_selected as isize + delta).clamp(0, last as isize) as usize;
  }

  /// Moves focus to the first item in the adjacent home category.
  pub fn move_home_category(&mut self, delta: isize) {
    let categories = self.home_category_items();
    let Some(current) = categories
      .iter()
      .position(|items| items.contains(&self.home_selected))
    else {
      return;
    };
    let target =
      (current as isize + delta).clamp(0, categories.len().saturating_sub(1) as isize) as usize;
    if let Some(item) = categories[target].first() {
      self.home_selected = *item;
    }
  }

  pub(super) fn home_category_items(&self) -> Vec<Vec<usize>> {
    let mut categories = Vec::new();
    let mut current = Vec::new();
    let mut item_index = 0;
    for row in self.home_rows() {
      match row {
        HomeRow::Header(_) => {
          if !current.is_empty() {
            categories.push(std::mem::take(&mut current));
          }
        }
        HomeRow::Item { .. } => {
          current.push(item_index);
          item_index += 1;
        }
        // The standalone entry is a category of its own, below the cards.
        HomeRow::Standalone { .. } => {
          if !current.is_empty() {
            categories.push(std::mem::take(&mut current));
          }
          categories.push(vec![item_index]);
          item_index += 1;
        }
      }
    }
    if !current.is_empty() {
      categories.push(current);
    }
    categories
  }

  /// Returns the responsive number of dashboard columns for a terminal width.
  pub const fn home_grid_columns(width: u16) -> usize {
    match width {
      140..=u16::MAX => 4,
      100..=139 => 3,
      70..=99 => 2,
      _ => 1,
    }
  }

  /// Opens the destination selected on the Home, if there is one.
  pub fn open_home(&mut self) {
    let Some(destination) = self.home_destinations().get(self.home_selected).copied() else {
      return;
    };
    self.open_destination(destination);
  }

  /// Routes to the page of one Home destination. Each arm keeps the route and
  /// the reload that the destination used before the catalog existed.
  pub(super) fn open_destination(&mut self, destination: HomeDestination) {
    match destination {
      #[cfg(feature = "appearance")]
      HomeDestination::DesktopStyle => self.open_appearance_page(AppearancePage::Desktop),
      #[cfg(feature = "appearance")]
      HomeDestination::ApplicationStyle => {
        self.open_appearance_page(AppearancePage::ApplicationStyle)
      }
      #[cfg(feature = "appearance")]
      HomeDestination::Taskbar => self.open_appearance_surface(AppearancePage::Taskbar),
      #[cfg(feature = "appearance")]
      HomeDestination::ControlCenter => self.open_appearance_surface(AppearancePage::ControlPanel),
      #[cfg(feature = "appearance")]
      HomeDestination::WidgetTelemetry => {
        self.open_appearance_surface(AppearancePage::WidgetTelemetry)
      }
      HomeDestination::Hyprland => self.route = Route::Hyprland,
      #[cfg(feature = "displays")]
      HomeDestination::Displays => {
        self.route = Route::Displays;
        self.displays.reload();
      }
      HomeDestination::KeyboardAndShortcuts => self.open_settings(Page::Keybindings),
      #[cfg(feature = "hardware")]
      HomeDestination::MouseTouchpad => self.open_settings(Page::MouseTouchpad),
      #[cfg(feature = "locale")]
      HomeDestination::LanguageRegion => self.open_settings(Page::LocaleRegion),
      #[cfg(feature = "locale")]
      HomeDestination::DateTime => self.open_settings(Page::DateTime),
      #[cfg(feature = "apps")]
      HomeDestination::DefaultApplications => self.open_settings(Page::DefaultApps),
      #[cfg(feature = "power")]
      HomeDestination::Power => {
        self.route = Route::Power;
        self.power.reload();
      }
      #[cfg(feature = "session")]
      HomeDestination::Session => {
        self.route = Route::Session;
        self.session.reload();
      }
      HomeDestination::Settings => self.route = Route::AppSettings,
      #[cfg(feature = "hardware")]
      HomeDestination::HardwareOverview => self.route = Route::Hardware,
      #[cfg(feature = "network")]
      HomeDestination::Network => {
        self.route = Route::Network;
        self.network.reload();
      }
      #[cfg(feature = "bluetooth")]
      HomeDestination::Bluetooth => {
        self.route = Route::Bluetooth;
        self.bluetooth.reload();
      }
      #[cfg(feature = "audio")]
      HomeDestination::Audio => {
        self.route = Route::Audio;
        self.audio.reload();
      }
      #[cfg(feature = "storage")]
      HomeDestination::Storage => {
        self.route = Route::Storage;
        self.storage.reload();
      }
      #[cfg(feature = "packages")]
      HomeDestination::Packages => {
        self.route = Route::Packages;
        self.packages.reload();
      }
      #[cfg(feature = "services")]
      HomeDestination::Services => {
        self.route = Route::Services;
        self.services.reload();
      }
      #[cfg(feature = "boot")]
      HomeDestination::Boot => {
        self.route = Route::Boot;
        self.boot.reload();
      }
      #[cfg(feature = "system")]
      HomeDestination::Firewall => self.open_settings(Page::Firewall),
      #[cfg(feature = "system")]
      HomeDestination::Hostname => self.open_settings(Page::Hostname),
      #[cfg(feature = "system")]
      HomeDestination::UsersGroups => self.open_settings(Page::UsersGroups),
      #[cfg(feature = "system")]
      HomeDestination::Notifications => self.open_settings(Page::Notifications),
      #[cfg(feature = "apps")]
      HomeDestination::Projects => self.open_settings(Page::Projects),
      #[cfg(feature = "apps")]
      HomeDestination::Snippets => self.open_settings(Page::Snippets),
      #[cfg(feature = "apps")]
      HomeDestination::SshKeys => self.open_settings(Page::SshKeys),
      #[cfg(feature = "apps")]
      HomeDestination::GpgKeys => self.open_settings(Page::GpgKeys),
      #[cfg(feature = "dev-services")]
      HomeDestination::DevServices => {
        self.route = Route::DevServices;
        self.dev_services.reload();
      }
      #[cfg(feature = "diagnostics")]
      HomeDestination::Diagnostics => {
        self.route = Route::Diagnostics;
        self.diagnostics.reload();
      }
      #[cfg(feature = "about")]
      HomeDestination::About => self.route = Route::About,
    }
  }

  /// Opens one page of the Appearance tree from the Home, reloading its state.
  #[cfg(feature = "appearance")]
  pub(super) fn open_appearance_surface(&mut self, page: AppearancePage) {
    self.appearance.open(page);
    self.route = Route::Appearance;
  }

  pub fn open_settings(&mut self, page: Page) {
    self.settings.navigation = argvus_control_center_settings::navigation::Navigation::new(page);
    self.settings.select_current();
    self.settings.enter_page(page);
    self.route = Route::Settings;
  }
}
