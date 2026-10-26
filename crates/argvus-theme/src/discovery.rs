//! The installed themes, for the theme pickers: the official packages under
//! `/usr/share/argvus/themes/<id>/theme.toml` and the user's own under
//! `$XDG_DATA_HOME/argvus/themes`, as `argvus-theme-core` discovers them.
//!
//! ARGVUS Dark and ARGVUS Light always come first.

use argvus_theme_core::discovery;
use argvus_theme_core::manifest::Category;
use argvus_theme_core::paths::ThemePaths;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

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
  /// The wallpaper file of the theme, or empty.
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

/// Discovers the installed themes under `system_config` and the user's theme
/// directory. ARGVUS Dark and ARGVUS Light come first and are always listed;
/// a theme whose manifest is broken is skipped with a warning.
///
/// **Note**: Call this function once per application startup and cache the result.
/// Do not call repeatedly in tight loops, as it reads from disk.
pub fn discover_themes(system_config: &Path) -> DiscoveryReport {
  let user_themes = ThemePaths::from_environment()
    .map(|paths| paths.user_themes)
    .unwrap_or_default();
  let paths = ThemePaths {
    system_root: system_config.to_path_buf(),
    user_themes,
    generated: PathBuf::new(),
  };
  let found = discovery::discover(&paths);
  let mut themes: Vec<ThemeEntry> = found
    .themes
    .iter()
    .map(|installed| {
      let manifest = &installed.manifest;
      ThemeEntry {
        id: manifest.id.clone(),
        name: manifest.name.clone(),
        category: match manifest.category {
          Category::Dark => ThemeCategory::Dark,
          Category::Light => ThemeCategory::Light,
        },
        accent: manifest
          .token("accent")
          .map(|color| color.hex())
          .unwrap_or_default(),
        wallpaper: manifest
          .wallpaper
          .as_ref()
          .map(|path| path.display().to_string())
          .unwrap_or_default(),
        name_i18n: manifest
          .name_i18n
          .iter()
          .map(|(locale, name)| (locale.clone(), name.clone()))
          .collect(),
      }
    })
    .collect();
  for builtin in builtin_themes().into_iter().rev() {
    match themes.iter().position(|theme| theme.id == builtin.id) {
      Some(index) => {
        let entry = themes.remove(index);
        themes.insert(0, entry);
      }
      None => themes.insert(0, builtin),
    }
  }
  DiscoveryReport {
    themes,
    warnings: found.warnings,
  }
}

/// ARGVUS Dark and ARGVUS Light, as listed when their manifests are missing.
fn builtin_themes() -> [ThemeEntry; 2] {
  let entry = |id: &str, name: &str, category, accent: &str| ThemeEntry {
    id: id.to_string(),
    name: name.to_string(),
    category,
    accent: accent.to_string(),
    wallpaper: String::new(),
    name_i18n: HashMap::from([
      ("en-US".to_string(), name.to_string()),
      ("pt-BR".to_string(), name.to_string()),
    ]),
  };
  [
    entry("argvus-dark", "ARGVUS Dark", ThemeCategory::Dark, "#3590bd"),
    entry(
      "argvus-light",
      "ARGVUS Light",
      ThemeCategory::Light,
      "#181818",
    ),
  ]
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
