//! Alarms: recurrence in local time, snooze and late alarms.
//!
//! An occurrence is always computed from the local date, the alarm time and
//! the chosen weekdays in the given time zone, never by adding 24 hours, so an
//! alarm keeps its local time across daylight-saving changes.
//!
//! Each alarm remembers the last occurrence it handled. The scheduler stores
//! that before it rings, so reopening the application never rings the same
//! occurrence twice.

use std::path::PathBuf;

use chrono::{DateTime, Datelike, Days, NaiveDate, NaiveTime, TimeDelta, TimeZone, Utc, Weekday};
use serde::{Deserialize, Serialize};

/// An occurrence that is at most this late still rings. Later than that, the
/// application was closed or the system was asleep or off, and the alarm is
/// reported as missed instead. Same window as the calendar reminders.
pub const MISSED_GRACE_MINUTES: i64 = 5;
/// Snooze lengths accepted, in minutes.
pub const MIN_SNOOZE_MINUTES: u32 = 1;
pub const MAX_SNOOZE_MINUTES: u32 = 60;
pub const DEFAULT_SNOOZE_MINUTES: u32 = 10;
/// Longest daylight-saving gap that is skipped when an alarm time does not
/// exist on a given day.
const MAX_GAP_MINUTES: u64 = 180;

/// Stable identifier of an alarm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AlarmId(pub u64);

/// Weekdays on which an alarm repeats. Empty means it rings once.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WeekdaySet(u8);

impl WeekdaySet {
  pub const ONCE: Self = Self(0);
  pub const WEEKDAYS: Self = Self(0b0001_1111);
  pub const WEEKEND: Self = Self(0b0110_0000);
  pub const EVERY_DAY: Self = Self(0b0111_1111);

  /// Monday first, as the editor lists them.
  pub const ORDER: [Weekday; 7] = [
    Weekday::Mon,
    Weekday::Tue,
    Weekday::Wed,
    Weekday::Thu,
    Weekday::Fri,
    Weekday::Sat,
    Weekday::Sun,
  ];

  fn bit(day: Weekday) -> u8 {
    1 << day.num_days_from_monday()
  }

  pub fn contains(self, day: Weekday) -> bool {
    self.0 & Self::bit(day) != 0
  }

  pub fn with(self, day: Weekday, included: bool) -> Self {
    if included {
      Self(self.0 | Self::bit(day))
    } else {
      Self(self.0 & !Self::bit(day))
    }
  }

  pub fn is_once(self) -> bool {
    self.0 & Self::EVERY_DAY.0 == 0
  }

  pub fn days(self) -> impl Iterator<Item = Weekday> {
    Self::ORDER
      .into_iter()
      .filter(move |day| self.contains(*day))
  }
}

/// Hour and minute of an alarm, in local time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct AlarmTime {
  hour: u8,
  minute: u8,
}

impl AlarmTime {
  pub const MIDNIGHT: Self = Self { hour: 0, minute: 0 };

  pub fn new(hour: u8, minute: u8) -> Option<Self> {
    (hour < 24 && minute < 60).then_some(Self { hour, minute })
  }

  pub fn hour(self) -> u8 {
    self.hour
  }

  pub fn minute(self) -> u8 {
    self.minute
  }

  fn naive(self) -> NaiveTime {
    NaiveTime::from_hms_opt(u32::from(self.hour), u32::from(self.minute), 0)
      .unwrap_or(NaiveTime::MIN)
  }

  /// A stored value outside the valid range falls back to midnight.
  fn normalized(self) -> Self {
    Self::new(self.hour, self.minute).unwrap_or(Self::MIDNIGHT)
  }
}

/// Sounds shipped with ARGVUS.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BundledSound {
  #[default]
  Chime,
  Beacon,
}

impl BundledSound {
  pub const ALL: [Self; 2] = [Self::Chime, Self::Beacon];

