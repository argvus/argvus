//! Application state and main-flow coordination.
//!
//! `Route` represents the currently displayed page; `InitialRoute` preserves the
//! intent of a direct route until application initialization completes. This
//! distinction lets the CLI, global search, and shortcuts share the same
//! navigation authority without duplicating screens or creating parallel paths.
use crate::config_app::ConfigApp;
use crate::home::{
  HomeAvailability, HomeDestination, HomeRow, catalog::home_rows as home_catalog_rows,
};
#[cfg(feature = "about")]
use argvus_control_center_about::{App as AboutState, Tab};
#[cfg(feature = "appearance")]
use argvus_control_center_appearance::{AppearanceApp, AppearancePage};
#[cfg(feature = "audio")]
use argvus_control_center_audio::{AudioApp, AudioPage};
#[cfg(feature = "bluetooth")]
use argvus_control_center_bluetooth::{BluetoothApp, BluetoothPage};
#[cfg(feature = "boot")]
use argvus_control_center_boot::{BootApp, BootPage};
use argvus_control_center_core::{
  capabilities::Capabilities,
  jobs::JobManager,
  search::{SearchEntry, SearchRegistry},
};
#[cfg(feature = "dev-services")]
use argvus_control_center_dev_services::DevServicesApp;
#[cfg(feature = "diagnostics")]
use argvus_control_center_diagnostics::{DiagnosticPage, DiagnosticsApp};
#[cfg(feature = "displays")]
use argvus_control_center_displays::{DisplayPage, DisplaysApp};
#[cfg(feature = "hardware")]
use argvus_control_center_hardware::{HardwareApp, HardwarePage};
#[cfg(feature = "network")]
use argvus_control_center_network::{NetworkApp, NetworkPage};
#[cfg(feature = "packages")]
use argvus_control_center_packages::{PackageSource, PackageTask, PackagesApp, PackagesPage};
#[cfg(feature = "power")]
use argvus_control_center_power::PowerApp;
#[cfg(feature = "services")]
use argvus_control_center_services::{ServicePage, ServicesApp};
#[cfg(feature = "session")]
use argvus_control_center_session::{SessionApp, SessionPage};
use argvus_control_center_settings::{App as SettingsState, Page};
#[cfg(feature = "snapshot")]
use argvus_control_center_snapshot::SnapshotApp;
#[cfg(feature = "storage")]
use argvus_control_center_storage::{StorageApp, StoragePage};
use argvus_i18n::{Lang, tr};
use argvus_theme::Theme;
use std::time::{Duration, Instant};

mod search_catalog;

use search_catalog::build_search_registry;

mod global_search;
mod help;
mod home_navigation;
mod page_openers;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
  Home,
  Welcome,
  /// Settings of the Control Center itself: Language and Style.
  AppSettings,
  Settings,
  Config,
  #[cfg(feature = "about")]
  About,
  #[cfg(feature = "hardware")]
  Hardware,
  #[cfg(feature = "services")]
  Services,
  #[cfg(feature = "dev-services")]
  DevServices,
  #[cfg(feature = "network")]
  Network,
  #[cfg(feature = "audio")]
  Audio,
  #[cfg(feature = "bluetooth")]
  Bluetooth,
  #[cfg(feature = "boot")]
  Boot,
  #[cfg(feature = "packages")]
  Packages,
  #[cfg(feature = "storage")]
  Storage,
  #[cfg(feature = "snapshot")]
  Snapshot,
  #[cfg(feature = "diagnostics")]
  Diagnostics,
  #[cfg(feature = "power")]
  Power,
  #[cfg(feature = "session")]
  Session,
  #[cfg(feature = "displays")]
  Displays,
  #[cfg(feature = "appearance")]
  Appearance,
  /// Shortcuts, Window rules, Window Spaces, Animations, Blur, Borders and
  /// the readonly Hyprland version — always present, even though most of
  /// its rows depend on the `apps`/`appearance` features individually.
  Hyprland,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitialRoute {
  Home,
  Welcome,
  Config,
  Settings(Page),
  #[cfg(feature = "apps")]
  Apps,
  #[cfg(feature = "fonts")]
  Fonts,
  #[cfg(feature = "locale")]
  LocaleRegion,
  #[cfg(feature = "language")]
  Language,
  #[cfg(feature = "system")]
  System,
  #[cfg(feature = "about")]
  About(Tab),
  #[cfg(feature = "hardware")]
  Hardware(HardwarePage),
  #[cfg(feature = "services")]
  Services(ServicePage),
  #[cfg(feature = "dev-services")]
  DevServices,
  #[cfg(feature = "network")]
  Network(NetworkPage),
  #[cfg(feature = "audio")]
  Audio(AudioPage),
  #[cfg(feature = "bluetooth")]
  Bluetooth(BluetoothPage),
  #[cfg(feature = "boot")]
  Boot(BootPage),
  #[cfg(feature = "packages")]
  Packages(PackagesPage),
  #[cfg(feature = "storage")]
  Storage(StoragePage),
  #[cfg(feature = "snapshot")]
  Snapshot,
  #[cfg(feature = "diagnostics")]
  Diagnostics(DiagnosticPage),
  #[cfg(feature = "power")]
  Power,
  #[cfg(feature = "session")]
  Session(SessionPage),
  #[cfg(feature = "displays")]
  Displays(DisplayPage),
  #[cfg(feature = "appearance")]
  Appearance(AppearancePage),
}

