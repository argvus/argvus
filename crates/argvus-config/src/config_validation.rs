//! Validation of each configuration section and the allowed values of effects and animations.

use super::*;

pub(super) fn validate_appearance(document: &ConfigDocument) -> ConfigResult<()> {
  let Some(appearance) = document.sections.get("appearance") else {
    return Ok(());
  };
  let object = appearance.as_object().expect("validated section object");
  if let Some(theme) = object.get("theme")
    && theme.as_str().is_none_or(|value| value.trim().is_empty())
  {
    return Err(ConfigError::Invalid(
      "appearance.theme must be a non-empty string".into(),
    ));
  }
  if let Some(accent) = object.get("accent") {
    let valid = accent.as_str().is_some_and(|value| {
      let value = value.strip_prefix('#').unwrap_or(value);
      value.len() == 6 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
    });
    if !valid {
      return Err(ConfigError::Invalid(
        "appearance.accent must be #RRGGBB".into(),
      ));
    }
  }
  if let Some(accent_custom) = object.get("accent_custom")
    && !accent_custom.is_boolean()
  {
    return Err(ConfigError::Invalid(
      "appearance.accent_custom must be a boolean".into(),
    ));
  }
  for key in ["wallpaper_custom"] {
    if object.get(key).is_some_and(|value| !value.is_boolean()) {
      return Err(ConfigError::Invalid(format!(
        "appearance.{key} must be a boolean"
      )));
    }
  }
  if let Some(mode) = object.get("gtk_mode")
    && !mode
      .as_str()
      .is_some_and(|value| matches!(value, "light" | "dark" | "auto" | "sticky"))
  {
    return Err(ConfigError::Invalid(
      "appearance.gtk_mode is invalid".into(),
    ));
  }
  if let Some(icon_theme) = object.get("icon_theme") {
    let valid = icon_theme.as_str().is_some_and(|value| {
      value.len() <= 128 && value != "." && value != ".." && !value.contains(['/', '\0'])
    });
    if !valid {
      return Err(ConfigError::Invalid(
        "appearance.icon_theme must be a plain icon theme name".into(),
      ));
    }
  }
  Ok(())
}

pub(super) fn validate_layout(document: &ConfigDocument) -> ConfigResult<()> {
  validate_numeric_section(document, "layout", 0, 100)?;
  if let Some(variant) = document
    .get(json_pointers::LAYOUT_VARIANT)
    .and_then(Value::as_str)
    && !matches!(variant, "sticky" | "float")
  {
    return Err(ConfigError::Invalid(
      "layout.variant must be sticky or float".into(),
    ));
  }
  Ok(())
}

/// Hyprland blur parameters stored under `effects`. Each entry is the key, the
/// inclusive range accepted by the schema and whether it must be an integer.
pub(super) const BLUR_SETTINGS: [(&str, f64, f64, bool); 7] = [
  ("blur_size", 1.0, 64.0, true),
  ("blur_passes", 1.0, 8.0, true),
  ("blur_brightness", 0.0, 2.0, false),
  ("blur_noise", 0.0, 1.0, false),
  ("blur_contrast", 0.0, 2.0, false),
  ("blur_vibrancy", 0.0, 1.0, false),
  ("blur_vibrancy_darkness", 0.0, 1.0, false),
];

/// Defaults written into new profiles. They match Hyprland's blur parameters
/// as shown in the Control Center and must stay in sync with `effects.json`.
pub(super) const BLUR_SETTINGS_DEFAULTS: [(&str, f64); 7] = [
  ("blur_size", 6.0),
  ("blur_passes", 2.0),
  ("blur_brightness", 1.0),
  ("blur_noise", 0.0),
  ("blur_contrast", 0.9),
  ("blur_vibrancy", 0.1),
  ("blur_vibrancy_darkness", 0.0),
];

/// Styles that render on the window open/close leaves in Hyprland 0.56.2. The
/// compositor accepts `slidefade`, `slidevert` and `slidefadevert` here too, but
/// it draws them as pop-in, so they are not offered.
pub(super) const WINDOW_ANIMATION_STYLES: [&str; 3] = ["slide", "popin", "gnomed"];