  /// Stable ID, also the i18n key suffix (`sound.<id>`).
  pub fn id(self) -> &'static str {
    match self {
      Self::Chime => "chime",
      Self::Beacon => "beacon",
    }
  }

  /// File name in the ARGVUS sounds directory.
  pub fn file_name(self) -> &'static str {
    match self {
      Self::Chime => "argvus-chime.oga",
      Self::Beacon => "argvus-beacon.oga",
    }
  }
}

/// What an alarm plays when it rings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SoundChoice {
  Bundled {
    sound: BundledSound,
  },
  /// A local audio file chosen by the user.
  File {
    path: PathBuf,
  },
  /// Chosen explicitly; never the default.
  Silent,
}

impl Default for SoundChoice {
  fn default() -> Self {
    Self::Bundled {
      sound: BundledSound::default(),
    }
  }
}

/// The fields the user edits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlarmDraft {
  pub label: String,
  pub time: AlarmTime,
  pub repeat: WeekdaySet,
  pub sound: SoundChoice,
  pub snooze_minutes: u32,
}

/// A stored alarm.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Alarm {
  pub id: AlarmId,
  #[serde(default)]
  pub label: String,
  pub time: AlarmTime,
  #[serde(default)]
  pub repeat: WeekdaySet,
  pub enabled: bool,
  #[serde(default)]
  pub sound: SoundChoice,
  #[serde(default = "default_snooze_minutes")]
  pub snooze_minutes: u32,
  /// When the alarm was created, edited or switched on. Occurrences before
  /// this instant are never due, so a new alarm is not reported as missed.
  pub armed_at: DateTime<Utc>,
  /// The last occurrence that rang or was reported as missed.
  #[serde(default)]
  pub last_handled: Option<DateTime<Utc>>,
  #[serde(default)]
  pub snoozed_until: Option<DateTime<Utc>>,
}

fn default_snooze_minutes() -> u32 {
  DEFAULT_SNOOZE_MINUTES
}

/// When an alarm rings next.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Trigger {
  pub at: DateTime<Utc>,
  pub snoozed: bool,
}

/// Whether a due alarm should ring.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DueKind {
  OnTime,
  /// Later than [`MISSED_GRACE_MINUTES`]: reported, not rung.
  Missed,
}

/// An alarm whose trigger time has come.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Due {
  pub id: AlarmId,
  pub occurrence: DateTime<Utc>,
  pub snoozed: bool,
  pub kind: DueKind,
}

impl Alarm {
  fn new(id: AlarmId, draft: AlarmDraft, now: DateTime<Utc>) -> Self {
    let mut alarm = Self {
      id,
      label: String::new(),
      time: draft.time,
      repeat: WeekdaySet::ONCE,
      enabled: true,
      sound: SoundChoice::default(),
      snooze_minutes: DEFAULT_SNOOZE_MINUTES,
      armed_at: now,
      last_handled: None,
      snoozed_until: None,
    };
    alarm.apply(draft, now);
    alarm
  }

  /// Replaces the edited fields and arms the alarm again from `now`.
  fn apply(&mut self, draft: AlarmDraft, now: DateTime<Utc>) {
    self.label = draft.label.trim().to_owned();
    self.time = draft.time.normalized();
    self.repeat = draft.repeat;
    self.sound = draft.sound;
    self.snooze_minutes = clamp_snooze(draft.snooze_minutes);
    self.enabled = true;
    self.armed_at = now;
    self.snoozed_until = None;
  }

  pub fn draft(&self) -> AlarmDraft {
    AlarmDraft {
      label: self.label.clone(),
      time: self.time,
      repeat: self.repeat,
      sound: self.sound.clone(),
      snooze_minutes: self.snooze_minutes,
    }
  }

  /// Switches the alarm on (from `now`) or off. Turning it off also cancels
  /// a pending snooze.
  pub fn set_enabled(&mut self, enabled: bool, now: DateTime<Utc>) {
    if enabled && !self.enabled {
      self.armed_at = now;
    }
    if !enabled {
      self.snoozed_until = None;
    }
    self.enabled = enabled;
  }

