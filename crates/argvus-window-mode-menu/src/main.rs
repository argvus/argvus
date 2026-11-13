//! Taskbar window policy selector. All compositor work runs on worker threads.
use argvus_config_core::window_mode::{self, Status, WindowMode};
use argvus_i18n::I18n;
use argvus_taskbar_popup::{PopupController, PopupSize};
use gtk::{gio, glib, prelude::*};
use std::cell::Cell;
use std::rc::Rc;
use std::sync::{Arc, mpsc};
use std::time::Duration;

fn key(mode: WindowMode) -> &'static str {
  match mode {
    WindowMode::Tile => "window_mode.tile",
    WindowMode::Floating => "window_mode.floating",
  }
}

fn state_key(status: &Status) -> &'static str {
  if status.error.is_some() {
    "window_mode.failed"
  } else if status.pending {
    "window_mode.pending"
  } else {
    "window_mode.hint"
  }
}

fn main() -> glib::ExitCode {
  let i18n = match I18n::new("taskbar") {
    Ok(i18n) => Arc::new(i18n),
    Err(error) => {
      eprintln!("argvus-window-mode-menu: {error}");
      return glib::ExitCode::FAILURE;
    }
  };
  let arguments: Vec<_> = std::env::args().skip(1).collect();
  if arguments == ["--status"] {
    return match window_mode::status() {
      Ok(status) => {
        let mode = status.applied.unwrap_or(status.preferred);
        let icon = match mode {
          WindowMode::Tile => "󰕰",
          WindowMode::Floating => "󰖯",
        };
        println!(
          "{}",
          serde_json::json!({
            "text": icon,
            "tooltip": format!("{}\n{}", i18n.tr(key(mode)), i18n.tr(state_key(&status))),
            "class": if status.error.is_some() { "error" } else if status.pending { "pending" } else { mode.as_str() }
          })
        );
        glib::ExitCode::SUCCESS
      }
      Err(error) => {
        eprintln!("argvus-window-mode-menu: {error}");
        println!(
          "{}",
          serde_json::json!({"text": "󰕰", "tooltip": i18n.tr("window_mode.failed"), "class": "error"})
        );
        glib::ExitCode::SUCCESS
      }
    };
  }
  if !arguments.is_empty() {
    return glib::ExitCode::FAILURE;
  }
  let application = gtk::Application::builder()
    .application_id("sh.argvus.WindowMode")
    .build();
  application.connect_activate(move |app| {
    gtk::Widget::set_default_direction(
      if i18n.direction() == argvus_i18n::TextDirection::RightToLeft {
        gtk::TextDirection::Rtl
      } else {
        gtk::TextDirection::Ltr
      },
    );
    if let Some(window) = app.active_window() {
      window.close();
      return;
    }
    open(app, Arc::clone(&i18n));
  });
  application.run_with_args::<&str>(&[])
}

