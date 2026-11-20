//! Centered storage picker for keyboard and command-line launches.

use std::{
  cell::{Cell, RefCell},
  rc::Rc,
  time::Duration,
};

use argvus_theme_selector::picker;
use gtk::{gdk, glib, prelude::*};

use crate::{
  actions::{ActionKind, Actions},
  config::Config,
  device::Device,
  gui::entries::{action_label, action_tooltip, device_entries, device_title},
  i18n, process, snapshot,
};

struct StoragePicker {
  devices: RefCell<Vec<Device>>,
  device_list: gtk::ListBox,
  action_list: gtk::ListBox,
  heading: gtk::Label,
  current_actions: RefCell<Vec<ActionKind>>,
  in_actions: Cell<bool>,
  busy: Cell<bool>,
  config: Config,
  runtime: tokio::runtime::Handle,
  main_loop: glib::MainLoop,
  result: Cell<i32>,
  open_only: bool,
}

impl StoragePicker {
  fn rebuild(&self, devices: Vec<Device>) {
    let selection = self
      .selected_device()
      .map(|device| (device.drive_path, device.object_path));
    let was_in_actions = self.in_actions.get();
    let previous_action = self.action_list.selected_row().and_then(|row| {
      self
        .current_actions
        .borrow()
        .get(row.index() as usize)
        .copied()
    });
    self.device_list.remove_all();
    for device in &devices {
      let icon = if device.locked {
        "changes-prevent-symbolic"
      } else {
        "drive-removable-media-symbolic"
      };
      self
        .device_list
        .append(&picker::row(icon, &device.name, &device_title(device), ""));
    }
    let index = selection.and_then(|(drive, object)| {
      devices
        .iter()
        .position(|device| device.drive_path == drive && device.object_path == object)
    });
    self.devices.replace(devices);
    let row = self.device_list.row_at_index(index.unwrap_or(0) as i32);
    self.device_list.select_row(row.as_ref());
    self.show_actions();
    if was_in_actions && index.is_some() {
      if let Some(action_index) = previous_action.and_then(|kind| {
        self
          .current_actions
          .borrow()
          .iter()
          .position(|action| *action == kind)
      }) {
        self
          .action_list
          .select_row(self.action_list.row_at_index(action_index as i32).as_ref());
      }
      self.focus_actions();
    } else {
      self.focus_devices();
    }
  }

  fn selected_device(&self) -> Option<Device> {
    let row = self.device_list.selected_row()?;
    self.devices.borrow().get(row.index() as usize).cloned()
  }

  fn show_actions(&self) {
    self.action_list.remove_all();
    let Some(device) = self.selected_device() else {
      self.heading.set_label("");
      self.current_actions.borrow_mut().clear();
      return;
    };
    self.heading.set_label(&format!(
      "{} › {}",
      i18n::tr("device.removable_devices"),
      device.name
    ));
    let mut actions = self.current_actions.borrow_mut();
    actions.clear();
    for entry in device_entries(&device) {
      let (icon, key) = match entry.kind {
        ActionKind::Open => ("document-open-symbolic", "Enter"),
        ActionKind::Unmount => ("media-eject-symbolic", "U"),
        ActionKind::PowerOff => ("system-shutdown-symbolic", "P"),
        ActionKind::Copy => ("edit-copy-symbolic", "C"),
        ActionKind::Mount => ("drive-removable-media-symbolic", ""),
        ActionKind::Unlock => ("changes-allow-symbolic", ""),
        ActionKind::Lock => ("changes-prevent-symbolic", ""),
        ActionKind::Eject => ("media-eject-symbolic", ""),
      };
      self.action_list.append(&picker::row(
        icon,
        action_label(entry.kind),
        action_tooltip(entry.kind),
        key,
      ));
      actions.push(entry.kind);
    }
    self
      .action_list
      .select_row(self.action_list.row_at_index(0).as_ref());
  }

  fn focus_actions(&self) {
    if let Some(row) = self.action_list.selected_row() {
      self.set_active_column(true);
      row.grab_focus();
    }
  }