  /// The next scheduled occurrence, ignoring snooze.
  pub fn next_occurrence<Tz: TimeZone>(&self, zone: &Tz) -> Option<DateTime<Utc>> {
    if !self.enabled {
      return None;
    }
    let floor = match self.last_handled {
      Some(handled) => handled.max(self.armed_at),
      None => self.armed_at,
    };
    let first_date = floor.with_timezone(zone).date_naive();
    // Eight days cover a whole week plus the day the search starts on.
    (0..=8)
      .filter_map(|offset| first_date.checked_add_days(Days::new(offset)))
      .filter(|date| self.repeat.is_once() || self.repeat.contains(date.weekday()))
      .filter_map(|date| resolve_local(zone, date, self.time.naive()))
      .find(|candidate| *candidate > floor)
  }

  /// When the alarm rings next: the snooze end if snoozed, otherwise the
  /// next occurrence.
  pub fn trigger<Tz: TimeZone>(&self, zone: &Tz) -> Option<Trigger> {
    if let Some(until) = self.snoozed_until {
      return Some(Trigger {
        at: until,
        snoozed: true,
      });
    }
    self
      .next_occurrence(zone)
      .map(|at| Trigger { at, snoozed: false })
  }

  /// The trigger, if it has come by `now`.
  pub fn due<Tz: TimeZone>(&self, now: DateTime<Utc>, zone: &Tz) -> Option<Due> {
    let trigger = self.trigger(zone).filter(|trigger| trigger.at <= now)?;
    let kind = if now - trigger.at <= TimeDelta::minutes(MISSED_GRACE_MINUTES) {
      DueKind::OnTime
    } else {
      DueKind::Missed
    };
    Some(Due {
      id: self.id,
      occurrence: trigger.at,
      snoozed: trigger.snoozed,
      kind,
    })
  }

  /// Records that a due trigger was handled, so it is never due again. A
  /// missed alarm consumes every occurrence up to `now`: a daily alarm left
  /// off for three days is reported once, not three times. A one-time alarm
  /// switches off.
  pub fn mark_handled(&mut self, due: &Due, now: DateTime<Utc>) {
    if due.snoozed {
      self.snoozed_until = None;
      return;
    }
    self.last_handled = Some(match due.kind {
      DueKind::OnTime => due.occurrence,
      DueKind::Missed => now,
    });
    if self.repeat.is_once() {
      self.enabled = false;
    }
  }

  /// Rings again after the alarm's snooze length. Returns the new time.
  pub fn snooze(&mut self, now: DateTime<Utc>) -> DateTime<Utc> {
    let until = now + TimeDelta::minutes(i64::from(clamp_snooze(self.snooze_minutes)));
    self.snoozed_until = Some(until);
    until
  }

  /// Cancels a pending snooze.
  pub fn dismiss(&mut self) {
    self.snoozed_until = None;
  }

  /// Whether the alarm can still ring.
  pub fn is_pending(&self) -> bool {
    self.enabled || self.snoozed_until.is_some()
  }
}

fn clamp_snooze(minutes: u32) -> u32 {
  minutes.clamp(MIN_SNOOZE_MINUTES, MAX_SNOOZE_MINUTES)
}

/// The instant of a local date and time. A time that does not exist (the
/// daylight-saving gap) moves to the first valid minute after it; a time that
/// exists twice (the fall-back hour) uses the earlier one.
fn resolve_local<Tz: TimeZone>(
  zone: &Tz,
  date: NaiveDate,
  time: NaiveTime,
) -> Option<DateTime<Utc>> {
  let local = date.and_time(time);
  (0..=MAX_GAP_MINUTES)
    .filter_map(|minutes| {
      local.checked_add_signed(TimeDelta::minutes(i64::try_from(minutes).ok()?))
    })
    .find_map(|candidate| zone.from_local_datetime(&candidate).earliest())
    .map(|instant| instant.with_timezone(&Utc))
}

