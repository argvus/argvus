//! Discovers the icon themes installed on this system and validates the
//! icon theme names written to `/appearance/icon_theme`.
//!
//! Discovery is pure file-system reading over roots passed in by the caller,
//! so tests point it at temporary directories and never read the real
//! `/usr/share/icons` or the user's home. Effects (config writes, the
//! `icon-switch.sh` run) live in `backend`.
use crate::model::{IconThemeEntry, IconThemeSource, IconThemeVariant};
use argvus_control_center_core::paths::{data_home, home};
use serde::Deserialize;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Default icon theme for dark themes, used when `/appearance/icon_theme` is
/// empty. `icon-switch.sh` resolves the same names.
pub const DEFAULT_ICON_THEME: &str = "Argvus Dark Icons";
/// Default icon theme for light themes.
pub const DEFAULT_LIGHT_ICON_THEME: &str = "Argvus Light Icons";
/// Built-in theme families that use the light mode. Mirrors `gtk_mode_for_theme`
/// in `theme-manifest.sh`; drop-in themes declare their category instead.
const BUILT_IN_LIGHT_THEMES: [&str; 7] = [
  "argvus-light",
  "github-light",
  "solarized-light",
  "one-light",
  "everforest-light",
  "gruvbox-light",
  "catppuccin-latte",
];

/// System root where packages install icon themes.
const SYSTEM_ICON_ROOT: &str = "/usr/share/icons";
/// Directory of the official ARGVUS icon theme manifests, relative to the
/// system config root.
const ICON_THEME_MANIFESTS_DIR: &str = "appearance/icon-themes.d";
/// Matches the limit enforced by `argvus-config` for `appearance.icon_theme`.
const MAX_ICON_THEME_NAME_LENGTH: usize = 128;
/// Shared fallback directories. They are inherited by every theme and are not
/// choices for the user.
const FALLBACK_THEME_DIRECTORIES: [&str; 2] = ["default", "hicolor"];

/// A directory searched for icon themes. Earlier roots take precedence when two
/// roots provide a theme with the same directory name, matching the user
/// overriding a system theme.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IconRoot {
  pub source: IconThemeSource,
  pub path: PathBuf,
}

/// Returns the default search roots: the user's `~/.icons` first, then
/// `$XDG_DATA_HOME/icons`, then the system directory.
pub fn default_icon_roots() -> Vec<IconRoot> {
  icon_roots(&home(), &data_home(), Path::new(SYSTEM_ICON_ROOT))
}

/// Builds the search roots from explicit locations, so the precedence can be
/// tested without depending on the environment.
pub fn icon_roots(home: &Path, data_home: &Path, system_root: &Path) -> Vec<IconRoot> {
  vec![
    IconRoot {
      source: IconThemeSource::User,
      path: home.join(".icons"),
    },
    IconRoot {
      source: IconThemeSource::User,
      path: data_home.join("icons"),
    },
    IconRoot {
      source: IconThemeSource::System,
      path: system_root.to_path_buf(),
    },
  ]
}

/// The `[Icon Theme]` keys ARGVUS reads from an `index.theme` file.
#[derive(Debug, Default, PartialEq, Eq)]
struct IndexTheme {
  name: Option<String>,
  comment: Option<String>,
  hidden: bool,
  directories: Vec<String>,
  inherits: Vec<String>,
}

impl IndexTheme {
  /// A theme is a choice when it is visible and either ships icon
  /// directories or inherits from a theme other than the bare `default`.
  /// Cursor-only themes have neither, so they are not listed.
  fn is_choice(&self) -> bool {
    if self.hidden {
      return false;
    }
    !self.directories.is_empty() || self.inherits.iter().any(|parent| parent != "default")
  }
}

/// Reads the `[Icon Theme]` section of an `index.theme` file. Other sections
/// are ignored, and unknown keys are skipped.
fn parse_index_theme(content: &str) -> IndexTheme {
  let mut theme = IndexTheme::default();
  let mut in_icon_theme_section = false;
  for line in content.lines() {
    let line = line.trim();
    if let Some(section) = line
      .strip_prefix('[')
      .and_then(|rest| rest.strip_suffix(']'))
    {
      in_icon_theme_section = section == "Icon Theme";
      continue;
    }
    if !in_icon_theme_section {
      continue;
    }
    let Some((key, value)) = line.split_once('=') else {
      continue;
    };
    let value = value.trim();
    match key.trim() {
      "Name" => theme.name = Some(value.to_string()),
      "Comment" => theme.comment = Some(value.to_string()),
      "Hidden" => theme.hidden = value.eq_ignore_ascii_case("true"),
      "Directories" => theme.directories = split_list(value),
      "Inherits" => theme.inherits = split_list(value),
      _ => {}
    }
  }
  theme
}

/// Splits a comma-separated `index.theme` list, dropping empty items.
fn split_list(value: &str) -> Vec<String> {
  value
    .split(',')
    .map(str::trim)
    .filter(|item| !item.is_empty())
    .map(str::to_string)
    .collect()
}

/// The subset of an official `icon-theme.toml` manifest ARGVUS reads.
#[derive(Debug, Deserialize)]
struct IconThemeManifest {
  name: String,
  variant: String,
  #[serde(default)]
  official: bool,
}

