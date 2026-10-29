//! The alarms file.

use std::path::Path;

use argvus_time_core::alarm::AlarmBook;
use argvus_time_core::store::{self, Loaded, StoreError};

const VERSION: u32 = 1;
pub const FILE_NAME: &str = "alarms.json";

pub fn load(path: &Path) -> Result<Loaded<AlarmBook>, StoreError> {
  store::load_json(path, VERSION)
}

pub fn save(path: &Path, book: &AlarmBook) -> Result<(), StoreError> {
  store::save_json(path, VERSION, book)
}

#[cfg(test)]
mod tests {
  use argvus_time_core::alarm::{AlarmDraft, AlarmTime, SoundChoice, WeekdaySet};
  use chrono::Utc;

  use super::*;

  #[test]
  fn alarms_survive_a_restart() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join(FILE_NAME);
    let mut book = AlarmBook::default();
    let id = book.add(
      AlarmDraft {
        label: "Standup".to_owned(),
        time: AlarmTime::new(9, 30).unwrap(),
        repeat: WeekdaySet::WEEKDAYS,
        sound: SoundChoice::Silent,
        snooze_minutes: 5,
      },
      Utc::now(),
    );
    save(&path, &book).unwrap();
    let reloaded = load(&path).unwrap().value;
    assert_eq!(reloaded, book);
    assert_eq!(reloaded.get(id).unwrap().label, "Standup");
  }
}
