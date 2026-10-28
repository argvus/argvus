//! Path resolution used by the session scripts.
//!
//! The shell helpers in `paths.sh` decide which copy of a packaged file is
//! live: a native override, the ARGVUS user copy, a legacy copy, the generated
//! copy, or the packaged file. This module reproduces that read-only order, so
//! the session reload can resolve files without the shell.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// Packaged defaults, used when `ARGVUS_SYSTEM_CONFIG` is not set.
pub(crate) const PACKAGED_SYSTEM_CONFIG: &str = "/usr/share/argvus";

/// Directories the resolution order is built from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionPaths {
  /// `ARGVUS_CONFIG_HOME`, the directory that contains `argvus/`.
  pub config_home: PathBuf,
  /// `ARGVUS_DATA_HOME`, normally `<config>/argvus/data`.
  pub data_home: PathBuf,
  /// `ARGVUS_SYSTEM_CONFIG`, the packaged defaults.
  pub system_config: PathBuf,
}

impl SessionPaths {
  /// Resolves the directories from the environment, with the same defaults
  /// the shell helpers use.
  pub fn from_environment() -> Option<Self> {
    let config_home = non_empty_var("ARGVUS_CONFIG_HOME")
      .or_else(|| non_empty_var("XDG_CONFIG_HOME"))
      .or_else(|| non_empty_var("HOME").map(|home| home.join(".config")))?;
    let data_home =
      non_empty_var("ARGVUS_DATA_HOME").unwrap_or_else(|| config_home.join("argvus/data"));
    let system_config = non_empty_var("ARGVUS_SYSTEM_CONFIG")
      .unwrap_or_else(|| PathBuf::from(PACKAGED_SYSTEM_CONFIG));
    Some(Self {
      config_home,
      data_home,
      system_config,
    })
  }

  /// The live copy of a packaged file, as `paths_config` returns it in read
  /// mode. The first candidate that exists wins; the packaged file is the
  /// fallback even when it does not exist.
  pub fn read_config(&self, relative: &str) -> PathBuf {
    let legacy = legacy_relative(relative);
    let candidates = [
      self.config_home.join(&legacy),
      self.config_home.join(relative),
      self.data_home.join(&legacy),
      self.config_home.join("argvus").join(&legacy),
      self.config_home.join("argvus").join(relative),
      self.data_home.join("generated").join(&legacy),
      self
        .config_home
        .join("argvus/data/generated")
        .join(relative),
    ];
    candidates
      .into_iter()
      .find(|candidate| exists(candidate))
      .unwrap_or_else(|| self.system_config.join(relative))
  }

  /// The ARGVUS user copy of a file (`paths_user_config`).
  pub fn user_config(&self, relative: &str) -> PathBuf {
    self.data_home.join(legacy_relative(relative))
  }

  /// The packaged copy of a file (`paths_system_config`).
  pub fn system_config(&self, relative: &str) -> PathBuf {
    self.system_config.join(relative)
  }

  /// The native override of a file (`paths_override_config`).
  pub fn override_config(&self, relative: &str) -> PathBuf {
    self.config_home.join(legacy_relative(relative))
  }

  /// The generated copy of a file (`paths_generated_config`).
  pub fn generated_config(&self, relative: &str) -> PathBuf {
    self
      .data_home
      .join("generated")
      .join(legacy_relative(relative))
  }

  /// The generated `fonts.conf` written by the projection.
  pub fn generated_fonts(&self) -> PathBuf {
    self.config_home.join("argvus/data/generated/fonts.conf")
  }
}

/// Maps a canonical packaged path to its pre-`data/` legacy location. The
/// first matching rule wins, as in the shell `case`.
pub fn legacy_relative(relative: &str) -> String {
  if relative == "power/config/hypridle.conf" || relative == "lock/config/hyprlock.conf" {
    return format!("hypr/{}", basename(relative));
  }
  if let Some(rest) = relative.strip_prefix("taskbar/config/") {
    return format!("waybar/{rest}");
  }
  if let Some(rest) = relative.strip_prefix("widget-telemetry/config/") {
    return format!("waybar/{rest}");
  }
  if let Some(rest) = relative.strip_prefix("removable-devices/config/") {
    return format!("removable-devices/{rest}");
  }
  if let Some(rest) = relative.strip_prefix("taskbar-storage/config/") {
    return format!("taskbar/storage/{rest}");
  }
  if let Some(rest) = relative.strip_prefix("launcher/config/") {
    return format!("rofi/{rest}");
  }
  if let Some(rest) = relative.strip_prefix("notifications/config/") {
    return format!("dunst/{rest}");
  }
  if let Some(rest) = relative.strip_prefix("control-panel/config/quickshell/") {
    return format!("quickshell/{rest}");
  }
  if let Some(rest) = relative.strip_prefix("appearance/config/") {
    return rest.to_string();
  }
  if let Some(rest) = relative.strip_prefix("app-profiles/config/") {
    return rest.to_string();
  }
  if relative.starts_with("hyprland/sh/") || relative.contains("/sh/") {
    return format!("scripts/argvus/{}", basename(relative));
  }
  relative.to_string()
}