impl InitialRoute {
  /// The page the Control Center opens on for this route.
  fn route(self) -> Route {
    match self {
      InitialRoute::Home => Route::Home,
      InitialRoute::Welcome => Route::Welcome,
      InitialRoute::Config => Route::Config,
      InitialRoute::Settings(_) => Route::Settings,
      #[cfg(feature = "apps")]
      InitialRoute::Apps => Route::Settings,
      #[cfg(feature = "fonts")]
      InitialRoute::Fonts => Route::Settings,
      #[cfg(feature = "locale")]
      InitialRoute::LocaleRegion => Route::Settings,
      #[cfg(feature = "language")]
      InitialRoute::Language => Route::Settings,
      #[cfg(feature = "system")]
      InitialRoute::System => Route::Settings,
      #[cfg(feature = "about")]
      InitialRoute::About(_) => Route::About,
      #[cfg(feature = "hardware")]
      InitialRoute::Hardware(_) => Route::Hardware,
      #[cfg(feature = "services")]
      InitialRoute::Services(_) => Route::Services,
      #[cfg(feature = "dev-services")]
      InitialRoute::DevServices => Route::DevServices,
      #[cfg(feature = "network")]
      InitialRoute::Network(_) => Route::Network,
      #[cfg(feature = "audio")]
      InitialRoute::Audio(_) => Route::Audio,
      #[cfg(feature = "bluetooth")]
      InitialRoute::Bluetooth(_) => Route::Bluetooth,
      #[cfg(feature = "boot")]
      InitialRoute::Boot(_) => Route::Boot,
      #[cfg(feature = "packages")]
      InitialRoute::Packages(_) => Route::Packages,
      #[cfg(feature = "storage")]
      InitialRoute::Storage(_) => Route::Storage,
      #[cfg(feature = "snapshot")]
      InitialRoute::Snapshot => Route::Snapshot,
      #[cfg(feature = "diagnostics")]
      InitialRoute::Diagnostics(_) => Route::Diagnostics,
      #[cfg(feature = "power")]
      InitialRoute::Power => Route::Power,
      #[cfg(feature = "session")]
      InitialRoute::Session(_) => Route::Session,
      #[cfg(feature = "displays")]
      InitialRoute::Displays(_) => Route::Displays,
      #[cfg(feature = "appearance")]
      InitialRoute::Appearance(_) => Route::Appearance,
    }
  }

  /// The settings page a settings route opens, or the settings main page for
  /// any other route.
  fn settings_page(self) -> Page {
    match self {
      InitialRoute::Settings(page) => page,
      #[cfg(feature = "apps")]
      InitialRoute::Apps => Page::DefaultApps,
      #[cfg(feature = "fonts")]
      InitialRoute::Fonts => Page::Fonts,
      #[cfg(feature = "locale")]
      InitialRoute::LocaleRegion => Page::LocaleRegion,
      #[cfg(feature = "language")]
      InitialRoute::Language => Page::Language,
      #[cfg(feature = "system")]
      InitialRoute::System => Page::Hostname,
      _ => Page::Main,
    }
  }
}

