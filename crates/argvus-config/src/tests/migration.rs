//! Migration of legacy configuration and component directories.

use super::*;

#[test]
fn legacy_migration_does_not_replace_canonical_values() {
  let directory = env::temp_dir().join(format!(
    "argvus-config-test-{}-migration",
    std::process::id()
  ));
  let _ = fs::remove_dir_all(&directory);
  let config_directory = directory.join("argvus");
  fs::create_dir_all(&config_directory).unwrap();
  fs::write(config_directory.join(".active-theme"), "argvus-light\n").unwrap();
  fs::write(config_directory.join(".accent-color"), "#112233\n").unwrap();
  let store = ConfigStore {
    path: config_directory.join("config.json"),
    lock_path: config_directory.join(LOCK_FILE),
  };

  store
    .update("/appearance/theme", Value::String("argvus-dark".into()))
    .unwrap();
  store.migrate_legacy().unwrap();

  let document = store.load().unwrap();
  assert_eq!(
    document.get("/appearance/theme").and_then(Value::as_str),
    Some("argvus-dark")
  );
  assert_eq!(
    document.get("/appearance/accent").and_then(Value::as_str),
    Some("#112233")
  );
  assert!(document.get("/appearance/accent_custom").is_none());
  fs::remove_dir_all(directory).unwrap();
}

#[test]
fn bootstrap_migration_marks_only_legacy_explicit_accents() {
  let directory = env::temp_dir().join(format!(
    "argvus-config-test-{}-accent-marker",
    std::process::id()
  ));
  let _ = fs::remove_dir_all(&directory);
  let config_directory = directory.join("argvus");
  fs::create_dir_all(&config_directory).unwrap();
  fs::write(config_directory.join(".accent-color"), "#ABCDEF\n").unwrap();
  fs::write(config_directory.join(".accent-custom"), "1\n").unwrap();
  let store = ConfigStore {
    path: config_directory.join("config.json"),
    lock_path: config_directory.join(LOCK_FILE),
  };

  store.ensure().unwrap();
  let document = store.load().unwrap();
  assert_eq!(
    document.get("/appearance/accent").and_then(Value::as_str),
    Some("#ABCDEF")
  );
  assert_eq!(
    document.get("/appearance/accent_custom"),
    Some(&Value::Bool(true))
  );
  assert_eq!(
    document.get("/appearance/theme").and_then(Value::as_str),
    Some("argvus-dark")
  );
  fs::remove_dir_all(directory).unwrap();
}

#[test]
fn ensure_migrates_component_directories_into_data_idempotently() {
  let directory = env::temp_dir().join(format!(
    "argvus-config-test-{}-component-migration",
    std::process::id()
  ));
  let _ = fs::remove_dir_all(&directory);
  let config_directory = directory.join("argvus");
  fs::create_dir_all(config_directory.join("generated")).unwrap();
  fs::create_dir_all(config_directory.join("control-center")).unwrap();
  fs::create_dir_all(config_directory.join("data/generated")).unwrap();
  fs::create_dir_all(config_directory.join("data/control-center")).unwrap();
  fs::write(
    config_directory.join("generated/theme-effective.conf"),
    b"legacy-theme",
  )
  .unwrap();
  fs::write(
    config_directory.join("control-center/legacy.ini"),
    b"legacy-settings",
  )
  .unwrap();
  fs::write(
    config_directory.join("data/generated/current.conf"),
    b"current-generated",
  )
  .unwrap();
  fs::write(
    config_directory.join("data/control-center/current.ini"),
    b"current-control-center",
  )
  .unwrap();

  let store = ConfigStore {
    path: config_directory.join("config.json"),
    lock_path: config_directory.join(LOCK_FILE),
  };
  store.ensure().unwrap();

  assert!(!config_directory.join("generated").exists());
  assert!(!config_directory.join("control-center").exists());
  assert_eq!(
    fs::read(config_directory.join("data/generated/theme-effective.conf")).unwrap(),
    b"legacy-theme"
  );
  assert_eq!(
    fs::read(config_directory.join("data/control-center/legacy.ini")).unwrap(),
    b"legacy-settings"
  );

  let first_tree = fs::read_dir(config_directory.join("data"))
    .unwrap()
    .map(|entry| entry.unwrap().file_name())
    .collect::<Vec<_>>();
  store.ensure().unwrap();
  let second_tree = fs::read_dir(config_directory.join("data"))
    .unwrap()
    .map(|entry| entry.unwrap().file_name())
    .collect::<Vec<_>>();
  assert_eq!(first_tree, second_tree);
  fs::remove_dir_all(directory).unwrap();
}

