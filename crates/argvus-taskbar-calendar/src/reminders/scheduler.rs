use std::time::Duration;

use crate::config::{Paths, service_events_enabled};
use crate::error::Result;
use crate::storage::Database;
use chrono::{Duration as ChronoDuration, Utc};
use log::{debug, info, warn};

use super::notify_event;

pub struct ReminderScheduler {
  paths: Paths,
  database_path: std::path::PathBuf,
  poll_interval: Duration,
}

impl ReminderScheduler {
  pub fn new(paths: &Paths) -> Self {
    Self {
      paths: paths.clone(),
      database_path: paths.database.clone(),
      poll_interval: Duration::from_secs(30),
    }
  }

  pub fn run(self) -> Result<()> {
    loop {
      if let Err(err) = self.tick() {
        eprintln!("argvus-taskbar-calendar: reminder scheduler tick failed: {err}");
        warn!("reminder scheduler tick failed: {err}");
      }
      std::thread::sleep(self.poll_interval);
    }
  }

  pub fn tick(&self) -> Result<usize> {
    if !service_events_enabled(&self.paths) {
      return Ok(0);
    }
    let db = Database::open(&self.database_path)?;
    let now = Utc::now();
    db.purge_expired_local_single_events(now)?;
    let added_start_reminders = db.ensure_start_reminders_for_unscheduled_due_events(now)?;
    if added_start_reminders > 0 {
      debug!("created {added_start_reminders} start reminders for unscheduled due events");
    }
    let due = db.due_reminders(now)?;
    let mut fired = 0;
    for (reminder, event) in due {
      if let Some(id) = reminder.id {
        if !is_stale_repeat(event.start, reminder.minutes_before, now) {
          notify_event(&event, reminder.minutes_before)?;
          fired += 1;
        }
        db.mark_reminder_fired(id, now)?;
      }
    }
    if fired > 0 {
      eprintln!("argvus-taskbar-calendar: fired {fired} reminders");
      info!("fired {fired} reminders");
    } else {
      debug!("fired {fired} reminders");
    }
    Ok(fired)
  }
}

fn is_stale_repeat(
  start: chrono::DateTime<Utc>,
  minutes_before: i64,
  now: chrono::DateTime<Utc>,
) -> bool {
  let trigger = start - ChronoDuration::minutes(minutes_before);
  minutes_before < 0 && now.signed_duration_since(trigger) > ChronoDuration::minutes(2)
}

#[cfg(test)]
mod tests {
  use chrono::TimeZone;

  use super::*;

  #[test]
  fn old_repetitions_are_skipped_without_skipping_primary_reminders() {
    let start = Utc.with_ymd_and_hms(2026, 8, 12, 0, 0, 0).unwrap();
    let noon = start + ChronoDuration::hours(12);
    assert!(is_stale_repeat(start, -120, noon));
    assert!(!is_stale_repeat(start, -720, noon));
    assert!(!is_stale_repeat(start, 10, noon));
  }
}