  fn focus_devices(&self) {
    self.set_active_column(false);
    if let Some(row) = self.device_list.selected_row() {
      row.grab_focus();
    }
  }

  fn set_active_column(&self, in_actions: bool) {
    self.in_actions.set(in_actions);
    for (list, inactive) in [
      (&self.device_list, in_actions),
      (&self.action_list, !in_actions),
    ] {
      if inactive {
        list.add_css_class("inactive-selection");
      } else {
        list.remove_css_class("inactive-selection");
      }
    }
  }

  fn activate(self: &Rc<Self>, kind: ActionKind) {
    let Some(device) = self.selected_device() else {
      return;
    };
    if self.busy.replace(true) {
      return;
    }
    let config = self.config.clone();
    let open_only = self.open_only;
    let work = self.runtime.spawn(async move {
      let current = snapshot(&config).await.into_iter().find(|candidate| {
        candidate.drive_path == device.drive_path && candidate.object_path == device.object_path
      });
      match current {
        Some(device)
          if open_only
            || device_entries(&device)
              .iter()
              .any(|entry| entry.kind == kind) =>
        {
          Actions::new(&config).perform(kind, &device).await
        }
        _ => {
          process::notify(
            "argvus-removable-devices",
            i18n::tr("device.cannot_open_the_storage_menu"),
            true,
          );
          1
        }
      }
    });
    let state = self.clone();
    glib::MainContext::default().spawn_local(async move {
      state.result.set(work.await.unwrap_or(1));
      state.main_loop.quit();
    });
  }

  fn refresh(self: &Rc<Self>) -> glib::JoinHandle<()> {
    let weak = Rc::downgrade(self);
    let runtime = self.runtime.clone();
    let config = self.config.clone();
    glib::MainContext::default().spawn_local(async move {
      loop {
        glib::timeout_future(Duration::from_secs(1)).await;
        let cfg = config.clone();
        let Ok(devices) = runtime.spawn(async move { snapshot(&cfg).await }).await else {
          break;
        };
        let Some(state) = weak.upgrade() else { break };
        if !state.busy.get() && *state.devices.borrow() != devices {
          state.rebuild(devices);
        }
      }
    })
  }
}

pub fn run(config: &Config, devices: Vec<Device>, open_only: bool) -> i32 {
  if let Err(error) = gtk::init() {
    eprintln!("argvus-removable-devices: {error}");
    process::notify(
      "argvus-removable-devices",
      i18n::tr("device.cannot_open_the_storage_menu"),
      true,
    );
    return 1;
  }
  gtk::Widget::set_default_direction(if i18n::is_rtl() {
    gtk::TextDirection::Rtl
  } else {
    gtk::TextDirection::Ltr
  });
  let title = i18n::tr("device.removable_devices");
  let window = picker::window(
    "argvus-removable-devices",
    title,
    &argvus_i18n::user_locale(),
  );
  let panel = picker::panel(&window, title);
  let body = gtk::Box::new(gtk::Orientation::Horizontal, 12);
  body.set_vexpand(true);
  let device_list = gtk::ListBox::new();
  let empty = gtk::Label::new(Some(i18n::tr("device.no_removable_storage_devices")));
  empty.set_wrap(true);
  device_list.set_placeholder(Some(&empty));
  let devices_scroll = picker::scroll(&device_list);
  devices_scroll.set_hexpand(open_only);
  if !open_only {
    devices_scroll.set_width_request(window.default_width() / 3);
  }
  body.append(&devices_scroll);
  let action_list = gtk::ListBox::new();
  let heading = gtk::Label::new(None);
  heading.set_xalign(0.0);
  heading.set_ellipsize(gtk::pango::EllipsizeMode::End);
  if !open_only {
    body.append(&gtk::Separator::new(gtk::Orientation::Vertical));
    let actions = gtk::Box::new(gtk::Orientation::Vertical, 8);
    actions.set_hexpand(true);
    actions.append(&heading);
    actions.append(&picker::scroll(&action_list));
    body.append(&actions);
  }
  panel.append(&body);
  let activate = if open_only {
    ("Enter", i18n::tr("device.open"))
  } else {
    ("Enter", "")
  };
  let tab = if open_only { ("", "") } else { ("Tab", "") };
  picker::footer(&panel, &[("↑ ↓", ""), tab, activate, ("Esc", "")]);
  let state = Rc::new(StoragePicker {
    devices: RefCell::new(Vec::new()),
    device_list,
    action_list,
    heading,
    current_actions: RefCell::new(Vec::new()),
    in_actions: Cell::new(false),
    busy: Cell::new(false),
    config: config.clone(),
    runtime: tokio::runtime::Handle::current(),
    main_loop: glib::MainLoop::new(None, false),
    result: Cell::new(if open_only { 1 } else { 0 }),
    open_only,
  });
  connect(&window, &state);
  state.rebuild(devices);
  window.present();
  let refresh = state.refresh();
  state.main_loop.run();
  refresh.abort();
  window.destroy();
  state.result.get()
}

