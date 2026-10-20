//! System primitives shared by the ARGVUS binaries that run during the session
//! start: the loading splash and the login greeter.

/// Returns a monotonic timestamp in nanoseconds, or `0` when the clock cannot be read.
///
/// `CLOCK_MONOTONIC` has no relation to wall-clock time and is therefore safe for
/// measuring a transition even when NTP adjusts the system clock.
pub fn monotonic_ns() -> u128 {
  let mut value = libc::timespec {
    tv_sec: 0,
    tv_nsec: 0,
  };
  // SAFETY: `clock_gettime` only writes into `value`, a valid `timespec` that this
  // function owns exclusively for the duration of the call.
  if unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut value) } == 0 {
    value.tv_sec as u128 * 1_000_000_000 + value.tv_nsec as u128
  } else {
    0
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn monotonic_clock_reads_a_nonzero_value() {
    assert_ne!(monotonic_ns(), 0);
  }

  #[test]
  fn monotonic_clock_never_goes_backwards() {
    let first_reading = monotonic_ns();
    let second_reading = monotonic_ns();

    assert!(second_reading >= first_reading);
  }
}
