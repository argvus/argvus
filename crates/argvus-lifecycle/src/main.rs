use argvus_lifecycle::{Action, Screen, State};
use argvus_session_actions::{Handoff, run};
use gtk::{gdk, glib, prelude::*};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use std::{cell::RefCell, process::Command, rc::Rc};

const ROWS: [(&str, &str, &str); 5] = [
  ("menu.lock", "system-lock-screen-symbolic", "L"),
  ("menu.suspend", "weather-clear-night-symbolic", "S"),
  ("menu.logout", "system-log-out-symbolic", "O"),
  ("menu.reboot", "system-reboot-symbolic", "R"),
  ("menu.shutdown", "system-shutdown-symbolic", "D"),
];

fn tr(domain: &str, key: &str) -> &'static str {
  argvus_i18n::tr(
    argvus_i18n::Lang::for_domain(&argvus_i18n::user_locale(), domain),
    key,
  )
}

fn common(key: &str) -> &'static str {
  tr("control-center", key)
}

fn main() -> glib::ExitCode {
  let preview = match std::env::args().skip(1).collect::<Vec<_>>().as_slice() {
    [] => false,
    [argument] if argument == "--preview" => true,
    [argument] if argument == "--help" => {
      println!("Usage: argvus-lifecycle [--preview]");
      return glib::ExitCode::SUCCESS;
    }
    _ => return glib::ExitCode::FAILURE,
  };
  let id = if preview {
    "org.argvus.Lifecycle.Preview"
  } else {
    "org.argvus.Lifecycle"
  };
  let app = gtk::Application::builder().application_id(id).build();
  app.connect_activate(move |app| {
    if let Some(window) = app.windows().first() {
      if window.is_visible() {
        window.present();
      }
    } else if gtk4_layer_shell::is_supported() {
      open(app, preview);
    } else {
      eprintln!("argvus-lifecycle: Wayland layer-shell is unavailable");
      app.quit();
    }
  });
  app.run_with_args::<&str>(&[])
}

fn open(app: &gtk::Application, preview: bool) {
  let window = gtk::ApplicationWindow::builder()
    .application(app)
    .title("ARGVUS Lifecycle")
    .build();
  window.init_layer_shell();
  window.set_namespace(Some("argvus-lifecycle"));
  window.set_layer(Layer::Overlay);
  window.set_keyboard_mode(KeyboardMode::Exclusive);
  window.set_exclusive_zone(-1);
  for edge in [Edge::Top, Edge::Bottom, Edge::Left, Edge::Right] {
    window.set_anchor(edge, true);
  }
  let display = gtk::prelude::WidgetExt::display(&window);
  if let Some(output) = focused_output() {
    let monitors = display.monitors();
    for index in 0..monitors.n_items() {
      if let Some(monitor) = monitors.item(index).and_downcast::<gdk::Monitor>()
        && monitor.connector().as_deref() == Some(output.as_str())
      {
        window.set_monitor(Some(&monitor));
        update_size(&window, &monitor);
        let weak = window.downgrade();
        monitor.connect_geometry_notify(move |monitor| {
          if let Some(window) = weak.upgrade() {
            update_size(&window, monitor);
          }
        });
        let weak = window.downgrade();
        monitor.connect_invalidate(move |_| {
          if let Some(window) = weak.upgrade() {
            window.close();
          }
        });
        break;
      }
    }
  }

  let _theme = argvus_time_ui::theme::ThemeStyle::install(include_str!("style.css"));
  window.add_css_class("lifecycle");
  let frame = gtk::Overlay::new();
  frame.add_css_class("lifecycle-frame");
  window.set_child(Some(&frame));
  let center = gtk::Box::new(gtk::Orientation::Vertical, 12);
  center.set_halign(gtk::Align::Center);
  center.set_valign(gtk::Align::Center);
  frame.set_child(Some(&center));

  let header = gtk::Box::new(gtk::Orientation::Horizontal, 20);
  header.add_css_class("lifecycle-header");
  header.set_halign(gtk::Align::Start);
  header.set_valign(gtk::Align::Start);
  let brand = gtk::Label::new(Some("ARGVUS"));
  brand.add_css_class("lifecycle-brand");
  header.append(&brand);
  header.append(&gtk::Label::new(Some("Lifecycle")));
  frame.add_overlay(&header);

  let footer = gtk::CenterBox::new();
  footer.add_css_class("lifecycle-footer");
  footer.set_valign(gtk::Align::End);
  footer.set_start_widget(Some(&gtk::Label::new(Some(common(
    "control_center.navigate",
  )))));
  footer.set_center_widget(Some(&gtk::Label::new(Some(&format!(
    "Enter  {}",
    common("control_center.confirm")
  )))));
  footer.set_end_widget(Some(&gtk::Label::new(Some(&format!(
    "Esc  {}",
    common("control_center.cancel")
  )))));
  frame.add_overlay(&footer);

  let ui = Rc::new(Ui {
    preview,
    window: window.downgrade(),
    state: RefCell::new(State::default()),
    center,
    status: gtk::Label::new(None),
    buttons: ROWS.map(|(label, icon, key)| {
      let button = gtk::Button::new();
      button.add_css_class("lifecycle-action");
      let row = gtk::Box::new(gtk::Orientation::Horizontal, 18);
      let image = gtk::Image::from_icon_name(icon);
      image.set_pixel_size(28);
      let title = gtk::Label::new(Some(tr("power", label)));
      title.set_hexpand(true);
      title.set_xalign(0.0);
      row.append(&image);
      row.append(&title);
      row.append(&gtk::Label::new(Some(&format!("[{key}]"))));
      button.set_child(Some(&row));
      button
    }),
    question: gtk::Label::new(None),
    confirm: gtk::Button::with_label(common("control_center.confirm")),
    cancel: gtk::Button::with_label(common("control_center.cancel")),
    choices: gtk::Box::new(gtk::Orientation::Horizontal, 12),
  });
  ui.connect(&window);
  ui.render();
  if preview {
    ui.status.set_label("--preview");
  }
  window.present();
}

