//! The ARGVUS path contract for the terminal applications. It mirrors the shell
//! helpers of `session/sh/paths.sh` and the exports of `bootstrap.sh`, which
//! `spf` and `yazi` depend on.

use crate::exec::{Failure, env_nonempty};
use std::ffi::OsString;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const DEFAULT_SYSTEM_CONFIG: &str = "/usr/share/argvus";
const DEFAULT_XDG_CONFIG_DIRS: &str = "/etc/xdg";

/// The configuration subdirectories of the packaged projects, searched by XDG
/// before the ARGVUS system root itself.
const PROJECT_CONFIG_SUBDIRECTORIES: [&str; 9] = [
  "portal/config",
  "appearance/config",
  "app-profiles/config",
  "terminal/config",
  "launcher/config",
  "notifications/config",
  "network/config",
  "control-panel/config",
  "taskbar/config",
];

/// The directories and environment roots that `bootstrap.sh` derives.
pub struct Layout {
  system_config: PathBuf,
  config_home: PathBuf,
  data_home: PathBuf,
}

impl Layout {
  /// Reads the roots the same way `variables.sh` does: explicit ARGVUS variables
  /// first, then XDG, then `$HOME`. The shell bootstrap requires `HOME`.
  pub fn from_environment() -> Result<Self, Failure> {
    let home = env_nonempty("HOME").ok_or_else(|| Failure::new("HOME is not set"))?;
    let config_home = env_path("ARGVUS_CONFIG_HOME")
      .or_else(|| env_path("XDG_CONFIG_HOME"))
      .unwrap_or_else(|| PathBuf::from(home).join(".config"));
    let data_home =
      env_path("ARGVUS_DATA_HOME").unwrap_or_else(|| config_home.join("argvus").join("data"));
    Ok(Self {
      system_config: env_path("ARGVUS_SYSTEM_CONFIG")
        .unwrap_or_else(|| PathBuf::from(DEFAULT_SYSTEM_CONFIG)),
      config_home,
      data_home,
    })
  }

  /// The user configuration root, where `yazi` and `superfile` settings may live.
  pub fn config_home(&self) -> &Path {
    &self.config_home
  }

  /// A packaged file under the ARGVUS system root.
  pub fn system_config(&self, relative: &str) -> PathBuf {
    self.system_config.join(relative)
  }

  /// The managed user copy under the data root (`paths_user_config`).
  pub fn user_config(&self, relative: &str) -> PathBuf {
    self.data_home.join(legacy_relative(relative))
  }

  /// The generated copy under the data root (`paths_generated_config`).
  pub fn generated_config(&self, relative: &str) -> PathBuf {
    self
      .data_home
      .join("generated")
      .join(legacy_relative(relative))
  }

  /// The configuration for `relative` as the applications read it
  /// (`paths_config`). In mutable mode the generated copy is created first by
  /// `argvus-config`, which owns the copy rules.
  pub fn config(&self, relative: &str) -> PathBuf {
    let mutable_mode = env_nonempty("ARGVUS_MUTABLE_CONFIG").is_some_and(|value| value == "1");
    if mutable_mode && !is_source_directory(relative) {
      return ensure_generated_copy(relative);
    }
    self.read_config(relative)
  }

  /// The first existing copy of `relative`, in precedence order, or the packaged
  /// file when no copy exists (`paths_read_config`). Native overrides come first,
  /// then managed user and generated copies.
  fn read_config(&self, relative: &str) -> PathBuf {
    let candidates = [
      self.config_home.join(legacy_relative(relative)),
      self.config_home.join(relative),
      self.user_config(relative),
      self
        .config_home
        .join("argvus")
        .join(legacy_relative(relative)),
      self.config_home.join("argvus").join(relative),
      self.generated_config(relative),
      self
        .config_home
        .join("argvus/data/generated")
        .join(relative),
    ];
    for candidate in candidates {
      if candidate.symlink_metadata().is_ok() {
        return candidate;
      }
    }
    self.system_config(relative)
  }

  /// `XDG_CONFIG_DIRS` as `bootstrap.sh` exports it: the packaged project
  /// directories are prepended unless they are already present. Returns `None`
  /// when the variable must stay as the environment set it.
  pub fn xdg_config_dirs_override(&self) -> Option<OsString> {
    let current =
      env_nonempty("XDG_CONFIG_DIRS").unwrap_or_else(|| OsString::from(DEFAULT_XDG_CONFIG_DIRS));
    let current_text = current.to_string_lossy();
    let portal_entry = format!(":{}/portal/config:", self.system_config.display());
    if format!(":{current_text}:").contains(&portal_entry) {
      return None;
    }
    let mut project_directories: Vec<String> = PROJECT_CONFIG_SUBDIRECTORIES
      .iter()
      .map(|subdirectory| self.system_config.join(subdirectory).display().to_string())
      .collect();
    project_directories.push(self.system_config.display().to_string());
    Some(OsString::from(format!(
      "{}:{current_text}",
      project_directories.join(":")
    )))
  }
}

