//! Implements persistent application state in crate `argvus control center apps`. This separation keeps external effects from contaminating models, routes, or rendering.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use argvus_control_center_core::config::reload_argvus_config_service;

use crate::catalog::{Category, STATE_VERSION};

/// Modular canonical configuration pointers are owned by the store at `/default_apps`.
fn config_pointer(category: Category) -> String {
  format!("/default_apps/{}", category.key())
}

/// Canonical ARGVUS default application per category, mirroring the
/// `/default_apps` values seeded by `argvus-config`. A stored value equal to
/// this is indistinguishable from "not set", so it is skipped on read and
/// unset on write.
fn config_default(category: Category) -> &'static str {
  match category {
    Category::Terminal => "argvus-terminal",
    Category::FileManager => "spf",
    Category::TextEditor => "mousepad",
    Category::TerminalEditor => "vim",
    Category::Browser => "firefox",
    Category::ImageViewer => "imv",
    Category::PdfViewer => "zathura",
    Category::VideoPlayer => "mpv",
    Category::AudioPlayer => "audacious",
    Category::Archive => "xarchiver",
    Category::Launcher => "rofi",
  }
}

/// Scans a section (the `default_apps` object or a flat `defaults.json`
/// document) for explicit picks. Categories whose value equals the canonical
/// default, or is empty, are indistinguishable from "not set" and are skipped,
/// so a fully materialized defaults map still reports nothing to change.
fn picks_from_section(section: &serde_json::Map<String, Value>) -> Vec<(Category, String)> {
  let mut picks = Vec::new();
  for category in Category::ORDER {
    let Some(Value::String(value)) = section.get(category.key()) else {
      continue;
    };
    if value.is_empty() || value == config_default(category) {
      continue;
    }
    picks.push((category, value.clone()));
  }
  picks
}

/// Reads explicit picks from the modular canonical document under `base` (an argvus
/// config home). Parses the section directly (no subprocess) so loading never
/// mutates state and still works when the `argvus-config` tool is unavailable.
/// Returns `Some(_)` whenever the document has a `default_apps` section — an
/// empty list means the canonical store is fully materialized with defaults, so
/// it is authoritative even without picks. Returns `None` when the document or
/// section is missing/unreadable.
fn read_default_apps_from_config(base: &Path) -> Option<Vec<(Category, String)>> {
  let content = fs::read_to_string(base.join("config/default_apps.json")).ok()?;
  let document: Value = serde_json::from_str(&content).ok()?;
  let section = document
    .get("default_apps")
    .and_then(Value::as_object)
    .or_else(|| document.as_object())?;
  Some(picks_from_section(section))
}

/// Builds an `AppState` from a non-empty set of explicit picks.
fn state_from_picks(picks: &[(Category, String)]) -> AppState {
  let mut state = AppState::new();
  for (category, value) in picks {
    state.set(*category, value.clone());
  }
  state
}

/// The value the modular store must hold for a category: the explicit pick when
/// present, otherwise the canonical ARGVUS default. Naming it separately from
/// `effective` keeps the Browser discrepancy visible — the modular store stores the
/// canonical `firefox`, while the control-center fallback is the empty string.
fn value_for_config(state: &AppState, category: Category) -> String {
  match state.get(category) {
    Some(value) if !value.is_empty() => value,
    _ => config_default(category).to_string(),
  }
}

/// The `defaults.json` content. Kept as a plain struct so the file stays
/// simple and readable; absent categories simply fall back to Argvus defaults.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct AppState {
  pub version: Option<u32>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub terminal: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub file_manager: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub text_editor: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub terminal_editor: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub browser: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub image_viewer: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub pdf_viewer: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub video_player: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub audio_player: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub archive: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub launcher: Option<String>,
}

