//! The one-time move of the per-theme copies older switches left in the user
//! data (`data/<app>/themes/<theme>/…`).
//!
//! Every themed file now comes from the theme build, so these copies are no
//! longer read. They are moved, not deleted, into
//! `data/backups/theme-copies-<seconds>/`, keeping their paths: a file the user
//! edited can be turned into a user theme (`overrides/` in a theme of
//! `$XDG_DATA_HOME/argvus/themes`). A marker keeps the move from running again.

use crate::session_paths::SessionPaths;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// The per-theme directories of the user data, relative to it.
const LEGACY_DIRECTORIES: [&str; 14] = [
  "waybar/themes",
  "rofi/themes",
  "hypr/themes",
  "tui/themes",
  "quickshell/argvus-control-panel/themes",
  "removable-devices/themes",
  "qt6ct/colors",
  "dunst/themes",
  "lock/config/themes",
  "terminal/config/themes",
  "terminal/kitty/themes",
  "system-monitor/config/btop/themes",
  "gtk-3.0/themes",
  "gtk-4.0/themes",
];

/// Written once the copies are moved.
const MARKER: &str = "state/theme-copies-migrated";

/// Moves the legacy copies, once. Returns the backup directory when something
/// was moved.
pub fn migrate(paths: &SessionPaths) -> io::Result<Option<PathBuf>> {
  let marker = paths.data_home.join(MARKER);
  if marker.exists() {
    return Ok(None);
  }
  let seconds = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map_or(0, |elapsed| elapsed.as_secs());
  let backup = paths
    .data_home
    .join(format!("backups/theme-copies-{seconds}"));
  let mut moved = false;
  for relative in LEGACY_DIRECTORIES {
    let source = paths.data_home.join(relative);
    if !source.is_dir() {
      continue;
    }
    let destination = backup.join(relative);
    if let Some(parent) = destination.parent() {
      fs::create_dir_all(parent)?;
    }
    fs::rename(&source, &destination)?;
    moved = true;
  }
  if let Some(parent) = marker.parent() {
    fs::create_dir_all(parent)?;
  }
  fs::write(&marker, format!("{seconds}\n"))?;
  Ok(moved.then_some(backup))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn copies_move_once_and_keep_their_paths() {
    let root = std::env::temp_dir().join(format!("argvus-theme-migration-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let paths = SessionPaths {
      config_home: root.join("config"),
      data_home: root.join("config/argvus/data"),
      system_config: root.join("system"),
    };
    let copy = paths.data_home.join("waybar/themes/sunset/theme.css");
    fs::create_dir_all(copy.parent().unwrap()).unwrap();
    fs::write(&copy, "edited").unwrap();
    fs::write(paths.data_home.join("waybar/argvus-taskbar.css"), "kept").unwrap();

    let backup = migrate(&paths).unwrap().expect("something moved");
    assert!(!copy.exists());
    assert_eq!(
      fs::read_to_string(backup.join("waybar/themes/sunset/theme.css")).unwrap(),
      "edited"
    );
    assert!(paths.data_home.join("waybar/argvus-taskbar.css").is_file());

    fs::create_dir_all(paths.data_home.join("rofi/themes/x")).unwrap();
    assert_eq!(migrate(&paths).unwrap(), None, "runs once");
    assert!(paths.data_home.join("rofi/themes/x").is_dir());
    let _ = fs::remove_dir_all(root);
  }
}
