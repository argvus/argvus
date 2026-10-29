//! Defaults of the configuration document: built-in values, packaged defaults and the sections that must exist as objects.

use super::config_validation::{BLUR_SETTINGS_DEFAULTS, SHADOW_SETTINGS_DEFAULTS};
use super::legacy_import::manifest_shortcut_defaults;
use super::*;

/// Stored objects that `apply_defaults` descends into. A value at one of these
/// paths that is not an object cannot take defaults; it is reported as invalid
/// configuration instead of stopping the command.
pub(super) const DEFAULTED_OBJECTS: &[&[&str]] = &[
  &["appearance"],
  &["layout"],
  &["layout", "taskbar"],
  &["layout", "window"],
  &["effects"],
  &["taskbar"],
  &["taskbar", "icons"],
  &["taskbar", "icons", "utilities"],
  &["taskbar", "date"],
  &["taskbar", "time"],
  &["taskbar", "shape"],
  &["default_apps"],
  &["keyboard_shortcuts"],
  &["hyprland"],
  &["hyprland", "input"],
  &["hyprland", "input", "mouse"],
  &["hyprland", "input", "touchpad"],
  &["power"],
  &["audio"],
];

pub(super) fn check_defaulted_objects(document: &ConfigDocument) -> ConfigResult<()> {
  for path in DEFAULTED_OBJECTS {
    let mut current = document.sections.get(path[0]);
    for key in &path[1..] {
      current = current.and_then(|value| value.get(key));
    }
    if current.is_some_and(|value| !value.is_object()) {
      return Err(ConfigError::Invalid(format!(
        "{} must be an object",
        path.join(".")
      )));
    }
  }
  Ok(())
}

/// Fills every missing key with its built-in default. Existing values are
/// never overwritten; packaged defaults are applied separately.
pub(super) fn apply_defaults(document: &mut ConfigDocument) -> ConfigResult<()> {
  check_defaulted_objects(document)?;
  let theme = default_appearance(document);
  default_layout(document, &theme);
  default_effects(document);
  default_taskbar(document);
  default_applications(document);
  default_keyboard_shortcuts(document);
  default_hyprland_input(document);
  default_power(document);
  default_audio(document);
  Ok(())
}

/// Theme, accent, wallpaper, GTK mode and icon theme. Returns the theme, which
/// the layout defaults depend on.
fn default_appearance(document: &mut ConfigDocument) -> String {
  let appearance = document
    .sections
    .entry("appearance".into())
    .or_insert_with(|| Value::Object(Map::new()));
  let object = appearance.as_object_mut().expect("appearance is object");
  object
    .entry("accent")
    .or_insert_with(|| Value::String("#3590BD".into()));
  object
    .entry("accent_custom")
    .or_insert_with(|| Value::Bool(false));
  object
    .entry("theme")
    .or_insert_with(|| Value::String("argvus-dark".into()));
  object
    .entry("wallpaper")
    .or_insert_with(|| Value::String("/usr/share/backgrounds/argvus/argvus-dark.jxl".into()));
  object
    .entry("wallpaper_custom")
    .or_insert_with(|| Value::Bool(false));
  object
    .entry("gtk_mode")
    .or_insert_with(|| Value::String("dark".into()));
  // Empty means "use the ARGVUS default icon theme" (see `argvus-icons`).
  object
    .entry("icon_theme")
    .or_insert_with(|| Value::String(String::new()));
  object
    .get("theme")
    .and_then(Value::as_str)
    .unwrap_or("argvus-dark")
    .to_owned()
}

/// Layout variant, window gaps and taskbar margins.
fn default_layout(document: &mut ConfigDocument, theme: &str) {
  let layout = document
    .sections
    .entry("layout".into())
    .or_insert_with(|| Value::Object(Map::new()));
  let layout = layout.as_object_mut().expect("layout is object");
  // Sticky/Float is independent of the theme (see `apply_layout_variant`);
  // `theme` here only matters for the legacy `-float`-suffixed installs
  // that `migrate_theme_suffix_to_variant` has not rewritten yet.
  layout
    .entry("variant")
    .or_insert_with(|| Value::String(theme_layout_variant(theme).into()));
  let taskbar_bottom_default = match layout.get("variant").and_then(Value::as_str) {
    Some("float") => 18,
    _ => 1,
  };
  {
    let taskbar = layout
      .entry("taskbar")
      .or_insert_with(|| Value::Object(Map::new()))
      .as_object_mut()
      .expect("layout.taskbar is object");
    for (key, value) in [
      ("margin_bottom", Value::from(taskbar_bottom_default)),
      ("margin_left", Value::from(0)),
      ("margin_right", Value::from(0)),
      ("margin_top", Value::from(0)),
      ("position", Value::String("top".into())),
    ] {
      taskbar.entry(key).or_insert(value);
    }
  }
  {
    let window = layout
      .entry("window")
      .or_insert_with(|| Value::Object(Map::new()))
      .as_object_mut()
      .expect("layout.window is object");
    for (key, value) in [
      ("border_size", Value::from(1)),
      ("gaps_in", Value::from(2)),
      ("gaps_out_bottom", Value::from(0)),
      ("gaps_out_left", Value::from(0)),
      ("gaps_out_right", Value::from(0)),
      ("gaps_out_top", Value::from(0)),
      ("rounded", Value::Bool(false)),
      ("rounding", Value::from(0)),
    ] {
      window.entry(key).or_insert(value);
    }
  }
}

