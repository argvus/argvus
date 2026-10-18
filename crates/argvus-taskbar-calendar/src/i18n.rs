use chrono::{Datelike, NaiveDate};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Language {
  EnUs,
  PtBr,
}

impl Language {
  /// Resolves the configured language. The `auto` value (the default) and
  /// unknown values follow the system locale; explicit values always win.
  pub fn resolve(configured: &str) -> Self {
    match configured {
      "pt-BR" => Self::PtBr,
      "en-US" => Self::EnUs,
      _ => Self::detect(),
    }
  }

  /// Detects the display language from the standard locale environment.
  /// Portuguese systems resolve to `pt-BR`; every other locale (including
  /// missing ones) falls back to English.
  pub fn detect() -> Self {
    Self::detect_with(|variable| std::env::var(variable).ok())
  }

  fn detect_with(get_var: impl Fn(&str) -> Option<String>) -> Self {
    for variable in ["LC_ALL", "LC_MESSAGES", "LANG"] {
      let Some(value) = get_var(variable).map(|value| value.trim().to_string()) else {
        continue;
      };
      if value.is_empty() {
        continue;
      }
      if matches!(value.as_str(), "C" | "POSIX") {
        return Self::EnUs;
      }
      return Self::from_locale_tag(&value);
    }
    Self::EnUs
  }

  /// Maps a locale tag such as `pt_BR.UTF-8` or `pt-BR` onto a supported
  /// language. Anything that is not Portuguese resolves to English.
  pub fn from_locale_tag(tag: &str) -> Self {
    let language = tag
      .split(['.', '@'])
      .next()
      .unwrap_or_default()
      .replace('_', "-")
      .to_lowercase();
    if language.starts_with("pt") {
      Self::PtBr
    } else {
      Self::EnUs
    }
  }
}

