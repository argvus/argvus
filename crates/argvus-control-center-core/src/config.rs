//! Implements configuration loading, validation, and persistence in crate `argvus control center core`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::process::{ProcessRequest, ProcessRunner, SystemProcessRunner};

/// Default location of the ARGVUS Control Center configuration file.
pub const CONFIG_DIR: &str = "/etc/argvus/control-center";
/// Defines the constant `CONFIG_PATH`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub const CONFIG_PATH: &str = "/etc/argvus/control-center/config.toml";

/// Applies the canonical ARGVUS configuration through the user service that
/// owns projection and runtime reload. Callers must persist the modular
/// canonical configuration
/// first; this function intentionally does not mutate canonical state.
pub fn reload_argvus_config_service() -> Result<(), String> {
  reload_argvus_config_service_with(&SystemProcessRunner)
}

/// Applies the canonical configuration through a caller-provided process
/// runner. Backends use this variant when their persistence and reload steps
/// must share the same injectable process boundary during tests.
pub fn reload_argvus_config_service_with(runner: &impl ProcessRunner) -> Result<(), String> {
  let output = runner
    .run(
      &ProcessRequest::new("systemctl")
        .arg("--user")
        .arg("reload")
        .arg("argvus-config.service")
        .timeout(Duration::from_secs(60)),
    )
    .map_err(|error| format!("failed to reload argvus-config.service: {error}"))?;
  if output.status == Some(0) {
    return Ok(());
  }
  let detail = String::from_utf8_lossy(&output.stderr).trim().to_owned();
  if detail.is_empty() {
    Err("argvus-config.service could not apply the configuration".into())
  } else {
    Err(detail)
  }
}

/// Maintains the static state `ICONS_ENABLED`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
static ICONS_ENABLED: AtomicBool = AtomicBool::new(true);

/// Global appearance/behavior settings for the ARGVUS Control Center.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
  #[serde(rename = "Appearance")]
  pub appearance: Appearance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
/// Represents `Appearance`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct Appearance {
  pub icons: bool,
}

impl Default for Appearance {
  /// Executes the `default` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn default() -> Self {
    Self { icons: true }
  }
}

impl AppConfig {
  /// Resolves the configuration file path. `ARGVUS_CONFIG_PATH` overrides the
  /// default location (useful for tests and portable setups).
  pub fn config_path() -> PathBuf {
    std::env::var_os("ARGVUS_CONFIG_PATH")
      .map(PathBuf::from)
      .unwrap_or_else(|| PathBuf::from(CONFIG_PATH))
  }

  /// Loads the configuration from disk and applies it to the live icon state.
  pub fn load() -> Self {
    let config: AppConfig = std::fs::read_to_string(Self::config_path())
      .ok()
      .and_then(|contents| toml::from_str(&contents).ok())
      .unwrap_or_default();
    ICONS_ENABLED.store(config.appearance.icons, Ordering::Relaxed);
    config
  }

