//! The transaction of a theme switch.
//!
//! The switch holds an exclusive lock, shows the loading overlay, and stops the
//! services whose files it rewrites. When the change is applied, it reloads the
//! configuration and starts those services again. The body that rewrites the
//! theme files is `theme_body`, and it runs here, under the lock.

use crate::session_paths::SessionPaths;
use crate::session_splash::StartOutcome;
use crate::theme_body;
use crate::theme_resolve;
use crate::theme_transition::{self, overlay_warning};
use std::env;
use std::fs::{self, OpenOptions};
use std::os::fd::AsRawFd;
use std::os::unix::process::ExitStatusExt;
use std::path::PathBuf;
use std::process::{Command, ExitStatus, Stdio};

/// Runs the transaction for `theme_argument` and returns the exit status.
/// `layout_variant` is the Sticky/Float variant the theme id resolves with.
pub fn run(paths: &SessionPaths, theme_argument: &str, layout_variant: &str) -> i32 {
  let theme = theme_resolve::resolve_theme(theme_argument, layout_variant);
  if theme.is_empty() {
    return run_menu(paths);
  }
  let runtime = env::var("ARGVUS_NO_RUNTIME").as_deref() != Ok("1");
  let systemctl = on_path("systemctl");
  let recorded_units = if systemctl {
    theme_transition::active_units().join(" ")
  } else {
    String::new()
  };

  let lock_path = cache_home().join("theme-switch.lock");
  if let Some(parent) = lock_path.parent() {
    let _ = fs::create_dir_all(parent);
  }

  let overlay_started = runtime && start_overlay(&theme);

  let mut status = match acquire_lock(&lock_path) {
    Ok(lock) => {
      let status = theme_body::run(paths, &theme, runtime);
      // Dropping the lock releases it; the body never inherits the file.
      drop(lock);
      status
    }
    Err(error) => {
      eprintln!("argvus-config: theme lock: {error}");
      1
    }
  };

  // Session preparation materializes files with NO_RUNTIME=1. Do not reload
  // systemd components while their startup transaction is still in progress.
  let projecting = env::var("ARGVUS_PROJECTING").as_deref() == Ok("1");
  let config_service = env::var("ARGVUS_CONFIG_SERVICE").as_deref() == Ok("1");
  if status == 0 && runtime && !projecting && !config_service && systemctl {
    let reload = Command::new("systemctl")
      .args(["--user", "reload", "argvus-config.service"])
      .stdout(Stdio::null())
      .stderr(Stdio::null())
      .status();
    match reload {
      Ok(result) if result.success() => {}
      Ok(result) => status = exit_code(result),
      Err(_) => status = 1,
    }
    // If the theme did not change, the body stops the services without
    // restarting them. Restart the ones that were active, so the wallpaper
    // and the other components come back.
    let _ = theme_transition::restart_units(&recorded_units);
  }

  if overlay_started {
    theme_transition::stop_overlay();
  }
  status
}

/// The theme menu, when no theme was named. The shell ran it with `exec`.
fn run_menu(paths: &SessionPaths) -> i32 {
  let menu = paths.read_config("appearance/sh/theme-menu.sh");
  Command::new("sh")
    .arg(menu)
    .status()
    .map(exit_code)
    .unwrap_or(127)
}

/// Shows the overlay for `theme` and reports whether this call started it.
fn start_overlay(theme: &str) -> bool {
  let outcome = theme_transition::start_overlay(theme);
  if let Some(message) = overlay_warning(outcome) {
    eprintln!("{message}");
  }
  outcome == StartOutcome::Started
}

/// Takes the exclusive lock, waiting for a switch that holds it. The file is
/// opened close-on-exec, so the body does not keep the lock alive.
fn acquire_lock(path: &PathBuf) -> std::io::Result<std::fs::File> {
  let file = OpenOptions::new()
    .create(true)
    .write(true)
    .truncate(false)
    .open(path)?;
  // SAFETY: the descriptor belongs to the file opened above and stays valid
  // for the lock's lifetime.
  let result = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX) };
  if result != 0 {
    return Err(std::io::Error::last_os_error());
  }
  Ok(file)
}

/// The exit status of a child, with a signal reported as the shell did (128 + n).
fn exit_code(status: ExitStatus) -> i32 {
  status
    .code()
    .unwrap_or_else(|| 128 + status.signal().unwrap_or(0))
}

/// `ARGVUS_CACHE_HOME`, or `XDG_CACHE_HOME/argvus` (default `~/.cache/argvus`).
fn cache_home() -> PathBuf {
  if let Some(path) = env::var_os("ARGVUS_CACHE_HOME").filter(|value| !value.is_empty()) {
    return PathBuf::from(path);
  }
  let base = env::var_os("XDG_CACHE_HOME")
    .filter(|value| !value.is_empty())
    .map(PathBuf::from)
    .unwrap_or_else(|| PathBuf::from(env::var_os("HOME").unwrap_or_default()).join(".cache"));
  base.join("argvus")
}

fn on_path(program: &str) -> bool {
  env::var_os("PATH")
    .map(|paths| env::split_paths(&paths).any(|dir| dir.join(program).is_file()))
    .unwrap_or(false)
}
