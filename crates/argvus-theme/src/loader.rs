//! Resource loading.
use std::fs;
use std::path::{Path, PathBuf};

use super::parser::{self, Rgba};

pub const DEFAULT_THEME: &str = "argvus-dark";

/// Official ARGVUS TUI theme identifiers accepted by pre-authentication
/// surfaces and shared theme consumers.
pub const OFFICIAL_THEMES: &[&str] = &[
  "argvus-dark",
  "argvus-dark-float",
  "dracula",
  "dracula-float",
  "gruvbox-dark",
  "gruvbox-dark-float",
  "gruvbox-high-dark",
  "gruvbox-high-dark-float",
  "monokai-dark",
  "monokai-dark-float",
  "one-dark",
  "one-dark-float",
  "rose-pine",
  "rose-pine-float",
  "silver-dark",
  "silver-dark-float",
  "slate-dark",
  "slate-dark-float",
  "sunset",
  "sunset-float",
  "tokyo-night",
  "tokyo-night-float",
  "matrix",
  "matrix-float",
  "solitude",
  "solitude-float",
  "universe",
  "universe-float",
  "argvus-light",
  "argvus-light-float",
  "catppuccin-latte",
  "catppuccin-latte-float",
  "github-light",
  "github-light-float",
  "gruvbox-light",
  "gruvbox-light-float",
  "solarized-light",
  "solarized-light-float",
  "one-light",
  "one-light-float",
  "everforest-light",
  "everforest-light-float",
];

/// One stylesheet the palette is read from.
pub enum Source {
  /// A file, read with its `@import` lines.
  File(PathBuf),
  /// A stylesheet rendered in memory.
  Text(String),
}

pub struct Loader {
  resource_dir: PathBuf,
  active_file: PathBuf,
  /// `generated/theme/current`, the build of the user's active theme.
  current_build: PathBuf,
  active_name_override: Option<String>,
  accent_override: Option<Rgba>,
}

impl Loader {
  pub fn new() -> Self {
    let explicit = std::env::var_os("ARGVUS_CONTROL_CENTER_RESOURCE_DIR").map(PathBuf::from);
    let installed_root = std::env::var_os("ARGVUS_SYSTEM_CONFIG")
      .map(PathBuf::from)
      .unwrap_or_else(|| PathBuf::from("/usr/share/argvus"));
    let installed = installed_root.join("appearance/config/tui");
    let legacy_installed = installed_root.join("control-center/config");
    let development = development_resources_dir();
    let resource_dir = explicit.unwrap_or_else(|| {
      if installed.is_dir() {
        installed
      } else if legacy_installed.is_dir() {
        legacy_installed
      } else {
        development
      }
    });
    Self {
      resource_dir,
      active_file: argvus_data_home().join(".active-theme"),
      current_build: argvus_data_home().join("generated/theme/current"),
      active_name_override: None,
      accent_override: None,
    }
  }

  /// Creates a loader whose active theme comes from a validated external
  /// selection instead of the current process user's private config.
  ///
  /// This is used by pre-authentication surfaces, which run as the `greeter`
  /// account and cannot safely read a target user's `$HOME` before login.
  pub fn with_active_name(mut self, name: &str) -> Self {
    self.active_name_override = Some(normalize_theme_name(name).to_string());
    self
  }

  /// Applies a validated opaque accent supplied by a pre-authentication
  /// projection. Invalid values are ignored so callers retain the theme
  /// palette as the safe fallback.
  pub fn with_accent_override(mut self, accent: &str) -> Self {
    let accent = accent.trim();
    if accent.len() == 7
      && accent.starts_with('#')
      && accent[1..]
        .chars()
        .all(|character| character.is_ascii_hexdigit())
    {
      self.accent_override = parser::parse_color(accent);
    }
    self
  }

  pub fn accent_override(&self) -> Option<Rgba> {
    self.accent_override
  }

  pub fn active_name(&self) -> String {
    if let Some(name) = &self.active_name_override {
      return name.clone();
    }
    fs::read_to_string(&self.active_file)
      .ok()
      .map(|value| value.trim().to_string())
      .filter(|value| !value.is_empty())
      .unwrap_or_else(|| DEFAULT_THEME.to_string())
  }

  /// The stylesheets of the palette, in order: the shared style, the base
  /// theme and the palette of the active theme.
  pub fn sources(&self) -> Vec<Source> {
    let active = self.active_name();
    let mut sources = vec![
      Source::File(self.resource_dir.join("style.css")),
      Source::File(self.resource_dir.join("theme.css")),
    ];
    if let Some(build) = self.active_build_palette(&active) {
      sources.push(Source::File(build));
    } else if let Some(rendered) = self.rendered_palette(&active) {
      sources.push(Source::Text(rendered));
    }
    sources
  }

