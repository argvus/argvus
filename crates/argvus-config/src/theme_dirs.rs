//! The per-theme directories a theme switch reads and writes.
//!
//! A component's theme directory lives in the ARGVUS user data. Before it is
//! used, the files that are missing are filled from the native override, the
//! generated projection and the packaged copy, in that order. Files that
//! already exist are never overwritten, so user edits survive every switch.

use crate::session_paths::SessionPaths;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// The directory `<relative>/<theme>` in the user data, with its missing files
/// filled in. Returns the parent of that directory (the `<relative>` root), or
/// `None` when no copy of the theme exists anywhere.
pub fn theme_parent(paths: &SessionPaths, relative: &str, theme: &str) -> Option<PathBuf> {
  let theme_relative = format!("{relative}/{theme}");
  let target = paths.user_config(&theme_relative);
  let sources = [
    paths.override_config(&theme_relative),
    paths.generated_config(&theme_relative),
    paths.system_config(&theme_relative),
  ];
  for source in sources {
    if source == target || !source.is_dir() {
      continue;
    }
    fs::create_dir_all(&target).ok()?;
    copy_missing(&source, &target).ok()?;
  }
  if !target.is_dir() {
    return None;
  }
  target.parent().map(Path::to_path_buf)
}

/// Copies every file of `source` that `target` does not have, recursively.
/// This is `cp -R --update=none`: an existing file is kept as it is.
fn copy_missing(source: &Path, target: &Path) -> io::Result<()> {
  for entry in fs::read_dir(source)? {
    let entry = entry?;
    let destination = target.join(entry.file_name());
    let file_type = entry.file_type()?;
    if file_type.is_dir() {
      fs::create_dir_all(&destination)?;
      copy_missing(&entry.path(), &destination)?;
    } else if file_type.is_symlink() {
      if fs::symlink_metadata(&destination).is_err() {
        std::os::unix::fs::symlink(fs::read_link(entry.path())?, &destination)?;
      }
    } else if !destination.exists() {
      fs::copy(entry.path(), &destination)?;
    }
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  fn paths_under(root: &Path) -> SessionPaths {
    SessionPaths {
      config_home: root.join("config"),
      data_home: root.join("config/argvus/data"),
      system_config: root.join("system"),
    }
  }

  fn scratch(name: &str) -> PathBuf {
    let root =
      std::env::temp_dir().join(format!("argvus-theme-dirs-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    root
  }

  #[test]
  fn missing_files_are_filled_and_existing_ones_are_kept() {
    let root = scratch("fill");
    let paths = paths_under(&root);
    let system_theme = root.join("system/appearance/config/gtk-3.0/themes/frost");
    fs::create_dir_all(system_theme.join("nested")).expect("system tree");
    fs::write(system_theme.join("gtk.css"), "packaged").expect("packaged");
    fs::write(system_theme.join("nested/extra.css"), "packaged nested").expect("nested");
    let user_theme = root.join("config/argvus/data/gtk-3.0/themes/frost");
    fs::create_dir_all(&user_theme).expect("user tree");
    fs::write(user_theme.join("gtk.css"), "user edit").expect("user edit");

    let parent =
      theme_parent(&paths, "appearance/config/gtk-3.0/themes", "frost").expect("theme directory");
    assert_eq!(parent, root.join("config/argvus/data/gtk-3.0/themes"));
    assert_eq!(
      fs::read_to_string(user_theme.join("gtk.css")).expect("kept"),
      "user edit"
    );
    assert_eq!(
      fs::read_to_string(user_theme.join("nested/extra.css")).expect("filled"),
      "packaged nested"
    );
    let _ = fs::remove_dir_all(root);
  }

  #[test]
  fn override_wins_over_packaged_copy_for_the_same_file() {
    let root = scratch("override");
    let paths = paths_under(&root);
    let override_theme = root.join("config/hypr/themes/frost");
    fs::create_dir_all(&override_theme).expect("override");
    fs::write(override_theme.join("hyprtoolkit.conf"), "override").expect("override file");
    let system_theme = root.join("system/appearance/config/hypr/themes/frost");
    fs::create_dir_all(&system_theme).expect("system");
    fs::write(system_theme.join("hyprtoolkit.conf"), "packaged").expect("packaged file");
    fs::write(system_theme.join("wallpaper.png"), "").expect("packaged only");

    theme_parent(&paths, "appearance/config/hypr/themes", "frost").expect("theme directory");
    let copied = root.join("config/argvus/data/hypr/themes/frost");
    assert_eq!(
      fs::read_to_string(copied.join("hyprtoolkit.conf")).expect("copy"),
      "override"
    );
    assert!(copied.join("wallpaper.png").is_file());
    let _ = fs::remove_dir_all(root);
  }

  #[test]
  fn unknown_theme_has_no_directory() {
    let root = scratch("unknown");
    let paths = paths_under(&root);
    fs::create_dir_all(&root).expect("root");
    assert_eq!(
      theme_parent(&paths, "appearance/config/hypr/themes", "nope"),
      None
    );
    let _ = fs::remove_dir_all(root);
  }
}
