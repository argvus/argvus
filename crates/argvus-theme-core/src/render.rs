//! Rendering an installed theme by its id: what a theme switch writes into a
//! build, and what a reader that has no build renders in memory (the greeter,
//! which cannot read the user's files, or a session before its first switch).

use crate::build::{self, RenderRequest, Rendered};
use crate::color::Color;
use crate::discovery;
use crate::layout::{Layout, Variant};
use crate::paths::ThemePaths;
use crate::registry;

/// The family and the layout of `<id>` or `<id>-float`.
pub fn split_variant(theme: &str) -> (&str, Variant) {
  match theme.strip_suffix("-float") {
    Some(family) => (family, Variant::Float),
    None => (theme, Variant::Sticky),
  }
}

/// Renders `theme` (`<id>` or `<id>-float`) with `accent` (the theme's own when
/// `None`). `only` limits the render to one consumer of the registry. The error
/// is a readable description of everything that went wrong.
pub fn render_installed(
  paths: &ThemePaths,
  theme: &str,
  accent: Option<Color>,
  only: Option<&str>,
) -> Result<Rendered, String> {
  let (family, variant) = split_variant(theme);
  let discovery = discovery::discover(paths);
  let installed = discovery.find(family).ok_or_else(|| {
    format!(
      "theme `{family}` is not installed (no theme.toml under {} or {})",
      paths.system_themes().display(),
      paths.user_themes.display()
    )
  })?;
  let mut consumers =
    registry::load(&paths.system_root, &paths.registry()).map_err(|error| error.to_string())?;
  if let Some(only) = only {
    consumers.retain(|consumer| consumer.id == only);
    if consumers.is_empty() {
      return Err(format!("no consumer `{only}` is registered"));
    }
  }
  let layout = Layout::load(&paths.layouts(), variant).map_err(|error| error.to_string())?;
  build::render(&RenderRequest {
    manifest: &installed.manifest,
    variant,
    accent,
    layout: layout.as_ref(),
    consumers: &consumers,
  })
  .map_err(|errors| {
    errors
      .iter()
      .map(ToString::to_string)
      .collect::<Vec<_>>()
      .join("; ")
  })
}

/// The rendered text of one consumer of `theme`, in memory.
pub fn render_file(
  paths: &ThemePaths,
  theme: &str,
  accent: Option<Color>,
  consumer: &str,
) -> Result<String, String> {
  let rendered = render_installed(paths, theme, accent, Some(consumer))?;
  rendered
    .files
    .into_iter()
    .next()
    .map(|file| file.contents)
    .ok_or_else(|| format!("consumer `{consumer}` rendered nothing"))
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::fs;
  use std::path::Path;

  fn write(path: &Path, text: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
  }

  fn tree() -> (tempfile::TempDir, ThemePaths) {
    let root = tempfile::tempdir().unwrap();
    let paths = ThemePaths {
      system_root: root.path().join("system"),
      user_themes: root.path().join("user"),
      generated: root.path().join("generated"),
    };
    write(
      &paths.system_themes().join("demo/theme.toml"),
      "schema = 2\nid = \"demo\"\nname = \"Demo\"\ncategory = \"dark\"\n\
       [palette]\nbg = \"#000000\"\nfg = \"#FFFFFF\"\naccent = \"#3590BD\"\n",
    );
    write(&paths.layouts().join("float.toml"), "rounding = 8\n");
    write(
      &paths.system_root.join("tui/t.css"),
      "{{ @card | hex }} {{ layout.rounding }}\n",
    );
    write(
      &paths.registry().join("tui.toml"),
      "id = \"tui\"\ntemplate = \"tui/t.css\"\noutput = \"tui/theme.css\"\n\
       [palette]\ncard = \"@bg | mix(@fg, 0.5)\"\n",
    );
    write(&paths.system_root.join("other/o.txt"), "{{ @missing }}\n");
    write(
      &paths.registry().join("other.toml"),
      "id = \"other\"\ntemplate = \"other/o.txt\"\noutput = \"other/o.txt\"\n",
    );
    (root, paths)
  }

  #[test]
  fn one_consumer_renders_with_its_palette_defaults_and_the_layout() {
    let (_root, paths) = tree();
    let text = render_file(&paths, "demo-float", None, "tui").unwrap();
    assert_eq!(text, "#808080 8\n");
    // The broken consumer is not rendered when another one is asked for.
    assert!(render_installed(&paths, "demo", None, None).is_err());
  }

  #[test]
  fn an_unknown_theme_or_consumer_is_an_error() {
    let (_root, paths) = tree();
    assert!(
      render_file(&paths, "nope", None, "tui")
        .unwrap_err()
        .contains("not installed")
    );
    assert!(
      render_file(&paths, "demo", None, "nope")
        .unwrap_err()
        .contains("no consumer")
    );
  }

  #[test]
  fn a_theme_entry_wins_over_a_registry_default() {
    let (_root, paths) = tree();
    write(
      &paths.system_themes().join("demo/theme.toml"),
      "schema = 2\nid = \"demo\"\nname = \"Demo\"\ncategory = \"dark\"\n\
       [palette]\nbg = \"#000000\"\nfg = \"#FFFFFF\"\naccent = \"#3590BD\"\ncard = \"#123456\"\n",
    );
    assert_eq!(
      render_file(&paths, "demo-float", None, "tui").unwrap(),
      "#123456 8\n"
    );
  }
}