/// All alarms, in the order the user sees them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AlarmBook {
  #[serde(default)]
  alarms: Vec<Alarm>,
  #[serde(default)]
  next_id: u64,
}

impl AlarmBook {
  pub fn alarms(&self) -> &[Alarm] {
    &self.alarms
  }

  pub fn get(&self, id: AlarmId) -> Option<&Alarm> {
    self.alarms.iter().find(|alarm| alarm.id == id)
  }

  pub fn get_mut(&mut self, id: AlarmId) -> Option<&mut Alarm> {
    self.alarms.iter_mut().find(|alarm| alarm.id == id)
  }

  pub fn add(&mut self, draft: AlarmDraft, now: DateTime<Utc>) -> AlarmId {
    let highest = self
      .alarms
      .iter()
      .map(|alarm| alarm.id.0 + 1)
      .max()
      .unwrap_or(0);
    let id = AlarmId(self.next_id.max(highest));
    self.next_id = id.0 + 1;
    self.alarms.push(Alarm::new(id, draft, now));
    self.sort();
    id
  }

  pub fn update(&mut self, id: AlarmId, draft: AlarmDraft, now: DateTime<Utc>) -> bool {
    let Some(alarm) = self.get_mut(id) else {
      return false;
    };
    alarm.apply(draft, now);
    self.sort();
    true
  }

  pub fn remove(&mut self, id: AlarmId) -> bool {
    let before = self.alarms.len();
    self.alarms.retain(|alarm| alarm.id != id);
    self.alarms.len() != before
  }

  /// Every alarm whose trigger has come, the earliest first.
  pub fn due<Tz: TimeZone>(&self, now: DateTime<Utc>, zone: &Tz) -> Vec<Due> {
    let mut due: Vec<Due> = self
      .alarms
      .iter()
      .filter_map(|alarm| alarm.due(now, zone))
      .collect();
    due.sort_by_key(|due| (due.occurrence, due.id));
    due
  }

  /// Takes every due trigger and marks it handled, so the same occurrence is
  /// never returned twice. The caller stores the book before ringing.
  pub fn take_due<Tz: TimeZone>(&mut self, now: DateTime<Utc>, zone: &Tz) -> Vec<Due> {
    let due = self.due(now, zone);
    for taken in &due {
      if let Some(alarm) = self.get_mut(taken.id) {
        alarm.mark_handled(taken, now);
      }
    }
    due
  }

  /// The alarm that rings next.
  pub fn next_trigger<Tz: TimeZone>(&self, zone: &Tz) -> Option<(AlarmId, Trigger)> {
    self
      .alarms
      .iter()
      .filter_map(|alarm| alarm.trigger(zone).map(|trigger| (alarm.id, trigger)))
      .min_by_key(|(id, trigger)| (trigger.at, *id))
  }

  pub fn has_pending(&self) -> bool {
    self.alarms.iter().any(Alarm::is_pending)
  }

  fn sort(&mut self) {
    self.alarms.sort_by_key(|alarm| (alarm.time, alarm.id));
  }
}

#[cfg(test)]
mod tests {
  use chrono::TimeZone as _;
  use chrono_tz::{America::Sao_Paulo, Europe::Berlin, Tz};

  use super::*;

  fn local(zone: Tz, year: i32, month: u32, day: u32, hour: u32, minute: u32) -> DateTime<Utc> {
    zone
      .with_ymd_and_hms(year, month, day, hour, minute, 0)
      .earliest()
      .expect("valid local time")
      .with_timezone(&Utc)
  }

  fn utc(year: i32, month: u32, day: u32, hour: u32, minute: u32) -> DateTime<Utc> {
    Utc
      .with_ymd_and_hms(year, month, day, hour, minute, 0)
      .unwrap()
  }

