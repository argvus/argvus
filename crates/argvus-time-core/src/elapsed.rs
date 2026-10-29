//! Pausable elapsed time, and the countdown and stopwatch built on it.
//!
//! Every reading is computed from the boot-time instant the clock last started
//! and the time it had accumulated before, never by adding up ticks, so a late
//! or skipped UI update cannot make a timer drift.
//!
//! All `now` and `at` arguments are [`crate::ticks::Ticks::boottime`] values.

use std::time::Duration;

use serde::{Deserialize, Serialize};

/// Elapsed time that can be started, paused and reset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Accumulator {
  elapsed_before: Duration,
  running_since: Option<Duration>,
}

impl Accumulator {
  /// A clock that never ran.
  pub const fn new() -> Self {
    Self {
      elapsed_before: Duration::ZERO,
      running_since: None,
    }
  }

  /// A paused clock that already counted `elapsed`.
  pub const fn paused_at(elapsed: Duration) -> Self {
    Self {
      elapsed_before: elapsed,
      running_since: None,
    }
  }

  pub fn is_running(&self) -> bool {
    self.running_since.is_some()
  }

  pub fn elapsed(&self, now: Duration) -> Duration {
    match self.running_since {
      Some(since) => self.elapsed_before + now.saturating_sub(since),
      None => self.elapsed_before,
    }
  }

  /// Starts or resumes; does nothing when already running.
  pub fn start(&mut self, now: Duration) {
    if self.running_since.is_none() {
      self.running_since = Some(now);
    }
  }

  /// Pauses at `now`.
  pub fn pause(&mut self, now: Duration) {
    self.pause_at(now);
  }

  /// Pauses as if it had been paused at `at`, an instant in the past. Used
  /// after a suspension, to drop the time the system slept.
  pub fn pause_at(&mut self, at: Duration) {
    if let Some(since) = self.running_since.take() {
      self.elapsed_before += at.saturating_sub(since);
    }
  }

  pub fn reset(&mut self) {
    *self = Self::new();
  }
}

/// What a countdown is doing at a given instant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CountdownState {
  /// Never started, or reset.
  Idle,
  Running,
  Paused,
  /// The whole duration has elapsed.
  Finished,
}

/// A countdown of a fixed duration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Countdown {
  total: Duration,
  clock: Accumulator,
}

impl Countdown {
  pub const fn new(total: Duration) -> Self {
    Self {
      total,
      clock: Accumulator::new(),
    }
  }

  pub fn total(&self) -> Duration {
    self.total
  }

  pub fn state(&self, now: Duration) -> CountdownState {
    let elapsed = self.clock.elapsed(now);
    if elapsed >= self.total && (self.clock.is_running() || elapsed > Duration::ZERO) {
      CountdownState::Finished
    } else if self.clock.is_running() {
      CountdownState::Running
    } else if elapsed > Duration::ZERO {
      CountdownState::Paused
    } else {
      CountdownState::Idle
    }
  }

  pub fn elapsed(&self, now: Duration) -> Duration {
    self.clock.elapsed(now).min(self.total)
  }

  pub fn remaining(&self, now: Duration) -> Duration {
    self.total.saturating_sub(self.clock.elapsed(now))
  }

  /// How long ago the countdown ended, or `None` while it has not ended.
  pub fn overdue(&self, now: Duration) -> Option<Duration> {
    let elapsed = self.clock.elapsed(now);
    (self.state(now) == CountdownState::Finished).then(|| elapsed.saturating_sub(self.total))
  }

  /// Fraction elapsed, from 0.0 to 1.0.
  pub fn progress(&self, now: Duration) -> f64 {
    if self.total.is_zero() {
      return 1.0;
    }
    (self.elapsed(now).as_secs_f64() / self.total.as_secs_f64()).clamp(0.0, 1.0)
  }

  pub fn start(&mut self, now: Duration) {
    if self.state(now) != CountdownState::Finished {
      self.clock.start(now);
    }
  }

  pub fn pause(&mut self, now: Duration) {
    self.clock.pause(now);
  }

  pub fn pause_at(&mut self, at: Duration) {
    self.clock.pause_at(at);
  }