pub struct App {
  pub welcome: crate::welcome::Welcome,
  /// Language detected by the shared i18n infrastructure.
  pub lang: Lang,
  /// Semantic theme used by every page during this TUI cycle.
  pub theme: Theme,
  /// Currently active page; rendering and events read this field.
  pub route: Route,
  /// Route restored when leaving a page the Hyprland category opened
  /// (Settings or Appearance); defaults back to `Route::Home` once consumed.
  pub return_route: Route,
  /// Cursor of the Control Center Settings hub.
  pub settings_hub: crate::settings_hub::SettingsHub,
  /// State of the "Hyprland" category page.
  pub hyprland: crate::hyprland::HyprlandState,
  /// Home selection index, kept separate from the route to support returning.
  pub home_selected: usize,
  /// First visible visual row in the home grid; rendering keeps it in range.
  pub home_scroll: usize,
  pub settings: SettingsState,
  pub config: ConfigApp,
  #[cfg(feature = "about")]
  pub about: AboutState,
  /// Requests contextual help without replacing the current route.
  pub help: bool,
  /// Signal consumed by the main loop to restore the terminal and exit.
  pub quit: bool,
  /// Current dimensions used by responsive layouts.
  pub width: u16,
  /// Current height used by responsive layouts.
  pub height: u16,
  #[cfg(feature = "hardware")]
  pub hardware: HardwareApp,
  #[cfg(feature = "services")]
  pub services: ServicesApp,
  #[cfg(feature = "dev-services")]
  pub dev_services: DevServicesApp,
  #[cfg(feature = "network")]
  pub network: NetworkApp,
  #[cfg(feature = "audio")]
  pub audio: AudioApp,
  #[cfg(feature = "bluetooth")]
  pub bluetooth: BluetoothApp,
  #[cfg(feature = "boot")]
  pub boot: BootApp,
  #[cfg(feature = "packages")]
  pub packages: PackagesApp,
  #[cfg(feature = "storage")]
  pub storage: StorageApp,
  #[cfg(feature = "snapshot")]
  pub snapshot: SnapshotApp,
  #[cfg(feature = "diagnostics")]
  pub diagnostics: DiagnosticsApp,
  #[cfg(feature = "power")]
  pub power: PowerApp,
  #[cfg(feature = "session")]
  pub session: SessionApp,
  #[cfg(feature = "displays")]
  pub displays: DisplaysApp,
  #[cfg(feature = "appearance")]
  pub appearance: AppearanceApp,
  /// Capabilities detected once and shared by optional domains.
  pub capabilities: Capabilities,
  /// Single registry used by Home global search.
  pub search_registry: SearchRegistry,
  /// Text entered in the global search field.
  pub search_query: String,
  /// Indicates whether keyboard events should go to the search editor.
  pub search_active: bool,
  /// Selected result index, independent of the Home index.
  pub search_selected: usize,
  /// Registered IDs used to open the real route after selection.
  pub search_result_ids: Vec<String>,
  /// Controls cursor blinking without changing entered text.
  pub search_cursor_visible: bool,
  search_cursor_last_blink: Instant,
  pub jobs: JobManager,
  /// The palette identity `theme` was loaded from, and when it was last read.
  theme_stamp: argvus_theme::loader::ThemeStamp,
  theme_checked: Instant,
}

