//! Local activation counts and most-recent-first application order.
use serde::{Deserialize, Serialize};
use std::{
  collections::BTreeMap,
  fs::{self, File, OpenOptions},
  io::Write,
  os::unix::fs::OpenOptionsExt,
  path::{Path, PathBuf},
};

#[derive(Default, Serialize, Deserialize)]
struct UsageHistory {
  #[serde(default)]
  applications: BTreeMap<String, u64>,
  #[serde(default)]
  recent_order: Vec<String>,
}

fn paths() -> Result<(PathBuf, PathBuf), String> {
  let directory = argvus_config_core::project::argvus_state_root()
    .map_err(|error| error.to_string())?
    .join("launcher");
  Ok((
    directory.join("application-usage.json"),
    directory.join("usage.lock"),
  ))
}

fn lock(path: &Path) -> Result<File, String> {
  let file = OpenOptions::new()
    .create(true)
    .truncate(false)
    .write(true)
    .mode(0o600)
    .open(path)
    .map_err(|error| error.to_string())?;
  file.lock().map_err(|error| error.to_string())?;
  Ok(file)
}

fn read(path: &Path) -> Result<UsageHistory, String> {
  match fs::read(path) {
    Ok(contents) => serde_json::from_slice(&contents).map_err(|error| error.to_string()),
    Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(UsageHistory::default()),
    Err(error) => Err(error.to_string()),
  }
}

fn write(path: &Path, history: &UsageHistory) -> Result<(), String> {
  let temporary = path.with_extension(format!("{}.tmp", std::process::id()));
  let result = (|| {
    let mut file = OpenOptions::new()
      .create(true)
      .truncate(true)
      .write(true)
      .mode(0o600)
      .open(&temporary)
      .map_err(|error| error.to_string())?;
    serde_json::to_writer(&mut file, history).map_err(|error| error.to_string())?;
    file.write_all(b"\n").map_err(|error| error.to_string())?;
    file.sync_all().map_err(|error| error.to_string())?;
    fs::rename(&temporary, path).map_err(|error| error.to_string())
  })();
  if result.is_err() {
    let _ = fs::remove_file(temporary);
  }
  result
}

pub fn recent_order() -> Result<Vec<String>, String> {
  let (path, lock_path) = paths()?;
  Ok(snapshot_at(&path, &lock_path)?.recent_order)
}

fn prepare(path: &Path, lock_path: &Path) -> Result<(), String> {
  let Some(parent) = path.parent() else {
    return Err("invalid launcher usage path".into());
  };
  fs::create_dir_all(parent).map_err(|error| error.to_string())?;
  if let Some(parent) = lock_path.parent() {
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
  }
  Ok(())
}

fn snapshot_at(path: &Path, lock_path: &Path) -> Result<UsageHistory, String> {
  prepare(path, lock_path)?;
  let _guard = lock(lock_path)?;
  read(path)
}

pub fn record(application_id: &str) -> Result<(), String> {
  if application_id.is_empty() {
    return Ok(());
  }
  let (path, lock_path) = paths()?;
  record_at(&path, &lock_path, application_id)
}

fn record_at(path: &Path, lock_path: &Path, application_id: &str) -> Result<(), String> {
  prepare(path, lock_path)?;
  let _guard = lock(lock_path)?;
  let mut history = read(path)?;
  let count = history
    .applications
    .entry(application_id.to_string())
    .or_default();
  *count = count.saturating_add(1);
  history.recent_order.retain(|id| id != application_id);
  history.recent_order.insert(0, application_id.to_string());
  write(path, &history)
}

pub fn clear() -> Result<(), String> {
  let (path, lock_path) = paths()?;
  clear_at(&path, &lock_path)
}

fn clear_at(path: &Path, lock_path: &Path) -> Result<(), String> {
  prepare(path, lock_path)?;
  let _guard = lock(lock_path)?;
  match fs::remove_file(path) {
    Ok(()) => Ok(()),
    Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
    Err(error) => Err(error.to_string()),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn counts_activations_and_clears_history() {
    let temporary = tempfile::tempdir().unwrap();
    let history_path = temporary.path().join("application-usage.json");
    let lock_path = temporary.path().join("usage.lock");

    record_at(&history_path, &lock_path, "editor.desktop").unwrap();
    record_at(&history_path, &lock_path, "editor.desktop").unwrap();
    record_at(&history_path, &lock_path, "browser.desktop").unwrap();

    let history = snapshot_at(&history_path, &lock_path).unwrap();
    assert_eq!(
      history.applications,
      BTreeMap::from([("browser.desktop".into(), 1), ("editor.desktop".into(), 2)])
    );
    assert_eq!(
      history.recent_order,
      ["browser.desktop".to_string(), "editor.desktop".to_string()]
    );
    clear_at(&history_path, &lock_path).unwrap();
    let history = snapshot_at(&history_path, &lock_path).unwrap();
    assert!(history.applications.is_empty());
    assert!(history.recent_order.is_empty());
  }

  #[test]
  fn reads_legacy_usage_counts_without_inventing_recency() {
    let temporary = tempfile::tempdir().unwrap();
    let history_path = temporary.path().join("application-usage.json");
    let lock_path = temporary.path().join("usage.lock");
    fs::write(&history_path, r#"{"applications":{"editor.desktop":4}}"#).unwrap();

    let history = snapshot_at(&history_path, &lock_path).unwrap();

    assert_eq!(history.applications["editor.desktop"], 4);
    assert!(history.recent_order.is_empty());
  }
}