  pub fn reset(&mut self) {
    self.clock.reset();
  }

  /// The persistable state. Boot-time instants mean nothing after a restart,
  /// so only the elapsed time is kept.
  pub fn snapshot(&self, now: Duration) -> CountdownSnapshot {
    CountdownSnapshot {
      total_ms: millis(self.total),
      elapsed_ms: millis(self.elapsed(now)),
      running: self.clock.is_running(),
    }
  }

  /// Restores a snapshot as a paused countdown. A countdown that was running
  /// when the snapshot was taken could not notify while the process was gone,
  /// so it comes back paused at the point it had reached.
  pub fn from_snapshot(snapshot: &CountdownSnapshot) -> Self {
    let total = Duration::from_millis(snapshot.total_ms);
    let elapsed = Duration::from_millis(snapshot.elapsed_ms).min(total);
    Self {
      total,
      clock: Accumulator::paused_at(elapsed),
    }
  }
}

/// Stored form of a [`Countdown`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CountdownSnapshot {
  pub total_ms: u64,
  pub elapsed_ms: u64,
  pub running: bool,
}

/// One lap of the stopwatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lap {
  /// 1 for the first lap.
  pub number: u32,
  /// Time since the previous lap.
  pub split_ms: u64,
  /// Time since the stopwatch started.
  pub total_ms: u64,
}

impl Lap {
  pub fn split(&self) -> Duration {
    Duration::from_millis(self.split_ms)
  }

  pub fn total(&self) -> Duration {
    Duration::from_millis(self.total_ms)
  }
}

/// A stopwatch with laps.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Stopwatch {
  clock: Accumulator,
  laps: Vec<Lap>,
}

impl Stopwatch {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn is_running(&self) -> bool {
    self.clock.is_running()
  }

  /// Whether it has anything to reset.
  pub fn has_started(&self, now: Duration) -> bool {
    self.clock.is_running() || self.clock.elapsed(now) > Duration::ZERO
  }

  pub fn elapsed(&self, now: Duration) -> Duration {
    self.clock.elapsed(now)
  }

  /// Laps, the most recent last.
  pub fn laps(&self) -> &[Lap] {
    &self.laps
  }

  pub fn start(&mut self, now: Duration) {
    self.clock.start(now);
  }

  pub fn pause(&mut self, now: Duration) {
    self.clock.pause(now);
  }

  pub fn reset(&mut self) {
    self.clock.reset();
    self.laps.clear();
  }

  /// Records a lap. Only a running stopwatch takes laps.
  pub fn lap(&mut self, now: Duration) -> Option<Lap> {
    if !self.clock.is_running() {
      return None;
    }
    let total_ms = millis(self.clock.elapsed(now));
    let previous_ms = self.laps.last().map_or(0, |lap| lap.total_ms);
    let lap = Lap {
      number: u32::try_from(self.laps.len() + 1).unwrap_or(u32::MAX),
      split_ms: total_ms.saturating_sub(previous_ms),
      total_ms,
    };
    self.laps.push(lap);
    Some(lap)
  }

  pub fn snapshot(&self, now: Duration) -> StopwatchSnapshot {
    StopwatchSnapshot {
      elapsed_ms: millis(self.clock.elapsed(now)),
      running: self.clock.is_running(),
      laps: self.laps.clone(),
    }
  }

  /// Restores a snapshot paused, for the same reason as
  /// [`Countdown::from_snapshot`].
  pub fn from_snapshot(snapshot: &StopwatchSnapshot) -> Self {
    Self {
      clock: Accumulator::paused_at(Duration::from_millis(snapshot.elapsed_ms)),
      laps: snapshot.laps.clone(),
    }
  }
}

/// Stored form of a [`Stopwatch`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct StopwatchSnapshot {
  pub elapsed_ms: u64,
  pub running: bool,
  #[serde(default)]
  pub laps: Vec<Lap>,
}

