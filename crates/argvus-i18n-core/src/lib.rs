//! Shared locale detection, catalog loading, and translation APIs for ARGVUS.

mod plural;

use std::{
  collections::{BTreeSet, HashMap},
  env, fs,
  path::{Path, PathBuf},
  sync::{Mutex, MutexGuard, OnceLock, PoisonError},
};

/// Compatibility handle for consumers that keep the selected translator in
/// their application state. Catalogs are loaded once per root and locale and
/// live for the rest of the process, which is what keeps the handle `Copy`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Lang(&'static I18n);

impl Lang {
  pub fn detect() -> Self {
    Self::for_locale(&detect_locale_from(&catalog_root()))
  }

  pub fn for_locale(locale: &str) -> Self {
    Self::for_domain(locale, "control-center")
  }

  /// Shared cached catalog handle for any application domain.
  pub fn for_domain(locale: &str, domain: &str) -> Self {
    type CatalogKey = (PathBuf, String, String);
    static LOADED: OnceLock<Mutex<HashMap<CatalogKey, Lang>>> = OnceLock::new();
    let root = catalog_root();
    let key = (root.clone(), normalize_locale(locale), domain.to_owned());
    let loaded = LOADED.get_or_init(Default::default);
    if let Some(lang) = lock(loaded).get(&key) {
      return *lang;
    }
    // Load outside the lock so concurrent callers never wait on file I/O.
    let lang = Self(leak(
      I18n::from_locale(&root, locale, domain)
        .expect("application i18n fallback catalog is required"),
    ));
    *lock(loaded).entry(key).or_insert(lang)
  }

  /// Deep-copies the loaded catalogs on every call. `Lang` is already `Copy`
  /// and shares them, so callers should keep the `Lang` instead.
  #[deprecated(note = "keep the `Lang` handle; it is `Copy` and shares the loaded catalogs")]
  pub fn translator(self) -> std::sync::Arc<I18n> {
    std::sync::Arc::new(self.0.clone())
  }

  pub fn locale(self) -> String {
    self.0.locale().to_owned()
  }

  pub fn direction(self) -> TextDirection {
    self.0.direction()
  }

  pub fn tr(self, key: &str) -> String {
    self.0.tr(key)
  }

  /// Translates an integer count using the selected catalog's cardinal rules.
  pub fn tr_plural(self, key: &str, count: i64) -> String {
    self.0.tr_plural(key, count)
  }

  pub fn tr_args<I, K, V>(self, key: &str, args: I) -> String
  where
    I: IntoIterator<Item = (K, V)>,
    K: AsRef<str>,
    V: AsRef<str>,
  {
    self.0.tr_args(key, args)
  }
}

/// Borrowed translation for hot render paths: no allocation and no locking on
/// a hit. Keys without a translation return the key itself, as [`I18n::tr`] does.
pub fn tr(lang: Lang, key: &str) -> &'static str {
  match lang
    .0
    .translations
    .get(key)
    .or_else(|| lang.0.fallback.get(key))
  {
    Some(value) => value,
    None => missing_key(key),
  }
}

/// Missing keys are rare and bounded by the set of keys the code uses, so each
/// one is leaked once and then served from this table.
fn missing_key(key: &str) -> &'static str {
  static MISSING: OnceLock<Mutex<HashMap<String, &'static str>>> = OnceLock::new();
  let mut missing = lock(MISSING.get_or_init(Default::default));
  if let Some(value) = missing.get(key) {
    return value;
  }
  let value: &'static str = Box::leak(Box::<str>::from(key));
  missing.insert(key.to_owned(), value);
  value
}

fn leak(i18n: I18n) -> &'static I18n {
  Box::leak(Box::new(i18n))
}

/// The caches hold plain maps with no invariant a panicking writer could
/// break, so a poisoned lock is recovered rather than propagated.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
  mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

fn catalog_root() -> PathBuf {
  env::var_os("ARGVUS_I18N_DIR")
    .map(PathBuf::from)
    .unwrap_or_else(|| SYSTEM_CATALOG_DIR.into())
}

pub fn na(lang: Lang) -> &'static str {
  tr(lang, "control_center.not_available")
}

