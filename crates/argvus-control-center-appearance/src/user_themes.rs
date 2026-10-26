//! The user's own themes: the themes of `$XDG_DATA_HOME/argvus/themes` (schema
//! 2, the format of the official packages and of the theme skeleton), exported
//! from the active theme and imported from an archive.
//!
//! An exported archive holds one theme directory, `<id>/theme.toml` with its
//! `overrides/` and its wallpaper: the layout of a user theme, so extracting it
//! into `~/.local/share/argvus/themes` installs it as well. Applying a user
//! theme is an ordinary theme switch.
//!
//! The appearance profiles of earlier versions (`<id>/profile.tar.gz` in the
//! same directory) are converted once into themes and moved to a backup.

use crate::model::{CustomTheme, canonical_theme_id};
use argvus_theme_core::color::Color;
use argvus_theme_core::discovery::{self, Source};
use argvus_theme_core::package;
use argvus_theme_core::paths::ThemePaths;
use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use serde_json::Value;
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tar::{Archive, Builder, EntryType, Header};
use tempfile::{TempDir, tempdir};

const MAX_MEMBERS: usize = 128;
const MAX_FILE: u64 = 8 * 1024 * 1024;
const MAX_WALLPAPER: u64 = 512 * 1024 * 1024;
const MAX_TOTAL: u64 = 520 * 1024 * 1024;
const MAX_NAME: usize = 120;
const WALLPAPER_EXTENSIONS: [&str; 5] = ["jxl", "png", "jpg", "jpeg", "webp"];

/// The theme roots of the session.
fn theme_paths() -> Result<ThemePaths, String> {
  ThemePaths::from_environment().ok_or_else(|| "cannot resolve the theme directories".to_string())
}

/// The user's themes, by name.
pub fn custom_themes() -> Vec<CustomTheme> {
  let Ok(paths) = theme_paths() else {
    return Vec::new();
  };
  let mut themes: Vec<CustomTheme> = discovery::discover(&paths)
    .themes
    .into_iter()
    .filter(|theme| theme.source == Source::User)
    .map(|theme| CustomTheme {
      id: theme.manifest.id,
      name: theme.manifest.name,
    })
    .collect();
  themes.sort_by_key(|theme| theme.name.to_lowercase());
  themes
}

/// The active theme, when it is one of the user's.
pub fn active_custom_theme() -> Option<String> {
  let active = fs::read_to_string(argvus_data_home().join(".active-theme")).ok()?;
  let active = canonical_theme_id(active.trim());
  let family = active.strip_suffix("-float").unwrap_or(&active).to_owned();
  custom_themes()
    .iter()
    .any(|theme| theme.id == family)
    .then_some(family)
}

/// Applies a user theme. Its highlight colour is in its palette.
pub fn apply_custom_theme(theme: &CustomTheme) -> Result<(), String> {
  crate::backend::set_theme(&theme.id)
}

/// Removes a user theme, after switching to ARGVUS Dark when it is active.
pub fn delete_custom_theme(theme: &CustomTheme) -> Result<(), String> {
  if active_custom_theme().as_deref() == Some(theme.id.as_str()) {
    crate::backend::set_theme("argvus-dark")?;
  }
  package::remove_user_theme(&theme_paths()?, &theme.id)
}

/// The theme name typed by the user, trimmed.
fn clean_name(name: &str) -> Result<String, String> {
  let name = name.trim();
  if name.is_empty() || name.chars().count() > MAX_NAME || name.chars().any(char::is_control) {
    return Err("invalid theme name".into());
  }
  Ok(name.to_owned())
}

/// The id and the archive of a theme exported as `name`: the first id free
/// among the installed themes and the archives in the home directory.
fn export_target(paths: &ThemePaths, name: &str) -> Result<(String, PathBuf), String> {
  let base = package::id_from_name(name).ok_or("the name has no letter or digit")?;
  let home = argvus_control_center_core::paths::home();
  let taken = |id: &str| {
    package::is_official(paths, id)
      || paths.user_themes.join(id).exists()
      || home.join(format!("argvus-theme-{id}.tar.gz")).exists()
  };
  let mut id = base.clone();
  let mut suffix = 2;
  while taken(&id) {
    id = format!("{base}-{suffix}");
    suffix += 1;
  }
  let archive = home.join(format!("argvus-theme-{id}.tar.gz"));
  Ok((id, archive))
}

