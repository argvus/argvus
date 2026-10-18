use gtk::glib::SignalHandlerId;
use gtk::prelude::*;

use crate::config::Settings;
use crate::i18n::{I18n, Text};

#[derive(Debug, Clone)]
pub struct SettingsDraft {
  pub font_family: String,
  pub font_size: u32,
  pub theme: String,
  pub language: String,
  pub week_start: String,
  pub show_events: bool,
  pub default_event_duration_minutes: i64,
  pub default_reminder_minutes: i64,
  pub sync_interval_minutes: u64,
  pub editor_command: String,
  pub editor_args: String,
  pub terminal_command: String,
  pub terminal_args: String,
}

impl SettingsDraft {
  pub fn from_config(settings: &Settings, theme: &str, events_enabled: bool) -> Self {
    Self {
      font_family: settings.appearance.font_family.clone(),
      font_size: u32::from(settings.appearance.font_size),
      theme: theme.to_string(),
      language: settings.locale.language.clone(),
      week_start: settings.calendar.week_start.clone(),
      show_events: events_enabled,
      default_event_duration_minutes: settings.calendar.default_event_duration_minutes,
      default_reminder_minutes: settings.calendar.default_reminder_minutes,
      sync_interval_minutes: settings.calendar.sync_interval_minutes,
      editor_command: settings.editor.command.clone(),
      editor_args: settings.editor.args.join(" "),
      terminal_command: settings.terminal.command.clone(),
      terminal_args: settings.terminal.args.join(" "),
    }
  }

  pub fn into_config(self) -> Settings {
    let mut settings = Settings::default();
    settings.appearance.font_family = if self.font_family.trim().is_empty() {
      "monospace".to_string()
    } else {
      self.font_family
    };
    settings.appearance.font_size = self.font_size.clamp(8, 32) as u8;
    settings.locale.language = match self.language.as_str() {
      "pt-BR" => "pt-BR",
      "en-US" => "en-US",
      // Unknown values follow the system locale.
      _ => "auto",
    }
    .to_string();
    settings.calendar.week_start = if self.week_start == "sunday" {
      "sunday"
    } else {
      "monday"
    }
    .to_string();
    settings.calendar.show_events = self.show_events;
    settings.calendar.default_event_duration_minutes = self.default_event_duration_minutes.max(0);
    settings.calendar.default_reminder_minutes = self.default_reminder_minutes.max(0);
    settings.calendar.sync_interval_minutes = self.sync_interval_minutes.max(1);
    settings.editor.command = self.editor_command;
    settings.editor.args = self
      .editor_args
      .split_whitespace()
      .map(str::to_string)
      .collect();
    settings.terminal.command = self.terminal_command;
    settings.terminal.args = self
      .terminal_args
      .split_whitespace()
      .map(str::to_string)
      .collect();
    settings
  }
}

pub struct SettingsWidgets {
  pub header: gtk::Label,
  pub font_family_label: gtk::Label,
  pub font_family: gtk::Entry,
  pub font_size_label: gtk::Label,
  pub font_size: gtk::SpinButton,
  pub theme_label: gtk::Label,
  pub theme: gtk::DropDown,
  pub theme_model: gtk::StringList,
  pub language_label: gtk::Label,
  pub language: gtk::DropDown,
  pub language_model: gtk::StringList,
  pub week_start_label: gtk::Label,
  pub week_start: gtk::DropDown,
  pub week_start_model: gtk::StringList,
  pub show_events_label: gtk::Label,
  pub show_events_experimental: gtk::Label,
  pub show_events: gtk::Switch,
  pub default_duration_label: gtk::Label,
  pub default_duration: gtk::SpinButton,
  pub default_reminder_label: gtk::Label,
  pub default_reminder: gtk::SpinButton,
  pub sync_interval_label: gtk::Label,
  pub sync_interval: gtk::SpinButton,
  pub editor_command_label: gtk::Label,
  pub editor_command: gtk::Entry,
  pub editor_args_label: gtk::Label,
  pub editor_args: gtk::Entry,
  pub terminal_command_label: gtk::Label,
  pub terminal_command: gtk::Entry,
  pub terminal_args_label: gtk::Label,
  pub terminal_args: gtk::Entry,
  pub edit_file: gtk::Button,
  pub cancel: gtk::Button,
  pub save: gtk::Button,
  pub changed_handlers: SettingsChangedHandlers,
}

pub struct SettingsChangedHandlers {
  pub font_family: SignalHandlerId,
  pub font_size: SignalHandlerId,
  pub theme: SignalHandlerId,
  pub language: SignalHandlerId,
  pub week_start: SignalHandlerId,
  pub show_events: SignalHandlerId,
  pub default_duration: SignalHandlerId,
  pub default_reminder: SignalHandlerId,
  pub sync_interval: SignalHandlerId,
  pub editor_command: SignalHandlerId,
  pub editor_args: SignalHandlerId,
  pub terminal_command: SignalHandlerId,
  pub terminal_args: SignalHandlerId,
}

