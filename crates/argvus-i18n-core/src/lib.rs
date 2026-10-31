//! Shared locale detection, catalog loading, and translation APIs for ARGVUS.

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
    static LOADED: OnceLock<Mutex<HashMap<(PathBuf, String), Lang>>> = OnceLock::new();
    let root = catalog_root();
    let key = (root.clone(), normalize_locale(locale));
    let loaded = LOADED.get_or_init(Default::default);
    if let Some(lang) = lock(loaded).get(&key) {
      return *lang;
    }
    // Load outside the lock so concurrent callers never wait on file I/O.
    let lang = Self(leak(
      I18n::from_locale(&root, locale, "control-center")
        .expect("control-center i18n catalog is required"),
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

  pub fn tr(self, key: &str) -> String {
    self.0.tr(key)
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
    let locale = normalize_locale(locale);
    let fallback = load_catalog(root, FALLBACK_LOCALE, domain)?;
    let translations = load_catalog(root, &locale, domain).unwrap_or_default();
    Ok(Self {
      locale,
      root: root.to_path_buf(),
      domain: domain.to_owned(),
      translations,
      fallback,
    })
  }

  pub fn locale(&self) -> &str {
    &self.locale
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
    let suffix = if count == 1 { "one" } else { "other" };
    let value = self.tr(&format!("{key}.{suffix}"));
    interpolate(&value, [("count", count.to_string())])
  }
}

pub fn normalize_locale(value: &str) -> String {
  let value = value.split(':').next().unwrap_or(value).trim();
  let value = value.split(['.', '@']).next().unwrap_or(value);
  let mut parts = value.split(['_', '-']).filter(|part| !part.is_empty());
  let language = parts.next().unwrap_or("en").to_ascii_lowercase();
  let region = parts.next().map(str::to_ascii_uppercase);
  region.map_or_else(|| language.clone(), |region| format!("{language}-{region}"))
}

/// The only persistent language preference of the logged-in user.
pub fn config_path() -> Option<PathBuf> {
  let root = env::var_os("XDG_CONFIG_HOME")
    .map(PathBuf::from)
    .filter(|path| path.is_absolute())
    .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;
  Some(root.join("locale.conf"))
}

/// Reads LANG without evaluating shell expressions or consulting global state.
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

/// An unconfigured user gets English even on a Portuguese host.
pub fn user_locale() -> String {
  config_path()
    .and_then(|path| fs::read_to_string(path).ok())
    .and_then(|contents| parse_user_locale(&contents))
    .unwrap_or_else(|| "en_US.UTF-8".into())
}

pub fn detect_locale_from(root: &Path) -> String {
  let requested = normalize_locale(&user_locale());
  if discover_locales(root)
    .iter()
    .any(|locale| locale.locale == requested)
  {
    requested
  } else {
    FALLBACK_LOCALE.to_owned()
  }
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
        locale,
      }
    })
    .collect::<Vec<_>>();
  result.sort_by(|left, right| left.locale.cmp(&right.locale));
  result
}

pub fn validate_catalogs(root: &Path, requested: Option<&str>) -> Result<(), Vec<String>> {
  let mut errors = Vec::new();
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
        errors.push(format!("{locale}/{domain}: extra key {key}"));
      }
      for (key, value) in expected {
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
  args
    .into_iter()
    .fold(value.to_owned(), |result, (key, replacement)| {
      result.replace(&format!("{{{}}}", key.as_ref()), replacement.as_ref())
    })
}