/// Styles that render on the layer leaves in Hyprland 0.56.2. Slide variants
/// other than `slide` are drawn as a plain slide, so they are not offered.
pub(super) const LAYER_ANIMATION_STYLES: [&str; 3] = ["slide", "popin", "fade"];

/// Keys of the `layers` and `switching` animation leaves that were removed. Hyprland
/// reads neither leaf directly (only their `In`/`Out` children), so their values
/// had no effect.
pub(super) const RETIRED_ANIMATION_KEYS: [&str; 6] = [
  "animation_layers_enabled",
  "animation_layers_speed",
  "animation_layers_curve",
  "animation_switching_style",
  "animation_switching_speed",
  "animation_switching_curve",
];

/// Styles Hyprland 0.56.2 accepts for the workspace leaves. `popin` and
/// `gnomed` are refused there by the compositor.
pub(super) const WORKSPACE_ANIMATION_STYLES: [&str; 5] =
  ["slide", "fade", "slidefade", "slidevert", "slidefadevert"];

/// Styles accepted by `effects.animation_<leaf>_style`, which depend on the leaf.
pub(super) fn animation_styles(key: &str) -> &'static [&'static str] {
  match key
    .trim_start_matches("animation_")
    .trim_end_matches("_style")
  {
    "opening" | "closing" => &WINDOW_ANIMATION_STYLES,
    "layers_in" | "layers_out" => &LAYER_ANIMATION_STYLES,
    _ => &WORKSPACE_ANIMATION_STYLES,
  }
}

/// Animation curves accepted by `effects.animation_<leaf>_curve`.
pub(super) const ANIMATION_CURVES: [&str; 5] =
  ["default", "bezier", "spring", "linear", "ease_out"];

pub(super) fn validate_effects(document: &ConfigDocument) -> ConfigResult<()> {
  let Some(effects) = document.sections.get("effects") else {
    return Ok(());
  };
  for (key, value) in effects.as_object().expect("validated section object") {
    // Checked before the boolean shortcut below, so a switch is accepted only
    // where the animation settings expect one.
    if key.starts_with("animation_") {
      validate_animation_setting(key, value)?;
      continue;
    }
    if value.is_boolean() {
      continue;
    }
    if let Some((_, minimum, maximum, integer)) =
      BLUR_SETTINGS.iter().find(|(name, ..)| name == key)
    {
      validate_blur_setting(key, value, *minimum, *maximum, *integer)?;
      continue;
    }
    let Some(number) = value.as_i64() else {
      return Err(ConfigError::Invalid(format!(
        "effects.{key} must be a boolean or an integer between 0 and 100"
      )));
    };
    if !(0..=100).contains(&number) {
      return Err(ConfigError::Invalid(format!(
        "effects.{key} must be between 0 and 100"
      )));
    }
  }
  Ok(())
}

/// Checks one `effects.animation_*` entry. The bounds match the table in
/// `argvus effects-toggle`; a value outside them is refused so the compositor never
/// receives a curve or speed it cannot parse.
pub(super) fn validate_animation_setting(key: &str, value: &Value) -> ConfigResult<()> {
  if key.ends_with("_enabled") {
    return if value.is_boolean() {
      Ok(())
    } else {
      Err(ConfigError::Invalid(format!(
        "effects.{key} must be a boolean"
      )))
    };
  }
  let allowed = if key.ends_with("_style") {
    Some(animation_styles(key))
  } else if key.ends_with("_curve") {
    Some(&ANIMATION_CURVES[..])
  } else {
    None
  };
  if let Some(allowed) = allowed {
    return match value.as_str() {
      Some(choice) if allowed.contains(&choice) => Ok(()),
      _ => Err(ConfigError::Invalid(format!(
        "effects.{key} must be one of {}",
        allowed.join(", ")
      ))),
    };
  }
  let (minimum, maximum, integer) = if key.ends_with("_speed") {
    (1.0, 10.0, false)
  } else if key.ends_with("_popin") {
    (1.0, 100.0, true)
  } else {
    match key {
      "animation_bezier_x1" | "animation_bezier_x2" => (0.0, 1.0, false),
      "animation_bezier_y1" | "animation_bezier_y2" => (-1.0, 2.0, false),
      "animation_spring_mass" => (0.5, 10.0, false),
      "animation_spring_stiffness" => (0.5, 2000.0, false),
      "animation_spring_dampening" => (0.5, 200.0, false),
      _ => {
        return Err(ConfigError::Invalid(format!(
          "effects.{key} is not an animation setting"
        )));
      }
    }
  };
  validate_blur_setting(key, value, minimum, maximum, integer)
}

