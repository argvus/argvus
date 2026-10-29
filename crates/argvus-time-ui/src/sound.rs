//! Sound playback through `GtkMediaFile`.
//!
//! GTK plays media with its GStreamer backend, which reaches PipeWire,
//! PulseAudio or ALSA through the usual GStreamer sinks, so no extra Rust
//! audio stack or external player is needed. The Ogg Vorbis decoder comes
//! from `gst-plugins-base` and the audio sinks from `gst-plugins-good` (or
//! `gst-plugin-pipewire`).
//!
//! A sound that does not start is reported to the caller: through the error
//! callback when GStreamer reports an error, or when the stream is still not
//! prepared after [`START_TIMEOUT`].

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Duration;

use gtk::glib;
use gtk::prelude::*;

/// Overrides the sounds directory, for running from the source tree.
pub const SOUNDS_DIR_VARIABLE: &str = "ARGVUS_SOUNDS_DIR";
const SYSTEM_SOUNDS_DIR: &str = "/usr/share/argvus/sounds";
/// The short bell ARGVUS Focus plays at the end of a phase.
pub const BELL_FILE: &str = "argvus-bell.oga";
/// How long a stream may take to become ready before it counts as failed.
pub const START_TIMEOUT: Duration = Duration::from_secs(3);

/// Directory of the sounds shipped with ARGVUS.
pub fn sounds_dir() -> PathBuf {
  std::env::var_os(SOUNDS_DIR_VARIABLE)
    .map(PathBuf::from)
    .filter(|path| !path.as_os_str().is_empty())
    .unwrap_or_else(|| PathBuf::from(SYSTEM_SOUNDS_DIR))
}

/// Full path of a packaged sound.
pub fn bundled_sound(file_name: &str) -> PathBuf {
  sounds_dir().join(file_name)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Playback {
  Once,
  /// Repeats until [`SoundPlayer::stop`].
  Loop,
}

/// Why a sound could not start.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SoundError {
  /// The file does not exist or is not a regular file.
  Missing(PathBuf),
  /// GStreamer reported an error, or the stream never became ready.
  Playback(String),
}

impl std::fmt::Display for SoundError {
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Missing(path) => write!(formatter, "sound file not found: {}", path.display()),
      Self::Playback(message) => formatter.write_str(message),
    }
  }
}

impl std::error::Error for SoundError {}

type FailureHandler = Box<dyn Fn(&SoundError)>;

#[derive(Default)]
struct Active {
  stream: Option<gtk::MediaFile>,
  watchdog: Option<glib::SourceId>,
  on_failure: Option<Rc<FailureHandler>>,
}

/// Plays one sound at a time. Cloning gives another handle to the same
/// player.
#[derive(Clone, Default)]
pub struct SoundPlayer {
  active: Rc<RefCell<Active>>,
}

impl SoundPlayer {
  pub fn new() -> Self {
    Self::default()
  }

  /// Starts `path`, replacing whatever this player was playing. `volume` is
  /// 0.0 to 1.0. `on_failure` runs at most once, on the main loop, if the
  /// sound fails after it started loading.
  pub fn play(
    &self,
    path: &Path,
    playback: Playback,
    volume: f64,
    on_failure: impl Fn(&SoundError) + 'static,
  ) -> Result<(), SoundError> {
    self.stop();
    if !path.is_file() {
      return Err(SoundError::Missing(path.to_owned()));
    }

    let stream = gtk::MediaFile::for_filename(path);
    stream.set_loop(playback == Playback::Loop);
    stream.set_volume(volume.clamp(0.0, 1.0));

    let player = self.downgrade();
    stream.connect_error_notify(move |stream| {
      if let (Some(player), Some(error)) = (player.upgrade(), stream.error()) {
        player.fail(SoundError::Playback(error.message().to_owned()));
      }
    });
    if playback == Playback::Once {
      let player = self.downgrade();
      stream.connect_ended_notify(move |stream| {
        if stream.is_ended()
          && let Some(player) = player.upgrade()
        {
          player.stop();
        }
      });
    }

    let player = self.downgrade();
    let watchdog = glib::timeout_add_local_once(START_TIMEOUT, move || {
      let Some(player) = player.upgrade() else {
        return;
      };
      let prepared = {
        let mut active = player.active.borrow_mut();
        active.watchdog = None;
        active.stream.as_ref().map(MediaStreamExt::is_prepared)
      };
      if prepared == Some(false) {
        player.fail(SoundError::Playback(
          "the audio stream did not start".to_owned(),
        ));
      }
    });

    stream.play();
    *self.active.borrow_mut() = Active {
      stream: Some(stream),
      watchdog: Some(watchdog),
      on_failure: Some(Rc::new(Box::new(on_failure))),
    };
    Ok(())
  }

  /// Stops the current sound, if any.
  pub fn stop(&self) {
    let previous = std::mem::take(&mut *self.active.borrow_mut());
    if let Some(watchdog) = previous.watchdog {
      watchdog.remove();
    }
    if let Some(stream) = previous.stream {
      stream.pause();
      stream.clear();
    }
  }

  pub fn is_playing(&self) -> bool {
    self.active.borrow().stream.is_some()
  }

  pub fn set_volume(&self, volume: f64) {
    if let Some(stream) = &self.active.borrow().stream {
      stream.set_volume(volume.clamp(0.0, 1.0));
    }
  }

  fn fail(&self, error: SoundError) {
    let handler = self.active.borrow_mut().on_failure.take();
    log::error!("sound playback failed: {error}");
    self.stop();
    if let Some(handler) = handler {
      handler(&error);
    }
  }

  fn downgrade(&self) -> WeakPlayer {
    WeakPlayer(Rc::downgrade(&self.active))
  }
}

struct WeakPlayer(std::rc::Weak<RefCell<Active>>);

impl WeakPlayer {
  fn upgrade(&self) -> Option<SoundPlayer> {
    self.0.upgrade().map(|active| SoundPlayer { active })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn bundled_sound_lives_in_the_sounds_directory() {
    let path = bundled_sound(BELL_FILE);
    assert!(path.ends_with(BELL_FILE));
    assert_eq!(path.parent(), Some(sounds_dir().as_path()));
  }

  #[test]
  fn packaged_sounds_exist_in_the_source_tree() {
    // `cargo test` points ARGVUS_SOUNDS_DIR at the package payload.
    for file_name in [BELL_FILE, "argvus-chime.oga", "argvus-beacon.oga"] {
      assert!(bundled_sound(file_name).is_file(), "{file_name} is missing");
    }
  }
}
