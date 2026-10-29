//! The focus session: the Pomodoro cycle, the countdown of the current phase
//! and what happens when a phase ends.
//!
//! The engine does no I/O and reads no clock: the application passes the
//! readings in (`now` is a boot-time instant, `wall` the current UTC time),
//! and reacts to the returned [`Event`]s with notifications, sounds and
//! history records.
//!
//! Lifecycle rules:
//! - A suspension while a phase runs pauses it at the moment the system went
//!   to sleep: time asleep is not focus time, and no notification could have
//!   been shown during it.
//! - A session restored after the process ended comes back paused at its last
//!   checkpoint, for the same reason.

use std::time::Duration;

use argvus_time_core::elapsed::{Countdown, CountdownSnapshot, CountdownState};
use argvus_time_core::focus_status::{FocusStatus, State};
use argvus_time_core::pomodoro::{Mode, Phase, Pomodoro, PomodoroConfig};
use argvus_time_core::store::{self, StoreError};
use argvus_time_core::ticks::{Ticks, was_suspended};
use chrono::{DateTime, TimeDelta, Utc};
use serde::{Deserialize, Serialize};

use crate::history::SessionRecord;

const SNAPSHOT_VERSION: u32 = 1;
pub const SNAPSHOT_FILE_NAME: &str = "session.json";

/// Why a phase is paused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PauseReason {
  User,
  /// The system was suspended while the phase ran.
  Suspended,
  /// The application was closed while the phase ran.
  Restored,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
  /// The current phase has not started.
  Ready,
  Running,
  Paused(PauseReason),
  /// A phase ended; the next one waits for the user.
  AwaitingNext {
    finished: Phase,
  },
}

/// Something the application has to react to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
  PhaseFinished {
    finished: Phase,
    next: Phase,
    /// Present when a focus session ended.
    record: Option<SessionRecord>,
    /// The next phase started by itself (auto-start settings).
    started_next: bool,
  },
  PausedBySuspend {
    asleep: Duration,
  },
}

/// What the window shows.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct View {
  pub mode: Mode,
  pub phase: Phase,
  pub status: Status,
  pub remaining: Duration,
  pub total: Duration,
  pub progress: f64,
  /// Focus sessions completed in the cycle, and the cycle length.
  pub cycle: (u32, u32),
}

#[derive(Debug, Clone)]
pub struct Engine {
  config: PomodoroConfig,
  pomodoro: Pomodoro,
  countdown: Countdown,
  status: Status,
  /// Wall-clock start of the current phase, for the history.
  phase_started_at: Option<DateTime<Utc>>,
}

impl Engine {
  pub fn new(config: PomodoroConfig, mode: Mode) -> Self {
    let pomodoro = Pomodoro::new(mode);
    Self {
      countdown: Countdown::new(config.duration(mode, pomodoro.phase())),
      config,
      pomodoro,
      status: Status::Ready,
      phase_started_at: None,
    }
  }

  pub fn status(&self) -> Status {
    self.status
  }

  /// Whether a cycle is under way, so the application must keep running.
  pub fn is_active(&self) -> bool {
    self.status != Status::Ready
  }

  pub fn is_running(&self) -> bool {
    self.status == Status::Running
  }

  pub fn mode(&self) -> Mode {
    self.pomodoro.mode()
  }

  pub fn view(&self, now: Duration) -> View {
    View {
      mode: self.pomodoro.mode(),
      phase: self.pomodoro.phase(),
      status: self.status,
      remaining: self.countdown.remaining(now),
      total: self.countdown.total(),
      progress: self.countdown.progress(now),
      cycle: self.pomodoro.cycle_progress(&self.config),
    }
  }

  /// Time until the displayed seconds change, to schedule the next update.
  pub fn until_next_second(&self, now: Duration) -> Duration {
    let remaining = self.countdown.remaining(now);
    match remaining.subsec_nanos() {
      0 => Duration::from_secs(1),
      nanos => Duration::from_nanos(u64::from(nanos)),
    }
  }

