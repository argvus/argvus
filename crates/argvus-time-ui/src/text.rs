//! Translations and localized times and dates.
//!
//! Strings come from the shared ARGVUS catalogs (`argvus-i18n`). Weekday and
//! month names, the long date pattern and the AM/PM markers live in the
//! `time` domain, shared by both applications; each application keeps its
//! own strings in its own domain.

use argvus_i18n::I18n;
use chrono::{Datelike, Timelike, Weekday};
use serde::{Deserialize, Serialize};

/// Domain of the strings both time applications use.
pub const SHARED_DOMAIN: &str = "time";

/// A translation catalog that always answers. When the catalogs cannot be
/// read (a broken install), keys are shown instead of crashing, and the
/// problem is logged once.
pub struct Catalog {
  translations: Option<I18n>,
}

impl Catalog {
  pub fn load(domain: &str) -> Self {
    let translations = I18n::new(domain)
      .inspect_err(|error| log::error!("cannot load the '{domain}' translations: {error}"))
      .ok();
    Self { translations }
  }

  pub fn locale(&self) -> &str {
    self.translations.as_ref().map_or("en-US", I18n::locale)
  }

  pub fn tr(&self, key: &str) -> String {
    match &self.translations {
      Some(translations) => translations.tr(key),
      None => key.to_owned(),
    }
  }

  pub fn tr_args<I, K, V>(&self, key: &str, args: I) -> String
  where
    I: IntoIterator<Item = (K, V)>,
    K: AsRef<str>,
    V: AsRef<str>,
  {
    match &self.translations {
      Some(translations) => translations.tr_args(key, args),
      None => key.to_owned(),
    }
  }

  pub fn tr_plural(&self, key: &str, count: i64) -> String {
    match &self.translations {
      Some(translations) => translations.tr_plural(key, count),
      None => format!("{key} {count}"),
    }
  }
}

/// 24-hour (the ARGVUS default) or 12-hour clock.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HourCycle {
  #[default]
  H24,
  H12,
}

/// A formatted time: the digits and, on a 12-hour clock, the AM/PM marker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeText {
  pub digits: String,
  pub period: Option<String>,
}

impl TimeText {
  /// Digits and marker in one string, for lists.
  pub fn joined(&self) -> String {
    match &self.period {
      Some(period) => format!("{} {period}", self.digits),
      None => self.digits.clone(),
    }
  }
}

/// Localized names and formats, from the shared `time` domain.
pub struct Calendar {
  catalog: Catalog,
}

impl Calendar {
  pub fn load() -> Self {
    Self {
      catalog: Catalog::load(SHARED_DOMAIN),
    }
  }

  pub fn from_catalog(catalog: Catalog) -> Self {
    Self { catalog }
  }

  pub fn catalog(&self) -> &Catalog {
    &self.catalog
  }

  pub fn weekday(&self, day: Weekday) -> String {
    self.catalog.tr(&format!("weekday.{}", weekday_key(day)))
  }

  pub fn weekday_short(&self, day: Weekday) -> String {
    self
      .catalog
      .tr(&format!("weekday.{}.short", weekday_key(day)))
  }

  pub fn month(&self, month: u32) -> String {
    self.catalog.tr(&format!("month.{month}"))
  }

  /// For example "Wednesday, October 28, 2026" or "quarta-feira, 28 de
  /// outubro de 2026".
  pub fn long_date(&self, date: &impl Datelike) -> String {
    self.catalog.tr_args(
      "date.long",
      [
        ("weekday", self.weekday(date.weekday())),
        ("day", date.day().to_string()),
        ("month", self.month(date.month())),
        ("year", date.year().to_string()),
      ],
    )
  }

  /// For example "Wed, Oct 28".
  pub fn short_date(&self, date: &impl Datelike) -> String {
    self.catalog.tr_args(
      "date.short",
      [
        ("weekday", self.weekday_short(date.weekday())),
        ("day", date.day().to_string()),
        ("month", self.month(date.month())),
      ],
    )
  }

