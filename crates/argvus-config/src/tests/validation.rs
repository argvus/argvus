//! Validation of animation, taskbar, font and accent values, and pointer lookups.

use super::*;

#[test]
fn animation_settings_accept_hyprland_values_and_refuse_the_rest() {
  assert!(validate_animation_setting("animation_opening_style", &Value::from("gnomed")).is_ok());
  assert!(validate_animation_setting("animation_opening_style", &Value::from("wobble")).is_err());
  assert!(validate_animation_setting("animation_opening_style", &Value::from("fade")).is_err());
  assert!(
    validate_animation_setting("animation_opening_style", &Value::from("slidefade")).is_err()
  );
  assert!(
    validate_animation_setting("animation_workspace_in_style", &Value::from("popin")).is_err()
  );
  assert!(validate_animation_setting("animation_workspace_in_style", &Value::from("fade")).is_ok());
  assert!(validate_animation_setting("animation_layers_in_style", &Value::from("gnomed")).is_err());
  assert!(
    validate_animation_setting("animation_layers_in_style", &Value::from("slidefade")).is_err()
  );
  assert!(validate_animation_setting("animation_layers_out_style", &Value::from("popin")).is_ok());
  assert!(validate_animation_setting("animation_fade_curve", &Value::from("ease_out")).is_ok());
  assert!(validate_animation_setting("animation_fade_curve", &Value::from("ease-out")).is_err());
  assert!(validate_animation_setting("animation_opening_speed", &Value::from(5)).is_ok());
  assert!(validate_animation_setting("animation_opening_speed", &Value::from(11)).is_err());
  assert!(validate_animation_setting("animation_opening_speed", &Value::from(4.79)).is_ok());
  assert!(validate_animation_setting("animation_opening_popin", &Value::from(85)).is_ok());
  assert!(validate_animation_setting("animation_moving_enabled", &Value::Bool(false)).is_ok());
  assert!(validate_animation_setting("animation_moving_enabled", &Value::from(1)).is_err());
  assert!(validate_animation_setting("animation_bezier_y1", &Value::from(-1.0)).is_ok());
  assert!(validate_animation_setting("animation_bezier_y1", &Value::from(-1.5)).is_err());
  assert!(validate_animation_setting("animation_bezier_y2", &Value::from(2.5)).is_err());
  assert!(validate_animation_setting("animation_bezier_x1", &Value::from(-0.1)).is_err());
  assert!(validate_animation_setting("animation_spring_mass", &Value::from(0.2)).is_err());
  assert!(validate_animation_setting("animation_spring_stiffness", &Value::from(238.119)).is_ok());
  assert!(validate_animation_setting("animation_unknown", &Value::from(1)).is_err());
}

#[test]
fn stored_animation_values_are_migrated_before_validation() {
  let mut document = ConfigDocument::default();
  document.sections.insert(
    "effects".into(),
    serde_json::json!({
      "animation_layers_speed": 3.81,
      "animation_switching_style": "fade",
      "animation_opening_style": "slidefade",
      "animation_layers_in_style": "fade",
      "animation_bezier_y1": 2.5,
      "animation_bezier_y2": 0.5,
    }),
  );
  document.migrate_animation_settings();
  let effects = document.sections["effects"].as_object().unwrap();
  assert!(!effects.contains_key("animation_layers_speed"));
  assert!(!effects.contains_key("animation_switching_style"));
  assert!(!effects.contains_key("animation_opening_style"));
  assert_eq!(effects["animation_layers_in_style"], "fade");
  assert_eq!(effects["animation_bezier_y1"], 2.0);
  assert_eq!(effects["animation_bezier_y2"], 0.5);
  assert!(document.validate().is_ok());
}

#[test]
fn defaults_report_nested_values_that_are_not_objects() {
  for (section, value, path) in [
    (
      "layout",
      serde_json::json!({ "taskbar": 5 }),
      "layout.taskbar",
    ),
    (
      "taskbar",
      serde_json::json!({ "icons": "x" }),
      "taskbar.icons",
    ),
    (
      "taskbar",
      serde_json::json!({ "icons": { "utilities": [] } }),
      "taskbar.icons.utilities",
    ),
    (
      "hyprland",
      serde_json::json!({ "input": { "mouse": 1 } }),
      "hyprland.input.mouse",
    ),
  ] {
    let mut document = ConfigDocument::default();
    document.sections.insert(section.to_owned(), value);
    let error = apply_defaults(&mut document).unwrap_err();
    assert!(error.to_string().contains(path), "{path}: {error}");
  }
  let mut document = ConfigDocument::default();
  apply_defaults(&mut document).unwrap();
  assert_eq!(
    document.get("/taskbar/icons/focus_enabled"),
    Some(&Value::Bool(true))
  );
  let mut disabled = ConfigDocument::default();
  disabled
    .set("/taskbar/icons/focus_enabled", Value::Bool(false))
    .unwrap();
  apply_defaults(&mut disabled).unwrap();
  assert_eq!(
    disabled.get("/taskbar/icons/focus_enabled"),
    Some(&Value::Bool(false))
  );
  assert!(
    document
      .get("/layout/taskbar")
      .is_some_and(Value::is_object)
  );
  assert_eq!(
    document.get("/layout/taskbar/margin_bottom"),
    Some(&Value::from(2))
  );

  let mut float = ConfigDocument::default();
  float
    .set("/layout/variant", Value::String("float".into()))
    .unwrap();
  apply_defaults(&mut float).unwrap();
  assert_eq!(
    float.get("/layout/taskbar/margin_bottom"),
    Some(&Value::from(18))
  );

  let mut customized = ConfigDocument::default();
  customized
    .set("/layout/taskbar/margin_bottom", Value::from(0))
    .unwrap();
  apply_defaults(&mut customized).unwrap();
  assert_eq!(
    customized.get("/layout/taskbar/margin_bottom"),
    Some(&Value::from(0)),
    "defaults must preserve explicit user margins"
  );
}