impl App {
  pub fn new(initial: InitialRoute) -> Self {
    let lang = Lang::detect();
    let theme = Theme::load();
    let route = initial.route();
    let page = initial.settings_page();
    #[cfg(feature = "about")]
    let tab = match initial {
      InitialRoute::About(tab) => tab,
      _ => Tab::System,
    };
    let capabilities = Capabilities::detect();
    let hyprland_version = capabilities
      .has_hyprctl
      .then(crate::hyprland::read_version)
      .flatten()
      .unwrap_or_default();
    #[cfg(feature = "hardware")]
    let mut hardware = HardwareApp::new(lang, theme.clone(), capabilities.clone());
    #[cfg(feature = "services")]
    let mut services = ServicesApp::new(lang, theme.clone());
    #[cfg(feature = "dev-services")]
    let mut dev_services = DevServicesApp::new(lang, theme.clone(), capabilities.clone());
    #[cfg(feature = "network")]
    let mut network = NetworkApp::new(lang, theme.clone(), capabilities.clone());
    #[cfg(feature = "audio")]
    let mut audio = AudioApp::new(lang, theme.clone(), capabilities.clone());
    #[cfg(feature = "bluetooth")]
    let mut bluetooth = BluetoothApp::new(lang, theme.clone(), capabilities.clone());
    #[cfg(feature = "boot")]
    let mut boot = BootApp::new(lang, theme.clone(), capabilities.clone());
    #[cfg(feature = "packages")]
    let mut packages = PackagesApp::new(lang, theme.clone(), capabilities.clone());
    #[cfg(feature = "snapshot")]
    let mut snapshot = SnapshotApp::new(lang, theme.clone());
    #[cfg(feature = "snapshot")]
    if matches!(initial, InitialRoute::Snapshot) {
      snapshot.reload();
    }
    #[cfg(all(feature = "storage", feature = "diagnostics"))]
    let storage_cache = argvus_control_center_storage::backend::StorageCache::default();
    #[cfg(all(feature = "storage", feature = "diagnostics"))]
    let mut storage = StorageApp::new_with_cache(
      lang,
      theme.clone(),
      capabilities.clone(),
      storage_cache.clone(),
    );
    #[cfg(all(feature = "storage", not(feature = "diagnostics")))]
    let mut storage = StorageApp::new(lang, theme.clone(), capabilities.clone());
    #[cfg(all(feature = "storage", feature = "diagnostics"))]
    let mut diagnostics =
      DiagnosticsApp::new_with_cache(lang, theme.clone(), capabilities.clone(), storage_cache);
    #[cfg(all(feature = "diagnostics", not(feature = "storage")))]
    let mut diagnostics = DiagnosticsApp::new(lang, theme.clone(), capabilities.clone());
    #[cfg(feature = "power")]
    let power = PowerApp::new(lang, theme.clone());
    #[cfg(feature = "session")]
    let mut session = SessionApp::new(lang, theme.clone());
    #[cfg(feature = "displays")]
    let mut displays = DisplaysApp::new(lang, theme.clone());
    #[cfg(feature = "appearance")]
    let mut appearance = AppearanceApp::new(lang, theme.clone());
    #[cfg(feature = "appearance")]
    {
      appearance.fonts_available = cfg!(feature = "fonts");
    }
    #[cfg(feature = "hardware")]
    if let InitialRoute::Hardware(page) = initial {
      hardware.page = page;
    }
    #[cfg(feature = "services")]
    if let InitialRoute::Services(page) = initial {
      services.page = page;
      services.reload();
    }
    #[cfg(feature = "dev-services")]
    if matches!(initial, InitialRoute::DevServices) {
      dev_services.reload();
    }
    #[cfg(feature = "network")]
    if let InitialRoute::Network(page) = initial {
      network.page = page;
      network.reload();
    }
    #[cfg(feature = "audio")]
    if let InitialRoute::Audio(page) = initial {
      audio.page = page;
      audio.reload();
    }
    #[cfg(feature = "bluetooth")]
    if let InitialRoute::Bluetooth(page) = initial {
      bluetooth.page = page;
      bluetooth.reload();
    }
    #[cfg(feature = "boot")]
    if let InitialRoute::Boot(page) = initial {
      boot.page = page;
      boot.reload();
    }
    #[cfg(feature = "packages")]
    if let InitialRoute::Packages(page) = initial {
      packages.open(page);
    }
    #[cfg(feature = "storage")]
    if let InitialRoute::Storage(page) = initial {
      storage.page = page;
      storage.reload();
    }
    #[cfg(feature = "diagnostics")]
    if let InitialRoute::Diagnostics(page) = initial {
      diagnostics.page = page;
      diagnostics.reload();
    }
    #[cfg(feature = "session")]
    if let InitialRoute::Session(page) = initial {
      session.page = page;
      session.reload();
    }
    #[cfg(feature = "displays")]
    if let InitialRoute::Displays(page) = initial {
      displays.page = page;
      displays.reload();
    }
    #[cfg(feature = "appearance")]
    if let InitialRoute::Appearance(page) = initial {
      appearance.open(page);
    }
    let search_registry = build_search_registry(lang);
    Self {
      lang,
      theme: theme.clone(),
      route,
      return_route: Route::Home,
      settings_hub: crate::settings_hub::SettingsHub::default(),
      hyprland: crate::hyprland::HyprlandState::new(hyprland_version),
      home_selected: 0,
      home_scroll: 0,
      settings: SettingsState::with_context(page, lang, theme.clone()),
      config: ConfigApp::new(lang, theme.clone()),
      #[cfg(feature = "about")]
      about: AboutState::with_context(tab, lang, theme),
      help: false,
      quit: false,
      welcome: crate::welcome::Welcome::default(),
      width: 80,
      height: 24,
      #[cfg(feature = "hardware")]
      hardware,
      #[cfg(feature = "services")]
      services,
      #[cfg(feature = "dev-services")]
      dev_services,
      #[cfg(feature = "network")]
      network,
      #[cfg(feature = "audio")]
      audio,
      #[cfg(feature = "bluetooth")]
      bluetooth,
      #[cfg(feature = "boot")]
      boot,
      #[cfg(feature = "packages")]
      packages,
      #[cfg(feature = "storage")]
      storage,
      #[cfg(feature = "snapshot")]
      snapshot,
      #[cfg(feature = "diagnostics")]
      diagnostics,
      #[cfg(feature = "power")]
      power,
      #[cfg(feature = "session")]
      session,
      #[cfg(feature = "displays")]
      displays,
      #[cfg(feature = "appearance")]
      appearance,
      capabilities,
      search_registry,
      search_query: String::new(),
      search_active: false,
      search_selected: 0,
      search_result_ids: Vec::new(),
      search_cursor_visible: true,
      search_cursor_last_blink: Instant::now(),
      theme_stamp: Theme::active_stamp(),
      theme_checked: Instant::now(),
      jobs: JobManager::default(),
    }
  }

