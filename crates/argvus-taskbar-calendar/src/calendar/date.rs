use chrono::{DateTime, Datelike, Days, Local, NaiveDate, TimeZone, Utc};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum WeekStart {
  Monday,
  Sunday,
}

impl WeekStart {
  pub fn offset_for(self, date: NaiveDate) -> u32 {
    match self {
      Self::Monday => date.weekday().num_days_from_monday(),
      Self::Sunday => date.weekday().num_days_from_sunday(),
    }
  }
}

pub fn today() -> NaiveDate {
  Local::now().date_naive()
}

#[allow(dead_code)]
pub fn start_of_local_day_utc(date: NaiveDate) -> DateTime<Utc> {
  Local
    .from_local_datetime(&date.and_hms_opt(0, 0, 0).expect("valid midnight"))
    .earliest()
    .expect("local day has a start")
    .with_timezone(&Utc)
}

#[allow(dead_code)]
pub fn start_of_today_utc() -> DateTime<Utc> {
  start_of_local_day_utc(today())
}

pub fn first_of_month(date: NaiveDate) -> NaiveDate {
  NaiveDate::from_ymd_opt(date.year(), date.month(), 1).expect("valid first day")
}

pub fn next_month(date: NaiveDate) -> NaiveDate {
  let (year, month) = if date.month() == 12 {
    (date.year() + 1, 1)
  } else {
    (date.year(), date.month() + 1)
  };
  NaiveDate::from_ymd_opt(year, month, 1).expect("valid next month")
}

pub fn previous_month(date: NaiveDate) -> NaiveDate {
  let (year, month) = if date.month() == 1 {
    (date.year() - 1, 12)
  } else {
    (date.year(), date.month() - 1)
  };
  NaiveDate::from_ymd_opt(year, month, 1).expect("valid previous month")
}

pub fn shift_year_clamped(date: NaiveDate, years: i32) -> NaiveDate {
  let target_year = date.year() + years;
  let mut day = date.day();
  loop {
    if let Some(next) = NaiveDate::from_ymd_opt(target_year, date.month(), day) {
      return next;
    }
    day -= 1;
  }
}

pub fn month_grid(anchor: NaiveDate, week_start: WeekStart) -> Vec<NaiveDate> {
  let first = first_of_month(anchor);
  let offset = week_start.offset_for(first) as u64;
  let start = first.checked_sub_days(Days::new(offset)).unwrap_or(first);
  (0..42)
    .map(|days| {
      start
        .checked_add_days(Days::new(days))
        .expect("valid grid day")
    })
    .collect()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn shift_year_preserves_valid_date() {
    let date = NaiveDate::from_ymd_opt(2026, 8, 11).unwrap();
    assert_eq!(
      shift_year_clamped(date, -1),
      NaiveDate::from_ymd_opt(2025, 8, 11).unwrap()
    );
  }

  #[test]
  fn shift_year_clamps_leap_day() {
    let date = NaiveDate::from_ymd_opt(2024, 2, 29).unwrap();
    assert_eq!(
      shift_year_clamped(date, 1),
      NaiveDate::from_ymd_opt(2025, 2, 28).unwrap()
    );
  }

  #[test]
  fn local_day_start_round_trips_to_midnight() {
    let date = NaiveDate::from_ymd_opt(2026, 8, 12).unwrap();
    let local = start_of_local_day_utc(date).with_timezone(&Local);
    assert_eq!(local.date_naive(), date);
    assert_eq!(local.time(), chrono::NaiveTime::MIN);
  }
}