/// Animations, blur, transparency per surface and the blur settings.
fn default_effects(document: &mut ConfigDocument) {
  let effects = document
    .sections
    .entry("effects".into())
    .or_insert_with(|| Value::Object(Map::new()));
  let effects = effects.as_object_mut().expect("effects is object");
  effects
    .entry("animations")
    .or_insert_with(|| Value::Bool(true));
  effects
    .entry("widget_telemetry_enabled")
    .or_insert_with(|| Value::Bool(true));
  effects
    .entry("blur_global_enabled")
    .or_insert_with(|| Value::Bool(true));
  for (surface, transparency) in [
    ("taskbar", 50),
    ("launchers", 15),
    ("widget-telemetry", 50),
    ("control-panel", 50),
    ("terminal", 50),
    ("control-center", 15),
  ] {
    effects
      .entry(format!("transparency_{surface}_enabled"))
      .or_insert_with(|| Value::Bool(true));
    effects
      .entry(format!("transparency_{surface}_value"))
      .or_insert_with(|| Value::Number(transparency.into()));
  }
  for (key, value) in BLUR_SETTINGS_DEFAULTS
    .into_iter()
    .chain(SHADOW_SETTINGS_DEFAULTS)
  {
    effects.entry(key).or_insert_with(|| {
      // Whole defaults are written as integers, like effects.json, so the
      // integer fields (size, passes) read back without a fractional part.
      if value.fract() == 0.0 {
        Value::from(value as i64)
      } else {
        Value::from(value)
      }
    });
  }
  effects
    .entry("shadow_enabled")
    .or_insert_with(|| Value::Bool(false));
  effects
    .entry("shadow_sharp")
    .or_insert_with(|| Value::Bool(false));
  effects
    .entry("shadow_color")
    .or_insert_with(|| Value::String(String::new()));
  effects
    .entry("transparency_control-center_enabled")
    .or_insert_with(|| Value::Bool(true));
  effects
    .entry("transparency_control-center_value")
    .or_insert_with(|| Value::Number(50.into()));
  effects
    .entry("blur_control-center_enabled")
    .or_insert_with(|| Value::Bool(true));
  // Per-surface blur switches were never read by the compositor or the
  // surfaces; blur follows the global switch. Drop them from existing profiles.
  for surface in [
    "taskbar",
    "launchers",
    "widget-telemetry",
    "control-panel",
    "terminal",
  ] {
    effects.remove(&format!("blur_{surface}_enabled"));
  }
}

/// Taskbar icons, utilities, date, time and shape.
fn default_taskbar(document: &mut ConfigDocument) {
  let taskbar = document
    .sections
    .entry("taskbar".into())
    .or_insert_with(|| Value::Object(Map::new()));
  let taskbar = taskbar.as_object_mut().expect("taskbar is object");
  let icons = taskbar
    .entry("icons")
    .or_insert_with(|| Value::Object(Map::new()));
  let icons = icons.as_object_mut().expect("taskbar.icons is object");
  icons
    .entry("audio_player_enabled")
    .or_insert_with(|| Value::Bool(true));
  icons
    .entry("focus_enabled")
    .or_insert_with(|| Value::Bool(true));
  icons
    .entry("launcher_enabled")
    .or_insert_with(|| Value::Bool(false));
  icons
    .entry("launcher_custom_icon_path")
    .or_insert(Value::Null);
  for widget in [
    "network",
    "power_profile",
    "keyboard_layout",
    "memory",
    "cpu",
    "cpu_temperature",
    "gpu_temperature",
  ] {
    icons
      .entry(format!("{widget}_enabled"))
      .or_insert_with(|| Value::Bool(true));
  }
  let utilities = icons
    .entry("utilities")
    .or_insert_with(|| Value::Object(Map::new()));
  let utilities = utilities
    .as_object_mut()
    .expect("taskbar.icons.utilities is object");
  utilities
    .entry("expanded")
    .or_insert_with(|| Value::Bool(true));
  let date = taskbar
    .entry("date")
    .or_insert_with(|| Value::Object(Map::new()));
  let date = date.as_object_mut().expect("taskbar.date is object");
  date
    .entry("format")
    .or_insert_with(|| Value::String("weekday_day_month".into()));
  let time = taskbar
    .entry("time")
    .or_insert_with(|| Value::Object(Map::new()));
  let time = time.as_object_mut().expect("taskbar.time is object");
  time
    .entry("seconds_enabled")
    .or_insert_with(|| Value::Bool(false));
  time
    .entry("format")
    .or_insert_with(|| Value::String("24h".into()));
  // An unset height or border follows the packaged taskbar and the theme class.
  let shape = taskbar
    .entry("shape")
    .or_insert_with(|| Value::Object(Map::new()));
  let shape = shape.as_object_mut().expect("taskbar.shape is object");
  shape.entry("height").or_insert(Value::Null);
  shape.entry("border_width").or_insert(Value::Null);
}

