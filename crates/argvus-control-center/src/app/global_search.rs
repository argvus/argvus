//! Control Center search: typing, moving through the results and opening the route of the chosen entry.

use super::*;

impl App {
  pub fn has_global_search(&self) -> bool {
    match self.route {
      Route::Welcome => false,
      #[cfg(feature = "about")]
      Route::About => false,
      _ => true,
    }
  }

  pub fn can_focus_search(&self) -> bool {
    #[cfg(feature = "packages")]
    if self.route == Route::Packages
      && self.packages.supports_filter()
      && !self.packages.filter_focused()
    {
      return self.can_focus_search_from(true);
    }
    self.can_focus_search_from(false)
  }

  pub fn can_focus_search_from(&self, editing_filter: bool) -> bool {
    let modal = match self.route {
      #[cfg(feature = "services")]
      Route::Services => self.services.blocks_global_search(),
      #[cfg(feature = "appearance")]
      Route::Appearance => self.appearance.blocks_global_search(),
      #[cfg(feature = "displays")]
      Route::Displays => self.displays.blocks_global_search(),
      #[cfg(feature = "network")]
      Route::Network => self.network.blocks_global_search(),
      #[cfg(feature = "bluetooth")]
      Route::Bluetooth => self.bluetooth.blocks_global_search(),
      #[cfg(feature = "snapshot")]
      Route::Snapshot => self.snapshot.blocks_global_search(),
      #[cfg(feature = "packages")]
      Route::Packages => self.packages.blocks_global_search(),
      #[cfg(feature = "power")]
      Route::Power => self.power.blocks_global_search(),
      _ => false,
    };
    self.has_global_search()
      && !modal
      && !self.help
      && (!self.captures_text() || editing_filter)
      && !(self.route == Route::Settings
        && (self.settings.admin.busy()
          || self.settings.confirm.is_some()
          || self.settings.task_open
          || self.settings.error_modal.is_some()
          || self.settings.has_keybinding_conflict()))
  }

  pub fn begin_global_search(&mut self) {
    self.search_notice = None;
    self.refresh_search_catalog();
    #[cfg(feature = "system")]
    self.search_accounts.load(false);
    self.search_active = true;
    self.search_cursor_visible = true;
    self.search_cursor_last_blink = Instant::now();
    self.search_query.clear();
    self.search_selected = 0;
    self.search_result_ids.clear();
  }

  fn index_page(&mut self, route: &str, (path, words): (String, Vec<String>)) {
    if let Some(entry) = self.search_registry.entry_for_route_mut(route) {
      if let Some((parent, _)) = path.rsplit_once(" > ") {
        entry.category = parent.to_string();
      }
      entry.keywords.push(path);
      entry.keywords.extend(words);
    }
  }