  pub fn time(&self, time: &impl Timelike, cycle: HourCycle, with_seconds: bool) -> TimeText {
    let (hour, period) = match cycle {
      HourCycle::H24 => (format!("{:02}", time.hour()), None),
      HourCycle::H12 => {
        let (is_pm, hour) = time.hour12();
        let marker = if is_pm { "time.pm" } else { "time.am" };
        (hour.to_string(), Some(self.catalog.tr(marker)))
      }
    };
    let digits = if with_seconds {
      format!("{hour}:{:02}:{:02}", time.minute(), time.second())
    } else {
      format!("{hour}:{:02}", time.minute())
    };
    TimeText { digits, period }
  }
}

fn weekday_key(day: Weekday) -> &'static str {
  match day {
    Weekday::Mon => "mon",
    Weekday::Tue => "tue",
    Weekday::Wed => "wed",
    Weekday::Thu => "thu",
    Weekday::Fri => "fri",
    Weekday::Sat => "sat",
    Weekday::Sun => "sun",
  }
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;

  use chrono::{NaiveDate, NaiveTime};

  use super::*;

  fn calendar(locale: &str) -> Calendar {
    // `cargo test` sets ARGVUS_I18N_DIR to the source catalogs.
    let root = PathBuf::from(std::env::var_os("ARGVUS_I18N_DIR").expect("ARGVUS_I18N_DIR"));
    let translations = I18n::from_locale(&root, locale, SHARED_DOMAIN).expect("time catalog");
    Calendar::from_catalog(Catalog {
      translations: Some(translations),
    })
  }

  #[test]
  fn long_date_is_localized() {
    let date = NaiveDate::from_ymd_opt(2026, 10, 28).unwrap();
    assert_eq!(
      calendar("en-US").long_date(&date),
      "Wednesday, October 28, 2026"
    );
    assert_eq!(
      calendar("pt-BR").long_date(&date),
      "quarta-feira, 28 de outubro de 2026"
    );
  }

  #[test]
  fn times_follow_the_hour_cycle() {
    let english = calendar("en-US");
    let afternoon = NaiveTime::from_hms_opt(14, 5, 9).unwrap();
    assert_eq!(
      english.time(&afternoon, HourCycle::H24, false).joined(),
      "14:05"
    );
    assert_eq!(
      english.time(&afternoon, HourCycle::H24, true).joined(),
      "14:05:09"
    );
    assert_eq!(
      english.time(&afternoon, HourCycle::H12, false).joined(),
      "2:05 PM"
    );
    let midnight = NaiveTime::from_hms_opt(0, 30, 0).unwrap();
    assert_eq!(
      english.time(&midnight, HourCycle::H12, false).joined(),
      "12:30 AM"
    );
  }

  #[test]
  fn every_weekday_and_month_has_a_name() {
    for locale in ["en-US", "pt-BR"] {
      let calendar = calendar(locale);
      for day in [
        Weekday::Mon,
        Weekday::Tue,
        Weekday::Wed,
        Weekday::Thu,
        Weekday::Fri,
        Weekday::Sat,
        Weekday::Sun,
      ] {
        assert!(
          !calendar.weekday(day).starts_with("weekday."),
          "{locale} {day}"
        );
        assert!(
          !calendar.weekday_short(day).starts_with("weekday."),
          "{locale} {day}"
        );
      }
      for month in 1..=12 {
        assert!(
          !calendar.month(month).starts_with("month."),
          "{locale} {month}"
        );
      }
    }
  }

  #[test]
  fn broken_catalog_shows_keys_instead_of_failing() {
    let catalog = Catalog { translations: None };
    assert_eq!(catalog.tr("phase.focus"), "phase.focus");
    assert_eq!(catalog.locale(), "en-US");
  }
}
