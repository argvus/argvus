//! Builds the popup widget tree and connects its signals to `AppMsg`.
//!
//! Each builder creates one area of the popup (calendar, event editor,
//! settings) and connects the signals of its own widgets, so `init` only
//! assembles the areas.

use super::*;

/// Widgets of the calendar page that the model updates after construction.
pub(super) struct CalendarView {
  pub(super) root: gtk::Box,
  pub(super) title: gtk::Label,
  pub(super) prev_year: gtk::Button,
  pub(super) prev_month: gtk::Button,
  pub(super) today_button: gtk::Button,
  pub(super) next_month: gtk::Button,
  pub(super) next_year: gtk::Button,
  pub(super) settings_button: gtk::Button,
  pub(super) day_buttons: Vec<gtk::Button>,
  pub(super) agenda_separator: gtk::Separator,
  pub(super) agenda_header: gtk::Box,
  pub(super) agenda_title: gtk::Label,
  pub(super) add_button: gtk::Button,
  pub(super) agenda: gtk::ListBox,
  pub(super) agenda_scroll: gtk::ScrolledWindow,
}

/// Re-reads the theme files every 700 ms and asks for a reload when their
/// fingerprint changes, so a theme switch restyles an open popup.
pub(super) fn watch_theme(
  paths: Paths,
  theme_watcher: Rc<Cell<ThemeFingerprint>>,
  sender: ComponentSender<AppModel>,
) {
  gtk::glib::timeout_add_local(Duration::from_millis(700), move || {
    let current = theme_fingerprint(&paths);
    if current != theme_watcher.get() {
      theme_watcher.set(current);
      sender.input(AppMsg::ReloadTheme);
    }
    gtk::glib::ControlFlow::Continue
  });
}

/// Forwards the commands of later `argvus-taskbar-calendar` launches (sent
/// over the IPC socket) to this instance.
pub(super) fn serve_ipc(paths: &Paths, sender: &ComponentSender<AppModel>) {
  if let Some(listener) = crate::ipc::listen(paths) {
    let sender = sender.clone();
    crate::ipc::serve(listener, move |message| {
      let msg = match message.command {
        IpcCommand::Toggle => AppMsg::Toggle(message.position),
        IpcCommand::Show => AppMsg::Show(message.position),
        IpcCommand::Hide => AppMsg::Hide,
        IpcCommand::ReloadTheme => AppMsg::ReloadTheme,
        IpcCommand::ReloadConfig => AppMsg::ReloadConfig,
      };
      sender.input(msg);
    });
  }
}

