//! Package cache and mirror list files: cache entries, the clean-up preview and private writes.

use super::*;

/// Writes `contents` to a new file only the user can read.
pub(super) fn write_private_file(path: &Path, contents: &[u8]) -> Result<(), PackageError> {
  use std::io::Write as _;
  use std::os::unix::fs::OpenOptionsExt as _;
  let mut file = fs::OpenOptions::new()
    .write(true)
    .create_new(true)
    .mode(0o600)
    .open(path)
    .map_err(|error| PackageError::Command(format!("{}: {error}", path.display())))?;
  file
    .write_all(contents)
    .map_err(|error| PackageError::Command(format!("{}: {error}", path.display())))
}

/// Computes the exact candidate set for the three supported paccache
/// policies. It is intentionally independent of filesystem mutation so the
/// result can be shown before confirmation and recalculated immediately
/// before execution.
pub fn cache_preview(mut entries: Vec<CachePackage>, policy: CachePolicy) -> CachePreview {
  entries.sort_by(|a, b| a.name.cmp(&b.name).then(a.version.cmp(&b.version)));
  let mut candidates = Vec::new();
  let mut groups: std::collections::BTreeMap<String, Vec<CachePackage>> =
    std::collections::BTreeMap::new();
  for entry in entries {
    groups.entry(entry.name.clone()).or_default().push(entry);
  }
  for (_name, mut versions) in groups {
    versions.sort_by(|a, b| b.version.cmp(&a.version));
    let keep = match policy {
      CachePolicy::KeepThree => 3,
      CachePolicy::KeepOne => 1,
      CachePolicy::Uninstalled => 0,
    };
    for (index, entry) in versions.into_iter().enumerate() {
      if (policy == CachePolicy::Uninstalled && entry.installed) || index < keep {
        continue;
      }
      candidates.push(entry);
    }
  }
  let bytes = candidates.iter().map(|entry| entry.bytes).sum();
  CachePreview {
    policy,
    candidates,
    bytes,
  }
}

pub fn cache_entries(dir: &Path) -> Vec<CachePackage> {
  fs::read_dir(dir)
    .ok()
    .into_iter()
    .flatten()
    .filter_map(Result::ok)
    .filter_map(|e| {
      let m = e.metadata().ok()?;
      let n = e.file_name().to_string_lossy().into_owned();
      (n.ends_with(".pkg.tar.zst") || n.ends_with(".pkg.tar.xz") || n.ends_with(".pkg.tar.gz"))
        .then(|| CachePackage {
          name: n.clone(),
          version: String::new(),
          path: e.path().to_string_lossy().into_owned(),
          bytes: m.len(),
          installed: false,
        })
    })
    .collect()
}

pub(super) fn cache_dir() -> PathBuf {
  fs::read_to_string("/etc/pacman.conf")
    .ok()
    .and_then(|text| {
      text
        .lines()
        .find_map(|line| line.trim().strip_prefix("CacheDir = ").map(PathBuf::from))
    })
    .unwrap_or_else(|| PathBuf::from("/var/cache/pacman/pkg"))
}

pub fn mirrorlist_path() -> Option<PathBuf> {
  let config = fs::read_to_string("/etc/pacman.conf").ok()?;
  config
    .lines()
    .find_map(|line| {
      let value = line.trim().strip_prefix("Include = ")?;
      value.contains("mirrorlist").then(|| PathBuf::from(value))
    })
    .or_else(|| Some(PathBuf::from("/etc/pacman.d/mirrorlist")))
}
