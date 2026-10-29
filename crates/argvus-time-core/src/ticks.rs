//! Clock readings used to measure durations.
//!
//! Durations are measured on `CLOCK_BOOTTIME`: it never jumps when the wall
//! clock or the time zone changes, and, unlike `CLOCK_MONOTONIC` (and
//! [`std::time::Instant`]), it keeps counting while the system is suspended.
//! Comparing how far the two clocks moved between two readings tells how long
//! the system slept, without asking logind.

use std::time::Duration;

use rustix::time::{ClockId, Timespec, clock_gettime};

/// One reading of both clocks, taken together.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ticks {
  /// Time since boot, without suspended time.
  pub monotonic: Duration,
  /// Time since boot, including suspended time. Timers run on this clock.
  pub boottime: Duration,
}

impl Ticks {
  /// Reads both clocks now.
  pub fn now() -> Self {
    Self {
      monotonic: read(ClockId::Monotonic),
      boottime: read(ClockId::Boottime),
    }
  }

  /// How long the system was suspended between `earlier` and `self`.
  pub fn suspended_since(self, earlier: Ticks) -> Duration {
    let boot_delta = self.boottime.saturating_sub(earlier.boottime);
    let monotonic_delta = self.monotonic.saturating_sub(earlier.monotonic);
    boot_delta.saturating_sub(monotonic_delta)
  }
}

/// Suspensions shorter than this are reading noise, not a real sleep.
pub const SUSPEND_THRESHOLD: Duration = Duration::from_secs(3);

/// Whether the system slept between two readings.
pub fn was_suspended(earlier: Ticks, later: Ticks) -> bool {
  later.suspended_since(earlier) >= SUSPEND_THRESHOLD
}

fn read(clock: ClockId) -> Duration {
  timespec_to_duration(clock_gettime(clock))
}

fn timespec_to_duration(spec: Timespec) -> Duration {
  // Both clocks start at boot, so the fields are never negative.
  let seconds = u64::try_from(spec.tv_sec).unwrap_or(0);
  let nanos = u32::try_from(spec.tv_nsec).unwrap_or(0);
  Duration::new(seconds, nanos)
}

#[cfg(test)]
mod tests {
  use super::*;

  fn ticks(monotonic_secs: u64, boottime_secs: u64) -> Ticks {
    Ticks {
      monotonic: Duration::from_secs(monotonic_secs),
      boottime: Duration::from_secs(boottime_secs),
    }
  }

  #[test]
  fn suspension_is_the_gap_between_boottime_and_monotonic() {
    let before_sleep = ticks(100, 100);
    let after_sleep = ticks(101, 701);
    assert_eq!(
      after_sleep.suspended_since(before_sleep),
      Duration::from_secs(600)
    );
    assert!(was_suspended(before_sleep, after_sleep));
  }

  #[test]
  fn normal_ticks_are_not_a_suspension() {
    let earlier = ticks(100, 150);
    let later = ticks(101, 151);
    assert_eq!(later.suspended_since(earlier), Duration::ZERO);
    assert!(!was_suspended(earlier, later));
  }

  #[test]
  fn live_readings_are_ordered() {
    let first = Ticks::now();
    let second = Ticks::now();
    assert!(second.monotonic >= first.monotonic);
    assert!(second.boottime >= first.boottime);
    assert!(first.boottime >= first.monotonic);
  }
}
