//! Alarm editor page: time, name, repeat days, sound and snooze length.

use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::rc::Rc;

use argvus_time_core::alarm::{
  AlarmBook, AlarmDraft, AlarmId, AlarmTime, BundledSound, DEFAULT_SNOOZE_MINUTES,
  MAX_SNOOZE_MINUTES, MIN_SNOOZE_MINUTES, SoundChoice, WeekdaySet,
};
use chrono::{Local, NaiveTime, Utc};
use gtk::prelude::*;
use gtk::{gio, glib};

use super::{card, labeled_row, page_box, relative_text};
use crate::app::ClockApp;

/// Entries of the sound list, in order.
const SOUND_CHIME: u32 = 0;
const SOUND_BEACON: u32 = 1;
const SOUND_FILE: u32 = 2;
const SOUND_SILENT: u32 = 3;

pub struct AlarmEditor {
  pub root: gtk::ScrolledWindow,
  hour: gtk::SpinButton,
  minute: gtk::SpinButton,
  preview: gtk::Label,
  label: gtk::Entry,
  days: Vec<gtk::ToggleButton>,
  repeat_hint: gtk::Label,
  sound: gtk::DropDown,
  file_row: gtk::Box,
  file_name: gtk::Label,
  snooze: gtk::SpinButton,
  delete: gtk::Button,
  editing: Rc<Cell<Option<AlarmId>>>,
  custom_file: Rc<RefCell<Option<PathBuf>>>,
}

fn time_spin(max: f64) -> gtk::SpinButton {
  let spin = gtk::SpinButton::with_range(0.0, max, 1.0);
  spin.set_numeric(true);
  spin.set_wrap(true);
  spin.set_orientation(gtk::Orientation::Vertical);
  spin.add_css_class("digits");
  spin.add_css_class("time-spin");
  spin.connect_output(|spin| {
    spin.set_text(&format!("{:02}", spin.value_as_int()));
    glib::Propagation::Stop
  });
  spin
}

