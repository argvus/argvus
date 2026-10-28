//! Theme identity and the facts that depend only on it: the canonical id, the
//! Sticky/Float variant, the GTK mode and the wallpaper.
//!
//! A theme is described by its manifest (`themes/<id>/theme.toml`, schema 2),
//! official under the system root or the user's own under
//! `$XDG_DATA_HOME/argvus/themes`. The built-in tables cover ids that have no
//! manifest: the legacy aliases, and the light list and wallpapers a greeter or
//! an old marker may still name.

use crate::session_paths;
use argvus_theme_core::discovery;
use argvus_theme_core::manifest::{Category, ThemeManifest};
use argvus_theme_core::paths::ThemePaths;
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

/// The installed manifest of a theme (`<id>` or `<id>-float`): the official
/// one, or else the user's. An invalid id or manifest is `None`.
pub fn manifest(paths: &ThemePaths, theme: &str) -> Option<ThemeManifest> {
  discovery::find(paths, theme.strip_suffix("-float").unwrap_or(theme))
}

/// The theme roots of the environment, as `argvus theme` resolves them.
pub fn theme_paths() -> ThemePaths {
  ThemePaths::from_environment().unwrap_or_else(|| ThemePaths {
    system_root: env_system_root(),
    user_themes: PathBuf::new(),
    generated: PathBuf::new(),
  })
}

fn env_system_root() -> PathBuf {
  std::env::var_os("ARGVUS_SYSTEM_CONFIG")
    .filter(|value| !value.is_empty())
    .map(PathBuf::from)
    .unwrap_or_else(|| PathBuf::from(session_paths::PACKAGED_SYSTEM_CONFIG))
}

/// The GTK colour scheme of a theme: `light` or `dark`. The manifest decides
/// (its `[gtk] scheme`, which defaults to its category); an id without one
/// falls back to the built-in light list.
pub fn gtk_mode(paths: &ThemePaths, theme: &str) -> &'static str {
  if let Some(manifest) = manifest(paths, theme) {
    return match manifest.gtk_scheme.as_str() {
      "prefer-light" => "light",
      "prefer-dark" => "dark",
      _ if manifest.category == Category::Light => "light",
      _ => "dark",
    };
  }
  if LIGHT_THEMES.contains(&theme) {
    "light"
  } else {
    "dark"
  }
}

/// Where the official wallpapers are installed; a manifest names them there.
const PACKAGED_WALLPAPERS: &str = "/usr/share/backgrounds/argvus";

/// The wallpaper file of a theme, when it exists: the manifest's, then the
/// built-in name under `wallpaper_root`. A manifest wallpaper under the
/// packaged wallpaper directory is taken from `wallpaper_root`, which is that
/// directory unless the environment moves it.
pub fn theme_wallpaper(paths: &ThemePaths, wallpaper_root: &Path, theme: &str) -> Option<PathBuf> {
  let family = theme.strip_suffix("-float").unwrap_or(theme);
  let from_manifest = manifest(paths, theme)
    .and_then(|manifest| manifest.wallpaper)
    .map(
      |wallpaper| match wallpaper.strip_prefix(PACKAGED_WALLPAPERS) {
        Ok(relative) => wallpaper_root.join(relative),
        Err(_) => wallpaper,
      },
    );
  let builtin = BUILTIN_WALLPAPERS
    .iter()
    .find(|(id, _)| *id == family)
    .map(|(_, name)| wallpaper_root.join(name));
  [from_manifest, builtin]
    .into_iter()
    .flatten()
    .find(|path| path.is_file())
}

/// The accent a theme owns (`palette.accent`, `#RRGGBB`), when it is installed.
pub fn theme_accent(paths: &ThemePaths, theme: &str) -> Option<String> {
  let manifest = manifest(paths, theme)?;
  manifest.token("accent").ok().map(|color| color.hex())
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

  fn theme_tree(name: &str) -> (PathBuf, ThemePaths) {
    let root = std::env::temp_dir().join(format!(
      "argvus-theme-resolve-{name}-{}",
      std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    let paths = ThemePaths {
      system_root: root.join("system"),
      user_themes: root.join("user"),
      generated: root.join("generated"),
    };
    (root, paths)
  }

  fn write_manifest(dir: &Path, id: &str, extra: &str) {
    std::fs::create_dir_all(dir.join(id)).expect("theme dir");
    std::fs::write(
      dir.join(id).join("theme.toml"),
      format!(
        "schema = 2\nid = \"{id}\"\nname = \"{id}\"\n{extra}\n\
         [palette]\nbg = \"#000000\"\nfg = \"#FFFFFF\"\naccent = \"#12AB34\"\n"
      ),
    )
    .expect("manifest");
  }

  #[test]
  fn gtk_mode_follows_the_builtin_light_list_without_a_manifest() {
    let (root, paths) = theme_tree("builtin");
    assert_eq!(gtk_mode(&paths, "github-light-float"), "light");
    assert_eq!(gtk_mode(&paths, "argvus-dark"), "dark");
    let _ = std::fs::remove_dir_all(root);
  }

  #[test]
  fn manifest_values_come_from_the_official_then_the_user_theme() {
    let (root, paths) = theme_tree("manifest");
    write_manifest(&paths.system_themes(), "demo", "category = \"light\"");
    write_manifest(
      &paths.user_themes,
      "mine",
      "category = \"dark\"\n[gtk]\nscheme = \"prefer-light\"",
    );
    assert_eq!(gtk_mode(&paths, "demo-float"), "light");
    assert_eq!(gtk_mode(&paths, "mine"), "light");
    assert_eq!(theme_accent(&paths, "mine").as_deref(), Some("#12AB34"));
    assert_eq!(theme_accent(&paths, "../demo"), None);
    let _ = std::fs::remove_dir_all(root);
  }

  #[test]
  fn wallpaper_prefers_the_manifest_then_the_builtin_table() {
    let (root, paths) = theme_tree("wallpaper");
    let wallpapers = root.join("wallpapers");
    std::fs::create_dir_all(wallpapers.join("abstract/dark")).expect("wallpaper dir");
    std::fs::write(wallpapers.join("demo.png"), "").expect("manifest wallpaper");
    write_manifest(
      &paths.system_themes(),
      "demo",
      &format!(
        "category = \"dark\"\nwallpaper = \"{}\"",
        wallpapers.join("demo.png").display()
      ),
    );
    std::fs::write(
      wallpapers.join("abstract/dark/dracula-abstract-dark.jxl"),
      "",
    )
    .expect("builtin");

    assert_eq!(
      theme_wallpaper(&paths, &wallpapers, "demo-float"),
      Some(wallpapers.join("demo.png"))
    );
    assert_eq!(
      theme_wallpaper(&paths, &wallpapers, "dracula"),
      Some(wallpapers.join("abstract/dark/dracula-abstract-dark.jxl"))
    );
    assert_eq!(theme_wallpaper(&paths, &wallpapers, "matrix"), None);
    let _ = std::fs::remove_dir_all(root);
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
