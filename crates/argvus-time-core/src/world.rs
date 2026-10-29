//! IANA time zones for the world clocks.
//!
//! Offsets and daylight-saving rules come from `chrono-tz` (the IANA
//! database); nothing here computes an offset by hand.

use chrono::{DateTime, NaiveDate, Offset, TimeZone, Utc};
use chrono_tz::{TZ_VARIANTS, Tz};
use serde::{Deserialize, Serialize};

/// Regions of the canonical zone names. Legacy aliases such as `US/Eastern`,
/// `Etc/GMT+3` or `EST` are not offered for new clocks.
const LISTED_REGIONS: [&str; 10] = [
  "Africa",
  "America",
  "Antarctica",
  "Arctic",
  "Asia",
  "Atlantic",
  "Australia",
  "Europe",
  "Indian",
  "Pacific",
];

/// A saved world clock.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldClock {
  /// IANA zone name, for example `America/Sao_Paulo`.
  pub zone: String,
  /// Name chosen by the user; the city name is used when empty.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub label: Option<String>,
}

impl WorldClock {
  pub fn new(zone: Tz) -> Self {
    Self {
      zone: zone.name().to_owned(),
      label: None,
    }
  }

  /// The zone, or `None` when the stored name is unknown.
  pub fn time_zone(&self) -> Option<Tz> {
    parse_zone(&self.zone)
  }

  pub fn display_name(&self) -> String {
    match self.label.as_deref().map(str::trim) {
      Some(label) if !label.is_empty() => label.to_owned(),
      _ => city_name(&self.zone),
    }
  }
}

pub fn parse_zone(name: &str) -> Option<Tz> {
  name.parse().ok()
}

/// Name of the system time zone, for example `Europe/Berlin`.
pub fn local_zone_name() -> Option<String> {
  iana_time_zone::get_timezone().ok()
}

/// The city part of a zone name: `America/Argentina/Buenos_Aires` gives
/// `Buenos Aires`.
pub fn city_name(zone: &str) -> String {
  zone.rsplit('/').next().unwrap_or(zone).replace('_', " ")
}

/// The region part of a zone name: `America/Argentina/Buenos_Aires` gives
/// `America / Argentina`.
pub fn region_name(zone: &str) -> String {
  match zone.rsplit_once('/') {
    Some((region, _)) => region.replace('_', " ").replace('/', " / "),
    None => String::new(),
  }
}

fn is_listed(zone: &str) -> bool {
  zone
    .split_once('/')
    .is_some_and(|(region, _)| LISTED_REGIONS.contains(&region))
}

/// Zones whose name or city matches `query`, ignoring case, accents,
/// underscores and slashes. An empty query lists every zone. Sorted by city.
pub fn search(query: &str) -> Vec<Tz> {
  let needle = fold(query);
  let mut zones: Vec<Tz> = TZ_VARIANTS
    .iter()
    .copied()
    .filter(|zone| is_listed(zone.name()))
    .filter(|zone| needle.is_empty() || fold(zone.name()).contains(&needle))
    .collect();
  zones.sort_by_cached_key(|zone| (fold(&city_name(zone.name())), zone.name()));
  zones
}

/// Lowercase, without accents, with separators turned into single spaces.
fn fold(text: &str) -> String {
  let mapped: String = text
    .chars()
    .flat_map(char::to_lowercase)
    .map(|character| match character {
      'á' | 'à' | 'â' | 'ã' | 'ä' | 'å' => 'a',
      'é' | 'è' | 'ê' | 'ë' => 'e',
      'í' | 'ì' | 'î' | 'ï' => 'i',
      'ó' | 'ò' | 'ô' | 'õ' | 'ö' => 'o',
      'ú' | 'ù' | 'û' | 'ü' => 'u',
      'ç' => 'c',
      'ñ' => 'n',
      '_' | '/' | '-' => ' ',
      other => other,
    })
    .collect();
  mapped.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Offset from UTC, in seconds, of `zone` at `at`.
pub fn utc_offset_seconds<Z: TimeZone>(zone: &Z, at: DateTime<Utc>) -> i32 {
  zone
    .offset_from_utc_datetime(&at.naive_utc())
    .fix()
    .local_minus_utc()
}

/// `UTC+03:00`, `UTC-03:30` or `UTC`.
pub fn format_utc_offset(seconds: i32) -> String {
  if seconds == 0 {
    return "UTC".to_owned();
  }
  let sign = if seconds < 0 { '-' } else { '+' };
  let minutes = seconds.unsigned_abs() / 60;
  format!("UTC{sign}{:02}:{:02}", minutes / 60, minutes % 60)
}

/// Where a date falls relative to the local date.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DayRelation {
  Yesterday,
  Today,
  Tomorrow,
}

