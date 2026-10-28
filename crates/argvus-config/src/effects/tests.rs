use super::commands::{all_effects_status, master_switch, toggle_target};
use super::css_text::{
  apply_border_transparency, delete_range, imported_build_theme, insert_after, launcher_background,
  opacity_factor, set_define, solid_over,
};
use super::effect_settings::is_leading_zero;
use super::*;

#[test]
fn launcher_background_takes_the_build_colour_with_the_given_opacity() {
  let build_theme = "* {\n    th-bg:            rgba(247, 247, 247, 96%);\n}\n";
  assert_eq!(
    launcher_background(build_theme, 0.5),
    "rgba(247, 247, 247, 50%)"
  );
  // Transparency disabled: the same colour, opaque.
  assert_eq!(
    launcher_background(build_theme, 1.0),
    "rgba(247, 247, 247, 100%)"
  );
}

#[test]
fn launcher_background_reads_an_rgb_build_colour() {
  let build_theme = "    th-bg:            rgb(40, 42, 54);\n";
  assert_eq!(
    launcher_background(build_theme, 0.88),
    "rgba(40, 42, 54, 88%)"
  );
}

#[test]
fn launcher_background_keeps_the_build_value_without_a_readable_colour() {
  assert_eq!(launcher_background("", 0.5), "@th-bg");
  assert_eq!(
    launcher_background("    th-bg:            @other;\n", 0.5),
    "@th-bg"
  );
}

#[test]
fn imported_build_theme_resolves_absolute_and_relative_imports() {
  let theme = Path::new("/home/user/.config/argvus/data/rofi/theme.rasi");
  let absolute = "@import \"/build/current/launcher/theme.rasi\"\n";
  assert_eq!(
    imported_build_theme(theme, absolute),
    Some(PathBuf::from("/build/current/launcher/theme.rasi"))
  );
  let relative = "@import \"../generated/theme/current/launcher/theme.rasi\"\n";
  assert_eq!(
    imported_build_theme(theme, relative),
    Some(PathBuf::from(
      "/home/user/.config/argvus/data/rofi/../generated/theme/current/launcher/theme.rasi"
    ))
  );
  assert_eq!(imported_build_theme(theme, "* {}\n"), None);
}

#[test]
fn each_settings_table_names_its_master_switch() {
  assert_eq!(master_switch("blur_"), Some("/effects/blur_global_enabled"));
  assert_eq!(master_switch("animation_"), Some("/effects/animations"));
  assert_eq!(master_switch("shadow_"), Some("/effects/shadow_enabled"));
}

#[test]
fn booleans_and_enums_accept_only_their_choices() {
  assert!(settings_valid("boolean", "-", "-", "true"));
  assert!(!settings_valid("boolean", "-", "-", "yes"));
  assert!(settings_valid("enum:slide,popin", "-", "-", "popin"));
  assert!(!settings_valid("enum:slide,popin", "-", "-", "pop"));
  assert!(!settings_valid("enum:slide,popin", "-", "-", ""));
}

#[test]
fn shadow_color_is_empty_for_the_accent_or_rrggbbaa() {
  assert!(settings_valid("color", "-", "-", ""));
  assert!(settings_valid("color", "-", "-", "3590bd99"));
  assert!(!settings_valid("color", "-", "-", "3590bd"));
  assert!(!settings_valid("color", "-", "-", "3590bdzz"));
}

#[test]
fn numbers_keep_their_shape_and_bounds() {
  assert!(settings_valid("integer", "1", "64", "10"));
  assert!(!settings_valid("integer", "1", "64", "99"));
  assert!(!settings_valid("integer", "1", "64", "4.5"));
  assert!(settings_valid("number", "0", "2", "1.25"));
  assert!(!settings_valid("number", "0", "2", "1.2.3"));
  assert!(!settings_valid("number", "0", "2", "."));
  assert!(settings_valid("number", "-2", "3", "-1.5"));
}

#[test]
fn the_leading_zero_test_refuses_a_lone_zero_as_the_shell_did() {
  assert!(is_leading_zero("0"));
  assert!(is_leading_zero("007"));
  assert!(is_leading_zero("-0"));
  assert!(!is_leading_zero("0.5"));
  assert!(!settings_valid("number", "0", "1", "0"));
  assert!(settings_valid("number", "0", "1", "0.5"));
}

#[test]
fn surface_lines_are_replaced_or_inserted_after_their_anchor() {
  let text = "@define-color transparent transparent;\nbody\n";
  assert_eq!(
    insert_after(
      text,
      "@define-color transparent transparent;",
      "@define-color x y;"
    ),
    "@define-color transparent transparent;\n@define-color x y;\nbody\n"
  );
  assert_eq!(
    set_define("@define-color th-power old;\n", "th-power", "new;"),
    "@define-color th-power new;\n"
  );
}

