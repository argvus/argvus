//! The countdown timer and the stopwatch, and their checkpoint on disk.
//!
//! Both run on boot time, so they keep counting through a suspension. A timer
//! that ended while the system slept rings as soon as the system wakes, and
//! says how late it is. After the process ended, both come back paused: they
//! could not ring or count while the process was gone.

use std::path::Path;
use std::time::Duration;

use argvus_time_core::elapsed::{
  Countdown, CountdownSnapshot, CountdownState, Stopwatch, StopwatchSnapshot,
};
use argvus_time_core::store::{self, StoreError};
use serde::{Deserialize, Serialize};

const VERSION: u32 = 1;
pub const FILE_NAME: &str = "timers.json";
/// A timer that ends more than this late (the system slept) says so.
pub const LATE_THRESHOLD: Duration = Duration::from_secs(5);
/// Longest timer accepted: 23:59:59.
pub const MAX_TIMER: Duration = Duration::from_secs(24 * 3_600 - 1);

/// The countdown timer. `None` until the user starts one.
#[derive(Debug, Clone, Default)]
pub struct Timer {
  countdown: Option<Countdown>,
}

/// A finished timer, ready to ring.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimerFinished {
  pub length: Duration,
  /// How late the end was noticed, when the system was asleep at the time.
  pub late_by: Option<Duration>,
}

impl Timer {
  pub fn countdown(&self) -> Option<&Countdown> {
    self.countdown.as_ref()
  }

  pub fn state(&self, now: Duration) -> CountdownState {
    self
      .countdown
      .as_ref()
      .map_or(CountdownState::Idle, |countdown| countdown.state(now))
  }

  pub fn is_running(&self, now: Duration) -> bool {
    self.state(now) == CountdownState::Running
  }

  /// Starts a new countdown of `length` (capped at [`MAX_TIMER`]).
  pub fn start(&mut self, length: Duration, now: Duration) -> bool {
    if length.is_zero() {
      return false;
    }
    let mut countdown = Countdown::new(length.min(MAX_TIMER));
    countdown.start(now);
    self.countdown = Some(countdown);
    true
  }

  pub fn pause(&mut self, now: Duration) {
    if let Some(countdown) = &mut self.countdown {
      countdown.pause(now);
    }
  }

  pub fn resume(&mut self, now: Duration) {
    if let Some(countdown) = &mut self.countdown {
      countdown.start(now);
    }
  }

  /// Starts the same length again from the beginning.
  pub fn restart(&mut self, now: Duration) {
    if let Some(length) = self.countdown.as_ref().map(Countdown::total) {
      self.start(length, now);
    }
  }

  pub fn cancel(&mut self) {
    self.countdown = None;
  }

  /// Returns the finished timer once and clears it, so it rings only once.
  pub fn take_finished(&mut self, now: Duration) -> Option<TimerFinished> {
    let countdown = self.countdown.as_ref()?;
    let overdue = countdown.overdue(now)?;
    let finished = TimerFinished {
      length: countdown.total(),
      late_by: (overdue > LATE_THRESHOLD).then_some(overdue),
    };
    self.countdown = None;
    Some(finished)
  }

  /// Time until the shown second changes, for scheduling the next update.
  pub fn until_next_second(&self, now: Duration) -> Option<Duration> {
    let countdown = self
      .countdown
      .as_ref()
      .filter(|countdown| countdown.state(now) == CountdownState::Running)?;
    let nanos = countdown.remaining(now).subsec_nanos();
    Some(if nanos == 0 {
      Duration::from_secs(1)
    } else {
      Duration::from_nanos(u64::from(nanos))
    })
  }
}

/// What was restored from the checkpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Restored {
  pub timer_paused: bool,
  pub stopwatch_paused: bool,
}

/// The checkpoint file.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct TimersState {
  #[serde(default)]
  pub timer: Option<CountdownSnapshot>,
  #[serde(default)]
  pub stopwatch: StopwatchSnapshot,
}

