//! Implements application state and main-flow coordination in crate `argvus control center`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! `Route` represents the currently displayed page; `InitialRoute` preserves the
//! intent of a direct route until application initialization completes. This
//! distinction lets the CLI, global search, and shortcuts share the same
//! navigation authority without duplicating screens or creating parallel paths.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
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
use argvus_control_center_packages::{PackagesApp, PackagesPage};
#[cfg(feature = "power")]
use argvus_control_center_power::PowerApp;
#[cfg(feature = "services")]
use argvus_control_center_services::{ServicePage, ServicesApp};
#[cfg(feature = "session")]
use argvus_control_center_session::{SessionApp, SessionPage};
use argvus_control_center_settings::{App as SettingsState, Page};
#[cfg(feature = "storage")]
use argvus_control_center_storage::{StorageApp, StoragePage};
use argvus_i18n::{Lang, tr};
use argvus_theme::Theme;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Defines `Route`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub enum Route {
  Home,
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
/// Defines `InitialRoute`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub enum InitialRoute {
  Home,
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

/// Represents `App`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct App {
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
}

impl App {
  /// Constructs `new` with this module's expected initial state. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn new(initial: InitialRoute) -> Self {
    let lang = Lang::detect();
    let theme = Theme::load();
    let route = match initial {
      InitialRoute::Home => Route::Home,
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
    };
    let page = match initial {
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
    };
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
    let network = NetworkApp::new(lang, theme.clone(), capabilities.clone());
    #[cfg(feature = "audio")]
    let audio = AudioApp::new(lang, theme.clone(), capabilities.clone());
    #[cfg(feature = "bluetooth")]
    let bluetooth = BluetoothApp::new(lang, theme.clone(), capabilities.clone());
    #[cfg(feature = "boot")]
    let boot = BootApp::new(lang, theme.clone(), capabilities.clone());
    #[cfg(feature = "packages")]
    let packages = PackagesApp::new(lang, theme.clone(), capabilities.clone());
    #[cfg(feature = "storage")]
    let storage = StorageApp::new(lang, theme.clone(), capabilities.clone());
    #[cfg(feature = "diagnostics")]
    let diagnostics = DiagnosticsApp::new(lang, theme.clone(), capabilities.clone());
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
    let mut network = network;
    #[cfg(feature = "audio")]
    let mut audio = audio;
    #[cfg(feature = "bluetooth")]
    let mut bluetooth = bluetooth;
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
    let mut boot = boot;
    #[cfg(feature = "boot")]
    if let InitialRoute::Boot(page) = initial {
      boot.page = page;
      boot.reload();
    }
    #[cfg(feature = "packages")]
    let mut packages = packages;
    #[cfg(feature = "packages")]
    if let InitialRoute::Packages(page) = initial {
      packages.page = page;
      packages.reload();
    }
    #[cfg(feature = "storage")]
    let mut storage = storage;
    #[cfg(feature = "storage")]
    if let InitialRoute::Storage(page) = initial {
      storage.page = page;
      storage.reload();
    }
    #[cfg(feature = "diagnostics")]
    let mut diagnostics = diagnostics;
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
      appearance.page = page;
      appearance.reload();
    }
    let mut search_registry = SearchRegistry::default();
    let entries: [(&str, &str, &str, &str, &str); _] = [
      #[cfg(feature = "fonts")]
      (
        "settings.fonts",
        "appearance",
        "control_center.fonts",
        "font fonts typeface typography",
        "settings/fonts",
      ),
      #[cfg(feature = "apps")]
      (
        "settings.default_apps",
        "preferences",
        "control_center.default_apps",
        "default application apps browser terminal file manager",
        "settings/default-apps",
      ),
      #[cfg(feature = "apps")]
      (
        "settings.window_rules",
        "hyprland",
        "control_center.window_rules",
        "window rules workspace class app placement regras janela area de trabalho",
        "settings/window-rules",
      ),
      #[cfg(feature = "apps")]
      (
        "settings.projects",
        "card_tools",
        "control_center.projects",
        "projects project folders path directory root shortcut seletor projetos pastas caminho diretorio raiz atalho",
        "settings/projects",
      ),
      #[cfg(feature = "apps")]
      (
        "settings.snippets",
        "card_tools",
        "control_center.snippets",
        "snippets snippet text type typing shortcut picker trechos texto digitar atalho seletor",
        "settings/snippets",
      ),
      #[cfg(feature = "apps")]
      (
        "settings.ssh_keys",
        "card_tools",
        "control_center.ssh_keys",
        "ssh keys key pair generate load unload agent passphrase chaves ssh gerar carregar descarregar agente senha par",
        "settings/ssh-keys",
      ),
      #[cfg(feature = "apps")]
      (
        "settings.gpg_keys",
        "card_tools",
        "control_center.gpg_keys",
        "gpg keys key pair generate export import pinentry passphrase chaves gpg gerar exportar importar senha pinentry",
        "settings/gpg-keys",
      ),
      #[cfg(feature = "system")]
      (
        "settings.firewall",
        "card_system",
        "control_center.firewall",
        "firewall ports rules block allow network security portas regras bloquear permitir rede seguranca",
        "settings/firewall",
      ),
      #[cfg(feature = "system")]
      (
        "settings.hostname",
        "card_system",
        "control_center.hostname",
        "hostname name machine computer host nome maquina computador",
        "settings/hostname",
      ),
      #[cfg(feature = "system")]
      (
        "settings.users_groups",
        "card_system",
        "control_center.users_groups",
        "users groups accounts usuarios grupos contas",
        "settings/users-groups",
      ),
      #[cfg(feature = "system")]
      (
        "settings.notifications",
        "card_system",
        "control_center.notifications",
        "notifications do not disturb dnd silence notificacoes nao perturbe silenciar",
        "settings/notifications",
      ),
      #[cfg(feature = "about")]
      (
        "about",
        "system",
        "control_center.about",
        "about information version license",
        "about/system",
      ),
      #[cfg(feature = "network")]
      (
        "network",
        "network",
        "control_center.network",
        "network internet connectivity",
        "network/home",
      ),
      #[cfg(feature = "audio")]
      (
        "audio",
        "audio",
        "control_center.audio",
        "audio sound volume",
        "audio/home",
      ),
      #[cfg(feature = "bluetooth")]
      (
        "bluetooth",
        "bluetooth",
        "control_center.bluetooth",
        "bluetooth wireless pairing",
        "bluetooth/home",
      ),
      #[cfg(feature = "hardware")]
      (
        "hardware",
        "hardware",
        "control_center.hardware",
        "hardware system devices",
        "hardware/home",
      ),
      (
        "settings.keybindings",
        "card_desktop_input",
        "control_center.keyboard_shortcuts",
        "keyboard shortcuts keybindings hotkeys bindings atalhos teclado",
        "settings/keybindings",
      ),
      #[cfg(feature = "services")]
      (
        "services",
        "services",
        "control_center.services",
        "services systemd daemon",
        "services/home",
      ),
      #[cfg(feature = "dev-services")]
      (
        "dev-services",
        "dev-services",
        "control_center.developer_services",
        "dev services systemd ports containers podman docker desenvolvimento portas",
        "dev-services/home",
      ),
      #[cfg(feature = "boot")]
      (
        "boot",
        "boot",
        "control_center.boot",
        "boot startup",
        "boot/home",
      ),
      #[cfg(feature = "packages")]
      (
        "packages",
        "packages",
        "control_center.section_packages",
        "package pacman software",
        "packages/home",
      ),
      #[cfg(feature = "storage")]
      (
        "storage",
        "storage",
        "control_center.storage",
        "storage disk filesystem",
        "storage/home",
      ),
      #[cfg(feature = "diagnostics")]
      (
        "diagnostics",
        "diagnostics",
        "control_center.diagnostics",
        "diagnostics health",
        "diagnostics/home",
      ),
      #[cfg(feature = "hardware")]
      (
        "hardware.summary",
        "hardware",
        "control_center.summary",
        "summary hardware overview",
        "hardware/summary",
      ),
      #[cfg(feature = "hardware")]
      (
        "hardware.cpu",
        "hardware",
        "control_center.hardware_cpu",
        "cpu processor",
        "hardware/cpu",
      ),
      #[cfg(feature = "hardware")]
      (
        "hardware.gpu",
        "hardware",
        "control_center.hardware_gpu",
        "gpu graphics video driver",
        "hardware/gpu",
      ),
      #[cfg(feature = "hardware")]
      (
        "hardware.memory",
        "hardware",
        "control_center.memory",
        "memory ram",
        "hardware/memory",
      ),
      #[cfg(feature = "hardware")]
      (
        "hardware.power",
        "hardware",
        "control_center.power",
        "power energy battery",
        "hardware/power",
      ),
      #[cfg(feature = "hardware")]
      (
        "hardware.devices",
        "hardware",
        "control_center.devices",
        "devices pci usb drm input",
        "hardware/devices",
      ),
      #[cfg(feature = "services")]
      (
        "services.system",
        "services",
        "control_center.system",
        "systemd services service daemon",
        "services/system",
      ),
      #[cfg(feature = "services")]
      (
        "services.user",
        "services",
        "control_center.user",
        "systemd user service",
        "services/user",
      ),
      #[cfg(feature = "services")]
      (
        "services.failed",
        "services",
        "control_center.failed",
        "failed failures",
        "services/failed",
      ),
      #[cfg(feature = "services")]
      (
        "services.logs",
        "services",
        "control_center.logs",
        "logs journal journalctl",
        "services/logs",
      ),
      #[cfg(feature = "network")]
      (
        "network.status",
        "network",
        "control_center.status",
        "network rede internet",
        "network/status",
      ),
      #[cfg(feature = "network")]
      (
        "network.interfaces",
        "network",
        "control_center.interfaces",
        "interfaces interface rede",
        "network/interfaces",
      ),
      #[cfg(feature = "network")]
      (
        "network.wifi",
        "network",
        "control_center.wi_fi",
        "wifi wireless wlan sem fio",
        "network/wifi",
      ),
      #[cfg(feature = "network")]
      (
        "network.ethernet",
        "network",
        "control_center.ethernet",
        "ethernet lan",
        "network/ethernet",
      ),
      #[cfg(feature = "network")]
      (
        "network.vpn",
        "network",
        "control_center.vpn",
        "vpn wireguard openvpn tunnel",
        "network/vpn",
      ),
      #[cfg(feature = "network")]
      (
        "network.dns",
        "network",
        "control_center.dns",
        "dns resolved nameserver",
        "network/dns",
      ),
      #[cfg(feature = "network")]
      (
        "network.proxy",
        "network",
        "control_center.proxy",
        "proxy http https",
        "network/proxy",
      ),
      #[cfg(feature = "audio")]
      (
        "audio.output",
        "audio",
        "control_center.output",
        "audio sound som output saída",
        "audio/output",
      ),
      #[cfg(feature = "audio")]
      (
        "audio.input",
        "audio",
        "control_center.input",
        "audio microphone microfone input entrada",
        "audio/input",
      ),
      #[cfg(feature = "audio")]
      (
        "audio.devices",
        "audio",
        "control_center.devices",
        "audio devices dispositivos",
        "audio/devices",
      ),
      #[cfg(feature = "bluetooth")]
      (
        "bluetooth.state",
        "bluetooth",
        "control_center.state",
        "bluetooth bt estado",
        "bluetooth/state",
      ),
      #[cfg(feature = "bluetooth")]
      (
        "bluetooth.devices",
        "bluetooth",
        "control_center.devices",
        "bluetooth dispositivos",
        "bluetooth/devices",
      ),
      #[cfg(feature = "bluetooth")]
      (
        "bluetooth.pair",
        "bluetooth",
        "control_center.pair",
        "bluetooth pair parear",
        "bluetooth/pair",
      ),
      #[cfg(feature = "boot")]
      (
        "boot.summary",
        "boot",
        "control_center.summary",
        "boot startup inicializacao kernel",
        "boot/summary",
      ),
      #[cfg(feature = "boot")]
      (
        "boot.kernel",
        "boot",
        "control_center.kernel",
        "kernel linux",
        "boot/kernel",
      ),
      #[cfg(feature = "boot")]
      (
        "boot.bootloader",
        "boot",
        "control_center.bootloader_f1a3c5",
        "bootloader grub systemd-boot loader",
        "boot/bootloader",
      ),
      #[cfg(feature = "boot")]
      (
        "boot.initramfs",
        "boot",
        "control_center.initramfs",
        "initramfs mkinitcpio",
        "boot/initramfs",
      ),
      #[cfg(feature = "boot")]
      (
        "boot.plymouth",
        "boot",
        "control_center.plymouth",
        "plymouth splash",
        "boot/plymouth",
      ),
      #[cfg(feature = "packages")]
      (
        "packages.search",
        "packages",
        "control_center.search_2c43ee",
        "pacman package pacote",
        "packages/search",
      ),
      #[cfg(feature = "packages")]
      (
        "packages.installed",
        "packages",
        "control_center.installed_e91b6d",
        "installed instalados",
        "packages/installed",
      ),
      #[cfg(feature = "packages")]
      (
        "packages.updates",
        "packages",
        "control_center.updates",
        "update upgrade atualizar",
        "packages/updates",
      ),
      #[cfg(feature = "packages")]
      (
        "packages.orphans",
        "packages",
        "control_center.orphans_29aae8",
        "orphan órfão orfao",
        "packages/orphans",
      ),
      #[cfg(feature = "packages")]
      (
        "packages.cache",
        "packages",
        "control_center.cache",
        "cache paccache",
        "packages/cache",
      ),
      #[cfg(feature = "packages")]
      (
        "packages.aur",
        "packages",
        "control_center.aur",
        "aur paru yay",
        "packages/aur",
      ),
      #[cfg(feature = "packages")]
      (
        "packages.history",
        "packages",
        "control_center.history",
        "history histórico pacman.log",
        "packages/history",
      ),
      #[cfg(feature = "packages")]
      (
        "packages.downgrade",
        "packages",
        "control_center.downgrade",
        "downgrade",
        "packages/downgrade",
      ),
      #[cfg(feature = "packages")]
      (
        "packages.mirrors",
        "packages",
        "control_center.mirrors",
        "mirror reflector",
        "packages/mirrors",
      ),
      #[cfg(feature = "storage")]
      (
        "storage.summary",
        "storage",
        "control_center.summary",
        "storage armazenamento disk disco",
        "storage/summary",
      ),
      #[cfg(feature = "storage")]
      (
        "storage.disks",
        "storage",
        "control_center.disks",
        "disk disco nvme ssd hdd",
        "storage/disks",
      ),
      #[cfg(feature = "storage")]
      (
        "storage.partitions",
        "storage",
        "control_center.partitions",
        "partition partição",
        "storage/partitions",
      ),
      #[cfg(feature = "storage")]
      (
        "storage.filesystems",
        "storage",
        "control_center.filesystems",
        "filesystem sistema arquivos",
        "storage/filesystems",
      ),
      #[cfg(feature = "storage")]
      (
        "storage.mounts",
        "storage",
        "control_center.mount_points",
        "mount montagem",
        "storage/mounts",
      ),
      #[cfg(feature = "storage")]
      (
        "storage.smart",
        "storage",
        "control_center.smart",
        "smart health saúde",
        "storage/smart",
      ),
      #[cfg(feature = "storage")]
      (
        "storage.usage",
        "storage",
        "control_center.disk_usage",
        "space espaço uso disco",
        "storage/usage",
      ),
      #[cfg(feature = "diagnostics")]
      (
        "diagnostics.summary",
        "diagnostics",
        "control_center.summary",
        "diagnostic diagnóstico health status",
        "diagnostics/summary",
      ),
      #[cfg(feature = "diagnostics")]
      (
        "diagnostics.services",
        "diagnostics",
        "control_center.services",
        "services serviços failed falhos",
        "diagnostics/services",
      ),
      #[cfg(feature = "diagnostics")]
      (
        "diagnostics.boot",
        "diagnostics",
        "control_center.kernel_boot",
        "boot kernel initramfs",
        "diagnostics/boot",
      ),
      #[cfg(feature = "diagnostics")]
      (
        "diagnostics.graphics",
        "diagnostics",
        "control_center.graphics",
        "graphics gráficos gpu vmwgfx",
        "diagnostics/graphics",
      ),
      #[cfg(feature = "diagnostics")]
      (
        "diagnostics.network",
        "diagnostics",
        "control_center.network",
        "network rede dns",
        "diagnostics/network",
      ),
      #[cfg(feature = "diagnostics")]
      (
        "diagnostics.audio",
        "diagnostics",
        "control_center.audio",
        "audio áudio pipewire",
        "diagnostics/audio",
      ),
      #[cfg(feature = "diagnostics")]
      (
        "diagnostics.bluetooth",
        "diagnostics",
        "control_center.bluetooth",
        "bluetooth",
        "diagnostics/bluetooth",
      ),
      #[cfg(feature = "diagnostics")]
      (
        "diagnostics.storage",
        "diagnostics",
        "control_center.storage",
        "storage armazenamento smart",
        "diagnostics/storage",
      ),
      #[cfg(feature = "diagnostics")]
      (
        "diagnostics.packages",
        "diagnostics",
        "control_center.section_packages",
        "packages pacotes updates",
        "diagnostics/packages",
      ),
      #[cfg(feature = "diagnostics")]
      (
        "diagnostics.argvus",
        "diagnostics",
        "control_center.argvus",
        "argvus session theme tema",
        "diagnostics/argvus",
      ),
      #[cfg(feature = "power")]
      (
        "power.summary",
        "power",
        "control_center.summary",
        "power energia energy battery bateria",
        "power/summary",
      ),
      #[cfg(feature = "session")]
      (
        "session.summary",
        "session",
        "control_center.summary",
        "session sessão inicio inicializacao",
        "session/summary",
      ),
      #[cfg(feature = "session")]
      (
        "session.components",
        "session",
        "control_center.components_7073b6",
        "components componentes desktop wayland hyprland",
        "session/components",
      ),
      #[cfg(feature = "session")]
      (
        "session.autostart",
        "session",
        "control_center.autostart",
        "autostart login inicialização iniciar",
        "session/autostart",
      ),
      #[cfg(feature = "session")]
      (
        "session.diagnostics",
        "session",
        "control_center.diagnostics",
        "diagnostic diagnóstico",
        "session/diagnostics",
      ),
      #[cfg(feature = "session")]
      (
        "session.logs",
        "session",
        "control_center.logs",
        "logs journal journalctl",
        "session/logs",
      ),
      #[cfg(feature = "displays")]
      (
        "displays.summary",
        "displays",
        "control_center.summary",
        "displays monitores monitor screens telas",
        "displays/summary",
      ),
      #[cfg(feature = "displays")]
      (
        "displays.mode",
        "displays",
        "control_center.resolution",
        "resolution resolucao modo mode refresh",
        "displays/mode",
      ),
      #[cfg(feature = "displays")]
      (
        "displays.scale",
        "displays",
        "control_center.scale",
        "scale escala zoom",
        "displays/scale",
      ),
      #[cfg(feature = "displays")]
      (
        "displays.position",
        "displays",
        "control_center.position",
        "position posicao layout arrangement",
        "displays/position",
      ),
      #[cfg(feature = "displays")]
      (
        "displays.vrr",
        "displays",
        "control_center.vrr",
        "vrr variable refresh rate freesync",
        "displays/vrr",
      ),
      #[cfg(feature = "displays")]
      (
        "displays.hdr",
        "displays",
        "control_center.hdr",
        "hdr high dynamic range",
        "displays/hdr",
      ),
      #[cfg(feature = "displays")]
      (
        "displays.orientation",
        "displays",
        "control_center.orientation",
        "orientation orientacao rotate rotacionar",
        "displays/orientation",
      ),
      #[cfg(feature = "displays")]
      (
        "displays.primary",
        "displays",
        "control_center.primary_monitor",
        "primary principal",
        "displays/primary",
      ),
      #[cfg(feature = "appearance")]
      (
        "appearance.summary",
        "appearance",
        "control_center.summary",
        "appearance aparencia theme tema wallpaper papel de parede accent cor destaque",
        "appearance/summary",
      ),
      #[cfg(feature = "appearance")]
      (
        "appearance.themes",
        "appearance",
        "control_center.themes",
        "themes temas theme tema appearance aparencia",
        "appearance/themes",
      ),
      #[cfg(feature = "appearance")]
      (
        "appearance.wallpapers",
        "appearance",
        "control_center.wallpapers",
        "wallpapers papéis de parede papel wallpaper plano de fundo",
        "appearance/wallpapers",
      ),
      #[cfg(feature = "appearance")]
      (
        "appearance.icons",
        "appearance",
        "control_center.icons",
        "icons ícones icones icon theme tema de ícones tema de icones papirus adwaita",
        "appearance/icons",
      ),
      #[cfg(feature = "appearance")]
      (
        "appearance.accents",
        "appearance",
        "control_center.highlight_color",
        "accent cor de destaque highlight color aparencia",
        "appearance/accents",
      ),
      #[cfg(feature = "appearance")]
      (
        "appearance.taskbar",
        "appearance",
        "control_center.taskbar",
        "taskbar barra grupo utility group hover expandido expanded sempre always",
        "appearance/taskbar",
      ),
      #[cfg(feature = "appearance")]
      (
        "appearance.taskbar_position",
        "taskbar",
        "control_center.taskbar_position",
        "taskbar barra posicao position gaps espacamento margin margem",
        "appearance/taskbar-position",
      ),
      #[cfg(feature = "appearance")]
      (
        "appearance.taskbar_icons",
        "taskbar",
        "control_center.taskbar_icons",
        "taskbar barra icones icons audio player spotify launcher search utilities network power profile keyboard layout memory cpu temperature gpu usage",
        "appearance/taskbar-icons",
      ),
      #[cfg(feature = "appearance")]
      (
        "appearance.taskbar_date",
        "taskbar",
        "control_center.taskbar_date",
        "taskbar barra data date format weekday dia mes month year ano",
        "appearance/taskbar-date",
      ),
      #[cfg(feature = "appearance")]
      (
        "appearance.taskbar_time",
        "taskbar",
        "control_center.taskbar_time",
        "taskbar barra hora time clock relogio seconds segundos 24h am pm",
        "appearance/taskbar-time",
      ),
      #[cfg(feature = "appearance")]
      (
        "appearance.widget_telemetry",
        "appearance",
        "control_center.widget_telemetry",
        "widget telemetry system cpu gpu memory storage processes network shortcuts",
        "appearance/widget-telemetry",
      ),
      #[cfg(feature = "appearance")]
      (
        "appearance.control_panel",
        "appearance",
        "control_center.control_panel",
        "control panel painel de controle cards sessions sessoes transparency transparencia",
        "appearance/control-panel",
      ),
    ];
    for (id, category, title_key, keywords, route) in entries {
      let _ = search_registry.register(SearchEntry {
        id: id.into(),
        category: category.into(),
        title: tr(lang, title_key).into(),
        keywords: keywords.split_whitespace().map(str::to_owned).collect(),
        route: route.into(),
      });
    }
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
      jobs: JobManager::default(),
    }
  }

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
  fn home_availability(&self) -> HomeAvailability {
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

  /// Executes the `home_item_count` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn home_item_count(&self) -> usize {
    self.home_destinations().len()
  }

  /// Executes the `move_home` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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

  fn home_category_items(&self) -> Vec<Vec<usize>> {
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
  fn open_destination(&mut self, destination: HomeDestination) {
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
  fn open_appearance_surface(&mut self, page: AppearancePage) {
    self.appearance.page = page;
    self.route = Route::Appearance;
    self.appearance.reload();
  }

  /// Executes the `open_settings` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn open_settings(&mut self, page: Page) {
    self.settings.navigation = argvus_control_center_settings::navigation::Navigation::new(page);
    self.settings.select_current();
    self.settings.enter_page(page);
    self.route = Route::Settings;
  }

  /// Reloads the semantic theme and propagates it to every page.
  ///
  /// Appearance changes are written to the generated theme files by external
  /// scripts, so the process must re-read them instead of reusing the palette
  /// captured at startup.
  pub fn refresh_theme(&mut self) {
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

  /// Executes the `begin_global_search` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn begin_global_search(&mut self) {
    self.search_active = true;
    self.search_cursor_visible = true;
    self.search_cursor_last_blink = Instant::now();
    self.search_query.clear();
    self.search_selected = 0;
    self.search_result_ids.clear();
  }

  /// Applies the `update_global_search` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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

  /// Executes the `pop_global_search` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn pop_global_search(&mut self) {
    let mut query = self.search_query.clone();
    query.pop();
    self.update_global_search(query);
  }

  /// Executes the `move_global_search` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn move_global_search(&mut self, delta: isize) {
    let last = self.search_result_ids.len().saturating_sub(1);
    self.search_selected = (self.search_selected as isize + delta).clamp(0, last as isize) as usize;
  }

  /// Executes the `clear_global_search` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn clear_global_search(&mut self) {
    self.search_active = false;
    self.search_cursor_visible = false;
    self.search_query.clear();
    self.search_selected = 0;
    self.search_result_ids.clear();
  }

  /// Executes the `poll_search_cursor` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn poll_search_cursor(&mut self) -> bool {
    if !self.search_active || self.search_cursor_last_blink.elapsed() < Duration::from_millis(500) {
      return false;
    }
    self.search_cursor_visible = !self.search_cursor_visible;
    self.search_cursor_last_blink = Instant::now();
    true
  }

  /// Executes the `open_global_search_result` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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

  /// Executes the `open_search_route` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn open_search_route(&mut self, route: &str) {
    let Some((domain, page)) = route.split_once('/') else {
      return;
    };
    match (domain, page) {
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
      ("packages", "search") => self.open_packages_page(PackagesPage::Search),
      #[cfg(feature = "packages")]
      ("packages", "home") => self.open_packages_page(PackagesPage::Home),
      #[cfg(feature = "packages")]
      ("packages", "installed") => self.open_packages_page(PackagesPage::Installed),
      #[cfg(feature = "packages")]
      ("packages", "updates") => self.open_packages_page(PackagesPage::Updates),
      #[cfg(feature = "packages")]
      ("packages", "orphans") => self.open_packages_page(PackagesPage::Orphans),
      #[cfg(feature = "packages")]
      ("packages", "cache") => self.open_packages_page(PackagesPage::Cache),
      #[cfg(feature = "packages")]
      ("packages", "aur") => self.open_packages_page(PackagesPage::Aur),
      #[cfg(feature = "packages")]
      ("packages", "history") => self.open_packages_page(PackagesPage::History),
      #[cfg(feature = "packages")]
      ("packages", "downgrade") => self.open_packages_page(PackagesPage::Downgrade),
      #[cfg(feature = "packages")]
      ("packages", "mirrors") => self.open_packages_page(PackagesPage::Mirrors),
      #[cfg(feature = "storage")]
      ("storage", "summary") => self.open_storage_page(StoragePage::Summary),
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

  #[cfg(feature = "hardware")]
  /// Executes the `open_hardware_page` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn open_hardware_page(&mut self, page: HardwarePage) {
    self.hardware.page = page;
    self.route = Route::Hardware;
  }
  #[cfg(feature = "services")]
  /// Executes the `open_services_page` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn open_services_page(&mut self, page: ServicePage) {
    self.services.page = page;
    self.services.reload();
    self.route = Route::Services;
  }
  #[cfg(feature = "network")]
  /// Executes the `open_network_page` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn open_network_page(&mut self, page: NetworkPage) {
    self.network.page = page;
    self.network.reload();
    self.route = Route::Network;
  }
  #[cfg(feature = "audio")]
  /// Executes the `open_audio_page` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn open_audio_page(&mut self, page: AudioPage) {
    self.audio.page = page;
    self.audio.reload();
    self.route = Route::Audio;
  }
  #[cfg(feature = "bluetooth")]
  /// Executes the `open_bluetooth_page` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn open_bluetooth_page(&mut self, page: BluetoothPage) {
    self.bluetooth.page = page;
    self.bluetooth.reload();
    self.route = Route::Bluetooth;
  }
  #[cfg(feature = "boot")]
  /// Executes the `open_boot_page` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn open_boot_page(&mut self, page: BootPage) {
    self.boot.page = page;
    self.boot.reload();
    self.route = Route::Boot;
  }
  #[cfg(feature = "packages")]
  /// Executes the `open_packages_page` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn open_packages_page(&mut self, page: PackagesPage) {
    self.packages.page = page;
    self.packages.reload();
    self.route = Route::Packages;
  }
  #[cfg(feature = "storage")]
  /// Executes the `open_storage_page` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn open_storage_page(&mut self, page: StoragePage) {
    self.storage.page = page;
    self.storage.reload();
    self.route = Route::Storage;
  }
  #[cfg(feature = "diagnostics")]
  /// Executes the `open_diagnostics_page` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn open_diagnostics_page(&mut self, page: DiagnosticPage) {
    self.diagnostics.page = page;
    self.diagnostics.reload();
    self.route = Route::Diagnostics;
  }
  #[cfg(feature = "session")]
  /// Executes the `open_session_page` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn open_session_page(&mut self, page: SessionPage) {
    self.session.page = page;
    self.session.reload();
    self.route = Route::Session;
  }
  #[cfg(feature = "displays")]
  /// Executes the `open_displays_page` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn open_displays_page(&mut self, page: DisplayPage) {
    self.displays.page = page;
    self.displays.reload();
    self.route = Route::Displays;
  }
  #[cfg(feature = "appearance")]
  /// Executes the `open_appearance_page` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn open_appearance_page(&mut self, page: AppearancePage) {
    self.appearance.page = page;
    self.appearance.reload();
    self.route = Route::Appearance;
  }

  /// Executes the `back` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  /// Whether the active page is typing into a text field, a search or a list
  /// filter. While it is, `q` and `?` are text and must not quit or open the
  /// help (decision D10 of `docs/ux-audit.md`); `Ctrl+C` still quits.
  pub fn captures_text(&self) -> bool {
    match self.route {
      Route::Home => self.search_active,
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

  /// Executes the `resize` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn resize(&mut self, width: u16, height: u16) {
    self.width = width;
    self.height = height;
    self.settings.resize(width, height);
    #[cfg(feature = "about")]
    self.about.resize(width, height);
  }

  /// Executes the `help_lines` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn help_lines(&self) -> Vec<String> {
    let mut lines: Vec<&str> = vec![
      tr(self.lang, "control_center.global"),
      tr(self.lang, "control_center.q_quit"),
      tr(self.lang, "control_center.esc_back"),
      tr(self.lang, "control_center.close_help"),
      "",
    ];
    if self.route == Route::Home {
      lines.extend([
        tr(self.lang, "control_center.menu"),
        tr(self.lang, "control_center.navigate"),
        tr(self.lang, "control_center.enter_open"),
        tr(self.lang, "control_center.search"),
        tr(self.lang, "control_center.s_configuration"),
      ]);
    }
    if self.route == Route::Config {
      lines.extend([
        tr(self.lang, "control_center.configuration"),
        tr(self.lang, "control_center.enter_space_toggle"),
        tr(self.lang, "control_center.esc_back_1dce0e"),
      ]);
    }
    if self.route == Route::Settings {
      lines.extend([
        tr(self.lang, "control_center.lists"),
        tr(self.lang, "control_center.navigate"),
        tr(self.lang, "control_center.enter_open_apply"),
        tr(self.lang, "control_center.search"),
      ]);
    }
    #[cfg(feature = "about")]
    if self.route == Route::About {
      lines.extend([
        tr(self.lang, "control_center.about_4cf69b"),
        tr(self.lang, "control_center.tabs"),
        tr(self.lang, "control_center.navigate_scroll"),
        tr(self.lang, "control_center.pgup_pgdn_scroll"),
        tr(self.lang, "control_center.enter_open_action"),
      ]);
    }
    #[cfg(any(
      feature = "hardware",
      feature = "services",
      feature = "dev-services",
      feature = "network",
      feature = "audio",
      feature = "bluetooth",
      feature = "boot",
      feature = "packages",
      feature = "storage",
      feature = "diagnostics",
      feature = "power",
      feature = "session",
      feature = "displays"
    ))]
    {
      let mut domain = false;
      #[cfg(feature = "hardware")]
      {
        domain |= self.route == Route::Hardware;
      }
      #[cfg(feature = "services")]
      {
        domain |= self.route == Route::Services;
      }
      #[cfg(feature = "dev-services")]
      {
        domain |= self.route == Route::DevServices;
      }
      #[cfg(feature = "network")]
      {
        domain |= self.route == Route::Network;
      }
      #[cfg(feature = "audio")]
      {
        domain |= self.route == Route::Audio;
      }
      #[cfg(feature = "bluetooth")]
      {
        domain |= self.route == Route::Bluetooth;
      }
      #[cfg(feature = "boot")]
      {
        domain |= self.route == Route::Boot;
      }
      #[cfg(feature = "packages")]
      {
        domain |= self.route == Route::Packages;
      }
      #[cfg(feature = "storage")]
      {
        domain |= self.route == Route::Storage;
      }
      #[cfg(feature = "diagnostics")]
      {
        domain |= self.route == Route::Diagnostics;
      }
      #[cfg(feature = "power")]
      {
        domain |= self.route == Route::Power;
      }
      #[cfg(feature = "session")]
      {
        domain |= self.route == Route::Session;
      }
      #[cfg(feature = "displays")]
      {
        domain |= self.route == Route::Displays;
      }
      if domain {
        lines.extend([
          tr(self.lang, "control_center.navigate"),
          tr(self.lang, "control_center.enter_open"),
          tr(self.lang, "control_center.esc_back_bd56d7"),
          tr(self.lang, "control_center.r_refresh"),
        ]);
      }
    }
    lines.into_iter().map(str::to_string).collect()
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
  /// Executes the `home_navigation_is_bounded` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn home_navigation_is_bounded() {
    let mut app = App::new(InitialRoute::Home);
    app.move_home(-1);
    assert_eq!(app.home_selected, 0);
    app.move_home(1_000);
    assert_eq!(app.home_selected, app.home_item_count().saturating_sub(1));
  }

  #[test]
  /// Executes the `home_navigation_never_selects_a_header` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
  /// Executes the `entering_and_leaving_about_preserves_tab` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
  /// Style is reached from the hub and Esc returns to the hub, not to the Home.
  fn style_returns_to_the_settings_hub() {
    let mut app = App::new(InitialRoute::Home);
    app.route = Route::Config;
    app.back();
    assert_eq!(app.route, Route::AppSettings);
  }
}
