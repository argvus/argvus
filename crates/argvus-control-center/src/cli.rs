//! Argument parsing and startup route selection.
use anyhow::{Result, bail};
#[cfg(feature = "about")]
use argvus_control_center_about::Tab;
#[cfg(feature = "appearance")]
use argvus_control_center_appearance::AppearancePage;
#[cfg(feature = "audio")]
use argvus_control_center_audio::AudioPage;
#[cfg(feature = "bluetooth")]
use argvus_control_center_bluetooth::BluetoothPage;
#[cfg(feature = "boot")]
use argvus_control_center_boot::BootPage;
#[cfg(feature = "diagnostics")]
use argvus_control_center_diagnostics::DiagnosticPage;
#[cfg(feature = "displays")]
use argvus_control_center_displays::{DisplayPage, MonitorSetting};
#[cfg(feature = "hardware")]
use argvus_control_center_hardware::HardwarePage;
#[cfg(feature = "network")]
use argvus_control_center_network::NetworkPage;
#[cfg(feature = "packages")]
use argvus_control_center_packages::{PackageSource, PackageTask, PackagesPage};
#[cfg(feature = "services")]
use argvus_control_center_services::ServicePage;
#[cfg(feature = "session")]
use argvus_control_center_session::SessionPage;
#[cfg(feature = "apps")]
use argvus_control_center_settings::Category;
use argvus_control_center_settings::Page;
#[cfg(feature = "fonts")]
use argvus_control_center_settings::{FontTarget, SettingKind};
#[cfg(feature = "storage")]
use argvus_control_center_storage::StoragePage;

use crate::app::InitialRoute;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn parse_or_print() -> Result<Option<InitialRoute>> {
  let args: Vec<String> = std::env::args().skip(1).collect();
  match parse(&args) {
    Ok(Some(route)) => Ok(Some(route)),
    Ok(None) => Ok(None),
    Err(error) => {
      eprintln!("argvus-control-center: {error}\n");
      print_help();
      bail!(error)
    }
  }
}