pub const FALLBACK_LOCALE: &str = "en-US";
pub const SYSTEM_CATALOG_DIR: &str = "/usr/share/argvus/i18n";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct I18n {
  locale: String,
  direction: TextDirection,
  root: PathBuf,
  domain: String,
  translations: HashMap<String, String>,
  fallback: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocaleInfo {
  pub locale: String,
  pub name: String,
  pub native_name: String,
  pub direction: TextDirection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextDirection {
  LeftToRight,
  RightToLeft,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum I18nError {
  Io(String),
  Json(String),
  InvalidCatalog(String),
  MissingFallback,
}

impl std::fmt::Display for I18nError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Io(value) | Self::Json(value) | Self::InvalidCatalog(value) => f.write_str(value),
      Self::MissingFallback => {
        write!(f, "required fallback locale {FALLBACK_LOCALE} is missing")
      }
    }
  }
}

impl std::error::Error for I18nError {}

impl I18n {
  pub fn new(domain: &str) -> Result<Self, I18nError> {
    Self::from_root(&catalog_root(), domain)
  }

  pub fn from_root(root: &Path, domain: &str) -> Result<Self, I18nError> {
    let locale = detect_locale_from(root);
    Self::from_locale(root, &locale, domain)
  }

  pub fn from_locale(root: &Path, locale: &str, domain: &str) -> Result<Self, I18nError> {
    let normalized = normalize_locale(locale);
    let locale = resolve_locale(root, &normalized).unwrap_or(normalized);
    let direction = read_json(&root.join(&locale).join("locale.json"))
      .ok()
      .and_then(|metadata| {
        metadata
          .get("direction")
          .and_then(|value| value.as_str())
          .map(str::to_owned)
      })
      .as_deref()
      .map(parse_text_direction)
      .unwrap_or(TextDirection::LeftToRight);
    let fallback = load_catalog(root, FALLBACK_LOCALE, domain)?;
    let translations = load_catalog(root, &locale, domain).unwrap_or_default();
    Ok(Self {
      locale,
      direction,
      root: root.to_path_buf(),
      domain: domain.to_owned(),
      translations,
      fallback,
    })
  }

  pub fn locale(&self) -> &str {
    &self.locale
  }

  pub fn direction(&self) -> TextDirection {
    self.direction
  }

  pub fn reload(&mut self) -> Result<(), I18nError> {
    *self = Self::from_root(&self.root, &self.domain)?;
    Ok(())
  }

  pub fn tr(&self, key: &str) -> String {
    self
      .translations
      .get(key)
      .or_else(|| self.fallback.get(key))
      .cloned()
      .unwrap_or_else(|| key.to_owned())
  }

  pub fn tr_args<I, K, V>(&self, key: &str, args: I) -> String
  where
    I: IntoIterator<Item = (K, V)>,
    K: AsRef<str>,
    V: AsRef<str>,
  {
    interpolate(&self.tr(key), args)
  }

  pub fn tr_plural(&self, key: &str, count: i64) -> String {
    let selected = format!("{key}.{}", plural::category(&self.locale, count));
    let other = format!("{key}.other");
    // Choose the fallback's own grammar when the translated form is absent.
    let fallback = format!("{key}.{}", plural::category(FALLBACK_LOCALE, count));
    let value = self
      .translations
      .get(&selected)
      .or_else(|| self.translations.get(&other))
      .or_else(|| self.fallback.get(&fallback))
      .map(String::as_str)
      .unwrap_or(&fallback);
    interpolate(value, [("count", count.to_string())])
  }
}

fn parse_text_direction(value: &str) -> TextDirection {
  match value {
    "rtl" => TextDirection::RightToLeft,
    _ => TextDirection::LeftToRight,
  }
}

pub fn normalize_locale(value: &str) -> String {
  let value = value.split(':').next().unwrap_or(value).trim();
  let value = value.split(['.', '@']).next().unwrap_or(value);
  let mut parts = value.split(['_', '-']).filter(|part| !part.is_empty());
  let language = parts.next().unwrap_or("en").to_ascii_lowercase();
  let mut normalized = language;
  for part in parts {
    normalized.push('-');
    if part.len() == 4 && part.bytes().all(|byte| byte.is_ascii_alphabetic()) {
      normalized.push_str(&part[..1].to_ascii_uppercase());
      normalized.push_str(&part[1..].to_ascii_lowercase());
    } else {
      normalized.push_str(&part.to_ascii_uppercase());
    }
  }
  normalized
}

