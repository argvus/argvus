//! Global search on the Home: typing, moving through the results and opening the route of the chosen entry.

use super::*;

impl App {
  pub fn begin_global_search(&mut self) {
    self.search_active = true;
    self.search_cursor_visible = true;
    self.search_cursor_last_blink = Instant::now();
    self.search_query.clear();
    self.search_selected = 0;
    self.search_result_ids.clear();
  }

  pub fn update_global_search(&mut self, query: String) {
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
    self.search_active = false;
    self.search_cursor_visible = false;
    self.search_query.clear();
    self.search_selected = 0;
    self.search_result_ids.clear();
  }

  pub fn poll_search_cursor(&mut self) -> bool {
    if !self.search_active || self.search_cursor_last_blink.elapsed() < Duration::from_millis(500) {
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
    self.clear_global_search();
    self.open_search_route(&route);
  }

  pub(crate) fn open_search_route(&mut self, route: &str) {
    let Some((domain, page)) = route.split_once('/') else {
      return;
    };
    match (domain, page) {
      (_, "welcome") => self.open_destination(crate::home::HomeDestination::Welcome),
      #[cfg(feature = "apps")]
      (_, "default-apps") => self.open_settings(Page::DefaultApps),
      #[cfg(feature = "apps")]
      (_, "window-rules") => self.open_settings(Page::WindowRules),
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
      #[cfg(feature = "about")]
      ("about", "system") => {
        self.about.active_tab = Tab::System;
        self.route = Route::About;
      }
      #[cfg(feature = "hardware")]
      ("hardware", "summary") => self.open_hardware_page(HardwarePage::Summary),
      #[cfg(feature = "hardware")]
      ("hardware", "home") => self.open_hardware_page(HardwarePage::Home),
      #[cfg(feature = "hardware")]
      ("hardware", "cpu") => self.open_hardware_page(HardwarePage::Cpu),
      #[cfg(feature = "hardware")]
      ("hardware", "gpu") => self.open_hardware_page(HardwarePage::Gpu),
      #[cfg(feature = "hardware")]
      ("hardware", "memory") => self.open_hardware_page(HardwarePage::Memory),
      #[cfg(feature = "hardware")]
      ("hardware", "power") => self.open_hardware_page(HardwarePage::Power),
      #[cfg(feature = "hardware")]
      ("hardware", "devices") => self.open_hardware_page(HardwarePage::Devices),
      #[cfg(feature = "services")]
      ("services", "system") => self.open_services_page(ServicePage::System),
      #[cfg(feature = "services")]
      ("services", "home") => self.open_services_page(ServicePage::Home),
      #[cfg(feature = "services")]
      ("services", "user") => self.open_services_page(ServicePage::User),
      #[cfg(feature = "services")]
      ("services", "failed") => self.open_services_page(ServicePage::Failed),
      #[cfg(feature = "services")]
      ("services", "logs") => self.open_services_page(ServicePage::Logs),
      #[cfg(feature = "dev-services")]
      ("dev-services", "home") => {
        self.route = Route::DevServices;
        self.dev_services.reload();
      }
      #[cfg(feature = "network")]
      ("network", "status") => self.open_network_page(NetworkPage::Status),
      #[cfg(feature = "network")]
      ("network", "home") => self.open_network_page(NetworkPage::Home),
      #[cfg(feature = "network")]
      ("network", "interfaces") => self.open_network_page(NetworkPage::Interfaces),
      #[cfg(feature = "network")]
      ("network", "wifi") => self.open_network_page(NetworkPage::Wifi),
      #[cfg(feature = "network")]
      ("network", "ethernet") => self.open_network_page(NetworkPage::Ethernet),
      #[cfg(feature = "network")]
      ("network", "vpn") => self.open_network_page(NetworkPage::Vpn),
      #[cfg(feature = "network")]
      ("network", "dns") => self.open_network_page(NetworkPage::Dns),
      #[cfg(feature = "network")]
      ("network", "proxy") => self.open_network_page(NetworkPage::Proxy),
      #[cfg(feature = "network")]
      ("network", "connections") => self.open_network_page(NetworkPage::Connections),
      #[cfg(feature = "network")]
      ("network", "activate") => self.open_network_page(NetworkPage::Activate),
      #[cfg(feature = "network")]
      ("network", "add") => self.open_network_page(NetworkPage::AddConnection),
      #[cfg(feature = "audio")]
      ("audio", "summary") => self.open_audio_page(AudioPage::Summary),
      #[cfg(feature = "audio")]
      ("audio", "home") => self.open_audio_page(AudioPage::Home),
      #[cfg(feature = "audio")]
      ("audio", "output") => self.open_audio_page(AudioPage::Output),
      #[cfg(feature = "audio")]
      ("audio", "input") => self.open_audio_page(AudioPage::Input),
      #[cfg(feature = "audio")]
      ("audio", "devices") => self.open_audio_page(AudioPage::Devices),
      #[cfg(feature = "bluetooth")]
      ("bluetooth", "state") => self.open_bluetooth_page(BluetoothPage::State),
      #[cfg(feature = "bluetooth")]
      ("bluetooth", "home") => self.open_bluetooth_page(BluetoothPage::Home),
      #[cfg(feature = "bluetooth")]
      ("bluetooth", "devices") => self.open_bluetooth_page(BluetoothPage::Devices),
      #[cfg(feature = "bluetooth")]
      ("bluetooth", "pair") => self.open_bluetooth_page(BluetoothPage::Pair),
      #[cfg(feature = "boot")]
      ("boot", "summary") => self.open_boot_page(BootPage::Summary),
      #[cfg(feature = "boot")]
      ("boot", "home") => self.open_boot_page(BootPage::Home),
      #[cfg(feature = "boot")]
      ("boot", "kernel") => self.open_boot_page(BootPage::Kernel),
      #[cfg(feature = "boot")]
      ("boot", "bootloader") => self.open_boot_page(BootPage::Bootloader),
      #[cfg(feature = "boot")]
      ("boot", "initramfs") => self.open_boot_page(BootPage::Initramfs),
      #[cfg(feature = "boot")]
      ("boot", "plymouth") => self.open_boot_page(BootPage::Plymouth),
      #[cfg(feature = "packages")]
      ("packages", "search") => self.open_packages_page(PackagesPage::Task(
        PackageSource::Official,
        PackageTask::Install,
      )),
      #[cfg(feature = "packages")]
      ("packages", "home") => self.open_packages_page(PackagesPage::Home),
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
      ("packages", "orphans") => self.open_packages_page(PackagesPage::Orphans),
      #[cfg(feature = "packages")]
      ("packages", "cache") => self.open_packages_page(PackagesPage::Cache),
      #[cfg(feature = "packages")]
      ("packages", "aur") => {
        self.open_packages_page(PackagesPage::Task(PackageSource::Aur, PackageTask::Install))
      }
      #[cfg(feature = "packages")]
      ("packages", "aur-remove") => {
        self.open_packages_page(PackagesPage::Task(PackageSource::Aur, PackageTask::Remove))
      }
      #[cfg(feature = "packages")]
      ("packages", "repositories") => self.open_packages_page(PackagesPage::Repositories),
      #[cfg(feature = "packages")]
      ("packages", "repositories-add") => self.open_packages_page(PackagesPage::RepositoryAdd),
      #[cfg(feature = "packages")]
      ("packages", "repositories-list") => self.open_packages_page(PackagesPage::RepositoryList),
      #[cfg(feature = "packages")]
      ("packages", "aur-updates") => {
        self.open_packages_page(PackagesPage::Task(PackageSource::Aur, PackageTask::Update))
      }
      #[cfg(feature = "packages")]
      ("packages", "history") => self.open_packages_page(PackagesPage::History),
      #[cfg(feature = "packages")]
      ("packages", "downgrade") => self.open_packages_page(PackagesPage::Downgrade),
      #[cfg(feature = "packages")]
      ("packages", "mirrors") => self.open_packages_page(PackagesPage::Mirrors),
      #[cfg(feature = "storage")]
      ("storage", "summary") => self.open_storage_page(StoragePage::Summary),
      #[cfg(feature = "snapshot")]
      ("snapshot", "home") => self.open_snapshot(),
      #[cfg(feature = "storage")]
      ("storage", "home") => self.open_storage_page(StoragePage::Home),
      #[cfg(feature = "storage")]
      ("storage", "disks") => self.open_storage_page(StoragePage::Disks),
      #[cfg(feature = "storage")]
      ("storage", "partitions") => self.open_storage_page(StoragePage::Partitions),
      #[cfg(feature = "storage")]
      ("storage", "filesystems") => self.open_storage_page(StoragePage::Filesystems),
      #[cfg(feature = "storage")]
      ("storage", "mounts") => self.open_storage_page(StoragePage::Mounts),
      #[cfg(feature = "storage")]
      ("storage", "smart") => self.open_storage_page(StoragePage::Smart),
      #[cfg(feature = "storage")]
      ("storage", "usage") => self.open_storage_page(StoragePage::Usage),
      #[cfg(feature = "diagnostics")]
      ("diagnostics", "summary") => self.open_diagnostics_page(DiagnosticPage::Summary),
      #[cfg(feature = "diagnostics")]
      ("diagnostics", "home") => self.open_diagnostics_page(DiagnosticPage::Home),
      #[cfg(feature = "diagnostics")]
      ("diagnostics", "services") => self.open_diagnostics_page(DiagnosticPage::Services),
      #[cfg(feature = "diagnostics")]
      ("diagnostics", "boot") => self.open_diagnostics_page(DiagnosticPage::Boot),
      #[cfg(feature = "diagnostics")]
      ("diagnostics", "graphics") => self.open_diagnostics_page(DiagnosticPage::Graphics),
      #[cfg(feature = "diagnostics")]
      ("diagnostics", "network") => self.open_diagnostics_page(DiagnosticPage::Network),
      #[cfg(feature = "diagnostics")]
      ("diagnostics", "audio") => self.open_diagnostics_page(DiagnosticPage::Audio),
      #[cfg(feature = "diagnostics")]
      ("diagnostics", "bluetooth") => self.open_diagnostics_page(DiagnosticPage::Bluetooth),
      #[cfg(feature = "diagnostics")]
      ("diagnostics", "storage") => self.open_diagnostics_page(DiagnosticPage::Storage),
      #[cfg(feature = "diagnostics")]
      ("diagnostics", "packages") => self.open_diagnostics_page(DiagnosticPage::Packages),
      #[cfg(feature = "diagnostics")]
      ("diagnostics", "argvus") => self.open_diagnostics_page(DiagnosticPage::Argvus),
      #[cfg(feature = "session")]
      ("session", "summary") => self.open_session_page(SessionPage::Home),
      #[cfg(feature = "session")]
      ("session", "components") => self.open_session_page(SessionPage::Components),
      #[cfg(feature = "session")]
      ("session", "autostart") => self.open_session_page(SessionPage::Autostart),
      #[cfg(feature = "session")]
      ("session", "diagnostics") => self.open_session_page(SessionPage::Diagnostics),
      #[cfg(feature = "session")]
      ("session", "logs") => self.open_session_page(SessionPage::Logs),
      #[cfg(feature = "displays")]
      ("displays", "summary") => self.open_displays_page(DisplayPage::Home),
      #[cfg(feature = "displays")]
      ("displays", "night-light") => self.open_displays_page(DisplayPage::NightLight),
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
