//! JSON pointers into the ARGVUS configuration document (`config.json`).
//!
//! Keys that several modules read or write live here so a renamed key is
//! changed in one place. Keys used by a single module stay next to that code.

pub(crate) const APPEARANCE_THEME: &str = "/appearance/theme";
pub(crate) const APPEARANCE_ACCENT: &str = "/appearance/accent";
pub(crate) const APPEARANCE_ACCENT_CUSTOM: &str = "/appearance/accent_custom";
pub(crate) const APPEARANCE_GTK_MODE: &str = "/appearance/gtk_mode";
pub(crate) const APPEARANCE_WALLPAPER: &str = "/appearance/wallpaper";

pub(crate) const LAYOUT_VARIANT: &str = "/layout/variant";

pub(crate) const TASKBAR_SHAPE_HEIGHT: &str = "/taskbar/shape/height";
pub(crate) const TASKBAR_SHAPE_BORDER_WIDTH: &str = "/taskbar/shape/border_width";

pub(crate) const EFFECTS_BLUR_GLOBAL_ENABLED: &str = "/effects/blur_global_enabled";
pub(crate) const EFFECTS_ANIMATIONS: &str = "/effects/animations";
pub(crate) const EFFECTS_CONTROL_CENTER_TRANSPARENCY: &str =
  "/effects/transparency_control-center_value";

pub(crate) const DEFAULT_APPS: &str = "/default_apps";
pub(crate) const KEYBOARD_SHORTCUTS: &str = "/keyboard_shortcuts";
pub(crate) const REMOVABLE_DEVICES: &str = "/removable_devices";
pub(crate) const FONTS_LEGACY: &str = "/fonts/legacy";
pub(crate) const POWER_KEEP_AWAKE: &str = "/power/keep_awake";
pub(crate) const SESSION_LANGUAGE: &str = "/session/language";

pub(crate) const HYPRLAND_INPUT: &str = "/hyprland/input";
pub(crate) const HYPRLAND_KEYBOARD: &str = "/hyprland/keyboard";

pub(crate) const DISPLAYS: &str = "/displays";
pub(crate) const DISPLAYS_MONITORS: &str = "/displays/monitors";
