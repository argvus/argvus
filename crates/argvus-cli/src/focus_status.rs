//! `argvus focus status --waybar [--follow]`: the taskbar indicator of ARGVUS
//! Focus. It runs inside the dispatcher process, with no GTK, because the bar
//! starts it once and keeps it running.
//!
//! Without `--follow` it prints one JSON line and exits. With `--follow` it
//! prints a line whenever the text changes: at every second while a phase
//! runs, and every two seconds otherwise to notice that ARGVUS Focus started
//! or stopped.

use std::io::{self, Write};
use std::process::ExitCode;
use std::thread::sleep;
use std::time::Duration;

use argvus_i18n::I18n;
use argvus_time_core::focus_status::{
  FocusStatus, Labels, State, WaybarOutput, render_waybar, status_path,
};
use chrono::Utc;

use crate::exec::{Failure, Outcome};

const IDLE_POLL: Duration = Duration::from_secs(2);
const USAGE: &str = "Usage: argvus focus status --waybar [--follow]";

pub fn run(arguments: &[String]) -> Outcome {
  let (mut waybar, mut follow) = (false, false);
  for argument in arguments {
    match argument.as_str() {
      "--waybar" => waybar = true,
      "--follow" => follow = true,
      other => {
        return Err(Failure::with_code(
          format!("unknown option {other}\n{USAGE}"),
          2,
        ));
      }
    }
  }
  if !waybar {
    return Err(Failure::with_code(USAGE.to_owned(), 2));
  }

  let labels = labels();
  let path = status_path();
  let mut last: Option<String> = None;
  let mut stdout = io::stdout().lock();
  loop {
    let status = path.as_deref().and_then(FocusStatus::load);
    let now = Utc::now();
    let output = render_waybar(status.as_ref(), now, &labels);
    let line = line(&output);
    if last.as_deref() != Some(line.as_str()) {
      // A closed pipe means the bar is gone: stop quietly.
      if writeln!(stdout, "{line}")
        .and_then(|()| stdout.flush())
        .is_err()
      {
        return Ok(ExitCode::SUCCESS);
      }
      last = Some(line);
    }
    if !follow {
      return Ok(ExitCode::SUCCESS);
    }
    sleep(wait(status.as_ref(), now));
  }
}

fn line(output: &WaybarOutput) -> String {
  serde_json::to_string(output).unwrap_or_default()
}

/// Time to the next change of the displayed text.
fn wait(status: Option<&FocusStatus>, now: chrono::DateTime<Utc>) -> Duration {
  let running_end = status
    .filter(|status| status.state == State::Running)
    .and_then(|status| status.ends_at);
  match running_end {
    Some(end) => {
      // The text shows the seconds left rounded up, so it changes when the
      // remaining time crosses a whole second.
      let left = (end - now).to_std().unwrap_or_default();
      let into_second = Duration::from_nanos(u64::from(left.subsec_nanos()));
      let wait = if into_second.is_zero() {
        Duration::from_secs(1)
      } else {
        into_second
      };
      wait + Duration::from_millis(20)
    }
    None => IDLE_POLL,
  }
}

fn labels() -> Labels {
  let texts = I18n::new("focus").ok();
  let tr = |key: &str| {
    texts
      .as_ref()
      .map_or_else(|| key.to_owned(), |texts| texts.tr(key))
  };
  Labels {
    idle: tr("taskbar.idle"),
    focus: tr("taskbar.focus"),
    short_break: tr("taskbar.short_break"),
    long_break: tr("taskbar.long_break"),
    paused: tr("taskbar.paused"),
    awaiting: tr("taskbar.awaiting"),
  }
}

#[cfg(test)]
mod tests {
  use chrono::TimeZone;

  use super::*;

  #[test]
  fn waits_for_the_next_displayed_second_while_running() {
    let now = Utc.timestamp_opt(1_800_000_000, 0).unwrap();
    let status = FocusStatus {
      pid: 1,
      mode: argvus_time_core::pomodoro::Mode::Pomodoro,
      phase: argvus_time_core::pomodoro::Phase::Focus,
      state: State::Running,
      ends_at: Some(now + chrono::TimeDelta::milliseconds(10_250)),
      remaining_ms: None,
    };
    assert_eq!(wait(Some(&status), now), Duration::from_millis(270));
    assert_eq!(wait(None, now), IDLE_POLL);
  }

  #[test]
  fn requires_waybar_and_rejects_unknown_options() {
    assert!(run(&[]).is_err());
    assert!(run(&["--waybar".to_owned(), "--bogus".to_owned()]).is_err());
  }
}
