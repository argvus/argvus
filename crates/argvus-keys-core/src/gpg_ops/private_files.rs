//! Files that hold private key material: owner-only directories and atomic writes.

use super::runner::{Failure, io_failure};
use super::*;

#[cfg(unix)]
pub(super) fn restrict_dir(dir: &Path) {
  use std::os::unix::fs::PermissionsExt;
  let _ = fs::set_permissions(dir, fs::Permissions::from_mode(0o700));
}

/// Writes a file through a private temporary file in the same directory, renamed
/// into place when it is complete.
pub(super) fn write_private_atomic(target: &Path, data: &[u8]) -> Result<(), Failure> {
  let dir = target.parent().unwrap_or(Path::new("."));
  let temp = private_temp(dir, ".gpg-agent.conf")?;
  fs::write(&temp, data).map_err(io_failure)?;
  fs::rename(&temp, target).map_err(io_failure)
}

/// A new private file in `dir` with the given prefix, created exclusively.
pub(super) fn private_temp(dir: &Path, prefix: &str) -> Result<PathBuf, Failure> {
  use std::os::unix::fs::OpenOptionsExt;
  for attempt in 0..100u32 {
    let stamp = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .map(|duration| duration.as_nanos())
      .unwrap_or(0);
    let candidate = dir.join(format!("{prefix}.{}{stamp}{attempt}", std::process::id()));
    match fs::OpenOptions::new()
      .write(true)
      .create_new(true)
      .mode(0o600)
      .open(&candidate)
    {
      Ok(_) => return Ok(candidate),
      Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
      Err(error) => return Err(io_failure(error)),
    }
  }
  Err(Failure::new("error.gpg_export_failed", &[]))
}

/// The absolute form of a path the user typed.
pub(super) fn absolute(path: &str) -> String {
  if path.starts_with('/') {
    return path.to_string();
  }
  let current = env::current_dir().unwrap_or_default();
  current.join(path).to_string_lossy().into_owned()
}

pub(super) fn readable(file: &str) -> bool {
  Path::new(file).is_file() && fs::File::open(file).is_ok()
}
