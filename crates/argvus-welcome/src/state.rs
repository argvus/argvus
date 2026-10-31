use std::collections::BTreeMap;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

const SCHEMA_VERSION: u32 = 2;
const STATE_FILE: &str = "progress.json";
pub const AUTO_START_MARKER: &str = "auto-start-pending";

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
  #[default]
  Pending,
  Reviewed,
  Skipped,
  NotApplicable,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Progress {
  pub schema_version: u32,
  pub opened: bool,
  pub started: bool,
  pub current_category: String,
  pub categories: BTreeMap<String, StepStatus>,
  pub completed: bool,
  pub auto_dismissed: bool,
  pub updated_unix_seconds: u64,
}

impl Default for Progress {
  fn default() -> Self {
    Self {
      schema_version: SCHEMA_VERSION,
      opened: false,
      started: false,
      current_category: String::new(),
      categories: BTreeMap::new(),
      completed: false,
      auto_dismissed: false,
      updated_unix_seconds: 0,
    }
  }
}

impl Progress {
  pub fn status(&self, category: &str) -> StepStatus {
    self.categories.get(category).copied().unwrap_or_default()
  }

  pub fn set_status(&mut self, category: &str, status: StepStatus) {
    if status == StepStatus::Pending {
      self.categories.remove(category);
    } else {
      self.categories.insert(category.to_owned(), status);
    }
  }
}

#[derive(Clone, Debug)]
pub struct StateStore {
  directory: PathBuf,
}

impl StateStore {
  pub fn new() -> io::Result<Self> {
    Ok(Self {
      directory: state_directory()?,
    })
  }

  pub fn at(directory: PathBuf) -> Self {
    Self { directory }
  }

  pub fn load(&self) -> (Progress, Option<String>) {
    let path = self.directory.join(STATE_FILE);
    let contents = match fs::read(&path) {
      Ok(contents) => contents,
      Err(error) if error.kind() == io::ErrorKind::NotFound => {
        return (Progress::default(), None);
      }
      Err(error) => return (Progress::default(), Some(error.to_string())),
    };
    let mut progress: Progress = match serde_json::from_slice(&contents) {
      Ok(progress) => progress,
      Err(error) => {
        return (
          Progress::default(),
          Some(format!("{}: {error}", path.display())),
        );
      }
    };
    if progress.schema_version > SCHEMA_VERSION {
      return (
        Progress::default(),
        Some(format!(
          "unsupported state version {}",
          progress.schema_version
        )),
      );
    }
    if progress.schema_version < SCHEMA_VERSION && progress.started {
      // Older Welcome versions set `started` after the user left the intro.
      // That necessarily means the TUI had already been opened.
      progress.opened = true;
    }
    progress.schema_version = SCHEMA_VERSION;
    (progress, None)
  }

  pub fn save(&self, progress: &mut Progress) -> io::Result<()> {
    fs::create_dir_all(&self.directory)?;
    #[cfg(unix)]
    {
      use std::os::unix::fs::PermissionsExt;
      fs::set_permissions(&self.directory, fs::Permissions::from_mode(0o700))?;
    }
    progress.schema_version = SCHEMA_VERSION;
    progress.updated_unix_seconds = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap_or_default()
      .as_secs();
    let contents = serde_json::to_vec_pretty(progress).map_err(io::Error::other)?;
    let nonce = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap_or_default()
      .as_nanos();
    let temporary_path = self.directory.join(format!(
      ".{STATE_FILE}.{}.{}.tmp",
      std::process::id(),
      nonce
    ));
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
      use std::os::unix::fs::OpenOptionsExt;
      options.mode(0o600);
    }
    let mut file = options.open(&temporary_path)?;
    if let Err(error) = preserve_unreadable_state(&self.directory) {
      drop(file);
      let _ = fs::remove_file(&temporary_path);
      return Err(error);
    }
    let write_result = (|| {
      file.write_all(&contents)?;
      file.sync_all()?;
      fs::rename(&temporary_path, self.directory.join(STATE_FILE))
    })();
    if write_result.is_err() {
      let _ = fs::remove_file(&temporary_path);
    }
    write_result
  }

  /// Ask an already running Control Center to enter onboarding when it can
  /// safely leave its current page. The request survives terminal startup.
  pub fn request_open(&self) -> io::Result<()> {
    fs::create_dir_all(&self.directory)?;
    #[cfg(unix)]
    {
      use std::os::unix::fs::PermissionsExt;
      fs::set_permissions(&self.directory, fs::Permissions::from_mode(0o700))?;
    }
    let mut options = OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
      use std::os::unix::fs::OpenOptionsExt;
      options.mode(0o600);
    }
    options
      .open(self.directory.join("open-request"))?
      .sync_all()
  }

  pub fn take_open_request(&self) -> io::Result<bool> {
    match fs::remove_file(self.directory.join("open-request")) {
      Ok(()) => Ok(true),
      Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
      Err(error) => Err(error),
    }
  }

  pub fn marker_path(&self) -> PathBuf {
    self.directory.join(AUTO_START_MARKER)
  }

  pub fn has_auto_start_marker(&self) -> bool {
    self.marker_path().is_file()
  }

  pub fn consume_auto_start_marker(&self) -> io::Result<()> {
    match fs::remove_file(self.marker_path()) {
      Ok(()) => Ok(()),
      Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
      Err(error) => Err(error),
    }
  }
}