#[test]
fn animation_block_is_removed_between_its_markers() {
  let text = "a\n/* ARGVUS animations disabled */\nx\n/* ARGVUS animations disabled end */\nb\n";
  assert_eq!(
    delete_range(
      text,
      "/* ARGVUS animations disabled */",
      "/* ARGVUS animations disabled end */"
    ),
    "a\nb\n"
  );
}

#[test]
fn aggregate_needs_animations_and_effects() {
  assert_eq!(all_effects_status("enabled", "enabled"), "enabled");
  assert_eq!(all_effects_status("disabled", "enabled"), "disabled");
  assert_eq!(all_effects_status("enabled", "disabled"), "disabled");
}

#[test]
fn effects_switch_is_on_when_blur_or_any_transparency_is_on() {
  assert_eq!(effects_status("enabled", "enabled"), "enabled");
  assert_eq!(effects_status("enabled", "disabled"), "enabled");
  assert_eq!(effects_status("disabled", "enabled"), "enabled");
  assert_eq!(effects_status("disabled", "disabled"), "disabled");
}

#[test]
fn translucent_border_becomes_the_opaque_color_it_shows_over_the_background() {
  assert_eq!(
    solid_over("rgba(255, 255, 255, 0.5)", "#000000").as_deref(),
    Some("#808080")
  );
  assert_eq!(solid_over("#336699", "#000000").as_deref(), Some("#336699"));
  assert_eq!(solid_over("transparent", "#000000"), None);
}

#[test]
fn border_transparency_is_idempotent_and_reversible() {
  let directory = std::env::temp_dir().join(format!("argvus-border-{}", std::process::id()));
  fs::create_dir_all(&directory).unwrap();
  fs::write(
    directory.join("theme.css"),
    "@define-color th-background #000000;\n@define-color th-border rgba(255, 255, 255, 0.5);\n",
  )
  .unwrap();
  let css = directory.join("bar.css");
  let original = "@import url(\"./theme.css\");\n@define-color th-window-effective @th-background;\n* { border: 1px solid @th-border; }\n";
  let anchor = "@define-color th-window-effective ";
  let mut text = original.to_string();
  apply_border_transparency(
    &css,
    &mut text,
    &["th-border"],
    "th-background",
    anchor,
    false,
  );
  assert!(text.contains("@define-color th-border-effective #808080;"));
  assert!(text.contains("solid @th-border-effective;"));
  let once = text.clone();
  apply_border_transparency(
    &css,
    &mut text,
    &["th-border"],
    "th-background",
    anchor,
    false,
  );
  assert_eq!(text, once);
  apply_border_transparency(
    &css,
    &mut text,
    &["th-border"],
    "th-background",
    anchor,
    true,
  );
  assert!(text.contains("@define-color th-border-effective @th-border;"));
  let _ = fs::remove_dir_all(&directory);
}

#[test]
fn completing_the_projection_keeps_values_and_rewrites_only_on_change() {
  let root = std::env::temp_dir().join(format!("argvus-effects-{}", std::process::id()));
  let effects = Effects {
    paths: SessionPaths {
      config_home: root.join("config"),
      data_home: root.join("data"),
      system_config: root.join("system"),
    },
    store: None,
    commands_dir: root.join("commands"),
    state_home: root.join("state"),
  };
  let file = effects.theme_effects_file();
  fs::create_dir_all(file.parent().unwrap()).unwrap();
  fs::write(&file, "taskbar.transparency=80\n").unwrap();

  effects.ensure_theme_effects().unwrap();
  let completed = fs::read_to_string(&file).unwrap();
  assert!(completed.starts_with("taskbar.transparency=80\n"));
  assert!(completed.contains("widget-telemetry.blur.enabled=enabled\n"));
  assert!(!completed.contains("taskbar.transparency=50"));

  // A complete projection is left untouched: no write, no temporary file.
  let modified = fs::metadata(&file).unwrap().modified().unwrap();
  effects.ensure_theme_effects().unwrap();
  assert_eq!(fs::read_to_string(&file).unwrap(), completed);
  assert_eq!(fs::metadata(&file).unwrap().modified().unwrap(), modified);
  let leftovers = fs::read_dir(file.parent().unwrap())
    .unwrap()
    .filter_map(Result::ok)
    .filter(|entry| entry.file_name().to_string_lossy().contains(".tmp."))
    .count();
  assert_eq!(leftovers, 0);
  let _ = fs::remove_dir_all(&root);
}

#[test]
fn toggle_moves_every_effect_away_from_the_aggregate() {
  assert_eq!(toggle_target("enabled"), "disabled");
  // A mixed state reports `disabled`, so the toggle turns everything on.
  assert_eq!(toggle_target("disabled"), "enabled");
}

#[test]
fn opacity_follows_the_inverse_of_the_transparency() {
  assert_eq!(opacity_factor(20), "0.80");
  assert_eq!(opacity_factor(0), "1.00");
  assert_eq!(opacity_factor(100), "0.00");
}
