//! Loop of the reminder service: finds due reminders and notifies them once.

use std::path::Path;
use std::time::Duration;

use crate::config::{Paths, Settings, service_events_enabled};
use crate::error::{ArgvusError, Result};
use crate::i18n::{I18n, Language};
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

  /// Polls until the executable is replaced by a package update. It then
  /// fails on purpose, so systemd (`Restart=on-failure`) starts the new
  /// version instead of keeping the old code running.
  pub fn run(self) -> Result<()> {
    loop {
      if std::fs::read_link("/proc/self/exe").is_ok_and(|exe| is_replaced_executable(&exe)) {
        return Err(ArgvusError::Service(
          "the executable was updated; exiting so the service restarts".to_string(),
        ));
      }
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
    let i18n = self.notification_language();
    let mut fired = 0;
    for (reminder, event) in due {
      if let Some(id) = reminder.id {
        if !is_stale_repeat(event.start, reminder.minutes_before, now) {
          notify_event(&event, reminder.minutes_before, i18n)?;
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

  /// The language of the notifications, read on each tick so a change made
  /// in the popup settings applies without restarting the service.
  fn notification_language(&self) -> I18n {
    let language = Settings::load(&self.paths)
      .map(|settings| settings.locale.language)
      .unwrap_or_default();
    I18n::new(Language::resolve(&language))
  }
}

/// Linux marks the target of `/proc/self/exe` as deleted once the running
/// file has been replaced on disk.
fn is_replaced_executable(exe: &Path) -> bool {
  exe.to_string_lossy().ends_with(" (deleted)")
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
  fn replaced_executable_is_detected() {
    assert!(is_replaced_executable(Path::new(
      "/usr/lib/argvus/commands/taskbar-calendar (deleted)"
    )));
    assert!(!is_replaced_executable(Path::new(
      "/usr/lib/argvus/commands/taskbar-calendar"
    )));
  }

  #[test]
  fn old_repetitions_are_skipped_without_skipping_primary_reminders() {
    let start = Utc.with_ymd_and_hms(2026, 8, 12, 0, 0, 0).unwrap();
    let noon = start + ChronoDuration::hours(12);
    assert!(is_stale_repeat(start, -120, noon));
    assert!(!is_stale_repeat(start, -720, noon));
    assert!(!is_stale_repeat(start, 10, noon));
  }
}
