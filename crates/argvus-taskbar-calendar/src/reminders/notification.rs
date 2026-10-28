//! Desktop notification and bell sound of a due reminder.

use std::process::{Command, Stdio};

use chrono::Local;
use notify_rust::{Hint, Notification, Timeout};

use crate::calendar::CalendarEvent;
use crate::error::{ArgvusError, Result};
use crate::i18n::{I18n, Text};

pub fn notify_event(event: &CalendarEvent, minutes_before: i64, i18n: I18n) -> Result<()> {
  let start = event
    .start
    .with_timezone(&Local)
    .format("%H:%M")
    .to_string();
  let body = reminder_body(i18n, event.all_day, minutes_before, &start);
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

/// Body of the reminder notification. A negative `minutes_before` is a
/// repetition during the event; on an all-day event the start is the
/// notification time chosen in the form, not the start of a meeting.
fn reminder_body(i18n: I18n, all_day: bool, minutes_before: i64, start: &str) -> String {
  if all_day && minutes_before <= 0 {
    i18n.text(Text::ReminderAllDay).to_string()
  } else if minutes_before < 0 {
    format!(
      "{}\n{} {start}",
      i18n.text(Text::ReminderInProgress),
      i18n.text(Text::ReminderStartedAt)
    )
  } else if minutes_before == 0 {
    format!("{}\n{start}", i18n.text(Text::ReminderStartsNow))
  } else {
    format!("{}\n{start}", i18n.reminder_starts_in(minutes_before))
  }
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

#[cfg(test)]
mod tests {
  use crate::i18n::Language;

  use super::*;

  #[test]
  fn reminder_body_follows_the_language() {
    let english = I18n::new(Language::EnUs);
    let portuguese = I18n::new(Language::PtBr);
    assert_eq!(
      reminder_body(english, false, 10, "09:00"),
      "Starts in 10 minutes\n09:00"
    );
    assert_eq!(
      reminder_body(english, false, 1, "09:00"),
      "Starts in 1 minute\n09:00"
    );
    assert_eq!(
      reminder_body(portuguese, false, 0, "09:00"),
      "Começa agora\n09:00"
    );
    assert_eq!(
      reminder_body(portuguese, false, -60, "09:00"),
      "Evento em andamento\nComeçou às 09:00"
    );
    assert_eq!(
      reminder_body(portuguese, true, -60, "00:00"),
      "Lembrete de evento de dia inteiro"
    );
    assert_eq!(
      reminder_body(english, true, 0, "14:00"),
      "All-day event reminder"
    );
  }
}