pub(super) fn validate_blur_setting(
  key: &str,
  value: &Value,
  minimum: f64,
  maximum: f64,
  integer: bool,
) -> ConfigResult<()> {
  let number = value.as_f64().filter(|number| number.is_finite());
  let Some(number) = number else {
    return Err(ConfigError::Invalid(format!(
      "effects.{key} must be a number between {minimum} and {maximum}"
    )));
  };
  if integer && number.fract() != 0.0 {
    return Err(ConfigError::Invalid(format!(
      "effects.{key} must be an integer between {minimum} and {maximum}"
    )));
  }
  if !(minimum..=maximum).contains(&number) {
    return Err(ConfigError::Invalid(format!(
      "effects.{key} must be between {minimum} and {maximum}"
    )));
  }
  Ok(())
}

pub(super) fn validate_taskbar(document: &ConfigDocument) -> ConfigResult<()> {
  if let Some(format) = document.get("/taskbar/date/format").and_then(Value::as_str)
    && !matches!(
      format,
      "weekday_day_month"
        | "weekday_day_month_year"
        | "weekday_day_slash_month_year"
        | "numeric_short_locale_aware"
    )
  {
    return Err(ConfigError::Invalid(format!(
      "taskbar.date.format must be one of weekday_day_month, \
             weekday_day_month_year, weekday_day_slash_month_year, \
             numeric_short_locale_aware (got {format})"
    )));
  }
  if let Some(format) = document.get("/taskbar/time/format").and_then(Value::as_str)
    && !matches!(format, "24h" | "12h")
  {
    return Err(ConfigError::Invalid(format!(
      "taskbar.time.format must be 24h or 12h (got {format})"
    )));
  }
  validate_taskbar_shape(
    document.get(json_pointers::TASKBAR_SHAPE_HEIGHT),
    "height",
    &waybar::TASKBAR_HEIGHTS,
  )?;
  validate_taskbar_shape(
    document.get(json_pointers::TASKBAR_SHAPE_BORDER_WIDTH),
    "border_width",
    &waybar::TASKBAR_BORDER_WIDTHS,
  )
}

/// A taskbar shape value is `null` (follow the packaged taskbar and the theme)
/// or one of the pixel sizes the Control Center offers.
pub(super) fn validate_taskbar_shape(
  value: Option<&Value>,
  key: &str,
  offered: &[u32],
) -> ConfigResult<()> {
  let Some(value) = value.filter(|value| !value.is_null()) else {
    return Ok(());
  };
  let valid = value
    .as_u64()
    .and_then(|pixels| u32::try_from(pixels).ok())
    .is_some_and(|pixels| offered.contains(&pixels));
  if valid {
    return Ok(());
  }
  let offered = offered
    .iter()
    .map(u32::to_string)
    .collect::<Vec<_>>()
    .join(", ");
  Err(ConfigError::Invalid(format!(
    "taskbar.shape.{key} must be null or one of {offered} (got {value})"
  )))
}

