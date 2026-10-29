//! The status file of ARGVUS Focus, read by the taskbar indicator.
//!
//! ARGVUS Focus writes `$XDG_RUNTIME_DIR/argvus/focus/status.json` on every
//! change of state and removes it on exit. The file holds an absolute end time
//! while a phase runs, so a reader needs no polling of the application and
//! can compute the time left itself. A file whose process no longer exists is
//! stale (a crash) and is ignored.

use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::duration::format_remaining;
use crate::pomodoro::{Mode, Phase};
use crate::store::{self, StoreError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum State {
  /// The current phase has not started.
  Ready,
  Running,
  Paused,
  /// A phase ended; the next one waits for the user.
  Awaiting,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FocusStatus {
  pub pid: u32,
  pub mode: Mode,
  pub phase: Phase,
  pub state: State,
  /// When the running phase ends.
  #[serde(default)]
  pub ends_at: Option<DateTime<Utc>>,
  /// Time left of a paused phase.
  #[serde(default)]
  pub remaining_ms: Option<u64>,
}

/// `$XDG_RUNTIME_DIR/argvus/focus/status.json`, when the runtime directory
/// is known.
pub fn status_path() -> Option<PathBuf> {
  let runtime = std::env::var_os("XDG_RUNTIME_DIR").map(PathBuf::from)?;
  runtime
    .is_absolute()
    .then(|| runtime.join("argvus/focus/status.json"))
}

impl FocusStatus {
  pub fn save(&self, path: &Path) -> Result<(), StoreError> {
    let mut bytes = serde_json::to_vec(self).map_err(|source| StoreError::Encode {
      path: path.to_owned(),
      source,
    })?;
    bytes.push(b'\n');
    store::atomic_write(path, &bytes)
  }

  /// The status of a running ARGVUS Focus, or `None` when there is no file,
  /// it cannot be read, or its process is gone.
  pub fn load(path: &Path) -> Option<Self> {
    let status: Self = serde_json::from_slice(&std::fs::read(path).ok()?).ok()?;
    Path::new(&format!("/proc/{}", status.pid))
      .exists()
      .then_some(status)
  }

  /// Removes the file; a missing file is not an error.
  pub fn remove(path: &Path) {
    let _ = std::fs::remove_file(path);
  }
}

/// Nerd Font glyph of the indicator (md-timer-outline), like the other
/// taskbar modules.
pub const ICON: &str = "\u{f051b}";

/// Translated texts of the indicator's tooltip.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Labels {
  /// Tooltip when nothing runs.
  pub idle: String,
  pub focus: String,
  pub short_break: String,
  pub long_break: String,
  pub paused: String,
  /// Tooltip when a phase ended and the next one waits for the user.
  pub awaiting: String,
}

/// One line for the Waybar `custom` module with `return-type: json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WaybarOutput {
  pub text: String,
  pub tooltip: String,
  /// CSS class: `focus`, `short-break` or `long-break` while running or
  /// paused (plus `paused`), `idle` otherwise.
  pub class: Vec<String>,
}