  /// Reloads the semantic theme and propagates it to every page.
  ///
  /// Appearance changes are written to the generated theme files by external
  /// scripts, so the process must re-read them instead of reusing the palette
  /// captured at startup.
  pub fn refresh_theme(&mut self) {
    self.theme_stamp = Theme::active_stamp();
    let theme = Theme::load();
    self.theme = theme.clone();
    self.settings.set_theme(&theme);
    self.config.set_theme(&theme);
    #[cfg(feature = "about")]
    self.about.set_theme(&theme);
    #[cfg(feature = "hardware")]
    self.hardware.set_theme(&theme);
    #[cfg(feature = "services")]
    self.services.set_theme(&theme);
    #[cfg(feature = "dev-services")]
    self.dev_services.set_theme(&theme);
    #[cfg(feature = "network")]
    self.network.set_theme(&theme);
    #[cfg(feature = "audio")]
    self.audio.set_theme(&theme);
    #[cfg(feature = "bluetooth")]
    self.bluetooth.set_theme(&theme);
    #[cfg(feature = "boot")]
    self.boot.set_theme(&theme);
    #[cfg(feature = "packages")]
    self.packages.set_theme(&theme);
    #[cfg(feature = "storage")]
    self.storage.set_theme(&theme);
    #[cfg(feature = "snapshot")]
    self.snapshot.set_theme(&theme);
    #[cfg(feature = "diagnostics")]
    self.diagnostics.set_theme(&theme);
    #[cfg(feature = "power")]
    self.power.set_theme(&theme);
    #[cfg(feature = "session")]
    self.session.set_theme(&theme);
    #[cfg(feature = "displays")]
    self.displays.set_theme(&theme);
    #[cfg(feature = "appearance")]
    self.appearance.set_theme(&theme);
  }