fn millis(duration: Duration) -> u64 {
  u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
  use super::*;

  fn at(seconds: u64) -> Duration {
    Duration::from_secs(seconds)
  }

  #[test]
  fn accumulator_counts_only_running_time() {
    let mut clock = Accumulator::new();
    clock.start(at(10));
    clock.pause(at(15));
    clock.start(at(100));
    assert_eq!(clock.elapsed(at(103)), at(8));
    clock.start(at(200));
    assert_eq!(clock.elapsed(at(103)), at(8), "a second start is ignored");
  }

  #[test]
  fn countdown_remaining_comes_from_instants_not_ticks() {
    let mut countdown = Countdown::new(at(60));
    assert_eq!(countdown.state(at(0)), CountdownState::Idle);
    countdown.start(at(1_000));
    // A UI that missed every update still reads the right remaining time.
    assert_eq!(countdown.remaining(at(1_045)), at(15));
    assert_eq!(countdown.state(at(1_045)), CountdownState::Running);
    assert!((countdown.progress(at(1_030)) - 0.5).abs() < f64::EPSILON);
  }

  #[test]
  fn countdown_pause_and_resume_keep_the_remaining_time() {
    let mut countdown = Countdown::new(at(60));
    countdown.start(at(0));
    countdown.pause(at(20));
    assert_eq!(countdown.state(at(500)), CountdownState::Paused);
    assert_eq!(countdown.remaining(at(500)), at(40));
    countdown.start(at(500));
    assert_eq!(countdown.remaining(at(530)), at(10));
  }

  #[test]
  fn countdown_finishes_and_reports_how_late_it_is() {
    let mut countdown = Countdown::new(at(60));
    countdown.start(at(0));
    assert_eq!(countdown.overdue(at(59)), None);
    assert_eq!(countdown.state(at(60)), CountdownState::Finished);
    assert_eq!(countdown.overdue(at(75)), Some(at(15)));
    assert_eq!(countdown.remaining(at(75)), Duration::ZERO);
    assert_eq!(countdown.elapsed(at(75)), at(60));
  }

  #[test]
  fn pause_at_drops_time_after_the_given_instant() {
    let mut countdown = Countdown::new(at(1_500));
    countdown.start(at(0));
    // The system slept from 600 to 4_200; the session pauses at 600.
    countdown.pause_at(at(600));
    assert_eq!(countdown.state(at(4_200)), CountdownState::Paused);
    assert_eq!(countdown.remaining(at(4_200)), at(900));
  }

  #[test]
  fn running_snapshot_restores_paused_at_the_same_point() {
    let mut countdown = Countdown::new(at(1_500));
    countdown.start(at(10));
    let snapshot = countdown.snapshot(at(310));
    assert!(snapshot.running);

    let restored = Countdown::from_snapshot(&snapshot);
    assert_eq!(restored.state(at(0)), CountdownState::Paused);
    assert_eq!(restored.remaining(at(99_999)), at(1_200));
  }

  #[test]
  fn stopwatch_records_splits_and_totals() {
    let mut stopwatch = Stopwatch::new();
    assert_eq!(
      stopwatch.lap(at(0)),
      None,
      "a stopped stopwatch takes no lap"
    );
    stopwatch.start(at(0));
    let first_lap = stopwatch.lap(at(30)).expect("first lap");
    let second_lap = stopwatch.lap(at(75)).expect("second lap");
    assert_eq!(
      (first_lap.number, first_lap.split(), first_lap.total()),
      (1, at(30), at(30))
    );
    assert_eq!(
      (second_lap.number, second_lap.split(), second_lap.total()),
      (2, at(45), at(75))
    );
    stopwatch.reset();
    assert!(stopwatch.laps().is_empty());
    assert!(!stopwatch.has_started(at(80)));
  }

  #[test]
  fn stopwatch_snapshot_keeps_laps_and_restores_paused() {
    let mut stopwatch = Stopwatch::new();
    stopwatch.start(at(0));
    stopwatch.lap(at(10));
    let snapshot = stopwatch.snapshot(at(25));
    let restored = Stopwatch::from_snapshot(&snapshot);
    assert!(!restored.is_running());
    assert_eq!(restored.elapsed(at(1_000)), at(25));
    assert_eq!(restored.laps().len(), 1);
  }
}