  /// The TUI palette of the theme build, when the active theme is the user's
  /// own (not one chosen for another account) and the build was rendered for it.
  fn active_build_palette(&self, active: &str) -> Option<PathBuf> {
    if self.active_name_override.is_some() {
      return None;
    }
    argvus_theme_core::build::current_file_for(&self.current_build, active, "tui/theme.css")
  }

  /// The TUI palette rendered from the theme's manifest, for a theme without a
  /// build. A theme chosen for another account (the greeter) is looked up among
  /// the official themes only, because that account's own themes are private.
  fn rendered_palette(&self, active: &str) -> Option<String> {
    let mut paths = argvus_theme_core::paths::ThemePaths::from_environment().unwrap_or_else(|| {
      argvus_theme_core::paths::ThemePaths {
        system_root: system_config(),
        user_themes: PathBuf::new(),
        generated: PathBuf::new(),
      }
    });
    if self.active_name_override.is_some() {
      paths.user_themes = PathBuf::new();
    }
    argvus_theme_core::render::render_file(&paths, active, None, "tui").ok()
  }

  /// What identifies the palette on disk: a different stamp means `sources`
  /// would load other colours.
  pub fn stamp(&self) -> ThemeStamp {
    ThemeStamp {
      build: fs::read_link(&self.current_build).ok(),
      active: self.active_name(),
    }
  }
}

/// The identity of the active palette on disk (see [`Loader::stamp`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemeStamp {
  build: Option<PathBuf>,
  active: String,
}

/// Returns whether `name` is an official ARGVUS theme identifier.
///
/// Built-in identifiers are listed in `OFFICIAL_THEMES`. A theme package is
/// official as soon as its manifest is installed, so it needs no entry in this
/// crate.
pub fn is_official_theme(name: &str) -> bool {
  OFFICIAL_THEMES.contains(&name) || manifest_installed_in(&system_config(), name)
}

fn system_config() -> PathBuf {
  std::env::var_os("ARGVUS_SYSTEM_CONFIG")
    .map(PathBuf::from)
    .unwrap_or_else(|| PathBuf::from("/usr/share/argvus"))
}

/// Checks `themes/<family>/theme.toml` under `system_config`. `name` is
/// untrusted (it can come from the greeter or a saved profile), so it is
/// validated as a theme ID before it is used as a path component.
fn manifest_installed_in(system_config: &Path, name: &str) -> bool {
  // Float is a layout mode of the same family; the manifest is keyed by family.
  let family = name.strip_suffix("-float").unwrap_or(name);
  argvus_theme_core::manifest::validate_id(family).is_ok()
    && system_config
      .join("themes")
      .join(family)
      .join("theme.toml")
      .is_file()
}

/// Normalizes untrusted or missing theme state to the safe ARGVUS default.
pub fn normalize_theme_name(name: &str) -> &str {
  let name = name.trim();
  if is_official_theme(name) {
    return name;
  }
  match name {
    "argvus-dark-aether" => "argvus-dark",
    "argvus-dark-aether-float" => "argvus-dark-float",
    "argvus-light-veil" => "argvus-light",
    "argvus-light-veil-float" => "argvus-light-float",
    "argvus-onedark" => "one-dark",
    "argvus-onedark-float" => "one-dark-float",
    "argvus-dark-dracula" => "dracula",
    "argvus-dark-dracula-float" => "dracula-float",
    "argvus-dark-silver" => "silver-dark",
    "argvus-dark-silver-float" => "silver-dark-float",
    "argvus-dark-slate" => "slate-dark",
    "argvus-dark-slate-float" => "slate-dark-float",
    "argvus-dark-universe" => "universe",
    "argvus-dark-universe-float" => "universe-float",
    "argvus-dark-gruvbox-high" => "gruvbox-high-dark",
    "argvus-dark-gruvbox-high-float" => "gruvbox-high-dark-float",
    "argvus-dark-gruvbox" => "gruvbox-dark",
    "argvus-dark-gruvbox-float" => "gruvbox-dark-float",
    "argvus-github-light" => "github-light",
    "argvus-github-light-float" => "github-light-float",
    "argvus-light-solarized" => "solarized-light",
    "argvus-light-solarized-float" => "solarized-light-float",
    "argvus-light-gruvbox" => "gruvbox-light",
    "argvus-light-gruvbox-float" => "gruvbox-light-float",
    "argvus-dark-rose-pine" => "rose-pine",
    "argvus-dark-rose-pine-float" => "rose-pine-float",
    "argvus-dark-tokio-night" => "tokyo-night",
    "argvus-dark-tokio-night-float" => "tokyo-night-float",
    "argvus-dark-solitude" => "solitude",
    "argvus-dark-solitude-float" => "solitude-float",
    "argvus-dark-sunset" => "sunset",
    "argvus-dark-sunset-float" => "sunset-float",
    "argvus-dark-matrix" => "matrix",
    "argvus-dark-matrix-float" => "matrix-float",
    "argvus-dark-monokai" => "monokai-dark",
    "argvus-dark-monokai-float" => "monokai-dark-float",
    "argvus-dark-catppuccin-latte" | "argvus-light-catppuccin-latte" => "catppuccin-latte",
    "argvus-dark-catppuccin-latte-float" | "argvus-light-catppuccin-latte-float" => {
      "catppuccin-latte-float"
    }
    _ => DEFAULT_THEME,
  }
}