  /// Starts the current phase, or resumes it.
  pub fn start(&mut self, now: Duration, wall: DateTime<Utc>) {
    match self.status {
      Status::Running => {}
      Status::Ready | Status::AwaitingNext { .. } => {
        self.countdown = self.fresh_countdown();
        self.countdown.start(now);
        self.phase_started_at = Some(wall);
        self.status = Status::Running;
      }
      Status::Paused(_) => {
        self.countdown.start(now);
        self.phase_started_at.get_or_insert(wall);
        self.status = Status::Running;
      }
    }
  }

  pub fn pause(&mut self, now: Duration) {
    if self.status == Status::Running {
      self.countdown.pause(now);
      self.status = Status::Paused(PauseReason::User);
    }
  }

  /// Abandons the current phase. Nothing is recorded; the cycle position is
  /// kept.
  pub fn cancel(&mut self) {
    self.countdown = self.fresh_countdown();
    self.status = Status::Ready;
    self.phase_started_at = None;
  }

  /// Ends the current phase now. A focus session is recorded with the time
  /// actually spent; a break simply ends.
  pub fn complete(&mut self, now: Duration, wall: DateTime<Utc>) -> Option<Event> {
    matches!(self.status, Status::Running | Status::Paused(_)).then(|| self.finish(now, wall, true))
  }

  /// Leaves a break and goes back to focus, ready to start.
  pub fn skip_break(&mut self) {
    if self.pomodoro.phase().is_break() {
      self.pomodoro.skip_break(&self.config);
      self.cancel();
    }
  }

  /// Advances the session. `previous` is the reading of the last tick.
  pub fn tick(&mut self, previous: Ticks, now: Ticks, wall: DateTime<Utc>) -> Option<Event> {
    if self.status != Status::Running {
      return None;
    }
    if was_suspended(previous, now) {
      self.countdown.pause_at(previous.boottime);
      self.status = Status::Paused(PauseReason::Suspended);
      return Some(Event::PausedBySuspend {
        asleep: now.suspended_since(previous),
      });
    }
    (self.countdown.state(now.boottime) == CountdownState::Finished)
      .then(|| self.finish(now.boottime, wall, false))
  }

  /// Switches between Pomodoro and single sessions. Refused while a phase
  /// runs or is paused, so a session is never lost by accident.
  pub fn set_mode(&mut self, mode: Mode) -> bool {
    if matches!(self.status, Status::Running | Status::Paused(_)) {
      return false;
    }
    if mode != self.pomodoro.mode() {
      self.pomodoro.reset(mode);
      self.cancel();
    }
    true
  }

  /// New lengths apply to phases that have not started yet.
  pub fn set_config(&mut self, config: PomodoroConfig) {
    self.config = config;
    if matches!(self.status, Status::Ready | Status::AwaitingNext { .. }) {
      self.countdown = self.fresh_countdown();
    }
  }

  fn fresh_countdown(&self) -> Countdown {
    Countdown::new(
      self
        .config
        .duration(self.pomodoro.mode(), self.pomodoro.phase()),
    )
  }

  fn finish(&mut self, now: Duration, wall: DateTime<Utc>, early: bool) -> Event {
    let finished = self.pomodoro.phase();
    let focused = self.countdown.elapsed(now);
    let record = (finished == Phase::Focus).then(|| SessionRecord {
      started_at: self.phase_started_at.unwrap_or_else(|| {
        wall - TimeDelta::seconds(i64::try_from(focused.as_secs()).unwrap_or(0))
      }),
      ended_at: wall,
      focused_seconds: focused.as_secs(),
      planned_seconds: self.countdown.total().as_secs(),
      completed_early: early,
    });

    let next = self.pomodoro.complete_phase(&self.config);
    self.countdown = self.fresh_countdown();
    let auto_start = self.pomodoro.mode() == Mode::Pomodoro
      && if next.is_break() {
        self.config.auto_start_breaks
      } else {
        self.config.auto_start_focus
      };
    if auto_start {
      self.countdown.start(now);
      self.phase_started_at = Some(wall);
      self.status = Status::Running;
    } else {
      self.phase_started_at = None;
      self.status = Status::AwaitingNext { finished };
    }
    Event::PhaseFinished {
      finished,
      next,
      record,
      started_next: auto_start,
    }
  }

