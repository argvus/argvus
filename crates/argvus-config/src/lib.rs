//! Canonical user configuration for the ARGVUS desktop.
//!
//! The document stores logical user preferences loaded from modular section
//! files. Generated files and native application overrides remain projections.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::os::fd::AsRawFd;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::path::{Path, PathBuf};
use thiserror::Error;

pub mod accent;
pub mod accent_switch;
pub mod appearance_runtime;
pub mod audio;
pub mod cli;
pub mod css_edit;
pub mod display;
pub mod effects;
pub mod hyprland_ipc;
pub mod i18n;
pub mod icons;
pub mod monitor_state;
pub mod project;
pub mod session_loading;
pub mod session_paths;
pub mod session_plan;
pub mod session_prepare;
pub mod session_reload;
pub mod session_splash;
pub mod session_timing;
pub mod session_units;
pub mod text_edit;
pub mod theme_apply;
pub mod theme_body;
pub mod theme_build;
pub mod theme_files;
pub mod theme_finish;
pub mod theme_migration;
pub mod theme_resolve;
pub mod theme_runtime;
pub mod theme_surfaces;
pub mod theme_transition;
pub mod user_copy;
pub mod waybar;

pub const CURRENT_SCHEMA_VERSION: u32 = 1;
const LOCK_FILE: &str = "data/internal/config.lock";
const BACKUP_FILE: &str = "data/backups/config.json.bak";
/// The previous backup, kept when a new one replaces `BACKUP_FILE`.
const OLDER_BACKUP_NAME: &str = "config.json.bak.1";

/// Replaces the backup with `contents`, keeping the previous backup as the older
/// generation. The write is atomic (temporary file, sync, rename), and an unchanged
/// backup is left alone so an identical save does not drop the older generation.
fn write_backup(backup: &Path, contents: &str) -> io::Result<()> {
  if let Some(parent) = backup.parent() {
    fs::DirBuilder::new()
      .recursive(true)
      .mode(0o700)
      .create(parent)?;
  }
  if fs::read_to_string(backup).ok().as_deref() == Some(contents) {
    return Ok(());
  }
  if let (true, Some(parent)) = (backup.is_file(), backup.parent()) {
    fs::rename(backup, parent.join(OLDER_BACKUP_NAME))?;
  }
  let temporary = backup.with_extension(format!("bak.tmp-{}", std::process::id()));
  let _ = fs::remove_file(&temporary);
  let mut file = OpenOptions::new()
    .create_new(true)
    .write(true)
    .mode(0o600)
    .open(&temporary)?;
  file.write_all(contents.as_bytes())?;
  file.sync_all()?;
  fs::rename(temporary, backup)
}
const MODULE_DIRECTORY: &str = "config";

const KNOWN_SECTIONS: [&str; 17] = [
  "appearance",
  "layout",
  "effects",
  "fonts",
  "control_panel",
  "default_apps",
  "keyboard_shortcuts",
  "hyprland",
  "displays",
  "power",
  "session",
  "calendar",
  "removable_devices",
  "audio",
  "taskbar",
  "projects",
  "snippets",
];

#[derive(Debug, Error)]
pub enum ConfigError {
  #[error("configuration directory is unavailable")]
  MissingConfigDirectory,
  #[error("invalid configuration JSON: {0}")]
  InvalidJson(#[from] serde_json::Error),
  #[error("invalid configuration: {0}")]
  Invalid(String),
  #[error("configuration I/O failed: {0}")]
  Io(#[from] io::Error),
}

pub type ConfigResult<T> = Result<T, ConfigError>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConfigDocument {
  #[serde(default = "default_schema_version")]
  pub schema_version: u32,
  #[serde(flatten)]
  pub sections: BTreeMap<String, Value>,
}

impl Default for ConfigDocument {
  fn default() -> Self {
    Self {
      schema_version: CURRENT_SCHEMA_VERSION,
      sections: BTreeMap::new(),
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigScope {
  Appearance,
  Desktop,
}

impl ConfigScope {
  pub fn parse(value: &str) -> Option<Self> {
    match value {
      "appearance" | "theme" => Some(Self::Appearance),
      "desktop" | "all" => Some(Self::Desktop),
      _ => None,
    }
  }

  pub fn sections(self) -> &'static [&'static str] {
    match self {
      Self::Appearance => &[
        "appearance",
        "layout",
        "effects",
        "fonts",
        "control_panel",
        "taskbar",
      ],
      Self::Desktop => &[
        "appearance",
        "layout",
        "effects",
        "fonts",
        "control_panel",
        "default_apps",
        "keyboard_shortcuts",
        "hyprland",
        "displays",
        "power",
        "session",
        "calendar",
        "removable_devices",
        "audio",
        "taskbar",
        "projects",
      ],
    }
  }
}

impl ConfigDocument {
  pub fn load(path: &Path) -> ConfigResult<Self> {
    if !path.exists() {
      return Ok(Self::default());
    }
    let text = fs::read_to_string(path)?;
    let mut document: Self = serde_json::from_str(&text)?;
    migrate_legacy_sections(&mut document)?;
    document.migrate_animation_settings();
    document.validate()?;
    Ok(document)
  }

  /// Brings stored animation values in line with Hyprland 0.56.2. Retired keys are
  /// dropped, styles the leaf no longer offers are dropped, and bezier Y points
  /// outside the range the compositor parses are clamped. The packaged defaults
  /// fill whatever is removed, so an older config still loads.
  fn migrate_animation_settings(&mut self) {
    let Some(effects) = self
      .sections
      .get_mut("effects")
      .and_then(Value::as_object_mut)
    else {
      return;
    };
    for key in RETIRED_ANIMATION_KEYS {
      effects.remove(key);
    }
    let stale_styles: Vec<String> = effects
      .iter()
      .filter(|(key, value)| {
        key.starts_with("animation_")
          && key.ends_with("_style")
          && !value
            .as_str()
            .is_some_and(|choice| animation_styles(key).contains(&choice))
      })
      .map(|(key, _)| key.clone())
      .collect();
    for key in stale_styles {
      effects.remove(&key);
    }
    for key in ["animation_bezier_y1", "animation_bezier_y2"] {
      if let Some(value) = effects.get(key).and_then(Value::as_f64)
        && !(-1.0..=2.0).contains(&value)
      {
        effects.insert(key.to_owned(), Value::from(value.clamp(-1.0, 2.0)));
      }
    }
  }

  pub fn load_effective(path: &Path) -> ConfigResult<Self> {
    let mut document = Self::load(path)?;
    apply_defaults(&mut document)?;
    Ok(document)
  }

  pub fn validate(&self) -> ConfigResult<()> {
    if self.schema_version != CURRENT_SCHEMA_VERSION {
      return Err(ConfigError::Invalid(format!(
        "unsupported schema_version {} (expected {})",
        self.schema_version, CURRENT_SCHEMA_VERSION
      )));
    }
    for (section, value) in &self.sections {
      if !KNOWN_SECTIONS.contains(&section.as_str()) && section != "extensions" {
        return Err(ConfigError::Invalid(format!("unknown section: {section}")));
      }
      if !value.is_object() {
        return Err(ConfigError::Invalid(format!(
          "section {section} must be a JSON object"
        )));
      }
    }
    validate_appearance(self)?;
    validate_layout(self)?;
    validate_effects(self)?;
    validate_fonts(self)?;
    validate_hyprland(self)?;
    validate_displays(self)?;
    validate_control_panel(self)?;
    validate_default_apps(self)?;
    validate_keyboard_shortcuts(self)?;
    validate_power(self)?;
    validate_audio(self)?;
    validate_session(self)?;
    validate_taskbar(self)?;
    Ok(())
  }

  pub fn get(&self, pointer: &str) -> Option<&Value> {
    if pointer.is_empty() || pointer == "/" {
      return None;
    }
    self.get_ref(pointer)
  }

  fn get_ref(&self, pointer: &str) -> Option<&Value> {
    let mut current: Option<&Value> = None;
    let object = &self.sections;
    let components = pointer
      .trim_start_matches('/')
      .split('/')
      .collect::<Vec<_>>();
    for (index, raw) in components.iter().enumerate() {
      let key = raw.replace("~1", "/").replace("~0", "~");
      if index == 0 {
        current = object.get(&key);
      } else {
        current = child_value(current?, &key);
      }
    }
    current
  }

  pub fn set(&mut self, pointer: &str, value: Value) -> ConfigResult<()> {
    let components = pointer_components(pointer)?;
    if components.len() == 1 {
      if !KNOWN_SECTIONS.contains(&components[0].as_str()) && components[0] != "extensions" {
        return Err(ConfigError::Invalid(format!(
          "unknown section: {}",
          components[0]
        )));
      }
      if !value.is_object() {
        return Err(ConfigError::Invalid(format!(
          "section {} must be an object",
          components[0]
        )));
      }
      self.sections.insert(components[0].clone(), value);
    } else {
      let section = components[0].clone();
      if !KNOWN_SECTIONS.contains(&section.as_str()) && section != "extensions" {
        return Err(ConfigError::Invalid(format!("unknown section: {section}")));
      }
      let root = self
        .sections
        .entry(section)
        .or_insert_with(|| Value::Object(Map::new()));
      set_nested(root, &components[1..], value)?;
    }
    self.validate()
  }

  pub fn unset(&mut self, pointer: &str) -> ConfigResult<()> {
    let components = pointer_components(pointer)?;
    if components.len() == 1 {
      self.sections.remove(&components[0]);
    } else if let Some(section) = self.sections.get_mut(&components[0]) {
      unset_nested(section, &components[1..]);
    }
    self.validate()
  }

  pub fn scoped(&self, scope: ConfigScope) -> Self {
    let sections = scope
      .sections()
      .iter()
      .filter_map(|name| {
        self
          .sections
          .get(*name)
          .map(|value| ((*name).to_owned(), value.clone()))
      })
      .collect();
    Self {
      schema_version: self.schema_version,
      sections,
    }
  }
}

#[derive(Debug, Clone)]
pub struct ConfigStore {
  pub path: PathBuf,
  lock_path: PathBuf,
}

impl ConfigStore {
  pub fn from_environment() -> ConfigResult<Self> {
    // Empty values count as unset, as in SessionPaths. A relative path would create the
    // configuration tree in whatever directory the command runs from, so it is refused.
    let config_home = non_empty_env("ARGVUS_CONFIG_HOME")
      .or_else(|| non_empty_env("XDG_CONFIG_HOME"))
      .or_else(|| non_empty_env("HOME").map(|home| home.join(".config")))
      .ok_or(ConfigError::MissingConfigDirectory)?;
    if !config_home.is_absolute() {
      return Err(ConfigError::MissingConfigDirectory);
    }
    Ok(Self::at(&config_home))
  }

  /// The store under `config_home`, the directory that contains `argvus/`.
  pub fn at(config_home: &Path) -> Self {
    let directory = config_home.join("argvus");
    Self {
      path: directory.join(MODULE_DIRECTORY),
      lock_path: directory.join(LOCK_FILE),
    }
  }

  /// Reads the stored document under the shared lock, so a read never sees a save
  /// that is half renamed. Write paths in this module call the unlocked loader
  /// instead, because they already hold the exclusive lock.
  pub fn load(&self) -> ConfigResult<ConfigDocument> {
    let root = self.config_root()?;
    let _lock = self.shared_lock()?;
    self.load_legacy_or_modules(root)
  }

  pub fn load_effective(&self) -> ConfigResult<ConfigDocument> {
    let root = self.config_root()?;
    let _lock = self.shared_lock()?;
    let mut document = self.load_legacy_or_modules(root)?;
    apply_packaged_defaults(&mut document)?;
    apply_defaults(&mut document)?;
    document.validate()?;
    Ok(document)
  }

  /// The shared lock for a read. Without a lock file no writer has run yet, so the
  /// read goes ahead unlocked; the returned file releases the lock when dropped.
  fn shared_lock(&self) -> ConfigResult<Option<File>> {
    let file = match File::open(&self.lock_path) {
      Ok(file) => file,
      Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
      Err(error) => return Err(error.into()),
    };
    // SAFETY: the descriptor belongs to the file opened above and stays valid
    // until the returned value is dropped.
    let result = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_SH) };
    if result != 0 {
      return Err(io::Error::last_os_error().into());
    }
    Ok(Some(file))
  }

  /// Create or complete the canonical document without replacing explicit
  /// values. This is the only initialization path used by session recovery.
  pub fn ensure(&self) -> ConfigResult<ConfigDocument> {
    let lock = self.open_lock()?;
    lock_file(&lock)?;
    let _lock_guard = LockGuard(&lock);
    let root = self.config_root()?;
    let directory = self.module_directory();
    let was_missing = !directory.exists();
    let mut document = self.load_legacy_or_modules(root)?;
    let before = document.clone();
    let imported = import_legacy_values(&mut document, root)?;
    let moved = migrate_legacy_layout(root)?;
    apply_packaged_defaults(&mut document)?;
    apply_defaults(&mut document)?;
    document.validate()?;
    if document != before || imported || moved || was_missing || !directory.exists() {
      self.save_modules_locked(&document)?;
    }
    Ok(document)
  }

  pub fn save(&self, document: &ConfigDocument) -> ConfigResult<()> {
    document.validate()?;
    let lock = self.open_lock()?;
    lock_file(&lock)?;
    let _lock_guard = LockGuard(&lock);
    self.save_modules_locked(document)
  }

  fn open_lock(&self) -> ConfigResult<File> {
    let directory = self
      .path
      .parent()
      .ok_or(ConfigError::MissingConfigDirectory)?;
    fs::create_dir_all(directory)?;
    if let Some(lock_parent) = self.lock_path.parent() {
      fs::create_dir_all(lock_parent)?;
    }
    Ok(
      OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(&self.lock_path)?,
    )
  }

  fn config_root(&self) -> ConfigResult<&Path> {
    self
      .path
      .parent()
      .ok_or(ConfigError::MissingConfigDirectory)
  }

  fn module_directory(&self) -> PathBuf {
    if self.path.file_name().and_then(|name| name.to_str()) == Some("config.json") {
      self
        .path
        .parent()
        .unwrap_or(&self.path)
        .join(MODULE_DIRECTORY)
    } else {
      self.path.clone()
    }
  }

  fn load_legacy_or_modules(&self, root: &Path) -> ConfigResult<ConfigDocument> {
    let directory = self.module_directory();
    if directory.is_dir() && fs::read_dir(&directory)?.next().is_some() {
      return self.load_modular(false);
    }
    let legacy = root.join("config.json");
    if legacy.is_file() {
      return ConfigDocument::load(&legacy);
    }
    Ok(ConfigDocument::default())
  }

  fn load_modular(&self, apply_default_values: bool) -> ConfigResult<ConfigDocument> {
    let directory = self.module_directory();
    let mut document = ConfigDocument::default();
    if directory.is_dir() {
      for entry in fs::read_dir(&directory)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
          continue;
        }
        let Some(section) = path.file_stem().and_then(|name| name.to_str()) else {
          continue;
        };
        let text = fs::read_to_string(&path)?;
        // Name the file: a damaged section is fixed by hand, and the error says where.
        let value: Value = serde_json::from_str(&text).map_err(|error| {
          ConfigError::Invalid(format!("{} is not valid JSON: {error}", path.display()))
        })?;
        if !value.is_object() {
          return Err(ConfigError::Invalid(format!(
            "section {section} must be a JSON object"
          )));
        }
        document.sections.insert(section.to_owned(), value);
      }
    }
    document.migrate_animation_settings();
    if apply_default_values {
      apply_packaged_defaults(&mut document)?;
      apply_defaults(&mut document)?;
    }
    document.validate()?;
    Ok(document)
  }

