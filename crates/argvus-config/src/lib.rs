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
mod json_pointers;
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

mod config_defaults;
mod config_lock;
mod config_validation;
mod json_tree;
mod legacy_formats;
mod legacy_import;
mod legacy_migration;

use config_defaults::{apply_defaults, apply_packaged_defaults};
use config_lock::{LockGuard, lock_file};
use config_validation::{
  RETIRED_ANIMATION_KEYS, animation_styles, validate_appearance, validate_audio,
  validate_control_panel, validate_default_apps, validate_displays, validate_effects,
  validate_fonts, validate_hyprland, validate_keyboard_shortcuts, validate_layout, validate_power,
  validate_session, validate_taskbar,
};
pub use json_tree::sorted_value;
use json_tree::{child_value, pointer_components, set_nested, unset_nested};
pub(crate) use legacy_import::config_key_for_binding;
use legacy_migration::{import_legacy_values, migrate_legacy_layout, migrate_legacy_sections};

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
      document.set(
        json_pointers::APPEARANCE_THEME,
        Value::String(theme.to_owned()),
      )?;
      // Sticky/Float is an independent setting (see `apply_layout_variant`);
      // applying a theme must not change the mode the user already chose, so
      // only the geometry for the *current* variant is reasserted here.
      let current_variant = document
        .get(json_pointers::LAYOUT_VARIANT)
        .and_then(Value::as_str)
        .unwrap_or("sticky")
        .to_owned();
      reset_layout_geometry(document, &current_variant)?;
      if reset_custom_accent {
        document.set(json_pointers::APPEARANCE_ACCENT_CUSTOM, Value::Bool(false))?;
      }
      let accent_custom = document
        .get(json_pointers::APPEARANCE_ACCENT_CUSTOM)
        .and_then(Value::as_bool)
        .unwrap_or(false);
      if !accent_custom {
        if let Some(accent) = accent {
          document.set(
            json_pointers::APPEARANCE_ACCENT,
            Value::String(accent.to_owned()),
          )?;
        }
        document.set(json_pointers::APPEARANCE_ACCENT_CUSTOM, Value::Bool(false))?;
      }
      if let Some(mode) = gtk_mode {
        document.set(
          json_pointers::APPEARANCE_GTK_MODE,
          Value::String(mode.to_owned()),
        )?;
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
        document.set(
          json_pointers::APPEARANCE_WALLPAPER,
          Value::String(wallpaper.to_owned()),
        )?;
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
      document.set(
        json_pointers::LAYOUT_VARIANT,
        Value::String(variant.to_owned()),
      )?;
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
    (2, 0, false, 0, 1, [0, 0, 0, 1])
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

fn default_schema_version() -> u32 {
  CURRENT_SCHEMA_VERSION
}

/// An environment variable as a path, when it is set and not empty.
fn non_empty_env(name: &str) -> Option<PathBuf> {
  env::var_os(name)
    .filter(|value| !value.is_empty())
    .map(PathBuf::from)
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

#[cfg(test)]
mod tests;

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