pub fn parse(args: &[String]) -> Result<Option<InitialRoute>, String> {
  if args.is_empty() {
    return Ok(Some(InitialRoute::Home));
  }
  match args[0].as_str() {
    "-h" | "--help" if args.len() == 1 => {
      print_help();
      Ok(None)
    }
    "-V" | "--version" if args.len() == 1 => {
      println!("argvus-control-center {VERSION}");
      Ok(None)
    }
    #[cfg(feature = "apps")]
    "apps" | "default-apps" => parse_apps(&args[1..]).map(settings_route),
    #[cfg(feature = "fonts")]
    "fonts" => parse_fonts(&args[1..]).map(settings_route),
    #[cfg(feature = "locale")]
    "locale" | "locale-region" => parse_locale(&args[1..]).map(settings_route),
    "input" | "mouse" | "touchpad" if args.len() == 1 => Ok(settings_route(Page::MouseTouchpad)),
    #[cfg(feature = "apps")]
    "window-rules" | "window_rules" if args.len() == 1 => Ok(settings_route(Page::WindowRules)),
    #[cfg(feature = "apps")]
    "projects" if args.len() == 1 => Ok(settings_route(Page::Projects)),
    #[cfg(feature = "apps")]
    "snippets" if args.len() == 1 => Ok(settings_route(Page::Snippets)),
    #[cfg(feature = "apps")]
    "ssh-keys" | "ssh_keys" if args.len() == 1 => Ok(settings_route(Page::SshKeys)),
    "gpg-keys" | "gpg_keys" if args.len() == 1 => Ok(settings_route(Page::GpgKeys)),
    #[cfg(feature = "system")]
    "firewall" if args.len() == 1 => Ok(settings_route(Page::Firewall)),
    "keybindings" | "shortcuts" | "keyboard" if args.len() == 1 => {
      Ok(settings_route(Page::Keybindings))
    }
    #[cfg(feature = "language")]
    "language" if args.len() == 1 => Ok(settings_route(Page::Language)),
    "config" if args.len() == 1 => Ok(Some(InitialRoute::Config)),
    #[cfg(feature = "system")]
    "system" => parse_system(&args[1..]).map(settings_route),
    #[cfg(feature = "hardware")]
    "hardware" => Ok(Some(InitialRoute::Hardware(parse_hardware(&args[1..])?))),
    #[cfg(feature = "services")]
    "services" | "systemd" => Ok(Some(InitialRoute::Services(parse_services(&args[1..])?))),
    #[cfg(feature = "dev-services")]
    "dev-services" | "dev_services" => Ok(Some(InitialRoute::DevServices)),
    // Firewall moved from Network to Security. The old route keeps working and
    // opens the same page in its new place.
    #[cfg(feature = "system")]
    "network" if args.len() == 2 && args[1] == "firewall" => Ok(settings_route(Page::Firewall)),
    #[cfg(feature = "network")]
    "network" => Ok(Some(InitialRoute::Network(parse_network(&args[1..])?))),
    #[cfg(feature = "audio")]
    "audio" => Ok(Some(InitialRoute::Audio(parse_audio(&args[1..])?))),
    #[cfg(feature = "bluetooth")]
    "bluetooth" => Ok(Some(InitialRoute::Bluetooth(parse_bluetooth(&args[1..])?))),
    #[cfg(feature = "boot")]
    "boot" => Ok(Some(InitialRoute::Boot(parse_boot(&args[1..])?))),
    #[cfg(feature = "packages")]
    "packages" | "pacman" => Ok(Some(InitialRoute::Packages(parse_packages(&args[1..])?))),
    #[cfg(feature = "storage")]
    "storage" => Ok(Some(InitialRoute::Storage(parse_storage(&args[1..])?))),
    #[cfg(feature = "diagnostics")]
    "system-doctor" | "diagnostics" | "diagnostic" => Ok(Some(InitialRoute::Diagnostics(
      parse_diagnostics(&args[1..])?,
    ))),
    #[cfg(feature = "about")]
    "about" if args.len() == 1 => Ok(Some(InitialRoute::About(Tab::System))),
    #[cfg(feature = "about")]
    "about" if args.len() == 2 => {
      parse_about_tab(&args[1]).map(|tab| Some(InitialRoute::About(tab)))
    }
    #[cfg(feature = "power")]
    "power" if args.len() == 1 => Ok(Some(InitialRoute::Power)),
    #[cfg(feature = "session")]
    "session" => Ok(Some(InitialRoute::Session(parse_session(&args[1..])?))),
    #[cfg(feature = "displays")]
    "displays" | "monitors" => Ok(Some(InitialRoute::Displays(parse_displays(&args[1..])?))),
    #[cfg(feature = "appearance")]
    "appearance" => Ok(Some(InitialRoute::Appearance(parse_appearance(
      &args[1..],
    )?))),
    value => Err(format!("unknown argument '{value}'")),
  }
}

#[cfg(feature = "appearance")]
fn parse_appearance(args: &[String]) -> Result<AppearancePage, String> {
  if args.len() > 1 {
    return Err("appearance accepts at most one page".into());
  }
  Ok(match args.first().map(|v| option(v)) {
    None | Some("summary") | Some("home") => AppearancePage::Home,
    Some("themes") => AppearancePage::Themes,
    Some("wallpapers") => AppearancePage::Wallpapers,
    Some("icons") | Some("icon") => AppearancePage::Icons,
    Some("accents") | Some("accent") | Some("color") => AppearancePage::Accents,
    Some("taskbar") | Some("taskbar-group") | Some("utility-group") => AppearancePage::Taskbar,
    Some("taskbar-position") | Some("position") => AppearancePage::TaskbarPosition,
    Some("taskbar-icons") => AppearancePage::TaskbarIcons,
    Some("taskbar-date") => AppearancePage::TaskbarDate,
    Some("taskbar-time") => AppearancePage::TaskbarTime,
    Some("taskbar-height") | Some("height") => AppearancePage::TaskbarHeight,
    Some("taskbar-border") | Some("border") => AppearancePage::TaskbarBorder,
    Some("widget-telemetry") | Some("telemetry") => AppearancePage::WidgetTelemetry,
    Some("spaces") | Some("window-spaces") => AppearancePage::WindowSpaces,
    Some("borders") => AppearancePage::Borders,
    Some("borders-shadow") | Some("shadow") => AppearancePage::BordersShadow,
    Some(v) => return Err(format!("unknown Appearance page '{v}'")),
  })
}

