//! Theme discovery from drop-in `theme.toml` manifests.
//!
//! This module discovers additional themes installed as packages via
//! `/usr/share/argvus/appearance/themes.d/<id>/theme.toml`.
//!
//! Built-in themes (`argvus-dark`, `argvus-light`) always take precedence.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Theme category enumeration (serde-compatible).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeCategory {
  Dark,
  Light,
}

/// Theme metadata from `theme.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThemeEntry {
  /// Canonical theme ID (lowercase, hyphens, alphanumeric).
  pub id: String,
  /// Display name (e.g., "Dracula", "GitHub Light").
  pub name: String,
  /// Category: Dark or Light.
  pub category: ThemeCategory,
  /// Default accent color (hex, e.g., "#BD93F9").
  #[serde(default)]
  pub accent: String,
  /// Wallpaper path relative to argvus-wallpapers root.
  #[serde(default)]
  pub wallpaper: String,
  /// Localized display names (e.g., {"en-US": "Dracula", "pt-BR": "Dracula"}).
  #[serde(default)]
  pub name_i18n: HashMap<String, String>,
}

impl ThemeEntry {
  /// Validates that this entry's `id` is safe (alphanumeric + hyphens, no path traversal).
  pub fn validate_id(&self) -> Result<(), String> {
    // ID must be lowercase alphanumeric and hyphens only
    if !self
      .id
      .chars()
      .all(|c| c.is_ascii_lowercase() || c == '-' || c.is_ascii_digit())
    {
      return Err(format!("Invalid theme ID: {}", self.id));
    }
    // No empty ID
    if self.id.is_empty() {
      return Err("Theme ID cannot be empty".to_string());
    }
    // No leading/trailing hyphens
    if self.id.starts_with('-') || self.id.ends_with('-') {
      return Err("Theme ID cannot start or end with hyphen".to_string());
    }
    // No -float suffix (that's a mode, not part of the ID)
    if self.id.ends_with("-float") {
      return Err("Theme ID should not include -float suffix".to_string());
    }
    Ok(())
  }
}

/// Report of discovered themes with any warnings encountered.
#[derive(Debug, Clone)]
pub struct DiscoveryReport {
  /// Discovered themes (built-in + drop-in).
  pub themes: Vec<ThemeEntry>,
  /// Warnings encountered during discovery (e.g., malformed manifests).
  pub warnings: Vec<String>,
}

/// Discovers installed themes from `/usr/share/argvus/appearance/themes.d/`.
///
/// Built-in themes (`argvus-dark` and `argvus-light`) are always included.
/// Additional themes are discovered from drop-in `theme.toml` files.
/// Malformed manifests are logged in warnings and skipped.
///
/// **Note**: Call this function once per application startup and cache the result.
/// Do not call repeatedly in tight loops, as it reads from disk.
pub fn discover_themes(system_config: &Path) -> DiscoveryReport {
  let mut themes = Vec::new();
  let mut warnings = Vec::new();

  // Add built-in themes first (highest priority, cannot be overridden)
  themes.push(ThemeEntry {
    id: "argvus-dark".to_string(),
    name: "ARGVUS Dark".to_string(),
    category: ThemeCategory::Dark,
    accent: "#3590bd".to_string(),
    wallpaper: "argvus-dark.jxl".to_string(),
    name_i18n: {
      let mut map = HashMap::new();
      map.insert("en-US".to_string(), "ARGVUS Dark".to_string());
      map.insert("pt-BR".to_string(), "ARGVUS Dark".to_string());
      map
    },
  });

  themes.push(ThemeEntry {
    id: "argvus-light".to_string(),
    name: "ARGVUS Light".to_string(),
    category: ThemeCategory::Light,
    accent: "#181818".to_string(),
    wallpaper: "argvus-light.jxl".to_string(),
    name_i18n: {
      let mut map = HashMap::new();
      map.insert("en-US".to_string(), "ARGVUS Light".to_string());
      map.insert("pt-BR".to_string(), "ARGVUS Light".to_string());
      map
    },
  });

  // Discover additional themes from drop-in manifests
  let themes_dir = system_config.join("appearance/themes.d");
  if let Ok(entries) = fs::read_dir(&themes_dir) {
    for entry in entries.flatten() {
      if let Ok(metadata) = entry.metadata()
        && metadata.is_dir()
      {
        let manifest_path = entry.path().join("theme.toml");
        if let Ok(content) = fs::read_to_string(&manifest_path) {
          match toml::from_str::<ThemeEntry>(&content) {
            Ok(theme) => {
              // Validate and skip if invalid
              if let Err(e) = theme.validate_id() {
                warnings.push(format!(
                  "Skipping invalid theme manifest {}: {}",
                  manifest_path.display(),
                  e
                ));
                continue;
              }
              // Skip if already exists (built-in wins)
              if themes.iter().any(|t| t.id == theme.id) {
                warnings.push(format!(
                  "Theme {} already exists (built-in), skipping drop-in",
                  theme.id
                ));
                continue;
              }
              themes.push(theme);
            }
            Err(e) => {
              warnings.push(format!(
                "Failed to parse {}: {}",
                manifest_path.display(),
                e
              ));
            }
          }
        }
      }
    }
  }

  DiscoveryReport { themes, warnings }
}