  /// Reloads the palette when the theme changed outside this window (a theme
  /// switch from the keyboard, the Control Panel or the accent picker). The
  /// disk is read at most once a second.
  pub fn poll_theme_change(&mut self) -> bool {
    if self.theme_checked.elapsed() < Duration::from_secs(1) {
      return false;
    }
    self.theme_checked = Instant::now();
    if Theme::active_stamp() == self.theme_stamp {
      return false;
    }
    self.refresh_theme();
    true
  }

  /// Whether the active page is typing into a text field, a search or a list
  /// filter. While it is, `q` and `?` are text and must not quit or open the
  /// help (decision D10 of `docs/ux-audit.md`); `Ctrl+C` still quits.
  pub fn captures_text(&self) -> bool {
    match self.route {
      Route::Home => self.search_active,
      #[cfg(feature = "snapshot")]
      Route::Snapshot => self.snapshot.captures_text(),
      Route::Settings => self.settings.captures_text(),
      #[cfg(feature = "network")]
      Route::Network => self.network.captures_text(),
      #[cfg(feature = "packages")]
      Route::Packages => self.packages.captures_text(),
      #[cfg(feature = "services")]
      Route::Services => self.services.captures_text(),
      #[cfg(feature = "dev-services")]
      Route::DevServices => self.dev_services.captures_text(),
      #[cfg(feature = "audio")]
      Route::Audio => self.audio.captures_text(),
      #[cfg(feature = "boot")]
      Route::Boot => self.boot.captures_text(),
      #[cfg(feature = "displays")]
      Route::Displays => self.displays.captures_text(),
      #[cfg(feature = "appearance")]
      Route::Appearance => self.appearance.captures_text(),
      // The remaining pages (Config, About, Hardware, Bluetooth, Storage,
      // Diagnostics, Power, Session) have no text fields; listing them would
      // need one cfg arm per optional feature.
      #[allow(unreachable_patterns)]
      _ => false,
    }
  }

  pub fn back(&mut self) {
    match self.route {
      Route::Welcome => self.route = Route::Home,
      Route::Home => {}
      Route::AppSettings => self.route = Route::Home,
      Route::Config => self.route = Route::AppSettings,
      #[cfg(feature = "about")]
      Route::About => self.route = Route::Home,
      #[cfg(feature = "hardware")]
      Route::Hardware => {
        if self.hardware.handle(crossterm::event::KeyCode::Esc) {
          self.route = Route::Home;
        }
      }
      #[cfg(feature = "services")]
      Route::Services => {
        if self.services.handle(crossterm::event::KeyCode::Esc) {
          self.route = Route::Home;
        }
      }
      #[cfg(feature = "dev-services")]
      Route::DevServices => {
        if self.dev_services.handle(crossterm::event::KeyCode::Esc) {
          self.route = Route::Home;
        }
      }
      #[cfg(feature = "network")]
      Route::Network => {
        if self.network.handle(crossterm::event::KeyCode::Esc) {
          self.route = Route::Home;
        }
      }
      #[cfg(feature = "audio")]
      Route::Audio => {
        if self.audio.handle(crossterm::event::KeyCode::Esc) {
          self.route = Route::Home;
        }
      }
      #[cfg(feature = "bluetooth")]
      Route::Bluetooth => {
        if self.bluetooth.handle(crossterm::event::KeyCode::Esc) {
          self.route = Route::Home;
        }
      }
      #[cfg(feature = "boot")]
      Route::Boot => {
        if self.boot.handle(crossterm::event::KeyCode::Esc) {
          self.route = Route::Home;
        }
      }
      #[cfg(feature = "packages")]
      Route::Packages => {
        if self.packages.handle(crossterm::event::KeyCode::Esc) {
          self.route = Route::Home;
        }
      }
      #[cfg(feature = "snapshot")]
      Route::Snapshot => {
        if self.snapshot.handle_key(crossterm::event::KeyEvent::from(
          crossterm::event::KeyCode::Esc,
        )) {
          self.route = Route::Home;
        }
      }
      #[cfg(feature = "storage")]
      Route::Storage => {
        if self.storage.handle(crossterm::event::KeyCode::Esc) {
          self.route = Route::Home;
        }
      }
      #[cfg(feature = "diagnostics")]
      Route::Diagnostics => {
        if self.diagnostics.handle(crossterm::event::KeyCode::Esc) {
          self.route = Route::Home;
        }
      }
      #[cfg(feature = "power")]
      Route::Power => {
        if self.power.handle(crossterm::event::KeyCode::Esc) {
          self.route = Route::Home;
        }
      }
      #[cfg(feature = "session")]
      Route::Session => {
        if self.session.handle(crossterm::event::KeyCode::Esc) {
          self.route = Route::Home;
        }
      }
      #[cfg(feature = "displays")]
      Route::Displays => {
        if self.displays.handle(crossterm::event::KeyCode::Esc) {
          self.route = Route::Home;
        }
      }
      #[cfg(feature = "appearance")]
      Route::Appearance => {
        if self.appearance.handle(crossterm::event::KeyCode::Esc) {
          self.route = std::mem::replace(&mut self.return_route, Route::Home);
        }
      }
      Route::Settings => {
        self.settings.back();
        if self.settings.page() == Page::Main {
          self.route = std::mem::replace(&mut self.return_route, Route::Home);
        }
      }
      Route::Hyprland => {
        if crate::hyprland::handle(self, crossterm::event::KeyCode::Esc) {
          self.route = Route::Home;
        }
      }
    }
  }

