//! The last steps of a theme switch: the accent, the lock screen and the
//! Hyprland geometry, the wallpaper, the terminal signal, the removable-device
//! and calendar stylesheets, and the effects.
//!
//! The lock-screen and Hyprland helpers belong to other packages, so they stay
//! scripts and run as processes. Each failure returns the key of its message,
//! which the shell translates and reports as the switch always did.

use crate::accent;
use crate::appearance_runtime;
use crate::effects;
use crate::session_paths::SessionPaths;
use crate::theme_surfaces::{self, StorageLocations};
use crate::user_copy;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// What the finishing steps need from the theme switch.
pub struct Finish<'a> {
  pub theme: &'a str,
  /// The theme's wallpaper, or empty when it has none.
  pub wallpaper: &'a str,
  /// The development workspace root, or empty.
  pub workspace_root: &'a str,
  pub runtime: bool,
}

/// Runs the finishing steps in order. The error is the key of the message the
/// shell shows, without its `theme.` prefix.
pub fn run(paths: &SessionPaths, finish: &Finish) -> Result<(), &'static str> {
  restore_accent()?;
  run_lock_theme(paths)?;
  run_hypr_geometry(paths)?;
  clear_custom_wallpaper(paths);
  if let Some(file) = user_copy::config_path(paths, "appearance/config/hypr/hyprpaper.conf") {
    appearance_runtime::apply_wallpaper(&file, finish.wallpaper, finish.runtime);
  }
  if finish.runtime {
    signal_kitty();
  }
  apply_storage_and_calendar(paths, finish);
  effects::run(&["apply".to_string()]);
  Ok(())
}

/// The accent of the theme. An explicit official selection, or no custom
/// accent, restores the theme's default; a custom accent is kept as it is.
fn restore_accent() -> Result<(), &'static str> {
  let official = env::var("ARGVUS_ACCENT_OFFICIAL").as_deref() == Ok("1");
  let custom = !official && {
    // A configuration that cannot be read does not say whether the accent is custom.
    // Guessing "not custom" would overwrite the user's color, so the switch fails.
    let store = crate::ConfigStore::from_environment().map_err(|_| "accent_restore_failed")?;
    store.load().map_err(|_| "accent_restore_failed")?;
    accent::canonical_accent(Some(&store)).is_some()
  };
  let mode = if custom {
    "--apply-static"
  } else {
    "--theme-default"
  };
  let Ok(executable) = env::current_exe() else {
    return Err("accent_restore_failed");
  };
  let status = Command::new(executable)
    .args(["config", "accent", mode])
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .status();
  match status {
    Ok(status) if status.success() => Ok(()),
    _ => Err("accent_restore_failed"),
  }
}

/// The lock screen theme, invalidated after the change, when the package has it.
fn run_lock_theme(paths: &SessionPaths) -> Result<(), &'static str> {
  let Some(script) =
    user_copy::config_path(paths, "lock/sh/hyprlock-theme.sh").filter(|path| path.is_file())
  else {
    return Ok(());
  };
  run_script(&script, &["--invalidate"], false).map_err(|()| "hyprlock_failed")
}

/// The Hyprland spacing and borders of the selected mode: reset, then apply the
/// borders with the user's layout.
fn run_hypr_geometry(paths: &SessionPaths) -> Result<(), &'static str> {
  if let Some(spaces) =
    user_copy::config_path(paths, "hyprland/sh/spaces-switch.sh").filter(|path| path.is_file())
  {
    run_script(&spaces, &["--reset"], true).map_err(|()| "spaces_failed")?;
  }
  if let Some(borders) =
    user_copy::config_path(paths, "hyprland/sh/borders-switch.sh").filter(|path| path.is_file())
  {
    run_script(&borders, &["--reset"], true).map_err(|()| "borders_failed")?;
    run_script(&borders, &["--apply-static"], true).map_err(|()| "borders_failed")?;
  }
  Ok(())
}

/// Runs one of the Hyprland helper scripts. `no_runtime` sets `ARGVUS_NO_RUNTIME`.
fn run_script(script: &Path, arguments: &[&str], no_runtime: bool) -> Result<(), ()> {
  let mut command = Command::new("sh");
  command
    .arg(script)
    .args(arguments)
    .stdout(Stdio::null())
    .stderr(Stdio::null());
  if no_runtime {
    command.env("ARGVUS_NO_RUNTIME", "1");
  }
  match command.status() {
    Ok(status) if status.success() => Ok(()),
    _ => Err(()),
  }
}

/// A theme change is an explicit request for the theme's wallpaper, so the
/// custom selection is cleared, in the data copy and in the legacy place.
fn clear_custom_wallpaper(paths: &SessionPaths) {
  let _ = fs::remove_file(paths.config_home.join("argvus/data/.wallpaper-custom"));
  let _ = fs::remove_file(paths.config_home.join("argvus/.wallpaper-custom"));
}

/// Tells the running kitty instances to reload their configuration.
fn signal_kitty() {
  // Only this user's kitty instances: other users' processes cannot be signalled
  // anyway, and must not be enumerated or targeted.
  let uid = crate::theme_surfaces::current_user_id().to_string();
  let Ok(output) = Command::new("pgrep")
    .args(["-u", uid.as_str(), "-x", "kitty"])
    .stderr(Stdio::null())
    .output()
  else {
    return;
  };
  for pid in String::from_utf8_lossy(&output.stdout).lines() {
    if let Ok(pid) = pid.trim().parse::<i32>() {
      // SAFETY: SIGUSR1 to a pid that pgrep just reported.
      unsafe {
        libc::kill(pid, libc::SIGUSR1);
      }
    }
  }
}

/// The removable-device panel and the calendar stylesheets. Their failures do
/// not stop the switch, as before.
fn apply_storage_and_calendar(paths: &SessionPaths, finish: &Finish) {
  let themes_dir = paths.system_config("appearance/themes.d");
  let workspace = (!finish.workspace_root.is_empty()).then(|| PathBuf::from(finish.workspace_root));
  let user_dir =
    user_copy::config_path(paths, "removable-devices/config/themes").unwrap_or_default();
  let destination =
    user_copy::config_path(paths, "removable-devices/config/theme.css").unwrap_or_default();
  let system_dir = paths.system_config("removable-devices/config/themes");
  let locations = StorageLocations {
    user_dir: &user_dir,
    system_dir: &system_dir,
    destination: &destination,
    workspace_root: workspace.as_deref(),
  };
  let _ = theme_surfaces::apply_storage_theme(&locations, &themes_dir, finish.theme);

  let cache_home = env::var_os("XDG_CACHE_HOME")
    .filter(|value| !value.is_empty())
    .map(PathBuf::from)
    .unwrap_or_else(|| PathBuf::from(env::var_os("HOME").unwrap_or_default()).join(".cache"));
  let cache_file = cache_home.join("argvus-taskbar-calendar/theme.css");
  let _ = theme_surfaces::apply_calendar_theme(
    paths,
    &themes_dir,
    &cache_file,
    finish.theme,
    workspace.as_deref(),
  );
}