/// Resolves regional variants only to an installed catalog. Traditional Chinese
/// never silently selects the Simplified Chinese translation.
pub fn resolve_locale(root: &Path, requested: &str) -> Option<String> {
  let requested = normalize_locale(requested);
  let installed = discover_locales(root);
  if installed.iter().any(|info| info.locale == requested) {
    return Some(requested);
  }
  let language = requested.split('-').next().unwrap_or("en");
  let default = match language {
    "en" => "en-US",
    "pt" => "pt-BR",
    "es" => "es-ES",
    "fr" => "fr-FR",
    "it" => "it-IT",
    "de" => "de-DE",
    "ru" => "ru-RU",
    "ja" => "ja-JP",
    "ko" => "ko-KR",
    "ar" => "ar",
    "bn" => "bn-BD",
    "hi" => "hi-IN",
    "id" => "id-ID",
    "tr" => "tr-TR",
    "ur" => "ur-PK",
    "vi" => "vi-VN",
    "zh"
      if !requested.split('-').any(|part| part == "Hant")
        && (requested.split('-').any(|part| part == "Hans")
          || !requested
            .split('-')
            .any(|part| matches!(part, "TW" | "HK" | "MO"))) =>
    {
      "zh-CN"
    }
    _ => return None,
  };
  installed
    .iter()
    .any(|info| info.locale == default)
    .then(|| default.to_owned())
}

/// Path to the user's ARGVUS language and regional locale preferences.
pub fn config_path() -> Option<PathBuf> {
  let root = env::var_os("XDG_CONFIG_HOME")
    .map(PathBuf::from)
    .filter(|path| path.is_absolute())
    .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;
  Some(root.join("locale.conf"))
}

/// Reads a validated `LANG` value without evaluating shell expressions.
pub fn parse_user_locale(contents: &str) -> Option<String> {
  contents
    .lines()
    .filter_map(|line| {
      let (key, value) = line.trim().split_once('=')?;
      if key != "LANG" {
        return None;
      }
      let value = value.trim().trim_matches(['\"', '\'']);
      (!value.is_empty()
        && value != "null"
        && value
          .bytes()
          .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'@' | b'-')))
      .then(|| value.to_owned())
    })
    .next_back()
}

/// Reads the optional ARGVUS interface-language preference.
pub fn parse_user_language(contents: &str) -> Option<String> {
  contents
    .lines()
    .filter_map(|line| {
      let (key, value) = line.trim().split_once('=')?;
      if key != "ARGVUS_LANGUAGE" {
        return None;
      }
      let value = value.trim().trim_matches(['"', '\'']);
      (!value.is_empty()
        && value != "null"
        && value
          .bytes()
          .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-')))
      .then(|| value.to_owned())
    })
    .next_back()
}

/// An unconfigured user gets English even on a Portuguese host.
pub fn user_locale() -> String {
  config_path()
    .and_then(|path| fs::read_to_string(path).ok())
    .and_then(|contents| parse_user_language(&contents).or_else(|| parse_user_locale(&contents)))
    .unwrap_or_else(|| "en_US.UTF-8".into())
}

/// The regional locale remains separate from the ARGVUS interface language.
pub fn user_regional_locale() -> String {
  config_path()
    .and_then(|path| fs::read_to_string(path).ok())
    .and_then(|contents| parse_user_locale(&contents))
    .unwrap_or_else(|| "en_US.UTF-8".into())
}

pub fn detect_locale_from(root: &Path) -> String {
  resolve_locale(root, &user_locale()).unwrap_or_else(|| FALLBACK_LOCALE.to_owned())
}

/// Installed catalogs from the configured runtime root.
pub fn installed_locales() -> Vec<LocaleInfo> {
  discover_locales(&catalog_root())
}

/// Resolves a requested language against the configured catalog installation.
pub fn resolve_installed_locale(requested: &str) -> Option<String> {
  resolve_locale(&catalog_root(), requested)
}

pub fn discover_locales(root: &Path) -> Vec<LocaleInfo> {
  let mut result = fs::read_dir(root)
    .ok()
    .into_iter()
    .flatten()
    .filter_map(Result::ok)
    .filter(|entry| entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false))
    .map(|entry| {
      let locale = normalize_locale(&entry.file_name().to_string_lossy());
      let metadata = read_json(&entry.path().join("locale.json")).ok();
      LocaleInfo {
        name: metadata
          .as_ref()
          .and_then(|v| v.get("name"))
          .and_then(|v| v.as_str())
          .unwrap_or(&locale)
          .into(),
        native_name: metadata
          .as_ref()
          .and_then(|v| v.get("native_name"))
          .and_then(|v| v.as_str())
          .unwrap_or(&locale)
          .into(),
        direction: match metadata
          .as_ref()
          .and_then(|v| v.get("direction"))
          .and_then(|v| v.as_str())
        {
          Some("rtl") => TextDirection::RightToLeft,
          _ => TextDirection::LeftToRight,
        },
        locale,
      }
    })
    .collect::<Vec<_>>();
  result.sort_by(|left, right| left.locale.cmp(&right.locale));
  result
}

