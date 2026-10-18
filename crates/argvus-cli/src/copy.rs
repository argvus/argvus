//! Recursive copy with the semantics of `cp -R`: symbolic links are copied as
//! links, and file permissions come along with the content.

use std::fs;
use std::io;
use std::os::unix::fs::symlink;
use std::path::Path;

/// Copies the contents of `source` into `destination`, creating it when needed.
/// Existing files are overwritten, as `cp -R` does.
pub fn copy_tree(source: &Path, destination: &Path) -> io::Result<()> {
  fs::create_dir_all(destination)?;
  for entry in fs::read_dir(source)? {
    let entry = entry?;
    let file_type = entry.file_type()?;
    let entry_source = entry.path();
    let entry_destination = destination.join(entry.file_name());
    if file_type.is_symlink() {
      // `cp -R` recreates a link instead of following it. An existing entry is
      // replaced, as `cp` does with an existing destination.
      if entry_destination.symlink_metadata().is_ok() {
        fs::remove_file(&entry_destination)?;
      }
      symlink(fs::read_link(&entry_source)?, &entry_destination)?;
    } else if file_type.is_dir() {
      copy_tree(&entry_source, &entry_destination)?;
    } else {
      fs::copy(&entry_source, &entry_destination)?;
    }
  }
  Ok(())
}