#[cfg(feature = "storage")]
fn parse_storage(args: &[String]) -> Result<StoragePage, String> {
  if args.len() > 1 {
    return Err("storage accepts at most one page".into());
  }
  Ok(match args.first().map(|v| option(v)) {
    None | Some("summary") => StoragePage::Home,
    Some("disks") => StoragePage::Disks,
    Some("partitions") => StoragePage::Partitions,
    Some("filesystems") => StoragePage::Filesystems,
    Some("mounts") => StoragePage::Mounts,
    Some("smart") => StoragePage::Smart,
    Some("usage") => StoragePage::Usage,
    Some(v) => return Err(format!("unknown Storage page '{v}'")),
  })
}

#[cfg(feature = "session")]
fn parse_session(args: &[String]) -> Result<SessionPage, String> {
  if args.len() > 1 {
    return Err("session accepts at most one page".into());
  }
  Ok(match args.first().map(|v| option(v)) {
    None | Some("summary") => SessionPage::Home,
    Some("components") => SessionPage::Components,
    Some("autostart") => SessionPage::Autostart,
    Some("diagnostics") => SessionPage::Diagnostics,
    Some("logs") => SessionPage::Logs,
    Some(v) => return Err(format!("unknown Session page '{v}'")),
  })
}

#[cfg(feature = "displays")]
fn parse_displays(args: &[String]) -> Result<DisplayPage, String> {
  if args.len() > 1 {
    return Err("displays accepts at most one page".into());
  }
  Ok(match args.first().map(|v| option(v)) {
    None | Some("summary") | Some("home") => DisplayPage::Home,
    Some("night-light") => DisplayPage::NightLight,
    Some("res") | Some("resolution") => DisplayPage::Picker {
      monitor: 0,
      setting: MonitorSetting::Resolution,
    },
    Some("refresh") | Some("refresh-rate") => DisplayPage::Picker {
      monitor: 0,
      setting: MonitorSetting::RefreshRate,
    },
    Some("scale") => DisplayPage::Picker {
      monitor: 0,
      setting: MonitorSetting::Scale,
    },
    Some("position") => DisplayPage::Picker {
      monitor: 0,
      setting: MonitorSetting::Position,
    },
    Some("orientation") => DisplayPage::Picker {
      monitor: 0,
      setting: MonitorSetting::Orientation,
    },
    Some("primary") => DisplayPage::Picker {
      monitor: 0,
      setting: MonitorSetting::Primary,
    },
    Some("vrr") => DisplayPage::Picker {
      monitor: 0,
      setting: MonitorSetting::Vrr,
    },
    Some("hdr") => DisplayPage::Picker {
      monitor: 0,
      setting: MonitorSetting::Hdr,
    },
    Some(v) => return Err(format!("unknown Displays page '{v}'")),
  })
}