pub fn validate_catalogs(root: &Path, requested: Option<&str>) -> Result<(), Vec<String>> {
  let mut errors = Vec::new();
  if let Some(requested) = requested
    && !discover_locales(root)
      .iter()
      .any(|info| info.locale == normalize_locale(requested))
  {
    return Err(vec![format!("unknown locale: {requested}")]);
  }
  let fallback_domains = match load_all_catalogs(root, FALLBACK_LOCALE) {
    Ok(value) => value,
    Err(error) => {
      errors.push(error.to_string());
      return Err(errors);
    }
  };
  for locale in discover_locales(root)
    .into_iter()
    .map(|info| info.locale)
    .filter(|locale| {
      requested
        .map(|wanted| normalize_locale(wanted) == *locale)
        .unwrap_or(true)
    })
  {
    match read_json(&root.join(&locale).join("locale.json")) {
      Ok(metadata) => {
        for field in ["locale", "name", "native_name", "direction"] {
          if metadata
            .get(field)
            .and_then(serde_json::Value::as_str)
            .is_none_or(|value| value.trim().is_empty())
          {
            errors.push(format!("{locale}/locale.json: missing or empty {field}"));
          }
        }
        if metadata.get("locale").and_then(serde_json::Value::as_str) != Some(locale.as_str()) {
          errors.push(format!(
            "{locale}/locale.json: locale does not match directory"
          ));
        }
        if !matches!(
          metadata
            .get("direction")
            .and_then(serde_json::Value::as_str),
          Some("ltr" | "rtl")
        ) {
          errors.push(format!(
            "{locale}/locale.json: direction must be ltr or rtl"
          ));
        }
      }
      Err(error) => errors.push(format!("{locale}/locale.json: {error}")),
    }
    let catalogs = match load_all_catalogs(root, &locale) {
      Ok(value) => value,
      Err(error) => {
        errors.push(format!("{locale}: {error}"));
        continue;
      }
    };
    for (domain, expected) in &fallback_domains {
      let actual = catalogs.get(domain).cloned().unwrap_or_default();
      for key in expected.keys().filter(|key| !actual.contains_key(*key)) {
        errors.push(format!("{locale}/{domain}: missing key {key}"));
      }
      for key in actual.keys().filter(|key| !expected.contains_key(*key)) {
        let plural_base = key
          .rsplit_once('.')
          .filter(|(_, suffix)| plural::extra_categories(&locale).contains(suffix));
        if let Some((base, _)) = plural_base
          && let Some(source) = expected.get(&format!("{base}.other"))
        {
          if actual[key].trim().is_empty() {
            errors.push(format!("{locale}/{domain}: empty translation for {key}"));
          }
          if placeholders(source) != placeholders(&actual[key]) {
            errors.push(format!("{locale}/{domain}: placeholder mismatch for {key}"));
          }
        } else {
          errors.push(format!("{locale}/{domain}: extra key {key}"));
        }
      }
      for (key, value) in expected {
        if let Some(base) = key.strip_suffix(".one")
          && expected.contains_key(&format!("{base}.other"))
        {
          for category in plural::extra_categories(&locale) {
            if !actual.contains_key(&format!("{base}.{category}")) {
              errors.push(format!(
                "{locale}/{domain}: missing plural form {base}.{category}"
              ));
            }
          }
        }
        if actual.get(key).is_some_and(|value| value.trim().is_empty()) && !value.trim().is_empty()
        {
          errors.push(format!("{locale}/{domain}: empty translation for {key}"));
        }
        if let Some(translated) = actual.get(key)
          && placeholders(value) != placeholders(translated)
        {
          errors.push(format!("{locale}/{domain}: placeholder mismatch for {key}"));
        }
      }
    }
    for domain in catalogs
      .keys()
      .filter(|domain| !fallback_domains.contains_key(*domain))
    {
      errors.push(format!("{locale}: extra domain {domain}"));
    }
  }
  if errors.is_empty() {
    Ok(())
  } else {
    Err(errors)
  }
}