/// Where `export_named(name)` would write, for the prompt.
pub fn preview_export_path(name: &str) -> Option<PathBuf> {
  let name = clean_name(name).ok()?;
  export_target(&theme_paths().ok()?, &name)
    .ok()
    .map(|(_, archive)| archive)
}

/// Exports the active theme, with the active highlight colour and wallpaper,
/// as the new theme `name`. Returns the archive written in the home directory.
pub fn export_named(name: &str) -> Result<PathBuf, String> {
  let name = clean_name(name)?;
  let paths = theme_paths()?;
  let (id, archive) = export_target(&paths, &name)?;
  let active = fs::read_to_string(argvus_data_home().join(".active-theme"))
    .map(|text| canonical_theme_id(text.trim()))
    .unwrap_or_else(|_| "argvus-dark".into());
  let family = active.strip_suffix("-float").unwrap_or(&active);
  let source = discovery::find(&paths, family)
    .ok_or_else(|| format!("the active theme `{family}` is not installed"))?;
  let accent = fs::read_to_string(argvus_data_home().join(".accent-color"))
    .ok()
    .and_then(|text| Color::parse(text.trim()));
  let wallpaper = current_wallpaper().or(source.wallpaper.clone());

  let staging = tempdir().map_err(|error| error.to_string())?;
  let dir = staging.path().join(&id);
  write_theme_dir(&dir, &source.dir, &id, &name, accent, wallpaper.as_deref())?;
  package::check_theme_dir(&paths, &dir)?;
  write_archive(&dir, &id, &archive)?;
  Ok(archive)
}

/// Writes the theme `id` made from the theme in `source`: its manifest, its
/// overrides and the wallpaper, beside the manifest.
fn write_theme_dir(
  dir: &Path,
  source: &Path,
  id: &str,
  name: &str,
  accent: Option<Color>,
  wallpaper: Option<&Path>,
) -> Result<(), String> {
  fs::create_dir_all(dir).map_err(|error| error.to_string())?;
  let wallpaper_name = wallpaper.and_then(|path| {
    let extension = path.extension()?.to_str()?.to_ascii_lowercase();
    WALLPAPER_EXTENSIONS
      .contains(&extension.as_str())
      .then(|| format!("wallpaper.{extension}"))
  });
  let manifest = package::derive_manifest(source, id, name, accent, wallpaper_name.as_deref())?;
  fs::write(dir.join("theme.toml"), manifest).map_err(|error| error.to_string())?;
  let overrides = source.join("overrides");
  if overrides.is_dir() {
    copy_files(&overrides, &dir.join("overrides")).map_err(|error| error.to_string())?;
  }
  if let (Some(wallpaper), Some(wallpaper_name)) = (wallpaper, wallpaper_name) {
    fs::copy(wallpaper, dir.join(wallpaper_name))
      .map_err(|error| format!("{}: {error}", wallpaper.display()))?;
  }
  Ok(())
}

/// The wallpaper on screen: a custom choice, else the one hyprpaper shows.
fn current_wallpaper() -> Option<PathBuf> {
  let custom = fs::read_to_string(argvus_data_home().join(".wallpaper-custom"))
    .ok()
    .map(|value| PathBuf::from(value.trim()))
    .filter(|path| path.is_file());
  custom.or_else(|| {
    let config = fs::read_to_string(argvus_data_home().join("hypr/hyprpaper.conf")).ok()?;
    config.lines().find_map(|line| {
      let (key, value) = line.split_once('=')?;
      if key.trim() != "path" {
        return None;
      }
      let value = value.trim();
      let path = match value.strip_prefix("~/") {
        Some(rest) => argvus_control_center_core::paths::home().join(rest),
        None => PathBuf::from(value),
      };
      path.is_file().then_some(path)
    })
  })
}

