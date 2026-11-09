//! Projects the model into the widgets: calendar grid, agenda, editor and settings pages.

use super::drafts::can_create_event;
use super::*;

pub(super) fn render_model(
  model: &AppModel,
  widgets: &mut AppWidgets,
  sender: ComponentSender<AppModel>,
) {
  widgets
    .title
    .set_label(&model.i18n.month_title(model.month));
  let show_events = model.events_enabled;
  for (button, (date, in_month)) in widgets
    .day_buttons
    .iter()
    .zip(day_cells(model.month, model.settings.week_start()))
  {
    let has_event = show_events && model.event_days.contains(&date);
    let label = if has_event {
      format!("{:02}•", date.day())
    } else {
      format!("{:02}", date.day())
    };
    button.set_label(&label);
    unsafe {
      button.set_data("date", date);
    }
    for class in [
      "day-outside-month",
      "day-current",
      "day-selected",
      "day-has-event",
    ] {
      button.remove_css_class(class);
    }
    if !in_month {
      button.add_css_class("day-outside-month");
    }
    if date == today() {
      button.add_css_class("day-current");
    }
    if date == model.selected {
      button.add_css_class("day-selected");
    }
    if has_event {
      button.add_css_class("day-has-event");
    }
  }
  unsafe {
    widgets.add_button.set_data("selected-date", model.selected);
  }
  widgets
    .agenda_title
    .set_label(&model.i18n.agenda_date(model.selected));
  widgets
    .add_button
    .set_label(model.i18n.text(Text::AddEvent));
  let can_add_event = show_events && can_create_event(model.selected);
  widgets.add_button.set_sensitive(can_add_event);
  widgets.add_button.set_tooltip_text(
    (!can_add_event && model.selected < today()).then_some(model.i18n.text(Text::PastDate)),
  );
  widgets
    .prev_year
    .set_tooltip_text(Some(model.i18n.text(Text::PreviousYear)));
  widgets
    .prev_month
    .set_tooltip_text(Some(model.i18n.text(Text::PreviousMonth)));
  widgets
    .today_button
    .set_tooltip_text(Some(model.i18n.text(Text::Today)));
  widgets
    .next_month
    .set_tooltip_text(Some(model.i18n.text(Text::NextMonth)));
  widgets
    .next_year
    .set_tooltip_text(Some(model.i18n.text(Text::NextYear)));
  widgets
    .settings_button
    .set_tooltip_text(Some(model.i18n.text(Text::Settings)));
  widgets.agenda_separator.set_visible(show_events);
  widgets.agenda_header.set_visible(show_events);
  widgets.agenda_scroll.set_visible(show_events);
  if show_events {
    widgets.agenda_scroll.set_min_content_height(74);
    widgets.agenda_scroll.set_max_content_height(128);
    rebuild_agenda(&widgets.agenda, &model.events, model.i18n, move |id| {
      sender.input(AppMsg::OpenEditEvent(id));
    });
    if model
      .read_errors
      .iter()
      .any(|error| matches!(error, Text::DatabaseReadFailed | Text::DailyReadFailed))
      && model.events.is_empty()
    {
      while let Some(child) = widgets.agenda.first_child() {
        widgets.agenda.remove(&child);
      }
    }
    for error in model.read_errors.iter().rev() {
      let label = gtk::Label::new(Some(model.i18n.text(*error)));
      label.set_wrap(true);
      label.set_xalign(0.0);
      widgets.agenda.prepend(&label);
    }
  } else {
    widgets.agenda_scroll.set_min_content_height(0);
    widgets.agenda_scroll.set_max_content_height(0);
    while let Some(child) = widgets.agenda.first_child() {
      widgets.agenda.remove(&child);
    }
  }
  if let ViewState::Editor(draft) = &model.view {
    widgets.stack.set_visible_child_name("editor");
    populate_editor(&widgets.editor, draft, model.i18n);
  } else if let ViewState::Settings(draft) = &model.view {
    widgets.stack.set_visible_child_name("settings");
    populate_settings(&widgets.settings, draft, model.i18n);
  } else {
    widgets.stack.set_visible_child_name("calendar");
  }
}

/// Independent results keep a failed monthly query from hiding valid daily events.
pub(super) struct CalendarRead {
  pub events: Result<Vec<CalendarEvent>, Text>,
  pub event_days: Result<HashSet<NaiveDate>, Text>,
}

fn read_queries<Reader>(
  open: impl FnOnce() -> Result<Reader, Text>,
  daily: impl FnOnce(&Reader) -> Result<Vec<CalendarEvent>, Text>,
  monthly: impl FnOnce(&Reader) -> Result<HashSet<NaiveDate>, Text>,
) -> CalendarRead {
  match open() {
    Ok(db) => CalendarRead {
      events: daily(&db),
      event_days: monthly(&db),
    },
    Err(error) => CalendarRead {
      events: Err(error),
      event_days: Err(error),
    },
  }
}