#[cfg(feature = "diagnostics")]
fn parse_diagnostics(args: &[String]) -> Result<DiagnosticPage, String> {
  if args.len() > 1 {
    return Err("diagnostics accepts at most one page".into());
  }
  Ok(match args.first().map(|v| option(v)) {
    None | Some("summary") => DiagnosticPage::Home,
    Some("services") => DiagnosticPage::Services,
    Some("boot") => DiagnosticPage::Boot,
    Some("graphics") => DiagnosticPage::Graphics,
    Some("network") => DiagnosticPage::Network,
    Some("audio") => DiagnosticPage::Audio,
    Some("bluetooth") => DiagnosticPage::Bluetooth,
    Some("storage") => DiagnosticPage::Storage,
    Some("packages") => DiagnosticPage::Packages,
    Some("argvus") => DiagnosticPage::Argvus,
    Some(v) => return Err(format!("unknown Diagnostics page '{v}'")),
  })
}

#[cfg(feature = "hardware")]
fn parse_hardware(args: &[String]) -> Result<HardwarePage, String> {
  if args.len() > 1 {
    return Err("hardware accepts at most one page".into());
  }
  Ok(match args.first().map(|v| option(v)) {
    None | Some("summary") => HardwarePage::Summary,
    Some("cpu") => HardwarePage::Cpu,
    Some("gpu") => HardwarePage::Gpu,
    Some("memory") => HardwarePage::Memory,
    Some("power") => HardwarePage::Power,
    Some("devices") => HardwarePage::Devices,
    Some(v) => return Err(format!("unknown Hardware page '{v}'")),
  })
}

#[cfg(feature = "services")]
fn parse_services(args: &[String]) -> Result<ServicePage, String> {
  if args.len() > 1 {
    return Err("services accepts at most one page".into());
  }
  Ok(match args.first().map(|v| option(v)) {
    None | Some("system") => ServicePage::System,
    Some("user") => ServicePage::User,
    Some("failed") => ServicePage::Failed,
    Some("logs") => ServicePage::Logs,
    Some(v) => return Err(format!("unknown Services page '{v}'")),
  })
}

#[cfg(feature = "network")]
fn parse_network(args: &[String]) -> Result<NetworkPage, String> {
  if args.len() > 1 {
    return Err("network accepts at most one page".into());
  }
  Ok(match args.first().map(|v| option(v)) {
    None | Some("status") => NetworkPage::Status,
    Some("interfaces") => NetworkPage::Interfaces,
    Some("wifi") => NetworkPage::Wifi,
    Some("ethernet") => NetworkPage::Ethernet,
    Some("vpn") => NetworkPage::Vpn,
    Some("dns") => NetworkPage::Dns,
    Some("proxy") => NetworkPage::Proxy,
    Some("connections") => NetworkPage::Connections,
    Some("activate") => NetworkPage::Activate,
    Some("add") => NetworkPage::AddConnection,
    Some(v) => return Err(format!("unknown Network page '{v}'")),
  })
}
#[cfg(feature = "audio")]
fn parse_audio(args: &[String]) -> Result<AudioPage, String> {
  if args.len() > 1 {
    return Err("audio accepts at most one page".into());
  }
  Ok(match args.first().map(|v| option(v)) {
    None | Some("output") => AudioPage::Output,
    Some("input") => AudioPage::Input,
    Some("devices") => AudioPage::Devices,
    Some(v) => return Err(format!("unknown Audio page '{v}'")),
  })
}
#[cfg(feature = "bluetooth")]
fn parse_bluetooth(args: &[String]) -> Result<BluetoothPage, String> {
  if args.len() > 1 {
    return Err("bluetooth accepts at most one page".into());
  }
  Ok(match args.first().map(|v| option(v)) {
    None | Some("state") => BluetoothPage::State,
    Some("devices") => BluetoothPage::Devices,
    Some("pair") => BluetoothPage::Pair,
    Some(v) => return Err(format!("unknown Bluetooth page '{v}'")),
  })
}

#[cfg(feature = "boot")]
fn parse_boot(args: &[String]) -> Result<BootPage, String> {
  if args.len() > 1 {
    return Err("boot accepts at most one page".into());
  }
  Ok(match args.first().map(|v| option(v)) {
    None | Some("summary") => BootPage::Summary,
    Some("kernel") => BootPage::Kernel,
    Some("bootloader") => BootPage::Bootloader,
    Some("initramfs") => BootPage::Initramfs,
    Some("plymouth") => BootPage::Plymouth,
    Some(v) => return Err(format!("unknown Boot page '{v}'")),
  })
}

