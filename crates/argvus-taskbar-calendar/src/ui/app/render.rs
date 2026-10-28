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

pub(super) fn load_day_and_month(
  paths: &Paths,
  selected: NaiveDate,
  month: NaiveDate,
) -> (Vec<CalendarEvent>, HashSet<NaiveDate>) {
  let Ok(db) = Database::open(&paths.database) else {
    return (Vec::new(), HashSet::new());
  };
  let _ = db.purge_expired_local_single_events(chrono::Utc::now());
  let events = db.events_for_day(selected).unwrap_or_default();
  let first = first_of_month(month);
  let last = next_month(month)
    .checked_sub_days(Days::new(1))
    .unwrap_or(first);
  let start = first.and_hms_opt(0, 0, 0).unwrap().and_utc();
  let end = last.and_hms_opt(23, 59, 59).unwrap().and_utc();
  let event_days = db
    .events_between(start, end)
    .unwrap_or_default()
    .into_iter()
    .map(|event| event.start.with_timezone(&Local).date_naive())
    .collect();
  (events, event_days)
}
