//! Catalog and selection: previewing never writes configuration.
use argvus_theme_core::{
  discovery::{self, Source},
  manifest::ThemeManifest,
  paths::ThemePaths,
  registry,
};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Filter {
  #[default]
  All,
  Dark,
  Light,
  Custom,
}

#[derive(Default)]
pub struct Catalog {
  pub themes: Vec<ThemeManifest>,
  pub sources: HashMap<String, Source>,
  pub visible: Vec<usize>,
  pub selected: usize,
  pub active: String,
}

impl Catalog {
  pub fn load(locale: &str, active: String) -> Result<Self, String> {
    let paths = ThemePaths::from_environment().ok_or("cannot resolve theme paths")?;
    Self::load_at(&paths, locale, active)
  }

  fn load_at(paths: &ThemePaths, locale: &str, active: String) -> Result<Self, String> {
    let discovered = discovery::discover(paths);
    for warning in discovered.warnings {
      eprintln!("argvus-theme-selector: {warning}");
    }
    let consumers =
      registry::load(&paths.system_root, &paths.registry()).map_err(|error| error.to_string())?;
    let defaults = registry::palette_defaults(&consumers);
    let mut sources = HashMap::new();
    let mut themes: Vec<_> = discovered
      .themes
      .into_iter()
      .filter_map(
        |theme| match theme.manifest.with_palette_defaults(&defaults) {
          Ok(manifest) if manifest.resolve_palette(None).is_ok() => {
            sources.insert(manifest.id.clone(), theme.source);
            Some(manifest)
          }
          Ok(_) => {
            eprintln!(
              "argvus-theme-selector: invalid palette: {}",
              theme.manifest.id
            );
            None
          }
          Err(error) => {
            eprintln!("argvus-theme-selector: {error}");
            None
          }
        },
      )
      .collect();
    themes.sort_by_key(|theme| {
      (
        !matches!(theme.id.as_str(), "argvus-dark" | "argvus-light"),
        name(theme, locale).to_lowercase(),
        theme.id.clone(),
      )
    });
    let selected = themes
      .iter()
      .position(|theme| theme.id == active)
      .unwrap_or(0);
    Ok(Self {
      visible: (0..themes.len()).collect(),
      themes,
      sources,
      selected,
      active,
    })
  }

  pub fn current(&self) -> Option<&ThemeManifest> {
    self
      .visible
      .get(self.selected)
      .and_then(|index| self.themes.get(*index))
  }

  pub fn filter(&mut self, query: &str, filter: Filter, locale: &str) {
    let previous = self.current().map(|theme| theme.id.clone());
    let query = query.to_lowercase();
    self.visible = self
      .themes
      .iter()
      .enumerate()
      .filter(|(_, theme)| {
        (match filter {
          Filter::All => true,
          Filter::Dark => {
            theme.category.as_str() == "dark"
              && self.sources.get(&theme.id) == Some(&Source::System)
          }
          Filter::Light => {
            theme.category.as_str() == "light"
              && self.sources.get(&theme.id) == Some(&Source::System)
          }
          Filter::Custom => self.sources.get(&theme.id) == Some(&Source::User),
        }) && name(theme, locale).to_lowercase().contains(&query)
      })
      .map(|(index, _)| index)
      .collect();
    self.selected = self
      .visible
      .iter()
      .position(|index| Some(&self.themes[*index].id) == previous.as_ref())
      .unwrap_or(0);
  }

  /// Current theme first, then unique neighbors; used to bound image work and memory.
  pub fn nearby(&self) -> Vec<&ThemeManifest> {
    if self.visible.is_empty() {
      return Vec::new();
    }
    let mut indices = Vec::new();
    for offset in [0, -1, 1] {
      let index =
        (self.selected as isize + offset).rem_euclid(self.visible.len() as isize) as usize;
      if !indices.contains(&index) {
        indices.push(index);
      }
    }
    indices
      .into_iter()
      .map(|index| &self.themes[self.visible[index]])
      .collect()
  }

  pub fn step(&mut self, offset: isize) {
    if !self.visible.is_empty() {
      self.selected =
        (self.selected as isize + offset).rem_euclid(self.visible.len() as isize) as usize;
    }
  }
}

pub fn name<'a>(theme: &'a ThemeManifest, locale: &str) -> &'a str {
  theme
    .name_i18n
    .get(locale)
    .or_else(|| theme.name_i18n.get("en-US"))
    .map(String::as_str)
    .unwrap_or(&theme.name)
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn navigation_wraps_and_handles_empty_catalog() {
    let mut catalog = Catalog::default();
    catalog.step(-1);
    assert_eq!(catalog.selected, 0);
    catalog.visible = vec![0, 1, 2];
    catalog.step(-1);
    assert_eq!(catalog.selected, 2);
    catalog.step(1);
    assert_eq!(catalog.selected, 0);
  }
}

#[cfg(test)]
mod catalog_tests {
  use super::*;
  use std::fs;

  fn fixture(root: &std::path::Path, id: &str, label: &str, category: &str) {
    let dir = root.join(id);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("theme.toml"), format!("schema = 2\nid = \"{id}\"\nname = \"{label}\"\ncategory = \"{category}\"\n[palette]\nbg = \"#112233\"\nfg = \"#eeeeee\"\naccent = \"#3590bd\"\n")).unwrap();
  }

  #[test]
  fn catalog_orders_builtins_preserves_ids_and_filters_without_writes() {
    fs::create_dir_all("/tmp/argvus").unwrap();
    let temp = tempfile::tempdir_in("/tmp/argvus").unwrap();
    let paths = ThemePaths {
      system_root: temp.path().join("system"),
      user_themes: temp.path().join("user"),
      generated: temp.path().join("generated"),
    };
    fs::create_dir_all(paths.registry()).unwrap();
    fixture(&paths.system_themes(), "argvus-dark", "ARGVUS Dark", "dark");
    fixture(&paths.system_themes(), "first", "Same name", "dark");
    fixture(&paths.user_themes, "second", "Same name", "light");
    fixture(&paths.user_themes, "third", "Custom dark", "dark");
    fixture(&paths.user_themes, "first", "Shadow", "light");
    let mut catalog = Catalog::load_at(&paths, "pt-BR", "second".into()).unwrap();
    assert_eq!(catalog.themes[0].id, "argvus-dark");
    assert_eq!(catalog.current().unwrap().id, "second");
    assert_eq!(catalog.themes.len(), 4);
    catalog.filter("Same", Filter::All, "pt-BR");
    assert_eq!(catalog.current().unwrap().id, "second");
    catalog.filter("Same", Filter::Dark, "pt-BR");
    assert_eq!(catalog.current().unwrap().id, "first");
    assert_eq!(catalog.nearby().len(), 1);
    catalog.filter("", Filter::Custom, "pt-BR");
    assert_eq!(catalog.visible.len(), 2);
    assert!(
      catalog
        .visible
        .iter()
        .all(|index| { catalog.sources.get(&catalog.themes[*index].id) == Some(&Source::User) })
    );
    catalog.filter("Custom", Filter::Custom, "pt-BR");
    assert_eq!(catalog.current().unwrap().id, "third");
    catalog.filter("missing", Filter::All, "pt-BR");
    assert!(catalog.current().is_none());
    assert!(catalog.nearby().is_empty());
    catalog.filter("", Filter::All, "pt-BR");
    assert_eq!(catalog.nearby().len(), 3);
    assert!(!paths.generated.exists());
  }
}