#[test]
fn ensure_leaves_only_config_and_data_at_the_configuration_root() {
  let directory = env::temp_dir().join(format!(
    "argvus-config-test-{}-root-layout",
    std::process::id()
  ));
  let _ = fs::remove_dir_all(&directory);
  let config_directory = directory.join("argvus");
  fs::create_dir_all(&config_directory).unwrap();
  for name in [
    "hypr",
    "waybar",
    "rofi",
    "dunst",
    "quickshell",
    "gtk-3.0",
    "gtk-4.0",
    "qt6ct",
    "yazi",
    "superfile",
    "state",
    "taskbar",
    "display",
    "control-panel",
  ] {
    fs::create_dir_all(config_directory.join(name)).unwrap();
    fs::write(config_directory.join(name).join("marker"), b"managed").unwrap();
  }
  for (name, contents) in [
    (".active-theme", "argvus-dark\n"),
    (".accent-color", "#112233\n"),
    (".gtk-mode", "dark\n"),
    (".spaces", "gaps_in=2\n"),
    (".borders", "rounded=0\n"),
    (".monitors", "DP-1,auto,1,auto\n"),
    (".keep-awake", "enabled\n"),
    (".weather-location", "Sao Paulo\n"),
  ] {
    fs::write(config_directory.join(name), contents).unwrap();
  }

  let store = ConfigStore {
    path: config_directory.join("config.json"),
    lock_path: config_directory.join(LOCK_FILE),
  };
  store.ensure().unwrap();

  // Everything the resolver owns must land in its semantic destination.
  for name in [
    "terminal",
    "kitty",
    "generated",
    "control-center",
    "waybar",
    "state",
  ] {
    fs::create_dir_all(config_directory.join(name)).unwrap();
  }
  fs::write(config_directory.join("terminal/marker"), b"terminal").unwrap();
  fs::write(config_directory.join("kitty/marker"), b"kitty").unwrap();
  fs::write(config_directory.join("generated/marker"), b"generated").unwrap();
  fs::write(config_directory.join("control-center/marker"), b"cc").unwrap();
  fs::write(config_directory.join("defaults.json"), b"{}\n").unwrap();
  fs::write(config_directory.join("keybindings.toml"), b"k\n").unwrap();
  fs::write(config_directory.join("input.toml"), b"i\n").unwrap();
  fs::write(config_directory.join("fonts.conf"), b"f\n").unwrap();
  fs::write(config_directory.join(".idle-timeout"), b"t\n").unwrap();
  store.ensure().unwrap();

  let mut remaining = fs::read_dir(&config_directory)
    .unwrap()
    .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
    .collect::<Vec<_>>();
  remaining.sort();
  assert_eq!(remaining, vec!["config", "data"]);

  // Directories and marker files keep their names one level deeper.
  assert_eq!(
    fs::read(config_directory.join("data/waybar/marker")).unwrap(),
    b"managed"
  );
  assert_eq!(
    fs::read(config_directory.join("data/.active-theme")).unwrap(),
    b"argvus-dark\n"
  );
  assert_eq!(
    fs::read(config_directory.join("data/power/.keep-awake")).unwrap(),
    b"enabled\n"
  );

  // Semantic exceptions keep their dedicated destination.
  for relative in [
    // The legacy `terminal/` component becomes `data/terminal/`, and a
    // standalone root `kitty/` sits beside the other terminals.
    "data/terminal/marker",
    "data/terminal/kitty/marker",
    "data/generated/marker",
    "data/generated/fonts.conf",
    "data/control-center/marker",
    "data/control-center/defaults.json",
    "data/hypr/keybindings.toml",
    "data/hypr/input.toml",
    "data/power/.idle-timeout",
  ] {
    assert!(
      config_directory.join(relative).exists(),
      "expected {relative} after migration"
    );
  }

  let _ = fs::remove_dir_all(directory);
}

#[test]
fn migration_keeps_terminal_components_nested_and_unstranded() {
  let directory = env::temp_dir().join(format!(
    "argvus-config-test-{}-terminal-layout",
    std::process::id()
  ));
  let _ = fs::remove_dir_all(&directory);
  let config_directory = directory.join("argvus");

  // The legacy root `terminal/` already nests one directory per terminal.
  fs::create_dir_all(config_directory.join("terminal/kitty")).unwrap();
  fs::create_dir_all(config_directory.join("terminal/kitty-tui")).unwrap();
  fs::write(
    config_directory.join("terminal/kitty/kitty.conf"),
    b"nested",
  )
  .unwrap();
  // A standalone root `kitty/` must land beside the others, not be
  // stranded in `data/legacy` because `data/terminal/` already exists.
  fs::create_dir_all(config_directory.join("kitty")).unwrap();
  fs::write(config_directory.join("kitty/custom.conf"), b"standalone").unwrap();

  let store = ConfigStore {
    path: config_directory.join("config.json"),
    lock_path: config_directory.join(LOCK_FILE),
  };
  store.ensure().unwrap();

  assert_eq!(
    fs::read(config_directory.join("data/terminal/kitty/kitty.conf")).unwrap(),
    b"nested"
  );
  assert!(config_directory.join("data/terminal/kitty-tui").is_dir());
  // The standalone directory merges into the terminal tree, and no
  // component is diverted to the legacy fallback.
  assert_eq!(
    fs::read(config_directory.join("data/terminal/kitty/custom.conf")).unwrap(),
    b"standalone"
  );
  assert!(!config_directory.join("data/legacy/kitty").exists());
  assert!(!config_directory.join("data/legacy/terminal").exists());

  let mut remaining = fs::read_dir(&config_directory)
    .unwrap()
    .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
    .collect::<Vec<_>>();
  remaining.sort();
  assert_eq!(remaining, vec!["config", "data"]);
  let _ = fs::remove_dir_all(directory);
}

#[test]
fn legacy_language_is_not_imported() {
  let directory = env::temp_dir().join(format!("argvus-language-migration-{}", std::process::id()));
  fs::create_dir_all(directory.join("data/internal")).unwrap();
  fs::write(directory.join("data/language"), "pt-BR").unwrap();
  fs::write(directory.join("data/internal/language"), "pt_BR").unwrap();
  let mut document = ConfigDocument::default();
  import_legacy_values(&mut document, &directory).unwrap();
  assert!(document.get("/session/language").is_none());
  assert_eq!(
    fs::read_to_string(directory.join("data/language")).unwrap(),
    "pt-BR"
  );
  fs::remove_dir_all(directory).unwrap();
}