pub(super) fn validate_fonts(document: &ConfigDocument) -> ConfigResult<()> {
  let Some(value) = document.sections.get("fonts") else {
    return Ok(());
  };
  if let Some(targets) = value.get("targets") {
    let targets = targets
      .as_object()
      .ok_or_else(|| ConfigError::Invalid("fonts.targets must be an object".into()))?;
    for (target, selection) in targets {
      let selection = selection
        .as_object()
        .ok_or_else(|| ConfigError::Invalid(format!("fonts.targets.{target} must be an object")))?;
      if selection
        .get("family")
        .is_some_and(|value| !value.is_string())
        || selection
          .get("style")
          .is_some_and(|value| !value.is_string())
      {
        return Err(ConfigError::Invalid(format!(
          "fonts.targets.{target}.family and style must be strings"
        )));
      }
      if let Some(size) = selection.get("size") {
        let size = size.as_u64().ok_or_else(|| {
          ConfigError::Invalid(format!("fonts.targets.{target}.size must be an integer"))
        })?;
        if !(8..=32).contains(&size) {
          return Err(ConfigError::Invalid(format!(
            "fonts.targets.{target}.size must be between 8 and 32"
          )));
        }
      }
    }
  }
  if let Some(rendering) = value.get("rendering") {
    let rendering = rendering
      .as_object()
      .ok_or_else(|| ConfigError::Invalid("fonts.rendering must be an object".into()))?;
    for key in ["antialias", "custom_dpi"] {
      if rendering.get(key).is_some_and(|value| !value.is_boolean()) {
        return Err(ConfigError::Invalid(format!(
          "fonts.rendering.{key} must be a boolean"
        )));
      }
    }
    for key in ["hinting", "subpixel"] {
      if rendering.get(key).is_some_and(|value| !value.is_string()) {
        return Err(ConfigError::Invalid(format!(
          "fonts.rendering.{key} must be a string"
        )));
      }
    }
    if let Some(dpi) = rendering.get("dpi") {
      let dpi = dpi
        .as_u64()
        .ok_or_else(|| ConfigError::Invalid("fonts.rendering.dpi must be an integer".into()))?;
      if !(72..=240).contains(&dpi) {
        return Err(ConfigError::Invalid(
          "fonts.rendering.dpi must be between 72 and 240".into(),
        ));
      }
    }
  }
  Ok(())
}

pub(super) fn validate_hyprland(document: &ConfigDocument) -> ConfigResult<()> {
  let Some(value) = document.sections.get("hyprland") else {
    return Ok(());
  };
  for key in ["keybindings", "input", "keyboard"] {
    if value.get(key).is_some_and(|value| !value.is_object()) {
      return Err(ConfigError::Invalid(format!(
        "hyprland.{key} must be an object"
      )));
    }
  }
  Ok(())
}

pub(super) fn validate_displays(document: &ConfigDocument) -> ConfigResult<()> {
  let Some(value) = document.sections.get("displays") else {
    return Ok(());
  };
  if value
    .get("primary_monitor")
    .is_some_and(|value| !value.is_string() && !value.is_null())
  {
    return Err(ConfigError::Invalid(
      "displays.primary_monitor must be a string or null".into(),
    ));
  }
  if value
    .get("monitors")
    .is_some_and(|value| !value.is_object())
    || value
      .get("workspaces")
      .is_some_and(|value| !value.is_object())
  {
    return Err(ConfigError::Invalid(
      "displays.monitors and workspaces must be objects".into(),
    ));
  }
  Ok(())
}

pub(super) fn validate_default_apps(document: &ConfigDocument) -> ConfigResult<()> {
  if let Some(value) = document.sections.get("default_apps") {
    let object = value
      .as_object()
      .ok_or_else(|| ConfigError::Invalid("default_apps must be an object".into()))?;
    const CATEGORIES: [&str; 11] = [
      "terminal",
      "file_manager",
      "text_editor",
      "terminal_editor",
      "browser",
      "image_viewer",
      "pdf_viewer",
      "video_player",
      "audio_player",
      "archive",
      "launcher",
    ];
    for (category, application) in object {
      if !CATEGORIES.contains(&category.as_str()) {
        return Err(ConfigError::Invalid(format!(
          "default_apps contains unknown category: {category}"
        )));
      }
      if !application.is_string() {
        return Err(ConfigError::Invalid(format!(
          "default_apps.{category} must be a string"
        )));
      }
    }
  }
  Ok(())
}

