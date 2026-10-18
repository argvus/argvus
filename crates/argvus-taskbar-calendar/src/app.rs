use std::path::PathBuf;

use crate::config::{Paths, Settings, service_events_enabled};
use crate::error::Result;
use crate::ical::{export_ics_file, import_ics_file};
use crate::storage::Database;

pub fn import_file(paths: &Paths, file: PathBuf) -> Result<()> {
  let mut db = Database::open(&paths.database)?;
  let calendar_id = db.default_calendar_id()?;
  let mut events = import_ics_file(&file, calendar_id)?;
  let count = events.len();
  for event in &mut events {
    db.upsert_event(event)?;
  }
  println!("Imported {count} events into the Local calendar");
  Ok(())
}

pub fn export_file(paths: &Paths, output: PathBuf) -> Result<()> {
  let db = Database::open(&paths.database)?;
  let events = db.all_events()?;
  let count = events.len();
  export_ics_file(&output, &events)?;
  println!("Exported {count} events to {}", output.display());
  Ok(())
}

pub fn sync_once(paths: &Paths, settings: &Settings) -> Result<()> {
  let db = Database::open(&paths.database)?;
  let calendars = db.calendars()?;
  let remote = calendars
    .iter()
    .filter(|calendar| calendar.enabled && calendar.remote_url.is_some())
    .count();
  println!(
    "Sync scheduler configured for every {} minutes; {remote} remote calendars configured",
    settings.calendar.sync_interval_minutes
  );
  Ok(())
}

pub fn status(paths: &Paths) -> Result<()> {
  let db = Database::open(&paths.database)?;
  println!("Database: OK ({})", paths.database.display());
  println!("Config: {}", paths.config_file.display());
  println!("Style: {}", paths.user_style.display());
  println!("Theme: {}", paths.theme_file.display());
  println!("Theme cache: {}", paths.cache_theme_file.display());
  println!(
    "Active ARGVUS theme: {}",
    paths.active_argvus_theme_file.display()
  );
  println!(
    "Events/reminders: {} ({})",
    if service_events_enabled(paths) {
      "enabled"
    } else {
      "disabled"
    },
    paths.events_enabled_file.display()
  );
  println!("Data: {}", paths.data_dir.display());
  println!("State: {}", paths.state_dir.display());
  println!("Cache: {}", paths.cache_dir.display());
  println!("Calendars: {}", db.calendars()?.len());
  println!("Events: {}", db.all_events()?.len());
  println!("Service: use `argvus-taskbar-calendar service` or install the user systemd unit");
  Ok(())
}
