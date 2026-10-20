use argvus_i18n::{
  I18n, Lang, detect_locale_from, discover_locales, normalize_locale, tr, validate_catalogs,
};
use std::{
  fs,
  path::PathBuf,
  sync::{Mutex, OnceLock},
};

fn root() -> PathBuf {
  let stamp = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_nanos();
  let root = std::env::temp_dir().join(format!("argvus-i18n-{}-{stamp}", std::process::id()));
  let _ = fs::remove_dir_all(&root);
  fs::create_dir_all(root.join("en-US")).unwrap();
  fs::create_dir_all(root.join("pt-BR")).unwrap();
  fs::write(root.join("en-US/common.json"), r#"{"save":"Save","welcome":"Welcome, {name}!","items.one":"{count} item","items.other":"{count} items"}"#).unwrap();
  fs::write(root.join("pt-BR/common.json"), r#"{"save":"Salvar","welcome":"Bem-vindo, {name}!","items.one":"{count} item","items.other":"{count} itens"}"#).unwrap();
  fs::write(root.join("en-US/locale.json"), r#"{"locale":"en-US","name":"English (United States)","native_name":"English (United States)"}"#).unwrap();
  fs::write(
    root.join("pt-BR/locale.json"),
    r#"{"locale":"pt-BR","name":"Portuguese (Brazil)","native_name":"Português (Brasil)"}"#,
  )
  .unwrap();
  root
}

fn env_lock() -> &'static Mutex<()> {
  static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
  LOCK.get_or_init(|| Mutex::new(()))
}

#[test]
fn normalizes_linux_locales() {
  assert_eq!(normalize_locale("pt_BR.UTF-8"), "pt-BR");
  assert_eq!(normalize_locale("en_US@variant"), "en-US");
}

#[test]
fn translates_interpolates_and_pluralizes() {
  let i18n = I18n::from_locale(&root(), "pt-BR", "common").unwrap();
  assert_eq!(i18n.tr("save"), "Salvar");
  assert_eq!(
    i18n.tr_args("welcome", [("name", "William")]),
    "Bem-vindo, William!"
  );
  assert_eq!(i18n.tr_plural("items", 2), "2 itens");
}

#[test]
fn falls_back_to_english_then_key() {
  let root = root();
  fs::write(root.join("pt-BR/common.json"), r#"{"save":"Salvar"}"#).unwrap();
  let i18n = I18n::from_locale(&root, "pt-BR", "common").unwrap();
  assert_eq!(i18n.tr("welcome"), "Welcome, {name}!");
  assert_eq!(i18n.tr("missing"), "missing");
}

#[test]
fn explicit_config_beats_locale_environment() {
  let _guard = env_lock().lock().unwrap();
  let root = root();
  let config = root.join("config");
  fs::create_dir_all(config.join("argvus/data")).unwrap();
  fs::write(config.join("argvus/data/language"), "pt-BR").unwrap();
  let old_config = std::env::var_os("XDG_CONFIG_HOME");
  let old_lang = std::env::var_os("LANG");
  unsafe {
    std::env::set_var("XDG_CONFIG_HOME", &config);
    std::env::set_var("LANG", "en_US.UTF-8");
  }
  assert_eq!(detect_locale_from(&root), "pt-BR");
  unsafe {
    if let Some(value) = old_config {
      std::env::set_var("XDG_CONFIG_HOME", value);
    } else {
      std::env::remove_var("XDG_CONFIG_HOME");
    }
    if let Some(value) = old_lang {
      std::env::set_var("LANG", value);
    } else {
      std::env::remove_var("LANG");
    }
  }
}

#[test]
fn validates_placeholder_mismatch() {
  let root = root();
  fs::write(root.join("pt-BR/common.json"), r#"{"save":"Salvar","welcome":"Bem-vindo!","items.one":"{count} item","items.other":"{count} itens"}"#).unwrap();
  let errors = validate_catalogs(&root, Some("pt-BR")).unwrap_err();
  assert!(
    errors
      .iter()
      .any(|error| error.contains("placeholder mismatch"))
  );
}

#[test]
fn discovers_installed_languages() {
  let locales = discover_locales(&root());
  assert_eq!(
    locales
      .iter()
      .map(|item| item.locale.as_str())
      .collect::<Vec<_>>(),
    vec!["en-US", "pt-BR"]
  );
}

#[test]
fn control_center_input_catalogs_resolve_page_keys() {
  let catalogs = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../resources/locales");
  let keys = [
    "control_center.mouse_touchpad",
    "control_center.input_mouse",
    "control_center.input_touchpad",
    "control_center.input_pointer_speed",
    "control_center.input_acceleration",
    "control_center.input_acceleration_adaptive",
    "control_center.input_acceleration_flat",
    "control_center.input_natural_scrolling",
    "control_center.input_scroll_speed",
    "control_center.input_left_handed",
    "control_center.input_tap_to_click",
    "control_center.input_tap_and_drag",
    "control_center.input_two_finger_right_click",
    "control_center.input_disable_while_typing",
    "control_center.input_hardware_mouse",
    "control_center.input_hardware_device",
    "control_center.input_hardware_device_unknown",
    "control_center.input_hardware_profile",
    "control_center.input_hardware_profile_number",
    "control_center.input_hardware_dpi",
    "control_center.input_hardware_dpi_unit",
    "control_center.input_hardware_polling_rate",
    "control_center.input_hardware_polling_unit",
    "control_center.input_hardware_applying",
    "control_center.input_hardware_apply_failed",
    "control_center.input_applied",
    "control_center.input_apply_failed",
    "control_center.input_applying",
    "control_center.input_loading",
  ];

  for locale in ["en-US", "pt-BR", "zz-ZZ"] {
    let catalog = I18n::from_locale(&catalogs, locale, "control-center").unwrap();
    for key in keys {
      assert_ne!(catalog.tr(key), key, "unresolved {key} for {locale}");
    }
  }
}

#[test]
fn lc_all_has_priority_over_other_locale_variables() {
  let _guard = env_lock().lock().unwrap();
  let root = root();
  let config = root.join("empty-config");
  fs::create_dir_all(&config).unwrap();
  let old_config = std::env::var_os("XDG_CONFIG_HOME");
  let old_all = std::env::var_os("LC_ALL");
  let old_messages = std::env::var_os("LC_MESSAGES");
  let old_lang = std::env::var_os("LANG");
  unsafe {
    std::env::set_var("XDG_CONFIG_HOME", &config);
    std::env::set_var("LC_ALL", "pt_BR.UTF-8");
    std::env::set_var("LC_MESSAGES", "en_US.UTF-8");
    std::env::set_var("LANG", "en_US.UTF-8");
  }
  assert_eq!(detect_locale_from(&root), "pt-BR");
  unsafe {
    for (name, value) in [
      ("XDG_CONFIG_HOME", old_config),
      ("LC_ALL", old_all),
      ("LC_MESSAGES", old_messages),
      ("LANG", old_lang),
    ] {
      if let Some(value) = value {
        std::env::set_var(name, value);
      } else {
        std::env::remove_var(name);
      }
    }
  }
}

#[test]
fn invalid_catalog_and_missing_fallback_are_reported() {
  let root = root();
  fs::write(root.join("pt-BR/common.json"), "{invalid").unwrap();
  assert!(validate_catalogs(&root, Some("pt-BR")).is_err());
  fs::remove_dir_all(root.join("en-US")).unwrap();
  assert!(validate_catalogs(&root, None).is_err());
}

#[test]
fn lc_messages_beats_lang() {
  let _guard = env_lock().lock().unwrap();
  let root = root();
  let config = root.join("empty-config");
  fs::create_dir_all(&config).unwrap();
  let old_config = std::env::var_os("XDG_CONFIG_HOME");
  let old_messages = std::env::var_os("LC_MESSAGES");
  let old_lang = std::env::var_os("LANG");
  unsafe {
    std::env::set_var("XDG_CONFIG_HOME", &config);
    std::env::remove_var("LC_ALL");
    std::env::set_var("LC_MESSAGES", "pt_BR.UTF-8");
    std::env::set_var("LANG", "en_US.UTF-8");
  }
  assert_eq!(detect_locale_from(&root), "pt-BR");
  unsafe {
    if let Some(value) = old_config {
      std::env::set_var("XDG_CONFIG_HOME", value);
    } else {
      std::env::remove_var("XDG_CONFIG_HOME");
    }
    if let Some(value) = old_messages {
      std::env::set_var("LC_MESSAGES", value);
    } else {
      std::env::remove_var("LC_MESSAGES");
    }
    if let Some(value) = old_lang {
      std::env::set_var("LANG", value);
    } else {
      std::env::remove_var("LANG");
    }
  }
}

#[test]
fn lang_is_used_when_higher_priority_values_are_empty() {
  let _guard = env_lock().lock().unwrap();
  let root = root();
  let config = root.join("empty-config");
  fs::create_dir_all(&config).unwrap();
  let old_config = std::env::var_os("XDG_CONFIG_HOME");
  let old_messages = std::env::var_os("LC_MESSAGES");
  let old_lang = std::env::var_os("LANG");
  unsafe {
    std::env::set_var("XDG_CONFIG_HOME", &config);
    std::env::remove_var("LC_ALL");
    std::env::remove_var("LC_MESSAGES");
    std::env::set_var("LANG", "pt_BR.UTF-8");
  }
  assert_eq!(detect_locale_from(&root), "pt-BR");
  unsafe {
    if let Some(value) = old_config {
      std::env::set_var("XDG_CONFIG_HOME", value);
    } else {
      std::env::remove_var("XDG_CONFIG_HOME");
    }
    if let Some(value) = old_messages {
      std::env::set_var("LC_MESSAGES", value);
    } else {
      std::env::remove_var("LC_MESSAGES");
    }
    if let Some(value) = old_lang {
      std::env::set_var("LANG", value);
    } else {
      std::env::remove_var("LANG");
    }
  }
}

#[test]
fn invalid_locale_falls_back_to_en_us() {
  let _guard = env_lock().lock().unwrap();
  let root = root();
  let config = root.join("empty-config");
  fs::create_dir_all(&config).unwrap();
  let old_config = std::env::var_os("XDG_CONFIG_HOME");
  let old_all = std::env::var_os("LC_ALL");
  let old_messages = std::env::var_os("LC_MESSAGES");
  let old_lang = std::env::var_os("LANG");
  unsafe {
    std::env::set_var("XDG_CONFIG_HOME", &config);
    std::env::set_var("LC_ALL", "xx_XX.UTF-8");
    std::env::set_var("LC_MESSAGES", "xx_XX.UTF-8");
    std::env::set_var("LANG", "xx_XX.UTF-8");
  }
  assert_eq!(detect_locale_from(&root), "en-US");
  unsafe {
    for (name, value) in [
      ("XDG_CONFIG_HOME", old_config),
      ("LC_ALL", old_all),
      ("LC_MESSAGES", old_messages),
      ("LANG", old_lang),
    ] {
      if let Some(value) = value {
        std::env::set_var(name, value);
      } else {
        std::env::remove_var(name);
      }
    }
  }
}

#[test]
fn absent_requested_locale_uses_fallback_catalog() {
  let i18n = I18n::from_locale(&root(), "de-DE", "common").unwrap();
  assert_eq!(i18n.locale(), "de-DE");
  assert_eq!(i18n.tr("save"), "Save");
}

#[test]
fn missing_locale_file_is_reported_against_canonical_catalog() {
  let root = root();
  fs::remove_file(root.join("pt-BR/common.json")).unwrap();
  let errors = validate_catalogs(&root, Some("pt-BR")).unwrap_err();
  assert!(
    errors
      .iter()
      .any(|error| error.contains("missing key save"))
  );
}

#[test]
fn extra_key_is_reported() {
  let root = root();
  fs::write(root.join("pt-BR/common.json"), r#"{"save":"Salvar","welcome":"Bem-vindo, {name}!","items.one":"{count} item","items.other":"{count} itens","extra":"Extra"}"#).unwrap();
  let errors = validate_catalogs(&root, Some("pt-BR")).unwrap_err();
  assert!(errors.iter().any(|error| error.contains("extra key extra")));
}

#[test]
fn invalid_value_type_is_reported() {
  let root = root();
  fs::write(root.join("pt-BR/common.json"), r#"{"save":42}"#).unwrap();
  assert!(validate_catalogs(&root, Some("pt-BR")).is_err());
}

#[test]
fn explicit_xdg_config_path_is_used() {
  let _guard = env_lock().lock().unwrap();
  let root = root();
  let config = root.join("custom-xdg");
  fs::create_dir_all(config.join("argvus/data")).unwrap();
  fs::write(config.join("argvus/data/language"), "pt-BR").unwrap();
  let old_config = std::env::var_os("XDG_CONFIG_HOME");
  let old_all = std::env::var_os("LC_ALL");
  unsafe {
    std::env::set_var("XDG_CONFIG_HOME", &config);
    std::env::set_var("LC_ALL", "en_US.UTF-8");
  }
  assert_eq!(detect_locale_from(&root), "pt-BR");
  unsafe {
    if let Some(value) = old_config {
      std::env::set_var("XDG_CONFIG_HOME", value);
    } else {
      std::env::remove_var("XDG_CONFIG_HOME");
    }
    if let Some(value) = old_all {
      std::env::set_var("LC_ALL", value);
    } else {
      std::env::remove_var("LC_ALL");
    }
  }
}

#[test]
fn borrowed_tr_matches_owned_translation_and_reuses_catalogs() {
  let _guard = env_lock().lock().unwrap();
  let root = root();
  fs::write(
    root.join("pt-BR/control-center.json"),
    r#"{"save":"Salvar"}"#,
  )
  .unwrap();
  fs::write(
    root.join("en-US/control-center.json"),
    r#"{"save":"Save","cancel":"Cancel"}"#,
  )
  .unwrap();
  let old_root = std::env::var_os("ARGVUS_I18N_DIR");
  unsafe {
    std::env::set_var("ARGVUS_I18N_DIR", &root);
  }

  let pt = Lang::for_locale("pt-BR");
  assert_eq!(pt, Lang::for_locale("pt_BR.UTF-8"));
  assert_eq!(tr(pt, "save"), pt.tr("save"));
  assert_eq!(tr(pt, "cancel"), "Cancel");
  assert_eq!(tr(pt, "missing.key"), "missing.key");
  assert!(std::ptr::eq(tr(pt, "missing.key"), tr(pt, "missing.key")));

  unsafe {
    match old_root {
      Some(value) => std::env::set_var("ARGVUS_I18N_DIR", value),
      None => std::env::remove_var("ARGVUS_I18N_DIR"),
    }
  }
}
