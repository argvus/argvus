//! Display formats for durations and hour/minute/second conversion.

use std::time::Duration;

const SECONDS_PER_MINUTE: u64 = 60;
const SECONDS_PER_HOUR: u64 = 3_600;

/// A duration split into the fields of a timer editor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Hms {
  pub hours: u64,
  pub minutes: u64,
  pub seconds: u64,
}

impl Hms {
  /// Splits whole seconds; the fraction of a second is dropped.
  pub fn from_duration(duration: Duration) -> Self {
    let total = duration.as_secs();
    Self {
      hours: total / SECONDS_PER_HOUR,
      minutes: total % SECONDS_PER_HOUR / SECONDS_PER_MINUTE,
      seconds: total % SECONDS_PER_MINUTE,
    }
  }

  /// The duration of the fields. Minutes and seconds above 59 are accepted
  /// and carry over, as a spin button that overflows would expect.
  pub fn to_duration(self) -> Duration {
    Duration::from_secs(
      self
        .hours
        .saturating_mul(SECONDS_PER_HOUR)
        .saturating_add(self.minutes.saturating_mul(SECONDS_PER_MINUTE))
        .saturating_add(self.seconds),
    )
  }
}

/// `H:MM:SS` from one hour up, `MM:SS` below it.
fn clock_text(total_seconds: u64) -> String {
  let hms = Hms::from_duration(Duration::from_secs(total_seconds));
  if hms.hours > 0 {
    format!("{}:{:02}:{:02}", hms.hours, hms.minutes, hms.seconds)
  } else {
    format!("{:02}:{:02}", hms.minutes, hms.seconds)
  }
}

/// Time left on a countdown. Seconds are rounded up, so a 25-minute timer
/// shows `25:00` when it starts and reaches `00:00` only when it ends.
pub fn format_remaining(remaining: Duration) -> String {
  let mut seconds = remaining.as_secs();
  if remaining.subsec_nanos() > 0 {
    seconds += 1;
  }
  clock_text(seconds)
}

/// Elapsed time in whole seconds (rounded down).
pub fn format_elapsed(elapsed: Duration) -> String {
  clock_text(elapsed.as_secs())
}

/// Elapsed time with hundredths of a second, for the stopwatch.
pub fn format_precise(elapsed: Duration) -> String {
  let hundredths = elapsed.subsec_millis() / 10;
  format!("{}.{hundredths:02}", clock_text(elapsed.as_secs()))
}

/// Whole minutes, rounded to the nearest minute, for summaries.
pub fn rounded_minutes(duration: Duration) -> u64 {
  (duration.as_secs() + SECONDS_PER_MINUTE / 2) / SECONDS_PER_MINUTE
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn remaining_time_rounds_partial_seconds_up() {
    assert_eq!(format_remaining(Duration::from_secs(25 * 60)), "25:00");
    assert_eq!(format_remaining(Duration::from_millis(1_500)), "00:02");
    assert_eq!(format_remaining(Duration::from_millis(1)), "00:01");
    assert_eq!(format_remaining(Duration::ZERO), "00:00");
  }

  #[test]
  fn elapsed_time_rounds_down_and_shows_hours() {
    assert_eq!(format_elapsed(Duration::from_millis(59_999)), "00:59");
    assert_eq!(format_elapsed(Duration::from_secs(3_600 + 61)), "1:01:01");
  }

  #[test]
  fn precise_format_shows_hundredths() {
    assert_eq!(format_precise(Duration::from_millis(83_456)), "01:23.45");
    assert_eq!(
      format_precise(Duration::from_millis(3_723_070)),
      "1:02:03.07"
    );
  }

  #[test]
  fn hms_round_trips_and_carries_overflow() {
    let editor_fields = Hms {
      hours: 1,
      minutes: 2,
      seconds: 3,
    };
    assert_eq!(editor_fields.to_duration(), Duration::from_secs(3_723));
    assert_eq!(
      Hms::from_duration(Duration::from_secs(3_723)),
      editor_fields
    );

    let overflowing_fields = Hms {
      hours: 0,
      minutes: 90,
      seconds: 75,
    };
    assert_eq!(
      Hms::from_duration(overflowing_fields.to_duration()),
      Hms {
        hours: 1,
        minutes: 31,
        seconds: 15
      }
    );
  }

  #[test]
  fn rounded_minutes_rounds_to_nearest() {
    assert_eq!(rounded_minutes(Duration::from_secs(29)), 0);
    assert_eq!(rounded_minutes(Duration::from_secs(30)), 1);
    assert_eq!(rounded_minutes(Duration::from_secs(25 * 60 + 10)), 25);
  }
}
