//! Runtime steps of a theme switch that reach the desktop: the GTK settings
//! applied through `gsettings`, and the wallpaper, applied through hyprpaper and
//! the session environment.
//!
//! The steps keep the failure behaviour of the shell they replace: a missing
//! tool skips its step, and a failed command does not stop the switch.

use crate::session_paths::SessionPaths;
use crate::text_edit::{self, Target};
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread::sleep;
use std::time::Duration;

const INTERFACE_SCHEMA: &str = "org.gnome.desktop.interface";
/// Pause between clearing the GTK theme and setting the target one, so the
/// running applications see the change.
const THEME_RESET_PAUSE: Duration = Duration::from_millis(50);

/// The GTK settings a theme sets at runtime.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GtkRuntime {
  /// `prefer-light` or `prefer-dark`, for `color-scheme`.
  pub scheme: String,
  pub theme_name: String,
  /// Theme used for the moment the target theme is cleared, so GTK reloads it.
  pub fallback_theme: String,
  pub font: String,
  pub icon_theme: String,
}

/// Applies the GTK settings through `gsettings`. Does nothing when `gsettings`
/// is not installed. Each `set` is best effort.
pub fn apply_gtk_runtime(request: &GtkRuntime) {
  if !command_exists("gsettings") {
    return;
  }
  gsettings_set("color-scheme", &request.scheme);
  gsettings_set("font-name", &request.font);
  gsettings_set("document-font-name", &request.font);
  // `gsettings get` prints the value quoted, e.g. `'Adwaita-dark'`.
  let current_theme = gsettings_get("gtk-theme");
  if current_theme == format!("'{}'", request.theme_name) {
    gsettings_set("gtk-theme", &request.fallback_theme);
    sleep(THEME_RESET_PAUSE);
  }
  gsettings_set("gtk-theme", &request.theme_name);
  gsettings_set("icon-theme", &request.icon_theme);
}

fn gsettings_set(key: &str, value: &str) {
  let _ = Command::new("gsettings")
    .args(["set", INTERFACE_SCHEMA, key, value])
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .status();
}

/// The value `gsettings get` prints, without its trailing newline. Empty when
/// the key cannot be read.
fn gsettings_get(key: &str) -> String {
  Command::new("gsettings")
    .args(["get", INTERFACE_SCHEMA, key])
    .stderr(Stdio::null())
    .output()
    .ok()
    .filter(|output| output.status.success())
    .map(|output| {
      String::from_utf8_lossy(&output.stdout)
        .trim_end_matches('\n')
        .to_string()
    })
    .unwrap_or_default()
}

/// Applies the wallpaper of the theme: the hyprpaper monitor and path lines,
/// then, when `runtime` is set, the `WALLPAPER_PATH` of the session. An empty
/// path is a no-op.
pub fn apply_wallpaper(hyprpaper_file: &Path, wallpaper: &str, runtime: bool) {
  if wallpaper.is_empty() {
    return;
  }
  let home = env::var("HOME").unwrap_or_default();
  if let Some(monitor) = active_monitor() {
    edit_hyprpaper(hyprpaper_file, Target::HyprpaperMonitor, &monitor);
  }
  edit_hyprpaper(
    hyprpaper_file,
    Target::HyprpaperPath,
    &display_path(wallpaper, &home),
  );
  if runtime {
    // The central reload starts the wallpaper after every config is ready.
    let _ = Command::new("systemctl")
      .args(["--user", "set-environment"])
      .arg(format!("WALLPAPER_PATH={wallpaper}"))
      .status();
  }
}

/// Writes the wallpaper path as hyprpaper reads it: a leading home directory
/// becomes `~`, as the shell's `sed "s|^$HOME|~|"` did.
pub fn display_path(wallpaper: &str, home: &str) -> String {
  // Only a path inside the home directory, at a directory boundary, is shortened:
  // `/home/bo` must not match `/home/boss/...`, and an empty home matches nothing.
  if home.is_empty() {
    return wallpaper.to_string();
  }
  let home = home.trim_end_matches('/');
  match wallpaper.strip_prefix(home) {
    Some(rest) if rest.is_empty() || rest.starts_with('/') => format!("~{rest}"),
    _ => wallpaper.to_string(),
  }
}

/// The name of the first monitor `hyprctl monitors` reports, if any.
pub fn active_monitor() -> Option<String> {
  if !command_exists("hyprctl") {
    return None;
  }
  let output = Command::new("hyprctl")
    .arg("monitors")
    .stderr(Stdio::null())
    .output()
    .ok()?;
  parse_monitor_name(&String::from_utf8_lossy(&output.stdout))
}

/// Reads the monitor name from the first `Monitor NAME ...` line.
pub fn parse_monitor_name(monitors: &str) -> Option<String> {
  monitors
    .lines()
    .find_map(|line| line.strip_prefix("Monitor "))
    .and_then(|rest| rest.split(' ').next())
    .filter(|name| !name.is_empty())
    .map(str::to_string)
}