  fn draft(hour: u8, minute: u8, repeat: WeekdaySet) -> AlarmDraft {
    AlarmDraft {
      label: "Wake up".to_owned(),
      time: AlarmTime::new(hour, minute).unwrap(),
      repeat,
      sound: SoundChoice::default(),
      snooze_minutes: DEFAULT_SNOOZE_MINUTES,
    }
  }

  fn book_with(draft: AlarmDraft, created: DateTime<Utc>) -> (AlarmBook, AlarmId) {
    let mut book = AlarmBook::default();
    let id = book.add(draft, created);
    (book, id)
  }

  #[test]
  fn weekday_alarm_skips_the_weekend() {
    // Friday 2026-10-30, 08:00 in Berlin, after the 07:00 alarm.
    let (book, id) = book_with(
      draft(7, 0, WeekdaySet::WEEKDAYS),
      local(Berlin, 2026, 10, 30, 8, 0),
    );
    let next = book.get(id).unwrap().next_occurrence(&Berlin);
    assert_eq!(next, Some(local(Berlin, 2026, 11, 2, 7, 0)), "next Monday");
  }

  #[test]
  fn one_time_alarm_rings_today_or_tomorrow() {
    let (early, early_id) = book_with(
      draft(7, 0, WeekdaySet::ONCE),
      local(Berlin, 2026, 6, 1, 6, 0),
    );
    assert_eq!(
      early.get(early_id).unwrap().next_occurrence(&Berlin),
      Some(local(Berlin, 2026, 6, 1, 7, 0))
    );
    let (late, late_id) = book_with(
      draft(7, 0, WeekdaySet::ONCE),
      local(Berlin, 2026, 6, 1, 7, 0),
    );
    assert_eq!(
      late.get(late_id).unwrap().next_occurrence(&Berlin),
      Some(local(Berlin, 2026, 6, 2, 7, 0)),
      "an alarm set for this very minute waits for tomorrow"
    );
  }

  #[test]
  fn daily_alarm_keeps_local_time_across_daylight_saving() {
    // Berlin leaves summer time on 2026-10-25.
    let (mut book, id) = book_with(draft(7, 0, WeekdaySet::EVERY_DAY), utc(2026, 10, 24, 0, 0));
    let alarm = book.get_mut(id).unwrap();
    let saturday = alarm.next_occurrence(&Berlin).unwrap();
    assert_eq!(saturday, utc(2026, 10, 24, 5, 0), "07:00 CEST");
    let due = alarm.due(saturday, &Berlin).unwrap();
    alarm.mark_handled(&due, saturday);
    assert_eq!(
      alarm.next_occurrence(&Berlin),
      Some(utc(2026, 10, 25, 6, 0)),
      "07:00 CET"
    );
  }

  #[test]
  fn time_inside_the_spring_gap_moves_to_the_first_valid_minute() {
    // 02:30 does not exist in Berlin on 2026-03-29.
    let (book, id) = book_with(draft(2, 30, WeekdaySet::ONCE), utc(2026, 3, 28, 12, 0));
    let next = book.get(id).unwrap().next_occurrence(&Berlin);
    assert_eq!(next, Some(local(Berlin, 2026, 3, 29, 3, 0)));
  }

  #[test]
  fn time_inside_the_repeated_hour_uses_the_first_one() {
    // 02:30 happens twice in Berlin on 2026-10-25.
    let (book, id) = book_with(draft(2, 30, WeekdaySet::ONCE), utc(2026, 10, 24, 12, 0));
    let next = book.get(id).unwrap().next_occurrence(&Berlin);
    assert_eq!(next, Some(utc(2026, 10, 25, 0, 30)), "02:30 CEST");
  }