impl AlarmEditor {
  pub fn build(app: &Rc<ClockApp>) -> Self {
    let texts = &app.texts;
    let content = page_box();

    let time_row = gtk::Box::builder()
      .orientation(gtk::Orientation::Horizontal)
      .spacing(8)
      .halign(gtk::Align::Center)
      .build();
    let hour = time_spin(23.0);
    let minute = time_spin(59.0);
    hour.update_property(&[gtk::accessible::Property::Label(&texts.tr("editor.hour"))]);
    minute.update_property(&[gtk::accessible::Property::Label(&texts.tr("editor.minute"))]);
    time_row.append(&hour);
    time_row.append(
      &gtk::Label::builder()
        .label(":")
        .css_classes(["display"])
        .build(),
    );
    time_row.append(&minute);
    let preview = gtk::Label::builder()
      .css_classes(["caption"])
      .wrap(true)
      .build();

    let label = gtk::Entry::builder()
      .placeholder_text(texts.tr("alarm.default_title"))
      .max_length(80)
      .width_chars(8)
      .hexpand(true)
      .build();

    let day_box = gtk::Box::builder()
      .orientation(gtk::Orientation::Horizontal)
      .spacing(4)
      .homogeneous(true)
      .css_classes(["weekdays"])
      .build();
    let days: Vec<gtk::ToggleButton> = WeekdaySet::ORDER
      .into_iter()
      .map(|day| {
        let button = gtk::ToggleButton::with_label(&app.calendar.weekday_short(day));
        button.update_property(&[gtk::accessible::Property::Label(&app.calendar.weekday(day))]);
        day_box.append(&button);
        button
      })
      .collect();
    let repeat_hint = gtk::Label::builder()
      .css_classes(["caption"])
      .xalign(0.0)
      .wrap(true)
      .build();

    let sound_names = [
      texts.tr("sound.chime"),
      texts.tr("sound.beacon"),
      texts.tr("sound.file"),
      texts.tr("sound.silent"),
    ];
    let sound_refs: Vec<&str> = sound_names.iter().map(String::as_str).collect();
    let sound = gtk::DropDown::from_strings(&sound_refs);
    let file_name = gtk::Label::builder()
      .xalign(0.0)
      .hexpand(true)
      .ellipsize(gtk::pango::EllipsizeMode::Middle)
      .css_classes(["caption"])
      .build();
    let choose_file = gtk::Button::with_label(&texts.tr("editor.choose_file"));
    let file_row = gtk::Box::builder()
      .orientation(gtk::Orientation::Horizontal)
      .spacing(8)
      .visible(false)
      .build();
    file_row.append(&file_name);
    file_row.append(&choose_file);
    let test_sound = gtk::Button::with_label(&texts.tr("action.test_sound"));

    let snooze = gtk::SpinButton::with_range(
      f64::from(MIN_SNOOZE_MINUTES),
      f64::from(MAX_SNOOZE_MINUTES),
      1.0,
    );
    snooze.set_numeric(true);

    let details = card();
    details.append(&labeled_row(&texts.tr("editor.name"), &label));
    details.append(
      &gtk::Label::builder()
        .label(texts.tr("editor.repeat"))
        .xalign(0.0)
        .build(),
    );
    details.append(&day_box);
    details.append(&repeat_hint);
    let alerts = card();
    alerts.append(&labeled_row(&texts.tr("editor.sound"), &sound));
    alerts.append(&file_row);
    alerts.append(&labeled_row(&texts.tr("editor.sound_test"), &test_sound));
    alerts.append(&labeled_row(&texts.tr("editor.snooze"), &snooze));

    let buttons = gtk::Box::builder()
      .orientation(gtk::Orientation::Horizontal)
      .spacing(10)
      .margin_top(6)
      .build();
    let delete = gtk::Button::builder()
      .label(texts.tr("editor.delete"))
      .css_classes(["destructive-action"])
      .build();
    let spacer = gtk::Box::builder().hexpand(true).build();
    let cancel = gtk::Button::with_label(&texts.tr("action.cancel"));
    let save = gtk::Button::builder()
      .label(texts.tr("editor.save"))
      .css_classes(["primary"])
      .build();
    buttons.append(&delete);
    buttons.append(&spacer);
    buttons.append(&cancel);
    buttons.append(&save);

    content.append(&time_row);
    content.append(&preview);
    content.append(&details);
    content.append(&alerts);
    content.append(&buttons);

    let editor = Self {
      root: gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .child(&content)
        .css_classes(["page"])
        .build(),
      hour,
      minute,
      preview,
      label,
      days,
      repeat_hint,
      sound,
      file_row,
      file_name,
      snooze,
      delete,
      editing: Rc::new(Cell::new(None)),
      custom_file: Rc::default(),
    };
    editor.connect(app, &choose_file, &test_sound, &cancel, &save);
    editor
  }

