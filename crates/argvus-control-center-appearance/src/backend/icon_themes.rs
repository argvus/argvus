//! Icon themes and the launcher icon choice.

use super::commands::{run_script, script};
use super::taskbar::reload_taskbar;
use super::themes::refresh_official_themes;
use super::*;

/// Reads `/appearance/icon_theme` through `argvus config get`. A failed read
/// resolves to the ARGVUS default, so the Icons page still opens.
pub fn read_configured_icon_theme(default: &str) -> String {
  let output = argvus_control_center_core::process::command("argvus")
    .args(["config", "get", "/appearance/icon_theme", "--raw"])
    .output()
    .ok()
    .filter(|output| output.status.success())
    .map(|output| String::from_utf8_lossy(&output.stdout).into_owned());
  crate::icons::configured_icon_theme(output.as_deref(), default)
}

/// Loads the configured icon theme and the installed icon themes for the
/// Icons page.
pub(crate) fn load_icon_themes(state: &mut AppearanceState) {
  // The empty-configuration default depends on the mode of the active theme,
  // so the official themes (which declare their category) must be loaded.
  refresh_official_themes(state);
  let default = crate::icons::default_icon_theme_for(&state.theme, &state.official_themes);
  state.icon_theme = read_configured_icon_theme(default);
  state.icon_themes = crate::icons::installed_icon_themes();
}

/// Writes the icon theme to `/appearance/icon_theme` and reloads the config
/// service so consumers read the new value.
pub(super) fn persist_icon_theme_canonical(name: &str) -> Result<(), String> {
  let patch = serde_json::json!({
    "/appearance/icon_theme": name,
  });
  let patch = serde_json::to_string(&patch).map_err(|error| error.to_string())?;
  let status = argvus_control_center_core::process::command("argvus")
    .arg("config")
    .args(["patch", &patch])
    .status()
    .map_err(|error| format!("failed to persist icon theme settings: {error}"))?;
  if !status.success() {
    return Err("argvus config rejected the icon theme settings".into());
  }
  reload_argvus_config_service()
}

/// Selects an installed icon theme and applies it to GNOME, GTK and Qt.
/// The name is checked before anything is written, so an unknown or
/// unsafe name leaves the configuration untouched.
pub fn set_icon_theme(name: &str) -> Result<(), String> {
  crate::icons::validate_icon_theme_name(name)?;
  let installed = crate::icons::installed_icon_themes()
    .iter()
    .any(|entry| entry.directory_name == name);
  if !installed {
    return Err(format!("icon theme is not installed: {name}"));
  }
  persist_icon_theme_canonical(name)?;
  run_script(&script("icon-switch.sh"), &["apply"])
}

/// Persists the taskbar launcher's custom icon path to the canonical
/// document and re-derives the generated Waybar config from it, restarting
/// the taskbar so the new icon appears immediately. Unlike the Enable
/// toggle (see `apply_taskbar_icons_and_format`), picking a custom icon is
/// an immediate action, not part of the Taskbar > Icons draft/Apply cycle —
/// the same way `persist_wallpaper_canonical` applies outside the draft.
pub(super) fn persist_launcher_icon_canonical(path: &Path) -> Result<(), String> {
  let patch = serde_json::json!({
    "/taskbar/icons/launcher_custom_icon_path": path.to_string_lossy(),
  });
  let patch = serde_json::to_string(&patch).map_err(|error| error.to_string())?;
  let status = argvus_control_center_core::process::command("argvus")
    .arg("config")
    .args(["patch", &patch])
    .status()
    .map_err(|error| format!("failed to persist launcher icon settings: {error}"))?;
  if !status.success() {
    return Err("argvus config rejected the launcher icon settings".into());
  }
  run_script(&script("taskbar-widgets-mode.sh"), &["apply"])?;
  reload_taskbar()
}

/// Opens the configured File Manager so the user can pick a custom launcher
/// icon (PNG/SVG) for the taskbar, in a window focused on top of the Control
/// Center. Mirrors `choose_wallpaper`'s GUI/TUI flow.
pub fn choose_launcher_icon() -> Result<(), String> {
  let Some(selected) = argvus_control_center_core::picker::pick_file(
    "argvus-launcher-icon-picker",
    "launcher-icon-selection",
  )?
  else {
    return Ok(());
  };
  persist_launcher_icon_canonical(&selected)
}
