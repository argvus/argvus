//! Usage of a mounted filesystem, read with `statvfs(3)`.

/// Space of a mounted filesystem, in bytes.
pub struct FilesystemUsage {
  pub total: u64,
  pub used: u64,
  pub free: u64,
  pub read_only: bool,
}

// Filesystem usage for a mount point via statvfs. Also reports read-only state.
pub fn statvfs_usage(mountpoint: &str) -> Option<FilesystemUsage> {
  // SAFETY: `statvfs` is a plain C struct of integers, so the all-zero value is
  // valid; the kernel overwrites it below.
  let mut stats: libc::statvfs = unsafe { std::mem::zeroed() };
  let c_path = std::ffi::CString::new(mountpoint).ok()?;
  // SAFETY: `c_path` is a NUL-terminated string that outlives the call and
  // `stats` is a valid, writable `statvfs`.
  if unsafe { libc::statvfs(c_path.as_ptr(), &mut stats) } != 0 {
    return None;
  }
  let fragment_size = if stats.f_frsize > 0 {
    stats.f_frsize as u64
  } else {
    512
  };
  let total = (stats.f_blocks as u64).wrapping_mul(fragment_size);
  let free = (stats.f_bavail as u64).wrapping_mul(fragment_size);
  let used = total.saturating_sub(free);
  let read_only = (stats.f_flag as u64 & libc::ST_RDONLY) != 0;
  Some(FilesystemUsage {
    total,
    used,
    free,
    read_only,
  })
}