/// Writes the theme directory `dir` as `<id>/…` in a gzip tar archive, through
/// a file beside it and a rename.
fn write_archive(dir: &Path, id: &str, archive: &Path) -> Result<(), String> {
  let temporary = archive.with_extension(format!("gz.tmp-{}", std::process::id()));
  let written = (|| -> io::Result<()> {
    let file = File::create(&temporary)?;
    let mut builder = Builder::new(GzEncoder::new(file, Compression::default()));
    append_tree(&mut builder, dir, Path::new(id))?;
    builder.into_inner()?.finish()?.sync_all()
  })();
  if let Err(error) = written.and_then(|()| fs::rename(&temporary, archive)) {
    let _ = fs::remove_file(&temporary);
    return Err(format!("{}: {error}", archive.display()));
  }
  Ok(())
}

fn append_tree<W: io::Write>(builder: &mut Builder<W>, dir: &Path, name: &Path) -> io::Result<()> {
  let mut header = Header::new_gnu();
  header.set_entry_type(EntryType::Directory);
  header.set_mode(0o755);
  header.set_size(0);
  header.set_mtime(0);
  builder.append_data(&mut header, name, io::empty())?;
  let mut entries: Vec<_> = fs::read_dir(dir)?.collect::<Result<_, _>>()?;
  entries.sort_by_key(fs::DirEntry::file_name);
  for entry in entries {
    let path = entry.path();
    let member = name.join(entry.file_name());
    if entry.file_type()?.is_dir() {
      append_tree(builder, &path, &member)?;
    } else {
      let mut header = Header::new_gnu();
      header.set_entry_type(EntryType::Regular);
      header.set_mode(0o644);
      header.set_size(entry.metadata()?.len());
      header.set_mtime(0);
      builder.append_data(&mut header, &member, File::open(&path)?)?;
    }
  }
  Ok(())
}

fn copy_files(source: &Path, destination: &Path) -> io::Result<()> {
  fs::create_dir_all(destination)?;
  for entry in fs::read_dir(source)? {
    let entry = entry?;
    let target = destination.join(entry.file_name());
    if entry.file_type()?.is_dir() {
      copy_files(&entry.path(), &target)?;
    } else if entry.file_type()?.is_file() {
      fs::copy(entry.path(), target)?;
    }
  }
  Ok(())
}

/// The `.tar.gz` files of the home directory that hold a theme.
pub fn list_import_archives() -> Vec<PathBuf> {
  let mut files: Vec<PathBuf> = fs::read_dir(argvus_control_center_core::paths::home())
    .into_iter()
    .flatten()
    .filter_map(Result::ok)
    .map(|entry| entry.path())
    .filter(|path| {
      path.is_file()
        && path
          .file_name()
          .and_then(|name| name.to_str())
          .is_some_and(|name| name.ends_with(".tar.gz"))
        && holds_a_theme(path)
    })
    .collect();
  files.sort();
  files
}

/// Whether an archive has `<directory>/theme.toml` among its first members.
fn holds_a_theme(path: &Path) -> bool {
  let Ok(file) = File::open(path) else {
    return false;
  };
  let mut archive = Archive::new(GzDecoder::new(file));
  let Ok(entries) = archive.entries() else {
    return false;
  };
  entries.take(MAX_MEMBERS).flatten().any(|entry| {
    entry.path().is_ok_and(|member| {
      let parts: Vec<_> = member.components().collect();
      parts.len() == 2 && parts[1].as_os_str() == "theme.toml"
    })
  })
}

/// A theme extracted from an archive into a temporary directory.
#[derive(Debug)]
struct Staged {
  _temp: TempDir,
  dir: PathBuf,
}