#[cfg(feature = "packages")]
fn parse_packages(args: &[String]) -> Result<PackagesPage, String> {
  if args.len() > 1 {
    return Err("packages accepts at most one page".into());
  }
  use {PackageSource::*, PackageTask::*};
  // `search`, `installed`, `updates` and `aur` keep opening the lists they
  // named before the pages were grouped by source.
  Ok(match args.first().map(|v| option(v)) {
    None | Some("search") => PackagesPage::Task(Official, Install),
    Some("installed") => PackagesPage::Task(Official, Remove),
    Some("updates") => PackagesPage::Task(Official, Update),
    Some("official") => PackagesPage::Source(Official),
    Some("orphans") => PackagesPage::Orphans,
    Some("cache") => PackagesPage::Cache,
    Some("aur") => PackagesPage::Task(Aur, Install),
    Some("aur-remove") => PackagesPage::Task(Aur, Remove),
    Some("aur-updates") => PackagesPage::Task(Aur, Update),
    Some("repositories") => PackagesPage::Repositories,
    Some("repositories-add") => PackagesPage::RepositoryAdd,
    Some("repositories-list") => PackagesPage::RepositoryList,
    Some("history") => PackagesPage::History,
    Some("downgrade") => PackagesPage::Downgrade,
    Some("mirrors") => PackagesPage::Mirrors,
    Some(v) => return Err(format!("unknown Packages page '{v}'")),
  })
}

fn settings_route(page: Page) -> Option<InitialRoute> {
  Some(InitialRoute::Settings(page))
}

#[cfg(any(
  feature = "apps",
  feature = "fonts",
  feature = "locale",
  feature = "system",
  feature = "storage",
  feature = "diagnostics",
  feature = "hardware",
  feature = "services",
  feature = "network",
  feature = "audio",
  feature = "bluetooth",
  feature = "boot",
  feature = "packages",
  feature = "session",
  feature = "displays",
  feature = "appearance"
))]
fn option(value: &str) -> &str {
  value.strip_prefix("--").unwrap_or(value)
}

#[cfg(feature = "apps")]
fn parse_apps(args: &[String]) -> Result<Page, String> {
  if args.is_empty() {
    return Ok(Page::DefaultApps);
  }
  if args.len() != 1 {
    return Err("apps accepts at most one page".to_string());
  }
  let category = match option(&args[0]) {
    "terminal" => Category::Terminal,
    "file-manager" => Category::FileManager,
    "text-editor" => Category::TextEditor,
    "terminal-editor" => Category::TerminalEditor,
    "browser" => Category::Browser,
    "image-viewer" => Category::ImageViewer,
    "pdf-viewer" => Category::PdfViewer,
    "video-player" => Category::VideoPlayer,
    "audio-player" => Category::AudioPlayer,
    "archive" => Category::Archive,
    "launcher" => Category::Launcher,
    value => return Err(format!("unknown Apps page '{value}'")),
  };
  Ok(Page::AppSelector(category))
}

#[cfg(feature = "fonts")]
fn parse_fonts(args: &[String]) -> Result<Page, String> {
  if args.is_empty() {
    return Ok(Page::Fonts);
  }
  if args.len() != 1 {
    return Err("fonts accepts at most one page".to_string());
  }
  let page = match option(&args[0]) {
    "taskbar" => Page::FontSelector(FontTarget::Taskbar),
    "widget-telemetry" | "desktop-telemetry" | "sysinfo" => Page::FontSelector(FontTarget::Sysinfo),
    "control-panel" => Page::FontSelector(FontTarget::ControlPanel),
    "system" => Page::FontSelector(FontTarget::System),
    "apps" => Page::FontSelector(FontTarget::Apps),
    "terminal" => Page::FontSelector(FontTarget::Terminal),
    "browser" => Page::FontSelector(FontTarget::Browser),
    "antialiasing" => Page::SettingSelector(SettingKind::Antialiasing),
    "hinting" => Page::SettingSelector(SettingKind::Hinting),
    "subpixel" => Page::SettingSelector(SettingKind::Subpixel),
    "dpi" => Page::SettingSelector(SettingKind::Dpi),
    value => return Err(format!("unknown Fonts page '{value}'")),
  };
  Ok(page)
}

