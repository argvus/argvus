//! Theme identity and the facts that depend only on it: the canonical id, the
//! Sticky/Float variant, the GTK mode and the wallpaper.
//!
//! A theme is described by an optional drop-in manifest
//! (`appearance/themes.d/<id>/theme.toml`), and the built-in tables cover the
//! themes that ship without one. This is the Rust home of the logic that the
//! former `theme-switch.sh`, `theme-manifest.sh` and `hypr.sh` held.

use std::fs;
use std::path::{Path, PathBuf};

/// Light themes, by the id they are known under (family and Float).
const LIGHT_THEMES: &[&str] = &[
  "argvus-light",
  "argvus-light-float",
  "github-light",
  "github-light-float",
  "solarized-light",
  "solarized-light-float",
  "one-light",
  "one-light-float",
  "everforest-light",
  "everforest-light-float",
  "catppuccin-latte",
  "catppuccin-latte-float",
  "gruvbox-light",
  "gruvbox-light-float",
  "nord-light",
  "nord-light-float",
];

/// Legacy and alternate ids that map to a canonical theme.
const CANONICAL_IDS: &[(&str, &str)] = &[
  ("argvus-dark-aether", "argvus-dark"),
  ("argvus-dark-aether-float", "argvus-dark-float"),
  ("argvus-light-veil", "argvus-light"),
  ("argvus-light-veil-float", "argvus-light-float"),
  ("argvus-onedark", "one-dark"),
  ("argvus-onedark-float", "one-dark-float"),
  ("argvus-dark-dracula", "dracula"),
  ("argvus-dark-dracula-float", "dracula-float"),
  ("argvus-dark-silver", "silver-dark"),
  ("argvus-dark-silver-float", "silver-dark-float"),
  ("argvus-dark-slate", "slate-dark"),
  ("argvus-dark-slate-float", "slate-dark-float"),
  ("argvus-dark-universe", "universe"),
  ("argvus-dark-universe-float", "universe-float"),
  ("argvus-dark-gruvbox-high", "gruvbox-high-dark"),
  ("argvus-dark-gruvbox-high-float", "gruvbox-high-dark-float"),
  ("argvus-dark-gruvbox", "gruvbox-dark"),
  ("argvus-dark-gruvbox-float", "gruvbox-dark-float"),
  ("argvus-github-light", "github-light"),
  ("argvus-github-light-float", "github-light-float"),
  ("argvus-light-solarized", "solarized-light"),
  ("argvus-light-solarized-float", "solarized-light-float"),
  ("argvus-light-gruvbox", "gruvbox-light"),
  ("argvus-light-gruvbox-float", "gruvbox-light-float"),
  ("argvus-dark-rose-pine", "rose-pine"),
  ("argvus-dark-rosepine", "rose-pine"),
  ("argvus-dark-rose-pine-float", "rose-pine-float"),
  ("argvus-dark-rosepine-float", "rose-pine-float"),
  ("argvus-dark-tokio-night", "tokyo-night"),
  ("argvus-dark-tokyo-night", "tokyo-night"),
  ("argvus-dark-tokio-night-float", "tokyo-night-float"),
  ("argvus-dark-tokyo-night-float", "tokyo-night-float"),
  ("argvus-dark-solitude", "solitude"),
  ("argvus-dark-solitude-float", "solitude-float"),
  ("argvus-dark-sunset", "sunset"),
  ("dark-sunset", "sunset"),
  ("dark-sunset-float", "sunset-float"),
  ("argvus-dark-sunset-float", "sunset-float"),
  ("argvus-dark-matrix", "matrix"),
  ("argvus-dark-matrix-float", "matrix-float"),
  ("argvus-dark-monokai", "monokai-dark"),
  ("argvus-dark-monokai-float", "monokai-dark-float"),
  ("argvus-catppuccin-latte", "catppuccin-latte"),
  ("argvus-catppuccin-latte-float", "catppuccin-latte-float"),
  ("argvus-light-catppuccin-latte", "catppuccin-latte"),
  (
    "argvus-light-catppuccin-latte-float",
    "catppuccin-latte-float",
  ),
];

