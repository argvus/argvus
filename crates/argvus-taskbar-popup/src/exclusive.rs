//! Coordinates the four ARGVUS surfaces that share the taskbar's screen edge.

use std::cell::{Cell, RefCell};
use std::ffi::OsStr;
use std::rc::Rc;

use gtk::gio;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PopupApp {
  Focus,
  Clock,
  Calendar,
}

/// Closes the other taskbar popups before `on_closed` opens this one.
pub fn close_competing_surfaces(active: PopupApp, on_closed: impl FnOnce() + 'static) {
  let mut commands = Vec::new();
  for (app, id) in [
    (PopupApp::Focus, "sh.argvus.Focus"),
    (PopupApp::Clock, "sh.argvus.Clock"),
  ] {
    if active != app {
      commands.push(vec!["gapplication", "action", id, "close"]);
    }
  }
  if active != PopupApp::Calendar {
    commands.push(vec!["argvus", "taskbar-calendar", "hide"]);
  }
  commands.push(vec![
    "qs",
    "-c",
    "argvus-control-panel",
    "ipc",
    "call",
    "sidebar",
    "close",
  ]);
  let remaining = Rc::new(Cell::new(0));
  let callback: Rc<RefCell<Option<Box<dyn FnOnce()>>>> =
    Rc::new(RefCell::new(Some(Box::new(on_closed))));

  for command in commands {
    let argv: Vec<&OsStr> = command.iter().map(OsStr::new).collect();
    let process = match gio::Subprocess::newv(
      &argv,
      gio::SubprocessFlags::STDOUT_SILENCE | gio::SubprocessFlags::STDERR_SILENCE,
    ) {
      Ok(process) => process,
      Err(_) => continue,
    };

    remaining.set(remaining.get() + 1);
    let remaining = Rc::clone(&remaining);
    let callback = Rc::clone(&callback);
    process.wait_async(None::<&gio::Cancellable>, move |result| {
      let _ = result;
      remaining.set(remaining.get() - 1);
      if remaining.get() == 0 {
        if let Some(callback) = callback.borrow_mut().take() {
          callback();
        }
      }
    });
  }

  if remaining.get() == 0 {
    if let Some(callback) = callback.borrow_mut().take() {
      callback();
    }
  }
}