#[test]
fn taskbar_shape_defaults_to_null_and_accepts_only_offered_sizes() {
  let mut document = ConfigDocument::default();
  apply_defaults(&mut document).unwrap();
  assert_eq!(document.get("/taskbar/shape/height"), Some(&Value::Null));
  assert_eq!(
    document.get("/taskbar/shape/border_width"),
    Some(&Value::Null)
  );
  document.validate().unwrap();

  document
    .set("/taskbar/shape/height", Value::from(36))
    .unwrap();
  document
    .set("/taskbar/shape/border_width", Value::from(0))
    .unwrap();
  document.validate().unwrap();

  for (pointer, value) in [
    ("/taskbar/shape/height", Value::from(30)),
    ("/taskbar/shape/border_width", Value::from(4)),
    ("/taskbar/shape/height", Value::from("36")),
  ] {
    let mut invalid = document.clone();
    let error = invalid
      .set(pointer, value)
      .and_then(|()| invalid.validate())
      .unwrap_err();
    assert!(
      error.to_string().contains("taskbar.shape"),
      "{pointer}: {error}"
    );
  }
}

#[test]
fn get_resolves_array_index_pointers() {
  let mut document = ConfigDocument::default();
  document.sections.insert(
    "projects".to_owned(),
    serde_json::json!({ "roots": ["~/Projects", "~/Work"] }),
  );
  assert_eq!(
    document.get("/projects/roots/0"),
    Some(&Value::String("~/Projects".to_owned()))
  );
  assert_eq!(
    document.get("/projects/roots/1"),
    Some(&Value::String("~/Work".to_owned()))
  );
  assert_eq!(document.get("/projects/roots/2"), None);
  assert_eq!(document.get("/projects/roots/x"), None);
}

#[test]
fn updates_nested_values_and_validates_accent() {
  let mut document = ConfigDocument::default();
  document
    .set("/appearance/accent", Value::String("#AABBCC".into()))
    .unwrap();
  assert_eq!(
    document.get("/appearance/accent").and_then(Value::as_str),
    Some("#AABBCC")
  );
  assert!(
    document
      .set("/appearance/accent", Value::String("bad".into()))
      .is_err()
  );
}

#[test]
fn scope_contains_only_requested_sections() {
  let mut document = ConfigDocument::default();
  document
    .set("/appearance/theme", Value::String("argvus-dark".into()))
    .unwrap();
  document
    .set("/default_apps/browser", Value::String("firefox".into()))
    .unwrap();
  let scoped = document.scoped(ConfigScope::Appearance);
  assert!(scoped.sections.contains_key("appearance"));
  assert!(!scoped.sections.contains_key("defaults"));
}

#[test]
fn validates_structured_font_ranges() {
  let mut document = ConfigDocument::default();
  document
    .set(
      "/fonts/targets/apps",
      serde_json::json!({
          "family": "IBM Plex Mono",
          "style": "Regular",
          "size": 14
      }),
    )
    .unwrap();
  assert!(
    document
      .set("/fonts/rendering/dpi", Value::Number(300.into()))
      .is_err()
  );
}

#[test]
fn language_is_no_longer_a_writable_config_setting() {
  let mut document = ConfigDocument::default();
  for (pointer, value) in [
    ("/session/language", serde_json::json!("pt-BR")),
    ("/session", serde_json::json!({"language": "pt-BR"})),
  ] {
    let error = document.set(pointer, value).unwrap_err().to_string();
    assert!(error.contains("locale.conf"));
    assert!(document.get("/session/language").is_none());
  }
  document
    .sections
    .insert("session".into(), serde_json::json!({"language": "pt-BR"}));
  assert!(document.validate().is_ok(), "old documents remain loadable");
  document
    .set("/appearance/theme", serde_json::json!("argvus-dark"))
    .unwrap();
}