pub(super) fn build_calendar_view(
  i18n: &I18n,
  settings: &Settings,
  sender: &ComponentSender<AppModel>,
) -> CalendarView {
  let calendar_root = gtk::Box::new(gtk::Orientation::Vertical, 10);
  calendar_root.add_css_class("calendar-view");
  calendar_root.set_vexpand(false);

  let header = gtk::Box::new(gtk::Orientation::Horizontal, 4);
  header.add_css_class("calendar-header");
  let prev_year = gtk::Button::with_label("‹‹");
  prev_year.add_css_class("nav-button");
  prev_year.set_tooltip_text(Some(i18n.text(Text::PreviousYear)));
  let prev = gtk::Button::with_label("‹");
  prev.add_css_class("nav-button");
  prev.set_tooltip_text(Some(i18n.text(Text::PreviousMonth)));
  let title = gtk::Label::new(None);
  title.set_xalign(0.0);
  title.set_hexpand(true);
  title.add_css_class("calendar-title");
  let today_button = gtk::Button::with_label("●");
  today_button.add_css_class("nav-button");
  today_button.add_css_class("today-button");
  let next = gtk::Button::with_label("›");
  next.add_css_class("nav-button");
  next.set_tooltip_text(Some(i18n.text(Text::NextMonth)));
  let next_year = gtk::Button::with_label("››");
  next_year.add_css_class("nav-button");
  next_year.set_tooltip_text(Some(i18n.text(Text::NextYear)));
  let settings_button = gtk::Button::from_icon_name("preferences-system-symbolic");
  settings_button.add_css_class("nav-button");
  settings_button.add_css_class("settings-button");
  settings_button.set_tooltip_text(Some(i18n.text(Text::Settings)));
  today_button.set_tooltip_text(Some(i18n.text(Text::Today)));
  header.append(&prev_year);
  header.append(&prev);
  header.append(&title);
  header.append(&today_button);
  header.append(&next);
  header.append(&next_year);
  header.append(&settings_button);
  calendar_root.append(&header);

  let grid = gtk::Grid::builder()
    .row_spacing(2)
    .column_spacing(2)
    .column_homogeneous(true)
    .build();
  grid.add_css_class("calendar-grid");
  for (column, label) in weekday_labels(i18n.weekday_headers(), settings.week_start())
    .into_iter()
    .enumerate()
  {
    let weekday = gtk::Label::new(Some(label));
    weekday.add_css_class("weekday");
    weekday.set_xalign(0.5);
    grid.attach(&weekday, column as i32, 0, 1, 1);
  }
  // Six rows of seven days cover every month layout; `render_model` stores
  // the date of each cell on its button.
  let mut day_buttons = Vec::new();
  for index in 0..42 {
    let button = gtk::Button::new();
    button.add_css_class("day");
    let sender = sender.clone();
    button.connect_clicked(move |button| {
      if let Some(date) = unsafe { button.data::<NaiveDate>("date") } {
        sender.input(AppMsg::SelectDay(unsafe { *date.as_ref() }));
      }
    });
    grid.attach(&button, index % 7, index / 7 + 1, 1, 1);
    day_buttons.push(button);
  }
  calendar_root.append(&grid);

  let separator = gtk::Separator::new(gtk::Orientation::Horizontal);
  separator.add_css_class("agenda-separator");
  calendar_root.append(&separator);

  let agenda_header = gtk::Box::new(gtk::Orientation::Horizontal, 6);
  agenda_header.add_css_class("agenda-header");
  let agenda_title = gtk::Label::new(None);
  agenda_title.add_css_class("agenda-title");
  agenda_title.set_xalign(0.0);
  agenda_title.set_hexpand(true);
  let add = gtk::Button::with_label(i18n.text(Text::AddEvent));
  add.add_css_class("add-event");
  add.set_tooltip_text(Some(i18n.text(Text::AddEvent)));
  agenda_header.append(&agenda_title);
  agenda_header.append(&add);
  calendar_root.append(&agenda_header);

  let agenda = gtk::ListBox::new();
  agenda.add_css_class("agenda");
  agenda.set_selection_mode(gtk::SelectionMode::None);
  let agenda_scroll = gtk::ScrolledWindow::builder()
    .hscrollbar_policy(gtk::PolicyType::Never)
    .vscrollbar_policy(gtk::PolicyType::Automatic)
    .min_content_height(74)
    .max_content_height(128)
    .child(&agenda)
    .build();
  agenda_scroll.add_css_class("agenda-scroll");
  calendar_root.append(&agenda_scroll);

  connect_click(&prev_year, sender, || AppMsg::PreviousYear);
  connect_click(&prev, sender, || AppMsg::PreviousMonth);
  connect_click(&next, sender, || AppMsg::NextMonth);
  connect_click(&next_year, sender, || AppMsg::NextYear);
  connect_click(&today_button, sender, || AppMsg::Today);
  connect_click(&add, sender, || AppMsg::OpenNewEvent);
  connect_click(&settings_button, sender, || AppMsg::OpenSettings);

  CalendarView {
    root: calendar_root,
    title,
    prev_year,
    prev_month: prev,
    today_button,
    next_month: next,
    next_year,
    settings_button,
    day_buttons,
    agenda_separator: separator,
    agenda_header,
    agenda_title,
    add_button: add,
    agenda,
    agenda_scroll,
  }
}

