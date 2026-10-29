//! Completed focus sessions, kept locally.

use std::path::Path;
use std::time::Duration;

use argvus_time_core::store::{self, StoreError};
use chrono::{DateTime, TimeDelta, TimeZone, Utc};
use serde::{Deserialize, Serialize};

const VERSION: u32 = 1;
pub const FILE_NAME: &str = "history.json";
/// Sessions older than this are dropped when a new one is recorded.
pub const RETENTION_DAYS: i64 = 365;

/// One completed focus session. Breaks and cancelled sessions are not
/// recorded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionRecord {
  pub started_at: DateTime<Utc>,
  pub ended_at: DateTime<Utc>,
  /// Time actually spent focusing, without pauses.
  pub focused_seconds: u64,
  pub planned_seconds: u64,
  /// Completed with the "Complete" button before the timer ran out.
  #[serde(default)]
  pub completed_early: bool,
}

impl SessionRecord {
  pub fn focused(&self) -> Duration {
    Duration::from_secs(self.focused_seconds)
  }
}

/// Totals of one day.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DaySummary {
  pub sessions: u32,
  pub focused: Duration,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct History {
  #[serde(default)]
  sessions: Vec<SessionRecord>,
}

impl History {
  pub fn load(path: &Path) -> Result<store::Loaded<Self>, StoreError> {
    store::load_json(path, VERSION)
  }

  pub fn save(&self, path: &Path) -> Result<(), StoreError> {
    store::save_json(path, VERSION, self)
  }

  /// Adds a session and drops the ones past the retention period.
  pub fn record(&mut self, session: SessionRecord, now: DateTime<Utc>) {
    let oldest = now - TimeDelta::days(RETENTION_DAYS);
    self.sessions.retain(|kept| kept.ended_at >= oldest);
    self.sessions.push(session);
    self.sessions.sort_by_key(|kept| kept.ended_at);
  }

  pub fn clear(&mut self) {
    self.sessions.clear();
  }

  pub fn is_empty(&self) -> bool {
    self.sessions.is_empty()
  }

  pub fn total_sessions(&self) -> usize {
    self.sessions.len()
  }

  pub fn total_focused(&self) -> Duration {
    self.sessions.iter().map(SessionRecord::focused).sum()
  }

  /// Sessions that ended on the local day of `now`.
  pub fn day_summary<Tz: TimeZone>(&self, now: DateTime<Utc>, zone: &Tz) -> DaySummary {
    let today = now.with_timezone(zone).date_naive();
    self
      .sessions
      .iter()
      .filter(|session| session.ended_at.with_timezone(zone).date_naive() == today)
      .fold(DaySummary::default(), |summary, session| DaySummary {
        sessions: summary.sessions + 1,
        focused: summary.focused + session.focused(),
      })
  }

  /// The most recent sessions first.
  pub fn recent(&self, limit: usize) -> impl Iterator<Item = &SessionRecord> {
    self.sessions.iter().rev().take(limit)
  }
}

#[cfg(test)]
mod tests {
  use chrono::FixedOffset;

  use super::*;

  /// A fixed UTC-3 zone, so the tests do not depend on the machine.
  fn utc_minus_three() -> FixedOffset {
    FixedOffset::west_opt(3 * 3_600).unwrap()
  }

  fn session(ended_at: DateTime<Utc>, minutes: u64) -> SessionRecord {
    SessionRecord {
      started_at: ended_at - TimeDelta::minutes(i64::try_from(minutes).unwrap()),
      ended_at,
      focused_seconds: minutes * 60,
      planned_seconds: minutes * 60,
      completed_early: false,
    }
  }

  fn utc(day: u32, hour: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 10, day, hour, 0, 0).unwrap()
  }

  #[test]
  fn day_summary_uses_the_local_day() {
    let mut history = History::default();
    // 01:00 UTC on the 28th is still the 27th in UTC-3.
    history.record(session(utc(28, 1), 25), utc(28, 1));
    history.record(session(utc(28, 13), 25), utc(28, 13));
    history.record(session(utc(28, 20), 50), utc(28, 20));
    let summary = history.day_summary(utc(28, 22), &utc_minus_three());
    assert_eq!(summary.sessions, 2);
    assert_eq!(summary.focused, Duration::from_secs(75 * 60));
    assert_eq!(history.total_sessions(), 3);
  }

  #[test]
  fn old_sessions_are_dropped_after_the_retention_period() {
    let mut history = History::default();
    let long_ago = utc(1, 12) - TimeDelta::days(RETENTION_DAYS + 30);
    history.record(session(long_ago, 25), long_ago);
    history.record(session(utc(28, 12), 25), utc(28, 12));
    assert_eq!(history.total_sessions(), 1);
  }

  #[test]
  fn recent_lists_newest_first_and_history_round_trips() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join(FILE_NAME);
    let mut history = History::default();
    history.record(session(utc(27, 12), 25), utc(27, 12));
    history.record(session(utc(28, 12), 50), utc(28, 12));
    history.save(&path).unwrap();
    let loaded = History::load(&path).unwrap().value;
    let newest: Vec<_> = loaded
      .recent(1)
      .map(|session| session.focused_seconds)
      .collect();
    assert_eq!(newest, [50 * 60]);
    assert_eq!(loaded.total_focused(), Duration::from_secs(75 * 60));
  }
}