impl AppState {
  /// Constructs `new` with this module's expected initial state. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn new() -> Self {
    AppState {
      version: Some(STATE_VERSION),
      ..Default::default()
    }
  }

  /// Executes the `field` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn field(&mut self, cat: Category) -> &mut Option<String> {
    macro_rules! field {
      ($c:expr, $name:ident) => {
        if cat == $c {
          return &mut self.$name;
        }
      };
    }
    field!(Category::Terminal, terminal);
    field!(Category::FileManager, file_manager);
    field!(Category::TextEditor, text_editor);
    field!(Category::TerminalEditor, terminal_editor);
    field!(Category::Browser, browser);
    field!(Category::ImageViewer, image_viewer);
    field!(Category::PdfViewer, pdf_viewer);
    field!(Category::VideoPlayer, video_player);
    field!(Category::AudioPlayer, audio_player);
    field!(Category::Archive, archive);
    field!(Category::Launcher, launcher);
    unreachable!("category {} not covered", cat.key())
  }

  /// Executes the `field_ref` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn field_ref(&self, cat: Category) -> Option<&String> {
    match cat {
      Category::Terminal => self.terminal.as_ref(),
      Category::FileManager => self.file_manager.as_ref(),
      Category::TextEditor => self.text_editor.as_ref(),
      Category::TerminalEditor => self.terminal_editor.as_ref(),
      Category::Browser => self.browser.as_ref(),
      Category::ImageViewer => self.image_viewer.as_ref(),
      Category::PdfViewer => self.pdf_viewer.as_ref(),
      Category::VideoPlayer => self.video_player.as_ref(),
      Category::AudioPlayer => self.audio_player.as_ref(),
      Category::Archive => self.archive.as_ref(),
      Category::Launcher => self.launcher.as_ref(),
    }
  }

  /// Stored value for a category (not the fallback).
  pub fn get(&self, cat: Category) -> Option<String> {
    self.field_ref(cat).cloned()
  }

  /// Effective value for a category: stored value, else the Argvus fallback.
  pub fn effective(&self, cat: Category) -> String {
    match self.get(cat) {
      Some(v) if !v.is_empty() => v,
      _ => cat.fallback().to_string(),
    }
  }

  /// Set a stored value (empty clears the entry).
  pub fn set(&mut self, cat: Category, value: impl Into<String>) {
    let value = value.into();
    if value.is_empty() {
      *self.field(cat) = None;
    } else {
      *self.field(cat) = Some(value);
    }
  }

  /// Clear a stored user override for one category.
  pub fn reset(&mut self, cat: Category) {
    *self.field(cat) = None;
  }

  /// Clear every stored user override while preserving the state version.
  pub fn reset_all(&mut self) {
    *self = AppState::new();
  }

  /// Categories that differ from the Argvus fallback (i.e. explicit picks).
  pub fn explicit(&self) -> Vec<(Category, String)> {
    Category::ORDER
      .iter()
      .copied()
      .filter_map(|c| self.get(c).map(|v| (c, v)))
      .filter(|(c, v)| !v.is_empty() && *v != c.fallback())
      .collect()
  }

  /// Load the effective state. `config/default_apps.json` is the canonical
  /// store and the source of truth: whenever it carries the section, its values
  /// win — fully materialized defaults mean "nothing changed". The
  /// `defaults.json` replica (and its legacy XDG state location) is only
  /// consulted when the canonical store is missing or unreadable.
  pub fn load() -> AppState {
    Self::load_in(&argvus_control_center_core::paths::argvus_config_home())
  }

  /// Load relative to an argvus config home (used by tests to stay hermetic).
  fn load_in(base: &Path) -> AppState {
    match read_default_apps_from_config(base) {
      Some(picks) => {
        if picks.is_empty() {
          AppState::new()
        } else {
          state_from_picks(&picks)
        }
      }
      None => Self::load_replica(base),
    }
  }

  /// Load the `defaults.json` replica, then the legacy state location.
  fn load_replica(base: &Path) -> AppState {
    // The replica is projected into the managed data tree. Accept the
    // pre-`data/` root location so an unmigrated profile keeps resolving.
    for primary in [
      base
        .join("data")
        .join("control-center")
        .join("defaults.json"),
      base.join("defaults.json"),
    ] {
      if primary.exists() {
        return Self::load_replica_from(&primary);
      }
    }
    let legacy = argvus_control_center_core::paths::legacy_defaults_file();
    if legacy.exists() {
      return Self::load_replica_from(&legacy);
    }
    AppState::new()
  }

  /// Parses a replica document with the same canonical-default semantics as
  /// Modular canonical state: only non-empty, non-canonical values become explicit picks,
  /// so the fully materialized replica written by `save` loads cleanly.
  fn load_replica_from(path: &Path) -> AppState {
    let content = match fs::read_to_string(path) {
      Ok(content) => content,
      Err(_) => return AppState::new(),
    };
    let Ok(Value::Object(section)) = serde_json::from_str::<Value>(&content) else {
      return AppState::new();
    };
    let picks = picks_from_section(&section);
    if picks.is_empty() {
      AppState::new()
    } else {
      state_from_picks(&picks)
    }
  }

  /// Load from a specific path (used by tests and by the GUI).
  pub fn load_from(path: &PathBuf) -> AppState {
    let content = match fs::read_to_string(path) {
      Ok(c) => c,
      Err(_) => return AppState::new(),
    };
    match serde_json::from_str::<AppState>(&content) {
      Ok(mut state) => {
        state.version.get_or_insert(STATE_VERSION);
        state
      }
      Err(_) => AppState::new(),
    }
  }

  /// Persist every category into the modular canonical store in one
  /// canonical patch, then request one service reload so the
  /// `/default_apps` section is always fully materialized: explicit picks keep
  /// their value and any category back on its canonical default is written back
  /// to that default rather than removed. The section is therefore never partial
  /// or empty. Failures are tolerated so a machine without the `argvus-config`
  /// tool still writes the file replica.
  pub fn sync_to_config(&self) {
    let mut patch = serde_json::Map::new();
    for category in Category::ORDER {
      patch.insert(
        config_pointer(category),
        Value::String(value_for_config(self, category)),
      );
    }
    let Ok(patch) = serde_json::to_string(&patch) else {
      return;
    };
    let persisted = argvus_control_center_core::process::command("argvus")
      .arg("config")
      .args(["patch", &patch])
      .status()
      .is_ok_and(|status| status.success());
    if persisted {
      let _ = reload_argvus_config_service();
    }
  }

  /// Persist to the state file (creating parent directories), keeping
  /// modular canonical store first via `sync_to_config` and materializing the
  /// full effective defaults into the `defaults.json` replica that runtime
  /// consumers (such as the default-app launcher) read.
  pub fn save(&self) -> std::io::Result<()> {
    self.sync_to_config();
    let path = argvus_control_center_core::paths::defaults_file();
    if let Some(parent) = path.parent() {
      fs::create_dir_all(parent)?;
    }
    let mut map = serde_json::Map::new();
    for category in Category::ORDER {
      map.insert(
        category.key().to_string(),
        Value::String(value_for_config(self, category)),
      );
    }
    let json = serde_json::to_string_pretty(&map).expect("effective defaults serialize to JSON");
    fs::write(&path, format!("{json}\n"))?;
    Ok(())
  }

  /// Persist to a specific path (tests).
  #[cfg(test)]
  pub fn save_to(&self, path: &PathBuf) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
      fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(self).unwrap();
    fs::write(path, format!("{json}\n"))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::env;
  use std::fs;

  /// Defines the constant `DIR`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
  const DIR: &str = "target/test-state";

  /// Executes the `tmp_file` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn tmp_file(name: &str) -> PathBuf {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
      .join(DIR)
      .join(name);
    if let Some(parent) = p.parent() {
      fs::create_dir_all(parent).unwrap();
    }
    p
  }

  #[test]
  /// Executes the `round_trip` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn round_trip() {
    let mut state = AppState::new();
    state.set(Category::Browser, "firefox");
    state.set(Category::Terminal, "kitty");

    let path = tmp_file("roundtrip.json");
    state.save_to(&path).unwrap();

    let loaded = AppState::load_from(&path);
    assert_eq!(loaded, state);
    assert_eq!(loaded.get(Category::Browser).as_deref(), Some("firefox"));
  }

  #[test]
  /// Executes the `missing_file_yields_empty_state` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn missing_file_yields_empty_state() {
    let path = tmp_file("missing.json");
    let _ = fs::remove_file(&path);
    let state = AppState::load_from(&path);
    assert_eq!(state.get(Category::Browser), None);
    assert_eq!(state.effective(Category::Terminal), "argvus-terminal");
  }

  #[test]
  /// Executes the `invalid_json_yields_empty_state` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn invalid_json_yields_empty_state() {
    let path = tmp_file("invalid.json");
    fs::write(&path, "not json {").unwrap();
    let state = AppState::load_from(&path);
    assert_eq!(state.effective(Category::Terminal), "argvus-terminal");
  }

  #[test]
  /// Applies the `set_clears_on_empty` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn set_clears_on_empty() {
    let mut state = AppState::new();
    state.set(Category::Browser, "firefox");
    assert_eq!(state.effective(Category::Browser), "firefox");
    state.set(Category::Browser, "");
    assert_eq!(state.effective(Category::Browser), "");
    assert_eq!(state.get(Category::Browser), None);
  }

  #[test]
  /// Executes the `explicit_only_lists_differences` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn explicit_only_lists_differences() {
    let mut state = AppState::new();
    state.set(Category::Browser, "chromium");
    state.set(Category::Terminal, "argvus-terminal"); // equals fallback
    let explicit = state.explicit();
    assert_eq!(explicit.len(), 1);
    assert_eq!(explicit[0].0, Category::Browser);
  }

  #[test]
  /// Executes the `env_config_home_is_respected` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn env_config_home_is_respected() {
    let p = PathBuf::from("/tmp/argvus-default-apps-test-config");
    unsafe {
      env::set_var("ARGVUS_CONFIG_HOME", &p);
    }
    assert_eq!(
      argvus_control_center_core::paths::argvus_config_home(),
      p.join("argvus")
    );
  }

  #[test]
  /// Executes the `config_pointer_matches_category_key` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn config_pointer_matches_category_key() {
    for category in Category::ORDER {
      assert_eq!(
        config_pointer(category),
        format!("/default_apps/{}", category.key())
      );
    }
  }

  #[test]
  /// Executes the `config_default_matches_catalog_fallback` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn config_default_matches_catalog_fallback() {
    for category in Category::ORDER {
      let fallback = category.fallback();
      if fallback.is_empty() {
        continue;
      }
      assert_eq!(
        config_default(category),
        fallback,
        "{} canonical default drifted from the catalog",
        category.key()
      );
    }
    assert_eq!(config_default(Category::Browser), "firefox");
  }

  #[test]
  /// Executes the modular canonical state precedence test in this module.
  fn modular_config_drives_effective_state() {
    let argvus =
      PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/test-state/config-home/argvus");
    fs::create_dir_all(&argvus).unwrap();
    fs::create_dir_all(argvus.join("config")).unwrap();

    // Canonical-only modular section plus a replica pick: the section
    // is authoritative even when fully materialized with defaults, so the
    // replica override is NOT shown.
    fs::write(
      argvus.join("config/default_apps.json"),
      r#"{"schema_version":1,"default_apps":{"terminal":"argvus-terminal","image_viewer":"imv","browser":"firefox"}}"#,
    )
    .unwrap();
    fs::write(argvus.join("defaults.json"), r#"{"browser":"chromium"}"#).unwrap();
    let state = AppState::load_in(&argvus);
    assert_eq!(state.get(Category::Browser), None);
    assert_eq!(state.get(Category::ImageViewer), None);
    assert_eq!(state.get(Category::Terminal), None);

    // Once the modular section carries an explicit pick it is the single truth, even
    // over a replica that disagrees. Canonical-typed values stay "unset".
    fs::write(
      argvus.join("config/default_apps.json"),
      r#"{"schema_version":1,"default_apps":{"image_viewer":"feh","browser":"firefox"}}"#,
    )
    .unwrap();
    let state = AppState::load_in(&argvus);
    assert_eq!(state.get(Category::Browser).as_deref(), None);
    assert_eq!(state.get(Category::ImageViewer).as_deref(), Some("feh"));

    // Fully materialized map with only canonical values: nothing to change.
    fs::write(
      argvus.join("config/default_apps.json"),
      r#"{"schema_version":1,"default_apps":{"terminal":"argvus-terminal","file_manager":"spf","text_editor":"mousepad","terminal_editor":"vim","browser":"firefox","image_viewer":"imv","pdf_viewer":"zathura","video_player":"mpv","audio_player":"audacious","archive":"xarchiver","launcher":"rofi"}}"#,
    )
    .unwrap();
    let state = AppState::load_in(&argvus);
    assert_eq!(state.get(Category::FileManager), None);
    assert_eq!(state.get(Category::Browser), None);

    // Missing modular section falls back to the defaults.json replica.
    fs::remove_file(argvus.join("config/default_apps.json")).unwrap();
    fs::write(argvus.join("defaults.json"), r#"{"file_manager":"yazi"}"#).unwrap();
    let state = AppState::load_in(&argvus);
    assert_eq!(state.get(Category::FileManager).as_deref(), Some("yazi"));
  }

  /// Executes the `explicit_ignores_canonical_typed_values` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  #[test]
  fn explicit_ignores_canonical_typed_values() {
    let mut state = AppState::new();
    state.set(Category::Browser, "chromium");
    state.set(Category::Terminal, "argvus-terminal"); // equals canonical
    assert_eq!(
      state.explicit(),
      vec![(Category::Browser, "chromium".to_string())]
    );
  }
}