  #[test]
  fn midnight_gap_in_sao_paulo_is_handled() {
    // Brazil started summer time at midnight on 2018-11-04: 00:00-00:59 did not exist.
    let (book, id) = book_with(draft(0, 30, WeekdaySet::ONCE), utc(2018, 11, 3, 12, 0));
    let next = book.get(id).unwrap().next_occurrence(&Sao_Paulo);
    assert_eq!(next, Some(utc(2018, 11, 4, 3, 0)), "01:00 -02");
  }

  #[test]
  fn alarm_within_the_grace_window_rings_and_later_is_missed() {
    let (book, id) = book_with(draft(7, 0, WeekdaySet::EVERY_DAY), utc(2026, 6, 1, 0, 0));
    let alarm = book.get(id).unwrap();
    let occurrence = local(Berlin, 2026, 6, 1, 7, 0);
    assert_eq!(alarm.due(occurrence - TimeDelta::seconds(1), &Berlin), None);
    let on_time = alarm
      .due(occurrence + TimeDelta::minutes(4), &Berlin)
      .unwrap();
    assert_eq!(on_time.kind, DueKind::OnTime);
    let missed = alarm
      .due(occurrence + TimeDelta::minutes(6), &Berlin)
      .unwrap();
    assert_eq!(missed.kind, DueKind::Missed);
    assert_eq!(missed.occurrence, occurrence);
  }

  #[test]
  fn handled_occurrence_is_not_due_again_after_reload() {
    let (mut book, id) = book_with(draft(7, 0, WeekdaySet::EVERY_DAY), utc(2026, 6, 1, 0, 0));
    let now = local(Berlin, 2026, 6, 1, 7, 0) + TimeDelta::seconds(2);
    let due = book.due(now, &Berlin);
    assert_eq!(due.len(), 1);
    book.get_mut(id).unwrap().mark_handled(&due[0], now);

    let stored = serde_json::to_string(&book).unwrap();
    let reloaded: AlarmBook = serde_json::from_str(&stored).unwrap();
    assert!(
      reloaded
        .due(now + TimeDelta::seconds(30), &Berlin)
        .is_empty()
    );
    assert_eq!(
      reloaded.get(id).unwrap().next_occurrence(&Berlin),
      Some(local(Berlin, 2026, 6, 2, 7, 0))
    );
  }

  #[test]
  fn take_due_returns_each_occurrence_once() {
    let (mut book, _) = book_with(draft(7, 0, WeekdaySet::EVERY_DAY), utc(2026, 6, 1, 0, 0));
    let now = local(Berlin, 2026, 6, 1, 7, 0);
    assert_eq!(book.take_due(now, &Berlin).len(), 1);
    assert!(
      book
        .take_due(now + TimeDelta::seconds(1), &Berlin)
        .is_empty()
    );
    assert!(
      book
        .take_due(now + TimeDelta::minutes(30), &Berlin)
        .is_empty()
    );
  }

  #[test]
  fn missed_alarm_after_days_off_is_reported_once() {
    let (mut book, id) = book_with(draft(7, 0, WeekdaySet::EVERY_DAY), utc(2026, 6, 1, 0, 0));
    let three_days_later = local(Berlin, 2026, 6, 4, 9, 0);
    let due = book.due(three_days_later, &Berlin);
    assert_eq!(due.len(), 1);
    assert_eq!(due[0].kind, DueKind::Missed);
    book
      .get_mut(id)
      .unwrap()
      .mark_handled(&due[0], three_days_later);
    assert!(book.due(three_days_later, &Berlin).is_empty());
    assert_eq!(
      book.get(id).unwrap().next_occurrence(&Berlin),
      Some(local(Berlin, 2026, 6, 5, 7, 0))
    );
  }