pub(super) fn build_editor_view(
  i18n: &I18n,
  sender: &ComponentSender<AppModel>,
) -> (gtk::Box, EditorWidgets) {
  let editor_root = build_editor();
  editor_root.add_css_class("event-editor");
  let editor_header = gtk::Box::new(gtk::Orientation::Horizontal, 6);
  editor_header.add_css_class("editor-header");
  let back = gtk::Button::with_label("‹");
  back.add_css_class("nav-button");
  back.set_tooltip_text(Some(i18n.text(Text::Back)));
  let editor_title = gtk::Label::new(None);
  editor_title.add_css_class("editor-title");
  editor_title.set_xalign(0.0);
  editor_title.set_hexpand(true);
  editor_header.append(&back);
  editor_header.append(&editor_title);
  editor_root.append(&editor_header);

  let title_label = editor_label();
  let title_entry = editor_entry();
  editor_root.append(&title_label);
  editor_root.append(&title_entry);

  let date_row = editor_row();
  let date_label = editor_label();
  let date_value = gtk::Label::new(None);
  date_value.add_css_class("editor-value");
  date_value.set_xalign(0.0);
  date_row.append(&date_label);
  date_row.append(&date_value);
  editor_root.append(&date_row);

  let start_time_row = editor_row();
  let start_time_label = editor_label();
  start_time_label.set_width_chars(12);
  let start_hour = editor_number_spin(23);
  let start_separator = gtk::Label::new(Some(":"));
  start_separator.add_css_class("time-separator");
  let start_minute = editor_number_spin(59);
  start_time_row.append(&start_time_label);
  start_time_row.append(&start_hour);
  start_time_row.append(&start_separator);
  start_time_row.append(&start_minute);
  editor_root.append(&start_time_row);

  let end_time_row = editor_row();
  let end_time_label = editor_label();
  end_time_label.set_width_chars(12);
  let end_hour = editor_number_spin(23);
  let end_separator = gtk::Label::new(Some(":"));
  end_separator.add_css_class("time-separator");
  let end_minute = editor_number_spin(59);
  end_time_row.append(&end_time_label);
  end_time_row.append(&end_hour);
  end_time_row.append(&end_separator);
  end_time_row.append(&end_minute);
  editor_root.append(&end_time_row);

  let all_day = gtk::CheckButton::new();
  all_day.add_css_class("editor-check");
  editor_root.append(&all_day);

  let location_label = editor_label();
  let location_entry = editor_entry();
  editor_root.append(&location_label);
  editor_root.append(&location_entry);

  let description_label = editor_label();
  let description_entry = editor_entry();
  editor_root.append(&description_label);
  editor_root.append(&description_entry);

  let reminder_row = editor_row();
  let reminder_enabled = gtk::CheckButton::new();
  reminder_enabled.add_css_class("editor-check");
  reminder_enabled.set_width_request(102);
  let reminder_hour_label = gtk::Label::new(None);
  reminder_hour_label.add_css_class("editor-value");
  let reminder_hours = editor_number_spin(23);
  let reminder_minute_label = gtk::Label::new(None);
  reminder_minute_label.add_css_class("editor-value");
  let reminder_minutes = editor_number_spin(59);
  let reminder_before = gtk::Label::new(None);
  reminder_before.add_css_class("editor-value");
  reminder_row.append(&reminder_enabled);
  reminder_row.append(&reminder_hour_label);
  reminder_row.append(&reminder_hours);
  reminder_row.append(&reminder_minute_label);
  reminder_row.append(&reminder_minutes);
  reminder_row.append(&reminder_before);
  editor_root.append(&reminder_row);

  let repeat_row = editor_row();
  let repeat_all_day = gtk::CheckButton::new();
  repeat_all_day.add_css_class("editor-check");
  repeat_all_day.set_width_request(102);
  let repeat_hour_label = gtk::Label::new(None);
  repeat_hour_label.add_css_class("editor-value");
  let repeat_hours = editor_number_spin(23);
  let repeat_minute_label = gtk::Label::new(None);
  repeat_minute_label.add_css_class("editor-value");
  let repeat_minutes = editor_number_spin(59);
  repeat_row.append(&repeat_all_day);
  repeat_row.append(&repeat_hour_label);
  repeat_row.append(&repeat_hours);
  repeat_row.append(&repeat_minute_label);
  repeat_row.append(&repeat_minutes);
  editor_root.append(&repeat_row);

  let editor_actions = gtk::Box::new(gtk::Orientation::Horizontal, 8);
  editor_actions.add_css_class("editor-actions");
  let delete = gtk::Button::new();
  delete.add_css_class("editor-action");
  delete.add_css_class("editor-delete");
  let spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  spacer.set_hexpand(true);
  let cancel = gtk::Button::new();
  cancel.add_css_class("editor-action");
  let save = gtk::Button::new();
  save.add_css_class("editor-action");
  save.add_css_class("editor-save");
  editor_actions.append(&delete);
  editor_actions.append(&spacer);
  editor_actions.append(&cancel);
  editor_actions.append(&save);
  editor_root.append(&editor_actions);

  connect_click(&back, sender, || AppMsg::CancelEditor);
  connect_click(&cancel, sender, || AppMsg::CancelEditor);
  connect_click(&save, sender, || AppMsg::SaveEditor);
  connect_click(&delete, sender, || AppMsg::DeleteEvent);

  // The handler ids are kept so `populate_editor` can block them while it
  // fills the fields, which would otherwise echo back as edits.
  let changed_handlers = EditorChangedHandlers {
    title: connect_entry(&title_entry, sender, AppMsg::EditorTitle),
    location: connect_entry(&location_entry, sender, AppMsg::EditorLocation),
    description: connect_entry(&description_entry, sender, AppMsg::EditorDescription),
    start_hour: connect_spin(&start_hour, sender, |value| {
      AppMsg::EditorStartHour(value as u32)
    }),
    start_minute: connect_spin(&start_minute, sender, |value| {
      AppMsg::EditorStartMinute(value as u32)
    }),
    end_hour: connect_spin(&end_hour, sender, |value| {
      AppMsg::EditorEndHour(value as u32)
    }),
    end_minute: connect_spin(&end_minute, sender, |value| {
      AppMsg::EditorEndMinute(value as u32)
    }),
    all_day: connect_check(&all_day, sender, AppMsg::EditorAllDay),
    reminder_enabled: connect_check(&reminder_enabled, sender, AppMsg::EditorReminderEnabled),
    reminder_hours: connect_spin(&reminder_hours, sender, |value| {
      AppMsg::EditorReminderHours(value as u32)
    }),
    reminder_minutes: connect_spin(&reminder_minutes, sender, |value| {
      AppMsg::EditorReminderMinutes(value as u32)
    }),
    repeat_all_day: connect_check(&repeat_all_day, sender, AppMsg::EditorRepeatAllDay),
    repeat_hours: connect_spin(&repeat_hours, sender, |value| {
      AppMsg::EditorRepeatHours(value as u32)
    }),
    repeat_minutes: connect_spin(&repeat_minutes, sender, |value| {
      AppMsg::EditorRepeatMinutes(value as u32)
    }),
  };

  let widgets = EditorWidgets {
    header: editor_title,
    title_label,
    title: title_entry,
    date_label,
    date_value,
    start_time_label,
    end_time_label,
    start_hour,
    start_minute,
    end_hour,
    end_minute,
    all_day,
    location_label,
    location: location_entry,
    description_label,
    description: description_entry,
    reminder_enabled,
    reminder_hour_label,
    reminder_hours,
    reminder_minute_label,
    reminder_minutes,
    reminder_before,
    repeat_row,
    repeat_all_day,
    repeat_hour_label,
    repeat_hours,
    repeat_minute_label,
    repeat_minutes,
    delete,
    cancel,
    save,
    changed_handlers,
  };
  (editor_root, widgets)
}