/// Maps a manifest's `variant` value to its typed form. Unknown values make
/// the manifest non-official instead of guessing a variant.
fn parse_variant(value: &str) -> Option<IconThemeVariant> {
  match value {
    "default" => Some(IconThemeVariant::Default),
    "dark" => Some(IconThemeVariant::Dark),
    "light" => Some(IconThemeVariant::Light),
    _ => None,
  }
}

/// Reads every `icon-theme.toml` under `manifests_root` and returns the
/// official ones keyed by theme name. Malformed manifests are skipped so one
/// bad file does not hide the others.
fn official_manifests(manifests_root: &Path) -> Vec<(String, IconThemeVariant)> {
  let Ok(directory_entries) = fs::read_dir(manifests_root) else {
    return Vec::new();
  };
  let mut manifest_paths: Vec<PathBuf> = directory_entries
    .flatten()
    .map(|entry| entry.path().join("icon-theme.toml"))
    .collect();
  manifest_paths.sort();

  manifest_paths
    .into_iter()
    .filter_map(|path| {
      let content = fs::read_to_string(path).ok()?;
      let manifest: IconThemeManifest = toml::from_str(&content).ok()?;
      if !manifest.official {
        return None;
      }
      Some((manifest.name, parse_variant(&manifest.variant)?))
    })
    .collect()
}

/// Lists the icon themes available under `roots`.
///
/// A directory name appears once: the first root that provides it wins. Only
/// the system root can contribute official entries, so a user copy that reuses
/// an official name is listed as a user theme. The result is sorted with
/// official themes first, then by display name.
pub fn discover_icon_themes(roots: &[IconRoot], manifests_root: &Path) -> Vec<IconThemeEntry> {
  let official = official_manifests(manifests_root);
  let mut seen_directories = BTreeSet::new();
  let mut entries = Vec::new();

  for root in roots {
    let Ok(directory_entries) = fs::read_dir(&root.path) else {
      continue;
    };
    let mut theme_dirs: Vec<PathBuf> = directory_entries
      .flatten()
      .map(|entry| entry.path())
      .filter(|path| path.is_dir())
      .collect();
    theme_dirs.sort();

    for theme_dir in theme_dirs {
      let Some(directory_name) = theme_dir.file_name().and_then(|name| name.to_str()) else {
        continue;
      };
      if FALLBACK_THEME_DIRECTORIES.contains(&directory_name)
        || seen_directories.contains(directory_name)
      {
        continue;
      }
      let Ok(content) = fs::read_to_string(theme_dir.join("index.theme")) else {
        continue;
      };
      let theme = parse_index_theme(&content);
      if !theme.is_choice() {
        continue;
      }

      seen_directories.insert(directory_name.to_string());
      let official_variant = match root.source {
        IconThemeSource::System => official
          .iter()
          .find(|(name, _)| name == directory_name)
          .map(|(_, variant)| *variant),
        IconThemeSource::User => None,
      };
      entries.push(IconThemeEntry {
        display_name: theme.name.unwrap_or_else(|| directory_name.to_string()),
        comment: theme.comment,
        directory_name: directory_name.to_string(),
        source: root.source,
        official: official_variant,
      });
    }
  }

  entries.sort_by(|left, right| {
    left
      .official
      .is_none()
      .cmp(&right.official.is_none())
      .then_with(|| {
        left
          .display_name
          .to_lowercase()
          .cmp(&right.display_name.to_lowercase())
      })
  });
  entries
}

/// Lists the icon themes installed on this machine.
pub fn installed_icon_themes() -> Vec<IconThemeEntry> {
  let manifests_root =
    argvus_control_center_core::paths::system_config_root().join(ICON_THEME_MANIFESTS_DIR);
  discover_icon_themes(&default_icon_roots(), &manifests_root)
}

/// Checks a name before it is written to `/appearance/icon_theme`. The name is
/// a directory name, so path separators and relative components are rejected.
pub fn validate_icon_theme_name(name: &str) -> Result<(), String> {
  if name.is_empty() {
    return Err("icon theme name is empty".into());
  }
  if name == "." || name == ".." || name.contains(['/', '\0']) {
    return Err(format!("icon theme name is not a plain name: {name}"));
  }
  if name.len() > MAX_ICON_THEME_NAME_LENGTH {
    return Err("icon theme name is too long".into());
  }
  Ok(())
}

/// The ARGVUS default icon theme of a theme, by its mode. A `-float` variant
/// uses the mode of its family.
pub fn default_icon_theme_for(
  theme_id: &str,
  official_themes: &[argvus_theme::discovery::ThemeEntry],
) -> &'static str {
  let family = theme_id.strip_suffix("-float").unwrap_or(theme_id);
  let is_light = match official_themes.iter().find(|entry| entry.id == family) {
    Some(entry) => entry.category == argvus_theme::discovery::ThemeCategory::Light,
    None => BUILT_IN_LIGHT_THEMES.contains(&family),
  };
  if is_light {
    DEFAULT_LIGHT_ICON_THEME
  } else {
    DEFAULT_ICON_THEME
  }
}

/// Resolves the raw output of `argvus config get /appearance/icon_theme --raw`.
/// A missing or empty value means the default of the active theme's mode.
pub fn configured_icon_theme(raw_output: Option<&str>, default: &str) -> String {
  raw_output
    .map(|value| value.trim().trim_matches('"').trim())
    .filter(|value| !value.is_empty())
    .map(str::to_string)
    .unwrap_or_else(|| default.to_string())
}

#[cfg(test)]
mod tests;