/// Extracts the theme of an archive after checking every member: one top
/// directory, relative paths without `..`, regular files and directories only,
/// and size limits.
fn stage(path: &Path) -> Result<Staged, String> {
  let temp = tempdir().map_err(|error| error.to_string())?;
  let file = File::open(path).map_err(|error| format!("{}: {error}", path.display()))?;
  let mut archive = Archive::new(GzDecoder::new(file));
  let mut top: Option<PathBuf> = None;
  let mut total = 0u64;
  let entries = archive.entries().map_err(|error| error.to_string())?;
  for (index, entry) in entries.enumerate() {
    if index >= MAX_MEMBERS {
      return Err("the archive has too many members".into());
    }
    let mut entry = entry.map_err(|error| error.to_string())?;
    let member = entry
      .path()
      .map_err(|error| error.to_string())?
      .into_owned();
    if !member
      .components()
      .all(|component| matches!(component, Component::Normal(_)))
    {
      return Err(format!("unsafe member {}", member.display()));
    }
    let first = member
      .components()
      .next()
      .map(|component| PathBuf::from(component.as_os_str()))
      .ok_or("empty member")?;
    if top.get_or_insert_with(|| first.clone()) != &first {
      return Err("the archive must hold a single theme directory".into());
    }
    let target = temp.path().join(&member);
    match entry.header().entry_type() {
      EntryType::Directory => fs::create_dir_all(&target).map_err(|error| error.to_string())?,
      EntryType::Regular => {
        let size = entry.header().size().map_err(|error| error.to_string())?;
        let limit = if is_image(&member) {
          MAX_WALLPAPER
        } else {
          MAX_FILE
        };
        total += size;
        if size > limit || total > MAX_TOTAL {
          return Err(format!("{} is too large", member.display()));
        }
        if let Some(parent) = target.parent() {
          fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        let mut output = File::create(&target).map_err(|error| error.to_string())?;
        io::copy(&mut (&mut entry).take(limit), &mut output).map_err(|error| error.to_string())?;
      }
      _ => return Err(format!("{} is not a regular file", member.display())),
    }
  }
  let top = top.ok_or("the archive is empty")?;
  Ok(Staged {
    dir: temp.path().join(top),
    _temp: temp,
  })
}

fn is_image(member: &Path) -> bool {
  member
    .extension()
    .and_then(|extension| extension.to_str())
    .is_some_and(|extension| {
      WALLPAPER_EXTENSIONS.contains(&extension.to_ascii_lowercase().as_str())
    })
}

/// Checks the theme of an archive. Returns its name and whether a theme of the
/// user already has its id; an official theme of that id is an error.
pub fn inspect_archive(path: &Path) -> Result<(String, bool), String> {
  let paths = theme_paths()?;
  let staged = stage(path)?;
  let manifest = package::check_theme_dir(&paths, &staged.dir)?;
  if package::is_official(&paths, &manifest.id) {
    return Err(format!(
      "an official theme is already named `{}`",
      manifest.id
    ));
  }
  Ok((manifest.name, package::is_user_theme(&paths, &manifest.id)))
}

/// Installs the theme of an archive among the user's themes, replacing a theme
/// of the same id (the caller asked first).
pub fn import_archive(path: &Path) -> Result<CustomTheme, String> {
  let paths = theme_paths()?;
  let staged = stage(path)?;
  let target = package::install_user_theme(&paths, &staged.dir, true)?;
  let manifest =
    argvus_theme_core::manifest::ThemeManifest::load(&target).map_err(|error| error.to_string())?;
  Ok(CustomTheme {
    id: manifest.id,
    name: manifest.name,
  })
}

/// Converts the appearance profiles of earlier versions into themes, once:
/// each `<dir>/profile.tar.gz` becomes a theme made from its base theme, with
/// its highlight colour and wallpaper, and the profile moves to
/// `data/backups/theme-profiles-<seconds>/`. Returns one line per profile.
pub fn convert_legacy_profiles() -> Vec<String> {
  let Ok(paths) = theme_paths() else {
    return Vec::new();
  };
  let Ok(entries) = fs::read_dir(&paths.user_themes) else {
    return Vec::new();
  };
  let legacy: Vec<PathBuf> = entries
    .filter_map(Result::ok)
    .map(|entry| entry.path())
    .filter(|dir| {
      dir.join("profile.tar.gz").is_file()
        && !dir.join("theme.toml").exists()
        && !dir
          .file_name()
          .is_some_and(|name| name.to_string_lossy().starts_with('.'))
    })
    .collect();
  if legacy.is_empty() {
    return Vec::new();
  }
  let seconds = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map_or(0, |elapsed| elapsed.as_secs());
  let backup = argvus_data_home().join(format!("backups/theme-profiles-{seconds}"));
  let mut report = Vec::new();
  for dir in legacy {
    let profile_name = dir
      .file_name()
      .map(|name| name.to_string_lossy().into_owned())
      .unwrap_or_default();
    let moved = backup.join(&profile_name);
    if let Err(error) = move_dir(&dir, &moved) {
      report.push(format!("{profile_name}: {error}"));
      continue;
    }
    match convert_profile(&paths, &moved.join("profile.tar.gz")) {
      Ok(theme) => report.push(format!("{profile_name} → {}", theme.name)),
      Err(error) => report.push(format!("{profile_name}: {error}")),
    }
  }
  for state in ["custom-themes.json", "state/custom-theme"] {
    let path = argvus_data_home().join(state);
    if path.is_file() {
      let _ = fs::create_dir_all(&backup);
      let _ = move_file(
        &path,
        &backup.join(Path::new(state).file_name().unwrap_or_default()),
      );
    }
  }
  report
}

/// The theme made from one legacy profile archive.
fn convert_profile(paths: &ThemePaths, archive: &Path) -> Result<CustomTheme, String> {
  let profile = LegacyProfile::read(archive)?;
  let base = canonical_theme_id(&profile.theme);
  let base = base.strip_suffix("-float").unwrap_or(&base);
  let source = discovery::find(paths, base)
    .ok_or_else(|| format!("its base theme `{base}` is not installed; kept in the backup"))?;
  let name = clean_name(&profile.name)?;
  let (id, _) = export_target(paths, &name)?;
  let staging = tempdir().map_err(|error| error.to_string())?;
  let wallpaper = profile
    .wallpaper
    .as_ref()
    .map(|(file, _)| staging.path().join(file));
  if let (Some(path), Some((_, bytes))) = (&wallpaper, &profile.wallpaper) {
    fs::write(path, bytes).map_err(|error| error.to_string())?;
  }
  let dir = staging.path().join(&id);
  write_theme_dir(
    &dir,
    &source.dir,
    &id,
    &name,
    profile.accent,
    wallpaper.as_deref(),
  )?;
  package::install_user_theme(paths, &dir, false)?;
  Ok(CustomTheme { id, name })
}

/// What a theme takes from a legacy profile.
struct LegacyProfile {
  name: String,
  theme: String,
  accent: Option<Color>,
  /// The file name and the bytes of its wallpaper.
  wallpaper: Option<(String, Vec<u8>)>,
}

impl LegacyProfile {
  fn read(path: &Path) -> Result<Self, String> {
    let file = File::open(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let mut archive = Archive::new(GzDecoder::new(file));
    let mut manifest: Option<Value> = None;
    let mut canonical: Option<Value> = None;
    let mut accent_file: Option<String> = None;
    let mut images: Vec<(String, Vec<u8>)> = Vec::new();
    let entries = archive.entries().map_err(|error| error.to_string())?;
    for entry in entries.take(MAX_MEMBERS) {
      let mut entry = entry.map_err(|error| error.to_string())?;
      if entry.header().entry_type() != EntryType::Regular {
        continue;
      }
      let member = entry
        .path()
        .map_err(|error| error.to_string())?
        .to_string_lossy()
        .into_owned();
      let limit = if member.contains("/wallpaper/") {
        MAX_WALLPAPER
      } else {
        MAX_FILE
      };
      let mut bytes = Vec::new();
      (&mut entry)
        .take(limit)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
      match member.as_str() {
        "argvus-theme-profile/manifest.json" => manifest = serde_json::from_slice(&bytes).ok(),
        "argvus-theme-profile/payload/config/argvus/config.json" => {
          canonical = serde_json::from_slice(&bytes).ok();
        }
        "argvus-theme-profile/payload/config/argvus/.accent-color" => {
          accent_file = Some(String::from_utf8_lossy(&bytes).trim().to_owned());
        }
        _ if member.starts_with("argvus-theme-profile/wallpaper/") => {
          let name = member.rsplit('/').next().unwrap_or_default().to_owned();
          images.push((name, bytes));
        }
        _ => {}
      }
    }
    let manifest = manifest.ok_or("the profile has no manifest")?;
    let name = manifest
      .get("name")
      .and_then(Value::as_str)
      .ok_or("the profile has no name")?
      .to_owned();
    let theme = manifest
      .pointer("/argvus/theme")
      .and_then(Value::as_str)
      .ok_or("the profile names no theme")?
      .to_owned();
    let custom_accent = canonical.as_ref().and_then(|document| {
      let owned = document.pointer("/appearance/accent_custom") == Some(&Value::Bool(true));
      owned
        .then(|| {
          document
            .pointer("/appearance/accent")
            .and_then(Value::as_str)
        })
        .flatten()
        .map(str::to_owned)
    });
    let accent = custom_accent
      .or(if canonical.is_none() {
        accent_file
      } else {
        None
      })
      .and_then(|text| Color::parse(&text));
    Ok(Self {
      name,
      theme,
      accent,
      wallpaper: images.into_iter().next(),
    })
  }
}

/// Moves a directory, across file systems too.
fn move_dir(source: &Path, destination: &Path) -> io::Result<()> {
  if let Some(parent) = destination.parent() {
    fs::create_dir_all(parent)?;
  }
  if fs::rename(source, destination).is_ok() {
    return Ok(());
  }
  copy_files(source, destination)?;
  fs::remove_dir_all(source)
}

fn move_file(source: &Path, destination: &Path) -> io::Result<()> {
  if fs::rename(source, destination).is_ok() {
    return Ok(());
  }
  fs::copy(source, destination)?;
  fs::remove_file(source)
}

fn argvus_data_home() -> PathBuf {
  argvus_control_center_core::paths::argvus_data_home()
}

#[cfg(test)]
mod tests {
  use super::*;

  fn write(path: &Path, bytes: &[u8]) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, bytes).unwrap();
  }

  /// A tar.gz with the given (member, bytes) entries.
  fn archive(path: &Path, members: &[(&str, &[u8])]) {
    let file = File::create(path).unwrap();
    let mut builder = Builder::new(GzEncoder::new(file, Compression::default()));
    for (member, bytes) in members {
      let mut header = Header::new_gnu();
      header.set_entry_type(EntryType::Regular);
      header.set_mode(0o644);
      header.set_size(bytes.len() as u64);
      header.set_cksum();
      builder.append_data(&mut header, member, *bytes).unwrap();
    }
    builder.into_inner().unwrap().finish().unwrap();
  }

  #[test]
  fn stage_accepts_one_theme_directory_only() {
    let root = tempdir().unwrap();
    let good = root.path().join("good.tar.gz");
    archive(
      &good,
      &[("mist/theme.toml", b"x"), ("mist/wallpaper.png", b"png")],
    );
    let staged = stage(&good).unwrap();
    assert!(staged.dir.ends_with("mist"));
    assert!(staged.dir.join("wallpaper.png").is_file());
    assert!(holds_a_theme(&good));

    let two = root.path().join("two.tar.gz");
    archive(&two, &[("a/theme.toml", b"x"), ("b/theme.toml", b"x")]);
    assert!(stage(&two).unwrap_err().contains("single theme"));

    let other = root.path().join("other.tar.gz");
    archive(&other, &[("notes.txt", b"x")]);
    assert!(!holds_a_theme(&other));

    let big = root.path().join("big.tar.gz");
    let large = vec![b'x'; (MAX_FILE + 1) as usize];
    archive(&big, &[("mist/theme.toml", &large)]);
    assert!(stage(&big).unwrap_err().contains("too large"));
  }

  #[test]
  fn legacy_profiles_give_their_name_theme_accent_and_wallpaper() {
    let root = tempdir().unwrap();
    let v4 = root.path().join("v4.tar.gz");
    archive(
      &v4,
      &[
        (
          "argvus-theme-profile/manifest.json",
          br#"{"name":"dove","argvus":{"theme":"argvus-dark-float"}}"#,
        ),
        (
          "argvus-theme-profile/payload/config/argvus/config.json",
          br##"{"appearance":{"accent":"#B5655A","accent_custom":true}}"##,
        ),
        ("argvus-theme-profile/wallpaper/dove.jpg", b"jpg"),
      ],
    );
    let profile = LegacyProfile::read(&v4).unwrap();
    assert_eq!(profile.name, "dove");
    assert_eq!(profile.theme, "argvus-dark-float");
    assert_eq!(
      profile.accent.map(|color| color.hex()).as_deref(),
      Some("#B5655A")
    );
    assert_eq!(
      profile.wallpaper.map(|(name, _)| name).as_deref(),
      Some("dove.jpg")
    );

    let v1 = root.path().join("v1.tar.gz");
    archive(
      &v1,
      &[
        (
          "argvus-theme-profile/manifest.json",
          br#"{"name":"mint","argvus":{"theme":"argvus-dark-universe-float"}}"#,
        ),
        (
          "argvus-theme-profile/payload/config/argvus/.accent-color",
          b"#17d174\n",
        ),
      ],
    );
    let profile = LegacyProfile::read(&v1).unwrap();
    assert_eq!(canonical_theme_id(&profile.theme), "universe-float");
    assert_eq!(
      profile.accent.map(|color| color.hex()).as_deref(),
      Some("#17D174")
    );
    assert!(profile.wallpaper.is_none());
  }

  /// A session tree: an installed `sunset` theme with one consumer, the
  /// active theme and accent, and a custom wallpaper.
  fn session(root: &Path) {
    let system = root.join("system");
    write(
      &system.join("themes/sunset/theme.toml"),
      b"schema = 2\nid = \"sunset\"\nname = \"Sunset\"\ncategory = \"dark\"\n\
        [palette]\nbg = \"#000000\"\nfg = \"#FFFFFF\"\naccent = \"#E2BE8A\"\n\
        [overrides]\nterminal = \"overrides/terminal.conf\"\n",
    );
    write(
      &system.join("themes/sunset/overrides/terminal.conf"),
      b"fg {{ @accent }}\n",
    );
    write(&system.join("theme/layouts/sticky.toml"), b"gap = 1\n");
    write(&system.join("theme/layouts/float.toml"), b"gap = 8\n");
    write(&system.join("t/a.txt"), b"{{ @fg }} {{ layout.gap }}\n");
    write(
      &system.join("theme/consumers.d/terminal.toml"),
      b"id = \"terminal\"\ntemplate = \"t/a.txt\"\noutput = \"a.txt\"\n",
    );
    let data = root.join("config/argvus/data");
    write(&data.join(".active-theme"), b"sunset-float\n");
    write(&data.join(".accent-color"), b"#112233\n");
    write(&root.join("home/Pictures/mine.png"), b"png");
    write(
      &data.join(".wallpaper-custom"),
      root
        .join("home/Pictures/mine.png")
        .display()
        .to_string()
        .as_bytes(),
    );
    // SAFETY: the crate-wide lock serializes every test that changes the environment.
    unsafe {
      std::env::set_var("HOME", root.join("home"));
      std::env::set_var("XDG_DATA_HOME", root.join("share"));
      std::env::set_var("ARGVUS_CONFIG_HOME", root.join("config"));
      std::env::set_var("ARGVUS_SYSTEM_CONFIG", &system);
    }
  }

  #[test]
  fn export_import_convert_and_delete_user_themes() {
    let _lock = crate::test_support::env_lock();
    let root = tempdir().unwrap();
    session(root.path());

    assert_eq!(
      preview_export_path("My Sunset"),
      Some(root.path().join("home/argvus-theme-my-sunset.tar.gz"))
    );
    let archive = export_named("My Sunset").unwrap();
    let (name, duplicate) = inspect_archive(&archive).unwrap();
    assert_eq!((name.as_str(), duplicate), ("My Sunset", false));
    assert_eq!(list_import_archives(), vec![archive.clone()]);

    let theme = import_archive(&archive).unwrap();
    assert_eq!(theme.id, "my-sunset");
    let installed = root.path().join("share/argvus/themes/my-sunset");
    let manifest = argvus_theme_core::manifest::ThemeManifest::load(&installed).unwrap();
    assert_eq!(manifest.token("accent").unwrap().hex(), "#112233");
    assert_eq!(manifest.wallpaper, Some(installed.join("wallpaper.png")));
    assert!(installed.join("overrides/terminal.conf").is_file());
    assert!(inspect_archive(&archive).unwrap().1, "now a duplicate");
    assert_eq!(custom_themes(), vec![theme.clone()]);
    assert_eq!(active_custom_theme(), None);

    // An exported official id is refused on import.
    let staging = tempdir().unwrap();
    let official = staging.path().join("sunset");
    copy_files(&installed, &official).unwrap();
    let text = fs::read_to_string(official.join("theme.toml")).unwrap();
    fs::write(
      official.join("theme.toml"),
      text.replace("my-sunset", "sunset"),
    )
    .unwrap();
    let official_archive = root.path().join("home/official.tar.gz");
    write_archive(&official, "sunset", &official_archive).unwrap();
    assert!(
      inspect_archive(&official_archive)
        .unwrap_err()
        .contains("official")
    );

    // A legacy appearance profile becomes a theme and moves to a backup.
    let legacy = root.path().join("share/argvus/themes/dove");
    fs::create_dir_all(&legacy).unwrap();
    archive_members(
      &legacy.join("profile.tar.gz"),
      &[
        (
          "argvus-theme-profile/manifest.json",
          br#"{"name":"dove","argvus":{"theme":"sunset-float"}}"#,
        ),
        (
          "argvus-theme-profile/payload/config/argvus/.accent-color",
          b"#B5655A\n",
        ),
        ("argvus-theme-profile/wallpaper/dove.jpg", b"jpg"),
      ],
    );
    let report = convert_legacy_profiles();
    assert_eq!(report, vec!["dove → dove".to_string()]);
    let dove = root.path().join("share/argvus/themes/dove");
    assert!(dove.join("theme.toml").is_file());
    assert!(dove.join("wallpaper.jpg").is_file());
    assert!(!dove.join("profile.tar.gz").exists());
    let backups = root.path().join("config/argvus/data/backups");
    assert_eq!(fs::read_dir(&backups).unwrap().count(), 1);
    assert!(convert_legacy_profiles().is_empty(), "runs once");

    delete_custom_theme(&theme).unwrap();
    assert!(!installed.exists());
    assert_eq!(custom_themes().len(), 1);
  }

  fn archive_members(path: &Path, members: &[(&str, &[u8])]) {
    archive(path, members);
  }

  #[test]
  fn an_exported_theme_directory_is_archived_under_its_id() {
    let root = tempdir().unwrap();
    let dir = root.path().join("mist");
    write(&dir.join("theme.toml"), b"t");
    write(&dir.join("overrides/terminal.conf"), b"o");
    let output = root.path().join("argvus-theme-mist.tar.gz");
    write_archive(&dir, "mist", &output).unwrap();
    let staged = stage(&output).unwrap();
    assert_eq!(
      fs::read(staged.dir.join("overrides/terminal.conf")).unwrap(),
      b"o"
    );
    assert!(!root.path().join("argvus-theme-mist.gz.tmp").exists());
  }
}