  /// Executes the `icons` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn icons(&self) -> bool {
    self.appearance.icons
  }

  /// Applies the icon state in-session only, without writing to disk. Used by
  /// the UI to toggle real-time before the elevated save round-trip completes.
  pub fn set_session_icons(enabled: bool) {
    Self::set_icon_state(enabled);
  }

  /// Toggles icon rendering and persists the change to the configuration file.
  /// The in-session result is applied even when the write fails, so the change
  /// is real-time; a failing write only loses persistence until the next launch.
  pub fn set_icons(&mut self, enabled: bool) -> Result<(), String> {
    self.appearance.icons = enabled;
    Self::set_icon_state(enabled);
    let path = Self::config_path();
    if let Some(parent) = path.parent() {
      std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let rendered = toml::to_string(self).map_err(|error| error.to_string())?;
    std::fs::write(&path, rendered).map_err(|error| error.to_string())
  }

  /// Applies the `set_icon_state` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn set_icon_state(enabled: bool) {
    ICONS_ENABLED.store(enabled, Ordering::Relaxed);
  }

  /// Whether decorative icons should be rendered right now.
  pub fn icons_enabled() -> bool {
    ICONS_ENABLED.load(Ordering::Relaxed)
  }

  /// Returns the given glyph when icons are enabled, otherwise an empty string.
  ///
  /// Ratatui 0.30.x has a backend diff bug with grapheme clusters carrying
  /// VS16 (U+FE0F): on terminals that render the cluster as two columns, later
  /// writes on the same row can drift one cell to the right. Keep TUI icons on
  /// their text-presentation form so a selected row can never overwrite the
  /// right border because of that width disagreement.
  pub fn icon(glyph: &str) -> &str {
    if ICONS_ENABLED.load(Ordering::Relaxed) {
      Self::terminal_safe_icon(glyph)
    } else {
      ""
    }
  }

  /// Executes the `terminal_safe_icon` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn terminal_safe_icon(glyph: &str) -> &str {
    match glyph {
      "⚙️" => "⚙",
      "⚠️" => "⚠",
      "🎙️" => "🎙",
      "🛡️" => "🛡",
      "🗂️" => "🗂",
      "⌨️" => "⌨",
      "🛰️" => "🛰",
      "🖥️" => "🖥",
      "🖼️" => "🖼",
      "🎛️" => "🎛",
      _ => glyph,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::sync::Mutex;
  use std::sync::atomic::AtomicUsize;

  /// Maintains the static state `CONFIG_TEST_LOCK`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
  static CONFIG_TEST_LOCK: Mutex<()> = Mutex::new(());
  /// Maintains the static state `CONFIG_TEST_SEQUENCE`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
  static CONFIG_TEST_SEQUENCE: AtomicUsize = AtomicUsize::new(0);

  /// Executes the `run_with_config` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn run_with_config(toml: &str, test: fn(config_path: &std::path::Path)) {
    let _guard = CONFIG_TEST_LOCK.lock().unwrap();
    let sequence = CONFIG_TEST_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let dir = temp_dir().join(format!(
      "argvus-core-config-test-{}-{sequence}",
      std::process::id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("config.toml");
    if !toml.is_empty() {
      std::fs::write(&path, toml).unwrap();
    }
    unsafe {
      std::env::set_var("ARGVUS_CONFIG_PATH", &path);
    }
    test(&path);
    unsafe {
      std::env::remove_var("ARGVUS_CONFIG_PATH");
    }
    let _ = std::fs::remove_dir_all(&dir);
  }

  /// Executes the `temp_dir` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn temp_dir() -> PathBuf {
    std::env::var_os("TMPDIR")
      .map(PathBuf::from)
      .unwrap_or_else(|| PathBuf::from("/tmp"))
  }

  #[test]
  /// Retrieves data for `load_parses_appearance_section` without mixing collection with TUI rendering. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn load_parses_appearance_section() {
    run_with_config("[Appearance]\nicons = true\n", |path| {
      let config = AppConfig::load();
      assert_eq!(AppConfig::config_path(), path);
      assert!(config.icons());
      assert!(AppConfig::icons_enabled());
    });
  }

  #[test]
  /// Retrieves data for `load_defaults_to_enabled_icons_when_missing` without mixing collection with TUI rendering. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn load_defaults_to_enabled_icons_when_missing() {
    run_with_config("", |_| {
      let config = AppConfig::load();
      assert!(config.icons());
      assert!(AppConfig::icons_enabled());
    });
  }

  #[test]
  /// Retrieves data for `load_matches_wrong_table_names` without mixing collection with TUI rendering. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn load_matches_wrong_table_names() {
    run_with_config("[nope]\nicons = false\n", |path| {
      let config = AppConfig::load();
      assert_eq!(AppConfig::config_path(), path);
      assert!(config.icons(), "wrong table must fall back to default");
    });
  }

  #[test]
  /// Applies the `set_icons_matches_disk_and_session_state` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn set_icons_matches_disk_and_session_state() {
    run_with_config("[Appearance]\nicons = true\n", |path| {
      let mut config = AppConfig::load();
      config.set_icons(false).unwrap();
      assert!(!AppConfig::icons_enabled());
      let contents = std::fs::read_to_string(path).unwrap();
      assert!(contents.contains("icons = false"), "{contents}");
      assert!(!AppConfig::load().icons());
    });
  }

  #[test]
  /// Executes the `icon_returns_empty_when_disabled` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn icon_returns_empty_when_disabled() {
    run_with_config("[Appearance]\nicons = false\n", |_| {
      AppConfig::load();
      assert_eq!(AppConfig::icon("💻"), "");
      assert_eq!(AppConfig::icon("🇧🇷"), "");
    });
  }

  #[test]
  /// Executes the `icon_returns_glyph_when_enabled` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn icon_returns_glyph_when_enabled() {
    run_with_config("[Appearance]\nicons = true\n", |_| {
      AppConfig::load();
      assert_eq!(AppConfig::icon("💻"), "💻");
      assert_eq!(AppConfig::icon("🇧🇷"), "🇧🇷");
    });
  }

  #[test]
  /// Executes the `icon_removes_vs16_from_terminal_icons` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn icon_removes_vs16_from_terminal_icons() {
    run_with_config("[Appearance]\nicons = true\n", |_| {
      AppConfig::load();
      for (input, expected) in [
        ("⚙️", "⚙"),
        ("⚠️", "⚠"),
        ("🎙️", "🎙"),
        ("🛡️", "🛡"),
        ("🗂️", "🗂"),
        ("⌨️", "⌨"),
        ("🛰️", "🛰"),
        ("🖥️", "🖥"),
        ("🖼️", "🖼"),
        ("🎛️", "🎛"),
      ] {
        let icon = AppConfig::icon(input);
        assert_eq!(icon, expected);
        assert!(!icon.contains('\u{fe0f}'));
      }
    });
  }
}
