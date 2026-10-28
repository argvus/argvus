//! Advisory file lock that serializes writers of the configuration.

use super::*;

pub(super) fn lock_file(file: &File) -> io::Result<()> {
  let result = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX) };
  if result == 0 {
    Ok(())
  } else {
    Err(io::Error::last_os_error())
  }
}

pub(super) struct LockGuard<'a>(pub(super) &'a File);

impl Drop for LockGuard<'_> {
  fn drop(&mut self) {
    let _ = unsafe { libc::flock(self.0.as_raw_fd(), libc::LOCK_UN) };
  }
}
