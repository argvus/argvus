//! Typed view of the canonical launcher section, shared with Control Center.
use argvus_config_core::ConfigStore;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const SOURCES: [&str; 8] = [
  "applications",
  "windows",
  "settings",
  "actions",
  "projects",
  "ssh",
  "files",
  "help",
];

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ApplicationSort {
  Alphabetical,
  #[default]
  Recent,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct Settings {
  /// Enabled sources in user-defined priority order.
  pub sources: Vec<String>,
  pub favorites: Vec<String>,
  pub recent_files: bool,
  pub index_enabled: bool,
  pub index_paused: bool,
  pub include_hidden: bool,
  pub roots: Vec<PathBuf>,
  pub exclusions: Vec<PathBuf>,
  pub reduced_motion: bool,
  pub show_icons: bool,
  pub show_open_now: bool,
  pub application_sort: ApplicationSort,
  pub selected_category: String,
}

impl Default for Settings {
  fn default() -> Self {
    Self {
      sources: SOURCES.iter().map(|source| (*source).into()).collect(),
      favorites: Vec::new(),
      recent_files: true,
      index_enabled: false,
      index_paused: false,
      include_hidden: false,
      roots: Vec::new(),
      exclusions: Vec::new(),
      reduced_motion: false,
      show_icons: true,
      show_open_now: false,
      application_sort: ApplicationSort::default(),
      selected_category: String::new(),
    }
  }
}

impl Settings {
  pub fn load() -> Result<Self, String> {
    let document = ConfigStore::from_environment()
      .and_then(|store| store.load_effective())
      .map_err(|e| e.to_string())?;
    let settings: Self = document
      .get("/launcher")
      .cloned()
      .map(serde_json::from_value)
      .transpose()
      .map_err(|e| e.to_string())?
      .unwrap_or_default();
    settings.validate()?;
    Ok(settings)
  }

  pub fn validate(&self) -> Result<(), String> {
    let mut unique = std::collections::HashSet::new();
    if self
      .sources
      .iter()
      .any(|source| !SOURCES.contains(&source.as_str()) || !unique.insert(source))
    {
      return Err("launcher sources must be known and unique".into());
    }
    if !self.selected_category.is_empty() && !SOURCES.contains(&self.selected_category.as_str()) {
      return Err("invalid selected category".into());
    }
    if self
      .roots
      .iter()
      .chain(&self.exclusions)
      .any(|path| !path.is_absolute())
    {
      return Err("launcher roots and exclusions must be absolute paths".into());
    }
    Ok(())
  }

  pub fn save(&self) -> Result<(), String> {
    self.validate()?;
    let value = serde_json::to_value(self).map_err(|e| e.to_string())?;
    ConfigStore::from_environment()
      .and_then(|store| store.update("/launcher", value))
      .map(|_| ())
      .map_err(|e| e.to_string())
  }

  /// Saves only fields edited by this caller, preserving independent concurrent edits.
  pub fn save_changes(&self, previous: &Self) -> Result<(), String> {
    let store = ConfigStore::from_environment().map_err(|error| error.to_string())?;
    self.save_changes_at(previous, &store)
  }

  fn save_changes_at(&self, previous: &Self, store: &ConfigStore) -> Result<(), String> {
    self.validate()?;
    let before = serde_json::to_value(previous).map_err(|error| error.to_string())?;
    let after = serde_json::to_value(self).map_err(|error| error.to_string())?;
    let mut patch = serde_json::Map::new();
    if let Some(fields) = after.as_object() {
      for (key, value) in fields {
        if before.get(key) != Some(value) {
          patch.insert(format!("/launcher/{key}"), value.clone());
        }
      }
    }
    store
      .patch(&patch)
      .map(|_| ())
      .map_err(|error| error.to_string())
  }

  pub fn enabled(&self, source: &str) -> bool {
    self.sources.iter().any(|value| value == source)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn indexing_requires_explicit_opt_in_and_absolute_paths() {
    let mut settings = Settings::default();
    assert!(!settings.index_enabled);
    assert!(settings.roots.is_empty());
    settings.roots.push("relative".into());
    assert!(settings.validate().is_err());
  }
  #[test]
  fn rejects_unknown_and_duplicate_sources() {
    let mut settings = Settings::default();
    settings.sources.push("applications".into());
    assert!(settings.validate().is_err());
    settings.sources = vec!["shell".into()];
    assert!(settings.validate().is_err());
  }
  #[test]
  fn independent_edits_do_not_overwrite_other_fields() {
    let temporary = tempfile::tempdir().unwrap();
    let store = ConfigStore::at(temporary.path());
    let initial = Settings::default();
    let mut files = initial.clone();
    files.index_enabled = true;
    files.show_open_now = true;
    files.application_sort = ApplicationSort::Alphabetical;
    files.save_changes_at(&initial, &store).unwrap();
    let mut favorites = initial.clone();
    favorites.favorites.push("app:example.desktop".into());
    favorites.save_changes_at(&initial, &store).unwrap();
    let document = store.load().unwrap();
    assert_eq!(
      document.get("/launcher/index_enabled"),
      Some(&serde_json::json!(true))
    );
    assert_eq!(
      document.get("/launcher/favorites"),
      Some(&serde_json::json!(["app:example.desktop"]))
    );
    assert_eq!(
      document.get("/launcher/show_open_now"),
      Some(&serde_json::json!(true))
    );
    assert_eq!(
      document.get("/launcher/application_sort"),
      Some(&serde_json::json!("alphabetical"))
    );
  }
  #[test]
  fn appearance_and_category_defaults_and_validation() {
    let mut settings: Settings = serde_json::from_str("{}").unwrap();
    assert!(settings.show_icons);
    assert!(!settings.show_open_now);
    assert_eq!(settings.application_sort, ApplicationSort::Recent);
    assert!(settings.selected_category.is_empty());
    settings.selected_category = "applications".into();
    assert!(settings.validate().is_ok());
    settings.selected_category = "invalid".into();
    assert!(settings.validate().is_err());
  }
}
