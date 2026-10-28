//! Active theme and accent: discovering the theme default accent, switching themes and setting the accent.

use super::commands::run_argvus_route;
use super::*;

pub(super) fn active_theme_file() -> PathBuf {
  argvus_data_home().join(".active-theme")
}

pub(super) fn accent_file() -> PathBuf {
  argvus_data_home().join(".accent-color")
}

/// Reads the default accent declared by an installed theme manifest. Drop-in
/// theme packages are discovered at runtime, so this needs no per-theme entry.
pub(super) fn discovered_default_accent(theme: &str) -> Option<String> {
  discovered_default_accent_in(&system_config_root(), theme)
}

pub(super) fn discovered_default_accent_in(system_config: &Path, theme: &str) -> Option<String> {
  let family = theme.strip_suffix("-float").unwrap_or(theme);
  argvus_theme::discovery::discover_themes(system_config)
    .themes
    .into_iter()
    .find(|entry| entry.id == family)
    .and_then(|entry| normalize_hex_color(&entry.accent))
}

/// Discovers and loads official themes (built-in + drop-in packages).
/// Errors during discovery are collected as warnings and do not fail the operation.
pub(super) fn refresh_official_themes(state: &mut AppearanceState) {
  let report = argvus_theme::discovery::discover_themes(&system_config_root());
  state.official_themes = report.themes;
  state.discovery_warnings = report.warnings;
}

/// The theme's own highlight colour comes with it, for an official theme and a
/// user theme alike.
pub fn set_theme(name: &str) -> Result<(), String> {
  let canonical_name = canonical_theme_id(name);
  run_argvus_route(
    "theme-switch",
    &[canonical_name.as_str()],
    &[("ARGVUS_ACCENT_OFFICIAL", "1")],
  )
}

/// A user-selected highlight color is a canonical decision, not a runtime
/// detail, so `/appearance/accent_custom` is persisted before the appearance
/// adapters run. Without that key the compositor keeps the per-theme border
/// color (`hyprland.lua` only honors `.accent-color` when the canonical flag is
/// set), and the next `argvus-config project` would otherwise re-derive the
/// accent from the theme default.
pub fn set_accent(color: &str) -> Result<(), String> {
  if color == "--theme-default" {
    return reset_accent_to_theme_default();
  }
  // `argvus config accent` writes the canonical custom accent and applies it in
  // one command; there is no separate patch step that could leave them apart.
  run_argvus_route("config", &["accent", color], &[])
}

/// Restores the active theme's default highlight color and clears the custom
/// flag so a later theme switch keeps the theme owning the accent again.
pub(super) fn reset_accent_to_theme_default() -> Result<(), String> {
  run_argvus_route("config", &["accent", "--theme-default"], &[])
}