#[derive(Debug, Clone, Copy)]
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
  Events,
  Experimental,
  EventsExperimentalTooltip,
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
  Theme,
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
    match self.language {
      Language::EnUs => match text {
        Text::Today => "Today",
        Text::AddEvent => "+ EVENT",
        Text::NewEvent => "NEW EVENT",
        Text::EditEvent => "EDIT EVENT",
        Text::Save => "SAVE",
        Text::Cancel => "CANCEL",
        Text::Title => "TITLE",
        Text::Location => "LOCATION",
        Text::Description => "DESCRIPTION",
        Text::Date => "DATE",
        Text::StartTime => "START TIME",
        Text::EndTime => "END TIME",
        Text::Reminder => "REMINDER",
        Text::Repeat => "REPEAT",
        Text::Before => "BEFORE",
        Text::HourShort => "H",
        Text::MinuteShort => "M",
        Text::AllDay => "ALL DAY",
        Text::NoEvents => "NO EVENTS",
        Text::Events => "Events",
        Text::Experimental => "Experimental",
        Text::EventsExperimentalTooltip => "May contain inconsistencies",
        Text::Delete => "DELETE",
        Text::Settings => "Settings",
        Text::PreviousMonth => "Previous month",
        Text::NextMonth => "Next month",
        Text::PreviousYear => "Previous year",
        Text::NextYear => "Next year",
        Text::Back => "Back",
        Text::PastDate => "Past dates cannot receive new events",
        Text::FontFamily => "FONT FAMILY",
        Text::FontSize => "FONT SIZE",
        Text::Theme => "THEME",
        Text::Language => "LANGUAGE",
        Text::WeekStart => "WEEK STARTS",
        Text::DefaultDuration => "DEFAULT LENGTH (MIN)",
        Text::DefaultReminder => "DEFAULT REMINDER (MIN)",
        Text::SyncEvery => "SYNC EVERY",
        Text::Command => "COMMAND",
        Text::Args => "ARGS",
        Text::EditConfigFile => "Edit config file",
        Text::Terminal => "TERMINAL",
        Text::Editor => "EDITOR",
      },
      Language::PtBr => match text {
        Text::Today => "Hoje",
        Text::AddEvent => "+ EVENTO",
        Text::NewEvent => "NOVO EVENTO",
        Text::EditEvent => "EDITAR EVENTO",
        Text::Save => "SALVAR",
        Text::Cancel => "CANCELAR",
        Text::Title => "TÍTULO",
        Text::Location => "LOCAL",
        Text::Description => "DESCRIÇÃO",
        Text::Date => "DATA",
        Text::StartTime => "HORA INICIAL",
        Text::EndTime => "HORA FINAL",
        Text::Reminder => "LEMBRETE",
        Text::Repeat => "REPETIR",
        Text::Before => "ANTES",
        Text::HourShort => "H",
        Text::MinuteShort => "M",
        Text::AllDay => "DIA INTEIRO",
        Text::NoEvents => "NENHUM EVENTO",
        Text::Events => "Eventos",
        Text::Experimental => "Experimental",
        Text::EventsExperimentalTooltip => "Pode conter inconsistências",
        Text::Delete => "EXCLUIR",
        Text::Settings => "Configurações",
        Text::PreviousMonth => "Mês anterior",
        Text::NextMonth => "Próximo mês",
        Text::PreviousYear => "Ano anterior",
        Text::NextYear => "Próximo ano",
        Text::Back => "Voltar",
        Text::PastDate => "Datas anteriores não aceitam novos eventos",
        Text::FontFamily => "FAMÍLIA DA FONTE",
        Text::FontSize => "TAMANHO DA FONTE",
        Text::Theme => "TEMA",
        Text::Language => "IDIOMA",
        Text::WeekStart => "SEMANA COMEÇA",
        Text::DefaultDuration => "DURAÇÃO PADRÃO (MIN)",
        Text::DefaultReminder => "LEMBRETE PADRÃO (MIN)",
        Text::SyncEvery => "SINCRONIZAR A CADA",
        Text::Command => "COMANDO",
        Text::Args => "ARGUMENTOS",
        Text::EditConfigFile => "Editar arquivo de configuração",
        Text::Terminal => "TERMINAL",
        Text::Editor => "EDITOR",
      },
    }
  }

  pub fn month_title(self, date: NaiveDate) -> String {
    format!("{} {}", self.month_name(date.month()), date.year())
  }

  pub fn agenda_date(self, date: NaiveDate) -> String {
    format!(
      "{} {:02} {}",
      self.weekday_name(date.weekday().num_days_from_monday()),
      date.day(),
      self.month_abbrev(date.month())
    )
  }

  pub fn weekday_headers(self) -> [&'static str; 7] {
    match self.language {
      Language::EnUs => ["MO", "TU", "WE", "TH", "FR", "SA", "SU"],
      Language::PtBr => ["SEG", "TER", "QUA", "QUI", "SEX", "SÁB", "DOM"],
    }
  }

  fn month_name(self, month: u32) -> &'static str {
    match self.language {
      Language::EnUs => [
        "",
        "JANUARY",
        "FEBRUARY",
        "MARCH",
        "APRIL",
        "MAY",
        "JUNE",
        "JULY",
        "AUGUST",
        "SEPTEMBER",
        "OCTOBER",
        "NOVEMBER",
        "DECEMBER",
      ][month as usize],
      Language::PtBr => [
        "",
        "JANEIRO",
        "FEVEREIRO",
        "MARÇO",
        "ABRIL",
        "MAIO",
        "JUNHO",
        "JULHO",
        "AGOSTO",
        "SETEMBRO",
        "OUTUBRO",
        "NOVEMBRO",
        "DEZEMBRO",
      ][month as usize],
    }
  }

  fn month_abbrev(self, month: u32) -> &'static str {
    match self.language {
      Language::EnUs => [
        "", "JAN", "FEB", "MAR", "APR", "MAY", "JUN", "JUL", "AUG", "SEP", "OCT", "NOV", "DEC",
      ][month as usize],
      Language::PtBr => [
        "", "JAN", "FEV", "MAR", "ABR", "MAI", "JUN", "JUL", "AGO", "SET", "OUT", "NOV", "DEZ",
      ][month as usize],
    }
  }

  fn weekday_name(self, monday_index: u32) -> &'static str {
    match self.language {
      Language::EnUs => ["MON", "TUE", "WED", "THU", "FRI", "SAT", "SUN"][monday_index as usize],
      Language::PtBr => ["SEG", "TER", "QUA", "QUI", "SEX", "SÁB", "DOM"][monday_index as usize],
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn english_months_and_weekdays() {
    let i18n = I18n::new(Language::EnUs);
    let date = NaiveDate::from_ymd_opt(2026, 8, 11).unwrap();
    assert_eq!(i18n.month_title(date), "AUGUST 2026");
    assert_eq!(i18n.agenda_date(date), "TUE 11 AUG");
    assert_eq!(i18n.weekday_headers()[0], "MO");
    assert_eq!(i18n.text(Text::StartTime), "START TIME");
    assert_eq!(i18n.text(Text::EndTime), "END TIME");
  }

  #[test]
  fn portuguese_months_and_weekdays() {
    let i18n = I18n::new(Language::PtBr);
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
    assert_eq!(Language::from_locale_tag("pt_BR.UTF-8"), Language::PtBr);
    assert_eq!(Language::from_locale_tag("pt-BR"), Language::PtBr);
    assert_eq!(Language::from_locale_tag("PT"), Language::PtBr);
    assert_eq!(Language::from_locale_tag("en_US.UTF-8"), Language::EnUs);
    assert_eq!(Language::from_locale_tag("es_ES.UTF-8"), Language::EnUs);
    assert_eq!(Language::from_locale_tag(""), Language::EnUs);
  }

  #[test]
  fn auto_detection_follows_locale_precedence() {
    fn env_of<'a>(vars: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
      move |name: &str| {
        vars
          .iter()
          .find(|(key, _)| *key == name)
          .map(|(_, value)| value.to_string())
      }
    }

    // An empty LC_ALL is treated as unset, so LANG decides.
    assert_eq!(
      Language::detect_with(env_of(&[("LC_ALL", ""), ("LANG", "pt_BR.UTF-8")])),
      Language::PtBr
    );
    // LC_MESSAGES outranks LANG.
    assert_eq!(
      Language::detect_with(env_of(&[
        ("LC_MESSAGES", "en_US.UTF-8"),
        ("LANG", "pt_BR.UTF-8")
      ])),
      Language::EnUs
    );
    assert_eq!(
      Language::detect_with(env_of(&[("LC_ALL", "C"), ("LANG", "pt_BR.UTF-8")])),
      Language::EnUs
    );
    assert_eq!(Language::detect_with(env_of(&[])), Language::EnUs);
  }

  #[test]
  fn explicit_language_beats_auto_detection() {
    assert_eq!(Language::resolve("pt-BR"), Language::PtBr);
    assert_eq!(Language::resolve("en-US"), Language::EnUs);
  }
}