/// Renders the indicator. `status` is `None` when ARGVUS Focus is not
/// running (or the file is stale).
pub fn render_waybar(
  status: Option<&FocusStatus>,
  now: DateTime<Utc>,
  labels: &Labels,
) -> WaybarOutput {
  let idle = |tooltip: &str| WaybarOutput {
    text: ICON.to_owned(),
    tooltip: tooltip.to_owned(),
    class: vec!["idle".to_owned()],
  };
  let Some(status) = status else {
    return idle(&labels.idle);
  };
  let (phase_label, phase_class) = match status.phase {
    Phase::Focus => (&labels.focus, "focus"),
    Phase::ShortBreak => (&labels.short_break, "short-break"),
    Phase::LongBreak => (&labels.long_break, "long-break"),
  };
  match status.state {
    State::Ready => idle(&labels.idle),
    State::Awaiting => idle(&labels.awaiting),
    State::Running => {
      let left = status
        .ends_at
        .and_then(|end| (end - now).to_std().ok())
        .unwrap_or_default();
      let time = format_remaining(left);
      WaybarOutput {
        text: format!("{ICON} {time}"),
        tooltip: format!("{phase_label} · {time}"),
        class: vec![phase_class.to_owned()],
      }
    }
    State::Paused => {
      let left = std::time::Duration::from_millis(status.remaining_ms.unwrap_or_default());
      let time = format_remaining(left);
      WaybarOutput {
        text: format!("{ICON} {time}"),
        tooltip: format!("{phase_label} ({}) · {time}", labels.paused),
        class: vec![phase_class.to_owned(), "paused".to_owned()],
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use chrono::TimeZone;

  use super::*;

  fn running_status(pid: u32) -> FocusStatus {
    FocusStatus {
      pid,
      mode: Mode::Pomodoro,
      phase: Phase::Focus,
      state: State::Running,
      ends_at: Some(Utc.with_ymd_and_hms(2026, 10, 28, 12, 0, 0).unwrap()),
      remaining_ms: None,
    }
  }

  #[test]
  fn status_of_a_live_process_round_trips() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("run/status.json");
    let status = running_status(std::process::id());
    status.save(&path).unwrap();
    assert_eq!(FocusStatus::load(&path), Some(status));
    FocusStatus::remove(&path);
    assert_eq!(FocusStatus::load(&path), None);
  }

  #[test]
  fn status_of_a_dead_process_is_stale() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("status.json");
    // PIDs above the kernel limit never exist.
    running_status(u32::MAX).save(&path).unwrap();
    assert_eq!(FocusStatus::load(&path), None);
  }

  #[test]
  fn unreadable_status_is_ignored() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("status.json");
    std::fs::write(&path, "{ nope").unwrap();
    assert_eq!(FocusStatus::load(&path), None);
  }

  fn labels() -> Labels {
    Labels {
      idle: "ARGVUS Focus".to_owned(),
      focus: "Focus".to_owned(),
      short_break: "Short break".to_owned(),
      long_break: "Long break".to_owned(),
      paused: "paused".to_owned(),
      awaiting: "Next phase waits".to_owned(),
    }
  }

  fn at(seconds: i64) -> DateTime<Utc> {
    Utc.timestamp_opt(1_800_000_000 + seconds, 0).unwrap()
  }

  #[test]
  fn running_phase_shows_the_time_left_and_its_phase_class() {
    let status = FocusStatus {
      ends_at: Some(at(1_121)),
      ..running_status(1)
    };
    let output = render_waybar(Some(&status), at(0), &labels());
    assert_eq!(output.text, format!("{ICON} 18:41"));
    assert_eq!(output.tooltip, "Focus · 18:41");
    assert_eq!(output.class, ["focus"]);

    let break_status = FocusStatus {
      phase: Phase::LongBreak,
      ..status
    };
    assert_eq!(
      render_waybar(Some(&break_status), at(0), &labels()).class,
      ["long-break"]
    );
  }

  #[test]
  fn overdue_end_time_shows_zero() {
    let status = FocusStatus {
      ends_at: Some(at(-5)),
      ..running_status(1)
    };
    assert_eq!(
      render_waybar(Some(&status), at(0), &labels()).text,
      format!("{ICON} 00:00")
    );
  }

  #[test]
  fn paused_phase_keeps_the_remaining_time_and_adds_the_paused_class() {
    let status = FocusStatus {
      state: State::Paused,
      ends_at: None,
      remaining_ms: Some(90_500),
      ..running_status(1)
    };
    let output = render_waybar(Some(&status), at(0), &labels());
    assert_eq!(output.text, format!("{ICON} 01:31"));
    assert_eq!(output.class, ["focus", "paused"]);
    assert!(output.tooltip.contains("paused"));
  }

  #[test]
  fn idle_states_show_only_the_icon() {
    for status in [
      None,
      Some(FocusStatus {
        state: State::Ready,
        ends_at: None,
        ..running_status(1)
      }),
      Some(FocusStatus {
        state: State::Awaiting,
        ends_at: None,
        ..running_status(1)
      }),
    ] {
      let output = render_waybar(status.as_ref(), at(0), &labels());
      assert_eq!(output.text, ICON);
      assert_eq!(output.class, ["idle"]);
    }
    assert_eq!(
      render_waybar(
        Some(&FocusStatus {
          state: State::Awaiting,
          ends_at: None,
          ..running_status(1)
        }),
        at(0),
        &labels()
      )
      .tooltip,
      "Next phase waits"
    );
  }
}
