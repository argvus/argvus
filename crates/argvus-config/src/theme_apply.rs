//! The transaction of a theme switch.
//!
//! The switch holds an exclusive lock while `theme_body` commits the theme,
//! renders its build and writes the consumer files, and while the projection
//! writes the generated configuration. The running desktop then takes the new
//! theme in place (`theme_runtime`): no overlay covers the screen and no service
//! stops, so the switch is seen as the colours changing.

use crate::session_paths::SessionPaths;
use crate::session_splash;
use crate::theme_body;
use crate::theme_resolve;
use crate::theme_runtime;
use std::env;
use std::fs::{self, OpenOptions};
use std::io;
use std::os::fd::AsRawFd;
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::thread::sleep;
use std::time::{Duration, Instant};

/// The marker a running switch keeps beside its lock. The configuration reload
/// that this switch starts reads it and leaves the theme to this switch, instead
/// of starting a second switch that would wait on the lock held here.
const TRANSACTION_MARKER: &str = "theme-switch.active";

/// How long a switch waits for the lock while another switch holds it.
const LOCK_WAIT_LIMIT: Duration = Duration::from_secs(60);

/// How long the projection may take before this switch gives up on it.
const PROJECTION_WAIT_LIMIT: Duration = Duration::from_secs(60);

/// Interval between checks of the lock and of the reload child.
const POLL_INTERVAL: Duration = Duration::from_millis(50);