impl TimersState {
  pub fn capture(timer: &Timer, stopwatch: &Stopwatch, now: Duration) -> Self {
    Self {
      timer: timer
        .countdown
        .as_ref()
        .filter(|countdown| countdown.state(now) != CountdownState::Finished)
        .map(|countdown| countdown.snapshot(now)),
      stopwatch: stopwatch.snapshot(now),
    }
  }

  pub fn restore(&self) -> (Timer, Stopwatch, Restored) {
    let timer = Timer {
      countdown: self.timer.as_ref().map(Countdown::from_snapshot),
    };
    let restored = Restored {
      timer_paused: self.timer.is_some_and(|timer| timer.running),
      stopwatch_paused: self.stopwatch.running,
    };
    (timer, Stopwatch::from_snapshot(&self.stopwatch), restored)
  }

  pub fn load(path: &Path) -> Result<store::Loaded<Self>, StoreError> {
    store::load_json(path, VERSION)
  }

  pub fn save(&self, path: &Path) -> Result<(), StoreError> {
    store::save_json(path, VERSION, self)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn at(seconds: u64) -> Duration {
    Duration::from_secs(seconds)
  }

  #[test]
  fn timer_rings_once_when_it_ends() {
    let mut timer = Timer::default();
    assert!(timer.start(at(300), at(0)));
    assert_eq!(timer.take_finished(at(299)), None);
    let finished = timer.take_finished(at(300)).expect("finished");
    assert_eq!(finished.length, at(300));
    assert_eq!(finished.late_by, None);
    assert_eq!(
      timer.take_finished(at(301)),
      None,
      "a finished timer rings once"
    );
    assert_eq!(timer.state(at(301)), CountdownState::Idle);
  }

  #[test]
  fn timer_that_ended_during_sleep_reports_the_delay() {
    let mut timer = Timer::default();
    timer.start(at(60), at(0));
    // Boot time kept counting while the system slept for ten minutes.
    let finished = timer.take_finished(at(660)).expect("finished");
    assert_eq!(finished.late_by, Some(at(600)));
  }

  #[test]
  fn pause_resume_and_restart() {
    let mut timer = Timer::default();
    timer.start(at(100), at(0));
    timer.pause(at(40));
    assert_eq!(timer.countdown().unwrap().remaining(at(500)), at(60));
    timer.resume(at(500));
    assert_eq!(timer.countdown().unwrap().remaining(at(510)), at(50));
    timer.restart(at(510));
    assert_eq!(timer.countdown().unwrap().remaining(at(510)), at(100));
    assert!(
      !timer.start(Duration::ZERO, at(0)),
      "a zero timer does not start"
    );
  }

  #[test]
  fn long_timers_are_capped() {
    let mut timer = Timer::default();
    timer.start(at(100 * 3_600), at(0));
    assert_eq!(timer.countdown().unwrap().total(), MAX_TIMER);
  }

  #[test]
  fn checkpoint_restores_running_clocks_paused() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join(FILE_NAME);
    let mut timer = Timer::default();
    timer.start(at(600), at(0));
    let mut stopwatch = Stopwatch::new();
    stopwatch.start(at(0));
    stopwatch.lap(at(30));
    TimersState::capture(&timer, &stopwatch, at(120))
      .save(&path)
      .unwrap();

    let (timer, stopwatch, restored) = TimersState::load(&path).unwrap().value.restore();
    assert!(restored.timer_paused && restored.stopwatch_paused);
    assert_eq!(timer.state(at(0)), CountdownState::Paused);
    assert_eq!(timer.countdown().unwrap().remaining(at(9_999)), at(480));
    assert!(!stopwatch.is_running());
    assert_eq!(stopwatch.elapsed(at(0)), at(120));
    assert_eq!(stopwatch.laps().len(), 1);
  }

  #[test]
  fn finished_timer_is_not_checkpointed() {
    let mut timer = Timer::default();
    timer.start(at(10), at(0));
    let state = TimersState::capture(&timer, &Stopwatch::new(), at(20));
    assert_eq!(state.timer, None);
  }
}
