use super::component_adapters::{AdapterInputs, AdapterPlan, plan_adapters};
use super::file_writes::atomic_write_lua;
use super::hyprland_projection::{
  effective_rounding, lua_table, render_window_rules, splash_background_color,
  virtual_machine_state,
};
use super::manifest::manifest_affects_compositor;
use super::*;

fn stable_login_inputs<'a>(changed_sections: &'a [&'a str]) -> AdapterInputs<'a> {
  AdapterInputs {
    changed_sections,
    appearance_visual_changed: false,
    theme_transaction: false,
    theme_materialized: true,
    effects_materialized: true,
  }
}

#[test]
fn splash_background_follows_the_theme_and_falls_back_to_the_default() {
  assert_eq!(splash_background_color("argvus-dark"), "rgb(191b27)");
  assert_eq!(splash_background_color("tokyo-night"), "rgb(1a1b26)");
  assert_eq!(splash_background_color("my-custom-theme"), "rgb(101218)");
}

#[test]
fn manifest_without_hyprland_consumer_skips_the_compositor() {
  let manifest = serde_json::json!({"affected_consumers": ["control-panel", "audio"]});
  assert!(!manifest_affects_compositor(&manifest));
}

#[test]
fn manifest_with_hyprland_consumer_or_without_the_field_reloads() {
  let affected = serde_json::json!({"affected_consumers": ["hyprland"]});
  let unknown = serde_json::json!({"schema_version": 3});
  assert!(manifest_affects_compositor(&affected));
  assert!(manifest_affects_compositor(&unknown));
}

#[test]
fn stable_projection_runs_no_adapter() {
  let plan = plan_adapters(&stable_login_inputs(&[]));
  assert_eq!(
    plan,
    AdapterPlan {
      theme_switch: false,
      effects_toggle: false,
    }
  );
}

#[test]
fn missing_outputs_are_repaired_when_nothing_changed() {
  let inputs = AdapterInputs {
    theme_materialized: false,
    effects_materialized: false,
    ..stable_login_inputs(&[])
  };
  let plan = plan_adapters(&inputs);
  assert!(plan.theme_switch);
  assert!(plan.effects_toggle);
}

#[test]
fn visual_and_effects_changes_run_their_adapters() {
  let inputs = AdapterInputs {
    appearance_visual_changed: true,
    ..stable_login_inputs(&["appearance", "effects"])
  };
  let plan = plan_adapters(&inputs);
  assert!(plan.theme_switch);
  assert!(plan.effects_toggle);
}

#[test]
fn running_theme_switch_blocks_nested_adapters() {
  let inputs = AdapterInputs {
    appearance_visual_changed: true,
    theme_transaction: true,
    theme_materialized: false,
    ..stable_login_inputs(&["effects"])
  };
  let plan = plan_adapters(&inputs);
  assert!(!plan.theme_switch);
  assert!(!plan.effects_toggle);
}

#[test]
fn hypridle_projection_uses_the_runtime_consumer_path() {
  let root = env::temp_dir().join(format!("argvus-project-test-{}", std::process::id()));
  let _ = fs::remove_dir_all(&root);
  let paths = ArgvusPaths {
    data: root.join("data"),
    generated: root.join("data/generated"),
    internal: root.join("data/internal"),
    backups: root.join("data/backups"),
    projection_manifest: root.join("state/config-projection.json"),
    root: root.clone(),
  };
  let mut document = ConfigDocument::default();
  document
    .set("/power/lock_minutes", Value::from(15))
    .expect("valid power value");
  document
    .set("/power/screen_off_minutes", Value::from(10))
    .expect("valid power value");

  project_hypridle(&document, &paths).expect("hypridle projection succeeds");
  let runtime =
    fs::read_to_string(root.join("data/hypr/hypridle.conf")).expect("runtime hypridle file exists");
  assert!(runtime.contains("timeout = 900"));
  // Idle actions are runtime commands; they must not go back through hyprctl.
  assert!(!runtime.contains("hyprctl"));
  assert!(runtime.contains("argvus config hyprland dpms off"));
  assert!(runtime.contains("timeout = 600"));
  assert!(!root.join("hypr/hypridle.conf").exists());
  let _ = fs::remove_dir_all(root);
}

#[test]
fn reload_targets_keep_power_changes_scoped_to_hypridle() {
  let targets = reload_targets(&["power"], false, false, false);
  assert_eq!(targets.get("hypridle"), Some(&Value::Bool(true)));
  assert_eq!(targets.len(), 1);
}

#[test]
fn reload_targets_keep_wallpaper_changes_scoped_to_wallpaper() {
  let targets = reload_targets(&["appearance"], false, true, false);
  assert_eq!(targets.get("wallpaper"), Some(&Value::Bool(true)));
  assert_eq!(targets.len(), 1);
}

/// The targets that recreate a surface, and so show the splash.
fn restarts(targets: &serde_json::Map<String, Value>) -> Vec<&str> {
  [
    "taskbar",
    "widget_telemetry",
    "control_panel",
    "notifications",
  ]
  .into_iter()
  .filter(|target| targets.contains_key(*target))
  .collect()
}

#[test]
fn theme_and_effect_changes_reload_in_place() {
  let theme = reload_targets(&["appearance"], true, true, false);
  for target in ["hyprland", "consumers", "wallpaper", "polkit"] {
    assert_eq!(theme.get(target), Some(&Value::Bool(true)), "{target}");
  }
  assert!(restarts(&theme).is_empty());
  let effects = reload_targets(&["effects"], false, false, false);
  assert!(effects.contains_key("hyprland") && effects.contains_key("consumers"));
  assert!(restarts(&effects).is_empty());
  let fonts = reload_targets(&["fonts"], false, false, false);
  assert_eq!(fonts.keys().collect::<Vec<_>>(), ["consumers"]);
}

#[test]
fn only_the_taskbar_layout_recreates_the_bars() {
  let window = reload_targets(&["layout"], false, false, false);
  assert_eq!(window.keys().collect::<Vec<_>>(), ["hyprland"]);
  let taskbar = reload_targets(&["layout"], false, false, true);
  assert_eq!(restarts(&taskbar), ["taskbar", "widget_telemetry"]);
  let panel = reload_targets(&["control_panel"], false, false, false);
  assert_eq!(restarts(&panel), ["control_panel"]);
}

#[test]
fn wallpaper_projection_updates_the_native_hyprpaper_file() {
  let root = env::temp_dir().join(format!("argvus-wallpaper-test-{}", std::process::id()));
  let _ = fs::remove_dir_all(&root);
  let paths = ArgvusPaths {
    data: root.join("data"),
    generated: root.join("data/generated"),
    internal: root.join("data/internal"),
    backups: root.join("data/backups"),
    projection_manifest: root.join("state/config-projection.json"),
    root: root.clone(),
  };
  let mut document = ConfigDocument::default();
  document
    .set(
      "/appearance/wallpaper",
      Value::from("/usr/share/backgrounds/argvus/argvus-dark.jxl"),
    )
    .expect("valid wallpaper value");

  project_wallpaper(&document, &paths).expect("wallpaper projection succeeds");
  let native = fs::read_to_string(root.join("data/hypr/hyprpaper.conf"))
    .expect("native wallpaper file exists");
  assert!(native.contains("path = /usr/share/backgrounds/argvus/argvus-dark.jxl"));
  let _ = fs::remove_dir_all(root);
}

#[test]
fn effective_rounding_is_zero_when_rounded_is_false() {
  assert_eq!(effective_rounding(false, 0), 0);
  assert_eq!(effective_rounding(false, 4), 0);
  assert_eq!(effective_rounding(false, 8), 0);
  assert_eq!(effective_rounding(false, 10), 0);
}

#[test]
fn effective_rounding_clamps_to_valid_range_when_rounded_is_true() {
  // Below minimum (2)
  assert_eq!(effective_rounding(true, 0), 2);
  assert_eq!(effective_rounding(true, 1), 2);
  // Within range
  assert_eq!(effective_rounding(true, 2), 2);
  assert_eq!(effective_rounding(true, 5), 5);
  assert_eq!(effective_rounding(true, 10), 10);
  // Above maximum (10)
  assert_eq!(effective_rounding(true, 11), 10);
  assert_eq!(effective_rounding(true, 100), 10);
}

#[test]
fn layout_projection_ignores_rounding_when_rounded_is_false() {
  let root = env::temp_dir().join(format!("argvus-borders-test-{}", std::process::id()));
  let _ = fs::remove_dir_all(&root);
  let paths = ArgvusPaths {
    data: root.join("data"),
    generated: root.join("data/generated"),
    internal: root.join("data/internal"),
    backups: root.join("data/backups"),
    projection_manifest: root.join("state/config-projection.json"),
    root: root.clone(),
  };
  let mut document = ConfigDocument::default();
  document
    .set("/layout/window/rounded", Value::Bool(false))
    .expect("valid rounded value");
  document
    .set("/layout/window/rounding", Value::from(8))
    .expect("valid rounding value");
  document
    .set("/layout/window/border_size", Value::from(1))
    .expect("valid border_size value");

  project_layout(&document, &paths).expect("layout projection succeeds");
  let borders = fs::read_to_string(root.join("data/.borders")).expect(".borders file exists");
  assert!(borders.contains("rounded=0"), "rounded should be 0");
  assert!(
    borders.contains("rounding=0"),
    "rounding should be forced to 0 when rounded=false"
  );
  assert!(
    borders.contains("thickness=1"),
    "thickness should be preserved"
  );
  let _ = fs::remove_dir_all(root);
}

#[test]
fn layout_projection_respects_rounding_when_rounded_is_true() {
  let root = env::temp_dir().join(format!("argvus-borders-true-test-{}", std::process::id()));
  let _ = fs::remove_dir_all(&root);
  let paths = ArgvusPaths {
    data: root.join("data"),
    generated: root.join("data/generated"),
    internal: root.join("data/internal"),
    backups: root.join("data/backups"),
    projection_manifest: root.join("state/config-projection.json"),
    root: root.clone(),
  };
  let mut document = ConfigDocument::default();
  document
    .set("/layout/window/rounded", Value::Bool(true))
    .expect("valid rounded value");
  document
    .set("/layout/window/rounding", Value::from(5))
    .expect("valid rounding value");
  document
    .set("/layout/window/border_size", Value::from(1))
    .expect("valid border_size value");

  project_layout(&document, &paths).expect("layout projection succeeds");
  let borders = fs::read_to_string(root.join("data/.borders")).expect(".borders file exists");
  assert!(borders.contains("rounded=1"), "rounded should be 1");
  assert!(
    borders.contains("rounding=5"),
    "rounding should be preserved when rounded=true"
  );
  assert!(
    borders.contains("thickness=1"),
    "thickness should be preserved"
  );
  let _ = fs::remove_dir_all(root);
}

#[test]
fn renders_window_rules_and_rejects_unsafe_patterns() {
  let rules = serde_json::json!({
      "browser": { "workspace": 2, "classes": ["firefox.*", "zen-browser.*"] },
      "ide": { "workspace": 1, "classes": ["[Cc]ode.*"] }
  });
  let lua = render_window_rules(rules.as_object().unwrap()).unwrap();
  assert!(lua.contains("{ workspace = 2, classes = { \"firefox.*\", \"zen-browser.*\", } },"));
  assert!(lua.contains("{ workspace = 1, classes = { \"[Cc]ode.*\", } },"));

  for invalid in [
    serde_json::json!({ "x": { "workspace": 11, "classes": ["a"] } }),
    serde_json::json!({ "x": { "workspace": 1, "classes": ["a\"b"] } }),
    serde_json::json!({ "x": { "workspace": 1, "classes": "code" } }),
  ] {
    assert!(render_window_rules(invalid.as_object().unwrap()).is_err());
  }
}

#[test]
fn virtual_machine_state_treats_none_as_physical_hardware() {
  assert!(!virtual_machine_state(Some("none"), Some("kvm")));
  assert!(virtual_machine_state(Some("kvm"), None));
  assert!(!virtual_machine_state(None, Some("none")));
  assert!(!virtual_machine_state(None, Some("")));
  assert!(!virtual_machine_state(None, None));
  // An empty published value is not an answer, so detection decides.
  assert!(virtual_machine_state(Some(""), Some("kvm")));
}

#[test]
fn lua_table_escapes_strings_and_keeps_nested_types() {
  let value = serde_json::json!({
      "quote\"and\\slash": "line\nbreak",
      "blur_size": 6,
      "animations": true,
  });
  let lua = lua_table(&value);
  assert!(lua.contains("[\"quote\\\"and\\\\slash\"] = \"line\\nbreak\""));
  assert!(lua.contains("[\"blur_size\"] = 6"));
  assert!(lua.contains("[\"animations\"] = true"));
}

#[test]
fn hypr_state_projection_writes_values_the_lua_config_reads() {
  if !luac_available() {
    return;
  }
  let root = env::temp_dir().join(format!("argvus-state-test-{}", std::process::id()));
  let _ = fs::remove_dir_all(&root);
  let paths = ArgvusPaths {
    data: root.join("data"),
    generated: root.join("data/generated"),
    internal: root.join("data/internal"),
    backups: root.join("data/backups"),
    projection_manifest: root.join("state/config-projection.json"),
    root: root.clone(),
  };
  let mut document = ConfigDocument::default();
  document
    .set("/layout/variant", Value::from("float"))
    .expect("valid layout variant");
  document
    .set("/appearance/accent_custom", Value::Bool(true))
    .expect("valid accent flag");
  document
    .set("/effects/blur_size", Value::from(6))
    .expect("valid blur size");

  project_hypr_state(&document, &paths).expect("state projection succeeds");
  let state =
    fs::read_to_string(root.join("data/generated/hypr/state.lua")).expect("state file exists");
  assert!(state.contains("layout_variant = \"float\""));
  assert!(state.contains("accent_custom = true"));
  assert!(state.contains("[\"blur_size\"] = 6"));
  let _ = fs::remove_dir_all(root);
}

/// Lua checks need `luac`; the tests skip (with a note) when it is absent,
/// because the projection itself only warns in that case.
fn luac_available() -> bool {
  let available = Command::new("luac").arg("-v").output().is_ok();
  if !available {
    eprintln!("skipping Lua projection test: luac is not installed");
  }
  available
}

#[test]
fn lua_projection_keeps_previous_file_when_candidate_is_invalid() {
  if !luac_available() {
    return;
  }
  let root = env::temp_dir().join(format!("argvus-lua-invalid-test-{}", std::process::id()));
  let _ = fs::remove_dir_all(&root);
  let target = root.join("hypr/monitors.lua");
  atomic_write_lua(&target, "return { ok = true }\n").expect("valid Lua is written");
  let previous_contents = fs::read_to_string(&target).expect("previous file exists");

  let error =
    atomic_write_lua(&target, "return { broken = \n").expect_err("invalid Lua must be rejected");
  assert!(matches!(error, ConfigError::Invalid(_)));
  assert_eq!(
    fs::read_to_string(&target).expect("previous file survives"),
    previous_contents
  );
  // The rejected candidate must not be left next to the live file.
  let remaining_entries = fs::read_dir(root.join("hypr"))
    .expect("generated directory exists")
    .count();
  assert_eq!(remaining_entries, 1);
  let _ = fs::remove_dir_all(root);
}

#[test]
fn lua_projection_replaces_file_when_candidate_is_valid() {
  if !luac_available() {
    return;
  }
  let root = env::temp_dir().join(format!("argvus-lua-valid-test-{}", std::process::id()));
  let _ = fs::remove_dir_all(&root);
  let target = root.join("hypr/keybindings.lua");
  atomic_write_lua(&target, "return { ok = true }\n").expect("first Lua is written");
  atomic_write_lua(&target, "return { ok = false }\n").expect("second Lua is written");
  assert_eq!(
    fs::read_to_string(&target).expect("generated file exists"),
    "return { ok = false }\n"
  );
  let _ = fs::remove_dir_all(root);
}
