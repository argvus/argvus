//! Calendar dates typed on the GPG keys page.
//!
//! The date field follows the language: pt-BR reads the day first (`DD/MM/YYYY`),
//! every other language reads the year first (`YYYY/MM/DD`). The launcher and gpg
//! always receive the ISO form (`YYYY-MM-DD`), so the conversion happens here, once.
//! There is no date crate in this workspace, so the calendar rules are kept small
//! and tested below.
use std::time::{SystemTime, UNIX_EPOCH};

/// The order of the fields in a date typed by the user.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateOrder {
  /// `DD/MM/YYYY`, used by pt-BR.
  DayFirst,
  /// `YYYY/MM/DD`, used by every other language.
  YearFirst,
}

impl DateOrder {
  /// The order for an ARGVUS locale such as `pt-BR` or `en-US`.
  pub fn for_locale(locale: &str) -> Self {
    if locale.starts_with("pt") {
      Self::DayFirst
    } else {
      Self::YearFirst
    }
  }

  /// The layout shown to the user, such as `DD/MM/YYYY`.
  pub fn layout(self) -> &'static str {
    match self {
      Self::DayFirst => "DD/MM/YYYY",
      Self::YearFirst => "YYYY/MM/DD",
    }
  }
}

/// A calendar date. Only valid dates are constructed, by [`CalendarDate::parse`],
/// [`CalendarDate::today`] or [`CalendarDate::add_years`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CalendarDate {
  year: i32,
  month: u32,
  day: u32,
}

impl CalendarDate {
  /// Parses a date typed in `order`. Fields may be separated by `/`, `-` or `.`.
  /// Returns `None` for anything that is not a real calendar date.
  pub fn parse(text: &str, order: DateOrder) -> Option<Self> {
    let parts: Vec<&str> = text.trim().split(['/', '-', '.']).map(str::trim).collect();
    let [first, month, last] = parts.as_slice() else {
      return None;
    };
    let (year, day) = match order {
      DateOrder::DayFirst => (*last, *first),
      DateOrder::YearFirst => (*first, *last),
    };
    // A year is always four digits, so `12` cannot be read as the year 12.
    if year.len() != 4 {
      return None;
    }
    let year: i32 = year.parse().ok()?;
    let month: u32 = month.parse().ok()?;
    let day: u32 = day.parse().ok()?;
    if !(1..=12).contains(&month) || day == 0 || day > days_in_month(year, month) {
      return None;
    }
    Some(Self { year, month, day })
  }

  /// The date as `YYYY-MM-DD`, the form `gpg --quick-gen-key` accepts.
  pub fn iso(self) -> String {
    format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
  }

  /// The date in the user's layout, such as `03/05/2027` for pt-BR.
  pub fn display(self, order: DateOrder) -> String {
    match order {
      DateOrder::DayFirst => format!("{:02}/{:02}/{:04}", self.day, self.month, self.year),
      DateOrder::YearFirst => format!("{:04}/{:02}/{:02}", self.year, self.month, self.day),
    }
  }

  /// The same day `years` later. February 29 becomes February 28 when the target
  /// year is not a leap year.
  pub fn add_years(self, years: i32) -> Self {
    let year = self.year + years;
    let day = self.day.min(days_in_month(year, self.month));
    Self {
      year,
      month: self.month,
      day,
    }
  }

  /// The current date in UTC, which is the calendar gpg itself uses for expiry.
  pub fn today() -> Self {
    let seconds = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .map(|elapsed| elapsed.as_secs() as i64)
      .unwrap_or(0);
    let (year, month, day) = civil_from_days(seconds.div_euclid(86_400));
    Self { year, month, day }
  }
}

/// Whether a year is a leap year in the Gregorian calendar.
fn is_leap_year(year: i32) -> bool {
  (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// The number of days in a month, with February following the leap year rule.
fn days_in_month(year: i32, month: u32) -> u32 {
  match month {
    1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
    4 | 6 | 9 | 11 => 30,
    2 if is_leap_year(year) => 29,
    _ => 28,
  }
}

/// Converts days since 1970-01-01 to a `(year, month, day)` triple. This is the
/// standard civil-calendar algorithm, which works for any day count.
fn civil_from_days(days_since_epoch: i64) -> (i32, u32, u32) {
  let shifted = days_since_epoch + 719_468;
  let era = shifted.div_euclid(146_097);
  let day_of_era = shifted - era * 146_097;
  let year_of_era =
    (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
  let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
  let month_index = (5 * day_of_year + 2) / 153;
  let day = day_of_year - (153 * month_index + 2) / 5 + 1;
  let month = if month_index < 10 {
    month_index + 3
  } else {
    month_index - 9
  };
  let year = year_of_era + era * 400 + i64::from(month <= 2);
  (year as i32, month as u32, day as u32)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn reads_the_day_first_layout_for_pt_br() {
    let order = DateOrder::for_locale("pt-BR");
    let parsed = CalendarDate::parse("03/05/2027", order).expect("valid date");
    assert_eq!(parsed.iso(), "2027-05-03");
    assert_eq!(parsed.display(order), "03/05/2027");
  }

  #[test]
  fn reads_the_year_first_layout_for_en_us() {
    let order = DateOrder::for_locale("en-US");
    let parsed = CalendarDate::parse("2027/05/03", order).expect("valid date");
    assert_eq!(parsed.iso(), "2027-05-03");
    assert_eq!(parsed.display(order), "2027/05/03");
  }

  #[test]
  fn refuses_dates_that_do_not_exist() {
    let order = DateOrder::for_locale("en-US");
    assert!(CalendarDate::parse("2027/02/29", order).is_none());
    assert!(CalendarDate::parse("2027/13/01", order).is_none());
    assert!(CalendarDate::parse("2027/04/31", order).is_none());
    assert!(CalendarDate::parse("2027/00/10", order).is_none());
  }

  #[test]
  fn accepts_leap_day_only_in_leap_years() {
    let order = DateOrder::for_locale("en-US");
    assert!(CalendarDate::parse("2028/02/29", order).is_some());
    assert!(CalendarDate::parse("2100/02/29", order).is_none());
    assert!(CalendarDate::parse("2000/02/29", order).is_some());
  }

  #[test]
  fn refuses_text_and_short_years() {
    let order = DateOrder::for_locale("pt-BR");
    assert!(CalendarDate::parse("", order).is_none());
    assert!(CalendarDate::parse("abc", order).is_none());
    assert!(CalendarDate::parse("03/05/27", order).is_none());
    assert!(CalendarDate::parse("03/05", order).is_none());
  }

  #[test]
  fn adds_years_and_keeps_a_valid_february() {
    let leap_day = CalendarDate::parse("2028/02/29", DateOrder::YearFirst).expect("valid date");
    assert_eq!(leap_day.add_years(1).iso(), "2029-02-28");
    assert_eq!(leap_day.add_years(4).iso(), "2032-02-29");
  }

  #[test]
  fn converts_the_epoch_and_a_later_day() {
    assert_eq!(civil_from_days(0), (1970, 1, 1));
    // 2000-03-01 is 11017 days after the epoch.
    assert_eq!(civil_from_days(11_017), (2000, 3, 1));
  }
}