/// The value of `key` in the generated `fonts.conf`, or `fallback` when the
/// file or the key is missing. Only the text before the first `=` is compared
/// with the key, and the value is trimmed, as the shell awk did.
pub fn font_state_value(fonts_conf: &Path, key: &str, fallback: &str) -> String {
  fs::read_to_string(fonts_conf)
    .ok()
    .and_then(|text| {
      text.lines().find_map(|line| {
        let (name, value) = line.split_once('=')?;
        (name == key).then(|| value.trim().to_string())
      })
    })
    .unwrap_or_else(|| fallback.to_string())
}

/// The default interface font as `"<family> <size>"`, from the apps font when
/// set, otherwise the default font, otherwise the ARGVUS defaults.
pub fn font_default_value(paths: &SessionPaths) -> String {
  let fonts = paths.generated_fonts();
  let family = font_state_value(
    &fonts,
    "apps_name",
    &font_state_value(&fonts, "default_name", "IBM Plex Mono"),
  );
  let size = font_state_value(
    &fonts,
    "apps_size",
    &font_state_value(&fonts, "default_size", "12"),
  );
  format!("{family} {size}")
}

fn basename(path: &str) -> &str {
  path.rsplit('/').next().unwrap_or(path)
}

fn exists(path: &Path) -> bool {
  fs::symlink_metadata(path).is_ok()
}

fn non_empty_var(name: &str) -> Option<PathBuf> {
  env::var_os(name)
    .filter(|value| !value.is_empty())
    .map(PathBuf::from)
}

#[cfg(test)]
mod tests {
  use super::*;

  fn paths_in(root: &Path) -> SessionPaths {
    SessionPaths {
      config_home: root.join("config"),
      data_home: root.join("config/argvus/data"),
      system_config: root.join("system"),
    }
  }

  #[test]
  fn legacy_mapping_follows_the_shell_rules() {
    assert_eq!(
      legacy_relative("power/config/hypridle.conf"),
      "hypr/hypridle.conf"
    );
    assert_eq!(
      legacy_relative("taskbar/config/argvus-taskbar.css"),
      "waybar/argvus-taskbar.css"
    );
    assert_eq!(
      legacy_relative("launcher/config/config.rasi"),
      "rofi/config.rasi"
    );
    assert_eq!(
      legacy_relative("control-panel/config/quickshell/Theme.qml"),
      "quickshell/Theme.qml"
    );
    assert_eq!(legacy_relative("appearance/config/tui/x.css"), "tui/x.css");
    assert_eq!(
      legacy_relative("hyprland/sh/scratchpad.sh"),
      "scripts/argvus/scratchpad.sh"
    );
    assert_eq!(
      legacy_relative("yazi/flavors/x.yazi"),
      "yazi/flavors/x.yazi"
    );
  }

  #[test]
  fn read_config_prefers_the_override_then_user_then_system() {
    let root = env::temp_dir().join(format!("argvus-paths-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let paths = paths_in(&root);
    let relative = "launcher/config/config.rasi";
    let system = paths.system_config(relative);
    fs::create_dir_all(system.parent().expect("parent")).expect("system dir");
    fs::write(&system, "system").expect("system file");
    assert_eq!(paths.read_config(relative), system, "packaged fallback");

    let user = paths.user_config(relative);
    fs::create_dir_all(user.parent().expect("parent")).expect("user dir");
    fs::write(&user, "user").expect("user file");
    assert_eq!(
      paths.read_config(relative),
      user,
      "ARGVUS user copy beats packaged"
    );

    let native = paths.config_home.join("rofi/config.rasi");
    fs::create_dir_all(native.parent().expect("parent")).expect("native dir");
    fs::write(&native, "native").expect("native file");
    assert_eq!(
      paths.read_config(relative),
      native,
      "native override beats user copy"
    );
    let _ = fs::remove_dir_all(root);
  }

  #[test]
  fn font_value_uses_apps_font_then_default_then_builtin() {
    let root = env::temp_dir().join(format!("argvus-fonts-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let paths = paths_in(&root);
    let fonts = paths.generated_fonts();
    fs::create_dir_all(fonts.parent().expect("parent")).expect("fonts dir");

    assert_eq!(
      font_default_value(&paths),
      "IBM Plex Mono 12",
      "no fonts file"
    );

    fs::write(&fonts, "default_name=Inter\ndefault_size=11\n").expect("fonts file");
    assert_eq!(font_default_value(&paths), "Inter 11");

    fs::write(
      &fonts,
      "default_name=Inter\ndefault_size=11\napps_name =  Fira  \napps_size=10\n",
    )
    .expect("fonts file");
    // "apps_name =" has a trailing space before "=", so it is not the key.
    assert_eq!(font_default_value(&paths), "Inter 10");

    fs::write(&fonts, "apps_name=Fira Code\napps_size=9\n").expect("fonts file");
    assert_eq!(font_default_value(&paths), "Fira Code 9");
    let _ = fs::remove_dir_all(root);
  }
}
