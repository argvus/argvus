//! Saving, defaults, layout variants and theme application on the store.

use super::*;

#[test]
fn float_variant_resets_geometry_to_the_documented_float_values() {
  let mut document = ConfigDocument::default();

  reset_layout_geometry(&mut document, "float").unwrap();

  assert_eq!(
    document
      .get("/layout/window/gaps_in")
      .and_then(Value::as_i64),
    Some(10)
  );
  assert_eq!(
    document
      .get("/layout/window/gaps_out_top")
      .and_then(Value::as_i64),
    Some(18)
  );
}

#[test]
fn save_keeps_previous_valid_configuration_as_backup() {
  let directory = env::temp_dir().join(format!("argvus-config-test-{}-backup", std::process::id()));
  let _ = fs::remove_dir_all(&directory);
  let config_directory = directory.join("argvus");
  let store = ConfigStore {
    path: config_directory.join("config.json"),
    lock_path: config_directory.join(LOCK_FILE),
  };
  let mut first = ConfigDocument::default();
  first
    .set("/appearance/theme", Value::String("argvus-dark".into()))
    .unwrap();
  store.save(&first).unwrap();

  let mut second = first.clone();
  second
    .set("/appearance/theme", Value::String("argvus-light".into()))
    .unwrap();
  store.save(&second).unwrap();

  let backup = fs::read_to_string(config_directory.join("data/backups/config.json.bak")).unwrap();
  let backup_document: ConfigDocument = serde_json::from_str(&backup).unwrap();
  assert_eq!(
    backup_document
      .get("/appearance/theme")
      .and_then(Value::as_str),
    Some("argvus-dark")
  );
  assert_eq!(
    store
      .load()
      .unwrap()
      .get("/appearance/theme")
      .and_then(Value::as_str),
    Some("argvus-light")
  );
  fs::remove_dir_all(directory).unwrap();
}

#[test]
fn canonical_update_preserves_unrelated_sections() {
  let directory = env::temp_dir().join(format!(
    "argvus-config-test-{}-preserve",
    std::process::id()
  ));
  let _ = fs::remove_dir_all(&directory);
  let config_directory = directory.join("argvus");
  let store = ConfigStore {
    path: config_directory.join("config.json"),
    lock_path: config_directory.join(LOCK_FILE),
  };

  store
    .update("/power/lock_minutes", Value::Number(15.into()))
    .unwrap();
  store
    .update(
      "/appearance/theme",
      Value::String("argvus-dark-float".into()),
    )
    .unwrap();

  let document = store.load().unwrap();
  assert_eq!(
    document.get("/power/lock_minutes"),
    Some(&Value::Number(15.into()))
  );
  assert_eq!(
    document.get("/appearance/theme").and_then(Value::as_str),
    Some("argvus-dark-float")
  );
  fs::remove_dir_all(directory).unwrap();
}

#[test]
fn new_profiles_receive_effect_defaults_without_overwriting_values() {
  let empty = ConfigDocument::default();
  let path = env::temp_dir().join(format!(
    "argvus-config-defaults-{}.json",
    std::process::id()
  ));
  let _ = fs::remove_file(&path);
  let effective = ConfigDocument::load_effective(&path).unwrap();
  let effects = effective.sections.get("effects").unwrap();
  assert_eq!(effects.get("animations"), Some(&Value::Bool(true)));
  assert_eq!(
    effects.get("transparency_taskbar_value"),
    Some(&Value::Number(50.into()))
  );
  assert_eq!(
    effects.get("transparency_terminal_value"),
    Some(&Value::Number(50.into()))
  );
  assert_eq!(
    effects.get("transparency_launchers_value"),
    Some(&Value::Number(15.into()))
  );
  assert_eq!(
    effects.get("transparency_control-center_value"),
    Some(&Value::Number(15.into()))
  );
  assert_eq!(effects.get("blur_size").and_then(Value::as_f64), Some(6.0));
  assert_eq!(
    effects.get("blur_passes").and_then(Value::as_f64),
    Some(2.0)
  );
  assert_eq!(effects.get("blur_contrast"), Some(&Value::from(0.9)));
  assert!(effects.get("blur_global_value").is_none());
  assert_eq!(
    effects.get("blur_control-center_enabled"),
    Some(&Value::Bool(true))
  );
  for surface in ["taskbar", "control-panel", "terminal", "launchers"] {
    assert!(
      effects.get(format!("blur_{surface}_enabled")).is_none(),
      "obsolete per-surface blur switch must not be seeded"
    );
  }

  let mut existing = empty;
  existing
    .set(
      "/effects/transparency_control-center_value",
      Value::Number(80.into()),
    )
    .unwrap();
  let existing_path = path.with_extension("existing.json");
  fs::write(&existing_path, serde_json::to_vec(&existing).unwrap()).unwrap();
  let preserved = ConfigDocument::load_effective(&existing_path).unwrap();
  assert_eq!(
    preserved.get("/effects/transparency_control-center_value"),
    Some(&Value::Number(80.into()))
  );
  let _ = fs::remove_file(existing_path);
}