  fn save_modules_locked(&self, document: &ConfigDocument) -> ConfigResult<()> {
    let root = self.config_root()?;
    let directory = self.module_directory();
    // The configuration is private: directories are created 0700 and every file is
    // created 0600 (the mode is set when the file is created, never after).
    fs::DirBuilder::new()
      .recursive(true)
      .mode(0o700)
      .create(&directory)?;
    let legacy = root.join("config.json");
    let backup = root.join(BACKUP_FILE);
    if directory.is_dir() && fs::read_dir(&directory)?.next().is_some() {
      let previous = self.load_modular(false)?;
      let contents = format!(
        "{}\n",
        serde_json::to_string_pretty(&previous.sorted_sections())?
      );
      write_backup(&backup, &contents)?;
    }
    if legacy.is_file() {
      let contents = fs::read_to_string(&legacy)?;
      write_backup(&backup, &contents)?;
    }
    // Write every section to its temporary file first, then rename them. A failure
    // while writing leaves the previous configuration untouched, and the window in
    // which readers can see a mix of old and new sections is only the renames.
    let mut staged = Vec::new();
    for (section, value) in &document.sections {
      let path = directory.join(format!("{section}.json"));
      let temporary = path.with_extension(format!("json.tmp-{}", std::process::id()));
      let _ = fs::remove_file(&temporary);
      let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o600)
        .open(&temporary)?;
      file.write_all(serde_json::to_string_pretty(&sorted_value(value))?.as_bytes())?;
      file.write_all(b"\n")?;
      file.sync_all()?;
      staged.push((temporary, path));
    }
    for (temporary, path) in staged {
      fs::rename(temporary, path)?;
    }
    File::open(&directory)?.sync_all()?;
    if directory.is_dir() {
      for entry in fs::read_dir(&directory)? {
        let entry = entry?;
        let path = entry.path();
        let Some(section) = path.file_stem().and_then(|name| name.to_str()) else {
          continue;
        };
        if path.extension().and_then(|extension| extension.to_str()) == Some("json")
          && !document.sections.contains_key(section)
        {
          fs::remove_file(path)?;
        }
      }
    }
    if legacy.is_file() {
      fs::remove_file(legacy)?;
    }
    Ok(())
  }

  /// Writes one pointer. Only that value is stored: defaults are not
  /// materialized, so legacy values can still fill the other gaps later.
  pub fn update(&self, pointer: &str, value: Value) -> ConfigResult<ConfigDocument> {
    self.modify_stored(false, |document| {
      document.set(pointer, value)?;
      Ok(true)
    })
  }

  pub fn unset(&self, pointer: &str) -> ConfigResult<ConfigDocument> {
    self.modify(|document| document.unset(pointer))
  }

  /// Apply a complete JSON patch under one canonical lock. The patch object
  /// maps JSON pointers to their replacement values; `null` is a real JSON
  /// value and is therefore never treated as an implicit deletion.
  pub fn patch(&self, values: &Map<String, Value>) -> ConfigResult<ConfigDocument> {
    self.modify(|document| {
      for (pointer, value) in values {
        document.set(pointer, value.clone())?;
      }
      Ok(())
    })
  }

  /// Atomically apply the theme-owned appearance fields while preserving
  /// explicit user overrides recorded in the canonical document.
  ///
  /// `reset_custom_accent` clears an explicit accent override so the theme
  /// default wins. Selecting an official theme sets it, so a highlight color
  /// chosen earlier never survives a theme change. Projections, session
  /// startup and custom profile applies leave it false and keep the override.
  pub fn apply_theme(
    &self,
    theme: &str,
    accent: Option<&str>,
    gtk_mode: Option<&str>,
    wallpaper: Option<&str>,
    reset_custom_wallpaper: bool,
    reset_custom_accent: bool,
  ) -> ConfigResult<ConfigDocument> {
    self.modify(|document| {
      document.set("/appearance/theme", Value::String(theme.to_owned()))?;
      // Sticky/Float is an independent setting (see `apply_layout_variant`);
      // applying a theme must not change the mode the user already chose, so
      // only the geometry for the *current* variant is reasserted here.
      let current_variant = document
        .get("/layout/variant")
        .and_then(Value::as_str)
        .unwrap_or("sticky")
        .to_owned();
      reset_layout_geometry(document, &current_variant)?;
      if reset_custom_accent {
        document.set("/appearance/accent_custom", Value::Bool(false))?;
      }
      let accent_custom = document
        .get("/appearance/accent_custom")
        .and_then(Value::as_bool)
        .unwrap_or(false);
      if !accent_custom {
        if let Some(accent) = accent {
          document.set("/appearance/accent", Value::String(accent.to_owned()))?;
        }
        document.set("/appearance/accent_custom", Value::Bool(false))?;
      }
      if let Some(mode) = gtk_mode {
        document.set("/appearance/gtk_mode", Value::String(mode.to_owned()))?;
      }
      let wallpaper_custom = document
        .get("/appearance/wallpaper_custom")
        .and_then(Value::as_bool)
        .unwrap_or(false);
      if reset_custom_wallpaper {
        document.set("/appearance/wallpaper_custom", Value::Bool(false))?;
      }
      if (reset_custom_wallpaper || !wallpaper_custom)
        && let Some(wallpaper) = wallpaper
      {
        document.set("/appearance/wallpaper", Value::String(wallpaper.to_owned()))?;
      }
      Ok(())
    })
  }

  /// Set the Sticky/Float layout mode independently of the active theme.
  ///
  /// Unlike `apply_theme`, this is the only place that writes
  /// `/layout/variant`, keeping it decoupled from theme selection: picking
  /// a theme never changes the mode, and picking a mode never changes the
  /// theme. The baseline geometry for the new mode is reasserted the same
  /// way a theme switch does, so Control Panel overrides from the previous
  /// mode do not leak into the new one.
  ///
  /// # Errors
  ///
  /// Returns an error when `variant` is not `"sticky"` or `"float"`, or
  /// when the canonical document cannot be read, validated or saved.
  pub fn apply_layout_variant(&self, variant: &str) -> ConfigResult<ConfigDocument> {
    if variant != "sticky" && variant != "float" {
      return Err(ConfigError::Invalid(
        "layout.variant must be sticky or float".into(),
      ));
    }
    self.modify(|document| {
      document.set("/layout/variant", Value::String(variant.to_owned()))?;
      reset_layout_geometry(document, variant)
    })
  }

  /// Perform a read-modify-write transaction under the same lock used by
  /// `save`. Callers must use this for every canonical mutation so two UI
  /// processes cannot overwrite each other's sections with stale documents.
  pub fn modify<F>(&self, operation: F) -> ConfigResult<ConfigDocument>
  where
    F: FnOnce(&mut ConfigDocument) -> ConfigResult<()>,
  {
    self.modify_if_changed(|document| {
      operation(document)?;
      Ok(true)
    })
  }

  fn modify_if_changed<F>(&self, operation: F) -> ConfigResult<ConfigDocument>
  where
    F: FnOnce(&mut ConfigDocument) -> ConfigResult<bool>,
  {
    self.modify_stored(true, operation)
  }

  /// Runs `operation` on the stored document under the canonical lock and
  /// saves it when the operation reports a change. With `with_defaults`
  /// the document is default-filled first; migrations pass `false` so they
  /// see only what is stored and can import legacy values into the gaps.
  fn modify_stored<F>(&self, with_defaults: bool, operation: F) -> ConfigResult<ConfigDocument>
  where
    F: FnOnce(&mut ConfigDocument) -> ConfigResult<bool>,
  {
    let lock = self.open_lock()?;
    lock_file(&lock)?;
    let _lock_guard = LockGuard(&lock);
    let mut document = self.load_legacy_or_modules(self.config_root()?)?;
    if with_defaults {
      apply_packaged_defaults(&mut document)?;
      apply_defaults(&mut document)?;
    }
    // Compare with what the operation reports, not with a default-filled
    // copy: writing a value equal to its default must still persist it, or
    // a later load (which applies no defaults) would not find it.
    if operation(&mut document)? {
      self.save_modules_locked(&document)?;
    }
    Ok(document)
  }

  pub fn migrate_legacy(&self) -> ConfigResult<ConfigDocument> {
    let root = self.config_root()?;
    let result = self.modify_stored(false, |document| import_legacy_values(document, root));
    migrate_legacy_layout(root)?;
    result
  }
}