  /// What the taskbar indicator shows: the state and, for a running phase,
  /// the absolute end time (so the reader needs no further updates).
  pub fn status_report(&self, now: Duration, wall: DateTime<Utc>, pid: u32) -> FocusStatus {
    let view = self.view(now);
    let remaining = view.remaining;
    let (state, ends_at, remaining_ms) = match self.status {
      Status::Ready => (State::Ready, None, None),
      Status::Running => (
        State::Running,
        TimeDelta::from_std(remaining).ok().map(|left| wall + left),
        None,
      ),
      Status::Paused(_) => (
        State::Paused,
        None,
        Some(u64::try_from(remaining.as_millis()).unwrap_or(u64::MAX)),
      ),
      Status::AwaitingNext { .. } => (State::Awaiting, None, None),
    };
    FocusStatus {
      pid,
      mode: view.mode,
      phase: view.phase,
      state,
      ends_at,
      remaining_ms,
    }
  }

  pub fn snapshot(&self, now: Duration) -> Snapshot {
    Snapshot {
      pomodoro: self.pomodoro,
      countdown: self.countdown.snapshot(now),
      status: match self.status {
        Status::Ready => StoredStatus::Ready,
        Status::Running | Status::Paused(_) => StoredStatus::Started,
        Status::AwaitingNext { finished } => StoredStatus::AwaitingNext { finished },
      },
      phase_started_at: self.phase_started_at,
    }
  }

  /// Rebuilds a session from a checkpoint. A phase that had started comes
  /// back paused ([`PauseReason::Restored`]) at the point of the checkpoint.
  pub fn restore(snapshot: &Snapshot, config: PomodoroConfig) -> Self {
    let status = match snapshot.status {
      StoredStatus::Ready => Status::Ready,
      StoredStatus::Started => Status::Paused(PauseReason::Restored),
      StoredStatus::AwaitingNext { finished } => Status::AwaitingNext { finished },
    };
    let mut engine = Self {
      config,
      pomodoro: snapshot.pomodoro,
      countdown: Countdown::from_snapshot(&snapshot.countdown),
      status,
      phase_started_at: snapshot.phase_started_at,
    };
    if status == Status::Ready {
      engine.countdown = engine.fresh_countdown();
    }
    engine
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum StoredStatus {
  Ready,
  /// Running or paused.
  Started,
  AwaitingNext {
    finished: Phase,
  },
}

/// Checkpoint of a session, written on every change and every
/// [`crate::app::CHECKPOINT_INTERVAL`] while a phase runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Snapshot {
  pub pomodoro: Pomodoro,
  pub countdown: CountdownSnapshot,
  pub status: StoredStatus,
  pub phase_started_at: Option<DateTime<Utc>>,
}

impl Snapshot {
  /// Reads the checkpoint; a missing or unreadable file gives `None`.
  pub fn load(path: &std::path::Path) -> Result<Option<Self>, StoreError> {
    let loaded = store::load_json::<Option<Self>>(path, SNAPSHOT_VERSION)?;
    Ok(loaded.value)
  }

  pub fn save(&self, path: &std::path::Path) -> Result<(), StoreError> {
    store::save_json(path, SNAPSHOT_VERSION, &Some(self))
  }
}

#[cfg(test)]
mod tests {
  use chrono::TimeZone;

  use super::*;

  const MINUTE: Duration = Duration::from_secs(60);

  fn wall(seconds: i64) -> DateTime<Utc> {
    Utc.timestamp_opt(1_800_000_000 + seconds, 0).unwrap()
  }

  fn ticks(seconds: u64) -> Ticks {
    Ticks {
      monotonic: Duration::from_secs(seconds),
      boottime: Duration::from_secs(seconds),
    }
  }

  fn config() -> PomodoroConfig {
    PomodoroConfig {
      focus_minutes: 25,
      short_break_minutes: 5,
      long_break_minutes: 15,
      sessions_before_long_break: 2,
      ..PomodoroConfig::default()
    }
  }

  fn run_to_end(engine: &mut Engine, started: u64) -> Event {
    let total = engine.view(Duration::from_secs(started)).total.as_secs();
    let end = started + total;
    engine
      .tick(
        ticks(end - 1),
        ticks(end),
        wall(i64::try_from(end).unwrap()),
      )
      .expect("phase finished")
  }

