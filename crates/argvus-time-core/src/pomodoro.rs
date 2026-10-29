//! Pomodoro phases, cycles and presets.
//!
//! A cycle is a number of focus sessions separated by short breaks and closed
//! by a long break. The [`Pomodoro`] only decides which phase comes next; the
//! application times each phase with a [`crate::elapsed::Countdown`].

use std::time::Duration;

use serde::{Deserialize, Serialize};

/// Shortest and longest phase, in minutes.
pub const MIN_PHASE_MINUTES: u32 = 1;
pub const MAX_PHASE_MINUTES: u32 = 240;
/// Most focus sessions before a long break.
pub const MAX_SESSIONS_BEFORE_LONG_BREAK: u32 = 12;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
  Focus,
  ShortBreak,
  LongBreak,
}

impl Phase {
  pub fn is_break(self) -> bool {
    !matches!(self, Self::Focus)
  }
}

/// Whether sessions follow the Pomodoro cycle or stand alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
  /// Focus, breaks and a long break after a number of sessions.
  #[default]
  Pomodoro,
  /// One focus session of its own length, with no breaks.
  Single,
}

/// Phase lengths and cycle settings, as the user configured them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PomodoroConfig {
  pub focus_minutes: u32,
  pub short_break_minutes: u32,
  pub long_break_minutes: u32,
  pub sessions_before_long_break: u32,
  /// Length of a session in [`Mode::Single`].
  pub single_session_minutes: u32,
  /// Start a break as soon as a focus session ends.
  pub auto_start_breaks: bool,
  /// Start the next focus session as soon as a break ends.
  pub auto_start_focus: bool,
}

impl Default for PomodoroConfig {
  fn default() -> Self {
    let classic = PRESETS[0];
    Self {
      focus_minutes: classic.focus_minutes,
      short_break_minutes: classic.short_break_minutes,
      long_break_minutes: classic.long_break_minutes,
      sessions_before_long_break: classic.sessions_before_long_break,
      single_session_minutes: 45,
      auto_start_breaks: false,
      auto_start_focus: false,
    }
  }
}

impl PomodoroConfig {
  /// Brings every value into its allowed range, so a hand-edited file cannot
  /// produce a zero-length phase.
  pub fn normalized(self) -> Self {
    let phase = |minutes: u32| minutes.clamp(MIN_PHASE_MINUTES, MAX_PHASE_MINUTES);
    Self {
      focus_minutes: phase(self.focus_minutes),
      short_break_minutes: phase(self.short_break_minutes),
      long_break_minutes: phase(self.long_break_minutes),
      sessions_before_long_break: self
        .sessions_before_long_break
        .clamp(1, MAX_SESSIONS_BEFORE_LONG_BREAK),
      single_session_minutes: phase(self.single_session_minutes),
      ..self
    }
  }

  /// Length of a phase in the given mode.
  pub fn duration(&self, mode: Mode, phase: Phase) -> Duration {
    let minutes = match (mode, phase) {
      (Mode::Single, _) => self.single_session_minutes,
      (Mode::Pomodoro, Phase::Focus) => self.focus_minutes,
      (Mode::Pomodoro, Phase::ShortBreak) => self.short_break_minutes,
      (Mode::Pomodoro, Phase::LongBreak) => self.long_break_minutes,
    };
    Duration::from_secs(u64::from(minutes) * 60)
  }

  /// Copies the lengths of a preset, keeping the other settings.
  pub fn with_preset(self, preset: Preset) -> Self {
    Self {
      focus_minutes: preset.focus_minutes,
      short_break_minutes: preset.short_break_minutes,
      long_break_minutes: preset.long_break_minutes,
      sessions_before_long_break: preset.sessions_before_long_break,
      ..self
    }
  }

  /// The preset whose lengths match, if any.
  pub fn matching_preset(&self) -> Option<Preset> {
    PRESETS.into_iter().find(|preset| {
      preset.focus_minutes == self.focus_minutes
        && preset.short_break_minutes == self.short_break_minutes
        && preset.long_break_minutes == self.long_break_minutes
        && preset.sessions_before_long_break == self.sessions_before_long_break
    })
  }
}

/// A named set of phase lengths.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Preset {
  /// Stable ID, also the i18n key suffix (`preset.<id>`).
  pub id: &'static str,
  pub focus_minutes: u32,
  pub short_break_minutes: u32,
  pub long_break_minutes: u32,
  pub sessions_before_long_break: u32,
}

pub const PRESETS: [Preset; 3] = [
  Preset {
    id: "classic",
    focus_minutes: 25,
    short_break_minutes: 5,
    long_break_minutes: 15,
    sessions_before_long_break: 4,
  },
  Preset {
    id: "extended",
    focus_minutes: 50,
    short_break_minutes: 10,
    long_break_minutes: 30,
    sessions_before_long_break: 3,
  },
  Preset {
    id: "deep",
    focus_minutes: 90,
    short_break_minutes: 20,
    long_break_minutes: 30,
    sessions_before_long_break: 2,
  },
];

/// Position in the cycle and the current phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pomodoro {
  mode: Mode,
  phase: Phase,
  /// Focus sessions completed since the last long break.
  completed_in_cycle: u32,
}