#[cfg(feature = "locale")]
fn parse_locale(args: &[String]) -> Result<Page, String> {
  if args.is_empty() {
    return Ok(Page::LocaleRegion);
  }
  if args.len() != 1 {
    return Err("locale accepts at most one page".to_string());
  }
  match option(&args[0]) {
    "timezone" => Ok(Page::TimeZone),
    "date-time" => Ok(Page::DateTime),
    "regional-locale" => Ok(Page::RegionalLocale),
    "system-locales" => Ok(Page::SystemLocales),
    "keyboard" => Ok(Page::Keyboard),
    "keyboard-layout" => Ok(Page::KeyboardLayout),
    "keyboard-variant" => Ok(Page::KeyboardVariant),
    "console-keymap" => Ok(Page::ConsoleKeymap),
    value => Err(format!("unknown Locale & Region page '{value}'")),
  }
}

#[cfg(feature = "system")]
fn parse_system(args: &[String]) -> Result<Page, String> {
  if args.is_empty() {
    return Ok(Page::Hostname);
  }
  if args.len() != 1 {
    return Err("system accepts at most one page".to_string());
  }
  match option(&args[0]) {
    "hostname" => Ok(Page::Hostname),
    "firewall" => Ok(Page::Firewall),
    "users" => Ok(Page::Users),
    "groups" => Ok(Page::Groups),
    value => Err(format!("unknown System page '{value}'")),
  }
}

#[cfg(feature = "about")]
fn parse_about_tab(value: &str) -> Result<Tab, String> {
  match value {
    "system" => Ok(Tab::System),
    "about" => Ok(Tab::About),
    "donate" => Ok(Tab::Donate),
    "credits" => Ok(Tab::Credits),
    "copyright" => Ok(Tab::Copyright),
    _ => Err(format!("unknown About tab '{value}'")),
  }
}

pub fn print_help() {
  println!(
    "argvus-control-center - keyboard-first ARGVUS control center\n\n\
     Usage:\n  argvus-control-center [apps|fonts|locale|language|config|hardware|services|system|power|session|displays|appearance|system-doctor|about [TAB]]\n  argvus-control-center system-settings <domain> <action> [value...]\n\n\
     Page selectors:\n  apps [terminal|file-manager|text-editor|terminal-editor|browser|image-viewer|pdf-viewer|video-player|audio-player|archive|launcher]\n  fonts [taskbar|widget-telemetry|control-panel|system|apps|terminal|browser|antialiasing|hinting|subpixel|dpi]\n  locale [timezone|date-time|regional-locale|system-locales|keyboard|keyboard-layout|keyboard-variant|console-keymap]\n  system [hostname|users|groups]\n  network [status|connections|activate|add|interfaces|wifi|ethernet|vpn|dns|proxy]\n  session [summary|components|autostart|diagnostics|logs]\n  system-doctor [summary|services|boot|graphics|network|audio|bluetooth|storage|packages|argvus]\n  displays [summary|night-light|res|refresh|scale|position|orientation|primary|vrr|hdr]\n  appearance [summary|themes|wallpapers|icons|accents|taskbar]\n\n\
     About tabs:\n  system, about, donate, credits, copyright\n\n\
     Options:\n  -h, --help       Print help\n  -V, --version    Print version\n\n\
     Global keys:\n  q                Quit\n  Esc              Back\n  ?                Contextual help"
  );
}

#[cfg(test)]
mod tests;
