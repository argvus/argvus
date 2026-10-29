//! XDG directories and atomic, versioned JSON files.
//!
//! Every file is an envelope `{"version": N, "data": …}`. A file that cannot be
//! parsed, or that has another version, is renamed to
//! `<name>.corrupt-<unix seconds>` and replaced by defaults, so a bad file
//! never stops the application and the user's data is kept for inspection.

use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StoreError {
  #[error("cannot resolve the user directories: HOME is not set")]
  NoHome,
  #[error("cannot read {}: {source}", path.display())]
  Read {
    path: PathBuf,
    #[source]
    source: io::Error,
  },
  #[error("cannot write {}: {source}", path.display())]
  Write {
    path: PathBuf,
    #[source]
    source: io::Error,
  },
  #[error("cannot encode {}: {source}", path.display())]
  Encode {
    path: PathBuf,
    #[source]
    source: serde_json::Error,
  },
}

/// Per-application directories.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppDirs {
  /// Preferences: `$XDG_CONFIG_HOME/argvus/data/<app>` (or
  /// `$ARGVUS_DATA_HOME/<app>`), next to the other ARGVUS preferences.
  pub config: PathBuf,
  /// Records the user keeps: `$XDG_DATA_HOME/argvus/<app>`.
  pub data: PathBuf,
  /// Runtime state that may be lost: `$XDG_STATE_HOME/argvus/<app>`.
  pub state: PathBuf,
}

impl AppDirs {
  pub fn for_app(app: &str) -> Result<Self, StoreError> {
    Self::from_lookup(app, |name| std::env::var_os(name))
  }

  fn from_lookup(app: &str, lookup: impl Fn(&str) -> Option<OsString>) -> Result<Self, StoreError> {
    // The XDG specification ignores relative paths.
    let absolute = |name: &str| {
      lookup(name)
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
    };
    let home = absolute("HOME");
    let under_home = |relative: &str| home.as_ref().map(|home| home.join(relative));

    let config_home = absolute("ARGVUS_CONFIG_HOME")
      .or_else(|| absolute("XDG_CONFIG_HOME"))
      .or_else(|| under_home(".config"))
      .ok_or(StoreError::NoHome)?;
    let preferences =
      absolute("ARGVUS_DATA_HOME").unwrap_or_else(|| config_home.join("argvus/data"));
    let data_home = absolute("XDG_DATA_HOME")
      .or_else(|| under_home(".local/share"))
      .ok_or(StoreError::NoHome)?;
    let state_home = absolute("XDG_STATE_HOME")
      .or_else(|| under_home(".local/state"))
      .ok_or(StoreError::NoHome)?;

    Ok(Self {
      config: preferences.join(app),
      data: data_home.join("argvus").join(app),
      state: state_home.join("argvus").join(app),
    })
  }
}

#[derive(Serialize, Deserialize)]
struct Envelope<T> {
  version: u32,
  data: T,
}

/// A loaded document.
#[derive(Debug)]
pub struct Loaded<T> {
  pub value: T,
  /// Where an unreadable file was moved before defaults were used.
  pub recovered_from: Option<PathBuf>,
}

/// Reads a document. A missing file gives the defaults; an unreadable one is
/// moved aside and gives the defaults. Only I/O errors other than "not found"
/// fail, so a file that exists but cannot be read is never overwritten.
pub fn load_json<T: DeserializeOwned + Default>(
  path: &Path,
  version: u32,
) -> Result<Loaded<T>, StoreError> {
  let text = match fs::read_to_string(path) {
    Ok(text) => text,
    Err(error) if error.kind() == io::ErrorKind::NotFound => {
      return Ok(Loaded {
        value: T::default(),
        recovered_from: None,
      });
    }
    Err(source) => {
      return Err(StoreError::Read {
        path: path.to_owned(),
        source,
      });
    }
  };

  let problem = match serde_json::from_str::<Envelope<T>>(&text) {
    Ok(envelope) if envelope.version == version => {
      return Ok(Loaded {
        value: envelope.data,
        recovered_from: None,
      });
    }
    Ok(envelope) => format!("version {} (expected {version})", envelope.version),
    Err(error) => error.to_string(),
  };

  let backup = backup_path(path);
  fs::rename(path, &backup).map_err(|source| StoreError::Write {
    path: backup.clone(),
    source,
  })?;
  log::warn!(
    "{} could not be used ({problem}); moved to {} and defaults were loaded",
    path.display(),
    backup.display()
  );
  Ok(Loaded {
    value: T::default(),
    recovered_from: Some(backup),
  })
}

/// Writes a document atomically.
pub fn save_json<T: Serialize>(path: &Path, version: u32, value: &T) -> Result<(), StoreError> {
  let envelope = Envelope {
    version,
    data: value,
  };
  let mut bytes = serde_json::to_vec_pretty(&envelope).map_err(|source| StoreError::Encode {
    path: path.to_owned(),
    source,
  })?;
  bytes.push(b'\n');
  atomic_write(path, &bytes)
}

