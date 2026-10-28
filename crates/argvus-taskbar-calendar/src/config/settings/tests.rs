use super::*;

#[test]
fn missing_config_uses_defaults() {
  let root = std::env::temp_dir().join(format!("argvus-test-{}", uuid::Uuid::new_v4()));
  let paths = Paths {
    data_dir: root.join("data"),
    state_dir: root.join("state"),
    cache_dir: root.join("cache"),
    database: root.join("data/calendar.db"),
    config_file: root.join("config/config.toml"),
    legacy_settings_file: root.join("config/settings.toml"),
    user_config_file: root.join("user/config.toml"),
    user_style: root.join("config/style.css"),
    theme_file: root.join("config/theme.css"),
    cache_theme_file: root.join("cache/theme.css"),
    active_argvus_theme_file: root.join("config/argvus/data/.active-theme"),
    events_enabled_file: root.join("state/events-enabled"),
  };
  let config = AppConfig::load(&paths).unwrap();
  assert_eq!(config.locale.language, "auto");
  assert_eq!(config.appearance.font_size, 12);
  assert!(!config.calendar.show_events);
}

#[test]
fn valid_config_loads() {
  let root = std::env::temp_dir().join(format!("argvus-test-{}", uuid::Uuid::new_v4()));
  std::fs::create_dir_all(root.join("config")).unwrap();
  let paths = Paths {
    data_dir: root.join("data"),
    state_dir: root.join("state"),
    cache_dir: root.join("cache"),
    database: root.join("data/calendar.db"),
    config_file: root.join("config/config.toml"),
    legacy_settings_file: root.join("config/settings.toml"),
    user_config_file: root.join("user/config.toml"),
    user_style: root.join("config/style.css"),
    theme_file: root.join("config/theme.css"),
    cache_theme_file: root.join("cache/theme.css"),
    active_argvus_theme_file: root.join("config/argvus/data/.active-theme"),
    events_enabled_file: root.join("state/events-enabled"),
  };
  std::fs::write(
    &paths.config_file,
    r#"[appearance]
font_family = "JetBrains Mono"
font_size = 13
[locale]
language = "pt-BR"
[calendar]
show_events = false
"#,
  )
  .unwrap();
  let config = AppConfig::load(&paths).unwrap();
  assert_eq!(config.appearance.font_family, "JetBrains Mono");
  assert_eq!(config.locale.language, "pt-BR");
  assert!(!config.calendar.show_events);
}

#[test]
fn invalid_toml_is_an_error() {
  let root = std::env::temp_dir().join(format!("argvus-test-{}", uuid::Uuid::new_v4()));
  std::fs::create_dir_all(root.join("config")).unwrap();
  let paths = Paths {
    data_dir: root.join("data"),
    state_dir: root.join("state"),
    cache_dir: root.join("cache"),
    database: root.join("data/calendar.db"),
    config_file: root.join("config/config.toml"),
    legacy_settings_file: root.join("config/settings.toml"),
    user_config_file: root.join("user/config.toml"),
    user_style: root.join("config/style.css"),
    theme_file: root.join("config/theme.css"),
    cache_theme_file: root.join("cache/theme.css"),
    active_argvus_theme_file: root.join("config/argvus/data/.active-theme"),
    events_enabled_file: root.join("state/events-enabled"),
  };
  std::fs::write(&paths.config_file, "[appearance").unwrap();
  assert!(AppConfig::load(&paths).is_err());
}

fn test_paths() -> Paths {
  let root = std::env::temp_dir().join(format!("argvus-test-{}", uuid::Uuid::new_v4()));
  Paths {
    data_dir: root.join("data"),
    state_dir: root.join("state"),
    cache_dir: root.join("cache"),
    database: root.join("data/calendar.db"),
    config_file: root.join("config/config.toml"),
    legacy_settings_file: root.join("config/settings.toml"),
    user_config_file: root.join("user/config.toml"),
    user_style: root.join("config/style.css"),
    theme_file: root.join("config/theme.css"),
    cache_theme_file: root.join("cache/theme.css"),
    active_argvus_theme_file: root.join("config/argvus/data/.active-theme"),
    events_enabled_file: root.join("state/events-enabled"),
  }
}

#[test]
fn user_config_overrides_system_config() {
  let paths = test_paths();
  std::fs::create_dir_all(paths.config_file.parent().unwrap()).unwrap();
  std::fs::create_dir_all(paths.user_config_file.parent().unwrap()).unwrap();
  std::fs::write(
    &paths.config_file,
    "[appearance]\nfont_family = \"System Mono\"\nfont_size = 13\n[locale]\nlanguage = \"en-US\"\n",
  )
  .unwrap();
  std::fs::write(
    &paths.user_config_file,
    "[appearance]\nfont_size = 20\n[locale]\nlanguage = \"pt-BR\"\n",
  )
  .unwrap();
  let config = AppConfig::load(&paths).unwrap();
  assert_eq!(config.appearance.font_family, "System Mono");
  assert_eq!(config.appearance.font_size, 20);
  assert_eq!(config.locale.language, "pt-BR");
}

#[test]
fn save_writes_user_config_and_loads_back() {
  let paths = test_paths();
  let mut config = AppConfig::default();
  config.appearance.font_size = 17;
  config.save(&paths).unwrap();
  assert!(paths.user_config_file.exists());
  let loaded = AppConfig::load(&paths).unwrap();
  assert_eq!(loaded.appearance.font_size, 17);
}

#[test]
fn events_are_disabled_until_explicitly_enabled() {
  let paths = test_paths();
  let config = AppConfig::default();
  assert!(!load_events_enabled(&paths, &config));
  assert!(!service_events_enabled(&paths));

  save_events_enabled(&paths, true).unwrap();
  assert!(load_events_enabled(&paths, &config));
  assert!(service_events_enabled(&paths));
}

#[test]
fn effective_config_prefers_user_file() {
  let paths = test_paths();
  assert_eq!(effective_config_file(&paths), paths.config_file);
  AppConfig::default().save(&paths).unwrap();
  assert_eq!(effective_config_file(&paths), paths.user_config_file);
}

#[test]
fn unsupported_language_falls_back() {
  let mut config = AppConfig::default();
  config.locale.language = "xx-YY".to_string();
  assert_eq!(AppConfig::validate(config).locale.language, "auto");
}

#[test]
fn auto_language_is_kept() {
  let mut config = AppConfig::default();
  config.locale.language = "auto".to_string();
  assert_eq!(AppConfig::validate(config).locale.language, "auto");
}

#[test]
fn font_size_is_clamped() {
  let mut config = AppConfig::default();
  config.appearance.font_size = 99;
  assert_eq!(AppConfig::validate(config).appearance.font_size, 32);
}

#[test]
fn editor_terminal_command_prepends_sudo() {
  let mut config = AppConfig::default();
  config.editor.command = "vim".to_string();
  config.editor.args = vec!["-c".to_string(), "set nu".to_string()];
  config.terminal.command = "kitty".to_string();
  let (program, args) = editor_terminal_command(
    &config,
    Path::new("/etc/argvus/taskbar/calendar/config.toml"),
  )
  .unwrap();
  assert_eq!(program, "kitty");
  assert_eq!(
    args,
    [
      "-e",
      "sudo",
      "vim",
      "-c",
      "set nu",
      "/etc/argvus/taskbar/calendar/config.toml"
    ]
  );
}
