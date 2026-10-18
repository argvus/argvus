use chrono::{Datelike, NaiveDate};

use crate::calendar::{WeekStart, month_grid};

pub fn day_cells(month: NaiveDate, week_start: WeekStart) -> Vec<(NaiveDate, bool)> {
  month_grid(month, week_start)
    .into_iter()
    .map(|date| (date, date.month() == month.month()))
    .collect()
}

pub fn weekday_labels(
  headers_monday_first: [&'static str; 7],
  week_start: WeekStart,
) -> [&'static str; 7] {
  if matches!(week_start, WeekStart::Sunday) {
    [
      headers_monday_first[6],
      headers_monday_first[0],
      headers_monday_first[1],
      headers_monday_first[2],
      headers_monday_first[3],
      headers_monday_first[4],
      headers_monday_first[5],
    ]
  } else {
    headers_monday_first
  }
}