fn load_catalog(
  root: &Path,
  locale: &str,
  domain: &str,
) -> Result<HashMap<String, String>, I18nError> {
  let path = root.join(locale).join(format!("{domain}.json"));
  let json = read_json(&path)?;
  let object = json.as_object().ok_or_else(|| {
    I18nError::InvalidCatalog(format!("{} must contain an object", path.display()))
  })?;
  object
    .iter()
    .map(|(key, value)| {
      Ok((
        key.clone(),
        value
          .as_str()
          .ok_or_else(|| I18nError::InvalidCatalog(format!("{domain}.{key} must be a string")))?
          .to_owned(),
      ))
    })
    .collect()
}

fn load_all_catalogs(
  root: &Path,
  locale: &str,
) -> Result<HashMap<String, HashMap<String, String>>, I18nError> {
  let directory = root.join(locale);
  if !directory.is_dir() {
    return Err(I18nError::MissingFallback);
  }
  let mut result = HashMap::new();
  for entry in fs::read_dir(&directory)
    .map_err(|e| I18nError::Io(e.to_string()))?
    .filter_map(Result::ok)
  {
    let path = entry.path();
    if path.extension().and_then(|ext| ext.to_str()) != Some("json")
      || path.file_name().and_then(|name| name.to_str()) == Some("locale.json")
    {
      continue;
    }
    let domain = path
      .file_stem()
      .and_then(|stem| stem.to_str())
      .unwrap_or_default()
      .to_owned();
    result.insert(domain.clone(), load_catalog(root, locale, &domain)?);
  }
  Ok(result)
}

fn read_json(path: &Path) -> Result<serde_json::Value, I18nError> {
  serde_json::from_str(
    &fs::read_to_string(path).map_err(|e| I18nError::Io(format!("{}: {e}", path.display())))?,
  )
  .map_err(|e| I18nError::Json(format!("{}: {e}", path.display())))
}

fn placeholders(value: &str) -> BTreeSet<String> {
  value
    .split('{')
    .skip(1)
    .filter_map(|part| part.split('}').next())
    .filter(|name| !name.is_empty())
    .map(str::to_owned)
    .collect()
}

fn interpolate<I, K, V>(value: &str, args: I) -> String
where
  I: IntoIterator<Item = (K, V)>,
  K: AsRef<str>,
  V: AsRef<str>,
{
  let args: HashMap<String, String> = args
    .into_iter()
    .map(|(key, value)| (key.as_ref().to_owned(), value.as_ref().to_owned()))
    .collect();
  let mut output = String::new();
  let mut remaining = value;
  while let Some(start) = remaining.find('{') {
    output.push_str(&remaining[..start]);
    let Some(end) = remaining[start..].find('}') else {
      output.push_str(&remaining[start..]);
      return output;
    };
    let end = start + end;
    let name = &remaining[start + 1..end];
    output.push_str(
      args
        .get(name)
        .map(String::as_str)
        .unwrap_or(&remaining[start..=end]),
    );
    remaining = &remaining[end + 1..];
  }
  output.push_str(remaining);
  output
}
