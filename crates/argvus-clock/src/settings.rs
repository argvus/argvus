//! Preferences of ARGVUS Clock, including the saved world clocks.

use std::path::Path;

use argvus_time_core::alarm::SoundChoice;
use argvus_time_core::store::{self, StoreError};
use argvus_time_core::world::WorldClock;
use argvus_time_ui::text::HourCycle;
use serde::{Deserialize, Serialize};

const VERSION: u32 = 1;
pub const FILE_NAME: &str = "settings.json";
pub const DEFAULT_ALARM_VOLUME: u8 = 80;
/// Lowest volume an audible alarm can be set to; "no sound" is chosen per
/// alarm instead, so an alarm is never silent by accident.
pub const MIN_ALARM_VOLUME: u8 = 10;
/// How long an alarm may ring before it stops by itself, in minutes.
pub const RING_LIMIT_CHOICES: [u32; 4] = [5, 10, 15, 30];
pub const DEFAULT_RING_LIMIT_MINUTES: u32 = 10;
pub const DEFAULT_TIMER_SECONDS: u64 = 5 * 60;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
  pub hour_cycle: HourCycle,
  pub show_seconds: bool,
  pub world_clocks: Vec<WorldClock>,
  /// Volume of alarms and the timer, 10 to 100.
  pub alarm_volume: u8,
  pub ring_limit_minutes: u32,
  /// Sound of the countdown timer.
  pub timer_sound: SoundChoice,
  /// Length last set on the timer, offered again next time.
  pub timer_seconds: u64,
  /// Keep running with the window closed while alarms or timers are
  /// pending.
  pub keep_in_background: bool,
}

impl Default for Settings {
  fn default() -> Self {
    Self {
      hour_cycle: HourCycle::default(),
      show_seconds: true,
      world_clocks: Vec::new(),
      alarm_volume: DEFAULT_ALARM_VOLUME,
      ring_limit_minutes: DEFAULT_RING_LIMIT_MINUTES,
      timer_sound: SoundChoice::default(),
      timer_seconds: DEFAULT_TIMER_SECONDS,
      keep_in_background: true,
    }
  }
}

impl Settings {
  pub fn normalized(mut self) -> Self {
    self.alarm_volume = self.alarm_volume.clamp(MIN_ALARM_VOLUME, 100);
    if !RING_LIMIT_CHOICES.contains(&self.ring_limit_minutes) {
      self.ring_limit_minutes = DEFAULT_RING_LIMIT_MINUTES;
    }
    if self.timer_seconds == 0 {
      self.timer_seconds = DEFAULT_TIMER_SECONDS;
    }
    // A world clock with an unknown zone, or the same zone twice, is dropped.
    let mut seen = Vec::new();
    self.world_clocks.retain(|clock| {
      let keep = clock.time_zone().is_some() && !seen.contains(&clock.zone);
      seen.push(clock.zone.clone());
      keep
    });
    self
  }

  pub fn volume_fraction(&self) -> f64 {
    f64::from(self.alarm_volume.clamp(MIN_ALARM_VOLUME, 100)) / 100.0
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
  fn defaults_are_audible_and_24_hour() {
    let settings = Settings::default();
    assert_eq!(settings.hour_cycle, HourCycle::H24);
    assert!(settings.alarm_volume >= MIN_ALARM_VOLUME);
    assert_ne!(settings.timer_sound, SoundChoice::Silent);
    assert!(settings.keep_in_background);
  }

  #[test]
  fn normalize_fixes_volume_limit_and_bad_world_clocks() {
    let settings = Settings {
      alarm_volume: 0,
      ring_limit_minutes: 7,
      world_clocks: vec![
        WorldClock {
          zone: "Asia/Tokyo".to_owned(),
          label: None,
        },
        WorldClock {
          zone: "Asia/Tokyo".to_owned(),
          label: Some("again".to_owned()),
        },
        WorldClock {
          zone: "Nowhere/Land".to_owned(),
          label: None,
        },
      ],
      ..Settings::default()
    }
    .normalized();
    assert_eq!(settings.alarm_volume, MIN_ALARM_VOLUME);
    assert_eq!(settings.ring_limit_minutes, DEFAULT_RING_LIMIT_MINUTES);
    assert_eq!(settings.world_clocks.len(), 1);
  }

  #[test]
  fn settings_round_trip() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join(FILE_NAME);
    let settings = Settings {
      hour_cycle: HourCycle::H12,
      show_seconds: false,
      ..Settings::default()
    };
    settings.save(&path).unwrap();
    assert_eq!(Settings::load(&path).unwrap().value, settings);
  }
}