  fn connect(
    &self,
    app: &Rc<ClockApp>,
    choose_file: &gtk::Button,
    test_sound: &gtk::Button,
    cancel: &gtk::Button,
    save: &gtk::Button,
  ) {
    // Local changes refresh the preview and hints without touching the app.
    for spin in [&self.hour, &self.minute] {
      let weak = Rc::downgrade(app);
      spin.connect_value_changed(move |_| {
        if let Some(app) = weak.upgrade() {
          app.refresh();
        }
      });
    }
    for day in &self.days {
      let weak = Rc::downgrade(app);
      day.connect_toggled(move |_| {
        if let Some(app) = weak.upgrade() {
          app.refresh();
        }
      });
    }
    let file_row = self.file_row.clone();
    self.sound.connect_selected_notify(move |dropdown| {
      file_row.set_visible(dropdown.selected() == SOUND_FILE);
    });

    let custom_file = Rc::clone(&self.custom_file);
    let file_name = self.file_name.clone();
    let weak = Rc::downgrade(app);
    choose_file.connect_clicked(move |_| {
      let Some(app) = weak.upgrade() else {
        return;
      };
      let filter = gtk::FileFilter::new();
      filter.set_name(Some(&app.texts.tr("editor.audio_files")));
      filter.add_mime_type("audio/*");
      let filters = gio::ListStore::new::<gtk::FileFilter>();
      filters.append(&filter);
      let dialog = gtk::FileDialog::builder()
        .title(app.texts.tr("editor.choose_file"))
        .filters(&filters)
        .default_filter(&filter)
        .modal(true)
        .build();
      let custom_file = Rc::clone(&custom_file);
      let file_name = file_name.clone();
      app.while_popup_hidden(move |reshow| {
        dialog.open(
          None::<&gtk::Window>,
          None::<&gio::Cancellable>,
          move |result| {
            if let Some(path) = result.ok().and_then(|file| file.path()) {
              file_name.set_label(&path.display().to_string());
              *custom_file.borrow_mut() = Some(path);
            }
            reshow();
          },
        );
      });
    });

    let weak = Rc::downgrade(app);
    let editor_sound = self.sound.clone();
    let custom_file = Rc::clone(&self.custom_file);
    test_sound.connect_clicked(move |_| {
      if let Some(app) = weak.upgrade() {
        app.test_sound(&sound_choice(
          editor_sound.selected(),
          custom_file.borrow().clone(),
        ));
      }
    });

    let weak = Rc::downgrade(app);
    cancel.connect_clicked(move |_| {
      if let Some(app) = weak.upgrade() {
        app.show_page(super::Page::Alarms);
      }
    });

    let weak = Rc::downgrade(app);
    let fields = self.fields();
    save.connect_clicked(move |_| {
      if let Some(app) = weak.upgrade() {
        let (id, draft) = fields.draft();
        app.save_alarm(id, draft);
      }
    });

    let weak = Rc::downgrade(app);
    let fields = self.fields();
    self.delete.connect_clicked(move |_| {
      if let (Some(app), (Some(id), _)) = (weak.upgrade(), fields.draft()) {
        app.delete_alarm(id);
      }
    });
  }

  /// Shared handles to the inputs, for the button callbacks.
  fn fields(&self) -> Fields {
    Fields {
      hour: self.hour.clone(),
      minute: self.minute.clone(),
      label: self.label.clone(),
      days: self.days.clone(),
      sound: self.sound.clone(),
      snooze: self.snooze.clone(),
      custom_file: Rc::clone(&self.custom_file),
      editing: Rc::clone(&self.editing),
    }
  }

  /// Fills the inputs for a new alarm (`None`) or an existing one.
  pub fn load(&self, app: &ClockApp, id: Option<AlarmId>) {
    let alarm = id.and_then(|id| app.alarms().get(id).cloned());
    self.editing.set(alarm.as_ref().map(|alarm| alarm.id));
    let draft = alarm.map_or_else(
      || AlarmDraft {
        label: String::new(),
        time: AlarmTime::new(7, 0).unwrap_or(AlarmTime::MIDNIGHT),
        repeat: WeekdaySet::ONCE,
        sound: SoundChoice::default(),
        snooze_minutes: DEFAULT_SNOOZE_MINUTES,
      },
      |alarm| alarm.draft(),
    );
    self.hour.set_value(f64::from(draft.time.hour()));
    self.minute.set_value(f64::from(draft.time.minute()));
    self.label.set_text(&draft.label);
    for (button, day) in self.days.iter().zip(WeekdaySet::ORDER) {
      button.set_active(draft.repeat.contains(day));
    }
    let (selected, file) = match draft.sound {
      SoundChoice::Bundled {
        sound: BundledSound::Chime,
      } => (SOUND_CHIME, None),
      SoundChoice::Bundled {
        sound: BundledSound::Beacon,
      } => (SOUND_BEACON, None),
      SoundChoice::File { path } => (SOUND_FILE, Some(path)),
      SoundChoice::Silent => (SOUND_SILENT, None),
    };
    self.sound.set_selected(selected);
    self.file_row.set_visible(selected == SOUND_FILE);
    self.file_name.set_label(&file.as_ref().map_or_else(
      || app.texts.tr("editor.no_file"),
      |path| path.display().to_string(),
    ));
    *self.custom_file.borrow_mut() = file;
    self.snooze.set_value(f64::from(draft.snooze_minutes));
    self.delete.set_visible(id.is_some());
    self.hour.grab_focus();
  }