/// Built-in wallpapers, relative to the wallpaper root, for themes without a
/// manifest.
const BUILTIN_WALLPAPERS: &[(&str, &str)] = &[
  ("argvus-dark", "argvus-dark.jxl"),
  ("argvus-light", "argvus-light.jxl"),
  ("dracula", "abstract/dark/dracula-abstract-dark.jxl"),
  ("gruvbox-dark", "abstract/dark/gruvbox-abstract-dark.jxl"),
  (
    "gruvbox-high-dark",
    "abstract/dark/gruvbox-high-abstract-dark.jxl",
  ),
  ("monokai-dark", "abstract/dark/monokai-abstract-dark.jxl"),
  ("one-dark", "abstract/dark/one-dark-abstract-dark.jxl"),
  ("rose-pine", "abstract/dark/rose-pine-abstract-dark.jxl"),
  ("silver-dark", "abstract/dark/silver-abstract-dark.jxl"),
  ("slate-dark", "abstract/dark/slate-abstract-dark.jxl"),
  ("sunset", "abstract/dark/sunset-abstract-dark.jxl"),
  ("tokyo-night", "abstract/dark/tokyo-night-abstract-dark.jxl"),
  ("matrix", "abstract/dark/matrix-abstract-dark.jxl"),
  ("solitude", "abstract/dark/solitude-abstract-dark.jxl"),
  ("universe", "abstract/dark/universe-abstract-dark.jxl"),
  (
    "catppuccin-latte",
    "abstract/light/catppuccin-latte-abstract-light.jxl",
  ),
  (
    "github-light",
    "abstract/light/github-light-abstract-light.jxl",
  ),
  (
    "solarized-light",
    "abstract/light/solarized-abstract-light.jxl",
  ),
  ("one-light", "abstract/light/one-light-abstract-light.jxl"),
  (
    "everforest-light",
    "abstract/light/everforest-abstract-light.jxl",
  ),
  ("gruvbox-light", "abstract/light/gruvbox-abstract-light.jxl"),
];

/// The canonical id of a theme. Unknown ids come back unchanged.
pub fn canonical_theme_id(id: &str) -> &str {
  CANONICAL_IDS
    .iter()
    .find(|(alias, _)| *alias == id)
    .map(|(_, canonical)| *canonical)
    .unwrap_or(id)
}

/// Whether `id` can name a theme: lowercase letters, digits, `.`, `_` and `-`,
/// starting with a letter or digit, and never a path. Theme ids are joined into
/// file paths, so anything else is refused before it reaches the filesystem.
pub fn is_valid_theme_id(id: &str) -> bool {
  let mut chars = id.chars();
  let starts_well = chars
    .next()
    .is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit());
  starts_well
    && id.len() <= 128
    && !id.contains("..")
    && id
      .chars()
      .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '.' | '_' | '-'))
}

/// Resolves the theme to apply. An empty id stays empty. The Sticky/Float mode
/// is an independent setting, so the id is reduced to its family and the
/// variant is taken from the layout mode, never from the caller.
pub fn resolve_theme(id: &str, variant: &str) -> String {
  let canonical = canonical_theme_id(id);
  if canonical.is_empty() {
    return String::new();
  }
  let family = canonical.strip_suffix("-float").unwrap_or(canonical);
  if variant == "float" {
    format!("{family}-float")
  } else {
    family.to_string()
  }
}

/// The GTK colour scheme of a theme: `light` or `dark`. A manifest decides when
/// one exists for the family; otherwise the built-in list does.
pub fn gtk_mode(themes_dir: &Path, theme: &str) -> &'static str {
  let family = theme.strip_suffix("-float").unwrap_or(theme);
  if manifest_path(themes_dir, family).is_file() {
    return match table_value(themes_dir, family, "appearance", "gtk_scheme").as_deref() {
      Some("prefer-light") => "light",
      _ => "dark",
    };
  }
  if LIGHT_THEMES.contains(&theme) {
    "light"
  } else {
    "dark"
  }
}

/// The wallpaper file of a theme under `wallpaper_root`, when it exists. The
/// manifest is consulted first, then the built-in table.
pub fn theme_wallpaper(themes_dir: &Path, wallpaper_root: &Path, theme: &str) -> Option<PathBuf> {
  let family = theme.strip_suffix("-float").unwrap_or(theme);
  let manifest_name = top_value(themes_dir, family, "wallpaper");
  let builtin_name = BUILTIN_WALLPAPERS
    .iter()
    .find(|(id, _)| *id == family)
    .map(|(_, name)| (*name).to_string());
  [manifest_name, builtin_name]
    .into_iter()
    .flatten()
    .map(|name| wallpaper_root.join(name))
    .find(|path| path.is_file())
}

fn manifest_path(themes_dir: &Path, family: &str) -> PathBuf {
  themes_dir.join(family).join("theme.toml")
}

/// A top-level string key of a theme manifest. Reading stops at the first
/// `[table]` header, so keys inside a table cannot shadow top-level keys.
pub fn top_value(themes_dir: &Path, family: &str, key: &str) -> Option<String> {
  let text = fs::read_to_string(manifest_path(themes_dir, family)).ok()?;
  let prefix = format!("{key} = \"");
  for line in text.lines() {
    if line.starts_with('[') {
      break;
    }
    if let Some(value) = quoted_value(line, &prefix) {
      return non_empty(value);
    }
  }
  None
}

/// A string key of one `[table]` of a theme manifest.
pub fn table_value(themes_dir: &Path, family: &str, table: &str, key: &str) -> Option<String> {
  let text = fs::read_to_string(manifest_path(themes_dir, family)).ok()?;
  let header = format!("[{table}]");
  let prefix = format!("{key} = \"");
  let mut inside = false;
  for line in text.lines() {
    if line == header {
      inside = true;
      continue;
    }
    if inside && line.starts_with('[') {
      break;
    }
    if inside && let Some(value) = quoted_value(line, &prefix) {
      return non_empty(value);
    }
  }
  None
}

