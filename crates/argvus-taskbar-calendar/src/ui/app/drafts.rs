//! View-state transitions for the event editor and the settings drafts, and the reminder service toggle.

use super::*;

pub(super) fn cancel_editor(view: &mut ViewState) {
  *view = ViewState::Calendar;
}

pub(super) fn update_draft<F>(view: &mut ViewState, update: F)
where
  F: FnOnce(&mut EditorDraft),
{
  if let ViewState::Editor(draft) = view {
    update(draft.as_mut());
  }
}

pub(super) fn update_settings_draft<F>(view: &mut ViewState, update: F)
where
  F: FnOnce(&mut SettingsDraft),
{
  if let ViewState::Settings(draft) = view {
    update(draft.as_mut());
  }
}

pub(super) fn can_create_event(date: NaiveDate) -> bool {
  date >= today()
}

pub(super) fn control_reminder_service(enabled: bool) {
  if enabled {
    let _ = Command::new("systemctl")
      .args(["--user", "reset-failed", "argvus-taskbar-calendar.service"])
      .status();
  }
  let systemctl_args: &[&str] = if enabled {
    &[
      "--user",
      "enable",
      "--now",
      "argvus-taskbar-calendar.service",
    ]
  } else {
    &[
      "--user",
      "disable",
      "--now",
      "argvus-taskbar-calendar.service",
    ]
  };
  let _ = Command::new("systemctl").args(systemctl_args).status();
  if !enabled {
    // Only this user's reminder service: the uid comes from /proc/self, as
    // this crate does not link libc. Without it nothing is signalled. The
    // dispatcher execs the private executable, so the command line is
    // `/usr/lib/argvus/commands/taskbar-calendar service`.
    if let Ok(metadata) = std::fs::metadata("/proc/self") {
      use std::os::unix::fs::MetadataExt as _;
      let _ = Command::new("pkill")
        .args([
          "-u",
          metadata.uid().to_string().as_str(),
          "-f",
          "taskbar-calendar service",
        ])
        .status();
    }
  }
}

/// Copies one edited field of the event editor into the draft.
pub(super) fn apply_editor_field(view: &mut ViewState, field: AppMsg) {
  match field {
    AppMsg::EditorTitle(value) => update_draft(view, |draft| draft.title = value),
    AppMsg::EditorLocation(value) => update_draft(view, |draft| draft.location = value),
    AppMsg::EditorDescription(value) => update_draft(view, |draft| draft.description = value),
    AppMsg::EditorStartHour(value) => update_draft(view, |draft| {
      draft.set_start_time(value, draft.start_minute)
    }),
    AppMsg::EditorStartMinute(value) => {
      update_draft(view, |draft| draft.set_start_time(draft.start_hour, value))
    }
    AppMsg::EditorEndHour(value) => update_draft(view, |draft| draft.end_hour = value),
    AppMsg::EditorEndMinute(value) => update_draft(view, |draft| draft.end_minute = value),
    AppMsg::EditorAllDay(value) => update_draft(view, |draft| draft.all_day = value),
    AppMsg::EditorReminderEnabled(value) => {
      update_draft(view, |draft| draft.reminder_enabled = value)
    }
    AppMsg::EditorReminderHours(value) => update_draft(view, |draft| draft.reminder_hours = value),
    AppMsg::EditorReminderMinutes(value) => {
      update_draft(view, |draft| draft.reminder_minutes = value)
    }
    AppMsg::EditorRepeatAllDay(value) => update_draft(view, |draft| draft.repeat_all_day = value),
    AppMsg::EditorRepeatHours(value) => update_draft(view, |draft| draft.repeat_hours = value),
    AppMsg::EditorRepeatMinutes(value) => update_draft(view, |draft| draft.repeat_minutes = value),
    // `update` routes only the editor field messages here.
    _ => {}
  }
}

/// Copies one edited field of the settings form into the draft.
pub(super) fn apply_settings_field(view: &mut ViewState, field: AppMsg) {
  match field {
    AppMsg::SettingsFontFamily(value) => {
      update_settings_draft(view, |draft| draft.font_family = value)
    }
    AppMsg::SettingsFontSize(value) => update_settings_draft(view, |draft| draft.font_size = value),
    AppMsg::SettingsLanguage(value) => update_settings_draft(view, |draft| draft.language = value),
    AppMsg::SettingsWeekStart(value) => {
      update_settings_draft(view, |draft| draft.week_start = value)
    }
    AppMsg::SettingsShowEvents(value) => {
      update_settings_draft(view, |draft| draft.show_events = value)
    }
    AppMsg::SettingsDefaultDuration(value) => {
      update_settings_draft(view, |draft| draft.default_event_duration_minutes = value)
    }
    AppMsg::SettingsDefaultReminder(value) => {
      update_settings_draft(view, |draft| draft.default_reminder_minutes = value)
    }
    AppMsg::SettingsSyncInterval(value) => {
      update_settings_draft(view, |draft| draft.sync_interval_minutes = value)
    }
    AppMsg::SettingsEditorCommand(value) => {
      update_settings_draft(view, |draft| draft.editor_command = value)
    }
    AppMsg::SettingsEditorArgs(value) => {
      update_settings_draft(view, |draft| draft.editor_args = value)
    }
    AppMsg::SettingsTerminalCommand(value) => {
      update_settings_draft(view, |draft| draft.terminal_command = value)
    }
    AppMsg::SettingsTerminalArgs(value) => {
      update_settings_draft(view, |draft| draft.terminal_args = value)
    }
    // `update` routes only the settings field messages here.
    _ => {}
  }
}
