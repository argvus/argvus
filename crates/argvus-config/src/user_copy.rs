//! The user copy of a packaged file, for the components that edit it.
//!
//! A mutable lookup returns the file in the ARGVUS user data, and creates that
//! copy first when it does not exist. The copy is taken from the first source
//! that has the file, in this order: the component-shaped user copy, the flat
//! pre-`data/` copy, the generated projection (moved, not copied), and the
//! packaged file. The waybar stylesheets are special: a generated shim that only
//! imports the packaged stylesheet is retired, and the packaged stylesheet is
//! copied instead, because the user copy is rewritten by every theme switch.

use crate::session_paths::{SessionPaths, legacy_relative};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// The mutable lookup of `paths_config`: script and documentation paths are read
/// from their live location, every other path gets its user copy first.
pub fn config_path(paths: &SessionPaths, relative: &str) -> Option<PathBuf> {
  let is_read_only = relative.starts_with("sh/")
    || relative.contains("/sh/")
    || relative.starts_with("docs/")
    || relative.contains("/docs/");
  if is_read_only {
    return Some(paths.read_config(relative));
  }
  ensure_user_copy(paths, relative).ok()
}

/// Returns the user copy of `relative`, creating it from the first available
/// source when it is missing. The result is the path the caller should edit.
pub fn ensure_user_copy(paths: &SessionPaths, relative: &str) -> io::Result<PathBuf> {
  let user = paths.user_config(relative);
  if exists_or_link(&user) {
    return Ok(user);
  }
  let legacy = legacy_relative(relative);

  // Retain edits from the component-shaped managed layout. Copy, not move,
  // so older consumers can still read the previous location.
  let component_user = paths.config_home.join("argvus").join(relative);
  if component_user != user && component_user.is_file() {
    return copy_into(&component_user, &user);
  }

  // Promote the pre-`data/` flat managed copy so an upgraded profile keeps
  // its user edits at the new location.
  let flat_user = paths.config_home.join("argvus").join(&legacy);
  if flat_user != user && flat_user.is_file() {
    return copy_into(&flat_user, &user);
  }

  // The generated projection may also be stored in the component shape.
  let mut generated = paths.generated_config(relative);
  if !exists_or_link(&generated) {
    let component_generated = paths
      .config_home
      .join("argvus/data/generated")
      .join(relative);
    if exists_or_link(&component_generated) {
      generated = component_generated;
    }
  }
  if exists_or_link(&generated) {
    if is_waybar_stylesheet(&legacy) && is_import_shim(&generated) {
      let system = paths.system_config(relative);
      if system.is_file() {
        let retired = PathBuf::from(format!("{}.retired", generated.display()));
        fs::rename(&generated, &retired)?;
        return copy_into(&system, &user);
      }
    }
    create_parent(&user)?;
    move_file(&generated, &user)?;
    return Ok(user);
  }

  let system = paths.system_config(relative);
  if system.is_dir() {
    fs::create_dir_all(&user)?;
    copy_tree(&system, &user)?;
  } else if system.is_file() {
    return copy_into(&system, &user);
  } else {
    create_parent(&user)?;
  }
  Ok(user)
}

fn exists_or_link(path: &Path) -> bool {
  fs::symlink_metadata(path).is_ok()
}

fn is_waybar_stylesheet(legacy: &str) -> bool {
  legacy.starts_with("waybar/") && legacy.ends_with(".css")
}

/// A generated stylesheet whose first line imports the packaged one. Such a
/// shim must not become the user copy.
fn is_import_shim(path: &Path) -> bool {
  fs::read_to_string(path)
    .ok()
    .and_then(|text| text.lines().next().map(str::to_string))
    .is_some_and(|first_line| first_line.starts_with("@import url(\"/usr/"))
}

fn create_parent(path: &Path) -> io::Result<()> {
  match path.parent() {
    Some(parent) => fs::create_dir_all(parent),
    None => Ok(()),
  }
}

fn copy_into(source: &Path, destination: &Path) -> io::Result<PathBuf> {
  create_parent(destination)?;
  fs::copy(source, destination)?;
  Ok(destination.to_path_buf())
}

/// Moves a file, falling back to copy and remove across filesystems.
fn move_file(source: &Path, destination: &Path) -> io::Result<()> {
  match fs::rename(source, destination) {
    Ok(()) => Ok(()),
    Err(error) if error.raw_os_error() == Some(libc::EXDEV) => {
      copy_link_or_file(source, destination)?;
      fs::remove_file(source)
    }
    Err(error) => Err(error),
  }
}

/// Copies a regular file, or recreates a symbolic link, overwriting the
/// destination, as `cp -R` does.
fn copy_link_or_file(source: &Path, destination: &Path) -> io::Result<()> {
  let metadata = fs::symlink_metadata(source)?;
  if metadata.file_type().is_symlink() {
    let _ = fs::remove_file(destination);
    std::os::unix::fs::symlink(fs::read_link(source)?, destination)
  } else {
    fs::copy(source, destination).map(|_| ())
  }
}