impl Default for Loader {
  fn default() -> Self {
    Self::new()
  }
}

fn argvus_config_home() -> PathBuf {
  std::env::var_os("ARGVUS_CONFIG_HOME")
    .map(PathBuf::from)
    .unwrap_or_else(|| {
      std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home().join(".config"))
    })
    .join("argvus")
}

/// Returns the mutable runtime tree inside the ARGVUS configuration root.
/// The root holds exactly two entries: `config/` and `data/`.
fn argvus_data_home() -> PathBuf {
  argvus_config_home().join("data")
}

fn home() -> PathBuf {
  std::env::var_os("HOME")
    .map(PathBuf::from)
    .unwrap_or_else(|| PathBuf::from("/tmp"))
}

fn development_resources_dir() -> PathBuf {
  #[cfg(not(debug_assertions))]
  {
    return PathBuf::from("resources");
  }

  #[cfg(debug_assertions)]
  {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let sources = [
      manifest_dir.join("../../../argvus-appearance/src/usr/share/argvus/appearance/config/tui"),
      manifest_dir.join("../../packaging/archlinux/src/usr/share/argvus/control-center/config"),
    ];
    if let Some(source) = sources.into_iter().find(|candidate| candidate.is_dir()) {
      return source;
    }
    std::env::current_dir()
      .ok()
      .and_then(|dir| {
        [
          dir.join("../argvus-appearance/src/usr/share/argvus/appearance/config/tui"),
          dir.join("src/usr/share/argvus/control-center/config"),
          dir.join("../../packaging/archlinux/src/usr/share/argvus/control-center/config"),
          dir.join("resources"),
          dir.join("argvus-control-center/resources"),
          dir.join("../../resources"),
        ]
        .into_iter()
        .find(|candidate| candidate.is_dir())
      })
      .unwrap_or_else(|| PathBuf::from("resources"))
  }
}

pub fn read_recursive(path: &Path, stack: &mut Vec<PathBuf>) -> Option<String> {
  let contents = fs::read_to_string(path).ok()?;
  let mut output = String::new();
  for line in contents.lines() {
    let trimmed = line.trim();
    let import = trimmed
      .strip_prefix("@import url(")
      .and_then(|value| value.strip_suffix(");").or_else(|| value.strip_suffix(')')))
      .map(|value| value.trim().trim_matches(['\'', '"']))
      .filter(|value| !value.is_empty())
      .map(|value| path.parent().unwrap_or(Path::new(".")).join(value));
    if let Some(import) = import {
      if !stack.contains(&import) {
        stack.push(import.clone());
        if let Some(imported) = read_recursive(&import, stack) {
          output.push_str(&imported);
        }
        stack.pop();
      }
    } else {
      output.push_str(line);
      output.push('\n');
    }
  }
  Some(output)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn a_theme_is_official_only_when_its_manifest_is_installed() {
    // A process-unique directory keeps parallel test runs isolated; the crate
    // has no tempfile dependency, so the directory is removed by hand.
    let system_config =
      std::env::temp_dir().join(format!("argvus-theme-manifest-test-{}", std::process::id()));
    let family_dir = system_config.join("themes/nord-light");
    std::fs::create_dir_all(&family_dir).unwrap();
    std::fs::write(family_dir.join("theme.toml"), "id = \"nord-light\"\n").unwrap();

    assert!(manifest_installed_in(&system_config, "nord-light"));
    assert!(manifest_installed_in(&system_config, "nord-light-float"));
    assert!(!manifest_installed_in(&system_config, "not-installed"));
    // Untrusted names must never escape the themes directory.
    assert!(!manifest_installed_in(&system_config, "../nord-light"));
    assert!(!manifest_installed_in(&system_config, "Nord-Light"));

    std::fs::remove_dir_all(&system_config).unwrap();
  }

  #[test]
  fn official_theme_validation_covers_the_declared_families() {
    assert_eq!(OFFICIAL_THEMES.len(), 42);
    for name in [
      "argvus-dark",
      "one-dark",
      "dracula",
      "silver-dark",
      "universe",
      "gruvbox-high-dark",
      "gruvbox-dark",
      "argvus-light",
      "github-light",
      "solarized-light",
      "gruvbox-light",
      "everforest-light",
      "rose-pine",
      "tokyo-night",
      "solitude",
      "sunset",
      "matrix",
      "monokai-dark",
    ] {
      assert!(is_official_theme(name));
      assert_eq!(normalize_theme_name(name), name);
    }
    assert_eq!(normalize_theme_name("invalid"), DEFAULT_THEME);
  }
}
