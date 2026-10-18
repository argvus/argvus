//! The icon theme of a theme switch and of an accent change.
//!
//! The icon rules live in `icon-switch.sh`, which the shell helpers share. This
//! module asks that helper for the names, so every caller resolves the same
//! icon theme.

use crate::session_paths::SessionPaths;
use std::process::{Command, Stdio};

/// The icon theme in use and the ARGVUS defaults for the dark and light modes.
pub struct IconNames {
  pub current: String,
  pub dark: String,
  pub light: String,
}

/// Resolves the icon names of a theme id, or of the active theme when `theme`
/// is empty. `diagnostics` receives the helper's warnings, such as an icon theme
/// that is not installed.
pub fn resolve(paths: &SessionPaths, theme: &str, diagnostics: Stdio) -> IconNames {
  let helper = paths.system_config("appearance/sh/icon-switch.sh");
  let output = Command::new("sh")
        .arg("-c")
        .arg(". \"$1\"; icon_theme_effective \"$2\"; printf '%s\\n%s\\n' \"$ARGVUS_ICON_DEFAULT_DARK\" \"$ARGVUS_ICON_DEFAULT_LIGHT\"")
        .arg("sh")
        .arg(&helper)
        .arg(theme)
        .env("ARGVUS_SYSTEM_CONFIG", &paths.system_config)
        .env("ARGVUS_CONFIG_HOME", &paths.config_home)
        .env("ARGVUS_DATA_HOME", &paths.data_home)
        .stderr(diagnostics)
        .output();
  let text = output
    .map(|output| String::from_utf8_lossy(&output.stdout).into_owned())
    .unwrap_or_default();
  let lines: Vec<&str> = text.lines().collect();
  IconNames {
    current: lines.first().copied().unwrap_or("").to_string(),
    dark: lines.get(1).copied().unwrap_or("").to_string(),
    light: lines.get(2).copied().unwrap_or("").to_string(),
  }
}