/// Copies the contents of `source` into `target`, overwriting files.
fn copy_tree(source: &Path, target: &Path) -> io::Result<()> {
  for entry in fs::read_dir(source)? {
    let entry = entry?;
    let destination = target.join(entry.file_name());
    if entry.file_type()?.is_dir() {
      fs::create_dir_all(&destination)?;
      copy_tree(&entry.path(), &destination)?;
    } else {
      copy_link_or_file(&entry.path(), &destination)?;
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
    let root = std::env::temp_dir().join(format!("argvus-user-copy-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("scratch");
    root
  }

  fn write(path: &Path, text: &str) {
    create_parent(path).expect("parent");
    fs::write(path, text).expect("write");
  }

  #[test]
  fn existing_user_copy_is_returned_untouched() {
    let root = scratch("existing");
    let paths = paths_under(&root);
    // The user copy uses the legacy name: taskbar/config is waybar/ on disk.
    let user = root.join("config/argvus/data/waybar/argvus-taskbar.css");
    write(&user, "user edit");
    write(
      &root.join("system/taskbar/config/argvus-taskbar.css"),
      "packaged",
    );
    assert_eq!(
      ensure_user_copy(&paths, "taskbar/config/argvus-taskbar.css").expect("copy"),
      user
    );
    assert_eq!(fs::read_to_string(&user).expect("read"), "user edit");
    let _ = fs::remove_dir_all(root);
  }

  #[test]
  fn packaged_file_is_copied_when_nothing_else_exists() {
    let root = scratch("packaged");
    let paths = paths_under(&root);
    write(&root.join("system/app/config/a.conf"), "packaged");
    let user = ensure_user_copy(&paths, "app/config/a.conf").expect("copy");
    assert_eq!(fs::read_to_string(&user).expect("read"), "packaged");
    let _ = fs::remove_dir_all(root);
  }

  #[test]
  fn missing_source_creates_only_the_parent() {
    let root = scratch("missing");
    let paths = paths_under(&root);
    let user = ensure_user_copy(&paths, "app/config/none.conf").expect("path");
    assert!(user.parent().expect("parent").is_dir());
    assert!(!user.exists());
    let _ = fs::remove_dir_all(root);
  }

  #[test]
  fn component_copy_is_preferred_over_the_packaged_file() {
    let root = scratch("component");
    let paths = paths_under(&root);
    write(&root.join("config/argvus/app/config/a.conf"), "component");
    write(&root.join("system/app/config/a.conf"), "packaged");
    let user = ensure_user_copy(&paths, "app/config/a.conf").expect("copy");
    assert_eq!(fs::read_to_string(&user).expect("read"), "component");
    let _ = fs::remove_dir_all(root);
  }

  #[test]
  fn generated_file_is_moved_to_the_user_copy() {
    let root = scratch("generated");
    let paths = paths_under(&root);
    let generated = root.join("config/argvus/data/generated/app/config/a.conf");
    write(&generated, "generated");
    let user = ensure_user_copy(&paths, "app/config/a.conf").expect("move");
    assert_eq!(fs::read_to_string(&user).expect("read"), "generated");
    assert!(!generated.exists(), "moved, not copied");
    let _ = fs::remove_dir_all(root);
  }

  #[test]
  fn waybar_import_shim_is_retired_and_the_packaged_css_is_used() {
    let root = scratch("shim");
    let paths = paths_under(&root);
    let generated = root.join("config/argvus/data/generated/taskbar/config/argvus-taskbar.css");
    write(
      &generated,
      "@import url(\"/usr/share/argvus/taskbar/config/argvus-taskbar.css\");\n",
    );
    write(
      &root.join("system/taskbar/config/argvus-taskbar.css"),
      "packaged css",
    );
    let user = ensure_user_copy(&paths, "taskbar/config/argvus-taskbar.css").expect("copy");
    assert_eq!(fs::read_to_string(&user).expect("read"), "packaged css");
    assert!(
      generated
        .with_file_name("argvus-taskbar.css.retired")
        .is_file()
    );
    let _ = fs::remove_dir_all(root);
  }

  #[test]
  fn packaged_directory_is_copied_recursively() {
    let root = scratch("dir");
    let paths = paths_under(&root);
    write(&root.join("system/app/themes/one/a.css"), "a");
    write(&root.join("system/app/themes/one/nested/b.css"), "b");
    let user = ensure_user_copy(&paths, "app/themes").expect("directory");
    assert_eq!(
      fs::read_to_string(user.join("one/nested/b.css")).expect("nested"),
      "b"
    );
    let _ = fs::remove_dir_all(root);
  }
}
