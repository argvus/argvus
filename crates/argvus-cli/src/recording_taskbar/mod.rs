//! Status only. Recording actions still belong to the Hyprland script.

mod bus;
mod files;
mod marker;

use std::cell::RefCell;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::rc::Rc;

use gtk::glib::{self, ffi, translate::*};
use serde::Serialize;

use crate::exec::{Outcome, env_nonempty};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RecordingState {
  Stopped,
  Recording,
  Paused,
}

impl RecordingState {
  fn from_sources(active: bool, marker: &[u8]) -> Self {
    if !active {
      return Self::Stopped;
    }
    if marker.starts_with(b"paused") && marker[6..].iter().all(|byte| *byte == b'\n') {
      Self::Paused
    } else {
      Self::Recording
    }
  }
}

#[derive(PartialEq, Eq, Serialize)]
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
  recording: Option<RecordingState>,
  labels: Option<files::Labels>,
  published: Option<Output>,
}

impl Status {
  fn publish(&mut self) -> io::Result<()> {
    let output = match (self.recording, self.labels.as_ref()) {
      (Some(recording), Some(labels)) => {
        let (text, tooltip, class) = match recording {
          RecordingState::Stopped => (
            "<span font_family='Symbols Nerd Font Mono'></span>",
            &labels.stopped,
            "stopped",
          ),
          RecordingState::Recording => (
            "<span font_family='Symbols Nerd Font Mono'></span>",
            &labels.recording,
            "recording",
          ),
          RecordingState::Paused => (
            "<span font_family='Symbols Nerd Font Mono'></span>",
            &labels.paused,
            "paused",
          ),
        };
        // Interpret the tooltip the same way as a consumer of the shell JSON.
        // Quotes and escapes in custom catalogs can make that JSON invalid.
        match serde_json::from_str::<String>(&format!("\"{tooltip}\"")) {
          Ok(tooltip) => Output::Status {
            text,
            tooltip,
            class,
          },
          Err(_) => Output::Fallback { fallback: true },
        }
      }
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

fn marker_path() -> Option<PathBuf> {
  let home = env_nonempty("HOME")?;
  if env_nonempty("ARGVUS_BOOTSTRAP").is_some()
    || env_nonempty("ARGVUS_SYSTEM_CONFIG").is_some_and(|root| root != "/usr/share/argvus")
    || glib::find_program_in_path("systemctl")
      .and_then(|path| path.canonicalize().ok())
      .as_deref()
      != Some(Path::new("/usr/bin/systemctl"))
  {
    return None;
  }
  // systemctl --user can talk to the manager through its private socket.
  // Only use events from the matching, standard user bus.
  let runtime = PathBuf::from(env_nonempty("XDG_RUNTIME_DIR")?);
  let socket = runtime.join("bus");
  if !runtime.is_absolute()
    || env_nonempty("DBUS_SESSION_BUS_ADDRESS").is_some_and(|address| {
      address
        .as_os_str()
        .as_encoded_bytes()
        .strip_prefix(b"unix:path=")
        != Some(socket.as_os_str().as_encoded_bytes())
    })
  {
    return None;
  }
  let cache = env_nonempty("ARGVUS_CACHE_HOME")
    .map(PathBuf::from)
    .unwrap_or_else(|| {
      env_nonempty("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(home).join(".cache"))
        .join("argvus")
    });
  std::path::absolute(cache.join("hypr/gpu-screen-recorder.state")).ok()
}

pub(super) fn run() -> Outcome {
  let main_loop = glib::MainLoop::new(None, false);
  let status = Rc::new(RefCell::new(Status::default()));
  let pipe = watch_stdout(&main_loop);
  if let Some(marker_path) = marker_path() {
    let translations = files::watch({
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
    let monitor = main_loop.context().spawn_local(bus::watch(marker_path, {
      let main_loop = main_loop.clone();
      move |recording| {
        let mut status = status.borrow_mut();
        status.recording = recording;
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn matches_legacy_marker_rules() {
    for (active, marker, expected) in [
      (false, b"paused\n".as_slice(), RecordingState::Stopped),
      (true, b"paused\n\n", RecordingState::Paused),
      (true, b"paused", RecordingState::Paused),
      (true, b"paused ", RecordingState::Recording),
      (true, b"paused\r\n", RecordingState::Recording),
      (true, b"recording", RecordingState::Recording),
      (true, b"", RecordingState::Recording),
    ] {
      assert_eq!(RecordingState::from_sources(active, marker), expected);
    }
  }
}