fn update_size(window: &gtk::ApplicationWindow, monitor: &gdk::Monitor) {
  let geometry = monitor.geometry();
  if geometry.height() < 650 || geometry.width() < 620 {
    window.add_css_class("compact");
  } else {
    window.remove_css_class("compact");
  }
}

fn focused_output() -> Option<String> {
  let output = Command::new("hyprctl")
    .args(["monitors", "-j"])
    .output()
    .ok()?;
  let data: serde_json::Value = serde_json::from_slice(&output.stdout).ok()?;
  data
    .as_array()?
    .iter()
    .find(|monitor| monitor["focused"] == true)?["name"]
    .as_str()
    .map(str::to_owned)
}

struct Ui {
  preview: bool,
  window: glib::WeakRef<gtk::ApplicationWindow>,
  state: RefCell<State>,
  center: gtk::Box,
  status: gtk::Label,
  buttons: [gtk::Button; 5],
  question: gtk::Label,
  confirm: gtk::Button,
  cancel: gtk::Button,
  choices: gtk::Box,
}

impl Ui {
  fn connect(self: &Rc<Self>, window: &gtk::ApplicationWindow) {
    self.choices.append(&self.confirm);
    self.choices.append(&self.cancel);
    for (button, action) in self.buttons.iter().zip(Action::ALL) {
      self.center.append(button);
      let motion = gtk::EventControllerMotion::new();
      let weak = Rc::downgrade(self);
      motion.connect_enter(move |_, _, _| {
        if let Some(ui) = weak.upgrade() {
          ui.state.borrow_mut().select(action);
          ui.render();
        }
      });
      button.add_controller(motion);
      let weak = Rc::downgrade(self);
      button.connect_clicked(move |_| {
        if let Some(ui) = weak.upgrade() {
          ui.state.borrow_mut().select(action);
          ui.activate();
        }
      });
    }
    self.center.append(&self.question);
    self.center.append(&self.choices);
    self.center.append(&self.status);
    self.status.set_wrap(true);
    for (button, confirm) in [(&self.confirm, true), (&self.cancel, false)] {
      let weak = Rc::downgrade(self);
      button.connect_clicked(move |_| {
        if let Some(ui) = weak.upgrade() {
          ui.state.borrow_mut().set_confirmation(confirm);
          ui.activate();
        }
      });
    }
    let keys = gtk::EventControllerKey::new();
    keys.set_propagation_phase(gtk::PropagationPhase::Capture);
    let ui = Rc::clone(self);
    let enter_down = Rc::new(std::cell::Cell::new(false));
    let released = Rc::clone(&enter_down);
    keys.connect_key_released(move |_, key, _, _| {
      if matches!(key, gdk::Key::Return | gdk::Key::KP_Enter) {
        released.set(false);
      }
    });
    keys.connect_key_pressed(move |_, key, _, modifiers| {
      if modifiers.intersects(
        gdk::ModifierType::CONTROL_MASK
          | gdk::ModifierType::ALT_MASK
          | gdk::ModifierType::SUPER_MASK,
      ) {
        return glib::Propagation::Proceed;
      }
      if matches!(key, gdk::Key::Return | gdk::Key::KP_Enter) {
        if !enter_down.replace(true) {
          ui.activate();
        }
      } else {
        ui.key(key);
      }
      glib::Propagation::Stop
    });
    window.add_controller(keys);
  }