/// Edits one line of the live hyprpaper configuration. A missing file is
/// reported and the switch continues, as the `sed -i` it replaces did.
fn edit_hyprpaper(file: &Path, target: Target, value: &str) {
  let file = file.to_path_buf();
  let result = fs::read_to_string(&file)
    .and_then(|text| text_edit::replace_file(&file, &text_edit::set_target(&text, target, value)));
  if let Err(error) = result {
    report(&file, &error);
  }
}

fn report(path: &Path, error: &std::io::Error) {
  eprintln!("argvus-config: {}: {error}", path.display());
}

fn command_exists(name: &str) -> bool {
  env::var_os("PATH")
    .map(|paths| env::split_paths(&paths).any(|directory| directory.join(name).is_file()))
    .unwrap_or(false)
}

/// What the GTK theme files need from a theme switch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GtkFiles {
  /// `light` or `dark`, written to the GTK mode marker.
  pub mode: String,
  pub gtk_theme_name: String,
  /// `1` when GTK should prefer its dark variant, `0` otherwise.
  pub prefer_dark: String,
  pub icon_theme: String,
  pub font: String,
}

const GTK_VERSIONS: [&str; 2] = ["gtk-3.0", "gtk-4.0"];

/// Writes the GTK settings of the theme into the ARGVUS and native GTK
/// directories, and records the mode. A failed step is
/// reported and the next one still runs, as the shell did.
pub fn apply_gtk_theme_files(paths: &SessionPaths, request: &GtkFiles) {
  let native_config = native_config_home();
  for version in GTK_VERSIONS {
    if let Err(error) = gtk_version_files(paths, &native_config, version, request) {
      eprintln!("argvus-config: {version}: {error}");
    }
  }
  let mode_file = paths.config_home.join("argvus/data/.gtk-mode");
  let written = mode_file
    .parent()
    .map(fs::create_dir_all)
    .unwrap_or(Ok(()))
    .and_then(|()| crate::text_edit::replace_file(&mode_file, &format!("{}\n", request.mode)));
  if let Err(error) = written {
    report(&mode_file, &error);
  }
}

fn gtk_version_files(
  paths: &SessionPaths,
  native_config: &Path,
  version: &str,
  request: &GtkFiles,
) -> io::Result<()> {
  let argvus_dir = paths.user_config(version);
  let native_dir = native_config.join(version);
  fs::create_dir_all(&argvus_dir)?;
  fs::create_dir_all(&native_dir)?;
  let entries = [
    ("gtk-theme-name", request.gtk_theme_name.as_str()),
    (
      "gtk-application-prefer-dark-theme",
      request.prefer_dark.as_str(),
    ),
    ("gtk-icon-theme-name", request.icon_theme.as_str()),
    ("gtk-font-name", request.font.as_str()),
    ("gtk-cursor-theme-name", "Adwaita"),
    ("gtk-cursor-theme-size", "24"),
  ];
  for settings in [
    argvus_dir.join("settings.ini"),
    native_dir.join("settings.ini"),
  ] {
    for (key, value) in entries {
      if let Err(error) = text_edit::set_ini_setting(&settings, key, value) {
        report(&settings, &error);
      }
    }
  }
  Ok(())
}

/// `$XDG_CONFIG_HOME`, or `$HOME/.config` when it is not set.
fn native_config_home() -> PathBuf {
  env::var_os("XDG_CONFIG_HOME")
    .filter(|value| !value.is_empty())
    .map(PathBuf::from)
    .unwrap_or_else(|| PathBuf::from(env::var_os("HOME").unwrap_or_default()).join(".config"))
}

/// The GTK theme of a light or a dark theme.
pub fn gtk_theme_name(light: bool) -> &'static str {
  if light { "Adwaita" } else { "Adwaita-dark" }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn home_prefix_becomes_tilde() {
    assert_eq!(
      display_path("/home/boss/Pictures/wall.jxl", "/home/boss"),
      "~/Pictures/wall.jxl"
    );
    assert_eq!(
      display_path("/usr/share/wall.jxl", "/home/boss"),
      "/usr/share/wall.jxl"
    );
  }

  #[test]
  fn first_monitor_line_names_the_monitor() {
    let output = "Monitor DP-1 (ID 0):\n\t2560x1440@144\nMonitor HDMI-A-1 (ID 1):\n";
    assert_eq!(parse_monitor_name(output).as_deref(), Some("DP-1"));
    assert_eq!(parse_monitor_name("no monitors here\n"), None);
  }

  #[test]
  fn empty_monitor_name_is_not_a_monitor() {
    assert_eq!(parse_monitor_name("Monitor  (ID 0):\n"), None);
  }
}

#[cfg(test)]
mod display_path_tests {
  use super::display_path;

  #[test]
  fn shortens_only_inside_the_home_directory() {
    assert_eq!(display_path("/home/boss/x.jxl", "/home/boss"), "~/x.jxl");
    assert_eq!(display_path("/home/boss", "/home/boss"), "~");
    assert_eq!(display_path("/home/boss/x", "/home/boss/"), "~/x");
    assert_eq!(display_path("/home/boss2/x", "/home/boss"), "/home/boss2/x");
    assert_eq!(display_path("/usr/share/x", ""), "/usr/share/x");
  }
}