impl Pomodoro {
  pub fn new(mode: Mode) -> Self {
    Self {
      mode,
      phase: Phase::Focus,
      completed_in_cycle: 0,
    }
  }

  pub fn mode(&self) -> Mode {
    self.mode
  }

  pub fn phase(&self) -> Phase {
    self.phase
  }

  /// Focus sessions completed in the current cycle, and the cycle length.
  pub fn cycle_progress(&self, config: &PomodoroConfig) -> (u32, u32) {
    let length = config.sessions_before_long_break.max(1);
    (self.completed_in_cycle.min(length), length)
  }

  /// Ends the current phase and moves to the next one, which is returned.
  pub fn complete_phase(&mut self, config: &PomodoroConfig) -> Phase {
    self.phase = match (self.mode, self.phase) {
      (Mode::Single, _) => Phase::Focus,
      (Mode::Pomodoro, Phase::Focus) => {
        self.completed_in_cycle += 1;
        if self.completed_in_cycle >= config.sessions_before_long_break.max(1) {
          Phase::LongBreak
        } else {
          Phase::ShortBreak
        }
      }
      (Mode::Pomodoro, Phase::LongBreak) => {
        self.completed_in_cycle = 0;
        Phase::Focus
      }
      (Mode::Pomodoro, Phase::ShortBreak) => Phase::Focus,
    };
    self.phase
  }

  /// Leaves a break early and goes back to focus. Does nothing during focus.
  pub fn skip_break(&mut self, config: &PomodoroConfig) -> Phase {
    if self.phase.is_break() {
      self.complete_phase(config);
    }
    self.phase
  }

  /// Starts a new cycle in the given mode.
  pub fn reset(&mut self, mode: Mode) {
    *self = Self::new(mode);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn cycle_alternates_focus_and_short_breaks_then_long_break() {
    let config = PomodoroConfig::default();
    let mut pomodoro = Pomodoro::new(Mode::Pomodoro);
    let mut phases = vec![pomodoro.phase()];
    for _ in 0..8 {
      phases.push(pomodoro.complete_phase(&config));
    }
    assert_eq!(
      phases,
      [
        Phase::Focus,
        Phase::ShortBreak,
        Phase::Focus,
        Phase::ShortBreak,
        Phase::Focus,
        Phase::ShortBreak,
        Phase::Focus,
        Phase::LongBreak,
        Phase::Focus,
      ]
    );
    assert_eq!(
      pomodoro.cycle_progress(&config),
      (0, 4),
      "long break restarts the cycle"
    );
  }

  #[test]
  fn one_session_cycle_always_takes_the_long_break() {
    let config = PomodoroConfig {
      sessions_before_long_break: 1,
      ..PomodoroConfig::default()
    };
    let mut pomodoro = Pomodoro::new(Mode::Pomodoro);
    assert_eq!(pomodoro.complete_phase(&config), Phase::LongBreak);
    assert_eq!(pomodoro.complete_phase(&config), Phase::Focus);
    assert_eq!(pomodoro.complete_phase(&config), Phase::LongBreak);
  }

  #[test]
  fn single_mode_has_no_breaks_and_its_own_length() {
    let config = PomodoroConfig::default();
    let mut pomodoro = Pomodoro::new(Mode::Single);
    assert_eq!(pomodoro.complete_phase(&config), Phase::Focus);
    assert_eq!(
      config.duration(Mode::Single, Phase::Focus),
      Duration::from_secs(45 * 60)
    );
  }

  #[test]
  fn skip_break_returns_to_focus_and_ignores_focus() {
    let config = PomodoroConfig::default();
    let mut pomodoro = Pomodoro::new(Mode::Pomodoro);
    assert_eq!(pomodoro.skip_break(&config), Phase::Focus);
    assert_eq!(pomodoro.cycle_progress(&config), (0, 4));
    pomodoro.complete_phase(&config);
    assert_eq!(pomodoro.skip_break(&config), Phase::Focus);
    assert_eq!(pomodoro.cycle_progress(&config), (1, 4));
  }

  #[test]
  fn normalized_config_rejects_zero_and_huge_values() {
    let config = PomodoroConfig {
      focus_minutes: 0,
      short_break_minutes: 9_999,
      sessions_before_long_break: 0,
      ..PomodoroConfig::default()
    }
    .normalized();
    assert_eq!(config.focus_minutes, MIN_PHASE_MINUTES);
    assert_eq!(config.short_break_minutes, MAX_PHASE_MINUTES);
    assert_eq!(config.sessions_before_long_break, 1);
  }

  #[test]
  fn presets_apply_and_are_recognized() {
    let config = PomodoroConfig::default().with_preset(PRESETS[1]);
    assert_eq!(
      config.matching_preset().map(|preset| preset.id),
      Some("extended")
    );
    let custom = PomodoroConfig {
      focus_minutes: 33,
      ..config
    };
    assert_eq!(custom.matching_preset(), None);
  }

  #[test]
  fn config_file_with_missing_fields_uses_defaults() {
    let config: PomodoroConfig = serde_json::from_str(r#"{"focus_minutes": 40}"#).expect("parse");
    assert_eq!(config.focus_minutes, 40);
    assert_eq!(config.short_break_minutes, 5);
  }
}
