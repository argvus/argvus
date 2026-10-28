//! Monotonic clock and the timing log shared with the greeter.

use super::*;

/// Appends `event` to `TIMING_LOG`. Timing is diagnostic, so a failure to
/// write never affects the surface.
pub(super) fn mark(event: &str) {
  let line = format!("monotonic_ns={} mark {event}\n", monotonic_nanos());
  if let Ok(mut file) = OpenOptions::new()
    .create(true)
    .append(true)
    .open(TIMING_LOG)
  {
    let _ = file.write_all(line.as_bytes());
  }
}

/// Milliseconds of `CLOCK_MONOTONIC`, the clock the GTK splash also reads, so
/// both surfaces show the same spinner frame at the same instant.
pub(super) fn monotonic_millis() -> u64 {
  monotonic_nanos() / 1_000_000
}

pub(super) fn monotonic_nanos() -> u64 {
  let mut now = libc::timespec {
    tv_sec: 0,
    tv_nsec: 0,
  };
  // SAFETY: `now` is a valid, writable timespec for the duration of the call.
  if unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut now) } != 0 {
    return 0;
  }
  let seconds = u64::try_from(now.tv_sec).unwrap_or_default();
  let nanoseconds = u64::try_from(now.tv_nsec).unwrap_or_default();
  seconds * 1_000_000_000 + nanoseconds
}