pub(super) fn load_day_and_month(
  paths: &Paths,
  selected: NaiveDate,
  month: NaiveDate,
) -> CalendarRead {
  read_queries(
    || Database::open(&paths.database).map_err(|_| Text::DatabaseReadFailed),
    |db| {
      // Preserve the existing expiry policy; its maintenance result is not a read result.
      let _ = db.purge_expired_local_single_events(chrono::Utc::now());
      db.events_for_day(&Local, selected)
        .map_err(|_| Text::DailyReadFailed)
    },
    |db| {
      let first = first_of_month(month);
      let last = next_month(month)
        .checked_sub_days(Days::new(1))
        .unwrap_or(first);
      let (start, _) = day_bounds(&Local, first);
      let (_, end) = day_bounds(&Local, last);
      db.events_between(start, end)
        .map(|events| {
          events
            .iter()
            .flat_map(|event| event.local_days(&Local))
            .filter(|day| (first..=last).contains(day))
            .collect()
        })
        .map_err(|_| Text::MonthlyReadFailed)
    },
  )
}

/// Retain last good values only for the same date/month, never under another heading.
pub(super) fn apply_read(
  loaded: CalendarRead,
  events: &mut Vec<CalendarEvent>,
  event_days: &mut HashSet<NaiveDate>,
  same_day: bool,
  same_month: bool,
) -> Vec<Text> {
  let mut errors = Vec::new();
  match loaded.events {
    Ok(value) => *events = value,
    Err(error) => {
      if !same_day {
        events.clear();
      }
      errors.push(error);
    }
  }
  match loaded.event_days {
    Ok(value) => *event_days = value,
    Err(error) => {
      if !same_month {
        event_days.clear();
      }
      if !errors.contains(&error) {
        errors.push(error);
      }
    }
  }
  errors
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn opening_failure_is_distinct_and_skips_queries() {
    let result = read_queries::<()>(
      || Err(Text::DatabaseReadFailed),
      |_| panic!("daily query must not run"),
      |_| panic!("monthly query must not run"),
    );
    assert_eq!(result.events.unwrap_err(), Text::DatabaseReadFailed);
    assert_eq!(result.event_days.unwrap_err(), Text::DatabaseReadFailed);
  }

  #[test]
  fn unavailable_database_reports_open_failure() {
    let result = read_queries(
      || {
        Database::open(std::path::Path::new("/dev/null/calendar.db"))
          .map_err(|_| Text::DatabaseReadFailed)
      },
      |_| panic!("daily query must not run"),
      |_| panic!("monthly query must not run"),
    );
    assert_eq!(result.events.unwrap_err(), Text::DatabaseReadFailed);
  }

  #[test]
  fn query_failures_preserve_the_other_successful_result() {
    let day = NaiveDate::from_ymd_opt(2030, 1, 1).unwrap();
    let result = read_queries(
      || Ok(()),
      |_| Err(Text::DailyReadFailed),
      |_| Ok(HashSet::from([day])),
    );
    assert_eq!(result.events.unwrap_err(), Text::DailyReadFailed);
    assert!(result.event_days.unwrap().contains(&day));
    let event = CalendarEvent::new_local(1, "Synthetic", chrono::Utc::now());
    let result = read_queries(
      || Ok(()),
      |_| Ok(vec![event]),
      |_| Err(Text::MonthlyReadFailed),
    );
    assert_eq!(result.events.unwrap().len(), 1);
    assert_eq!(result.event_days.unwrap_err(), Text::MonthlyReadFailed);
  }

  #[test]
  fn genuine_empty_success_and_retry_clear_errors() {
    let mut events = vec![CalendarEvent::new_local(1, "Synthetic", chrono::Utc::now())];
    let day = NaiveDate::from_ymd_opt(2030, 1, 1).unwrap();
    let mut days = HashSet::from([day]);
    let errors = apply_read(
      CalendarRead {
        events: Err(Text::DailyReadFailed),
        event_days: Err(Text::MonthlyReadFailed),
      },
      &mut events,
      &mut days,
      true,
      true,
    );
    assert_eq!(errors.len(), 2);
    assert_eq!(events.len(), 1);
    assert!(days.contains(&day));
    let loaded = read_queries(|| Ok(()), |_| Ok(Vec::new()), |_| Ok(HashSet::new()));
    assert!(apply_read(loaded, &mut events, &mut days, true, true).is_empty());
    assert!(events.is_empty());
    assert!(days.is_empty());
  }

  #[test]
  fn failure_after_navigation_does_not_show_previous_dates_data() {
    let mut events = vec![CalendarEvent::new_local(1, "Synthetic", chrono::Utc::now())];
    let mut days = HashSet::from([NaiveDate::from_ymd_opt(2030, 1, 1).unwrap()]);
    apply_read(
      CalendarRead {
        events: Err(Text::DatabaseReadFailed),
        event_days: Err(Text::DatabaseReadFailed),
      },
      &mut events,
      &mut days,
      false,
      false,
    );
    assert!(events.is_empty());
    assert!(days.is_empty());
  }

  #[test]
  fn read_errors_are_translated_and_distinct_from_empty() {
    for language in [
      crate::i18n::Language::from_locale_tag("en-US"),
      crate::i18n::Language::from_locale_tag("pt-BR"),
    ] {
      let i18n = I18n::new(language);
      let messages = [
        Text::DatabaseReadFailed,
        Text::DailyReadFailed,
        Text::MonthlyReadFailed,
      ]
      .map(|text| i18n.text(text));
      assert!(
        messages
          .iter()
          .all(|text| !text.is_empty() && *text != i18n.text(Text::NoEvents))
      );
      assert_ne!(messages[0], messages[1]);
      assert_ne!(messages[1], messages[2]);
    }
  }
}
