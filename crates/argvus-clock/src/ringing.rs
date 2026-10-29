//! Alarms and timers that are ringing now.
//!
//! Several can ring at once: a second alarm is added to the queue and shown
//! with the first, never replacing it silently. One sound plays for the whole
//! queue: the sound of the oldest entry that has one. Each entry stops by
//! itself after the ring limit, so a forgotten alarm does not play forever.

use std::path::PathBuf;
use std::time::Duration;

use argvus_time_core::alarm::{AlarmId, BundledSound, SoundChoice};
use chrono::{DateTime, Utc};

/// What is ringing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
  Alarm(AlarmId),
  Timer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
  pub source: Source,
  pub title: String,
  /// When it was due.
  pub due_at: DateTime<Utc>,
  /// Boot time when it started ringing.
  pub started: Duration,
  pub sound: SoundChoice,
  /// Set when it rings late because the system was asleep.
  pub late_by: Option<Duration>,
}

#[derive(Debug, Clone, Default)]
pub struct RingQueue {
  entries: Vec<Entry>,
}

impl RingQueue {
  pub fn entries(&self) -> &[Entry] {
    &self.entries
  }

  pub fn is_empty(&self) -> bool {
    self.entries.is_empty()
  }

  /// Adds an entry. An alarm or timer already ringing is replaced by its new
  /// occurrence instead of being listed twice.
  pub fn push(&mut self, entry: Entry) {
    self
      .entries
      .retain(|ringing| ringing.source != entry.source);
    self.entries.push(entry);
  }

  pub fn remove(&mut self, source: Source) -> Option<Entry> {
    let index = self
      .entries
      .iter()
      .position(|entry| entry.source == source)?;
    Some(self.entries.remove(index))
  }

  /// Removes and returns the entries that rang for `limit` or longer.
  pub fn take_expired(&mut self, now: Duration, limit: Duration) -> Vec<Entry> {
    let (expired, ringing) = self
      .entries
      .drain(..)
      .partition(|entry| now.saturating_sub(entry.started) >= limit);
    self.entries = ringing;
    expired
  }

  /// The sound the queue plays: the oldest entry that is not silent.
  pub fn sound(&self) -> Option<&SoundChoice> {
    self
      .entries
      .iter()
      .map(|entry| &entry.sound)
      .find(|sound| **sound != SoundChoice::Silent)
  }
}

/// The file to play for a choice. A user file that no longer exists falls
/// back to the packaged chime; the second value tells the caller to warn.
pub fn sound_path(choice: &SoundChoice) -> Option<(PathBuf, bool)> {
  let bundled = |sound: BundledSound| argvus_time_ui::sound::bundled_sound(sound.file_name());
  match choice {
    SoundChoice::Silent => None,
    SoundChoice::Bundled { sound } => Some((bundled(*sound), false)),
    SoundChoice::File { path } if path.is_file() => Some((path.clone(), false)),
    SoundChoice::File { .. } => Some((bundled(BundledSound::Chime), true)),
  }
}

#[cfg(test)]
mod tests {
  use chrono::TimeZone;

  use super::*;

  fn entry(source: Source, started: u64, sound: SoundChoice) -> Entry {
    Entry {
      source,
      title: "Alarm".to_owned(),
      due_at: Utc.with_ymd_and_hms(2026, 10, 28, 7, 0, 0).unwrap(),
      started: Duration::from_secs(started),
      sound,
      late_by: None,
    }
  }

  #[test]
  fn second_alarm_joins_the_queue_without_replacing_the_first() {
    let mut queue = RingQueue::default();
    queue.push(entry(Source::Alarm(AlarmId(1)), 0, SoundChoice::default()));
    queue.push(entry(Source::Alarm(AlarmId(2)), 5, SoundChoice::Silent));
    assert_eq!(queue.entries().len(), 2);
    assert_eq!(queue.sound(), Some(&SoundChoice::default()));
    queue.push(entry(Source::Alarm(AlarmId(1)), 9, SoundChoice::default()));
    assert_eq!(
      queue.entries().len(),
      2,
      "the same alarm is not listed twice"
    );
  }

  #[test]
  fn silent_entries_do_not_play_and_others_still_do() {
    let mut queue = RingQueue::default();
    queue.push(entry(Source::Alarm(AlarmId(1)), 0, SoundChoice::Silent));
    assert_eq!(queue.sound(), None);
    let beacon = SoundChoice::Bundled {
      sound: BundledSound::Beacon,
    };
    queue.push(entry(Source::Timer, 1, beacon.clone()));
    assert_eq!(queue.sound(), Some(&beacon));
    queue.remove(Source::Timer);
    assert_eq!(queue.sound(), None);
  }

  #[test]
  fn entries_stop_after_the_ring_limit() {
    let mut queue = RingQueue::default();
    queue.push(entry(Source::Alarm(AlarmId(1)), 0, SoundChoice::default()));
    queue.push(entry(Source::Timer, 300, SoundChoice::default()));
    let expired = queue.take_expired(Duration::from_secs(600), Duration::from_secs(600));
    assert_eq!(expired.len(), 1);
    assert_eq!(expired[0].source, Source::Alarm(AlarmId(1)));
    assert_eq!(queue.entries().len(), 1);
  }

  #[test]
  fn missing_user_file_falls_back_to_the_chime() {
    let (path, warn) = sound_path(&SoundChoice::File {
      path: PathBuf::from("/nonexistent/argvus/alarm.ogg"),
    })
    .unwrap();
    assert!(warn);
    assert!(path.ends_with(BundledSound::Chime.file_name()));
    assert_eq!(sound_path(&SoundChoice::Silent), None);
  }
}
