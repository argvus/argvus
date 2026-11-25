//! Read-only notification indicator. Actions still go through argvus-notifications.

mod bus;
mod translations;

use std::cell::RefCell;
use std::io::{self, Write};
use std::path::Path;
use std::process::ExitCode;
use std::rc::Rc;

use gtk::glib::{self, ffi, translate::*};
use serde::Serialize;

use crate::commands::resolve_executable;
use crate::exec::Outcome;

#[derive(Clone, PartialEq, Eq, Serialize)]
#[serde(untagged)]
enum Output {
  Fallback {
    fallback: bool,
  },
  Status {
    text: &'static str,
    tooltip: String,
    class: &'static str,
  },
}

#[derive(Default)]
struct Status {
  paused: Option<bool>,
  labels: Option<translations::Labels>,
  published: Option<Output>,
}

impl Status {
  fn publish(&mut self) -> io::Result<()> {
    let output = match (self.paused, &self.labels) {
      (Some(true), Some(labels)) => Output::Status {
        text: "󰂛",
        tooltip: labels.paused.clone(),
        class: "dnd",
      },
      (Some(false), Some(labels)) => Output::Status {
        text: "󰂚",
        tooltip: labels.enabled.clone(),
        class: "enabled",
      },
      _ => Output::Fallback { fallback: true },
    };
    if self.published.as_ref() != Some(&output) {
      let mut stdout = io::stdout().lock();
      serde_json::to_writer(&mut stdout, &output)?;
      stdout.write_all(b"\n")?;
      stdout.flush()?;
      self.published = Some(output);
    }
    Ok(())
  }
}

fn uses_packaged_status() -> bool {
  let matches = |path: Option<std::path::PathBuf>, packaged: &str| {
    path.and_then(|path| path.canonicalize().ok()).as_deref() == Some(Path::new(packaged))
  };
  matches(
    resolve_executable("notifications"),
    "/usr/lib/argvus/commands/notifications",
  ) && matches(glib::find_program_in_path("dunstctl"), "/usr/bin/dunstctl")
    && matches(
      glib::find_program_in_path("dbus-send"),
      "/usr/bin/dbus-send",
    )
}

pub(super) fn run() -> Outcome {
  let main_loop = glib::MainLoop::new(None, false);
  let status = Rc::new(RefCell::new(Status::default()));
  let pipe = watch_stdout(&main_loop);
  if uses_packaged_status() {
    let translations = translations::watch({
      let status = status.clone();
      let main_loop = main_loop.clone();
      move |labels| {
        let mut status = status.borrow_mut();
        status.labels = labels;
        if status.published.is_some() && status.publish().is_err() {
          main_loop.quit();
        }
      }
    });
    let monitor = main_loop.context().spawn_local(bus::watch({
      let main_loop = main_loop.clone();
      move |paused| {
        let mut status = status.borrow_mut();
        status.paused = paused;
        if status.publish().is_err() {
          main_loop.quit();
        }
      }
    }));
    main_loop.run();
    monitor.abort();
    drop(translations);
  } else if status.borrow_mut().publish().is_ok() {
    main_loop.run();
  }
  pipe.destroy();
  Ok(ExitCode::SUCCESS)
}

// glib 0.22 exposes the IOChannel watch through its FFI. Watching only error
// conditions lets an idle producer detect a closed reader without a heartbeat.
fn watch_stdout(main_loop: &glib::MainLoop) -> glib::Source {
  unsafe extern "C" fn on_closed(
    _channel: *mut ffi::GIOChannel,
    _condition: ffi::GIOCondition,
    user_data: ffi::gpointer,
  ) -> ffi::gboolean {
    // SAFETY: the source owns this boxed MainLoop until its destroy notifier.
    unsafe { &*user_data.cast::<glib::MainLoop>() }.quit();
    ffi::G_SOURCE_REMOVE
  }

  unsafe extern "C" fn free_loop(user_data: ffi::gpointer) {
    // SAFETY: GLib invokes the destroy notifier once for the pointer we boxed.
    drop(unsafe { Box::from_raw(user_data.cast::<glib::MainLoop>()) });
  }

  // SAFETY: GIO dispatches an IOFunc for this type of source. The watch retains
  // the channel, which does not own the stdout fd, and the callback owns its data.
  unsafe {
    let channel = ffi::g_io_channel_unix_new(libc::STDOUT_FILENO);
    let source: glib::Source = from_glib_full(ffi::g_io_create_watch(
      channel,
      ffi::G_IO_ERR | ffi::G_IO_HUP | ffi::G_IO_NVAL,
    ));
    ffi::g_io_channel_unref(channel);
    let callback = std::mem::transmute::<
      unsafe extern "C" fn(*mut ffi::GIOChannel, ffi::GIOCondition, ffi::gpointer) -> ffi::gboolean,
      unsafe extern "C" fn(ffi::gpointer) -> ffi::gboolean,
    >(on_closed);
    ffi::g_source_set_callback(
      source.to_glib_none().0,
      Some(callback),
      Box::into_raw(Box::new(main_loop.clone())).cast(),
      Some(free_loop),
    );
    source.attach(Some(&main_loop.context()));
    source
  }
}