pub(super) fn build_settings_view(
  i18n: &I18n,
  sender: &ComponentSender<AppModel>,
) -> (gtk::ScrolledWindow, SettingsWidgets) {
  let settings_root = build_settings();
  settings_root.add_css_class("event-editor");
  settings_root.add_css_class("settings-view");
  let settings_header = gtk::Box::new(gtk::Orientation::Horizontal, 6);
  settings_header.add_css_class("editor-header");
  let settings_back = gtk::Button::with_label("‹");
  settings_back.add_css_class("nav-button");
  settings_back.set_tooltip_text(Some(i18n.text(Text::Back)));
  let settings_title = gtk::Label::new(None);
  settings_title.add_css_class("editor-title");
  settings_title.set_xalign(0.0);
  settings_title.set_hexpand(true);
  settings_header.append(&settings_back);
  settings_header.append(&settings_title);
  settings_root.append(&settings_header);

  let font_family_label = settings_label();
  let font_family = editor_entry();
  settings_root.append(&font_family_label);
  settings_root.append(&font_family);

  let font_size_label = settings_label();
  let font_size = settings_number_spin(32, 2);
  settings_root.append(&font_size_label);
  settings_root.append(&font_size);

  let language_label = settings_label();
  let language_model = gtk::StringList::new(&["auto", "en-US", "pt-BR"]);
  let language = gtk::DropDown::new(Some(language_model.clone()), None::<gtk::Expression>);
  language.add_css_class("editor-dropdown");
  settings_root.append(&language_label);
  settings_root.append(&language);

  let week_start_label = settings_label();
  let week_start_model = gtk::StringList::new(&["monday", "sunday"]);
  let week_start = gtk::DropDown::new(Some(week_start_model.clone()), None::<gtk::Expression>);
  week_start.add_css_class("editor-dropdown");
  settings_root.append(&week_start_label);
  settings_root.append(&week_start);

  let show_events_label = settings_label();
  let show_events = gtk::Switch::new();
  show_events.add_css_class("editor-switch");
  let show_events_spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  show_events_spacer.set_hexpand(true);
  let show_events_row = editor_row();
  show_events_row.append(&show_events_label);
  show_events_row.append(&show_events_spacer);
  show_events_row.append(&show_events);
  settings_root.append(&show_events_row);

  let default_duration_label = settings_label();
  let default_duration = settings_number_spin(1440, 4);
  let default_duration_row = editor_row();
  default_duration_row.append(&default_duration_label);
  default_duration_row.append(&default_duration);
  settings_root.append(&default_duration_row);

  let default_reminder_label = settings_label();
  let default_reminder = settings_number_spin(1440, 4);
  let default_reminder_row = editor_row();
  default_reminder_row.append(&default_reminder_label);
  default_reminder_row.append(&default_reminder);
  settings_root.append(&default_reminder_row);

  let sync_interval_label = settings_label();
  let sync_interval = settings_number_spin(1440, 4);
  let sync_interval_suffix = gtk::Label::new(Some(i18n.text(Text::MinuteShort)));
  sync_interval_suffix.add_css_class("editor-value");
  let sync_interval_row = editor_row();
  sync_interval_row.append(&sync_interval_label);
  sync_interval_row.append(&sync_interval);
  sync_interval_row.append(&sync_interval_suffix);
  settings_root.append(&sync_interval_row);

  settings_root.append(&settings_section_label(i18n.text(Text::Terminal)));
  let terminal_command_label = settings_label();
  let terminal_command = editor_entry();
  settings_root.append(&terminal_command_label);
  settings_root.append(&terminal_command);
  let terminal_args_label = settings_label();
  let terminal_args = editor_entry();
  settings_root.append(&terminal_args_label);
  settings_root.append(&terminal_args);

  settings_root.append(&settings_section_label(i18n.text(Text::Editor)));
  let editor_command_label = settings_label();
  let editor_command = editor_entry();
  settings_root.append(&editor_command_label);
  settings_root.append(&editor_command);
  let editor_args_label = settings_label();
  let editor_args = editor_entry();
  settings_root.append(&editor_args_label);
  settings_root.append(&editor_args);

  let edit_file = gtk::Button::new();
  edit_file.add_css_class("editor-action");
  settings_root.append(&edit_file);

  let settings_actions = gtk::Box::new(gtk::Orientation::Horizontal, 8);
  settings_actions.add_css_class("editor-actions");
  let settings_spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  settings_spacer.set_hexpand(true);
  let settings_cancel = gtk::Button::new();
  settings_cancel.add_css_class("editor-action");
  let settings_save = gtk::Button::new();
  settings_save.add_css_class("editor-action");
  settings_save.add_css_class("editor-save");
  settings_actions.append(&settings_spacer);
  settings_actions.append(&settings_cancel);
  settings_actions.append(&settings_save);
  settings_root.append(&settings_actions);

  let settings_scroll = gtk::ScrolledWindow::new();
  settings_scroll.add_css_class("settings-scroll");
  settings_scroll.set_overlay_scrolling(false);
  settings_scroll.set_child(Some(&settings_root));
  settings_scroll.set_min_content_height(620);
  settings_scroll.set_max_content_height(640);
  settings_scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);

  connect_click(&settings_back, sender, || AppMsg::CancelSettings);
  connect_click(&settings_cancel, sender, || AppMsg::CancelSettings);
  connect_click(&settings_save, sender, || AppMsg::SaveSettings);
  connect_click(&edit_file, sender, || AppMsg::OpenConfig);

  // Blocked by `populate_settings` while it fills the form (see the editor).
  let changed_handlers = SettingsChangedHandlers {
    font_family: connect_entry(&font_family, sender, AppMsg::SettingsFontFamily),
    font_size: connect_spin(&font_size, sender, |value| {
      AppMsg::SettingsFontSize(value as u32)
    }),
    language: connect_dropdown(&language, sender, AppMsg::SettingsLanguage),
    week_start: connect_dropdown(&week_start, sender, AppMsg::SettingsWeekStart),
    show_events: show_events.connect_active_notify(clone!(
      #[strong]
      sender,
      move |switch: &gtk::Switch| {
        sender.input(AppMsg::SettingsShowEvents(switch.is_active()));
      }
    )),
    default_duration: connect_spin(&default_duration, sender, |value| {
      AppMsg::SettingsDefaultDuration(value as i64)
    }),
    default_reminder: connect_spin(&default_reminder, sender, |value| {
      AppMsg::SettingsDefaultReminder(value as i64)
    }),
    sync_interval: connect_spin(&sync_interval, sender, |value| {
      AppMsg::SettingsSyncInterval(value as u64)
    }),
    editor_command: connect_entry(&editor_command, sender, AppMsg::SettingsEditorCommand),
    editor_args: connect_entry(&editor_args, sender, AppMsg::SettingsEditorArgs),
    terminal_command: connect_entry(&terminal_command, sender, AppMsg::SettingsTerminalCommand),
    terminal_args: connect_entry(&terminal_args, sender, AppMsg::SettingsTerminalArgs),
  };

  let widgets = SettingsWidgets {
    header: settings_title,
    font_family_label,
    font_family,
    font_size_label,
    font_size,
    language_label,
    language,
    language_model,
    week_start_label,
    week_start,
    week_start_model,
    show_events_label,
    show_events,
    default_duration_label,
    default_duration,
    default_reminder_label,
    default_reminder,
    sync_interval_label,
    sync_interval,
    editor_command_label,
    editor_command,
    editor_args_label,
    editor_args,
    terminal_command_label,
    terminal_command,
    terminal_args_label,
    terminal_args,
    edit_file,
    cancel: settings_cancel,
    save: settings_save,
    changed_handlers,
  };
  (settings_scroll, widgets)
}