/// Checks if a theme ID is known (built-in or discovered).
pub fn is_known_theme(id: &str, all_themes: &[ThemeEntry]) -> bool {
  all_themes.iter().any(|t| t.id == id)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_theme_entry_validation_valid() {
    let entry = ThemeEntry {
      id: "valid-theme".to_string(),
      name: "Valid Theme".to_string(),
      category: ThemeCategory::Dark,
      accent: "#000000".to_string(),
      wallpaper: "test.jxl".to_string(),
      name_i18n: HashMap::new(),
    };
    assert!(entry.validate_id().is_ok());

    let entry2 = ThemeEntry {
      id: "valid-theme-2".to_string(),
      ..entry
    };
    assert!(entry2.validate_id().is_ok());
  }

  #[test]
  fn test_theme_entry_validation_invalid() {
    let mut entry = ThemeEntry {
      id: "ValidTheme".to_string(),
      name: "Invalid".to_string(),
      category: ThemeCategory::Dark,
      accent: "#000000".to_string(),
      wallpaper: "test.jxl".to_string(),
      name_i18n: HashMap::new(),
    };

    // Uppercase not allowed
    assert!(entry.validate_id().is_err());

    // Underscores not allowed
    entry.id = "valid_theme".to_string();
    assert!(entry.validate_id().is_err());

    // Empty ID not allowed
    entry.id = String::new();
    assert!(entry.validate_id().is_err());

    // Leading hyphen not allowed
    entry.id = "-valid-theme".to_string();
    assert!(entry.validate_id().is_err());

    // -float suffix not allowed
    entry.id = "valid-theme-float".to_string();
    assert!(entry.validate_id().is_err());
  }

  #[test]
  fn test_builtin_themes_always_present() {
    let report = discover_themes(std::path::Path::new("/tmp"));
    assert!(report.themes.iter().any(|t| t.id == "argvus-dark"));
    assert!(report.themes.iter().any(|t| t.id == "argvus-light"));
    assert_eq!(report.themes[0].id, "argvus-dark");
    assert_eq!(report.themes[1].id, "argvus-light");
  }

  #[test]
  fn test_builtin_names_correct() {
    let report = discover_themes(std::path::Path::new("/tmp"));
    let dark = report
      .themes
      .iter()
      .find(|t| t.id == "argvus-dark")
      .unwrap();
    assert_eq!(dark.name, "ARGVUS Dark");
    assert_eq!(dark.category, ThemeCategory::Dark);

    let light = report
      .themes
      .iter()
      .find(|t| t.id == "argvus-light")
      .unwrap();
    assert_eq!(light.name, "ARGVUS Light");
    assert_eq!(light.category, ThemeCategory::Light);
  }
}