/// The path of `relative` relative to the root that `paths_legacy_relative`
/// names: older layouts stored each component under a flat, legacy name.
pub fn legacy_relative(relative: &str) -> String {
  let basename = relative.rsplit('/').next().unwrap_or(relative);
  if matches!(
    relative,
    "power/config/hypridle.conf" | "lock/config/hyprlock.conf"
  ) {
    return format!("hypr/{basename}");
  }
  if relative.starts_with("taskbar/config/") || relative.starts_with("widget-telemetry/config/") {
    return format!("waybar/{}", after_config_marker(relative));
  }
  if relative.starts_with("removable-devices/config/") {
    return format!("removable-devices/{}", after_config_marker(relative));
  }
  if relative.starts_with("taskbar-storage/config/") {
    return format!("taskbar/storage/{}", after_config_marker(relative));
  }
  if relative.starts_with("launcher/config/") {
    return format!("rofi/{}", after_config_marker(relative));
  }
  if relative.starts_with("notifications/config/") {
    return format!("dunst/{}", after_config_marker(relative));
  }
  if let Some(rest) = relative.strip_prefix("control-panel/config/quickshell/") {
    return format!("quickshell/{rest}");
  }
  if let Some(rest) = relative.strip_prefix("appearance/config/") {
    return rest.to_owned();
  }
  if let Some(rest) = relative.strip_prefix("app-profiles/config/") {
    return rest.to_owned();
  }
  if relative.contains("/sh/") {
    return format!("scripts/argvus/{basename}");
  }
  relative.to_owned()
}

/// The text after the first `/config/`, which is what `${1#*/config/}` yields.
fn after_config_marker(relative: &str) -> &str {
  relative
    .split_once("/config/")
    .map_or(relative, |(_, rest)| rest)
}

/// Sources whose files are read in place rather than copied to the mutable tree.
fn is_source_directory(relative: &str) -> bool {
  relative.starts_with("sh/")
    || relative.contains("/sh/")
    || relative.starts_with("docs/")
    || relative.contains("/docs/")
}

/// Asks `argvus config` for the mutable copy of `relative` and returns its path.
/// The same binary answers, so the helper is never looked up on PATH. A binary
/// that cannot start yields an empty path, the same result the shell produced.
fn ensure_generated_copy(relative: &str) -> PathBuf {
  let executable = match std::env::current_exe() {
    Ok(executable) => executable,
    Err(error) => {
      let _ = writeln!(std::io::stderr(), "argvus: cannot locate argvus: {error}");
      return PathBuf::new();
    }
  };
  let output = Command::new(executable)
    .args(["config", "mutable-path"])
    .arg(relative)
    .stdin(Stdio::inherit())
    .stderr(Stdio::inherit())
    .output();
  match output {
    Ok(output) => {
      let text = String::from_utf8_lossy(&output.stdout);
      PathBuf::from(text.trim_end_matches('\n'))
    }
    Err(error) => {
      // The message goes to stderr like the shell's "command not found" did.
      let _ = writeln!(
        std::io::stderr(),
        "argvus: cannot run argvus config: {error}"
      );
      PathBuf::new()
    }
  }
}

/// Reads a path from the environment, treating an empty value as unset.
fn env_path(name: &str) -> Option<PathBuf> {
  env_nonempty(name).map(PathBuf::from)
}

/// Whether `path` exists as a file, a directory or a link, like `-e || -L`.
pub fn exists(path: &Path) -> bool {
  path.symlink_metadata().is_ok()
}

/// Reads the first line of a file, or `None` when the file cannot be read.
pub fn first_line(path: &Path) -> Option<String> {
  let content = fs::read_to_string(path).ok()?;
  Some(content.lines().next().unwrap_or_default().to_owned())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn maps_hypridle_to_the_hypr_directory() {
    assert_eq!(
      legacy_relative("power/config/hypridle.conf"),
      "hypr/hypridle.conf"
    );
  }

  #[test]
  fn maps_taskbar_config_to_waybar() {
    assert_eq!(
      legacy_relative("taskbar/config/modules/clock.jsonc"),
      "waybar/modules/clock.jsonc"
    );
  }

  #[test]
  fn maps_launcher_config_to_rofi() {
    assert_eq!(
      legacy_relative("launcher/config/rasi/main.rasi"),
      "rofi/rasi/main.rasi"
    );
  }

  #[test]
  fn maps_app_profile_config_to_the_flat_name() {
    assert_eq!(legacy_relative("app-profiles/config/yazi"), "yazi");
  }

  #[test]
  fn maps_script_directories_to_the_flat_scripts_directory() {
    assert_eq!(
      legacy_relative("hyprland/sh/scratchpad.sh"),
      "scripts/argvus/scratchpad.sh"
    );
  }

  #[test]
  fn keeps_unmatched_paths_unchanged() {
    assert_eq!(
      legacy_relative("themes/argvus-dark.toml"),
      "themes/argvus-dark.toml"
    );
  }
}
