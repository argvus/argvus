use super::*;

fn args(values: &[&str]) -> Vec<String> {
  values.iter().map(ToString::to_string).collect()
}

#[cfg(all(
  feature = "apps",
  feature = "fonts",
  feature = "locale",
  feature = "system",
  feature = "network",
  feature = "about"
))]
#[test]
fn parses_direct_routes() {
  assert_eq!(
    parse(&args(&["apps"])).unwrap(),
    Some(InitialRoute::Settings(Page::DefaultApps))
  );
  assert_eq!(
    parse(&args(&["fonts"])).unwrap(),
    Some(InitialRoute::Settings(Page::Fonts))
  );
  assert_eq!(
    parse(&args(&["locale", "--keyboard"])).unwrap(),
    Some(InitialRoute::Settings(Page::Keyboard))
  );
  assert_eq!(
    parse(&args(&["input"])).unwrap(),
    Some(InitialRoute::Settings(Page::MouseTouchpad))
  );
  assert_eq!(
    parse(&args(&["system", "hostname"])).unwrap(),
    Some(InitialRoute::Settings(Page::Hostname))
  );
  assert_eq!(
    parse(&args(&["network", "firewall"])).unwrap(),
    Some(InitialRoute::Settings(Page::Firewall))
  );
  #[cfg(feature = "network")]
  for (page, expected) in [
    ("connections", NetworkPage::Connections),
    ("activate", NetworkPage::Activate),
    ("add", NetworkPage::AddConnection),
    ("--add", NetworkPage::AddConnection),
  ] {
    assert_eq!(
      parse(&args(&["network", page])).unwrap(),
      Some(InitialRoute::Network(expected))
    );
  }
  assert_eq!(
    parse(&args(&["about", "credits"])).unwrap(),
    Some(InitialRoute::About(Tab::Credits))
  );
}

#[cfg(all(
  feature = "apps",
  feature = "fonts",
  feature = "locale",
  feature = "language",
  feature = "system"
))]
#[test]
fn maps_every_settings_page() {
  let cases = [
    (
      vec!["apps", "terminal"],
      Page::AppSelector(Category::Terminal),
    ),
    (
      vec!["apps", "file-manager"],
      Page::AppSelector(Category::FileManager),
    ),
    (
      vec!["apps", "text-editor"],
      Page::AppSelector(Category::TextEditor),
    ),
    (
      vec!["apps", "terminal-editor"],
      Page::AppSelector(Category::TerminalEditor),
    ),
    (
      vec!["apps", "browser"],
      Page::AppSelector(Category::Browser),
    ),
    (
      vec!["apps", "image-viewer"],
      Page::AppSelector(Category::ImageViewer),
    ),
    (
      vec!["apps", "pdf-viewer"],
      Page::AppSelector(Category::PdfViewer),
    ),
    (
      vec!["apps", "video-player"],
      Page::AppSelector(Category::VideoPlayer),
    ),
    (
      vec!["apps", "audio-player"],
      Page::AppSelector(Category::AudioPlayer),
    ),
    (
      vec!["apps", "archive"],
      Page::AppSelector(Category::Archive),
    ),
    (
      vec!["apps", "launcher"],
      Page::AppSelector(Category::Launcher),
    ),
    (
      vec!["fonts", "taskbar"],
      Page::FontSelector(FontTarget::Taskbar),
    ),
    (
      vec!["fonts", "widget-telemetry"],
      Page::FontSelector(FontTarget::Sysinfo),
    ),
    (
      vec!["fonts", "control-panel"],
      Page::FontSelector(FontTarget::ControlPanel),
    ),
    (
      vec!["fonts", "system"],
      Page::FontSelector(FontTarget::System),
    ),
    (vec!["fonts", "apps"], Page::FontSelector(FontTarget::Apps)),
    (
      vec!["fonts", "terminal"],
      Page::FontSelector(FontTarget::Terminal),
    ),
    (
      vec!["fonts", "browser"],
      Page::FontSelector(FontTarget::Browser),
    ),
    (
      vec!["fonts", "antialiasing"],
      Page::SettingSelector(SettingKind::Antialiasing),
    ),
    (
      vec!["fonts", "hinting"],
      Page::SettingSelector(SettingKind::Hinting),
    ),
    (
      vec!["fonts", "subpixel"],
      Page::SettingSelector(SettingKind::Subpixel),
    ),
    (
      vec!["fonts", "dpi"],
      Page::SettingSelector(SettingKind::Dpi),
    ),
    (vec!["locale", "timezone"], Page::TimeZone),
    (vec!["locale", "date-time"], Page::DateTime),
    (vec!["locale", "regional-locale"], Page::RegionalLocale),
    (vec!["locale", "system-locales"], Page::SystemLocales),
    (vec!["locale", "keyboard"], Page::Keyboard),
    (vec!["locale", "keyboard-layout"], Page::KeyboardLayout),
    (vec!["locale", "keyboard-variant"], Page::KeyboardVariant),
    (vec!["locale", "console-keymap"], Page::ConsoleKeymap),
    (vec!["language"], Page::Language),
    (vec!["system"], Page::Hostname),
    (vec!["system", "hostname"], Page::Hostname),
    (vec!["system", "firewall"], Page::Firewall),
    (vec!["system", "--users"], Page::Users),
    (vec!["system", "groups"], Page::Groups),
  ];

  for (arguments, expected) in cases {
    assert_eq!(
      parse(&args(&arguments)).unwrap(),
      Some(InitialRoute::Settings(expected)),
      "arguments: {arguments:?}"
    );
  }
}

#[test]
fn rejects_invalid_routes() {
  assert!(parse(&args(&["about", "unknown"])).is_err());
  assert!(parse(&args(&["fonts", "extra"])).is_err());
}

