//! Rebuildable name-only index and XDG recent-file reader.
use crate::{
  search::{Action, Entry},
  settings::Settings,
};
use serde::{Deserialize, Serialize};
use std::{
  env, fs,
  io::Write,
  os::unix::fs::OpenOptionsExt,
  path::{Path, PathBuf},
  sync::atomic::{AtomicBool, Ordering},
};

#[derive(Default, Serialize, Deserialize)]
pub struct Index {
  pub files: Vec<PathBuf>,
  pub scanned: u64,
  pub skipped: u64,
}

pub fn cache_path() -> Result<PathBuf, String> {
  let base = env::var_os("XDG_CACHE_HOME")
    .map(PathBuf::from)
    .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache")))
    .ok_or("HOME is unavailable")?;
  Ok(base.join("argvus/launcher/index.json"))
}

pub fn load() -> Index {
  cache_path()
    .ok()
    .and_then(|path| fs::read(path).ok())
    .and_then(|data| serde_json::from_slice(&data).ok())
    .unwrap_or_default()
}

/// Serializes cache writes and clearing across the engine and Control Center.
fn cache_lock() -> Result<fs::File, String> {
  let path = cache_path()?.with_extension("lock");
  fs::create_dir_all(path.parent().ok_or("invalid cache path")?)
    .map_err(|error| error.to_string())?;
  let lock = fs::OpenOptions::new()
    .create(true)
    .truncate(false)
    .write(true)
    .mode(0o600)
    .open(path)
    .map_err(|error| error.to_string())?;
  lock.lock().map_err(|error| error.to_string())?;
  Ok(lock)
}

pub fn clear() -> Result<(), String> {
  let _lock = cache_lock()?;
  let path = cache_path()?;
  match fs::remove_file(path) {
    Ok(()) => Ok(()),
    Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
    Err(e) => Err(e.to_string()),
  }
}

fn allowed(path: &Path, settings: &Settings) -> bool {
  let relative = settings
    .roots
    .iter()
    .find_map(|root| path.strip_prefix(root).ok())
    .unwrap_or(path);
  path.is_absolute()
    && !settings
      .exclusions
      .iter()
      .any(|root| path.starts_with(root))
    && (settings.include_hidden
      || !relative
        .components()
        .any(|part| part.as_os_str().to_string_lossy().starts_with('.')))
}

fn roots(settings: &Settings) -> Vec<PathBuf> {
  settings
    .roots
    .iter()
    .filter_map(|root| fs::canonicalize(root).ok())
    .collect()
}

pub fn scan(settings: &Settings, stop: &AtomicBool) -> Index {
  let mut index = Index::default();
  if !settings.index_enabled || settings.index_paused {
    return index;
  }
  let mut pending = roots(settings);
  let mut seen = std::collections::HashSet::new();
  while let Some(path) = pending.pop() {
    if stop.load(Ordering::Relaxed) {
      break;
    }
    if !allowed(&path, settings) || !seen.insert(path.clone()) {
      continue;
    }
    let Ok(entries) = fs::read_dir(&path) else {
      index.skipped += 1;
      continue;
    };
    for entry in entries {
      if stop.load(Ordering::Relaxed) {
        break;
      }
      let Ok(entry) = entry else {
        index.skipped += 1;
        continue;
      };
      let child = entry.path();
      if !allowed(&child, settings) {
        continue;
      }
      let Ok(kind) = entry.file_type() else {
        index.skipped += 1;
        continue;
      };
      // Never follow symlinks: traversal stays under the configured roots.
      if kind.is_dir() {
        pending.push(child);
      } else if kind.is_file() {
        index.files.push(child);
      }
    }
  }
  index.files.sort();
  index.files.dedup();
  index.scanned = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap_or_default()
    .as_secs();
  index
}

