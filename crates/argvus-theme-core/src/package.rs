//! A theme as a directory that moves between machines: deriving a new theme
//! from an installed one, checking a theme directory, and installing or
//! removing the user's themes (`$XDG_DATA_HOME/argvus/themes/<id>/`).
//!
//! A theme directory holds `theme.toml`, its `overrides/` and, for a user
//! theme, its wallpaper beside the manifest. The archive format that carries
//! it belongs to the caller.

use crate::build::{self, RenderRequest};
use crate::color::Color;
use crate::layout::{Layout, Variant};
use crate::manifest::{MANIFEST_FILE, ThemeManifest, validate_id};
use crate::paths::ThemePaths;
use crate::registry;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// The `theme.toml` of a new theme made from the theme in `source`: the same
/// palette, ANSI colours, GTK scheme and overrides, under a new `id` and
/// `name`, with `accent` as its highlight colour and `wallpaper` (relative to
/// the new theme directory, or absolute) as its wallpaper.
pub fn derive_manifest(
  source: &Path,
  id: &str,
  name: &str,
  accent: Option<Color>,
  wallpaper: Option<&str>,
) -> Result<String, String> {
  validate_id(id)?;
  let path = source.join(MANIFEST_FILE);
  let text = fs::read_to_string(&path).map_err(|error| format!("{}: {error}", path.display()))?;
  let mut table: toml::Table =
    toml::from_str(&text).map_err(|error| format!("{}: {error}", path.display()))?;
  table.insert("id".into(), toml::Value::String(id.into()));
  table.insert("name".into(), toml::Value::String(name.into()));
  let names = ["en-US", "pt-BR"]
    .into_iter()
    .map(|locale| (locale.to_owned(), toml::Value::String(name.into())))
    .collect();
  table.insert("name_i18n".into(), toml::Value::Table(names));
  match wallpaper {
    Some(wallpaper) => {
      table.insert("wallpaper".into(), toml::Value::String(wallpaper.into()));
    }
    None => {
      table.remove("wallpaper");
    }
  }
  table.remove("preview");
  if let Some(accent) = accent {
    let palette = table
      .get_mut("palette")
      .and_then(toml::Value::as_table_mut)
      .ok_or_else(|| format!("{}: no [palette]", path.display()))?;
    palette.insert("accent".into(), toml::Value::String(accent.hex()));
  }
  let body = toml::to_string(&table).map_err(|error| error.to_string())?;
  let manifest = format!(
    "# ARGVUS theme manifest (schema 2), made from the theme `{}`.\n\
     # The palette is rendered into every component when the theme is applied.\n{body}",
    source
      .file_name()
      .map(|name| name.to_string_lossy().into_owned())
      .unwrap_or_default()
  );
  // The new theme must still parse; a broken source is reported here.
  ThemeManifest::parse(source, &manifest).map_err(|error| error.to_string())?;
  Ok(manifest)
}

/// Checks a theme directory: its manifest, an `id` equal to the directory
/// name, the wallpaper it names, and a render of every registered consumer in
/// both layouts. Returns the manifest, or every problem found.
pub fn check_theme_dir(paths: &ThemePaths, dir: &Path) -> Result<ThemeManifest, String> {
  let manifest = ThemeManifest::load(dir).map_err(|error| error.to_string())?;
  let directory_name = dir.file_name().map(|name| name.to_string_lossy());
  if directory_name.as_deref() != Some(manifest.id.as_str()) {
    return Err(format!(
      "theme `{}` must be in a directory named `{}`",
      manifest.id, manifest.id
    ));
  }
  if let Some(wallpaper) = &manifest.wallpaper
    && !wallpaper.is_file()
  {
    return Err(format!("wallpaper {} is missing", wallpaper.display()));
  }
  let consumers =
    registry::load(&paths.system_root, &paths.registry()).map_err(|error| error.to_string())?;
  let mut problems = Vec::new();
  for variant in [Variant::Sticky, Variant::Float] {
    let layout = Layout::load(&paths.layouts(), variant).map_err(|error| error.to_string())?;
    if let Err(errors) = build::render(&RenderRequest {
      manifest: &manifest,
      variant,
      accent: None,
      layout: layout.as_ref(),
      consumers: &consumers,
    }) {
      problems.extend(
        errors
          .iter()
          .map(|error| format!("{}: {error}", variant.as_str())),
      );
    }
  }
  if problems.is_empty() {
    Ok(manifest)
  } else {
    Err(problems.join("; "))
  }
}

/// Whether an official theme (under the system root) has the id `id`: a user
/// theme of that id would never be used.
pub fn is_official(paths: &ThemePaths, id: &str) -> bool {
  validate_id(id).is_ok() && paths.system_themes().join(id).join(MANIFEST_FILE).is_file()
}