/// Writes `bytes` to a temporary file beside `path`, flushes it to disk and
/// renames it over `path`. Readers see either the old or the new file, never
/// a partial one, and a failed write leaves the old file in place.
pub fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), StoreError> {
  let write_error = |source| StoreError::Write {
    path: path.to_owned(),
    source,
  };
  let parent = path
    .parent()
    .filter(|parent| !parent.as_os_str().is_empty())
    .unwrap_or(Path::new("."));
  fs::create_dir_all(parent).map_err(write_error)?;

  let file_name = path
    .file_name()
    .map(|name| name.to_string_lossy().into_owned())
    .unwrap_or_default();
  let temporary = parent.join(format!(".{file_name}.{}.tmp", std::process::id()));
  let result = write_synced(&temporary, bytes).and_then(|()| fs::rename(&temporary, path));
  if let Err(error) = result {
    let _ = fs::remove_file(&temporary);
    return Err(write_error(error));
  }
  // Persist the rename itself; a failure here does not lose the new contents.
  if let Ok(directory) = File::open(parent) {
    let _ = directory.sync_all();
  }
  Ok(())
}

fn write_synced(path: &Path, bytes: &[u8]) -> io::Result<()> {
  let mut file = OpenOptions::new()
    .write(true)
    .create(true)
    .truncate(true)
    .open(path)?;
  file.write_all(bytes)?;
  file.sync_all()
}

fn backup_path(path: &Path) -> PathBuf {
  let seconds = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map_or(0, |elapsed| elapsed.as_secs());
  let mut name = path.file_name().map(OsString::from).unwrap_or_default();
  name.push(format!(".corrupt-{seconds}"));
  path.with_file_name(name)
}

#[cfg(test)]
mod tests {
  use std::collections::HashMap;

  use super::*;

  #[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
  struct Sample {
    count: u32,
    name: String,
  }

  #[test]
  fn saved_document_loads_back() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("nested/sample.json");
    let sample = Sample {
      count: 3,
      name: "focus".to_owned(),
    };
    save_json(&path, 1, &sample).unwrap();
    let loaded = load_json::<Sample>(&path, 1).unwrap();
    assert_eq!(loaded.value, sample);
    assert_eq!(loaded.recovered_from, None);
  }

  #[test]
  fn missing_file_gives_defaults() {
    let directory = tempfile::tempdir().unwrap();
    let loaded = load_json::<Sample>(&directory.path().join("absent.json"), 1).unwrap();
    assert_eq!(loaded.value, Sample::default());
    assert_eq!(loaded.recovered_from, None);
  }

  #[test]
  fn corrupt_file_is_kept_aside_and_defaults_are_used() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("sample.json");
    fs::write(&path, "{ not json").unwrap();
    let loaded = load_json::<Sample>(&path, 1).unwrap();
    assert_eq!(loaded.value, Sample::default());
    let backup = loaded.recovered_from.expect("backup path");
    assert_eq!(fs::read_to_string(backup).unwrap(), "{ not json");
    assert!(!path.exists());
  }

  #[test]
  fn other_version_is_kept_aside() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("sample.json");
    save_json(&path, 2, &Sample::default()).unwrap();
    let loaded = load_json::<Sample>(&path, 1).unwrap();
    assert!(loaded.recovered_from.is_some());
  }

  #[test]
  fn atomic_write_replaces_without_leaving_temporary_files() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("sample.json");
    atomic_write(&path, b"old").unwrap();
    atomic_write(&path, b"new").unwrap();
    assert_eq!(fs::read(&path).unwrap(), b"new");
    let entries: Vec<_> = fs::read_dir(directory.path()).unwrap().collect();
    assert_eq!(entries.len(), 1);
  }

  #[test]
  fn failed_write_keeps_the_previous_file() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("sample.json");
    atomic_write(&path, b"kept").unwrap();
    // A directory in place of the temporary file makes the write fail.
    let blocker = directory
      .path()
      .join(format!(".sample.json.{}.tmp", std::process::id()));
    fs::create_dir(&blocker).unwrap();
    assert!(atomic_write(&path, b"lost").is_err());
    assert_eq!(fs::read(&path).unwrap(), b"kept");
  }

  #[test]
  fn directories_follow_xdg_and_argvus_overrides() {
    let environment: HashMap<&str, &str> = [
      ("HOME", "/home/user"),
      ("XDG_STATE_HOME", "/srv/state"),
      ("XDG_DATA_HOME", "relative/ignored"),
    ]
    .into();
    let directories =
      AppDirs::from_lookup("clock", |name| environment.get(name).map(OsString::from)).unwrap();
    assert_eq!(
      directories.config,
      PathBuf::from("/home/user/.config/argvus/data/clock")
    );
    assert_eq!(
      directories.data,
      PathBuf::from("/home/user/.local/share/argvus/clock")
    );
    assert_eq!(directories.state, PathBuf::from("/srv/state/argvus/clock"));

    let with_override: HashMap<&str, &str> =
      [("HOME", "/home/user"), ("ARGVUS_DATA_HOME", "/tmp/prefs")].into();
    let overridden =
      AppDirs::from_lookup("focus", |name| with_override.get(name).map(OsString::from)).unwrap();
    assert_eq!(overridden.config, PathBuf::from("/tmp/prefs/focus"));
  }

  #[test]
  fn missing_home_is_an_error() {
    assert!(matches!(
      AppDirs::from_lookup("clock", |_| None),
      Err(StoreError::NoHome)
    ));
  }
}