pub fn build_settings() -> gtk::Box {
  gtk::Box::new(gtk::Orientation::Vertical, 10)
}

pub fn set_dropdown(dropdown: &gtk::DropDown, model: &gtk::StringList, value: &str) {
  let index = (0..model.n_items())
    .find(|index| model.string(*index).is_some_and(|item| item == value))
    .unwrap_or(0);
  dropdown.set_selected(index);
}

pub fn dropdown_value(dropdown: &gtk::DropDown) -> String {
  dropdown
    .selected_item()
    .and_then(|item| item.downcast::<gtk::StringObject>().ok())
    .map_or_else(String::new, |item| item.string().to_string())
}

pub fn populate_settings(widgets: &SettingsWidgets, draft: &SettingsDraft, i18n: I18n) {
  let handlers = &widgets.changed_handlers;
  for (widget, id) in [
    (
      widgets.font_family.upcast_ref::<gtk::Widget>(),
      &handlers.font_family,
    ),
    (
      widgets.font_size.upcast_ref::<gtk::Widget>(),
      &handlers.font_size,
    ),
    (widgets.theme.upcast_ref::<gtk::Widget>(), &handlers.theme),
    (
      widgets.language.upcast_ref::<gtk::Widget>(),
      &handlers.language,
    ),
    (
      widgets.week_start.upcast_ref::<gtk::Widget>(),
      &handlers.week_start,
    ),
    (
      widgets.show_events.upcast_ref::<gtk::Widget>(),
      &handlers.show_events,
    ),
    (
      widgets.default_duration.upcast_ref::<gtk::Widget>(),
      &handlers.default_duration,
    ),
    (
      widgets.default_reminder.upcast_ref::<gtk::Widget>(),
      &handlers.default_reminder,
    ),
    (
      widgets.sync_interval.upcast_ref::<gtk::Widget>(),
      &handlers.sync_interval,
    ),
    (
      widgets.editor_command.upcast_ref::<gtk::Widget>(),
      &handlers.editor_command,
    ),
    (
      widgets.editor_args.upcast_ref::<gtk::Widget>(),
      &handlers.editor_args,
    ),
    (
      widgets.terminal_command.upcast_ref::<gtk::Widget>(),
      &handlers.terminal_command,
    ),
    (
      widgets.terminal_args.upcast_ref::<gtk::Widget>(),
      &handlers.terminal_args,
    ),
  ] {
    widget.block_signal(id);
  }

  widgets.header.set_label(i18n.text(Text::Settings));
  widgets
    .font_family_label
    .set_label(i18n.text(Text::FontFamily));
  widgets.font_family.set_text(&draft.font_family);
  widgets.font_size_label.set_label(i18n.text(Text::FontSize));
  widgets.font_size.set_value(f64::from(draft.font_size));
  widgets.theme_label.set_label(i18n.text(Text::Theme));
  set_dropdown(&widgets.theme, &widgets.theme_model, &draft.theme);
  widgets.language_label.set_label(i18n.text(Text::Language));
  set_dropdown(&widgets.language, &widgets.language_model, &draft.language);
  widgets
    .week_start_label
    .set_label(i18n.text(Text::WeekStart));
  set_dropdown(
    &widgets.week_start,
    &widgets.week_start_model,
    &draft.week_start,
  );
  widgets.show_events_label.set_label(i18n.text(Text::Events));
  widgets
    .show_events_experimental
    .set_label(i18n.text(Text::Experimental));
  widgets
    .show_events_experimental
    .set_tooltip_text(Some(i18n.text(Text::EventsExperimentalTooltip)));
  widgets
    .show_events
    .set_tooltip_text(Some(i18n.text(Text::EventsExperimentalTooltip)));
  widgets.show_events.set_active(draft.show_events);
  widgets
    .default_duration_label
    .set_label(i18n.text(Text::DefaultDuration));
  widgets
    .default_duration
    .set_value(draft.default_event_duration_minutes as f64);
  widgets
    .default_reminder_label
    .set_label(i18n.text(Text::DefaultReminder));
  widgets
    .default_reminder
    .set_value(draft.default_reminder_minutes as f64);
  widgets
    .sync_interval_label
    .set_label(i18n.text(Text::SyncEvery));
  widgets
    .sync_interval
    .set_value(draft.sync_interval_minutes as f64);
  widgets
    .editor_command_label
    .set_label(i18n.text(Text::Command));
  widgets.editor_command.set_text(&draft.editor_command);
  widgets.editor_args_label.set_label(i18n.text(Text::Args));
  widgets.editor_args.set_text(&draft.editor_args);
  widgets
    .terminal_command_label
    .set_label(i18n.text(Text::Command));
  widgets.terminal_command.set_text(&draft.terminal_command);
  widgets.terminal_args_label.set_label(i18n.text(Text::Args));
  widgets.terminal_args.set_text(&draft.terminal_args);
  widgets.edit_file.set_label(i18n.text(Text::EditConfigFile));
  widgets.cancel.set_label(i18n.text(Text::Cancel));
  widgets.save.set_label(i18n.text(Text::Save));

  for (widget, id) in [
    (
      widgets.font_family.upcast_ref::<gtk::Widget>(),
      &handlers.font_family,
    ),
    (
      widgets.font_size.upcast_ref::<gtk::Widget>(),
      &handlers.font_size,
    ),
    (widgets.theme.upcast_ref::<gtk::Widget>(), &handlers.theme),
    (
      widgets.language.upcast_ref::<gtk::Widget>(),
      &handlers.language,
    ),
    (
      widgets.week_start.upcast_ref::<gtk::Widget>(),
      &handlers.week_start,
    ),
    (
      widgets.show_events.upcast_ref::<gtk::Widget>(),
      &handlers.show_events,
    ),
    (
      widgets.default_duration.upcast_ref::<gtk::Widget>(),
      &handlers.default_duration,
    ),
    (
      widgets.default_reminder.upcast_ref::<gtk::Widget>(),
      &handlers.default_reminder,
    ),
    (
      widgets.sync_interval.upcast_ref::<gtk::Widget>(),
      &handlers.sync_interval,
    ),
    (
      widgets.editor_command.upcast_ref::<gtk::Widget>(),
      &handlers.editor_command,
    ),
    (
      widgets.editor_args.upcast_ref::<gtk::Widget>(),
      &handlers.editor_args,
    ),
    (
      widgets.terminal_command.upcast_ref::<gtk::Widget>(),
      &handlers.terminal_command,
    ),
    (
      widgets.terminal_args.upcast_ref::<gtk::Widget>(),
      &handlers.terminal_args,
    ),
  ] {
    widget.unblock_signal(id);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn draft_round_trips_all_fields() {
    let mut config = Settings::default();
    config.appearance.font_family = "Noto Sans".to_string();
    config.appearance.font_size = 14;
    config.locale.language = "auto".to_string();
    config.calendar.week_start = "sunday".to_string();
    config.calendar.default_event_duration_minutes = 90;
    config.calendar.default_reminder_minutes = 30;
    config.calendar.sync_interval_minutes = 5;
    config.editor.command = "kitty".to_string();
    config.editor.args = vec!["-e".to_string(), "nvim".to_string()];
    config.terminal.command = "kitty".to_string();
    config.terminal.args = vec!["-f".to_string()];

    let draft = SettingsDraft::from_config(&config, "slate-dark", true);
    let restored = draft.into_config();

    assert_eq!(restored.appearance.font_family, "Noto Sans");
    assert_eq!(restored.appearance.font_size, 14);
    assert_eq!(restored.locale.language, "auto");
    assert_eq!(restored.calendar.week_start, "sunday");
    assert!(restored.calendar.show_events);
    assert_eq!(restored.calendar.default_event_duration_minutes, 90);
    assert_eq!(restored.calendar.default_reminder_minutes, 30);
    assert_eq!(restored.calendar.sync_interval_minutes, 5);
    assert_eq!(restored.editor.command, "kitty");
    assert_eq!(restored.editor.args, vec!["-e", "nvim"]);
    assert_eq!(restored.terminal.command, "kitty");
    assert_eq!(restored.terminal.args, vec!["-f"]);
  }

  #[test]
  fn draft_keeps_explicit_language_override() {
    let mut config = Settings::default();
    config.locale.language = "pt-BR".to_string();
    let restored = SettingsDraft::from_config(&config, "", true).into_config();
    assert_eq!(restored.locale.language, "pt-BR");

    config.locale.language = "en-US".to_string();
    let restored = SettingsDraft::from_config(&config, "", true).into_config();
    assert_eq!(restored.locale.language, "en-US");
  }

  #[test]
  fn draft_clamps_invalid_values() {
    let mut draft = SettingsDraft::from_config(&Settings::default(), "", true);
    draft.font_size = 300;
    draft.font_family = "".to_string();
    draft.language = "xx".to_string();
    draft.week_start = "friday".to_string();
    draft.editor_args = "  --flag  'quoted value'  ".to_string();
    draft.sync_interval_minutes = 0;
    draft.default_reminder_minutes = -3;

    let config = draft.into_config();
    assert_eq!(config.appearance.font_size, 32);
    assert_ne!(config.appearance.font_family, "");
    assert_eq!(config.locale.language, "auto");
    assert_eq!(config.calendar.week_start, "monday");
    assert_eq!(config.editor.args, vec!["--flag", "'quoted", "value'"]);
    assert_eq!(config.calendar.sync_interval_minutes, 1);
    assert_eq!(config.calendar.default_reminder_minutes, 0);
  }
}