/// Whether the user has a theme of id `id`.
pub fn is_user_theme(paths: &ThemePaths, id: &str) -> bool {
  validate_id(id).is_ok() && paths.user_themes.join(id).join(MANIFEST_FILE).is_file()
}

/// Installs the checked theme directory `source` among the user's themes.
/// An existing user theme of the same id is replaced only with `replace`;
/// an official theme of the same id is refused. The copy is made beside the
/// target and renamed into place, and a replaced theme comes back if that
/// fails.
pub fn install_user_theme(
  paths: &ThemePaths,
  source: &Path,
  replace: bool,
) -> Result<PathBuf, String> {
  let manifest = check_theme_dir(paths, source)?;
  let id = manifest.id;
  if is_official(paths, &id) {
    return Err(format!("an official theme is already named `{id}`"));
  }
  let target = paths.user_themes.join(&id);
  if target.exists() && !(replace && is_user_theme(paths, &id)) {
    return Err(format!("{} already exists", target.display()));
  }
  fs::create_dir_all(&paths.user_themes)
    .map_err(|error| format!("{}: {error}", paths.user_themes.display()))?;
  let pid = std::process::id();
  let staging = paths.user_themes.join(format!(".{id}.tmp-{pid}"));
  let backup = paths.user_themes.join(format!(".{id}.backup-{pid}"));
  let _ = fs::remove_dir_all(&staging);
  copy_tree(source, &staging).map_err(|error| {
    let _ = fs::remove_dir_all(&staging);
    format!("{}: {error}", staging.display())
  })?;
  let replaced = target.exists();
  if replaced {
    fs::rename(&target, &backup).map_err(|error| {
      let _ = fs::remove_dir_all(&staging);
      format!("{}: {error}", target.display())
    })?;
  }
  if let Err(error) = fs::rename(&staging, &target) {
    let _ = fs::remove_dir_all(&staging);
    if replaced {
      let _ = fs::rename(&backup, &target);
    }
    return Err(format!("{}: {error}", target.display()));
  }
  if replaced {
    let _ = fs::remove_dir_all(&backup);
  }
  Ok(target)
}

/// Removes the user's theme `id`. Only a directory with a valid manifest of
/// that id is removed.
pub fn remove_user_theme(paths: &ThemePaths, id: &str) -> Result<(), String> {
  validate_id(id)?;
  let target = paths.user_themes.join(id);
  let manifest = ThemeManifest::load(&target).map_err(|error| error.to_string())?;
  if manifest.id != id {
    return Err(format!("{} is not the theme `{id}`", target.display()));
  }
  fs::remove_dir_all(&target).map_err(|error| format!("{}: {error}", target.display()))
}

/// The id of a theme named `name`: lowercase letters and digits, the rest
/// joined by single hyphens, never ending in `-float`. `None` when nothing is
/// left.
pub fn id_from_name(name: &str) -> Option<String> {
  let mut id = String::new();
  for character in name.trim().chars().flat_map(char::to_lowercase) {
    if character.is_ascii_lowercase() || character.is_ascii_digit() {
      id.push(character);
    } else if !id.is_empty() && !id.ends_with('-') {
      id.push('-');
    }
  }
  let mut id = id.trim_end_matches('-').to_owned();
  while let Some(family) = id.strip_suffix("-float") {
    id = family.to_owned();
  }
  validate_id(&id).ok().map(|()| id)
}