fn open(app: &gtk::Application, i18n: Arc<I18n>) {
  let window = gtk::ApplicationWindow::builder()
    .application(app)
    .title(i18n.tr("window_mode.title"))
    .default_width(320)
    .resizable(false)
    .build();
  window.add_css_class("argvus-window-mode");
  let colors = gtk::CssProvider::new();
  if let Some(display) = gtk::gdk::Display::default() {
    gtk::style_context_add_provider_for_display(
      &display,
      &colors,
      gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
  }
  let mut last_css = String::new();
  load_theme(&colors, &mut last_css);
  let surface: &gtk::Window = window.upcast_ref();
  PopupController::apply_layer_shell(surface, "argvus-window-mode");
  let controller = PopupController::new();
  let weak = window.downgrade();
  controller.connect_focus_dismiss(surface, move || {
    if let Some(window) = weak.upgrade() {
      window.close();
    }
  });
  let keys = gtk::EventControllerKey::new();
  let weak = window.downgrade();
  keys.connect_key_pressed(move |_, key, _, _| {
    if key == gtk::gdk::Key::Escape {
      if let Some(window) = weak.upgrade() {
        window.close();
      }
      glib::Propagation::Stop
    } else if key == gtk::gdk::Key::Down || key == gtk::gdk::Key::Up {
      if let Some(window) = weak.upgrade() {
        window.child_focus(if key == gtk::gdk::Key::Down {
          gtk::DirectionType::TabForward
        } else {
          gtk::DirectionType::TabBackward
        });
      }
      glib::Propagation::Stop
    } else {
      glib::Propagation::Proceed
    }
  });
  window.add_controller(keys);
  let content = gtk::Box::new(gtk::Orientation::Vertical, 8);
  content.set_margin_top(16);
  content.set_margin_bottom(16);
  content.set_margin_start(16);
  content.set_margin_end(16);
  let title = gtk::Label::new(Some(&i18n.tr("window_mode.title")));
  title.add_css_class("heading");
  content.append(&title);
  let message = gtk::Label::new(Some(&i18n.tr("window_mode.working")));
  message.set_wrap(true);
  message.set_max_width_chars(38);
  let (sender, receiver) = mpsc::channel();
  let busy = Rc::new(Cell::new(true));
  let mut buttons = Vec::new();
  for mode in [WindowMode::Tile, WindowMode::Floating] {
    let button = gtk::Button::with_label(&i18n.tr(key(mode)));
    let sender = sender.clone();
    let busy = Rc::clone(&busy);
    let message = message.clone();
    let i18n = Arc::clone(&i18n);
    button.connect_clicked(move |_| {
      if busy.replace(true) {
        return;
      }
      message.set_text(&i18n.tr("window_mode.working"));
      let sender = sender.clone();
      std::thread::spawn(move || {
        let _ = sender.send((
          true,
          window_mode::set(mode).map_err(|error| error.to_string()),
        ));
      });
    });
    content.append(&button);
    buttons.push((mode, button));
  }
  content.append(&message);
  window.set_child(Some(&content));
  let initial_sender = sender.clone();
  std::thread::spawn(move || {
    let _ = initial_sender.send((
      false,
      window_mode::status().map_err(|error| error.to_string()),
    ));
  });
  let weak = window.downgrade();
  let mut refreshed = std::time::Instant::now();
  glib::timeout_add_local(Duration::from_millis(100), move || {
    let Some(window) = weak.upgrade() else {
      return glib::ControlFlow::Break;
    };
    if let Ok((action, result)) = receiver.try_recv() {
      busy.set(false);
      refreshed = std::time::Instant::now();
      match result {
        Ok(status) => {
          for (mode, button) in &buttons {
            button.set_label(&format!(
              "{}{}",
              if *mode == status.preferred {
                "✓ "
              } else {
                ""
              },
              i18n.tr(key(*mode))
            ));
          }
          message.set_text(&i18n.tr(state_key(&status)));
          if let Some(error) = status.error.as_ref() {
            eprintln!("argvus-window-mode-menu: {error}");
          }
          if action && !status.pending {
            window.close();
            return glib::ControlFlow::Break;
          }
        }
        Err(error) => {
          eprintln!("argvus-window-mode-menu: {error}");
          message.set_text(&i18n.tr("window_mode.failed"));
        }
      }
    }
    for (_, button) in &buttons {
      button.set_sensitive(!busy.get());
    }
    if !busy.get() && refreshed.elapsed() >= Duration::from_secs(2) {
      busy.set(true);
      load_theme(&colors, &mut last_css);
      let sender = sender.clone();
      std::thread::spawn(move || {
        let _ = sender.send((
          false,
          window_mode::status().map_err(|error| error.to_string()),
        ));
      });
    }
    glib::ControlFlow::Continue
  });
  controller.present(
    surface,
    &content,
    None,
    PopupSize {
      width: 320,
      height_estimate: 210,
    },
  );
  // Keep controller alive while its window is visible through its callbacks.
  let close = gio::SimpleAction::new("close", None);
  let weak = window.downgrade();
  close.connect_activate(move |_, _| {
    let _ = &controller;
    if let Some(window) = weak.upgrade() {
      window.close();
    }
  });
  app.add_action(&close);
}

fn load_theme(provider: &gtk::CssProvider, previous: &mut String) {
  let Some(paths) = argvus_config_core::session_paths::SessionPaths::from_environment() else {
    return;
  };
  let path = paths
    .data_home
    .join("generated/theme/current/window-mode/theme.css");
  if let Ok(css) = std::fs::read_to_string(path)
    && css != *previous
  {
    provider.load_from_string(&css);
    *previous = css;
  }
}