#[test]
fn ensure_materializes_defaults_and_preserves_explicit_false_and_zero() {
  let directory = env::temp_dir().join(format!("argvus-config-test-{}-ensure", std::process::id()));
  let _ = fs::remove_dir_all(&directory);
  let config_directory = directory.join("argvus");
  let store = ConfigStore {
    path: config_directory.join("config.json"),
    lock_path: config_directory.join(LOCK_FILE),
  };

  store.ensure().unwrap();
  assert!(store.module_directory().is_dir());
  let document = store.load().unwrap();
  assert_eq!(
    document.get("/appearance/theme").and_then(Value::as_str),
    Some("argvus-dark")
  );
  assert_eq!(
    document.get("/effects/animations"),
    Some(&Value::Bool(true))
  );

  store
    .set_explicit_for_test("/effects/animations", Value::Bool(false))
    .unwrap();
  store
    .set_explicit_for_test(
      "/effects/transparency_terminal_value",
      Value::Number(0.into()),
    )
    .unwrap();
  store.ensure().unwrap();
  let preserved = store.load().unwrap();
  assert_eq!(
    preserved.get("/effects/animations"),
    Some(&Value::Bool(false))
  );
  assert_eq!(
    preserved.get("/effects/transparency_terminal_value"),
    Some(&Value::Number(0.into()))
  );
  fs::remove_dir_all(directory).unwrap();
}

#[test]
fn ensure_does_not_replace_malformed_configuration() {
  let directory = env::temp_dir().join(format!(
    "argvus-config-test-{}-malformed",
    std::process::id()
  ));
  let _ = fs::remove_dir_all(&directory);
  let config_directory = directory.join("argvus");
  fs::create_dir_all(&config_directory).unwrap();
  let path = config_directory.join("config.json");
  fs::write(&path, b"{ invalid\n").unwrap();
  let store = ConfigStore {
    path: path.clone(),
    lock_path: config_directory.join(LOCK_FILE),
  };

  assert!(store.ensure().is_err());
  assert_eq!(fs::read_to_string(path).unwrap(), "{ invalid\n");
  fs::remove_dir_all(directory).unwrap();
}

#[test]
fn apply_theme_is_one_transaction_and_preserves_explicit_overrides() {
  let directory = env::temp_dir().join(format!("argvus-config-test-{}-theme", std::process::id()));
  let _ = fs::remove_dir_all(&directory);
  let config_directory = directory.join("argvus");
  let store = ConfigStore {
    path: config_directory.join("config.json"),
    lock_path: config_directory.join(LOCK_FILE),
  };
  store.ensure().unwrap();
  let mut patch = Map::new();
  patch.insert("/appearance/accent".into(), Value::String("#ABCDEF".into()));
  patch.insert("/appearance/accent_custom".into(), Value::Bool(true));
  patch.insert(
    "/appearance/wallpaper".into(),
    Value::String("/tmp/custom.jxl".into()),
  );
  patch.insert("/appearance/wallpaper_custom".into(), Value::Bool(true));
  store.patch(&patch).unwrap();
  store
    .apply_theme(
      "universe",
      Some("#EEEEEE"),
      Some("dark"),
      Some("/usr/share/backgrounds/universe.jxl"),
      false,
      false,
    )
    .unwrap();
  let document = store.load().unwrap();
  assert_eq!(
    document.get("/appearance/theme").and_then(Value::as_str),
    Some("universe")
  );
  assert_eq!(
    document.get("/layout/variant").and_then(Value::as_str),
    Some("sticky")
  );
  assert_eq!(
    document
      .get("/layout/window/gaps_in")
      .and_then(Value::as_i64),
    Some(2)
  );
  assert_eq!(
    document
      .get("/layout/window/rounded")
      .and_then(Value::as_bool),
    Some(false)
  );
  assert_eq!(
    document
      .get("/layout/taskbar/margin_bottom")
      .and_then(Value::as_i64),
    Some(0)
  );
  assert_eq!(
    document.get("/appearance/accent").and_then(Value::as_str),
    Some("#ABCDEF")
  );
  assert_eq!(
    document
      .get("/appearance/wallpaper")
      .and_then(Value::as_str),
    Some("/tmp/custom.jxl")
  );
  fs::remove_dir_all(directory).unwrap();
}

#[test]
fn apply_theme_resets_custom_accent_only_when_requested() {
  let directory = env::temp_dir().join(format!(
    "argvus-config-test-{}-accent-reset",
    std::process::id()
  ));
  let _ = fs::remove_dir_all(&directory);
  let config_directory = directory.join("argvus");
  let store = ConfigStore {
    path: config_directory.join("config.json"),
    lock_path: config_directory.join(LOCK_FILE),
  };
  store.ensure().unwrap();
  let mut patch = Map::new();
  patch.insert("/appearance/accent".into(), Value::String("#ABCDEF".into()));
  patch.insert("/appearance/accent_custom".into(), Value::Bool(true));
  store.patch(&patch).unwrap();

  // A projection that does not request the reset keeps the override.
  store
    .apply_theme(
      "universe",
      Some("#EEEEEE"),
      Some("dark"),
      None,
      false,
      false,
    )
    .unwrap();
  let document = store.load().unwrap();
  assert_eq!(
    document.get("/appearance/accent").and_then(Value::as_str),
    Some("#ABCDEF")
  );
  assert_eq!(
    document
      .get("/appearance/accent_custom")
      .and_then(Value::as_bool),
    Some(true)
  );

  // Selecting an official theme clears the override so the theme wins.
  store
    .apply_theme("dracula", Some("#AAAAAA"), Some("dark"), None, false, true)
    .unwrap();
  let document = store.load().unwrap();
  assert_eq!(
    document.get("/appearance/accent").and_then(Value::as_str),
    Some("#AAAAAA")
  );
  assert_eq!(
    document
      .get("/appearance/accent_custom")
      .and_then(Value::as_bool),
    Some(false)
  );
  fs::remove_dir_all(directory).unwrap();
}