#[cfg(feature = "power")]
#[test]
fn parses_the_power_route() {
  assert_eq!(parse(&args(&["power"])).unwrap(), Some(InitialRoute::Power));
}

#[cfg(feature = "session")]
#[test]
fn parses_every_session_page() {
  let cases = [
    (vec!["session"], SessionPage::Home),
    (vec!["session", "summary"], SessionPage::Home),
    (vec!["session", "components"], SessionPage::Components),
    (vec!["session", "autostart"], SessionPage::Autostart),
    (vec!["session", "diagnostics"], SessionPage::Diagnostics),
    (vec!["session", "logs"], SessionPage::Logs),
  ];
  for (arguments, expected) in cases {
    assert_eq!(
      parse(&args(&arguments)).unwrap(),
      Some(InitialRoute::Session(expected)),
      "arguments: {arguments:?}"
    );
  }
  assert!(parse(&args(&["session", "unknown"])).is_err());
}

#[cfg(feature = "displays")]
#[test]
fn parses_displays_pages() {
  assert!(matches!(
    parse(&args(&["displays", "night-light"])).unwrap(),
    Some(InitialRoute::Displays(DisplayPage::NightLight))
  ));
  assert_eq!(
    parse(&args(&["displays"])).unwrap(),
    Some(InitialRoute::Displays(DisplayPage::Home))
  );
  assert_eq!(
    parse(&args(&["monitors", "vrr"])).unwrap(),
    Some(InitialRoute::Displays(DisplayPage::Picker {
      monitor: 0,
      setting: MonitorSetting::Vrr,
    }))
  );
  assert_eq!(
    parse(&args(&["displays", "orientation"])).unwrap(),
    Some(InitialRoute::Displays(DisplayPage::Picker {
      monitor: 0,
      setting: MonitorSetting::Orientation,
    }))
  );
  assert!(parse(&args(&["displays", "unknown"])).is_err());
}

#[cfg(feature = "appearance")]
#[test]
fn parses_appearance_pages() {
  assert_eq!(
    parse(&args(&["appearance"])).unwrap(),
    Some(InitialRoute::Appearance(AppearancePage::Home))
  );
  assert_eq!(
    parse(&args(&["appearance", "themes"])).unwrap(),
    Some(InitialRoute::Appearance(AppearancePage::Themes))
  );
  assert_eq!(
    parse(&args(&["appearance", "wallpapers"])).unwrap(),
    Some(InitialRoute::Appearance(AppearancePage::Wallpapers))
  );
  assert_eq!(
    parse(&args(&["appearance", "icons"])).unwrap(),
    Some(InitialRoute::Appearance(AppearancePage::Icons))
  );
  assert_eq!(
    parse(&args(&["appearance", "accent"])).unwrap(),
    Some(InitialRoute::Appearance(AppearancePage::Accents))
  );
  assert_eq!(
    parse(&args(&["appearance", "taskbar"])).unwrap(),
    Some(InitialRoute::Appearance(AppearancePage::Taskbar))
  );
  assert_eq!(
    parse(&args(&["appearance", "taskbar-position"])).unwrap(),
    Some(InitialRoute::Appearance(AppearancePage::TaskbarPosition))
  );
  assert_eq!(
    parse(&args(&["appearance", "position"])).unwrap(),
    Some(InitialRoute::Appearance(AppearancePage::TaskbarPosition))
  );
  assert_eq!(
    parse(&args(&["appearance", "taskbar-height"])).unwrap(),
    Some(InitialRoute::Appearance(AppearancePage::TaskbarHeight))
  );
  assert_eq!(
    parse(&args(&["appearance", "taskbar-border"])).unwrap(),
    Some(InitialRoute::Appearance(AppearancePage::TaskbarBorder))
  );
  assert_eq!(
    parse(&args(&["appearance", "taskbar-group"])).unwrap(),
    Some(InitialRoute::Appearance(AppearancePage::Taskbar))
  );
  assert_eq!(
    parse(&args(&["appearance", "utility-group"])).unwrap(),
    Some(InitialRoute::Appearance(AppearancePage::Taskbar))
  );
  assert_eq!(
    parse(&args(&["appearance", "widget-telemetry"])).unwrap(),
    Some(InitialRoute::Appearance(AppearancePage::WidgetTelemetry))
  );
  assert!(parse(&args(&["appearance", "unknown"])).is_err());
}

#[test]
fn parses_the_config_route() {
  assert_eq!(
    parse(&args(&["config"])).unwrap(),
    Some(InitialRoute::Config)
  );
}

#[cfg(feature = "diagnostics")]
#[test]
fn system_doctor_and_legacy_routes_share_all_pages() {
  for route in ["system-doctor", "diagnostics", "diagnostic"] {
    for (selector, page) in [
      ("summary", DiagnosticPage::Home),
      ("services", DiagnosticPage::Services),
      ("boot", DiagnosticPage::Boot),
      ("graphics", DiagnosticPage::Graphics),
      ("network", DiagnosticPage::Network),
      ("audio", DiagnosticPage::Audio),
      ("bluetooth", DiagnosticPage::Bluetooth),
      ("storage", DiagnosticPage::Storage),
      ("packages", DiagnosticPage::Packages),
      ("argvus", DiagnosticPage::Argvus),
    ] {
      assert_eq!(
        parse(&args(&[route, selector])).unwrap(),
        Some(InitialRoute::Diagnostics(page))
      );
    }
  }
}

#[cfg(feature = "snapshot")]
#[test]
fn snapshot_route_has_no_arbitrary_page_or_action_arguments() {
  assert_eq!(
    parse(&args(&["snapshot"])).unwrap(),
    Some(InitialRoute::Snapshot)
  );
  assert!(parse(&args(&["snapshot", "delete", "1"])).is_err());
}