/// A theme mode owns the baseline geometry for windows and the taskbar. Keep
/// those canonical values synchronized with the variant so the next
/// projection cannot restore the previous mode's derived `.spaces`/`.borders`.
fn reset_layout_geometry(document: &mut ConfigDocument, variant: &str) -> ConfigResult<()> {
  // Float values match the documented Float mode: internal gap 10, outer gap
  // 18, and the same 18 px taskbar margin.
  let (gaps_in, gaps_out, rounded, rounding, border_size, margins) = if variant == "float" {
    (10, 18, true, 4, 1, [18, 18, 18, 18])
  } else {
    (2, 0, false, 0, 1, [0, 0, 0, 0])
  };
  let values = [
    ("/layout/window/gaps_in", Value::from(gaps_in)),
    ("/layout/window/gaps_out_top", Value::from(gaps_out)),
    ("/layout/window/gaps_out_left", Value::from(gaps_out)),
    ("/layout/window/gaps_out_right", Value::from(gaps_out)),
    ("/layout/window/gaps_out_bottom", Value::from(gaps_out)),
    ("/layout/window/rounded", Value::Bool(rounded)),
    ("/layout/window/rounding", Value::from(rounding)),
    ("/layout/window/border_size", Value::from(border_size)),
    ("/layout/taskbar/position", Value::String("top".into())),
    ("/layout/taskbar/margin_top", Value::from(margins[0])),
    ("/layout/taskbar/margin_left", Value::from(margins[1])),
    ("/layout/taskbar/margin_right", Value::from(margins[2])),
    ("/layout/taskbar/margin_bottom", Value::from(margins[3])),
  ];
  for (pointer, value) in values {
    document.set(pointer, value)?;
  }
  Ok(())
}

fn theme_layout_variant(theme: &str) -> &'static str {
  if theme.ends_with("-float") {
    "float"
  } else {
    "sticky"
  }
}

fn import_legacy_values(document: &mut ConfigDocument, root: &Path) -> ConfigResult<bool> {
  let mut changed = false;
  changed |= import_line(
    document,
    "/appearance/theme",
    resolved(root, ".active-theme"),
  )?;
  // Legacy installs encoded Sticky/Float as a `-float` suffix on the theme
  // name itself (e.g. `.active-theme` containing `argvus-dark-float`).
  // Strip it into the now-independent `/layout/variant` so the user's
  // existing mode survives the migration instead of being silently lost.
  changed |= migrate_theme_suffix_to_variant(document)?;
  let imported_accent = import_line(
    document,
    "/appearance/accent",
    resolved(root, ".accent-color"),
  )?;
  changed |= imported_accent;
  if imported_accent && resolved(root, ".accent-custom").is_some_and(|path| path.is_file()) {
    document.set("/appearance/accent_custom", Value::Bool(true))?;
    changed = true;
  }
  changed |= import_line(
    document,
    "/appearance/gtk_mode",
    resolved(root, ".gtk-mode"),
  )?;
  changed |= import_line(
    document,
    "/appearance/wallpaper",
    resolved(root, ".wallpaper-custom"),
  )?;
  changed |= import_layout_state(document, root)?;
  changed |= import_fonts(document, resolved(root, "fonts.conf"))?;
  changed |= import_hyprland(document, root)?;
  changed |= import_displays(document, root)?;
  changed |= import_defaults(document, resolved(root, "defaults.json"))?;
  changed |= import_keyboard_shortcuts(document, root)?;
  changed |= import_language(document, resolved(root, "language"))?;
  changed |= import_power(document, root)?;
  changed |= import_removable_devices(document, root)?;
  Ok(changed)
}

fn migrate_theme_suffix_to_variant(document: &mut ConfigDocument) -> ConfigResult<bool> {
  let Some(theme) = document.get("/appearance/theme").and_then(Value::as_str) else {
    return Ok(false);
  };
  let Some(family) = theme.strip_suffix("-float") else {
    return Ok(false);
  };
  document.set("/appearance/theme", Value::String(family.to_owned()))?;
  if document.get("/layout/variant").is_none() {
    document.set("/layout/variant", Value::String("float".into()))?;
  }
  Ok(true)
}

fn import_layout_state(document: &mut ConfigDocument, root: &Path) -> ConfigResult<bool> {
  let mut changed = false;
  let spaces = resolved(root, ".spaces")
    .map(|path| parse_legacy_pairs(&path))
    .unwrap_or_default();
  let borders = resolved(root, ".borders")
    .map(|path| parse_legacy_pairs(&path))
    .unwrap_or_default();
  for (key, value) in spaces {
    let pointer = match key.as_str() {
      "gaps_in" => "/layout/window/gaps_in",
      "gaps_out" => "/layout/window/gaps_out",
      "gaps_out_top" => "/layout/window/gaps_out_top",
      "gaps_out_left" => "/layout/window/gaps_out_left",
      "gaps_out_right" => "/layout/window/gaps_out_right",
      "gaps_out_bottom" => "/layout/window/gaps_out_bottom",
      "waybar_pos" => "/layout/taskbar/position",
      "waybar_top" => "/layout/taskbar/margin_top",
      "waybar_left" => "/layout/taskbar/margin_left",
      "waybar_right" => "/layout/taskbar/margin_right",
      "waybar_bottom" => "/layout/taskbar/margin_bottom",
      _ => continue,
    };
    let parsed = if pointer.ends_with("position") {
      Value::String(value)
    } else {
      Value::Number(
        value
          .parse::<u64>()
          .map_err(|_| ConfigError::Invalid(format!("invalid legacy layout value for {key}")))?
          .into(),
      )
    };
    if document.get(pointer).is_none() {
      document.set(pointer, parsed)?;
      changed = true;
    }
  }
  for (key, value) in borders {
    let pointer = match key.as_str() {
      "rounded" => "/layout/window/rounded",
      "rounding" => "/layout/window/rounding",
      "thickness" => "/layout/window/border_size",
      _ => continue,
    };
    let parsed = if pointer.ends_with("rounded") {
      Value::Bool(value == "1" || value.eq_ignore_ascii_case("true"))
    } else {
      Value::Number(
        value
          .parse::<u64>()
          .map_err(|_| ConfigError::Invalid(format!("invalid legacy border value for {key}")))?
          .into(),
      )
    };
    if document.get(pointer).is_none() {
      document.set(pointer, parsed)?;
      changed = true;
    }
  }
  Ok(changed)
}