  /// Preview of the time and of when the alarm would ring.
  pub fn update(&self, app: &ClockApp) {
    let (_, draft) = self.fields().draft();
    let cycle = app.settings().hour_cycle;
    let time = NaiveTime::from_hms_opt(
      u32::from(draft.time.hour()),
      u32::from(draft.time.minute()),
      0,
    )
    .unwrap_or_default();
    let mut book = AlarmBook::default();
    let id = book.add(draft.clone(), Utc::now());
    let next = book.get(id).and_then(|alarm| alarm.next_occurrence(&Local));
    let relative = next.map_or_else(String::new, |at| {
      relative_text(&app.texts, (at - Utc::now()).to_std().unwrap_or_default())
    });
    self.preview.set_label(&app.texts.tr_args(
      "editor.preview",
      [
        ("time", app.calendar.time(&time, cycle, false).joined()),
        ("relative", relative),
      ],
    ));
    let hint = if draft.repeat.is_once() {
      app.texts.tr("editor.rings_once")
    } else {
      super::alarms_page::repeat_text(app, draft.repeat)
    };
    self.repeat_hint.set_label(&hint);
  }
}

struct Fields {
  hour: gtk::SpinButton,
  minute: gtk::SpinButton,
  label: gtk::Entry,
  days: Vec<gtk::ToggleButton>,
  sound: gtk::DropDown,
  snooze: gtk::SpinButton,
  custom_file: Rc<RefCell<Option<PathBuf>>>,
  editing: Rc<Cell<Option<AlarmId>>>,
}

impl Fields {
  fn draft(&self) -> (Option<AlarmId>, AlarmDraft) {
    let field = |spin: &gtk::SpinButton| u8::try_from(spin.value_as_int()).unwrap_or(0);
    let repeat = self
      .days
      .iter()
      .zip(WeekdaySet::ORDER)
      .fold(WeekdaySet::ONCE, |set, (button, day)| {
        set.with(day, button.is_active())
      });
    let draft = AlarmDraft {
      label: self.label.text().to_string(),
      time: AlarmTime::new(field(&self.hour), field(&self.minute)).unwrap_or(AlarmTime::MIDNIGHT),
      repeat,
      sound: sound_choice(self.sound.selected(), self.custom_file.borrow().clone()),
      snooze_minutes: u32::try_from(self.snooze.value_as_int()).unwrap_or(DEFAULT_SNOOZE_MINUTES),
    };
    (self.editing.get(), draft)
  }
}

/// The choice for the selected list entry. "Custom file" without a file
/// falls back to the chime rather than to silence.
fn sound_choice(selected: u32, file: Option<PathBuf>) -> SoundChoice {
  match (selected, file) {
    (SOUND_BEACON, _) => SoundChoice::Bundled {
      sound: BundledSound::Beacon,
    },
    (SOUND_FILE, Some(path)) => SoundChoice::File { path },
    (SOUND_SILENT, _) => SoundChoice::Silent,
    _ => SoundChoice::default(),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn custom_file_without_a_file_is_not_silent() {
    assert_eq!(sound_choice(SOUND_FILE, None), SoundChoice::default());
    assert_eq!(sound_choice(SOUND_SILENT, None), SoundChoice::Silent);
    assert_eq!(
      sound_choice(SOUND_FILE, Some(PathBuf::from("/a.ogg"))),
      SoundChoice::File {
        path: PathBuf::from("/a.ogg")
      }
    );
  }
}