/// Copies a directory tree of regular files; links are refused.
fn copy_tree(source: &Path, destination: &Path) -> io::Result<()> {
  fs::create_dir_all(destination)?;
  for entry in fs::read_dir(source)? {
    let entry = entry?;
    let file_type = entry.file_type()?;
    let target = destination.join(entry.file_name());
    if file_type.is_dir() {
      copy_tree(&entry.path(), &target)?;
    } else if file_type.is_file() {
      fs::copy(entry.path(), &target)?;
    } else {
      return Err(io::Error::new(
        io::ErrorKind::InvalidData,
        format!("{} is not a regular file", entry.path().display()),
      ));
    }
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

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
      &paths.system_themes().join("sunset/theme.toml"),
      "schema = 2\nid = \"sunset\"\nname = \"Sunset\"\ncategory = \"dark\"\n\
       [name_i18n]\npt-BR = \"Pôr do sol\"\n\
       [palette]\nbg = \"#000000\"\nfg = \"#FFFFFF\"\naccent = \"#E2BE8A\"\nline = \"@accent | alpha(0.5)\"\n\
       [overrides]\nterminal = \"overrides/terminal.conf\"\n",
    );
    write(
      &paths.system_themes().join("sunset/overrides/terminal.conf"),
      "fg {{ @fg }}\n",
    );
    write(&paths.layouts().join("sticky.toml"), "gap = 1\n");
    write(&paths.layouts().join("float.toml"), "gap = 8\n");
    write(
      &paths.system_root.join("t/a.txt"),
      "{{ @line }} {{ layout.gap }}\n",
    );
    write(
      &paths.registry().join("a.toml"),
      "id = \"terminal\"\ntemplate = \"t/a.txt\"\noutput = \"a.txt\"\n",
    );
    (root, paths)
  }

  /// A theme directory made from sunset, ready to install.
  fn derived(root: &Path, paths: &ThemePaths, id: &str) -> PathBuf {
    let dir = root.join("staging").join(id);
    let manifest = derive_manifest(
      &paths.system_themes().join("sunset"),
      id,
      "My Sunset",
      Color::parse("#112233"),
      Some("wallpaper.png"),
    )
    .unwrap();
    write(&dir.join("theme.toml"), &manifest);
    write(&dir.join("overrides/terminal.conf"), "fg {{ @fg }}\n");
    write(&dir.join("wallpaper.png"), "png");
    dir
  }

  #[test]
  fn a_derived_theme_keeps_the_palette_with_the_new_identity() {
    let (root, paths) = tree();
    let dir = derived(root.path(), &paths, "my-sunset");
    let manifest = ThemeManifest::load(&dir).unwrap();
    assert_eq!(manifest.id, "my-sunset");
    assert_eq!(manifest.name, "My Sunset");
    assert_eq!(manifest.name_i18n["pt-BR"], "My Sunset");
    assert_eq!(manifest.wallpaper, Some(dir.join("wallpaper.png")));
    assert_eq!(manifest.token("accent").unwrap().hex(), "#112233");
    assert_eq!(
      manifest.token("line").unwrap().hex_with_alpha(),
      "#11223380"
    );
    assert!(manifest.override_for("terminal").is_some());
    assert!(
      derive_manifest(
        &paths.system_themes().join("sunset"),
        "x-float",
        "x",
        None,
        None
      )
      .is_err()
    );
  }

  #[test]
  fn install_replace_and_remove_a_user_theme() {
    let (root, paths) = tree();
    let dir = derived(root.path(), &paths, "my-sunset");
    let target = install_user_theme(&paths, &dir, false).unwrap();
    assert!(is_user_theme(&paths, "my-sunset"));
    assert!(target.join("wallpaper.png").is_file());

    assert!(
      install_user_theme(&paths, &dir, false)
        .unwrap_err()
        .contains("already exists")
    );
    fs::write(target.join("extra"), "old").unwrap();
    install_user_theme(&paths, &dir, true).unwrap();
    assert!(!target.join("extra").exists(), "replaced, not merged");
    assert_eq!(
      fs::read_dir(&paths.user_themes).unwrap().count(),
      1,
      "no leftovers"
    );

    remove_user_theme(&paths, "my-sunset").unwrap();
    assert!(!target.exists());
    assert!(remove_user_theme(&paths, "my-sunset").is_err());
  }

  #[test]
  fn broken_or_conflicting_themes_are_refused() {
    let (root, paths) = tree();
    let official = derived(root.path(), &paths, "sunset");
    assert!(
      install_user_theme(&paths, &official, false)
        .unwrap_err()
        .contains("official")
    );

    let renamed = root.path().join("staging/other");
    fs::rename(derived(root.path(), &paths, "mine"), &renamed).unwrap();
    assert!(
      check_theme_dir(&paths, &renamed)
        .unwrap_err()
        .contains("directory named")
    );

    let no_wallpaper = derived(root.path(), &paths, "bare");
    fs::remove_file(no_wallpaper.join("wallpaper.png")).unwrap();
    assert!(
      check_theme_dir(&paths, &no_wallpaper)
        .unwrap_err()
        .contains("wallpaper")
    );

    let broken = derived(root.path(), &paths, "broken");
    fs::write(broken.join("overrides/terminal.conf"), "{{ @nope }}\n").unwrap();
    assert!(check_theme_dir(&paths, &broken).is_err());
    assert!(!paths.user_themes.join("broken").exists());
  }

  #[test]
  fn ids_come_from_names() {
    assert_eq!(id_from_name("Ocean Night").as_deref(), Some("ocean-night"));
    assert_eq!(id_from_name("  Meu_Tema 2 ").as_deref(), Some("meu-tema-2"));
    assert_eq!(id_from_name("Sunset Float").as_deref(), Some("sunset"));
    assert_eq!(
      id_from_name("Ção!").as_deref(),
      Some("o"),
      "letters outside a-z are dropped"
    );
    assert_eq!(id_from_name("!!!"), None);
  }
}
