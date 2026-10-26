//! Finding installed themes: the official packages under the system root and
//! the user's own under `$XDG_DATA_HOME/argvus/themes`. An official theme wins
//! over a user theme with the same id, so official ids keep their meaning.

use crate::error::ThemeError;
use crate::manifest::{MANIFEST_FILE, ThemeManifest};
use crate::paths::ThemePaths;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
  System,
  User,
}

impl Source {
  pub fn as_str(self) -> &'static str {
    match self {
      Self::System => "system",
      Self::User => "user",
    }
  }
}

#[derive(Debug, Clone)]
pub struct InstalledTheme {
  pub manifest: ThemeManifest,
  pub source: Source,
}

#[derive(Debug, Default)]
pub struct Discovery {
  /// Sorted by id.
  pub themes: Vec<InstalledTheme>,
  /// Manifests that were skipped, with the reason.
  pub warnings: Vec<String>,
}

impl Discovery {
  pub fn find(&self, id: &str) -> Option<&InstalledTheme> {
    self.themes.iter().find(|theme| theme.manifest.id == id)
  }
}

pub fn discover(paths: &ThemePaths) -> Discovery {
  let mut discovery = Discovery::default();
  for (directory, source) in [
    (paths.system_themes(), Source::System),
    (paths.user_themes.clone(), Source::User),
  ] {
    scan(&directory, source, &mut discovery);
  }
  discovery
    .themes
    .sort_by(|left, right| left.manifest.id.cmp(&right.manifest.id));
  discovery
}

fn scan(directory: &Path, source: Source, discovery: &mut Discovery) {
  let Ok(entries) = fs::read_dir(directory) else {
    return;
  };
  let mut theme_dirs: Vec<_> = entries
    .flatten()
    .map(|entry| entry.path())
    .filter(|path| path.join(MANIFEST_FILE).is_file())
    .collect();
  theme_dirs.sort();
  for theme_dir in theme_dirs {
    match load_named(&theme_dir) {
      Ok(manifest) => {
        if let Some(existing) = discovery.find(&manifest.id) {
          discovery.warnings.push(format!(
            "{}: theme `{}` is already installed by the {} themes",
            theme_dir.display(),
            manifest.id,
            existing.source.as_str()
          ));
        } else {
          discovery.themes.push(InstalledTheme { manifest, source });
        }
      }
      Err(error) => discovery.warnings.push(error.to_string()),
    }
  }
}

/// The manifest of one installed theme, by its id (without `-float`): the
/// official one, else the user's. An invalid id or a broken manifest is `None`.
pub fn find(paths: &ThemePaths, id: &str) -> Option<ThemeManifest> {
  crate::manifest::validate_id(id).ok()?;
  [paths.system_themes(), paths.user_themes.clone()]
    .into_iter()
    .find_map(|root| load_named(&root.join(id)).ok())
}

/// Loads a manifest whose directory name must be its id, so the id in a path
/// and the id in the file cannot disagree.
fn load_named(theme_dir: &Path) -> Result<ThemeManifest, ThemeError> {
  let manifest = ThemeManifest::load(theme_dir)?;
  let directory_name = theme_dir.file_name().map(|name| name.to_string_lossy());
  if directory_name.as_deref() != Some(manifest.id.as_str()) {
    return Err(ThemeError::invalid(
      &theme_dir.join(MANIFEST_FILE),
      format!("id `{}` does not match its directory name", manifest.id),
    ));
  }
  Ok(manifest)
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::path::PathBuf;

  fn write_theme(root: &Path, directory: &str, id: &str) {
    let theme_dir = root.join(directory);
    fs::create_dir_all(&theme_dir).unwrap();
    fs::write(
      theme_dir.join(MANIFEST_FILE),
      format!(
        "schema = 2\nid = \"{id}\"\nname = \"{id}\"\ncategory = \"dark\"\n\
         [palette]\nbg = \"#000000\"\nfg = \"#FFFFFF\"\naccent = \"#3590BD\"\n"
      ),
    )
    .unwrap();
  }

  #[test]
  fn official_themes_win_and_problems_become_warnings() {
    let root = tempfile::tempdir().unwrap();
    let paths = ThemePaths {
      system_root: root.path().join("system"),
      user_themes: root.path().join("user"),
      generated: PathBuf::from("/unused"),
    };
    write_theme(&paths.system_themes(), "sunset", "sunset");
    write_theme(&paths.user_themes, "sunset", "sunset");
    write_theme(&paths.user_themes, "mine", "mine");
    write_theme(&paths.user_themes, "renamed", "other");
    fs::create_dir_all(paths.user_themes.join("broken")).unwrap();
    fs::write(
      paths.user_themes.join("broken").join(MANIFEST_FILE),
      "schema = 1",
    )
    .unwrap();

    let discovery = discover(&paths);
    let installed: Vec<_> = discovery
      .themes
      .iter()
      .map(|theme| (theme.manifest.id.as_str(), theme.source))
      .collect();
    assert_eq!(
      installed,
      [("mine", Source::User), ("sunset", Source::System)]
    );
    assert_eq!(discovery.warnings.len(), 3, "{:?}", discovery.warnings);

    assert_eq!(
      find(&paths, "sunset").map(|manifest| manifest.dir),
      Some(paths.system_themes().join("sunset"))
    );
    assert!(find(&paths, "mine").is_some());
    assert!(
      find(&paths, "renamed").is_none(),
      "the id must match the directory"
    );
    assert!(find(&paths, "../sunset").is_none());
  }
}