pub(super) fn validate_keyboard_shortcuts(document: &ConfigDocument) -> ConfigResult<()> {
  let Some(value) = document.sections.get("keyboard_shortcuts") else {
    return Ok(());
  };
  let object = value
    .as_object()
    .ok_or_else(|| ConfigError::Invalid("keyboard_shortcuts must be an object".into()))?;
  for (key, shortcut) in object {
    if !key.chars().all(|character| {
      character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_'
    }) {
      return Err(ConfigError::Invalid(format!(
        "keyboard_shortcuts.{key} is not a stable config key"
      )));
    }
    if !shortcut.is_null() && !shortcut.is_string() {
      return Err(ConfigError::Invalid(format!(
        "keyboard_shortcuts.{key} must be a string or null"
      )));
    }
  }
  Ok(())
}

pub(super) fn validate_power(document: &ConfigDocument) -> ConfigResult<()> {
  let Some(power) = document.sections.get("power") else {
    return Ok(());
  };
  for key in ["screen_off_minutes", "lock_minutes"] {
    if let Some(value) = power.get(key)
      && value.as_u64().is_none()
    {
      return Err(ConfigError::Invalid(format!(
        "power.{key} must be an integer"
      )));
    }
  }
  for key in ["keep_awake"] {
    if power.get(key).is_some_and(|value| !value.is_boolean()) {
      return Err(ConfigError::Invalid(format!("power.{key} must be boolean")));
    }
  }
  for key in ["lid_battery", "lid_ac", "power_button"] {
    if power.get(key).is_some_and(|value| !value.is_string()) {
      return Err(ConfigError::Invalid(format!(
        "power.{key} must be a string"
      )));
    }
  }
  Ok(())
}

pub(super) fn validate_audio(document: &ConfigDocument) -> ConfigResult<()> {
  let Some(audio) = document.sections.get("audio") else {
    return Ok(());
  };
  if let Some(value) = audio.get("output_volume")
    && value.as_u64().is_none_or(|volume| volume > 100)
  {
    return Err(ConfigError::Invalid(
      "audio.output_volume must be an integer between 0 and 100".into(),
    ));
  }
  if audio
    .get("output_muted")
    .is_some_and(|value| !value.is_boolean())
  {
    return Err(ConfigError::Invalid(
      "audio.output_muted must be boolean".into(),
    ));
  }
  Ok(())
}

pub(super) fn validate_session(document: &ConfigDocument) -> ConfigResult<()> {
  if let Some(value) = document.sections.get("session")
    && value
      .get("language")
      .is_some_and(|language| !language.is_string())
  {
    return Err(ConfigError::Invalid(
      "session.language must be a string".into(),
    ));
  }
  Ok(())
}

pub(super) fn validate_numeric_section(
  document: &ConfigDocument,
  section: &str,
  minimum: i64,
  maximum: i64,
) -> ConfigResult<()> {
  let Some(value) = document.sections.get(section) else {
    return Ok(());
  };
  for (key, value) in value.as_object().expect("validated section object") {
    if key.ends_with("_legacy") {
      continue;
    }
    if let Some(number) = value.as_i64()
      && !(minimum..=maximum).contains(&number)
    {
      return Err(ConfigError::Invalid(format!(
        "{section}.{key} must be between {minimum} and {maximum}"
      )));
    }
  }
  Ok(())
}

pub(super) fn validate_control_panel(document: &ConfigDocument) -> ConfigResult<()> {
  let Some(value) = document.sections.get("control_panel") else {
    return Ok(());
  };
  let Some(cards) = value.get("cards") else {
    return Ok(());
  };
  if !cards.is_array() {
    return Err(ConfigError::Invalid(
      "control_panel.cards must be an array".into(),
    ));
  }
  Ok(())
}