  fn refresh_search_catalog(&mut self) {
    self.search_registry = build_search_registry(self.lang);
    #[cfg(feature = "audio")]
    for &(route, page) in search_catalog::AUDIO_PAGES {
      let content = self.audio.search_content(page);
      self.index_page(route, content);
    }
    #[cfg(feature = "hardware")]
    for &(route, page) in search_catalog::HARDWARE_PAGES {
      let content = self.hardware.search_content(page);
      self.index_page(route, content);
    }
    #[cfg(feature = "boot")]
    for &(route, page) in search_catalog::BOOT_PAGES {
      let content = self.boot.search_content(page);
      self.index_page(route, content);
    }
    #[cfg(feature = "network")]
    for &(route, page) in search_catalog::NETWORK_PAGES {
      let content = self.network.search_content(page);
      self.index_page(route, content);
    }
    #[cfg(feature = "storage")]
    for &(route, page) in search_catalog::STORAGE_PAGES {
      let content = self.storage.search_content(page);
      self.index_page(route, content);
    }
    #[cfg(feature = "displays")]
    for &(route, page) in search_catalog::DISPLAYS_PAGES {
      let content = self.displays.search_content(page);
      self.index_page(route, content);
    }
    #[cfg(feature = "packages")]
    for &(route, page) in search_catalog::PACKAGES_PAGES {
      let content = self.packages.search_content(page);
      self.index_page(route, content);
    }
    #[cfg(feature = "diagnostics")]
    for &(route, page) in search_catalog::DIAGNOSTICS_PAGES {
      let content = self.diagnostics.search_content(page);
      self.index_page(route, content);
    }
    #[cfg(feature = "session")]
    for &(route, page) in search_catalog::SESSION_PAGES {
      let content = self.session.search_content(page);
      self.index_page(route, content);
    }
    #[cfg(feature = "bluetooth")]
    for &(route, page) in search_catalog::BLUETOOTH_PAGES {
      if let Some(entry) = self.search_registry.entry_for_route_mut(route) {
        entry.keywords.extend(self.bluetooth.search_rows(page));
      }
    }
    #[cfg(feature = "services")]
    for &(route, page) in search_catalog::SERVICES_PAGES {
      let content = self.services.search_content(page);
      self.index_page(route, content);
    }
    #[cfg(feature = "snapshot")]
    self.index_page("snapshot/home", self.snapshot.search_content());
    #[cfg(feature = "about")]
    for &(route, tab) in search_catalog::ABOUT_TABS {
      let original = std::mem::replace(&mut self.about.active_tab, tab);
      let words = self
        .about
        .current_doc()
        .lines
        .iter()
        .map(ToString::to_string)
        .collect();
      self.about.active_tab = original;
      let path = format!(
        "{} > {} > {}",
        argvus_control_center_core::navigation::HomeCardId::Tools.label(self.lang),
        tr(self.lang, "control_center.about_argvus"),
        tab.label(self.lang)
      );
      self.index_page(route, (path, words));
    }
    #[cfg(feature = "dev-services")]
    self.index_page(
      "dev-services/home",
      (self.dev_services.breadcrumb(), self.dev_services.rows()),
    );
    for &(route, page) in search_catalog::SETTINGS_PAGES {
      if self
        .search_registry
        .entries()
        .iter()
        .any(|entry| entry.route == route)
      {
        let path = self.settings.breadcrumb_for(page);
        let words = self
          .settings
          .rows_for(page)
          .into_iter()
          .map(|row| row.label().to_owned())
          .collect();
        self.index_page(route, (path, words));
      }
    }
    #[cfg(feature = "system")]
    self.index_search_accounts();
  }

  #[cfg(feature = "system")]
  fn index_search_accounts(&mut self) {
    self.search_registry.remove_account_entries();

    let category = format!(
      "{} > {} > {}",
      argvus_control_center_core::navigation::HomeCardId::System.label(self.lang),
      tr(self.lang, "control_center.users_groups"),
      tr(self.lang, "control_center.users")
    );
    let keywords: Vec<String> = [self.lang, Lang::for_locale("en-US")]
      .into_iter()
      .flat_map(|lang| {
        self
          .search_accounts
          .rows(Page::User, lang, "")
          .into_iter()
          .map(|row| row.label().to_owned())
      })
      .collect();
    for name in self.search_accounts.searchable_users() {
      let _ = self.search_registry.register(SearchEntry {
        id: format!("account:{name}"),
        category: category.clone(),
        title: name.clone(),
        keywords: keywords.clone(),
        route: format!("account/{name}"),
      });
    }
    if self.search_active {
      self.update_global_search(self.search_query.clone());
    }
  }

  #[cfg(feature = "system")]
  pub fn poll_search_accounts(&mut self) -> bool {
    let Some(result) = self.search_accounts.poll() else {
      return false;
    };
    if !self.search_active {
      return false;
    }
    let pending = self.pending_search_account.take();
    let selected = self.search_result_ids.get(self.search_selected).cloned();
    self.index_search_accounts();
    if let Some(id) = selected
      && let Some(index) = self
        .search_result_ids
        .iter()
        .position(|candidate| *candidate == id)
    {
      self.search_selected = index;
    }
    if let Some(name) = pending {
      if result.is_ok() && self.search_accounts.searchable_users().contains(&name) {
        self.clear_global_search();
        self.open_search_route(&format!("account/{name}"));
      } else {
        self.search_notice = Some(format!(
          "{}: {name}",
          tr(self.lang, "control_center.entry_not_found")
        ));
      }
    }
    true
  }

  pub fn update_global_search(&mut self, query: String) {
    self.search_notice = None;
    #[cfg(feature = "system")]
    {
      self.pending_search_account = None;
    }
    self.search_query = query;
    self.search_selected = 0;
    self.search_result_ids = self
      .search_registry
      .search(&self.search_query)
      .into_iter()
      .map(|entry| entry.id.clone())
      .collect();
  }

  pub fn pop_global_search(&mut self) {
    let mut query = self.search_query.clone();
    query.pop();
    self.update_global_search(query);
  }

  pub fn move_global_search(&mut self, delta: isize) {
    let last = self.search_result_ids.len().saturating_sub(1);
    self.search_selected = (self.search_selected as isize + delta).clamp(0, last as isize) as usize;
  }