fn connect(window: &gtk::Window, state: &Rc<StoragePicker>) {
  let weak = Rc::downgrade(state);
  state.device_list.connect_row_selected(move |_, _| {
    if let Some(state) = weak.upgrade() {
      state.show_actions();
    }
  });
  let weak = Rc::downgrade(state);
  state.device_list.connect_row_activated(move |list, row| {
    let Some(state) = weak.upgrade() else { return };
    list.select_row(Some(row));
    if state.open_only {
      state.activate(ActionKind::Open);
    } else {
      state.focus_devices();
    }
  });
  let weak = Rc::downgrade(state);
  state.action_list.connect_row_activated(move |_, row| {
    let Some(state) = weak.upgrade() else { return };
    let kind = state
      .current_actions
      .borrow()
      .get(row.index() as usize)
      .copied();
    if let Some(kind) = kind {
      state.activate(kind);
    }
  });
  let weak = Rc::downgrade(state);
  window.connect_close_request(move |_| {
    if let Some(state) = weak.upgrade().filter(|state| !state.busy.get()) {
      state.main_loop.quit();
    }
    glib::Propagation::Stop
  });
  for (list, in_actions) in [(&state.device_list, false), (&state.action_list, true)] {
    let focus = gtk::EventControllerFocus::new();
    let weak = Rc::downgrade(state);
    focus.connect_enter(move |_| {
      if let Some(state) = weak.upgrade() {
        state.set_active_column(in_actions);
      }
    });
    list.add_controller(focus);
  }
  let keys = gtk::EventControllerKey::new();
  keys.set_propagation_phase(gtk::PropagationPhase::Capture);
  let weak = Rc::downgrade(state);
  keys.connect_key_pressed(move |_, key, _, modifiers| {
    let Some(state) = weak.upgrade() else {
      return glib::Propagation::Proceed;
    };
    if state.busy.get() {
      return glib::Propagation::Stop;
    }
    match key {
      gdk::Key::Escape => state.main_loop.quit(),
      gdk::Key::Tab | gdk::Key::ISO_Left_Tab if !state.open_only => {
        if state.in_actions.get() {
          state.focus_devices();
        } else {
          state.focus_actions();
        }
      }
      gdk::Key::Left | gdk::Key::Right if !state.open_only => {}
      gdk::Key::Return | gdk::Key::KP_Enter => {
        let list = if state.in_actions.get() {
          &state.action_list
        } else {
          &state.device_list
        };
        if let Some(row) = list.selected_row() {
          list.emit_by_name::<()>("row-activated", &[&row]);
        }
      }
      gdk::Key::u | gdk::Key::p | gdk::Key::c
        if state.in_actions.get()
          && !modifiers.intersects(
            gdk::ModifierType::CONTROL_MASK
              | gdk::ModifierType::ALT_MASK
              | gdk::ModifierType::SUPER_MASK,
          ) =>
      {
        let kind = match key {
          gdk::Key::u => ActionKind::Unmount,
          gdk::Key::p => ActionKind::PowerOff,
          _ => ActionKind::Copy,
        };
        if state.current_actions.borrow().contains(&kind) {
          state.activate(kind);
        }
      }
      _ => return glib::Propagation::Proceed,
    }
    glib::Propagation::Stop
  });
  window.add_controller(keys);
}
