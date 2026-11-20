//! Keyboard picker for the canonical inactivity lock setting.

use std::{
  cell::Cell,
  process::{Command, ExitCode},
  rc::Rc,
};

use argvus_config_core::ConfigStore;
use argvus_i18n::{Lang, TextDirection};
use argvus_theme_selector::picker;
use gtk::{gdk, gio, glib, prelude::*};

use crate::exec::{Failure, Outcome};

const SETTING: &str = "/power/lock_minutes";
const MINUTES: [u64; 6] = [1, 5, 10, 15, 30, 0];

fn apply(minutes: u64) -> Result<(), (&'static str, String)> {
  ConfigStore::from_environment()
    .and_then(|store| store.update(SETTING, minutes.into()))
    .map_err(|error| ("idle.timeout.save_error", error.to_string()))?;
  let output = Command::new("systemctl")
    .args(["--user", "reload", "argvus-config.service"])
    .output()
    .map_err(|error| ("idle.timeout.reload_error", error.to_string()))?;
  if output.status.success() {
    Ok(())
  } else {
    Err((
      "idle.timeout.reload_error",
      String::from_utf8_lossy(&output.stderr).trim().into(),
    ))
  }
}

pub fn run() -> Outcome {
  let locale = argvus_i18n::user_locale();
  let lang = Lang::for_domain(&locale, "power");
  let hints = Lang::for_domain(&locale, "clipboard");
  let config = ConfigStore::from_environment()
    .and_then(|store| store.load_effective())
    .map_err(|error| Failure::new(format!("{} {error}", lang.tr("idle.timeout.read_error"))))?;
  let current = config.get(SETTING).and_then(serde_json::Value::as_u64);
  let initial = MINUTES
    .iter()
    .position(|&minutes| Some(minutes) == current)
    .unwrap_or(0);

  gtk::init().map_err(|error| Failure::new(format!("GTK: {error}")))?;
  gtk::Widget::set_default_direction(match lang.direction() {
    TextDirection::LeftToRight => gtk::TextDirection::Ltr,
    TextDirection::RightToLeft => gtk::TextDirection::Rtl,
  });
  let title = lang.tr("idle.timeout.title");
  let window = picker::window("argvus-lock-timeout", &format!("ARGVUS {title}"), &locale);
  let panel = picker::panel(&window, &title);
  let list = gtk::ListBox::new();
  let scroll = picker::scroll(&list);
  list.set_activate_on_single_click(false);
  for minutes in MINUTES {
    let label = match minutes {
      0 => lang.tr("idle.timeout.never"),
      1 => lang.tr("idle.timeout.one_minute"),
      _ => lang.tr_args("idle.timeout.minutes", [("count", minutes.to_string())]),
    };
    list.append(&picker::row("system-lock-screen-symbolic", &label, "", ""));
  }
  panel.append(&scroll);

  let status = gtk::Label::new(None);
  status.set_wrap(true);
  status.set_visible(false);
  panel.append(&status);
  let footer = gtk::Box::new(gtk::Orientation::Horizontal, 12);
  footer.add_css_class("footer");
  let shortcuts = gtk::Box::new(gtk::Orientation::Vertical, 0);
  shortcuts.set_hexpand(true);
  picker::footer(
    &shortcuts,
    &[
      ("↑ ↓", &hints.tr("navigate")),
      ("Enter", &lang.tr("idle.timeout.select")),
      ("Esc", &hints.tr("close")),
    ],
  );
  if let Some(hints) = shortcuts.first_child() {
    hints.remove_css_class("footer");
  }
  footer.append(&shortcuts);
  panel.append(&footer);

  let main_loop = glib::MainLoop::new(None, false);
  let busy = Rc::new(Cell::new(false));
  let applying = busy.clone();
  let done = main_loop.clone();
  let message = status.clone();
  list.connect_row_activated(move |list, row| {
    let Some(minutes) = MINUTES.get(row.index() as usize).copied() else {
      return;
    };
    if applying.replace(true) {
      return;
    }
    let list = list.clone();
    let applying = applying.clone();
    let done = done.clone();
    let message = message.clone();
    list.set_sensitive(false);
    message.set_label(&lang.tr("idle.timeout.applying"));
    message.set_visible(true);
    glib::MainContext::default().spawn_local(async move {
      let result = gio::spawn_blocking(move || apply(minutes)).await;
      let error = match result {
        Ok(Ok(())) => {
          done.quit();
          return;
        }
        Ok(Err(error)) => error,
        Err(_) => ("idle.timeout.save_error", String::new()),
      };
      message.set_label(&lang.tr(error.0));
      message.set_tooltip_text((!error.1.is_empty()).then_some(error.1.as_str()));
      applying.set(false);
      list.set_sensitive(true);
      if let Some(row) = list.selected_row() {
        row.grab_focus();
      }
    });
  });

  let keys = gtk::EventControllerKey::new();
  keys.set_propagation_phase(gtk::PropagationPhase::Capture);
  let done = main_loop.clone();
  let key_list = list.clone();
  let applying = busy.clone();
  keys.connect_key_pressed(move |_, key, _, _| {
    if applying.get() {
      return glib::Propagation::Stop;
    }
    let selected = key_list.selected_row();
    match key {
      gdk::Key::Escape => done.quit(),
      gdk::Key::Return | gdk::Key::KP_Enter => {
        if let Some(row) = selected {
          key_list.emit_by_name::<()>("row-activated", &[&row]);
        }
      }
      gdk::Key::Up | gdk::Key::Down => {
        let step = if key == gdk::Key::Up { -1 } else { 1 };
        if let Some(row) = selected.and_then(|row| key_list.row_at_index(row.index() + step)) {
          key_list.select_row(Some(&row));
          row.grab_focus();
        }
      }
      _ => return glib::Propagation::Proceed,
    }
    glib::Propagation::Stop
  });
  window.add_controller(keys);
  let done = main_loop.clone();
  window.connect_close_request(move |_| {
    if !busy.get() {
      done.quit();
    }
    glib::Propagation::Stop
  });

  window.present();
  if let Some(row) = list.row_at_index(initial as i32) {
    list.select_row(Some(&row));
    row.grab_focus();
  }
  main_loop.run();
  window.destroy();
  Ok(ExitCode::SUCCESS)
}