/// The value of a `key = "value"` line, when the line has exactly that shape.
fn quoted_value<'a>(line: &'a str, prefix: &str) -> Option<&'a str> {
  // sed's `[[:space:]]*$` allows trailing blanks after the closing quote.
  let rest = line.strip_prefix(prefix)?.trim_end();
  let value = rest.strip_suffix('"')?;
  if value.contains('"') {
    return None;
  }
  Some(value)
}

fn non_empty(value: &str) -> Option<String> {
  (!value.is_empty()).then(|| value.to_string())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn aliases_resolve_to_their_canonical_id() {
    assert_eq!(canonical_theme_id("argvus-dark-tokyo-night"), "tokyo-night");
    assert_eq!(
      canonical_theme_id("argvus-light-catppuccin-latte-float"),
      "catppuccin-latte-float"
    );
    assert_eq!(
      canonical_theme_id("dracula"),
      "dracula",
      "unknown ids are kept"
    );
  }

  #[test]
  fn variant_decides_the_float_suffix_not_the_caller() {
    assert_eq!(resolve_theme("dracula-float", "sticky"), "dracula");
    assert_eq!(resolve_theme("dracula", "float"), "dracula-float");
    assert_eq!(resolve_theme("argvus-dark-matrix", "float"), "matrix-float");
    assert_eq!(resolve_theme("", "float"), "");
  }

  #[test]
  fn gtk_mode_follows_the_builtin_light_list_without_a_manifest() {
    let missing = Path::new("/nonexistent-themes-d");
    assert_eq!(gtk_mode(missing, "github-light-float"), "light");
    assert_eq!(gtk_mode(missing, "argvus-dark"), "dark");
  }

  #[test]
  fn manifest_values_are_read_top_level_and_per_table() {
    let root = std::env::temp_dir().join(format!("argvus-theme-manifest-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let family = root.join("demo");
    fs::create_dir_all(&family).expect("family dir");
    fs::write(
            family.join("theme.toml"),
            "wallpaper = \"demo.jxl\"\n[appearance]\ngtk_scheme = \"prefer-light\"\nwallpaper = \"ignored\"\n",
        )
        .expect("manifest");
    assert_eq!(
      top_value(&root, "demo", "wallpaper").as_deref(),
      Some("demo.jxl")
    );
    assert_eq!(
      table_value(&root, "demo", "appearance", "gtk_scheme").as_deref(),
      Some("prefer-light")
    );
    assert_eq!(gtk_mode(&root, "demo-float"), "light");
    assert_eq!(
      table_value(&root, "demo", "appearance", "wallpaper").as_deref(),
      Some("ignored")
    );
    let _ = fs::remove_dir_all(root);
  }

  #[test]
  fn wallpaper_prefers_the_manifest_then_the_builtin_table() {
    let root = std::env::temp_dir().join(format!("argvus-theme-wall-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let themes = root.join("themes.d");
    let wallpapers = root.join("wallpapers");
    fs::create_dir_all(themes.join("demo")).expect("themes dir");
    fs::create_dir_all(wallpapers.join("abstract/dark")).expect("wallpaper dir");
    fs::write(themes.join("demo/theme.toml"), "wallpaper = \"demo.png\"\n").expect("manifest");
    fs::write(wallpapers.join("demo.png"), "").expect("manifest wallpaper");
    fs::write(
      wallpapers.join("abstract/dark/dracula-abstract-dark.jxl"),
      "",
    )
    .expect("builtin");

    assert_eq!(
      theme_wallpaper(&themes, &wallpapers, "demo-float"),
      Some(wallpapers.join("demo.png"))
    );
    assert_eq!(
      theme_wallpaper(&themes, &wallpapers, "dracula"),
      Some(wallpapers.join("abstract/dark/dracula-abstract-dark.jxl"))
    );
    assert_eq!(theme_wallpaper(&themes, &wallpapers, "matrix"), None);
    let _ = fs::remove_dir_all(root);
  }
}

#[cfg(test)]
mod id_tests {
  use super::is_valid_theme_id;

  #[test]
  fn accepts_real_theme_ids() {
    for id in [
      "argvus-dark",
      "catppuccin-latte-float",
      "gruvbox-high-dark",
      "sunset",
      "v2.1",
    ] {
      assert!(is_valid_theme_id(id), "{id}");
    }
  }

  #[test]
  fn refuses_paths_and_control_characters() {
    for id in [
      "",
      "../x",
      "a/b",
      "/etc/passwd",
      ".hidden",
      "Dark",
      "a b",
      "x\n",
      "a..b",
    ] {
      assert!(!is_valid_theme_id(id), "{id:?}");
    }
  }
}
