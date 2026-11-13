//! Shared catalog-backed calendar text and date formatting.
use chrono::{Datelike, NaiveDate};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Language(argvus_i18n::Lang);
impl Language {
  pub fn resolve(configured: &str) -> Self {
    if configured != "auto" && argvus_i18n::resolve_installed_locale(configured).is_some() {
      Self::from_locale_tag(configured)
    } else {
      Self::detect()
    }
  }
  pub fn detect() -> Self {
    Self::from_locale_tag(&argvus_i18n::user_locale())
  }
  pub fn from_locale_tag(tag: &str) -> Self {
    let locale = argvus_i18n::resolve_installed_locale(tag).unwrap_or_else(|| "en-US".to_owned());
    Self(argvus_i18n::Lang::for_domain(&locale, "taskbar-calendar"))
  }
  pub fn is_rtl(self) -> bool {
    self.0.direction() == argvus_i18n::TextDirection::RightToLeft
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Text {
  Today,
  AddEvent,
  NewEvent,
  EditEvent,
  Save,
  Cancel,
  Title,
  Location,
  Description,
  Date,
  StartTime,
  EndTime,
  Reminder,
  Repeat,
  Before,
  HourShort,
  MinuteShort,
  AllDay,
  NoEvents,
  DatabaseReadFailed,
  DailyReadFailed,
  MonthlyReadFailed,
  Events,
  UntitledEvent,
  NotifyAt,
  ReminderStartsNow,
  ReminderInProgress,
  ReminderStartedAt,
  ReminderAllDay,
  Delete,
  Settings,
  PreviousMonth,
  NextMonth,
  PreviousYear,
  NextYear,
  Back,
  PastDate,
  FontFamily,
  FontSize,
  Language,
  WeekStart,
  DefaultDuration,
  DefaultReminder,
  SyncEvery,
  Command,
  Args,
  EditConfigFile,
  Terminal,
  Editor,
}

#[derive(Debug, Clone, Copy)]
pub struct I18n {
  language: Language,
}
impl I18n {
  pub fn new(language: Language) -> Self {
    Self { language }
  }
  pub fn text(self, text: Text) -> &'static str {
    let key = match text {
      Text::Today => "label.today",
      Text::AddEvent => "label.add_event",
      Text::NewEvent => "label.new_event",
      Text::EditEvent => "label.edit_event",
      Text::Save => "label.save",
      Text::Cancel => "label.cancel",
      Text::Title => "label.title",
      Text::Location => "label.location",
      Text::Description => "label.description",
      Text::Date => "label.date",
      Text::StartTime => "label.start_time",
      Text::EndTime => "label.end_time",
      Text::Reminder => "label.reminder",
      Text::Repeat => "label.repeat",
      Text::Before => "label.before",
      Text::HourShort => "label.hour_short",
      Text::MinuteShort => "label.minute_short",
      Text::AllDay => "label.all_day",
      Text::NoEvents => "label.no_events",
      Text::DatabaseReadFailed => "label.database_read_failed",
      Text::DailyReadFailed => "label.daily_read_failed",
      Text::MonthlyReadFailed => "label.monthly_read_failed",
      Text::Events => "label.events",
      Text::UntitledEvent => "label.untitled_event",
      Text::NotifyAt => "label.notify_at",
      Text::ReminderStartsNow => "label.reminder_starts_now",
      Text::ReminderInProgress => "label.reminder_in_progress",
      Text::ReminderStartedAt => "label.reminder_started_at",
      Text::ReminderAllDay => "label.reminder_all_day",
      Text::Delete => "label.delete",
      Text::Settings => "label.settings",
      Text::PreviousMonth => "label.previous_month",
      Text::NextMonth => "label.next_month",
      Text::PreviousYear => "label.previous_year",
      Text::NextYear => "label.next_year",
      Text::Back => "label.back",
      Text::PastDate => "label.past_date",
      Text::FontFamily => "label.font_family",
      Text::FontSize => "label.font_size",
      Text::Language => "label.language",
      Text::WeekStart => "label.week_start",
      Text::DefaultDuration => "label.default_duration",
      Text::DefaultReminder => "label.default_reminder",
      Text::SyncEvery => "label.sync_every",
      Text::Command => "label.command",
      Text::Args => "label.args",
      Text::EditConfigFile => "label.edit_config_file",
      Text::Terminal => "label.terminal",
      Text::Editor => "label.editor",
    };
    argvus_i18n::tr(self.language.0, key)
  }
  pub fn reminder_starts_in(self, minutes: i64) -> String {
    self.language.0.tr_plural("reminder.starts_in", minutes)
  }
  pub fn month_title(self, date: NaiveDate) -> String {
    self.language.0.tr_args(
      "date.month_title",
      [
        ("month", self.month_name(date.month()).to_owned()),
        ("year", date.year().to_string()),
      ],
    )
  }
  pub fn agenda_date(self, date: NaiveDate) -> String {
    self.language.0.tr_args(
      "date.agenda",
      [
        (
          "weekday",
          self
            .weekday_name(date.weekday().num_days_from_monday())
            .to_owned(),
        ),
        ("day", format!("{:02}", date.day())),
        ("month", self.month_abbrev(date.month()).to_owned()),
      ],
    )
  }
  pub fn weekday_headers(self) -> [&'static str; 7] {
    std::array::from_fn(|i| argvus_i18n::tr(self.language.0, &format!("weekday.header.{i}")))
  }
  fn month_name(self, month: u32) -> &'static str {
    argvus_i18n::tr(self.language.0, &format!("month.{month}"))
  }
  fn month_abbrev(self, month: u32) -> &'static str {
    argvus_i18n::tr(self.language.0, &format!("month.short.{month}"))
  }
  fn weekday_name(self, index: u32) -> &'static str {
    argvus_i18n::tr(self.language.0, &format!("weekday.{index}"))
  }
}
#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn english_months_and_weekdays() {
    let i18n = I18n::new(Language::from_locale_tag("en-US"));
    let date = NaiveDate::from_ymd_opt(2026, 8, 11).unwrap();
    assert_eq!(i18n.month_title(date), "AUGUST 2026");
    assert_eq!(i18n.agenda_date(date), "TUE 11 AUG");
    assert_eq!(i18n.weekday_headers()[0], "MO");
    assert_eq!(i18n.text(Text::StartTime), "START TIME");
    assert_eq!(i18n.text(Text::EndTime), "END TIME");
  }

  #[test]
  fn portuguese_months_and_weekdays() {
    let i18n = I18n::new(Language::from_locale_tag("pt-BR"));
    let january = NaiveDate::from_ymd_opt(2026, 1, 5).unwrap();
    let august = NaiveDate::from_ymd_opt(2026, 8, 11).unwrap();
    let march = NaiveDate::from_ymd_opt(2026, 3, 7).unwrap();
    assert_eq!(i18n.month_title(january), "JANEIRO 2026");
    assert_eq!(i18n.month_title(august), "AGOSTO 2026");
    assert_eq!(i18n.month_title(march), "MARÇO 2026");
    assert_eq!(i18n.agenda_date(august), "TER 11 AGO");
    assert_eq!(i18n.weekday_headers()[5], "SÁB");
    assert_eq!(i18n.text(Text::StartTime), "HORA INICIAL");
    assert_eq!(i18n.text(Text::EndTime), "HORA FINAL");
  }

  #[test]
  fn unsupported_language_falls_back_to_english() {
    assert_eq!(Language::resolve("xx-YY"), Language::detect());
  }

  #[test]
  fn locale_tags_map_to_supported_languages() {
    assert_eq!(
      Language::from_locale_tag("pt_BR.UTF-8"),
      Language::from_locale_tag("pt-BR")
    );
    assert_eq!(
      Language::from_locale_tag("pt-BR"),
      Language::from_locale_tag("pt-BR")
    );
    assert_eq!(
      Language::from_locale_tag("PT"),
      Language::from_locale_tag("pt-BR")
    );
    assert_eq!(
      Language::from_locale_tag("en_US.UTF-8"),
      Language::from_locale_tag("en-US")
    );
    assert_eq!(
      Language::from_locale_tag("es_ES.UTF-8"),
      Language::from_locale_tag("es-ES")
    );
    assert_eq!(
      Language::from_locale_tag(""),
      Language::from_locale_tag("en-US")
    );
  }

  #[test]
  fn explicit_language_beats_auto_detection() {
    assert_eq!(
      Language::resolve("pt-BR"),
      Language::from_locale_tag("pt-BR")
    );
    assert_eq!(
      Language::resolve("en-US"),
      Language::from_locale_tag("en-US")
    );
  }
}
