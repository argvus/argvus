//! The last steps of a theme switch: the accent, the Hyprland geometry, the
//! wallpaper and the effects.
//!
//! The Hyprland helpers belong to another package, so they stay scripts and
//! run as processes. Each failure returns the key of its message, which the
//! switch translates and reports as it always did.

use crate::accent;
use crate::appearance_runtime;
use crate::effects;
use crate::session_paths::SessionPaths;
use crate::user_copy;
use std::env;
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

/// What the finishing steps need from the theme switch.
pub struct Finish<'a> {
  /// The theme's wallpaper, or empty when it has none.
  pub wallpaper: &'a str,
  pub runtime: bool,
}

/// Runs the finishing steps in order. The error is the key of the message the
/// switch shows, without its `theme.` prefix.
pub fn run(paths: &SessionPaths, finish: &Finish) -> Result<(), &'static str> {
  restore_accent()?;
  run_hypr_geometry(paths)?;
  clear_custom_wallpaper(paths);
  if let Some(file) = user_copy::config_path(paths, "appearance/config/hypr/hyprpaper.conf") {
    appearance_runtime::apply_wallpaper(&file, finish.wallpaper, finish.runtime);
  }
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
  // The switch has written the build copies and reloads the consumers itself.
  let status = Command::new(executable)
    .args(["config", "accent", mode])
    .env("ARGVUS_THEME_SWITCH", "1")
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .status();
  match status {
    Ok(status) if status.success() => Ok(()),
    _ => Err("accent_restore_failed"),
  }
}

/// The Hyprland spacing and borders of the selected mode: reset, then apply the
/// borders with the user's layout.
fn run_hypr_geometry(paths: &SessionPaths) -> Result<(), &'static str> {
  if let Some(spaces) =
    user_copy::config_path(paths, "hyprland/sh/spaces-switch.sh").filter(|path| path.is_file())
  {
    run_script(&spaces, "--reset").map_err(|()| "spaces_failed")?;
  }
  if let Some(borders) =
    user_copy::config_path(paths, "hyprland/sh/borders-switch.sh").filter(|path| path.is_file())
  {
    run_script(&borders, "--reset").map_err(|()| "borders_failed")?;
    run_script(&borders, "--apply-static").map_err(|()| "borders_failed")?;
  }
  Ok(())
}

/// Runs one of the Hyprland helper scripts without its runtime step: the
/// switch applies the geometry to the running compositor itself.
fn run_script(script: &Path, argument: &str) -> Result<(), ()> {
  match Command::new("sh")
    .arg(script)
    .arg(argument)
    .env("ARGVUS_NO_RUNTIME", "1")
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .status()
  {
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