fn parse_legacy_pairs(path: &Path) -> Vec<(String, String)> {
  fs::read_to_string(path)
    .unwrap_or_default()
    .lines()
    .filter_map(|line| {
      let (key, value) = line.split_once('=')?;
      Some((key.trim().to_owned(), value.trim().to_owned()))
    })
    .collect()
}

fn default_schema_version() -> u32 {
  CURRENT_SCHEMA_VERSION
}

/// Resolves one JSON Pointer segment, using numeric segments as array indexes.
fn child_value<'a>(value: &'a Value, key: &str) -> Option<&'a Value> {
  match value {
    Value::Array(items) => key.parse::<usize>().ok().and_then(|index| items.get(index)),
    other => other.get(key),
  }
}

fn pointer_components(pointer: &str) -> ConfigResult<Vec<String>> {
  if !pointer.starts_with('/') {
    return Err(ConfigError::Invalid(format!(
      "JSON pointer must start with '/': {pointer}"
    )));
  }
  let components = pointer
    .trim_start_matches('/')
    .split('/')
    .filter(|component| !component.is_empty())
    .map(|component| component.replace("~1", "/").replace("~0", "~"))
    .collect::<Vec<_>>();
  if components.is_empty() {
    return Err(ConfigError::Invalid("empty JSON pointer".into()));
  }
  Ok(components)
}

fn set_nested(root: &mut Value, components: &[String], value: Value) -> ConfigResult<()> {
  let mut current = root;
  for component in &components[..components.len().saturating_sub(1)] {
    let object = current
      .as_object_mut()
      .ok_or_else(|| ConfigError::Invalid("intermediate value must be an object".into()))?;
    current = object
      .entry(component)
      .or_insert_with(|| Value::Object(Map::new()));
  }
  current
    .as_object_mut()
    .ok_or_else(|| ConfigError::Invalid("target parent must be an object".into()))?
    .insert(components.last().cloned().unwrap_or_default(), value);
  Ok(())
}

fn unset_nested(root: &mut Value, components: &[String]) {
  if components.is_empty() {
    return;
  }
  if components.len() == 1 {
    if let Some(object) = root.as_object_mut() {
      object.remove(&components[0]);
    }
    return;
  }
  if let Some(next) = root.get_mut(&components[0]) {
    unset_nested(next, &components[1..]);
  }
}

/// Stored objects that `apply_defaults` descends into. A value at one of these
/// paths that is not an object cannot take defaults; it is reported as invalid
/// configuration instead of stopping the command.
const DEFAULTED_OBJECTS: &[&[&str]] = &[
  &["appearance"],
  &["layout"],
  &["layout", "taskbar"],
  &["layout", "window"],
  &["effects"],
  &["taskbar"],
  &["taskbar", "icons"],
  &["taskbar", "icons", "utilities"],
  &["taskbar", "date"],
  &["taskbar", "time"],
  &["taskbar", "shape"],
  &["default_apps"],
  &["keyboard_shortcuts"],
  &["hyprland"],
  &["hyprland", "input"],
  &["hyprland", "input", "mouse"],
  &["hyprland", "input", "touchpad"],
  &["power"],
  &["audio"],
];

fn check_defaulted_objects(document: &ConfigDocument) -> ConfigResult<()> {
  for path in DEFAULTED_OBJECTS {
    let mut current = document.sections.get(path[0]);
    for key in &path[1..] {
      current = current.and_then(|value| value.get(key));
    }
    if current.is_some_and(|value| !value.is_object()) {
      return Err(ConfigError::Invalid(format!(
        "{} must be an object",
        path.join(".")
      )));
    }
  }
  Ok(())
}

fn apply_defaults(document: &mut ConfigDocument) -> ConfigResult<()> {
  check_defaulted_objects(document)?;
  let appearance = document
    .sections
    .entry("appearance".into())
    .or_insert_with(|| Value::Object(Map::new()));
  let object = appearance.as_object_mut().expect("appearance is object");
  object
    .entry("accent")
    .or_insert_with(|| Value::String("#3590BD".into()));
  object
    .entry("accent_custom")
    .or_insert_with(|| Value::Bool(false));
  object
    .entry("theme")
    .or_insert_with(|| Value::String("argvus-dark".into()));
  object
    .entry("wallpaper")
    .or_insert_with(|| Value::String("/usr/share/backgrounds/argvus/argvus-dark.jxl".into()));
  object
    .entry("wallpaper_custom")
    .or_insert_with(|| Value::Bool(false));
  object
    .entry("gtk_mode")
    .or_insert_with(|| Value::String("dark".into()));
  // Empty means "use the ARGVUS default icon theme" (see `argvus-icons`).
  object
    .entry("icon_theme")
    .or_insert_with(|| Value::String(String::new()));
  let theme = object
    .get("theme")
    .and_then(Value::as_str)
    .unwrap_or("argvus-dark")
    .to_owned();

  let layout = document
    .sections
    .entry("layout".into())
    .or_insert_with(|| Value::Object(Map::new()));
  let layout = layout.as_object_mut().expect("layout is object");
  // Sticky/Float is independent of the theme (see `apply_layout_variant`);
  // `theme` here only matters for the legacy `-float`-suffixed installs
  // that `migrate_theme_suffix_to_variant` has not rewritten yet.
  layout
    .entry("variant")
    .or_insert_with(|| Value::String(theme_layout_variant(&theme).into()));
  {
    let taskbar = layout
      .entry("taskbar")
      .or_insert_with(|| Value::Object(Map::new()))
      .as_object_mut()
      .expect("layout.taskbar is object");
    for (key, value) in [
      ("margin_bottom", Value::from(0)),
      ("margin_left", Value::from(0)),
      ("margin_right", Value::from(0)),
      ("margin_top", Value::from(0)),
      ("position", Value::String("top".into())),
    ] {
      taskbar.entry(key).or_insert(value);
    }
  }
  {
    let window = layout
      .entry("window")
      .or_insert_with(|| Value::Object(Map::new()))
      .as_object_mut()
      .expect("layout.window is object");
    for (key, value) in [
      ("border_size", Value::from(1)),
      ("gaps_in", Value::from(2)),
      ("gaps_out_bottom", Value::from(0)),
      ("gaps_out_left", Value::from(0)),
      ("gaps_out_right", Value::from(0)),
      ("gaps_out_top", Value::from(0)),
      ("rounded", Value::Bool(false)),
      ("rounding", Value::from(0)),
    ] {
      window.entry(key).or_insert(value);
    }
  }

  let effects = document
    .sections
    .entry("effects".into())
    .or_insert_with(|| Value::Object(Map::new()));
  let effects = effects.as_object_mut().expect("effects is object");
  effects
    .entry("animations")
    .or_insert_with(|| Value::Bool(true));
  effects
    .entry("widget_telemetry_enabled")
    .or_insert_with(|| Value::Bool(true));
  effects
    .entry("blur_global_enabled")
    .or_insert_with(|| Value::Bool(true));
  for (surface, transparency) in [
    ("taskbar", 50),
    ("launchers", 15),
    ("widget-telemetry", 50),
    ("control-panel", 50),
    ("terminal", 50),
    ("control-center", 15),
  ] {
    effects
      .entry(format!("transparency_{surface}_enabled"))
      .or_insert_with(|| Value::Bool(true));
    effects
      .entry(format!("transparency_{surface}_value"))
      .or_insert_with(|| Value::Number(transparency.into()));
  }
  for (key, value) in BLUR_SETTINGS_DEFAULTS {
    effects.entry(key).or_insert_with(|| {
      // Whole defaults are written as integers, like effects.json, so the
      // integer fields (size, passes) read back without a fractional part.
      if value.fract() == 0.0 {
        Value::from(value as i64)
      } else {
        Value::from(value)
      }
    });
  }
  effects
    .entry("transparency_control-center_enabled")
    .or_insert_with(|| Value::Bool(true));
  effects
    .entry("transparency_control-center_value")
    .or_insert_with(|| Value::Number(50.into()));
  effects
    .entry("blur_control-center_enabled")
    .or_insert_with(|| Value::Bool(true));
  // Per-surface blur switches were never read by the compositor or the
  // surfaces; blur follows the global switch. Drop them from existing profiles.
  for surface in [
    "taskbar",
    "launchers",
    "widget-telemetry",
    "control-panel",
    "terminal",
  ] {
    effects.remove(&format!("blur_{surface}_enabled"));
  }

  let taskbar = document
    .sections
    .entry("taskbar".into())
    .or_insert_with(|| Value::Object(Map::new()));
  let taskbar = taskbar.as_object_mut().expect("taskbar is object");
  let icons = taskbar
    .entry("icons")
    .or_insert_with(|| Value::Object(Map::new()));
  let icons = icons.as_object_mut().expect("taskbar.icons is object");
  icons
    .entry("audio_player_enabled")
    .or_insert_with(|| Value::Bool(true));
  icons
    .entry("launcher_enabled")
    .or_insert_with(|| Value::Bool(false));
  icons
    .entry("launcher_custom_icon_path")
    .or_insert(Value::Null);
  for widget in [
    "network",
    "power_profile",
    "keyboard_layout",
    "memory",
    "cpu",
    "cpu_temperature",
    "gpu_temperature",
  ] {
    icons
      .entry(format!("{widget}_enabled"))
      .or_insert_with(|| Value::Bool(true));
  }
  let utilities = icons
    .entry("utilities")
    .or_insert_with(|| Value::Object(Map::new()));
  let utilities = utilities
    .as_object_mut()
    .expect("taskbar.icons.utilities is object");
  utilities
    .entry("expanded")
    .or_insert_with(|| Value::Bool(true));
  let date = taskbar
    .entry("date")
    .or_insert_with(|| Value::Object(Map::new()));
  let date = date.as_object_mut().expect("taskbar.date is object");
  date
    .entry("format")
    .or_insert_with(|| Value::String("weekday_day_month".into()));
  let time = taskbar
    .entry("time")
    .or_insert_with(|| Value::Object(Map::new()));
  let time = time.as_object_mut().expect("taskbar.time is object");
  time
    .entry("seconds_enabled")
    .or_insert_with(|| Value::Bool(false));
  time
    .entry("format")
    .or_insert_with(|| Value::String("24h".into()));
  // An unset height or border follows the packaged taskbar and the theme class.
  let shape = taskbar
    .entry("shape")
    .or_insert_with(|| Value::Object(Map::new()));
  let shape = shape.as_object_mut().expect("taskbar.shape is object");
  shape.entry("height").or_insert(Value::Null);
  shape.entry("border_width").or_insert(Value::Null);

  let default_apps = document
    .sections
    .entry("default_apps".into())
    .or_insert_with(|| Value::Object(Map::new()));
  let default_apps = default_apps
    .as_object_mut()
    .expect("default_apps is object");
  for (category, application) in [
    ("terminal", "argvus-terminal"),
    ("file_manager", "spf"),
    ("text_editor", "mousepad"),
    ("terminal_editor", "vim"),
    ("browser", "firefox"),
    ("image_viewer", "imv"),
    ("pdf_viewer", "zathura"),
    ("video_player", "mpv"),
    ("audio_player", "audacious"),
    ("archive", "xarchiver"),
    ("launcher", "rofi"),
  ] {
    default_apps
      .entry(category)
      .or_insert_with(|| Value::String(application.into()));
  }

  let keyboard_shortcuts = document
    .sections
    .entry("keyboard_shortcuts".into())
    .or_insert_with(|| Value::Object(Map::new()));
  let keyboard_shortcuts = keyboard_shortcuts
    .as_object_mut()
    .expect("keyboard_shortcuts is object");
  for (config_key, shortcut) in manifest_shortcut_defaults() {
    keyboard_shortcuts.entry(config_key).or_insert(shortcut);
  }

  let hyprland = document
    .sections
    .entry("hyprland".into())
    .or_insert_with(|| Value::Object(Map::new()));
  let hyprland = hyprland.as_object_mut().expect("hyprland is object");
  let input = hyprland
    .entry("input")
    .or_insert_with(|| Value::Object(Map::new()));
  let input = input.as_object_mut().expect("hyprland.input is object");
  let mouse = input
    .entry("mouse")
    .or_insert_with(|| Value::Object(Map::new()));
  let mouse = mouse
    .as_object_mut()
    .expect("hyprland.input.mouse is object");
  for (key, value) in [
    ("accel_profile", Value::String("flat".into())),
    ("left_handed", Value::Bool(false)),
    ("natural_scroll", Value::Bool(false)),
    ("scroll_factor", Value::from(1.0)),
    ("sensitivity", Value::from(0.0)),
  ] {
    mouse.entry(key).or_insert(value);
  }
  let touchpad = input
    .entry("touchpad")
    .or_insert_with(|| Value::Object(Map::new()));
  let touchpad = touchpad
    .as_object_mut()
    .expect("hyprland.input.touchpad is object");
  for (key, value) in [
    ("disable_while_typing", Value::Bool(true)),
    ("natural_scroll", Value::Bool(false)),
    ("tap_and_drag", Value::Bool(true)),
    ("tap_to_click", Value::Bool(true)),
    ("two_finger_right_click", Value::Bool(false)),
  ] {
    touchpad.entry(key).or_insert(value);
  }

  let power = document
    .sections
    .entry("power".into())
    .or_insert_with(|| Value::Object(Map::new()));
  let power = power.as_object_mut().expect("power is object");
  power.entry("lock_minutes").or_insert(30.into());
  power.entry("screen_off_minutes").or_insert(0.into());
  power
    .entry("keep_awake")
    .or_insert_with(|| Value::Bool(false));

  let audio = document
    .sections
    .entry("audio".into())
    .or_insert_with(|| Value::Object(Map::new()));
  let audio = audio.as_object_mut().expect("audio is object");
  audio.entry("output_volume").or_insert(50.into());
  audio
    .entry("output_muted")
    .or_insert_with(|| Value::Bool(false));
  Ok(())
}

