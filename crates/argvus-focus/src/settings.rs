//! Preferences of ARGVUS Focus.

use std::path::Path;

use argvus_time_core::pomodoro::{Mode, PomodoroConfig};
use argvus_time_core::store::{self, StoreError};
use serde::{Deserialize, Serialize};

const VERSION: u32 = 1;
pub const FILE_NAME: &str = "settings.json";
pub const DEFAULT_VOLUME: u8 = 70;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
  pub pomodoro: PomodoroConfig,
  pub mode: Mode,
  /// Desktop notifications when a phase ends.
  pub notifications: bool,
  /// The bell at the end of a phase. Separate from the alarm sounds of
  /// ARGVUS Clock.
  pub sounds: bool,
  /// Bell volume, 0 to 100.
  pub volume: u8,
}

impl Default for Settings {
  fn default() -> Self {
    Self {
      pomodoro: PomodoroConfig::default(),
      mode: Mode::default(),
      notifications: true,
      sounds: true,
      volume: DEFAULT_VOLUME,
    }
  }
}

impl Settings {
  pub fn normalized(self) -> Self {
    Self {
      pomodoro: self.pomodoro.normalized(),
      volume: self.volume.min(100),
      ..self
    }
  }

  pub fn volume_fraction(&self) -> f64 {
    f64::from(self.volume.min(100)) / 100.0
  }

  pub fn load(path: &Path) -> Result<store::Loaded<Self>, StoreError> {
    let mut loaded = store::load_json::<Self>(path, VERSION)?;
    loaded.value = loaded.value.normalized();
    Ok(loaded)
  }

  pub fn save(&self, path: &Path) -> Result<(), StoreError> {
    store::save_json(path, VERSION, self)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn settings_round_trip_and_normalize() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join(FILE_NAME);
    let mut settings = Settings::default();
    settings.pomodoro.focus_minutes = 0;
    settings.volume = 250;
    settings.mode = Mode::Single;
    settings.save(&path).unwrap();

    let loaded = Settings::load(&path).unwrap().value;
    assert_eq!(loaded.mode, Mode::Single);
    assert_eq!(loaded.pomodoro.focus_minutes, 1);
    assert_eq!(loaded.volume, 100);
  }

  #[test]
  fn defaults_keep_sound_and_notifications_on() {
    let settings = Settings::default();
    assert!(settings.sounds);
    assert!(settings.notifications);
    assert!(settings.volume > 0);
  }
}