/// Closes the popup when it loses focus to a click outside it.
pub(super) fn connect_focus_dismiss(
  window: &gtk::Window,
  sender: &ComponentSender<AppModel>,
  popup_state: &Rc<PopupState>,
) {
  window.connect_has_focus_notify(clone!(
    #[strong]
    sender,
    #[strong]
    popup_state,
    move |window| {
      if window.has_focus() {
        popup_state.focused_once.set(true);
      } else if window.is_visible()
        && popup_state.dismiss_armed.get()
        && popup_state.focused_once.get()
        && pointer_is_outside_popup(popup_state.bounds.get())
      {
        sender.input(AppMsg::ClickOutside);
      }
    }
  ));
}

/// Window-wide shortcuts. While a text field or spin button has focus only
/// Escape is handled, so typing `t` or `n` stays in the field.
pub(super) fn install_keyboard_shortcuts(window: &gtk::Window, sender: &ComponentSender<AppModel>) {
  let key = gtk::EventControllerKey::new();
  key.connect_key_pressed(clone!(
    #[strong]
    sender,
    #[strong]
    window,
    move |_, key, _, modifier| {
      if key != gtk::gdk::Key::Escape && editor_control_has_focus(&window) {
        return gtk::glib::Propagation::Proceed;
      }
      match key {
        gtk::gdk::Key::Escape => {
          sender.input(AppMsg::Hide);
        }
        gtk::gdk::Key::t | gtk::gdk::Key::T => sender.input(AppMsg::Today),
        gtk::gdk::Key::n | gtk::gdk::Key::N => sender.input(AppMsg::OpenNewEvent),
        gtk::gdk::Key::Left => sender.input(AppMsg::MoveDays(-1)),
        gtk::gdk::Key::Right => sender.input(AppMsg::MoveDays(1)),
        gtk::gdk::Key::Page_Up if modifier.contains(gtk::gdk::ModifierType::CONTROL_MASK) => {
          sender.input(AppMsg::PreviousYear)
        }
        gtk::gdk::Key::Page_Down if modifier.contains(gtk::gdk::ModifierType::CONTROL_MASK) => {
          sender.input(AppMsg::NextYear)
        }
        gtk::gdk::Key::Page_Up => sender.input(AppMsg::PreviousMonth),
        gtk::gdk::Key::Page_Down => sender.input(AppMsg::NextMonth),
        _ => return gtk::glib::Propagation::Proceed,
      }
      gtk::glib::Propagation::Stop
    }
  ));
  window.add_controller(key);
}