/// Default application of each category.
fn default_applications(document: &mut ConfigDocument) {
  let default_apps = document
    .sections
    .entry("default_apps".into())
    .or_insert_with(|| Value::Object(Map::new()));
  let default_apps = default_apps
    .as_object_mut()
    .expect("default_apps is object");
  for (category, application) in [
    ("terminal", "argvus-terminal"),
    ("file_manager", "spf"),
    ("text_editor", "mousepad"),
    ("terminal_editor", "vim"),
    ("browser", "firefox"),
    ("image_viewer", "imv"),
    ("pdf_viewer", "zathura"),
    ("video_player", "mpv"),
    ("audio_player", "audacious"),
    ("archive", "xarchiver"),
    ("launcher", "rofi"),
  ] {
    default_apps
      .entry(category)
      .or_insert_with(|| Value::String(application.into()));
  }
}

/// Keyboard shortcuts from the packaged manifest.
fn default_keyboard_shortcuts(document: &mut ConfigDocument) {
  let keyboard_shortcuts = document
    .sections
    .entry("keyboard_shortcuts".into())
    .or_insert_with(|| Value::Object(Map::new()));
  let keyboard_shortcuts = keyboard_shortcuts
    .as_object_mut()
    .expect("keyboard_shortcuts is object");
  for (config_key, shortcut) in manifest_shortcut_defaults() {
    keyboard_shortcuts.entry(config_key).or_insert(shortcut);
  }
}

/// Mouse and touchpad settings of Hyprland.
fn default_hyprland_input(document: &mut ConfigDocument) {
  let hyprland = document
    .sections
    .entry("hyprland".into())
    .or_insert_with(|| Value::Object(Map::new()));
  let hyprland = hyprland.as_object_mut().expect("hyprland is object");
  let input = hyprland
    .entry("input")
    .or_insert_with(|| Value::Object(Map::new()));
  let input = input.as_object_mut().expect("hyprland.input is object");
  let mouse = input
    .entry("mouse")
    .or_insert_with(|| Value::Object(Map::new()));
  let mouse = mouse
    .as_object_mut()
    .expect("hyprland.input.mouse is object");
  for (key, value) in [
    ("accel_profile", Value::String("flat".into())),
    ("left_handed", Value::Bool(false)),
    ("natural_scroll", Value::Bool(false)),
    ("scroll_factor", Value::from(1.0)),
    ("sensitivity", Value::from(0.0)),
  ] {
    mouse.entry(key).or_insert(value);
  }
  let touchpad = input
    .entry("touchpad")
    .or_insert_with(|| Value::Object(Map::new()));
  let touchpad = touchpad
    .as_object_mut()
    .expect("hyprland.input.touchpad is object");
  for (key, value) in [
    ("disable_while_typing", Value::Bool(true)),
    ("natural_scroll", Value::Bool(false)),
    ("tap_and_drag", Value::Bool(true)),
    ("tap_to_click", Value::Bool(true)),
    ("two_finger_right_click", Value::Bool(false)),
  ] {
    touchpad.entry(key).or_insert(value);
  }
}

/// Lock, screen-off and keep-awake settings.
fn default_power(document: &mut ConfigDocument) {
  let power = document
    .sections
    .entry("power".into())
    .or_insert_with(|| Value::Object(Map::new()));
  let power = power.as_object_mut().expect("power is object");
  power.entry("lock_minutes").or_insert(30.into());
  power.entry("screen_off_minutes").or_insert(0.into());
  power
    .entry("keep_awake")
    .or_insert_with(|| Value::Bool(false));
}

/// Output volume and mute state.
fn default_audio(document: &mut ConfigDocument) {
  let audio = document
    .sections
    .entry("audio".into())
    .or_insert_with(|| Value::Object(Map::new()));
  let audio = audio.as_object_mut().expect("audio is object");
  audio.entry("output_volume").or_insert(50.into());
  audio
    .entry("output_muted")
    .or_insert_with(|| Value::Bool(false));
}

pub(super) fn apply_packaged_defaults(document: &mut ConfigDocument) -> ConfigResult<()> {
  let system_config = env::var_os("ARGVUS_SYSTEM_CONFIG")
    .map(PathBuf::from)
    .unwrap_or_else(|| PathBuf::from(session_paths::PACKAGED_SYSTEM_CONFIG));
  let defaults_root = system_config.join(MODULE_DIRECTORY);
  for section in KNOWN_SECTIONS {
    if document.sections.contains_key(section) {
      continue;
    }
    let path = defaults_root.join(format!("{section}.json"));
    if !path.is_file() {
      continue;
    }
    let value: Value = serde_json::from_str(&fs::read_to_string(path)?)?;
    if !value.is_object() {
      return Err(ConfigError::Invalid(format!(
        "section {section} must be a JSON object"
      )));
    }
    document.sections.insert(section.to_owned(), value);
  }
  Ok(())
}