fn apply_packaged_defaults(document: &mut ConfigDocument) -> ConfigResult<()> {
  let system_config = env::var_os("ARGVUS_SYSTEM_CONFIG")
    .map(PathBuf::from)
    .unwrap_or_else(|| PathBuf::from("/usr/share/argvus"));
  let defaults_root = system_config.join(MODULE_DIRECTORY);
  for section in KNOWN_SECTIONS {
    if document.sections.contains_key(section) {
      continue;
    }
    let path = defaults_root.join(format!("{section}.json"));
    if !path.is_file() {
      continue;
    }
    let value: Value = serde_json::from_str(&fs::read_to_string(path)?)?;
    if !value.is_object() {
      return Err(ConfigError::Invalid(format!(
        "section {section} must be a JSON object"
      )));
    }
    document.sections.insert(section.to_owned(), value);
  }
  Ok(())
}

fn validate_appearance(document: &ConfigDocument) -> ConfigResult<()> {
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

fn validate_layout(document: &ConfigDocument) -> ConfigResult<()> {
  validate_numeric_section(document, "layout", 0, 100)?;
  if let Some(variant) = document.get("/layout/variant").and_then(Value::as_str)
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
const BLUR_SETTINGS: [(&str, f64, f64, bool); 7] = [
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
const BLUR_SETTINGS_DEFAULTS: [(&str, f64); 7] = [
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
const WINDOW_ANIMATION_STYLES: [&str; 3] = ["slide", "popin", "gnomed"];

/// Styles that render on the layer leaves in Hyprland 0.56.2. Slide variants
/// other than `slide` are drawn as a plain slide, so they are not offered.
const LAYER_ANIMATION_STYLES: [&str; 3] = ["slide", "popin", "fade"];

/// Keys of the `layers` and `switching` animation leaves that were removed. Hyprland
/// reads neither leaf directly (only their `In`/`Out` children), so their values
/// had no effect.
const RETIRED_ANIMATION_KEYS: [&str; 6] = [
  "animation_layers_enabled",
  "animation_layers_speed",
  "animation_layers_curve",
  "animation_switching_style",
  "animation_switching_speed",
  "animation_switching_curve",
];

/// Styles Hyprland 0.56.2 accepts for the workspace leaves. `popin` and
/// `gnomed` are refused there by the compositor.
const WORKSPACE_ANIMATION_STYLES: [&str; 5] =
  ["slide", "fade", "slidefade", "slidevert", "slidefadevert"];

/// Styles accepted by `effects.animation_<leaf>_style`, which depend on the leaf.
fn animation_styles(key: &str) -> &'static [&'static str] {
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
const ANIMATION_CURVES: [&str; 5] = ["default", "bezier", "spring", "linear", "ease_out"];

fn validate_effects(document: &ConfigDocument) -> ConfigResult<()> {
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
fn validate_animation_setting(key: &str, value: &Value) -> ConfigResult<()> {
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

fn validate_blur_setting(
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

fn validate_taskbar(document: &ConfigDocument) -> ConfigResult<()> {
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
    document.get("/taskbar/shape/height"),
    "height",
    &waybar::TASKBAR_HEIGHTS,
  )?;
  validate_taskbar_shape(
    document.get("/taskbar/shape/border_width"),
    "border_width",
    &waybar::TASKBAR_BORDER_WIDTHS,
  )
}

/// A taskbar shape value is `null` (follow the packaged taskbar and the theme)
/// or one of the pixel sizes the Control Center offers.
fn validate_taskbar_shape(value: Option<&Value>, key: &str, offered: &[u32]) -> ConfigResult<()> {
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

fn validate_fonts(document: &ConfigDocument) -> ConfigResult<()> {
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

fn validate_hyprland(document: &ConfigDocument) -> ConfigResult<()> {
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

fn validate_displays(document: &ConfigDocument) -> ConfigResult<()> {
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

fn validate_default_apps(document: &ConfigDocument) -> ConfigResult<()> {
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

fn validate_keyboard_shortcuts(document: &ConfigDocument) -> ConfigResult<()> {
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

fn validate_power(document: &ConfigDocument) -> ConfigResult<()> {
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

fn validate_audio(document: &ConfigDocument) -> ConfigResult<()> {
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

fn validate_session(document: &ConfigDocument) -> ConfigResult<()> {
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

fn validate_numeric_section(
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

fn validate_control_panel(document: &ConfigDocument) -> ConfigResult<()> {
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

fn import_line(
  document: &mut ConfigDocument,
  pointer: &str,
  path: Option<PathBuf>,
) -> ConfigResult<bool> {
  let Some(path) = path else {
    return Ok(false);
  };
  if document.get(pointer).is_some() || !path.is_file() {
    return Ok(false);
  }
  let value = fs::read_to_string(&path)?
    .lines()
    .next()
    .unwrap_or_default()
    .trim()
    .to_owned();
  if value.is_empty() {
    return Ok(false);
  }
  document.set(pointer, Value::String(value))?;
  Ok(true)
}

fn import_fonts(document: &mut ConfigDocument, path: Option<PathBuf>) -> ConfigResult<bool> {
  let Some(path) = path else {
    return Ok(false);
  };
  if !path.is_file() {
    return Ok(false);
  }
  let legacy = fs::read_to_string(&path)?;
  let state = legacy
    .lines()
    .filter_map(|line| line.trim().split_once('='))
    .map(|(key, value)| (key.trim(), value.trim()))
    .collect::<BTreeMap<_, _>>();
  let mut changed = false;

  if document.get("/fonts/legacy").is_none() {
    document.set("/fonts/legacy", Value::String(legacy.clone()))?;
    changed = true;
  }

  if document.get("/fonts/targets").is_none() {
    let mut targets = Map::new();
    for target in [
      "taskbar",
      "sysinfo",
      "control_panel",
      "system",
      "apps",
      "terminal",
      "browser",
    ] {
      let mut selection = Map::new();
      let family_key = format!("{target}_family");
      if let Some(value) = state.get(family_key.as_str()) {
        selection.insert("family".into(), Value::String((*value).into()));
      }
      let style_key = format!("{target}_style");
      if let Some(value) = state.get(style_key.as_str()) {
        selection.insert("style".into(), Value::String((*value).into()));
      }
      let size_key = format!("{target}_size");
      if let Some(value) = state.get(size_key.as_str())
        && let Ok(value) = (*value).parse::<u16>()
      {
        selection.insert("size".into(), value.into());
      }
      if !selection.is_empty() {
        targets.insert(target.into(), Value::Object(selection));
      }
    }
    if !targets.is_empty() {
      document.set("/fonts/targets", Value::Object(targets))?;
      changed = true;
    }
  }

  if document.get("/fonts/rendering").is_none() {
    let mut rendering = Map::new();
    if let Some(value) = state.get("antialias") {
      rendering.insert("antialias".into(), Value::Bool(*value == "true"));
    }
    for key in ["hinting", "subpixel"] {
      if let Some(value) = state.get(key) {
        rendering.insert(key.into(), Value::String((*value).into()));
      }
    }
    if let Some(value) = state.get("custom_dpi") {
      rendering.insert("custom_dpi".into(), Value::Bool(*value == "true"));
    }
    if let Some(value) = state.get("dpi")
      && let Ok(value) = (*value).parse::<u16>()
    {
      rendering.insert("dpi".into(), value.into());
    }
    if !rendering.is_empty() {
      document.set("/fonts/rendering", Value::Object(rendering))?;
      changed = true;
    }
  }

  Ok(changed)
}

fn import_defaults(document: &mut ConfigDocument, path: Option<PathBuf>) -> ConfigResult<bool> {
  if document.get("/default_apps").is_some() {
    return Ok(false);
  }
  let Some(path) = path else {
    return Ok(false);
  };
  let Ok(contents) = fs::read_to_string(&path) else {
    return Ok(false);
  };
  let value: Value = serde_json::from_str(&contents).map_err(ConfigError::InvalidJson)?;
  let Some(object) = value.as_object() else {
    return Ok(false);
  };
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
  let migrated = object
    .iter()
    .filter(|(key, value)| CATEGORIES.contains(&key.as_str()) && value.is_string())
    .map(|(key, value)| (key.clone(), value.clone()))
    .collect();
  document.set("/default_apps", Value::Object(migrated))?;
  Ok(true)
}

fn import_keyboard_shortcuts(document: &mut ConfigDocument, root: &Path) -> ConfigResult<bool> {
  if document.get("/keyboard_shortcuts").is_some() {
    return Ok(false);
  }
  let old_value = if let Some(value) = document.get("/hyprland/keybindings") {
    Some(value.clone())
  } else if let Some(path) = resolved(root, "keybindings.toml") {
    parse_keybindings_toml(&path)?
  } else {
    None
  };
  let Some(value) = old_value else {
    return Ok(false);
  };
  let shortcuts = legacy_keybindings_to_shortcuts(&value);
  document.set("/keyboard_shortcuts", shortcuts)?;
  document.unset("/hyprland/keybindings")?;
  Ok(true)
}

fn migrate_legacy_sections(document: &mut ConfigDocument) -> ConfigResult<()> {
  if document.get("/default_apps").is_none()
    && let Some(value) = document.sections.remove("defaults")
  {
    document.set("/default_apps", value)?;
  } else {
    document.sections.remove("defaults");
  }
  if document.get("/keyboard_shortcuts").is_none()
    && let Some(value) = document
      .sections
      .get_mut("hyprland")
      .and_then(Value::as_object_mut)
      .and_then(|object| object.remove("keybindings"))
  {
    document.set(
      "/keyboard_shortcuts",
      legacy_keybindings_to_shortcuts(&value),
    )?;
  }
  if let Some(hyprland) = document.sections.get_mut("hyprland") {
    hyprland
      .as_object_mut()
      .expect("hyprland is object")
      .remove("keybindings");
  }
  Ok(())
}

fn legacy_keybindings_to_shortcuts(value: &Value) -> Value {
  let mut shortcuts = Map::new();
  if let Some(entries) = value.as_object() {
    for (id, binding) in entries {
      let config_key = config_key_for_binding(id);
      let shortcut = binding
        .get("enabled")
        .and_then(Value::as_bool)
        .filter(|enabled| !enabled)
        .map(|_| Value::Null)
        .or_else(|| binding.get("keys").cloned())
        .unwrap_or(Value::Null);
      shortcuts.insert(config_key, shortcut);
    }
  }
  Value::Object(shortcuts)
}

pub(crate) fn config_key_for_binding(id: &str) -> String {
  match id {
    "window.close" => "close_window",
    "window.drag_mouse" => "drag_window__floating_window_only",
    "window.maximize" => "maximize_window__toggle",
    "app.browser" => "open_browser",
    "system.about" => "open_about",
    "appearance.theme" => "open_theme_selector",
    _ => return id.replace(['.', '-'], "_"),
  }
  .into()
}

fn manifest_shortcut_defaults() -> BTreeMap<String, Value> {
  let path = env::var_os("ARGVUS_SYSTEM_CONFIG")
    .map(PathBuf::from)
    .unwrap_or_else(|| PathBuf::from("/usr/share/argvus"))
    .join("hyprland/keybindings.json");
  let Ok(contents) = fs::read_to_string(path) else {
    return BTreeMap::new();
  };
  let Ok(manifest) = serde_json::from_str::<Value>(&contents) else {
    return BTreeMap::new();
  };
  let mut defaults = BTreeMap::new();
  let Some(bindings) = manifest.get("bindings").and_then(Value::as_array) else {
    return defaults;
  };
  for binding in bindings {
    let Some(id) = binding.get("id").and_then(Value::as_str) else {
      continue;
    };
    let key = binding
      .get("config_key")
      .and_then(Value::as_str)
      .map(str::to_owned)
      .unwrap_or_else(|| config_key_for_binding(id));
    if let Some(shortcut) = binding.get("keys") {
      defaults.insert(key, shortcut.clone());
    }
  }
  defaults
}

/// Destination of a pre-`data/` root entry, relative to the configuration root.
///
/// Every entry other than `config/` and `data/` itself belongs under `data/`.
/// A few names already had a semantic home from an earlier migration and keep
/// it; everything else follows the uniform `data/<name>` rule so a new
/// component directory never has to be registered here.
fn data_destination(name: &str) -> PathBuf {
  match name {
    ".config.lock" => PathBuf::from("data/internal/config.lock"),
    "config.json.bak" => PathBuf::from("data/backups/config.json.bak"),
    "defaults.json" => PathBuf::from("data/control-center/defaults.json"),
    "keybindings.toml" => PathBuf::from("data/hypr/keybindings.toml"),
    "input.toml" => PathBuf::from("data/hypr/input.toml"),
    ".idle-timeout" | ".keep-awake" => PathBuf::from(format!("data/power/{name}")),
    "fonts.conf" => PathBuf::from("data/generated/fonts.conf"),
    "generated" => PathBuf::from("data/generated"),
    "control-center" => PathBuf::from("data/control-center"),
    // The legacy root `terminal/` directory already holds one subdirectory
    // per terminal, so it maps onto `data/terminal/` itself. A standalone
    // root `kitty/` directory belongs beside the others.
    "terminal" => PathBuf::from("data/terminal"),
    "kitty" => PathBuf::from("data/terminal/kitty"),
    _ => PathBuf::from(format!("data/{name}")),
  }
}

/// Resolve a migrated entry for reading, accepting the pre-`data/` root layout.
///
/// Migration runs after the legacy import, so a profile that has not been
/// migrated yet still has its markers at the root. Prefer the managed location
/// and fall back to the root so a single import pass works on both layouts.
fn resolved(root: &Path, name: &str) -> Option<PathBuf> {
  let managed = root.join(data_destination(name));
  if managed.exists() {
    return Some(managed);
  }
  let legacy = root.join(name);
  if legacy.exists() {
    return Some(legacy);
  }
  None
}

fn migrate_legacy_layout(root: &Path) -> ConfigResult<bool> {
  let data = root.join("data");
  fs::create_dir_all(data.join("internal"))?;
  fs::create_dir_all(data.join("backups"))?;
  let mut moved = false;
  let mut entries = fs::read_dir(root)?.collect::<Result<Vec<_>, io::Error>>()?;
  // `generated` and `terminal` are parents of other migrated components, so
  // they must be moved before their children; otherwise the child would find
  // its destination already present and be diverted to `data/legacy`.
  entries.sort_by_key(|entry| match entry.file_name().to_string_lossy().as_ref() {
    "generated" => 0,
    "terminal" => 1,
    "kitty" => 2,
    _ => 3,
  });
  for entry in entries {
    let source = entry.path();
    let name = entry.file_name();
    if name == "config.json" || name == MODULE_DIRECTORY || name == "data" {
      continue;
    }
    let name = name.to_string_lossy();
    let relative = data_destination(name.as_ref());
    let destination = root.join(relative);
    if matches!(
      name.as_ref(),
      "generated" | "control-center" | "terminal" | "kitty"
    ) && source.is_dir()
    {
      merge_legacy_tree(&source, &destination, &data.join("legacy"))?;
    } else if destination.exists() {
      let fallback = root.join("data/legacy").join(name.as_ref());
      if fallback.exists() {
        continue;
      }
      fs::create_dir_all(fallback.parent().expect("legacy parent"))?;
      fs::rename(&source, fallback)?;
    } else {
      fs::create_dir_all(destination.parent().expect("migration parent"))?;
      fs::rename(&source, destination)?;
    }
    moved = true;
  }
  Ok(moved)
}

/// Merge an old component directory into its new data-owned location.
///
/// New files win on collision because they are the current layout. Conflicting
/// legacy files are retained under data/legacy instead of being discarded.
fn merge_legacy_tree(source: &Path, destination: &Path, legacy_root: &Path) -> ConfigResult<bool> {
  fs::create_dir_all(destination)?;
  let component_name = source
    .file_name()
    .ok_or_else(|| ConfigError::Invalid("legacy component has no name".into()))?;
  let component_legacy_root = legacy_root.join(component_name);
  let mut changed = false;

  for entry in fs::read_dir(source)?.collect::<Result<Vec<_>, io::Error>>()? {
    let source_entry = entry.path();
    let entry_name = entry.file_name();
    let destination_entry = destination.join(&entry_name);

    if destination_entry.is_dir() && source_entry.is_dir() {
      changed |= merge_legacy_tree(&source_entry, &destination_entry, &component_legacy_root)?;
      continue;
    }

    if destination_entry.exists() {
      fs::create_dir_all(&component_legacy_root)?;
      let mut legacy_entry = component_legacy_root.join(&entry_name);
      let mut suffix = 1u32;
      while legacy_entry.exists() {
        legacy_entry =
          component_legacy_root.join(format!("{}.legacy-{suffix}", entry_name.to_string_lossy()));
        suffix += 1;
      }
      fs::rename(&source_entry, legacy_entry)?;
    } else {
      fs::rename(&source_entry, destination_entry)?;
    }
    changed = true;
  }

  if fs::read_dir(source)?.next().is_none() {
    fs::remove_dir(source)?;
  }
  Ok(changed)
}

fn import_language(document: &mut ConfigDocument, path: Option<PathBuf>) -> ConfigResult<bool> {
  if document.get("/session/language").is_some() {
    return Ok(false);
  }
  let Some(path) = path else {
    return Ok(false);
  };
  let Ok(value) = fs::read_to_string(&path) else {
    return Ok(false);
  };
  let language = value.trim();
  if language.is_empty() {
    return Ok(false);
  }
  document.set(
    "/session/language",
    Value::String(language.replace('_', "-")),
  )?;
  Ok(true)
}

fn import_power(document: &mut ConfigDocument, root: &Path) -> ConfigResult<bool> {
  let mut changed = false;
  if document.get("/power/keep_awake").is_none()
    && let Some(path) = resolved(root, ".keep-awake")
    && let Ok(value) = fs::read_to_string(&path)
    && value.trim() == "enabled"
  {
    document.set("/power/keep_awake", Value::Bool(true))?;
    changed = true;
  }
  let Some(path) = resolved(root, "hypr/hypridle.conf") else {
    return Ok(changed);
  };
  let Ok(contents) = fs::read_to_string(&path) else {
    return Ok(changed);
  };
  let mut timeout = None;
  let mut action = String::new();
  for line in contents.lines() {
    let line = line.trim();
    if let Some(value) = line.strip_prefix("timeout =") {
      timeout = value.trim().parse::<u64>().ok();
    } else if let Some(value) = line.strip_prefix("on-timeout =") {
      action = value.trim().to_ascii_lowercase();
    } else if line == "}" {
      if let Some(seconds) = timeout.take() {
        let key = if action.contains("dpms") && action.contains("off") {
          "screen_off_minutes"
        } else if action.contains("lock") {
          "lock_minutes"
        } else {
          ""
        };
        if !key.is_empty() {
          let pointer = format!("/power/{key}");
          if document.get(&pointer).is_none() {
            document.set(&pointer, Value::from(seconds / 60))?;
            changed = true;
          }
        }
      }
      action.clear();
    }
  }
  Ok(changed)
}

fn import_removable_devices(document: &mut ConfigDocument, root: &Path) -> ConfigResult<bool> {
  if document.get("/removable_devices").is_some() {
    return Ok(false);
  }
  let Some(path) = resolved(root, "removable-devices/config.json") else {
    return Ok(false);
  };
  let Ok(contents) = fs::read_to_string(&path) else {
    return Ok(false);
  };
  let value: Value =
    serde_json::from_str(&strip_json_comments(&contents)).map_err(ConfigError::InvalidJson)?;
  if !value.is_object() {
    return Ok(false);
  }
  document.set("/removable_devices", value)?;
  Ok(true)
}

fn strip_json_comments(contents: &str) -> String {
  let mut output = String::with_capacity(contents.len());
  let mut chars = contents.chars().peekable();
  let mut in_string = false;
  let mut escaped = false;
  while let Some(character) = chars.next() {
    if in_string {
      output.push(character);
      if escaped {
        escaped = false;
      } else if character == '\\' {
        escaped = true;
      } else if character == '"' {
        in_string = false;
      }
      continue;
    }
    if character == '"' {
      in_string = true;
      output.push(character);
    } else if character == '/' && chars.peek() == Some(&'/') {
      chars.next();
      for comment_character in chars.by_ref() {
        if comment_character == '\n' {
          output.push('\n');
          break;
        }
      }
    } else {
      output.push(character);
    }
  }
  output
}

fn import_hyprland(document: &mut ConfigDocument, root: &Path) -> ConfigResult<bool> {
  let mut changed = false;
  if document.get("/hyprland/input").is_none()
    && let Some(path) = resolved(root, "input.toml")
    && let Some(value) = parse_input_toml(&path)?
  {
    document.set("/hyprland/input", value)?;
    changed = true;
  }
  if document.get("/hyprland/keyboard").is_none()
    && let Some(path) = resolved(root, "generated/hypr/input.lua")
    && let Some(value) = parse_keyboard_lua(&path)?
  {
    document.set("/hyprland/keyboard", value)?;
    changed = true;
  }
  Ok(changed)
}

fn import_displays(document: &mut ConfigDocument, root: &Path) -> ConfigResult<bool> {
  if document.get("/displays").is_some() {
    return Ok(false);
  }
  let Some(path) = resolved(root, "generated/hypr/monitors.lua") else {
    return Ok(false);
  };
  let Ok(contents) = fs::read_to_string(&path) else {
    return Ok(false);
  };
  let mut monitors = Map::new();
  let mut workspaces: Map<String, Value> = Map::new();
  let mut primary_monitor = None;

  for body in lua_call_bodies(&contents, "hl.monitor") {
    let fields = lua_fields(&body);
    let Some(name) = fields.get("output").and_then(Value::as_str) else {
      continue;
    };
    let mut monitor = Map::new();
    for key in [
      "mode",
      "position",
      "scale",
      "transform",
      "vrr",
      "mirror",
      "bitdepth",
      "disabled",
      "supports_hdr",
      "hdr",
      "sdrbrightness",
      "sdrsaturation",
    ] {
      if let Some(value) = fields.get(key) {
        let canonical = match key {
          "supports_hdr" => "hdr",
          "sdrbrightness" => "sdr_brightness",
          "sdrsaturation" => "sdr_saturation",
          _ => key,
        };
        monitor.insert(canonical.into(), value.clone());
      }
    }
    monitors.insert(name.to_string(), Value::Object(monitor));
  }

  for body in lua_call_bodies(&contents, "hl.workspace_rule") {
    let fields = lua_fields(&body);
    let (Some(workspace), Some(monitor)) = (
      fields.get("workspace").and_then(Value::as_str),
      fields.get("monitor").and_then(Value::as_str),
    ) else {
      continue;
    };
    let Ok(workspace) = workspace.parse::<u64>() else {
      continue;
    };
    let ids = workspaces
      .entry(monitor.to_string())
      .or_insert_with(|| Value::Array(Vec::new()));
    if let Some(ids) = ids.as_array_mut()
      && !ids.iter().any(|value| value.as_u64() == Some(workspace))
    {
      ids.push(Value::Number(workspace.into()));
    }
    if workspace == 1 && fields.get("default") == Some(&Value::Bool(true)) {
      primary_monitor = Some(Value::String(monitor.to_string()));
    }
  }

  if monitors.is_empty() && workspaces.is_empty() {
    return Ok(false);
  }
  for ids in workspaces.values_mut() {
    if let Some(ids) = ids.as_array_mut() {
      ids.sort_by_key(Value::as_u64);
    }
  }
  let mut displays = Map::new();
  displays.insert(
    "primary_monitor".into(),
    primary_monitor.unwrap_or(Value::Null),
  );
  displays.insert("monitors".into(), Value::Object(monitors));
  displays.insert("workspaces".into(), Value::Object(workspaces));
  document.set("/displays", Value::Object(displays))?;
  Ok(true)
}

fn lua_call_bodies(contents: &str, call: &str) -> Vec<String> {
  let mut bodies = Vec::new();
  let mut offset = 0;
  while let Some(relative) = contents[offset..].find(&format!("{call}({{")) {
    let start = offset + relative + call.len() + 2;
    let Some(end) = contents[start..].find("})") else {
      break;
    };
    bodies.push(contents[start..start + end].to_string());
    offset = start + end + 2;
  }
  bodies
}

fn lua_fields(body: &str) -> Map<String, Value> {
  body
    .split(',')
    .filter_map(|field| {
      let (key, raw) = field.split_once('=')?;
      Some((key.trim().to_string(), parse_lua_value(raw.trim())?))
    })
    .collect()
}

fn parse_lua_value(raw: &str) -> Option<Value> {
  parse_scalar(raw).or_else(|| {
    raw
      .parse::<f64>()
      .ok()
      .and_then(serde_json::Number::from_f64)
      .map(Value::Number)
  })
}

fn parse_keybindings_toml(path: &Path) -> ConfigResult<Option<Value>> {
  let Ok(contents) = fs::read_to_string(path) else {
    return Ok(None);
  };
  let mut bindings = Map::new();
  let mut current = None;
  for line in contents.lines() {
    let line = line.trim();
    if let Some(section) = line.strip_prefix("[keybindings.")
      && let Some(section) = section.strip_suffix(']')
    {
      current = Some(section.trim_matches('"').to_string());
      bindings.insert(current.clone().unwrap(), Value::Object(Map::new()));
      continue;
    }
    let Some(id) = current.as_ref() else {
      continue;
    };
    let Some((key, raw)) = line.split_once('=') else {
      continue;
    };
    let Some(value) = parse_scalar(raw.trim()) else {
      continue;
    };
    if let Some(object) = bindings.get_mut(id).and_then(Value::as_object_mut) {
      object.insert(key.trim().to_string(), value);
    }
  }
  Ok((!bindings.is_empty()).then_some(Value::Object(bindings)))
}

fn parse_input_toml(path: &Path) -> ConfigResult<Option<Value>> {
  let Ok(contents) = fs::read_to_string(path) else {
    return Ok(None);
  };
  let mut sections = Map::new();
  let mut current = None;
  for line in contents.lines() {
    let line = line.trim();
    if let Some(section) = line
      .strip_prefix('[')
      .and_then(|line| line.strip_suffix(']'))
      && matches!(section, "mouse" | "touchpad")
    {
      current = Some(section.to_string());
      sections.insert(section.to_string(), Value::Object(Map::new()));
      continue;
    }
    let Some(section) = current.as_ref() else {
      continue;
    };
    let Some((key, raw)) = line.split_once('=') else {
      continue;
    };
    let Some(value) = parse_scalar(raw.trim()) else {
      continue;
    };
    if let Some(object) = sections.get_mut(section).and_then(Value::as_object_mut) {
      object.insert(key.trim().to_string(), value);
    }
  }
  Ok((!sections.is_empty()).then_some(Value::Object(sections)))
}

fn parse_keyboard_lua(path: &Path) -> ConfigResult<Option<Value>> {
  let Ok(contents) = fs::read_to_string(path) else {
    return Ok(None);
  };
  let mut keyboard = Map::new();
  for key in ["kb_layout", "kb_variant", "kb_options"] {
    let Some(value) = contents.lines().find_map(|line| {
      let value = line.trim().strip_prefix(&format!("{key} ="))?.trim();
      let value = value.strip_suffix(',').unwrap_or(value);
      let value = value.strip_prefix('"')?.strip_suffix('"')?;
      Some(value.to_string())
    }) else {
      continue;
    };
    keyboard.insert(
      match key {
        "kb_layout" => "layout",
        "kb_variant" => "variant",
        _ => "options",
      }
      .into(),
      Value::String(value),
    );
  }
  Ok((!keyboard.is_empty()).then_some(Value::Object(keyboard)))
}

fn parse_scalar(raw: &str) -> Option<Value> {
  serde_json::from_str(raw).ok().or_else(|| {
    raw
      .parse::<f64>()
      .ok()
      .and_then(serde_json::Number::from_f64)
      .map(Value::Number)
  })
}

fn lock_file(file: &File) -> io::Result<()> {
  let result = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX) };
  if result == 0 {
    Ok(())
  } else {
    Err(io::Error::last_os_error())
  }
}

struct LockGuard<'a>(&'a File);

impl Drop for LockGuard<'_> {
  fn drop(&mut self) {
    let _ = unsafe { libc::flock(self.0.as_raw_fd(), libc::LOCK_UN) };
  }
}

impl ConfigDocument {
  /// A copy whose section contents have their object keys sorted. The struct
  /// fields keep their order, so `schema_version` still comes first.
  pub fn sorted_sections(&self) -> ConfigDocument {
    ConfigDocument {
      schema_version: self.schema_version,
      sections: self
        .sections
        .iter()
        .map(|(name, value)| (name.clone(), sorted_value(value)))
        .collect(),
    }
  }
}

/// Returns `value` with every object's keys in sorted order, recursively.
///
/// `serde_json` keeps insertion order when its `preserve_order` feature is enabled,
/// and the Control Center enables it. Cargo unifies features across a workspace
/// build, so the configuration writers would otherwise change their output with
/// the set of crates built together. The configuration files have always been
/// written with sorted keys, so every writer goes through this function.
pub fn sorted_value(value: &Value) -> Value {
  match value {
    Value::Object(map) => {
      let mut entries: Vec<(&String, &Value)> = map.iter().collect();
      entries.sort_by(|left, right| left.0.cmp(right.0));
      Value::Object(
        entries
          .into_iter()
          .map(|(key, item)| (key.clone(), sorted_value(item)))
          .collect(),
      )
    }
    Value::Array(items) => Value::Array(items.iter().map(sorted_value).collect()),
    other => other.clone(),
  }
}

#[cfg(test)]
mod tests {
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
    assert!(
      validate_animation_setting("animation_workspace_in_style", &Value::from("fade")).is_ok()
    );
    assert!(
      validate_animation_setting("animation_layers_in_style", &Value::from("gnomed")).is_err()
    );
    assert!(
      validate_animation_setting("animation_layers_in_style", &Value::from("slidefade")).is_err()
    );
    assert!(
      validate_animation_setting("animation_layers_out_style", &Value::from("popin")).is_ok()
    );
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
    assert!(
      validate_animation_setting("animation_spring_stiffness", &Value::from(238.119)).is_ok()
    );
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
    assert!(
      document
        .get("/layout/taskbar")
        .is_some_and(Value::is_object)
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
  fn save_keeps_previous_valid_configuration_as_backup() {
    let directory =
      env::temp_dir().join(format!("argvus-config-test-{}-backup", std::process::id()));
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
    let directory =
      env::temp_dir().join(format!("argvus-config-test-{}-ensure", std::process::id()));
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
    let directory =
      env::temp_dir().join(format!("argvus-config-test-{}-theme", std::process::id()));
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

  impl ConfigStore {
    fn set_explicit_for_test(&self, pointer: &str, value: Value) -> ConfigResult<()> {
      self.update(pointer, value).map(|_| ())
    }
  }
}

#[cfg(test)]
mod sorted_value_tests {
  use super::*;
  use serde_json::json;

  #[test]
  fn sorts_object_keys_at_every_depth() {
    let unsorted = json!({"b": {"d": 1, "c": 2}, "a": [{"z": 0, "y": 1}]});
    let text = serde_json::to_string(&sorted_value(&unsorted)).expect("serialize");
    assert_eq!(text, r#"{"a":[{"y":1,"z":0}],"b":{"c":2,"d":1}}"#);
  }
}

/// An environment variable as a path, when it is set and not empty.
fn non_empty_env(name: &str) -> Option<PathBuf> {
  env::var_os(name)
    .filter(|value| !value.is_empty())
    .map(PathBuf::from)
}

#[cfg(test)]
mod save_tests {
  use super::*;
  use std::os::unix::fs::PermissionsExt;

  fn root(name: &str) -> PathBuf {
    let root = env::temp_dir().join(format!("argvus-save-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    root
  }

  fn set_theme(store: &ConfigStore, theme: &str) {
    store
      .modify(|document| document.set("/appearance/theme", Value::String(theme.into())))
      .expect("save");
  }

  #[test]
  fn saves_keep_two_backup_generations_and_private_modes() {
    let config_home = root("generations");
    let store = ConfigStore::at(&config_home);
    // The first save has no previous state to back up; the second backs up the first
    // state; the third moves that backup to the older generation.
    set_theme(&store, "sunset");
    set_theme(&store, "dracula");
    let backups = config_home.join("argvus").join("data/backups");
    assert!(backups.join("config.json.bak").is_file());
    assert!(!backups.join("config.json.bak.1").exists());
    set_theme(&store, "tokyo-night");
    assert!(backups.join("config.json.bak.1").is_file());
    let section = config_home.join("argvus/config/appearance.json");
    let mode = fs::metadata(&section)
      .expect("section")
      .permissions()
      .mode()
      & 0o777;
    assert_eq!(mode, 0o600);
    let _ = fs::remove_dir_all(config_home);
  }

  #[test]
  fn an_empty_variable_counts_as_unset() {
    let name = "ARGVUS_TEST_EMPTY_CONFIG_VARIABLE";
    // SAFETY: the variable name is unique to this test, so no other test reads it.
    unsafe { env::set_var(name, "") };
    assert!(non_empty_env(name).is_none());
    unsafe { env::remove_var(name) };
  }

  #[test]
  fn a_read_waits_for_a_writer_that_holds_the_lock() {
    use std::sync::mpsc;
    use std::time::Duration;

    let directory = env::temp_dir().join(format!(
      "argvus-config-test-{}-read-lock",
      std::process::id()
    ));
    let _ = fs::remove_dir_all(&directory);
    let store = ConfigStore::at(&directory);
    store.save(&ConfigDocument::default()).unwrap();

    let writer = store.open_lock().unwrap();
    lock_file(&writer).unwrap();
    let (sender, receiver) = mpsc::channel();
    let reader = ConfigStore::at(&directory);
    std::thread::spawn(move || {
      let _ = sender.send(reader.load().is_ok());
    });
    // The read must wait while the writer holds the exclusive lock.
    assert!(receiver.recv_timeout(Duration::from_millis(200)).is_err());
    drop(LockGuard(&writer));
    assert_eq!(receiver.recv_timeout(Duration::from_secs(5)), Ok(true));
    let _ = fs::remove_dir_all(&directory);
  }
}
