use crate::config::{Paths, service_events_enabled};
use crate::error::Result;
use crate::reminders::ReminderScheduler;

pub fn run_service(paths: Paths) -> Result<()> {
  if !service_events_enabled(&paths) {
    return Ok(());
  }
  ReminderScheduler::new(&paths).run()
}