  fn key(&self, key: gdk::Key) {
    let mut state = self.state.borrow_mut();
    if key == gdk::Key::Escape {
      if state.cancel()
        && let Some(window) = self.window.upgrade()
      {
        window.close();
      }
    } else if let Screen::Confirm { confirmed, .. } = state.screen {
      match key {
        gdk::Key::Left | gdk::Key::Up | gdk::Key::k => state.set_confirmation(true),
        gdk::Key::Right | gdk::Key::Down | gdk::Key::j => state.set_confirmation(false),
        gdk::Key::Tab | gdk::Key::ISO_Left_Tab => state.set_confirmation(!confirmed),
        _ => {}
      }
    } else {
      match key.to_lower() {
        gdk::Key::Up | gdk::Key::k => state.move_selection(-1),
        gdk::Key::Down | gdk::Key::j => state.move_selection(1),
        gdk::Key::l => state.select(Action::Lock),
        gdk::Key::s => state.select(Action::Suspend),
        gdk::Key::o => state.select(Action::LogOut),
        gdk::Key::r => state.select(Action::Reboot),
        gdk::Key::d => state.select(Action::Shutdown),
        _ => {}
      }
    }
    drop(state);
    self.render();
  }

  fn render(&self) {
    let state = self.state.borrow();
    let confirmation = matches!(state.screen, Screen::Confirm { .. });
    for (button, action) in self.buttons.iter().zip(Action::ALL) {
      button.set_visible(!confirmation);
      button.set_sensitive(state.screen == Screen::List);
      select(button, action == state.selected);
    }
    self.question.set_visible(confirmation);
    self.choices.set_visible(confirmation);
    if let Screen::Confirm { action, confirmed } = state.screen {
      let question = match action {
        Action::LogOut => "menu.logout",
        Action::Reboot => "menu.reboot",
        Action::Shutdown => "menu.shutdown",
        _ => unreachable!("only destructive actions request confirmation"),
      };
      self
        .question
        .set_label(&format!("{}?", tr("power", question)));
      select(&self.confirm, confirmed);
      select(&self.cancel, !confirmed);
    }
  }

  fn activate(self: &Rc<Self>) {
    let action = self.state.borrow_mut().activate();
    self.render();
    let Some(action) = action else { return };
    if self.preview {
      self.state.borrow_mut().execution_failed();
      self.render();
      return;
    }
    self.status.set_label(common("control_center.loading"));
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
      let _ = sender.send(run(action, || {
        let _ = sender.send(Ok(Handoff::Ready));
      }));
    });
    let weak = Rc::downgrade(self);
    glib::timeout_add_local(std::time::Duration::from_millis(40), move || {
      let Some(ui) = weak.upgrade() else {
        return glib::ControlFlow::Break;
      };
      match receiver.try_recv() {
        Ok(Ok(Handoff::Ready)) => {
          if let Some(window) = ui.window.upgrade() {
            window.set_visible(false);
          }
          return glib::ControlFlow::Continue;
        }
        Ok(Ok(Handoff::Finished)) => {
          if let Some(window) = ui.window.upgrade() {
            window.close();
          }
        }
        Ok(Err(error)) => {
          eprintln!("argvus-lifecycle: {action:?}: {error}");
          ui.status.set_label(common("control_center.error"));
          ui.state.borrow_mut().execution_failed();
          ui.render();
          if let Some(window) = ui.window.upgrade() {
            window.present();
          }
        }
        Err(std::sync::mpsc::TryRecvError::Empty) => return glib::ControlFlow::Continue,
        Err(std::sync::mpsc::TryRecvError::Disconnected) => {
          ui.status.set_label(common("control_center.error"));
          ui.state.borrow_mut().execution_failed();
          ui.render();
          if let Some(window) = ui.window.upgrade() {
            window.present();
          }
        }
      }
      glib::ControlFlow::Break
    });
  }
}

fn select(button: &gtk::Button, selected: bool) {
  if selected {
    button.add_css_class("selected");
  } else {
    button.remove_css_class("selected");
  }
}