fn connect_click(
  button: &gtk::Button,
  sender: &ComponentSender<AppModel>,
  message: fn() -> AppMsg,
) {
  let sender = sender.clone();
  button.connect_clicked(move |_| sender.input(message()));
}

fn connect_entry(
  entry: &gtk::Entry,
  sender: &ComponentSender<AppModel>,
  message: fn(String) -> AppMsg,
) -> gtk::glib::SignalHandlerId {
  let sender = sender.clone();
  entry.connect_changed(move |entry: &gtk::Entry| {
    sender.input(message(entry.text().to_string()));
  })
}

fn connect_spin(
  spin: &gtk::SpinButton,
  sender: &ComponentSender<AppModel>,
  message: fn(f64) -> AppMsg,
) -> gtk::glib::SignalHandlerId {
  let sender = sender.clone();
  spin.connect_value_changed(move |spin: &gtk::SpinButton| {
    sender.input(message(spin.value()));
  })
}

fn connect_check(
  check: &gtk::CheckButton,
  sender: &ComponentSender<AppModel>,
  message: fn(bool) -> AppMsg,
) -> gtk::glib::SignalHandlerId {
  let sender = sender.clone();
  check.connect_toggled(move |check: &gtk::CheckButton| {
    sender.input(message(check.is_active()));
  })
}

fn connect_dropdown(
  dropdown: &gtk::DropDown,
  sender: &ComponentSender<AppModel>,
  message: fn(String) -> AppMsg,
) -> gtk::glib::SignalHandlerId {
  let sender = sender.clone();
  dropdown.connect_selected_notify(move |dropdown: &gtk::DropDown| {
    sender.input(message(dropdown_value(dropdown)));
  })
}
