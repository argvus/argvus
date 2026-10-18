use std::process::{Command, Stdio};

use chrono::Local;
use notify_rust::{Hint, Notification, Timeout};

use crate::calendar::CalendarEvent;
use crate::error::{ArgvusError, Result};

pub fn notify_event(event: &CalendarEvent, minutes_before: i64) -> Result<()> {
  let start = event.start.with_timezone(&Local);
  let body = if event.all_day && minutes_before < 0 {
    "All-day event reminder".to_string()
  } else if minutes_before < 0 {
    format!("Event in progress\nStarted at {}", start.format("%H:%M"))
  } else if minutes_before == 0 {
    format!("Starts now\n{}", start.format("%H:%M"))
  } else {
    format!(
      "Starts in {minutes_before} minutes\n{}",
      start.format("%H:%M")
    )
  };
  Notification::new()
    .appname("Argvus Calendar")
    .summary(&event.title)
    .body(&body)
    .icon("x-office-calendar")
    .hint(Hint::SoundName("bell".to_string()))
    .timeout(Timeout::Milliseconds(8000))
    .show()
    .map_err(|err| ArgvusError::Notification(err.to_string()))?;
  play_bell_sound();
  Ok(())
}

fn play_bell_sound() {
  if spawn_silent("canberra-gtk-play", &["-i", "bell"]).is_ok()
    || spawn_silent("canberra-gtk-play", &["-i", "message"]).is_ok()
  {
    return;
  }

  for file in [
    "/usr/share/sounds/freedesktop/stereo/bell.oga",
    "/usr/share/sounds/freedesktop/stereo/complete.oga",
  ] {
    if spawn_silent("paplay", &[file]).is_ok() || spawn_silent("pw-play", &[file]).is_ok() {
      return;
    }
  }
}

fn spawn_silent(command: &str, args: &[&str]) -> std::io::Result<()> {
  Command::new(command)
    .args(args)
    .stdin(Stdio::null())
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .spawn()
    .map(|_| ())
}