  #[test]
  fn focus_end_records_the_session_and_waits_for_the_break() {
    let mut engine = Engine::new(config(), Mode::Pomodoro);
    engine.start(Duration::ZERO, wall(0));
    assert_eq!(engine.tick(ticks(0), ticks(60), wall(60)), None);

    let event = run_to_end(&mut engine, 0);
    let Event::PhaseFinished {
      finished,
      next,
      record,
      started_next,
    } = event
    else {
      panic!("unexpected {event:?}");
    };
    assert_eq!(
      (finished, next, started_next),
      (Phase::Focus, Phase::ShortBreak, false)
    );
    let record = record.expect("focus is recorded");
    assert_eq!(record.focused_seconds, 25 * 60);
    assert_eq!(record.started_at, wall(0));
    assert!(!record.completed_early);
    assert_eq!(
      engine.status(),
      Status::AwaitingNext {
        finished: Phase::Focus
      }
    );
    assert_eq!(
      engine.view(Duration::from_secs(9_999)).remaining,
      5 * MINUTE
    );
  }

  #[test]
  fn breaks_are_not_recorded_and_cycle_reaches_the_long_break() {
    let mut engine = Engine::new(config(), Mode::Pomodoro);
    let mut now = 0;
    let mut phases = Vec::new();
    for _ in 0..4 {
      engine.start(Duration::from_secs(now), wall(0));
      let event = run_to_end(&mut engine, now);
      now += 100_000;
      if let Event::PhaseFinished {
        finished, record, ..
      } = event
      {
        assert_eq!(record.is_some(), finished == Phase::Focus);
        phases.push(finished);
      }
    }
    assert_eq!(
      phases,
      [
        Phase::Focus,
        Phase::ShortBreak,
        Phase::Focus,
        Phase::LongBreak
      ]
    );
  }

  #[test]
  fn auto_start_begins_the_break_at_once() {
    let mut engine = Engine::new(
      PomodoroConfig {
        auto_start_breaks: true,
        ..config()
      },
      Mode::Pomodoro,
    );
    engine.start(Duration::ZERO, wall(0));
    let Event::PhaseFinished { started_next, .. } = run_to_end(&mut engine, 0) else {
      panic!("phase should finish");
    };
    assert!(started_next);
    assert!(engine.is_running());
    assert_eq!(engine.view(Duration::ZERO).phase, Phase::ShortBreak);
  }

  #[test]
  fn pause_and_resume_keep_the_remaining_time() {
    let mut engine = Engine::new(config(), Mode::Pomodoro);
    engine.start(Duration::ZERO, wall(0));
    engine.pause(Duration::from_secs(600));
    assert_eq!(engine.status(), Status::Paused(PauseReason::User));
    engine.start(Duration::from_secs(5_000), wall(5_000));
    assert_eq!(
      engine.view(Duration::from_secs(5_060)).remaining,
      14 * MINUTE
    );
  }

  #[test]
  fn suspension_pauses_at_the_moment_of_sleep() {
    let mut engine = Engine::new(config(), Mode::Pomodoro);
    engine.start(Duration::ZERO, wall(0));
    let before_sleep = ticks(300);
    let after_sleep = Ticks {
      monotonic: Duration::from_secs(301),
      boottime: Duration::from_secs(3_901),
    };
    let event = engine.tick(before_sleep, after_sleep, wall(3_901));
    assert_eq!(
      event,
      Some(Event::PausedBySuspend {
        asleep: Duration::from_secs(3_600)
      })
    );
    assert_eq!(engine.status(), Status::Paused(PauseReason::Suspended));
    assert_eq!(engine.view(after_sleep.boottime).remaining, 20 * MINUTE);
  }

  #[test]
  fn complete_early_records_the_time_spent() {
    let mut engine = Engine::new(config(), Mode::Pomodoro);
    engine.start(Duration::ZERO, wall(0));
    let event = engine.complete(Duration::from_secs(600), wall(600));
    let Some(Event::PhaseFinished {
      record: Some(record),
      ..
    }) = event
    else {
      panic!("expected a record, got {event:?}");
    };
    assert_eq!(record.focused_seconds, 600);
    assert!(record.completed_early);
    assert_eq!(
      engine.complete(Duration::from_secs(601), wall(601)),
      None,
      "nothing runs"
    );
  }