/// Runs the transaction for `theme_argument` and returns the exit status.
/// `layout_variant` is the Sticky/Float variant the theme id resolves with.
pub fn run(paths: &SessionPaths, theme_argument: &str, layout_variant: &str) -> i32 {
  // Refuse an unsafe id before any service stops or any file is written.
  if !theme_argument.is_empty() && !theme_resolve::is_valid_theme_id(theme_argument) {
    eprintln!("argvus-config: invalid theme id '{theme_argument}'");
    return 1;
  }
  let theme = theme_resolve::resolve_theme(theme_argument, layout_variant);
  if theme.is_empty() {
    return run_menu(paths);
  }
  let runtime = env::var("ARGVUS_NO_RUNTIME").as_deref() != Ok("1");
  let lock_path = cache_home().join("theme-switch.lock");
  if let Some(parent) = lock_path.parent() {
    let _ = fs::create_dir_all(parent);
  }

  // The whole transaction runs under the lock. The bindings below are dropped
  // in reverse order: marker, then lock.
  let _lock = match acquire_lock(&lock_path) {
    Ok(lock) => lock,
    Err(error) => {
      eprintln!("argvus-config: theme lock: {error}");
      return 1;
    }
  };
  let _transaction = match TransactionMarker::record(cache_home().join(TRANSACTION_MARKER)) {
    Ok(marker) => marker,
    Err(error) => {
      eprintln!("argvus-config: theme transaction: {error}");
      return 1;
    }
  };
  // The per-theme copies of older switches are not read any more: they move
  // to a backup once, before this switch writes anything.
  match crate::theme_migration::migrate(paths) {
    Ok(Some(backup)) => crate::i18n::print(
      "theme.copies_moved",
      &[format!("path={}", backup.display())],
      crate::i18n::Stream::Stderr,
    ),
    Ok(None) => {}
    Err(error) => eprintln!("argvus-config: theme copies: {error}"),
  }
  // Session preparation and the configuration service run their own
  // projection and runtime steps; a switch started by them only writes files.
  let projecting = env::var("ARGVUS_PROJECTING").as_deref() == Ok("1");
  let config_service = env::var("ARGVUS_CONFIG_SERVICE").as_deref() == Ok("1");
  let live = runtime && !projecting && !config_service;
  let before = theme_runtime::Before::capture(paths);
  let mut status = theme_body::run(paths, &theme, runtime, live);
  if live && status == 0 {
    // The projection sees this switch's marker and leaves the theme to it.
    match run_bounded(
      Command::new("argvus")
        .args(["config", "project"])
        .stdout(Stdio::null())
        .stderr(Stdio::null()),
      PROJECTION_WAIT_LIMIT,
    ) {
      Ok(result) if result.success() => {}
      Ok(result) => status = exit_code(result),
      Err(error) => {
        eprintln!("argvus-config: projection: {error}");
        status = 1;
      }
    }
    theme_runtime::apply(paths, &before, &theme);
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

/// The record of a running switch: the pid of its process and that process's
/// start time. The file is removed when the switch ends.
struct TransactionMarker {
  path: PathBuf,
}

impl TransactionMarker {
  /// Records this process as the running switch. The start time is stored with
  /// the pid, so a recycled pid is not taken for a switch that is still running.
  fn record(path: PathBuf) -> io::Result<Self> {
    let pid = std::process::id();
    let start_time = session_splash::process_start_time(pid as i32)
      .ok_or_else(|| io::Error::other("cannot read the start time of this process"))?;
    fs::write(&path, format!("{pid}\n{start_time}\n"))?;
    Ok(Self { path })
  }
}

impl Drop for TransactionMarker {
  fn drop(&mut self) {
    let _ = fs::remove_file(&self.path);
  }
}

/// Whether a theme switch is running right now, for the projection that a
/// switch's own reload starts. A marker left by a switch that died is ignored.
pub(crate) fn transaction_in_progress() -> bool {
  transaction_recorded_at(&cache_home().join(TRANSACTION_MARKER))
}

/// Whether the marker at `path` names a process that is still the switch it
/// recorded, by pid and start time.
fn transaction_recorded_at(path: &Path) -> bool {
  let Ok(text) = fs::read_to_string(path) else {
    return false;
  };
  let mut lines = text.lines();
  let pid = lines
    .next()
    .and_then(|line| line.trim().parse::<i32>().ok());
  let recorded_start = lines
    .next()
    .and_then(|line| line.trim().parse::<u64>().ok());
  match (pid, recorded_start) {
    (Some(pid), Some(recorded_start)) => {
      session_splash::process_start_time(pid) == Some(recorded_start)
    }
    _ => false,
  }
}

/// Takes the exclusive lock, waiting at most `LOCK_WAIT_LIMIT` for a switch that
/// holds it. The file is opened close-on-exec, so the body does not keep the lock
/// alive.
fn acquire_lock(path: &Path) -> io::Result<fs::File> {
  let file = OpenOptions::new()
    .create(true)
    .write(true)
    .truncate(false)
    .open(path)?;
  let started = Instant::now();
  loop {
    // SAFETY: the descriptor belongs to the file opened above and stays valid
    // for the lock's lifetime.
    let result = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
    if result == 0 {
      return Ok(file);
    }
    let error = io::Error::last_os_error();
    if error.kind() != io::ErrorKind::WouldBlock {
      return Err(error);
    }
    if started.elapsed() >= LOCK_WAIT_LIMIT {
      return Err(io::Error::new(
        io::ErrorKind::TimedOut,
        "another theme switch is still running",
      ));
    }
    sleep(POLL_INTERVAL);
  }
}

/// Runs `command` and waits for it at most `limit`. A child still running at the
/// limit is killed and reported as `TimedOut`. Killing `systemctl` does not stop a
/// job that systemd already started, so this bounds the wait, not the job.
fn run_bounded(command: &mut Command, limit: Duration) -> io::Result<ExitStatus> {
  let mut child = command.spawn()?;
  let started = Instant::now();
  loop {
    if let Some(status) = child.try_wait()? {
      return Ok(status);
    }
    if started.elapsed() >= limit {
      // The child is ours, so waiting after the kill reaps it.
      let _ = child.kill();
      let _ = child.wait();
      return Err(io::Error::new(
        io::ErrorKind::TimedOut,
        format!("still running after {}s", limit.as_secs()),
      ));
    }
    sleep(POLL_INTERVAL);
  }
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

#[cfg(test)]
mod tests {
  use super::*;

  /// A scratch directory for one test. Each test uses its own name, and the
  /// directory is removed by the test that created it.
  fn scratch_dir(name: &str) -> PathBuf {
    let dir = env::temp_dir().join(format!(
      "argvus-theme-transaction-{name}-{}",
      std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("scratch directory");
    dir
  }

  #[test]
  fn recorded_switch_is_seen_until_its_marker_is_removed() {
    let dir = scratch_dir("recorded");
    let path = dir.join(TRANSACTION_MARKER);
    let marker = TransactionMarker::record(path.clone()).expect("marker is written");
    assert!(transaction_recorded_at(&path));
    drop(marker);
    assert!(!path.exists());
    assert!(!transaction_recorded_at(&path));
    let _ = fs::remove_dir_all(&dir);
  }

  #[test]
  fn marker_with_a_recycled_start_time_is_not_a_running_switch() {
    let dir = scratch_dir("recycled");
    let path = dir.join(TRANSACTION_MARKER);
    // A start time of zero never matches a live process, so the pid is reused.
    fs::write(&path, format!("{}\n0\n", std::process::id())).expect("marker is written");
    assert!(!transaction_recorded_at(&path));
    let _ = fs::remove_dir_all(&dir);
  }

  #[test]
  fn marker_of_a_dead_process_is_not_a_running_switch() {
    let dir = scratch_dir("dead");
    let path = dir.join(TRANSACTION_MARKER);
    // No process has this pid, so its start time cannot be read.
    fs::write(&path, format!("{}\n1\n", i32::MAX)).expect("marker is written");
    assert!(!transaction_recorded_at(&path));
    let _ = fs::remove_dir_all(&dir);
  }

  #[test]
  fn bounded_run_reports_a_child_that_outlives_the_limit() {
    let status = run_bounded(&mut Command::new("true"), Duration::from_secs(5))
      .expect("true runs to completion");
    assert!(status.success());

    let timed_out = run_bounded(Command::new("sleep").arg("30"), Duration::from_millis(100));
    assert_eq!(
      timed_out.expect_err("sleep outlives the limit").kind(),
      io::ErrorKind::TimedOut
    );
  }
}