  pub fn resize(&mut self, width: u16, height: u16) {
    self.width = width;
    self.height = height;
    self.settings.resize(width, height);
    #[cfg(feature = "about")]
    self.about.resize(width, height);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  /// Every search result shows its translated title. A raw catalog key here
  /// means an entry points at a key that the catalogs do not define.
  fn search_entries_show_translated_titles() {
    let app = App::new(InitialRoute::Home);
    assert!(!app.search_registry.entries().is_empty());
    for entry in app.search_registry.entries() {
      assert!(!entry.title.is_empty(), "{}", entry.id);
      assert!(!entry.title.starts_with("control_center."), "{}", entry.id);
    }
  }

  #[test]
  #[cfg(feature = "displays")]
  fn night_light_search_and_cli_open_the_same_page() {
    let mut app = App::new(InitialRoute::Home);
    assert!(
      app
        .search_registry
        .entries()
        .iter()
        .any(|entry| entry.id == "displays.night-light")
    );
    app.open_search_route("displays/night-light");
    assert_eq!(app.route, Route::Displays);
    assert_eq!(app.displays.page, DisplayPage::NightLight);
    let cli = App::new(InitialRoute::Displays(DisplayPage::NightLight));
    assert_eq!(cli.route, Route::Displays);
    assert_eq!(cli.displays.page, DisplayPage::NightLight);
  }

  #[test]
  fn home_navigation_is_bounded() {
    let mut app = App::new(InitialRoute::Home);
    app.move_home(-1);
    assert_eq!(app.home_selected, 0);
    app.move_home(1_000);
    assert_eq!(app.home_selected, app.home_item_count().saturating_sub(1));
  }

  #[test]
  fn home_navigation_never_selects_a_header() {
    let mut app = App::new(InitialRoute::Home);
    let rows = app.home_rows();
    for index in 0..app.home_item_count() {
      app.home_selected = index;
      assert!(matches!(
        rows
          .iter()
          .filter(|row| !matches!(row, HomeRow::Header(_)))
          .nth(app.home_selected),
        Some(HomeRow::Item { .. } | HomeRow::Standalone { .. })
      ));
    }
  }

  #[test]
  fn home_grid_columns_follow_responsive_breakpoints() {
    assert_eq!(App::home_grid_columns(60), 1);
    assert_eq!(App::home_grid_columns(80), 2);
    assert_eq!(App::home_grid_columns(120), 3);
    assert_eq!(App::home_grid_columns(160), 4);
  }

  #[test]
  fn home_category_navigation_moves_between_category_items() {
    // Categories: Desktop Shell (first card) › Language & Region › Appearance
    // › ... The Desktop Shell card lists its items first, so the first item of
    // Language & Region follows them and depends on how many items it has.
    let mut app = App::new(InitialRoute::Home);
    let desktop_shell_items = app.home_category_items()[0].len();
    app.move_home_category(1);
    assert_eq!(app.home_selected, desktop_shell_items);
    app.move_home(1);
    assert_eq!(app.home_selected, desktop_shell_items + 1);
    app.move_home_category(-1);
    assert_eq!(app.home_selected, 0);
  }

  #[cfg(feature = "hardware")]
  #[test]
  /// Mouse & Touchpad belongs to the Desktop & Input card, with the keyboard.
  fn mouse_touchpad_is_in_the_desktop_input_card() {
    let app = App::new(InitialRoute::Home);
    let rows = app.home_rows();
    let mouse = rows
      .iter()
      .position(|row| {
        matches!(
          row,
          HomeRow::Item {
            destination: HomeDestination::MouseTouchpad,
            ..
          }
        )
      })
      .expect("mouse and touchpad item");
    let heading = rows[..mouse]
      .iter()
      .rev()
      .find_map(|row| match row {
        HomeRow::Header(category) => Some(category.title),
        HomeRow::Item { .. } | HomeRow::Standalone { .. } => None,
      })
      .expect("card heading");
    assert_eq!(
      heading,
      argvus_control_center_core::navigation::HomeCardId::DesktopInput.label(app.lang)
    );
  }

  #[cfg(feature = "about")]
  #[test]
  fn entering_and_leaving_about_preserves_tab() {
    let mut app = App::new(InitialRoute::About(Tab::Credits));
    app.back();
    assert_eq!(app.route, Route::Home);
    app.home_selected = app
      .home_destinations()
      .iter()
      .position(|destination| *destination == HomeDestination::About)
      .expect("About row must be listed");
    app.open_home();
    assert_eq!(app.route, Route::About);
    assert_eq!(app.about.active_tab, Tab::Credits);
  }

  #[test]
  /// Settings is listed below the cards and opens the Settings hub, where Esc returns to the Home.
  fn home_lists_and_opens_the_settings_hub() {
    let mut app = App::new(InitialRoute::Home);
    app.home_selected = app
      .home_destinations()
      .iter()
      .position(|destination| *destination == HomeDestination::Settings)
      .expect("Settings row must be listed");
    assert_eq!(app.home_selected, app.home_item_count() - 1);
    app.open_home();
    assert_eq!(app.route, Route::AppSettings);
    app.back();
    assert_eq!(app.route, Route::Home);
  }

  #[test]
  fn welcome_is_available_from_search_under_tools() {
    let app = App::new(InitialRoute::Home);
    let entry = app
      .search_registry
      .entries()
      .iter()
      .find(|entry| entry.id == "welcome")
      .expect("Welcome search entry");
    assert_eq!(entry.category, "card_tools");
    assert_eq!(entry.route, "tools/welcome");
  }

  #[test]
  /// Style is reached from the hub and Esc returns to the hub, not to the Home.
  fn style_returns_to_the_settings_hub() {
    let mut app = App::new(InitialRoute::Home);
    app.route = Route::Config;
    app.back();
    assert_eq!(app.route, Route::AppSettings);
  }
}

#[cfg(all(test, feature = "snapshot"))]
mod snapshot_tests {
  use super::*;

  #[test]
  fn home_search_and_cli_open_the_same_snapshot_page_and_start_loading() {
    let mut app = App::new(InitialRoute::Home);
    app.open_destination(HomeDestination::Snapshot);
    assert_eq!(app.route, Route::Snapshot);
    assert!(app.snapshot.busy());
    let mut app = App::new(InitialRoute::Home);
    app.open_search_route("snapshot/home");
    assert_eq!(app.route, Route::Snapshot);
    assert!(app.snapshot.busy());
    let app = App::new(InitialRoute::Snapshot);
    assert_eq!(app.route, Route::Snapshot);
    assert!(app.snapshot.busy());
  }
}