  #[test]
  fn cancel_keeps_the_cycle_and_records_nothing() {
    let mut engine = Engine::new(config(), Mode::Pomodoro);
    engine.start(Duration::ZERO, wall(0));
    engine.cancel();
    assert_eq!(engine.status(), Status::Ready);
    assert!(!engine.is_active());
    assert_eq!(engine.view(Duration::from_secs(100)).remaining, 25 * MINUTE);
  }

  #[test]
  fn skip_break_returns_to_focus() {
    let mut engine = Engine::new(config(), Mode::Pomodoro);
    engine.start(Duration::ZERO, wall(0));
    run_to_end(&mut engine, 0);
    engine.skip_break();
    assert_eq!(engine.view(Duration::ZERO).phase, Phase::Focus);
    assert_eq!(engine.status(), Status::Ready);
  }

  #[test]
  fn mode_cannot_change_while_a_phase_runs() {
    let mut engine = Engine::new(config(), Mode::Pomodoro);
    engine.start(Duration::ZERO, wall(0));
    assert!(!engine.set_mode(Mode::Single));
    engine.cancel();
    assert!(engine.set_mode(Mode::Single));
    assert_eq!(engine.view(Duration::ZERO).total, 45 * MINUTE);
  }

  #[test]
  fn new_lengths_apply_only_to_phases_not_started() {
    let mut engine = Engine::new(config(), Mode::Pomodoro);
    engine.start(Duration::ZERO, wall(0));
    engine.set_config(PomodoroConfig {
      focus_minutes: 50,
      ..config()
    });
    assert_eq!(engine.view(Duration::ZERO).total, 25 * MINUTE);
    engine.cancel();
    assert_eq!(engine.view(Duration::ZERO).total, 50 * MINUTE);
  }

  #[test]
  fn running_session_is_restored_paused_at_the_checkpoint() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join(SNAPSHOT_FILE_NAME);
    let mut engine = Engine::new(config(), Mode::Pomodoro);
    engine.start(Duration::from_secs(100), wall(0));
    engine
      .snapshot(Duration::from_secs(400))
      .save(&path)
      .unwrap();

    let snapshot = Snapshot::load(&path).unwrap().expect("checkpoint");
    let restored = Engine::restore(&snapshot, config());
    assert_eq!(restored.status(), Status::Paused(PauseReason::Restored));
    assert_eq!(restored.view(Duration::ZERO).remaining, 20 * MINUTE);
    assert_eq!(restored.phase_started_at, Some(wall(0)));
  }

  #[test]
  fn missing_checkpoint_is_none() {
    let directory = tempfile::tempdir().unwrap();
    assert_eq!(
      Snapshot::load(&directory.path().join("none.json")).unwrap(),
      None
    );
  }

  #[test]
  fn status_report_gives_an_end_time_while_running_and_the_rest_when_paused() {
    let mut engine = Engine::new(config(), Mode::Pomodoro);
    assert_eq!(
      engine.status_report(Duration::ZERO, wall(0), 7).state,
      State::Ready
    );
    engine.start(Duration::from_secs(100), wall(0));
    let running = engine.status_report(Duration::from_secs(160), wall(60), 7);
    assert_eq!(running.state, State::Running);
    assert_eq!(running.ends_at, Some(wall(60 + 24 * 60)));
    engine.pause(Duration::from_secs(160));
    let paused = engine.status_report(Duration::from_secs(999), wall(900), 7);
    assert_eq!((paused.state, paused.ends_at), (State::Paused, None));
    assert_eq!(paused.remaining_ms, Some(24 * 60 * 1000));
  }

  #[test]
  fn until_next_second_follows_the_countdown() {
    let mut engine = Engine::new(config(), Mode::Pomodoro);
    engine.start(Duration::ZERO, wall(0));
    let delay = engine.until_next_second(Duration::from_millis(1_250));
    // 1_498.75 s remain: the display shows 24:59 until 1_498 s remain.
    assert_eq!(delay, Duration::from_millis(750));
  }
}
