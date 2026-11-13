use argvus_i18n::{
  I18n, Lang, detect_locale_from, discover_locales, normalize_locale, tr, validate_catalogs,
};
use std::{
  collections::HashMap,
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
  fs::write(root.join("pt-BR/common.json"), r#"{"save":"Salvar","welcome":"Bem-vindo, {name}!","items.one":"{count} item","items.other":"{count} itens","items.many":"{count} itens"}"#).unwrap();
  fs::write(root.join("en-US/locale.json"), r#"{"locale":"en-US","name":"English (United States)","native_name":"English (United States)"}"#).unwrap();
  fs::write(
    root.join("pt-BR/locale.json"),
    r#"{"locale":"pt-BR","name":"Portuguese (Brazil)","native_name":"Português (Brasil)"}"#,
  )
  .unwrap();
  root
}

fn control_center_catalogs() -> PathBuf {
  let catalogs = root();
  fs::write(
    catalogs.join("en-US/control-center.json"),
    include_str!("../../../resources/locales/en-US/control-center.json"),
  )
  .unwrap();
  fs::write(
    catalogs.join("pt-BR/control-center.json"),
    include_str!("../../../resources/locales/pt-BR/control-center.json"),
  )
  .unwrap();
  catalogs
}

fn env_lock() -> &'static Mutex<()> {
  static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
  LOCK.get_or_init(|| Mutex::new(()))
}

#[test]
fn normalizes_linux_locales() {
  assert_eq!(normalize_locale("pt_BR.UTF-8"), "pt-BR");
  assert_eq!(normalize_locale("en_US@variant"), "en-US");
  assert_eq!(normalize_locale("zh_hans_cn.UTF-8"), "zh-Hans-CN");
}

#[test]
fn resolves_installed_regional_variants_without_changing_chinese_script() {
  let root = root();
  for locale in ["es-ES", "zh-CN", "ja-JP"] {
    fs::create_dir_all(root.join(locale)).unwrap();
  }
  for (requested, expected) in [
    ("es_MX.UTF-8", Some("es-ES")),
    ("ja", Some("ja-JP")),
    ("zh-Hans-SG", Some("zh-CN")),
    ("zh-Hans-TW", Some("zh-CN")),
    ("zh-Hant-CN", None),
    ("zh_TW.UTF-8", None),
    ("zh-Hant", None),
  ] {
    assert_eq!(
      argvus_i18n::resolve_locale(&root, requested).as_deref(),
      expected
    );
  }
}

#[test]
fn resolves_new_application_languages_and_keeps_portuguese_variants_distinct() {
  let root = root();
  for locale in [
    "ar", "bn-BD", "hi-IN", "id-ID", "pt-PT", "tr-TR", "ur-PK", "vi-VN",
  ] {
    fs::create_dir_all(root.join(locale)).unwrap();
  }
  for (requested, expected) in [
    ("ar_EG.UTF-8", Some("ar")),
    ("bn", Some("bn-BD")),
    ("hi", Some("hi-IN")),
    ("id", Some("id-ID")),
    ("pt_PT.UTF-8", Some("pt-PT")),
    ("pt", Some("pt-BR")),
    ("tr", Some("tr-TR")),
    ("ur", Some("ur-PK")),
    ("vi", Some("vi-VN")),
  ] {
    assert_eq!(
      argvus_i18n::resolve_locale(&root, requested).as_deref(),
      expected
    );
  }
}

#[test]
fn plural_fallback_uses_english_grammar_and_interpolation_is_single_pass() {
  let root = root();
  fs::create_dir_all(root.join("ru-RU")).unwrap();
  fs::write(root.join("ru-RU/common.json"), "{}").unwrap();
  let russian = I18n::from_locale(&root, "ru-RU", "common").unwrap();
  assert_eq!(russian.tr_plural("items", 21), "21 items");
  let english = I18n::from_locale(&root, "en-US", "common").unwrap();
  assert_eq!(
    english.tr_args("welcome", [("name", "{count}"), ("count", "42")]),
    "Welcome, {count}!"
  );
  assert!(validate_catalogs(&root, Some("unknown")).is_err());
}

#[test]
fn shipped_catalogs_cover_all_eighteen_languages_and_plural_forms() {
  let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../resources/locales");
  validate_catalogs(&root, None).unwrap_or_else(|errors| panic!("{}", errors.join("\n")));
  let locales = discover_locales(&root);
  for locale in [
    "ar", "bn-BD", "de-DE", "en-US", "es-ES", "fr-FR", "hi-IN", "id-ID", "it-IT", "ja-JP", "ko-KR",
    "pt-BR", "pt-PT", "ru-RU", "tr-TR", "ur-PK", "vi-VN", "zh-CN",
  ] {
    assert!(locales.iter().any(|info| info.locale == locale));
    let i18n = I18n::from_locale(&root, locale, "common").unwrap();
    let direction = if matches!(locale, "ar" | "ur-PK") {
      argvus_i18n::TextDirection::RightToLeft
    } else {
      argvus_i18n::TextDirection::LeftToRight
    };
    assert_eq!(i18n.direction(), direction, "{locale}");
    for count in [0, 1, 2, 5, 11, 21, 22, 1_000_000] {
      assert!(
        !i18n
          .tr_plural("updates.available", count)
          .contains("updates.available")
      );
    }
  }
  assert_eq!(locales.len(), 18);
  let russian = I18n::from_locale(&root, "ru-RU", "common").unwrap();
  assert!(
    russian
      .tr_plural("updates.available", 21)
      .contains("обновление")
  );
  assert!(
    russian
      .tr_plural("updates.available", 22)
      .contains("обновления")
  );
  assert!(
    russian
      .tr_plural("updates.available", 25)
      .contains("обновлений")
  );
}

#[test]
fn locale_metadata_controls_text_direction() {
  let root = root();
  fs::create_dir_all(root.join("ar")).unwrap();
  fs::write(
    root.join("ar/locale.json"),
    r#"{"locale":"ar","name":"Arabic","native_name":"العربية","direction":"rtl"}"#,
  )
  .unwrap();
  fs::write(root.join("ar/common.json"), "{}").unwrap();
  let language = I18n::from_locale(&root, "ar", "common").unwrap();
  assert_eq!(
    language.direction(),
    argvus_i18n::TextDirection::RightToLeft
  );
}

#[test]
fn shipped_translations_preserve_command_options_and_code_examples() {
  let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../resources/locales");
  for entry in fs::read_dir(root.join("en-US")).unwrap() {
    let path = entry.unwrap().path();
    if path.extension().and_then(|value| value.to_str()) != Some("json")
      || path.file_name().unwrap() == "locale.json"
    {
      continue;
    }
    let source: HashMap<String, String> =
      serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    for locale in discover_locales(&root) {
      let translated: HashMap<String, String> = serde_json::from_str(
        &fs::read_to_string(root.join(&locale.locale).join(path.file_name().unwrap())).unwrap(),
      )
      .unwrap();
      for (key, text) in &source {
        let actual = &translated[key];
        for code in text.split('`').skip(1).step_by(2) {
          assert!(
            actual.contains(code),
            "{} {key}: changed code {code}",
            locale.locale
          );
        }
        for word in text.split_whitespace() {
          let word = word.trim_start_matches(['(', '[', '\'', '"', '`']);
          if word.starts_with("--") {
            let option: String = word
              .chars()
              .take_while(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '-' | '_')
              })
              .collect();
            assert!(
              actual.contains(&option),
              "{} {key}: changed option {option}",
              locale.locale
            );
          }
        }
      }
    }
  }
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
  fs::write(config.join("locale.conf"), "LANG=pt_BR.UTF-8\n").unwrap();
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
  fs::write(root.join("pt-BR/common.json"), r#"{"save":"Salvar","welcome":"Bem-vindo!","items.one":"{count} item","items.other":"{count} itens","items.many":"{count} itens"}"#).unwrap();
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
  let catalogs = control_center_catalogs();
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
fn control_center_common_labels_are_translated_to_brazilian_portuguese() {
  let catalogs = control_center_catalogs();
  let portuguese = I18n::from_locale(&catalogs, "pt-BR", "control-center").unwrap();
  let translations = [
    ("control_center.welcome_title", "Boas-vindas ao ARGVUS"),
    ("control_center.config", "Configuração:"),
    ("control_center.autostart", "Inicialização automática"),
    ("control_center.donate", "Doar"),
    ("control_center.blur", "Desfoque"),
    ("control_center.download", "Baixar:"),
    ("control_center.section_presets", "Predefinições"),
    ("control_center.home_directory", "Pasta pessoal"),
    ("control_center.logs", "Registros"),
    ("control_center.search_domains", "Domínios de pesquisa:"),
    ("control_center.snippets", "Trechos"),
    ("control_center.hostname", "Nome do host"),
    ("control_center.logs_details", "Registros > Detalhes"),
    ("control_center.taskbar_focus_icon", "Foco"),
    ("control_center.timeout", "Tempo limite"),
    ("control_center.timeout_default", "Tempo limite: padrão"),
    (
      "control_center.timeout_must_be_between_0_and_60_seconds",
      "O tempo limite deve estar entre 0 e 60 segundos.",
    ),
    ("control_center.snapshot.title", "Instantâneo"),
    ("control_center.workspaces", "Espaços de trabalho"),
  ];

  for (key, expected) in translations {
    assert_eq!(
      portuguese.tr(key),
      expected,
      "incorrect translation for {key}"
    );
  }
}

#[test]
fn inherited_lc_all_does_not_configure_a_new_user() {
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
fn invalid_catalog_and_missing_fallback_are_reported() {
  let root = root();
  fs::write(root.join("pt-BR/common.json"), "{invalid").unwrap();
  assert!(validate_catalogs(&root, Some("pt-BR")).is_err());
  fs::remove_dir_all(root.join("en-US")).unwrap();
  assert!(validate_catalogs(&root, None).is_err());
}

#[test]
fn inherited_lc_messages_does_not_configure_a_new_user() {
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
  assert_eq!(detect_locale_from(&root), "en-US");
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
fn inherited_lang_does_not_configure_a_new_user() {
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
  assert_eq!(detect_locale_from(&root), "en-US");
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
  fs::write(config.join("locale.conf"), "LANG=pt_BR.UTF-8\n").unwrap();
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

#[test]
fn only_user_locale_conf_controls_detection() {
  let _guard = env_lock().lock().unwrap();
  let root = root();
  let config = root.join("isolated");
  fs::create_dir_all(config.join("argvus/data/internal")).unwrap();
  fs::create_dir_all(config.join("argvus/config")).unwrap();
  fs::write(config.join("argvus/data/language"), "pt-BR").unwrap();
  fs::write(config.join("argvus/data/internal/language"), "pt_BR").unwrap();
  fs::write(
    config.join("argvus/config/session.json"),
    r#"{"language":"pt-BR"}"#,
  )
  .unwrap();
  let names = [
    "XDG_CONFIG_HOME",
    "LANG",
    "LC_ALL",
    "LC_MESSAGES",
    "LANGUAGE",
  ];
  let previous: Vec<_> = names.iter().map(std::env::var_os).collect();
  unsafe {
    std::env::set_var("XDG_CONFIG_HOME", &config);
    for name in &names[1..] {
      std::env::set_var(name, "pt_BR.UTF-8");
    }
  }
  assert_eq!(detect_locale_from(&root), "en-US");
  assert!(!config.join("locale.conf").exists());
  for (contents, expected) in [
    ("", "en-US"),
    ("#LANG=pt_BR.UTF-8\n", "en-US"),
    ("LANG=\"pt_BR.UTF-8\"\nLC_TIME=en_US.UTF-8\n", "pt-BR"),
    ("LANG=en_US.UTF-8\n", "en-US"),
    ("LANG=$(id)\n", "en-US"),
    ("LANG=xx_XX.UTF-8\n", "en-US"),
  ] {
    fs::write(config.join("locale.conf"), contents).unwrap();
    assert_eq!(detect_locale_from(&root), expected, "{contents}");
  }
  unsafe {
    for (name, value) in names.iter().zip(previous) {
      match value {
        Some(value) => std::env::set_var(name, value),
        None => std::env::remove_var(name),
      }
    }
  }
  fs::remove_dir_all(root).unwrap();
}

#[test]
fn user_locale_parser_ignores_invalid_values_and_handles_quotes() {
  for (contents, expected) in [
    ("# comment\n LANG='pt_BR.UTF-8'\n", Some("pt_BR.UTF-8")),
    ("LANG=en_US.UTF-8\nLANG=pt_BR.UTF-8\n", Some("pt_BR.UTF-8")),
    ("LC_MESSAGES=pt_BR.UTF-8\n", None),
    ("LANG=\n", None),
    ("LANG=null\n", None),
    ("LANG=$(touch x)\n", None),
  ] {
    assert_eq!(
      argvus_i18n::parse_user_locale(contents).as_deref(),
      expected
    );
  }
}

#[test]
fn application_language_preference_is_parsed_separately_from_regional_locale() {
  for (contents, expected) in [
    ("ARGVUS_LANGUAGE=ar\nLANG=pt_BR.UTF-8\n", Some("ar")),
    ("ARGVUS_LANGUAGE='pt-PT'\n", Some("pt-PT")),
    ("ARGVUS_LANGUAGE=$(touch x)\n", None),
    ("LANG=ur_PK.UTF-8\n", None),
  ] {
    assert_eq!(
      argvus_i18n::parse_user_language(contents).as_deref(),
      expected
    );
  }
  assert_eq!(
    argvus_i18n::parse_user_locale("ARGVUS_LANGUAGE=ar\nLANG=pt_BR.UTF-8\n").as_deref(),
    Some("pt_BR.UTF-8")
  );
}

#[test]
fn app_language_overrides_catalog_detection_without_replacing_regional_locale() {
  let _guard = env_lock().lock().unwrap();
  let root = root();
  fs::create_dir_all(root.join("ar")).unwrap();
  fs::write(
    root.join("ar/locale.json"),
    r#"{"locale":"ar","name":"Arabic","native_name":"العربية","direction":"rtl"}"#,
  )
  .unwrap();
  let config = root.join("config");
  fs::create_dir_all(&config).unwrap();
  fs::write(
    config.join("locale.conf"),
    "ARGVUS_LANGUAGE=ar\nLANG=pt_BR.UTF-8\n",
  )
  .unwrap();
  let previous = std::env::var_os("XDG_CONFIG_HOME");
  unsafe { std::env::set_var("XDG_CONFIG_HOME", &config) };
  assert_eq!(detect_locale_from(&root), "ar");
  assert_eq!(argvus_i18n::user_regional_locale(), "pt_BR.UTF-8");
  unsafe {
    match previous {
      Some(value) => std::env::set_var("XDG_CONFIG_HOME", value),
      None => std::env::remove_var("XDG_CONFIG_HOME"),
    }
  }
}