pub fn save(index: &Index, expected: &Settings) -> Result<(), String> {
  let _lock = cache_lock()?;
  // A clear operation persists pause before taking this lock. An older worker
  // must not recreate data after the user has removed or excluded its roots.
  if Settings::load()? != *expected {
    return Ok(());
  }
  let path = cache_path()?;
  let parent = path.parent().ok_or("invalid cache path")?;
  fs::create_dir_all(parent).map_err(|e| e.to_string())?;
  let temporary = path.with_extension(format!("{}.tmp", std::process::id()));
  let result = (|| {
    let mut file = fs::OpenOptions::new()
      .create(true)
      .truncate(true)
      .write(true)
      .mode(0o600)
      .open(&temporary)
      .map_err(|e| e.to_string())?;
    file
      .write_all(&serde_json::to_vec(index).map_err(|e| e.to_string())?)
      .map_err(|e| e.to_string())?;
    file.sync_all().map_err(|e| e.to_string())?;
    fs::rename(&temporary, &path).map_err(|e| e.to_string())
  })();
  if result.is_err() {
    let _ = fs::remove_file(temporary);
  }
  result
}

fn file_entry(path: PathBuf) -> Entry {
  Entry {
    id: format!("file:{}", path.display()),
    title: path
      .file_name()
      .unwrap_or_default()
      .to_string_lossy()
      .into_owned(),
    description: path.to_string_lossy().into_owned(),
    category: "files".into(),
    icon: "text-x-generic".into(),
    keywords: Vec::new(),
    is_active: false,
    usage_id: String::new(),
    action: Action::File(path.to_string_lossy().into_owned()),
  }
}

pub fn entries(index: &Index, settings: &Settings) -> Vec<Entry> {
  let roots = roots(settings);
  let mut paths = Vec::new();
  if settings.index_enabled {
    paths.extend(
      index
        .files
        .iter()
        .filter(|path| {
          roots.iter().any(|root| path.starts_with(root))
            && allowed(path, settings)
            && fs::canonicalize(path).is_ok_and(|canonical| canonical == **path)
            && path.is_file()
        })
        .cloned(),
    );
  }
  if settings.recent_files {
    paths.extend(recent(settings));
  }
  paths.sort();
  paths.dedup();
  paths.into_iter().map(file_entry).collect()
}

fn recent(settings: &Settings) -> Vec<PathBuf> {
  let Some(data) = env::var_os("XDG_DATA_HOME")
    .map(PathBuf::from)
    .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share")))
  else {
    return Vec::new();
  };
  let Ok(contents) = fs::read(data.join("recently-used.xbel")) else {
    return Vec::new();
  };
  let mut reader = quick_xml::Reader::from_reader(contents.as_slice());
  let mut paths = Vec::new();
  loop {
    match reader.read_event() {
      Ok(quick_xml::events::Event::Start(event)) if event.name().as_ref() == b"bookmark" => {
        for attribute in event
          .attributes()
          .flatten()
          .filter(|attribute| attribute.key.as_ref() == b"href")
        {
          if let Ok(value) = attribute
            .decoded_and_normalized_value(quick_xml::XmlVersion::default(), reader.decoder())
            && let Ok(url) = url::Url::parse(&value)
            && let Ok(path) = url.to_file_path()
            && allowed(&path, settings)
            && path.is_file()
          {
            paths.push(path);
          }
        }
      }
      Ok(quick_xml::events::Event::Eof) | Err(_) => break,
      _ => {}
    }
  }
  paths
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn traversal_respects_hidden_exclusions_removals_and_symlinks() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("visible.txt"), "").unwrap();
    fs::write(root.path().join(".secret"), "").unwrap();
    fs::create_dir(root.path().join("excluded")).unwrap();
    fs::write(root.path().join("excluded/private"), "").unwrap();
    std::os::unix::fs::symlink("/etc", root.path().join("outside")).unwrap();
    let mut settings = Settings {
      index_enabled: true,
      recent_files: false,
      roots: vec![root.path().into()],
      exclusions: vec![root.path().join("excluded")],
      ..Settings::default()
    };
    let index = scan(&settings, &AtomicBool::new(false));
    assert_eq!(index.files.len(), 1);
    assert_eq!(entries(&index, &settings).len(), 1);
    settings.roots.clear();
    assert!(entries(&index, &settings).is_empty());
    settings.roots.push(root.path().into());
    fs::remove_file(&index.files[0]).unwrap();
    assert!(entries(&index, &settings).is_empty());
  }
}