/// Keeps an invalid or future-version file recoverable before replacing it.
fn preserve_unreadable_state(directory: &Path) -> io::Result<()> {
  let path = directory.join(STATE_FILE);
  let Ok(contents) = fs::read(&path) else {
    return Ok(());
  };
  let Ok(progress) = serde_json::from_slice::<Progress>(&contents) else {
    return archive_state(&path);
  };
  if progress.schema_version > SCHEMA_VERSION {
    archive_state(&path)?;
  }
  Ok(())
}

fn archive_state(path: &Path) -> io::Result<()> {
  let timestamp = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap_or_default()
    .as_nanos();
  let backup = path.with_file_name(format!("{STATE_FILE}.recovered-{timestamp}.json"));
  fs::rename(path, backup)
}

fn state_directory() -> io::Result<PathBuf> {
  if let Some(path) = env::var_os("XDG_STATE_HOME").map(PathBuf::from)
    && path.is_absolute()
  {
    return Ok(path.join("argvus/welcome"));
  }
  let home = env::var_os("HOME")
    .map(PathBuf::from)
    .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "HOME and XDG_STATE_HOME are unset"))?;
  Ok(home.join(".local/state/argvus/welcome"))
}

pub fn pending_marker_path(directory: &Path) -> PathBuf {
  directory.join(AUTO_START_MARKER)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn open_request_is_private_and_consumed_once() {
    let directory = tempfile::tempdir().unwrap();
    let store = StateStore::at(directory.path().join("welcome"));
    assert!(!store.take_open_request().unwrap());
    store.request_open().unwrap();
    assert!(store.take_open_request().unwrap());
    assert!(!store.take_open_request().unwrap());
  }

  #[test]
  fn missing_state_starts_new_progress() {
    let directory = tempfile::tempdir().unwrap();
    let (progress, warning) = StateStore::at(directory.path().to_path_buf()).load();
    assert!(!progress.started);
    assert!(warning.is_none());
  }

  #[test]
  fn state_round_trips_reviewed_skipped_and_completion() {
    let directory = tempfile::tempdir().unwrap();
    let store = StateStore::at(directory.path().to_path_buf());
    let mut progress = Progress {
      started: true,
      completed: true,
      ..Progress::default()
    };
    progress.set_status("language", StepStatus::Reviewed);
    progress.set_status("audio", StepStatus::Skipped);
    store.save(&mut progress).unwrap();
    let (loaded, warning) = store.load();
    assert!(warning.is_none());
    assert!(loaded.started && loaded.completed);
    assert_eq!(loaded.status("language"), StepStatus::Reviewed);
    assert_eq!(loaded.status("audio"), StepStatus::Skipped);
  }

  #[test]
  fn corrupt_and_future_state_return_recoverable_warning() {
    let directory = tempfile::tempdir().unwrap();
    let store = StateStore::at(directory.path().to_path_buf());
    fs::create_dir_all(directory.path()).unwrap();
    fs::write(directory.path().join(STATE_FILE), b"{").unwrap();
    assert!(store.load().1.is_some());
    fs::write(
      directory.path().join(STATE_FILE),
      br#"{"schema_version":99}"#,
    )
    .unwrap();
    assert!(
      store
        .load()
        .1
        .unwrap()
        .contains("unsupported state version")
    );
  }

  #[test]
  fn old_schema_is_loaded_and_unreadable_state_is_archived_before_replacement() {
    let directory = tempfile::tempdir().unwrap();
    let store = StateStore::at(directory.path().to_path_buf());
    fs::create_dir_all(directory.path()).unwrap();
    fs::write(
      directory.path().join(STATE_FILE),
      br#"{"schema_version":0,"started":true}"#,
    )
    .unwrap();
    let (migrated, warning) = store.load();
    assert!(warning.is_none());
    assert!(migrated.started);
    assert!(migrated.opened);
    assert_eq!(migrated.schema_version, SCHEMA_VERSION);

    fs::write(
      directory.path().join(STATE_FILE),
      br#"{"schema_version":99}"#,
    )
    .unwrap();
    let (future, warning) = store.load();
    assert!(warning.is_some());
    assert!(!future.started);
    store.save(&mut Progress::default()).unwrap();
    assert!(
      fs::read_dir(directory.path())
        .unwrap()
        .filter_map(Result::ok)
        .any(|entry| entry.file_name().to_string_lossy().contains("recovered-"))
    );
  }

  #[test]
  fn marker_is_explicit_and_consumed_idempotently() {
    let directory = tempfile::tempdir().unwrap();
    let store = StateStore::at(directory.path().to_path_buf());
    assert!(!store.has_auto_start_marker());
    fs::create_dir_all(directory.path()).unwrap();
    fs::write(pending_marker_path(directory.path()), b"").unwrap();
    assert!(store.has_auto_start_marker());
    store.consume_auto_start_marker().unwrap();
    store.consume_auto_start_marker().unwrap();
    assert!(!store.has_auto_start_marker());
  }

  #[cfg(unix)]
  #[test]
  fn saved_state_is_private_to_the_user() {
    use std::os::unix::fs::PermissionsExt;
    let directory = tempfile::tempdir().unwrap();
    let store = StateStore::at(directory.path().to_path_buf());
    store.save(&mut Progress::default()).unwrap();
    assert_eq!(
      fs::metadata(directory.path().join(STATE_FILE))
        .unwrap()
        .permissions()
        .mode()
        & 0o777,
      0o600
    );
  }
}
