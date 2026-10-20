//! Loading of the small system-wide Greeter configuration.
//!
//! Configuration is intentionally read-only at runtime. The greeter needs
//! presentation defaults and a session preference, but it must not mutate
//! system configuration while a user is waiting to authenticate.

use serde::Deserialize;
use std::{fs, path::PathBuf};
use tracing::{info, warn};

const CONFIG_PATH: &str = "/etc/argvus/greeter.toml";
// Kept as a compatibility default for existing configuration files. The TUI
// itself does not render a wallpaper, but deserializing this field preserves
// the established configuration contract.
const DEFAULT_WALLPAPER: &str = "/usr/share/backgrounds/argvus/argvus-dark.jxl";

/// Top-level Greeter configuration parsed from TOML.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Config {
  /// Presentation preferences used by the TUI.
  pub appearance: AppearanceConfig,
  /// Session preference used to mark the initial Wayland selection.
  pub session: SessionConfig,
}

/// Optional clock/date presentation settings.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct AppearanceConfig {
  /// Legacy wallpaper path retained for configuration compatibility.
  pub wallpaper: PathBuf,
  /// Whether the local clock should be drawn.
  pub show_clock: bool,
  /// Whether the date should accompany the clock.
  pub show_date: bool,
}

/// Session selection policy for the initial login screen.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct SessionConfig {
  /// Case-insensitive session id, name, or desktop name to prefer.
  pub default: String,
}

impl Default for AppearanceConfig {
  fn default() -> Self {
    // These defaults are safe for a fresh installation and keep the login
    // screen useful when the optional configuration file is absent.
    Self {
      wallpaper: PathBuf::from(DEFAULT_WALLPAPER),
      show_clock: true,
      show_date: true,
    }
  }
}

impl Default for SessionConfig {
  fn default() -> Self {
    // ARGVUS is preferred, with discovery still retaining every installed
    // Wayland session as an available fallback.
    Self {
      default: "argvus".to_string(),
    }
  }
}

impl Config {
  /// Loads `/etc/argvus/greeter.toml`, falling back only when it is absent.
  ///
  /// A malformed or unreadable existing file is an error rather than silently
  /// becoming defaults; this makes administrator mistakes observable.
  pub fn load() -> anyhow::Result<Self> {
    match fs::read_to_string(CONFIG_PATH) {
      Ok(contents) => {
        let config = toml::from_str(&contents)?;
        info!(path = CONFIG_PATH, "configuration loaded");
        Ok(config)
      }
      Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
        info!(
          path = CONFIG_PATH,
          "configuration not found, using defaults"
        );
        Ok(Self::default())
      }
      Err(error) => {
        warn!(path = CONFIG_PATH, %error, "could not read configuration");
        Err(error.into())
      }
    }
  }
}
