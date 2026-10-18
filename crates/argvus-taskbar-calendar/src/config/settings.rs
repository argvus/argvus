use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::calendar::WeekStart;
use crate::error::{ArgvusError, Result};

#[derive(Debug, Clone)]
pub struct Paths {
  #[allow(dead_code)]
  pub config_dir: PathBuf,
  pub data_dir: PathBuf,
  pub state_dir: PathBuf,
  pub cache_dir: PathBuf,
  pub database: PathBuf,
  pub config_file: PathBuf,
  pub legacy_settings_file: PathBuf,
  pub user_config_file: PathBuf,
  pub user_style: PathBuf,
  pub theme_file: PathBuf,
  pub theme_dir: PathBuf,
  pub cache_theme_file: PathBuf,
  pub active_argvus_theme_file: PathBuf,
  pub events_enabled_file: PathBuf,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppConfig {
  #[serde(default)]
  pub appearance: AppearanceConfig,
  #[serde(default)]
  pub locale: LocaleConfig,
  #[serde(default)]
  pub calendar: CalendarConfig,
  #[serde(default)]
  pub editor: EditorConfig,
  #[serde(default)]
  pub terminal: TerminalConfig,
}

pub type Settings = AppConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceConfig {
  #[serde(default = "default_font_family")]
  pub font_family: String,
  #[serde(default = "default_font_size")]
  pub font_size: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocaleConfig {
  #[serde(default = "default_language")]
  pub language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarConfig {
  #[serde(default = "default_week_start")]
  pub week_start: String,
  // Legacy config key. New installs keep this user-level state in cache.
  #[serde(default = "default_show_events")]
  pub show_events: bool,
  #[serde(default = "default_event_duration")]
  pub default_event_duration_minutes: i64,
  #[serde(default = "default_reminder")]
  pub default_reminder_minutes: i64,
  #[serde(default = "default_sync_interval")]
  pub sync_interval_minutes: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EditorConfig {
  #[serde(default)]
  pub command: String,
  #[serde(default)]
  pub args: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TerminalConfig {
  #[serde(default)]
  pub command: String,
  #[serde(default)]
  pub args: Vec<String>,
}

impl Default for AppearanceConfig {
  fn default() -> Self {
    Self {
      font_family: "monospace".to_string(),
      font_size: 12,
    }
  }
}

impl Default for LocaleConfig {
  fn default() -> Self {
    Self {
      language: default_language(),
    }
  }
}

impl Default for CalendarConfig {
  fn default() -> Self {
    Self {
      week_start: "monday".to_string(),
      show_events: false,
      default_event_duration_minutes: 60,
      default_reminder_minutes: 10,
      sync_interval_minutes: 15,
    }
  }
}

impl AppConfig {
  pub fn week_start(&self) -> WeekStart {
    match self.calendar.week_start.as_str() {
      "sunday" => WeekStart::Sunday,
      _ => WeekStart::Monday,
    }
  }

  pub fn load(paths: &Paths) -> Result<Self> {
    let system = load_system_config(paths)?;
    let merged = match load_user_config(paths)? {
      Some(user) => {
        let base = toml::Value::try_from(&system)
          .map_err(|err| ArgvusError::Serialization(err.to_string()))?;
        let merged = merge_toml(base, user);
        merged
          .try_into()
          .map_err(|err| ArgvusError::Serialization(err.to_string()))?
      }
      None => system,
    };
    Ok(Self::validate(merged))
  }

  /// Persist the full configuration to the user-level file so no elevated
  /// privileges are needed. The system file under /etc keeps acting as the
  /// default base when the user file does not exist.
  pub fn save(&self, paths: &Paths) -> Result<()> {
    let contents =
      toml::to_string_pretty(self).map_err(|err| ArgvusError::Serialization(err.to_string()))?;
    if let Some(parent) = paths.user_config_file.parent() {
      std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&paths.user_config_file, contents)?;
    Ok(())
  }

  fn validate(mut config: Self) -> Self {
    config.appearance.font_size = config.appearance.font_size.clamp(8, 32);
    if !matches!(config.locale.language.as_str(), "auto" | "en-US" | "pt-BR") {
      // Unknown languages follow the system locale, which itself falls
      // back to English.
      config.locale.language = "auto".to_string();
    }
    if !matches!(config.calendar.week_start.as_str(), "monday" | "sunday") {
      config.calendar.week_start = "monday".to_string();
    }
    config
  }
}

fn default_font_family() -> String {
  "monospace".to_string()
}

fn default_font_size() -> u8 {
  12
}

fn default_language() -> String {
  "auto".to_string()
}

fn default_week_start() -> String {
  "monday".to_string()
}

fn default_show_events() -> bool {
  false
}

fn default_event_duration() -> i64 {
  60
}

fn default_reminder() -> i64 {
  10
}

fn default_sync_interval() -> u64 {
  15
}

fn load_system_config(paths: &Paths) -> Result<AppConfig> {
  let path = if paths.config_file.exists() {
    &paths.config_file
  } else if paths.legacy_settings_file.exists() {
    &paths.legacy_settings_file
  } else {
    return Ok(AppConfig::default());
  };
  let contents = std::fs::read_to_string(path)?;
  let parsed = toml::from_str(&contents)
    .map_err(|err| ArgvusError::Serialization(format!("{}: {err}", path.display())))?;
  Ok(AppConfig::validate(parsed))
}

fn load_user_config(paths: &Paths) -> Result<Option<toml::Value>> {
  if !paths.user_config_file.exists() {
    return Ok(None);
  }
  let contents = std::fs::read_to_string(&paths.user_config_file)?;
  let parsed = toml::from_str(&contents).map_err(|err| {
    ArgvusError::Serialization(format!("{}: {err}", paths.user_config_file.display()))
  })?;
  Ok(Some(parsed))
}

fn merge_toml(base: toml::Value, override_: toml::Value) -> toml::Value {
  match (base, override_) {
    (toml::Value::Table(mut left), toml::Value::Table(right)) => {
      for (key, value) in right {
        let merged = match left.get(&key) {
          Some(existing)
            if matches!(existing, toml::Value::Table(_))
              && matches!(&value, toml::Value::Table(_)) =>
          {
            merge_toml(existing.clone(), value)
          }
          _ => value,
        };
        left.insert(key, merged);
      }
      toml::Value::Table(left)
    }
    (_, value) => value,
  }
}

/// The configuration file that takes effect: the user-level one when present,
/// otherwise the system-wide one.
pub fn effective_config_file(paths: &Paths) -> PathBuf {
  if paths.user_config_file.exists() {
    paths.user_config_file.clone()
  } else {
    paths.config_file.clone()
  }
}

/// Names of the themes available in the theme directory, used by the theme
/// picker in the settings screen.
pub fn list_theme_names(paths: &Paths) -> Vec<String> {
  let mut names = Vec::new();
  if let Ok(entries) = std::fs::read_dir(&paths.theme_dir) {
    for entry in entries.flatten() {
      let path = entry.path();
      if path.extension().is_some_and(|extension| extension == "css")
        && let Some(stem) = path.file_stem().and_then(|stem| stem.to_str())
      {
        names.push(stem.to_string());
      }
    }
  }
  names.sort();
  names
}

/// The active theme name, when it still exists in the theme directory.
pub fn active_theme_name(paths: &Paths) -> Option<String> {
  let name = std::fs::read_to_string(&paths.active_argvus_theme_file).ok()?;
  let name = name.trim().to_string();
  if name.is_empty() {
    return None;
  }
  list_theme_names(paths)
    .iter()
    .any(|available| available == &name)
    .then_some(name)
}

/// Persist the selected theme name to the user-level ARGVUS state.
pub fn write_active_theme(paths: &Paths, name: &str) -> Result<()> {
  if let Some(parent) = paths.active_argvus_theme_file.parent() {
    std::fs::create_dir_all(parent)?;
  }
  std::fs::write(&paths.active_argvus_theme_file, format!("{name}\n"))?;
  Ok(())
}

pub fn resolve_paths() -> Result<Paths> {
  let config_dir = PathBuf::from("/etc/argvus/taskbar/calendar");
  let data_dir =
    ensure(xdg_home("XDG_DATA_HOME", ".local/share")?.join("argvus-taskbar-calendar"))?;
  let state_dir = ensure(
    xdg_home("XDG_STATE_HOME", ".local/state")?
      .join("argvus")
      .join("taskbar")
      .join("calendar"),
  )?;
  let cache_dir = ensure(xdg_home("XDG_CACHE_HOME", ".cache")?.join("argvus-taskbar-calendar"))?;
  let config_home = xdg_home("XDG_CONFIG_HOME", ".config")?;
  let argvus_config_dir = config_home.join("argvus");
  Ok(Paths {
    database: data_dir.join("calendar.db"),
    config_file: config_dir.join("config.toml"),
    legacy_settings_file: config_dir.join("settings.toml"),
    user_config_file: config_home
      .join("argvus-taskbar-calendar")
      .join("config.toml"),
    user_style: config_dir.join("style.css"),
    theme_file: config_dir.join("theme.css"),
    theme_dir: config_dir.join("themes"),
    cache_theme_file: cache_dir.join("theme.css"),
    active_argvus_theme_file: argvus_config_dir.join("data/.active-theme"),
    events_enabled_file: state_dir.join("events-enabled"),
    config_dir,
    data_dir,
    state_dir,
    cache_dir,
  })
}

pub fn load_events_enabled(paths: &Paths, config: &AppConfig) -> bool {
  read_events_enabled(paths).unwrap_or(config.calendar.show_events)
}

pub fn service_events_enabled(paths: &Paths) -> bool {
  read_events_enabled(paths).unwrap_or(false)
}

pub fn save_events_enabled(paths: &Paths, enabled: bool) -> Result<()> {
  if let Some(parent) = paths.events_enabled_file.parent() {
    std::fs::create_dir_all(parent)?;
  }
  std::fs::write(
    &paths.events_enabled_file,
    if enabled { "true\n" } else { "false\n" },
  )?;
  Ok(())
}

fn read_events_enabled(paths: &Paths) -> Option<bool> {
  let value = std::fs::read_to_string(&paths.events_enabled_file).ok()?;
  match value.trim() {
    "true" | "1" | "yes" | "on" => Some(true),
    "false" | "0" | "no" | "off" => Some(false),
    _ => None,
  }
}

pub fn open_config(paths: &Paths, config: &AppConfig) -> Result<()> {
  let (program, args) = editor_terminal_command(config, &paths.config_file)?;
  Command::new(program)
    .args(args)
    .spawn()
    .map(|_| ())
    .map_err(|err| ArgvusError::Configuration(format!("could not open config editor: {err}")))
}

/// Build the `terminal -e sudo <editor> [args] <path>` command used to edit
/// the system-wide configuration in /etc.
fn editor_terminal_command(config: &AppConfig, path: &Path) -> Result<(String, Vec<String>)> {
  let editor = if !config.editor.command.trim().is_empty() {
    config.editor.command.clone()
  } else if let Some(value) = std::env::var_os("VISUAL") {
    value.to_string_lossy().to_string()
  } else if let Some(value) = std::env::var_os("EDITOR") {
    value.to_string_lossy().to_string()
  } else {
    "nano".to_string()
  };
  let terminal = if !config.terminal.command.trim().is_empty() {
    config.terminal.command.clone()
  } else if let Some(value) = std::env::var_os("TERMINAL") {
    value.to_string_lossy().to_string()
  } else {
    "kitty".to_string()
  };
  let mut args = config.terminal.args.clone();
  args.extend(["-e".to_string(), "sudo".to_string(), editor]);
  args.extend(config.editor.args.clone());
  args.push(path.display().to_string());
  Ok((terminal, args))
}

fn ensure(path: PathBuf) -> Result<PathBuf> {
  std::fs::create_dir_all(&path)?;
  Ok(path)
}

fn xdg_home(var: &str, fallback: &str) -> Result<PathBuf> {
  if let Some(path) = std::env::var_os(var) {
    return Ok(PathBuf::from(path));
  }
  let home = std::env::var_os("HOME")
    .ok_or_else(|| ArgvusError::Configuration("HOME is not set".to_string()))?;
  Ok(PathBuf::from(home).join(fallback))
}

pub fn validate_export_path(path: &Path) -> Result<()> {
  if path.exists() && path.is_dir() {
    return Err(ArgvusError::Configuration(format!(
      "{} is a directory",
      path.display()
    )));
  }
  if let Some(parent) = path.parent() {
    std::fs::create_dir_all(parent)?;
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn missing_config_uses_defaults() {
    let root = std::env::temp_dir().join(format!("argvus-test-{}", uuid::Uuid::new_v4()));
    let paths = Paths {
      config_dir: root.join("config"),
      data_dir: root.join("data"),
      state_dir: root.join("state"),
      cache_dir: root.join("cache"),
      database: root.join("data/calendar.db"),
      config_file: root.join("config/config.toml"),
      legacy_settings_file: root.join("config/settings.toml"),
      user_config_file: root.join("user/config.toml"),
      user_style: root.join("config/style.css"),
      theme_file: root.join("config/theme.css"),
      theme_dir: root.join("config/themes"),
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
      config_dir: root.join("config"),
      data_dir: root.join("data"),
      state_dir: root.join("state"),
      cache_dir: root.join("cache"),
      database: root.join("data/calendar.db"),
      config_file: root.join("config/config.toml"),
      legacy_settings_file: root.join("config/settings.toml"),
      user_config_file: root.join("user/config.toml"),
      user_style: root.join("config/style.css"),
      theme_file: root.join("config/theme.css"),
      theme_dir: root.join("config/themes"),
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
      config_dir: root.join("config"),
      data_dir: root.join("data"),
      state_dir: root.join("state"),
      cache_dir: root.join("cache"),
      database: root.join("data/calendar.db"),
      config_file: root.join("config/config.toml"),
      legacy_settings_file: root.join("config/settings.toml"),
      user_config_file: root.join("user/config.toml"),
      user_style: root.join("config/style.css"),
      theme_file: root.join("config/theme.css"),
      theme_dir: root.join("config/themes"),
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
      config_dir: root.join("config"),
      data_dir: root.join("data"),
      state_dir: root.join("state"),
      cache_dir: root.join("cache"),
      database: root.join("data/calendar.db"),
      config_file: root.join("config/config.toml"),
      legacy_settings_file: root.join("config/settings.toml"),
      user_config_file: root.join("user/config.toml"),
      user_style: root.join("config/style.css"),
      theme_file: root.join("config/theme.css"),
      theme_dir: root.join("config/themes"),
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
}