pub fn day_relation(local_date: NaiveDate, other_date: NaiveDate) -> DayRelation {
  match other_date.cmp(&local_date) {
    std::cmp::Ordering::Less => DayRelation::Yesterday,
    std::cmp::Ordering::Equal => DayRelation::Today,
    std::cmp::Ordering::Greater => DayRelation::Tomorrow,
  }
}

#[cfg(test)]
mod tests {
  use chrono::TimeZone as _;
  use chrono_tz::{America::Sao_Paulo, Asia::Kolkata, Europe::Berlin};

  use super::*;

  #[test]
  fn city_and_region_come_from_the_zone_name() {
    assert_eq!(city_name("America/Argentina/Buenos_Aires"), "Buenos Aires");
    assert_eq!(
      region_name("America/Argentina/Buenos_Aires"),
      "America / Argentina"
    );
    assert_eq!(city_name("UTC"), "UTC");
  }

  #[test]
  fn search_ignores_accents_case_and_legacy_aliases() {
    let accented = search("São Paulo");
    assert!(accented.contains(&Sao_Paulo));
    assert_eq!(search("sao_paulo"), accented);
    assert!(search("").iter().all(|zone| is_listed(zone.name())));
    assert!(
      !search("eastern")
        .iter()
        .any(|zone| zone.name() == "US/Eastern")
    );
  }

  #[test]
  fn offsets_follow_daylight_saving() {
    let winter = Utc.with_ymd_and_hms(2026, 1, 15, 12, 0, 0).unwrap();
    let summer = Utc.with_ymd_and_hms(2026, 7, 15, 12, 0, 0).unwrap();
    assert_eq!(
      format_utc_offset(utc_offset_seconds(&Berlin, winter)),
      "UTC+01:00"
    );
    assert_eq!(
      format_utc_offset(utc_offset_seconds(&Berlin, summer)),
      "UTC+02:00"
    );
    assert_eq!(
      format_utc_offset(utc_offset_seconds(&Kolkata, summer)),
      "UTC+05:30"
    );
    assert_eq!(
      format_utc_offset(utc_offset_seconds(&Sao_Paulo, summer)),
      "UTC-03:00"
    );
    assert_eq!(format_utc_offset(0), "UTC");
  }

  #[test]
  fn day_relation_compares_dates() {
    let today = NaiveDate::from_ymd_opt(2026, 10, 28).unwrap();
    assert_eq!(
      day_relation(today, today.pred_opt().unwrap()),
      DayRelation::Yesterday
    );
    assert_eq!(day_relation(today, today), DayRelation::Today);
    assert_eq!(
      day_relation(today, today.succ_opt().unwrap()),
      DayRelation::Tomorrow
    );
  }

  #[test]
  fn world_clock_uses_label_or_city() {
    let mut clock = WorldClock::new(Kolkata);
    assert_eq!(clock.display_name(), "Kolkata");
    clock.label = Some("  Office  ".to_owned());
    assert_eq!(clock.display_name(), "Office");
    assert_eq!(clock.time_zone(), Some(Kolkata));
    let unknown = WorldClock {
      zone: "Mars/Olympus".to_owned(),
      label: None,
    };
    assert_eq!(unknown.time_zone(), None);
  }
}