  #[test]
  fn one_time_alarm_switches_off_but_can_still_snooze() {
    let (mut book, id) = book_with(draft(7, 0, WeekdaySet::ONCE), utc(2026, 6, 1, 0, 0));
    let now = local(Berlin, 2026, 6, 1, 7, 0);
    let alarm = book.get_mut(id).unwrap();
    let due = alarm.due(now, &Berlin).unwrap();
    alarm.mark_handled(&due, now);
    assert!(!alarm.enabled);
    assert_eq!(alarm.next_occurrence(&Berlin), None);

    let until = alarm.snooze(now);
    assert_eq!(until, now + TimeDelta::minutes(10));
    assert_eq!(alarm.due(until - TimeDelta::seconds(1), &Berlin), None);
    let snoozed = alarm.due(until, &Berlin).unwrap();
    assert!(snoozed.snoozed);
    alarm.mark_handled(&snoozed, until);
    assert!(!alarm.is_pending());
  }

  #[test]
  fn switching_off_cancels_snooze_and_switching_on_rearms() {
    let (mut book, id) = book_with(draft(7, 0, WeekdaySet::EVERY_DAY), utc(2026, 6, 1, 0, 0));
    let alarm = book.get_mut(id).unwrap();
    alarm.snooze(utc(2026, 6, 1, 5, 0));
    alarm.set_enabled(false, utc(2026, 6, 1, 5, 1));
    assert_eq!(alarm.snoozed_until, None);
    // Switched on again days later: the old occurrences are not missed.
    alarm.set_enabled(true, local(Berlin, 2026, 6, 10, 8, 0));
    assert!(
      alarm
        .due(local(Berlin, 2026, 6, 10, 8, 0), &Berlin)
        .is_none()
    );
  }

  #[test]
  fn two_alarms_due_together_are_both_returned_in_order() {
    let created = utc(2026, 6, 1, 0, 0);
    let mut book = AlarmBook::default();
    let second = book.add(draft(7, 1, WeekdaySet::EVERY_DAY), created);
    let first = book.add(draft(7, 0, WeekdaySet::EVERY_DAY), created);
    let due = book.due(local(Berlin, 2026, 6, 1, 7, 2), &Berlin);
    assert_eq!(
      due.iter().map(|due| due.id).collect::<Vec<_>>(),
      [first, second]
    );
    assert_eq!(book.alarms()[0].id, first, "alarms are listed by time");
  }

  #[test]
  fn ids_stay_unique_after_removal() {
    let created = utc(2026, 6, 1, 0, 0);
    let mut book = AlarmBook::default();
    let first = book.add(draft(7, 0, WeekdaySet::ONCE), created);
    let second = book.add(draft(8, 0, WeekdaySet::ONCE), created);
    assert!(book.remove(second));
    let third = book.add(draft(9, 0, WeekdaySet::ONCE), created);
    assert_ne!(third, first);
    assert_ne!(third, second);
  }

  #[test]
  fn edit_rearms_and_normalizes_the_snooze() {
    let (mut book, id) = book_with(draft(7, 0, WeekdaySet::ONCE), utc(2026, 6, 1, 0, 0));
    let mut edited = draft(6, 45, WeekdaySet::WEEKEND);
    edited.snooze_minutes = 500;
    assert!(book.update(id, edited, utc(2026, 6, 1, 1, 0)));
    let alarm = book.get(id).unwrap();
    assert_eq!(alarm.snooze_minutes, MAX_SNOOZE_MINUTES);
    assert!(alarm.repeat.contains(Weekday::Sat));
    assert!(!alarm.repeat.contains(Weekday::Mon));
  }

  #[test]
  fn sound_choice_round_trips_as_tagged_json() {
    let choices = [
      SoundChoice::default(),
      SoundChoice::File {
        path: PathBuf::from("/home/user/bell.ogg"),
      },
      SoundChoice::Silent,
    ];
    for choice in choices {
      let stored = serde_json::to_string(&choice).unwrap();
      assert_eq!(
        serde_json::from_str::<SoundChoice>(&stored).unwrap(),
        choice
      );
    }
    assert_eq!(
      serde_json::to_string(&SoundChoice::default()).unwrap(),
      r#"{"kind":"bundled","sound":"chime"}"#
    );
  }
}