  pub fn clear_global_search(&mut self) {
    self.search_notice = None;
    #[cfg(feature = "system")]
    {
      self.pending_search_account = None;
    }
    self.search_active = false;
    self.search_cursor_visible = false;
    self.search_query.clear();
    self.search_selected = 0;
    self.search_result_ids.clear();
  }

  pub fn poll_search_cursor(&mut self) -> bool {
    if self.search_cursor_last_blink.elapsed() < Duration::from_millis(500) {
      return false;
    }
    self.search_cursor_visible = !self.search_cursor_visible;
    self.search_cursor_last_blink = Instant::now();
    true
  }

  pub fn open_global_search_result(&mut self) {
    let Some(id) = self.search_result_ids.get(self.search_selected).cloned() else {
      return;
    };
    let Some(route) = self
      .search_registry
      .entries()
      .iter()
      .find(|entry| entry.id == id)
      .map(|entry| entry.route.clone())
    else {
      return;
    };
    #[cfg(feature = "system")]
    if let Some(name) = route.strip_prefix("account/") {
      self.pending_search_account = Some(name.to_owned());
      self.search_accounts.load(false);
      return;
    }
    self.clear_global_search();
    self.open_search_route(&route);
  }

  pub fn open_search_route(&mut self, route: &str) {
    #[cfg(feature = "appearance")]
    if let Some((_, page)) = search_catalog::APPEARANCE_PAGES
      .iter()
      .find(|(target, _)| *target == route)
    {
      if page.is_animations()
        || matches!(
          page,
          AppearancePage::Animations
            | AppearancePage::AnimationsCustomCurves
            | AppearancePage::Blur
            | AppearancePage::Borders
            | AppearancePage::GeneralBorders
            | AppearancePage::EdgeThickness
            | AppearancePage::BordersShadow
            | AppearancePage::WindowSpacesInner
            | AppearancePage::WindowSpacesOuter
        )
      {
        crate::hyprland::open_appearance_page(self, *page);
      } else {
        self.return_route = Route::Home;
        self.open_appearance_page(*page);
      }
      return;
    }
    if let Some((_, page)) = search_catalog::SETTINGS_PAGES
      .iter()
      .find(|(target, _)| *target == route)
    {
      self.return_route = Route::Home;
      let parents: &[Page] = match page {
        Page::AppSelector(_) => &[Page::DefaultApps],
        Page::FontSelector(_) | Page::SettingSelector(_) => &[Page::Fonts],
        Page::Users | Page::Groups => &[Page::UsersGroups],
        Page::UserList => &[Page::UsersGroups, Page::Users],
        Page::GroupList => &[Page::UsersGroups, Page::Groups],
        Page::KeyboardVariant | Page::ConsoleKeymap => &[Page::LocaleRegion, Page::Keyboard],
        _ => &[],
      };
      if let Some((root, rest)) = parents.split_first() {
        self.open_settings(*root);
        for &ancestor in rest {
          self.settings.navigation.push(ancestor);
        }
        self.settings.navigation.push(*page);
        self.settings.select_current();
        self.settings.enter_page(*page);
      } else {
        self.open_settings(*page);
      }
      return;
    }
    #[cfg(feature = "audio")]
    if let Some((_, page)) = search_catalog::AUDIO_PAGES
      .iter()
      .find(|(target, _)| *target == route)
    {
      self.open_audio_page(*page);
      return;
    }
    #[cfg(feature = "hardware")]
    if let Some((_, page)) = search_catalog::HARDWARE_PAGES
      .iter()
      .find(|(target, _)| *target == route)
    {
      self.open_hardware_page(*page);
      return;
    }
    #[cfg(feature = "boot")]
    if let Some((_, page)) = search_catalog::BOOT_PAGES
      .iter()
      .find(|(target, _)| *target == route)
    {
      self.open_boot_page(*page);
      return;
    }
    #[cfg(feature = "network")]
    if let Some((_, page)) = search_catalog::NETWORK_PAGES
      .iter()
      .find(|(target, _)| *target == route)
    {
      self.open_network_page(*page);
      return;
    }
    #[cfg(feature = "storage")]
    if let Some((_, page)) = search_catalog::STORAGE_PAGES
      .iter()
      .find(|(target, _)| *target == route)
    {
      self.open_storage_page(*page);
      return;
    }
    #[cfg(feature = "displays")]
    if let Some((_, page)) = search_catalog::DISPLAYS_PAGES
      .iter()
      .find(|(target, _)| *target == route)
    {
      self.open_displays_page(*page);
      return;
    }
    #[cfg(feature = "packages")]
    if let Some((_, page)) = search_catalog::PACKAGES_PAGES
      .iter()
      .find(|(target, _)| *target == route)
    {
      self.open_packages_page(*page);
      return;
    }
    #[cfg(feature = "diagnostics")]
    if let Some((_, page)) = search_catalog::DIAGNOSTICS_PAGES
      .iter()
      .find(|(target, _)| *target == route)
    {
      self.open_diagnostics_page(*page);
      return;
    }
    #[cfg(feature = "session")]
    if let Some((_, page)) = search_catalog::SESSION_PAGES
      .iter()
      .find(|(target, _)| *target == route)
    {
      self.open_session_page(*page);
      return;
    }
    #[cfg(feature = "about")]
    if let Some((_, tab)) = search_catalog::ABOUT_TABS
      .iter()
      .find(|(target, _)| *target == route)
    {
      self.about.active_tab = *tab;
      self.route = Route::About;
      return;
    }
    #[cfg(feature = "bluetooth")]
    if let Some((_, page)) = search_catalog::BLUETOOTH_PAGES
      .iter()
      .find(|(target, _)| *target == route)
    {
      self.open_bluetooth_page(*page);
      return;
    }
    #[cfg(feature = "services")]
    if let Some((_, page)) = search_catalog::SERVICES_PAGES
      .iter()
      .find(|(target, _)| *target == route)
    {
      self.open_services_page(*page);
      return;
    }
    let Some((domain, page)) = route.split_once('/') else {
      return;
    };
    match (domain, page) {
      #[cfg(feature = "system")]
      ("account", name)
        if self
          .settings
          .admin
          .select_search_user(&self.search_accounts, name) =>
      {
        self.return_route = Route::Home;
        self.open_settings(Page::UsersGroups);
        for page in [Page::Users, Page::UserList, Page::User] {
          self.settings.navigation.push(page);
        }
        self.settings.select_current();
      }
      #[cfg(feature = "appearance")]
      ("appearance", "application_style") => {
        self.open_appearance_page(AppearancePage::ApplicationStyle)
      }
      #[cfg(feature = "appearance")]
      ("appearance", "desktop_style") => self.open_appearance_page(AppearancePage::Desktop),
      ("hyprland", "home") => self.open_destination(crate::home::HomeDestination::Hyprland),
      ("hyprland", "info") => {
        self.route = Route::Hyprland;
        crate::hyprland::activate(self, crate::hyprland::Item::Info);
      }
      ("control-center", "settings") => {
        self.open_destination(crate::home::HomeDestination::Settings)
      }
      ("control-center", "style") => self.route = Route::Config,
      ("launcher", "settings") => self.open_launcher(),
      #[cfg(feature = "appearance")]
      ("hyprland", "animations-windows") => crate::hyprland::open_appearance_page(
        self,
        AppearancePage::AnimationsGroup {
          group: argvus_control_center_appearance::AnimationGroup::Windows,
        },
      ),
      (_, "welcome") => self.open_destination(crate::home::HomeDestination::Welcome),
      #[cfg(feature = "apps")]
      (_, "default-apps") => self.open_settings(Page::DefaultApps),
      #[cfg(feature = "language")]
      (_, "language") => self.open_settings(Page::Language),
      ("hyprland", "windows") => crate::hyprland::open_windows(self),
      #[cfg(feature = "apps")]
      (_, "window-rules" | "windows-rules") => self.open_settings(Page::WindowRules),
      (_, "window-mode" | "windows-mode") => self.open_settings(Page::WindowMode),
      #[cfg(feature = "appearance")]
      (_, "window-spaces" | "windows-spaces") => {
        crate::hyprland::open_appearance_page(self, AppearancePage::WindowSpaces)
      }
      #[cfg(feature = "apps")]
      (_, "projects") => self.open_settings(Page::Projects),
      (_, "snippets") => self.open_settings(Page::Snippets),
      #[cfg(feature = "apps")]
      (_, "ssh-keys") => self.open_settings(Page::SshKeys),
      #[cfg(feature = "apps")]
      (_, "gpg-keys") => self.open_settings(Page::GpgKeys),
      #[cfg(feature = "system")]
      (_, "firewall") => self.open_settings(Page::Firewall),
      (_, "hostname") => self.open_settings(Page::Hostname),
      (_, "users-groups") => self.open_settings(Page::UsersGroups),
      (_, "notifications") => self.open_settings(Page::Notifications),
      #[cfg(feature = "fonts")]
      (_, "fonts") => self.open_settings(Page::Fonts),
      (_, "keybindings") => self.open_settings(Page::Keybindings),
      (_, "shortcuts-create") => self.open_settings(Page::CustomShortcut),
      (_, "shortcuts-list") => self.open_settings(Page::AllShortcuts),
      #[cfg(feature = "about")]
      ("about", "system") => {
        self.about.active_tab = Tab::System;
        self.route = Route::About;
      }

      #[cfg(feature = "dev-services")]
      ("dev-services", "home") => {
        self.route = Route::DevServices;
        self.dev_services.reload();
      }

      #[cfg(feature = "packages")]
      ("packages", "search") => self.open_packages_page(PackagesPage::Task(
        PackageSource::Official,
        PackageTask::Install,
      )),

      #[cfg(feature = "packages")]
      ("packages", "official") => {
        self.open_packages_page(PackagesPage::Source(PackageSource::Official))
      }
      #[cfg(feature = "packages")]
      ("packages", "installed") => self.open_packages_page(PackagesPage::Task(
        PackageSource::Official,
        PackageTask::Remove,
      )),
      #[cfg(feature = "packages")]
      ("packages", "updates") => self.open_packages_page(PackagesPage::Task(
        PackageSource::Official,
        PackageTask::Update,
      )),

      #[cfg(feature = "packages")]
      ("packages", "aur") => {
        self.open_packages_page(PackagesPage::Task(PackageSource::Aur, PackageTask::Install))
      }
      #[cfg(feature = "packages")]
      ("packages", "aur-remove") => {
        self.open_packages_page(PackagesPage::Task(PackageSource::Aur, PackageTask::Remove))
      }

      #[cfg(feature = "packages")]
      ("packages", "aur-updates") => {
        self.open_packages_page(PackagesPage::Task(PackageSource::Aur, PackageTask::Update))
      }

      #[cfg(feature = "snapshot")]
      ("snapshot", "home") => self.open_snapshot(),

      #[cfg(feature = "appearance")]
      ("appearance", "summary") => self.open_appearance_page(AppearancePage::Home),
      #[cfg(feature = "appearance")]
      ("appearance", "themes") => self.open_appearance_page(AppearancePage::Themes),
      #[cfg(feature = "appearance")]
      ("appearance", "wallpapers") => self.open_appearance_page(AppearancePage::Wallpapers),
      #[cfg(feature = "appearance")]
      ("appearance", "icons") => self.open_appearance_page(AppearancePage::Icons),
      #[cfg(feature = "appearance")]
      ("appearance", "accents") => self.open_appearance_page(AppearancePage::Accents),
      #[cfg(feature = "appearance")]
      ("appearance", "taskbar")
      | ("appearance", "taskbar-group")
      | ("appearance", "utility-group") => self.open_appearance_page(AppearancePage::Taskbar),
      #[cfg(feature = "appearance")]
      ("appearance", "taskbar-position") | ("appearance", "position") => {
        self.open_appearance_page(AppearancePage::TaskbarPosition)
      }
      #[cfg(feature = "appearance")]
      ("appearance", "taskbar-icons") => self.open_appearance_page(AppearancePage::TaskbarIcons),
      #[cfg(feature = "appearance")]
      ("appearance", "taskbar-date") => self.open_appearance_page(AppearancePage::TaskbarDate),
      #[cfg(feature = "appearance")]
      ("appearance", "taskbar-time") => self.open_appearance_page(AppearancePage::TaskbarTime),
      #[cfg(feature = "appearance")]
      ("appearance", "taskbar-height") | ("appearance", "height") => {
        self.open_appearance_page(AppearancePage::TaskbarHeight)
      }
      #[cfg(feature = "appearance")]
      ("appearance", "taskbar-border") | ("appearance", "border") => {
        self.open_appearance_page(AppearancePage::TaskbarBorder)
      }
      #[cfg(feature = "appearance")]
      ("appearance", "widget-telemetry") => {
        self.open_appearance_page(AppearancePage::WidgetTelemetry)
      }
      #[cfg(feature = "appearance")]
      ("appearance", "control-panel") => self.open_appearance_page(AppearancePage::ControlPanel),
      #[cfg(feature = "power")]
      ("power", "summary") => {
        self.route = Route::Power;
        self.power.reload();
      }
      _ => {}
    }
  }
}
