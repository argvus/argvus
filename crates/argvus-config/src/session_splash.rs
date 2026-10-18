//! Loading splash shown while a configuration reload runs.
//!
//! The splash is an optional cover. It starts once the compositor accepts
//! connections, waits for the splash's own readiness file, and records its pid
//! so the reload that started it can stop it. Failing to start the splash never
//! fails the reload.

use crate::session_loading;
use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::{FileTypeExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread::sleep;
use std::time::Duration;

const PID_FILE_NAME: &str = "argvus-loading-theme.pid";
/// Theme used when the effective configuration names none.
pub const DEFAULT_THEME: &str = "argvus-dark";
/// How many 100 ms waits for the compositor before giving up.
const COMPOSITOR_WAIT_STEPS: u32 = 50;
/// How many 50 ms waits for the splash to write its readiness file.
const READY_WAIT_STEPS: u32 = 40;
/// Compositor environment names the splash needs, adopted from systemd when
/// the calling process does not have them.
const ADOPTED_ENVIRONMENT: [&str; 5] = [
  "WAYLAND_DISPLAY",
  "DISPLAY",
  "XDG_RUNTIME_DIR",
  "HYPRLAND_INSTANCE_SIGNATURE",
  "HYPRLAND_CMD",
];

/// Environment for the splash and its readiness checks.
#[derive(Debug, Clone, Default)]
pub struct SplashEnvironment {
  values: BTreeMap<String, String>,
}

impl SplashEnvironment {
  /// Reads the process environment, then fills missing names from the
  /// systemd user environment. Empty values count as missing.
  pub fn resolve() -> Self {
    let mut values = BTreeMap::new();
    for name in ADOPTED_ENVIRONMENT {
      let value = std::env::var(name)
        .ok()
        .filter(|value| !value.is_empty())
        .or_else(|| systemd_value(name));
      if let Some(value) = value {
        values.insert(name.to_string(), value);
      }
    }
    Self { values }
  }

  fn get(&self, name: &str) -> Option<&str> {
    self.values.get(name).map(String::as_str)
  }

  /// The directory for runtime files: `XDG_RUNTIME_DIR`, or `/tmp`.
  fn runtime_dir(&self) -> PathBuf {
    PathBuf::from(self.get("XDG_RUNTIME_DIR").unwrap_or("/tmp"))
  }
}

/// Reads the last `name=value` line from `systemctl --user show-environment`.
fn systemd_value(name: &str) -> Option<String> {
  let output = Command::new("systemctl")
    .args(["--user", "show-environment"])
    .stderr(Stdio::null())
    .output()
    .ok()?;
  let prefix = format!("{name}=");
  String::from_utf8_lossy(&output.stdout)
    .lines()
    .rev()
    .find_map(|line| line.strip_prefix(prefix.as_str()))
    .filter(|value| !value.is_empty())
    .map(str::to_string)
}

/// Where the splash records its pid.
pub fn pid_file(environment: &SplashEnvironment) -> PathBuf {
  environment.runtime_dir().join(PID_FILE_NAME)
}

fn read_pid(path: &Path) -> Option<i32> {
  let text = fs::read_to_string(path).ok()?;
  let first = text.lines().next()?;
  if first.is_empty() || !first.bytes().all(|byte| byte.is_ascii_digit()) {
    return None;
  }
  first.parse::<i32>().ok().filter(|pid| *pid > 0)
}

/// Whether a process with this pid exists.
fn process_alive(pid: i32) -> bool {
  // SAFETY: signal 0 only checks that the process exists and sends nothing.
  unsafe { libc::kill(pid, 0) == 0 }
}

/// Whether a splash started earlier is still running. A stale pid file is
/// removed, as the session script did.
pub fn active(environment: &SplashEnvironment) -> bool {
  let path = pid_file(environment);
  match read_pid(&path) {
    Some(pid) if process_alive(pid) => true,
    _ => {
      let _ = fs::remove_file(&path);
      false
    }
  }
}

fn is_executable(path: &Path) -> bool {
  fs::metadata(path)
    .map(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
    .unwrap_or(false)
}

/// The compositor socket is ready when `WAYLAND_DISPLAY` names a socket.
fn wayland_socket_ready(environment: &SplashEnvironment) -> bool {
  let Some(display) = environment.get("WAYLAND_DISPLAY") else {
    return false;
  };
  if environment.get("XDG_RUNTIME_DIR").is_none() {
    return false;
  }
  let socket = if display.starts_with('/') {
    PathBuf::from(display)
  } else {
    environment.runtime_dir().join(display)
  };
  fs::metadata(socket)
    .map(|metadata| metadata.file_type().is_socket())
    .unwrap_or(false)
}

/// Hyprland answers `instances` once its IPC is up. Without a Hyprland
/// signature there is nothing to ask, so the check passes.
fn hyprland_ready(environment: &SplashEnvironment) -> bool {
  if environment.get("HYPRLAND_INSTANCE_SIGNATURE").is_none() {
    return true;
  }
  let Ok(mut child) = Command::new("hyprctl")
    .arg("instances")
    .envs(&environment.values)
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .spawn()
  else {
    // No hyprctl on PATH: nothing to wait for.
    return true;
  };
  for _ in 0..10 {
    match child.try_wait() {
      Ok(Some(status)) => return status.success(),
      Ok(None) => sleep(Duration::from_millis(100)),
      Err(_) => return false,
    }
  }
  let _ = child.kill();
  let _ = child.wait();
  false
}

/// Colours the themed overlay passes to the splash, as `#RRGGBB`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SplashColors {
  pub foreground: String,
  pub background: String,
  pub accent: String,
}

/// One splash start. The session splash waits for the compositor and keeps
/// the splash's own colours. The overlay of a theme switch already runs inside
/// a live session, so it does not wait and passes the theme's colours.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SplashRequest<'a> {
  pub theme: &'a str,
  pub colors: Option<SplashColors>,
  pub await_compositor: bool,
}

impl<'a> SplashRequest<'a> {
  /// The request the session reload makes.
  pub fn session(theme: &'a str) -> Self {
    Self {
      theme,
      colors: None,
      await_compositor: true,
    }
  }
}

/// The result of one start. Only `Started` leaves a splash running, and only
/// then does the pid file name it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartOutcome {
  Started,
  /// A splash recorded in the pid file is still running; it is left alone.
  AlreadyRunning,
  /// The splash binary is missing or not executable.
  Unavailable,
  /// The compositor did not accept connections in time.
  NoCompositor,
  /// The splash did not write its readiness file, or could not be launched.
  NotReady,
}

/// Starts the splash with the binary the session uses. Returns `true` only when
/// this call started it. Every failure returns `false` and leaves the reload to
/// continue.
pub fn start(environment: &SplashEnvironment, theme: &str) -> bool {
  start_with(
    environment,
    &session_loading::splash_binary(),
    &SplashRequest::session(theme),
  ) == StartOutcome::Started
}

/// Starts the splash with `binary` and the options of `request`.
pub fn start_with(
  environment: &SplashEnvironment,
  binary: &Path,
  request: &SplashRequest,
) -> StartOutcome {
  if active(environment) {
    return StartOutcome::AlreadyRunning;
  }
  if !is_executable(binary) {
    return StartOutcome::Unavailable;
  }
  let environment = if request.await_compositor {
    match wait_for_compositor() {
      Some(ready) => ready,
      None => return StartOutcome::NoCompositor,
    }
  } else {
    environment.clone()
  };
  let environment = &environment;

  let ready_file = environment
    .runtime_dir()
    .join(format!("argvus-loading-theme-ready.{}", std::process::id()));
  let _ = fs::remove_file(&ready_file);
  let mut command = Command::new(binary);
  command.arg("--theme").arg(request.theme);
  if let Some(colors) = &request.colors {
    command
      .arg("--background")
      .arg(&colors.background)
      .arg("--foreground")
      .arg(&colors.foreground)
      .arg("--accent")
      .arg(&colors.accent);
  }
  // stdout is not inherited: the caller reads it to learn whether this call
  // started the splash, and the splash lives on after the caller returns.
  // A pipe held open by the splash would block that read until the splash
  // exits. stderr stays attached to the unit's log.
  let Ok(mut child) = command
    .arg("--ready-file")
    .arg(&ready_file)
    .envs(&environment.values)
    .stdin(Stdio::null())
    .stdout(Stdio::null())
    .spawn()
  else {
    return StartOutcome::NotReady;
  };

  let mut waited = 0;
  while !is_ready(&ready_file) && matches!(child.try_wait(), Ok(None)) {
    if waited >= READY_WAIT_STEPS {
      break;
    }
    sleep(Duration::from_millis(50));
    waited += 1;
  }
  let ready = is_ready(&ready_file);
  let _ = fs::remove_file(&ready_file);
  if !ready {
    // SIGTERM, as the session script did, so the splash can clean up.
    // SAFETY: the pid belongs to the child this process just spawned.
    unsafe {
      libc::kill(child.id() as i32, libc::SIGTERM);
    }
    let _ = child.wait();
    return StartOutcome::NotReady;
  }
  // The splash outlives this process; its pid is what `stop` will signal.
  if fs::write(pid_file(environment), format!("{}\n", child.id())).is_ok() {
    StartOutcome::Started
  } else {
    StartOutcome::NotReady
  }
}

fn is_ready(ready_file: &Path) -> bool {
  fs::metadata(ready_file)
    .map(|metadata| metadata.len() > 0)
    .unwrap_or(false)
}

/// Waits until the compositor socket and Hyprland IPC answer. The environment
/// is re-read on every attempt, because systemd may publish it while waiting.
/// Returns the environment that was ready, or `None` after the wait.
fn wait_for_compositor() -> Option<SplashEnvironment> {
  for _ in 0..COMPOSITOR_WAIT_STEPS {
    let environment = SplashEnvironment::resolve();
    if wayland_socket_ready(&environment) && hyprland_ready(&environment) {
      return Some(environment);
    }
    sleep(Duration::from_millis(100));
  }
  None
}

/// Stops the splash started by an earlier call, if it is still running.
///
/// The pid file is removed first, so a failed stop does not leave a stale
/// record behind. Waits up to five seconds for the process to exit.
pub fn stop(environment: &SplashEnvironment) {
  let path = pid_file(environment);
  let pid = read_pid(&path);
  let _ = fs::remove_file(&path);
  let Some(pid) = pid else {
    return;
  };
  // SAFETY: sends SIGTERM to a pid recorded by `start`.
  unsafe {
    libc::kill(pid, libc::SIGTERM);
  }
  for _ in 0..100 {
    if !process_alive(pid) {
      return;
    }
    sleep(Duration::from_millis(50));
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn pid_must_be_a_positive_number() {
    let dir = std::env::temp_dir().join(format!("argvus-splash-pid-{}", std::process::id()));
    let _ = fs::create_dir_all(&dir);
    let path = dir.join("pid");
    for (content, expected) in [
      ("1234\n", Some(1234)),
      ("12x\n", None),
      ("\n", None),
      ("0\n", None),
      ("", None),
    ] {
      fs::write(&path, content).expect("pid file");
      assert_eq!(read_pid(&path), expected, "{content:?}");
    }
    let _ = fs::remove_dir_all(dir);
  }

  fn environment_under(runtime: &Path) -> SplashEnvironment {
    SplashEnvironment {
      values: BTreeMap::from([("XDG_RUNTIME_DIR".to_string(), runtime.display().to_string())]),
    }
  }

  #[test]
  fn missing_binary_is_unavailable_and_writes_nothing() {
    let runtime =
      std::env::temp_dir().join(format!("argvus-splash-missing-{}", std::process::id()));
    let _ = fs::create_dir_all(&runtime);
    let environment = environment_under(&runtime);
    let outcome = start_with(
      &environment,
      &runtime.join("no-such-splash"),
      &SplashRequest {
        theme: "frost",
        colors: None,
        await_compositor: false,
      },
    );
    assert_eq!(outcome, StartOutcome::Unavailable);
    assert!(!pid_file(&environment).exists());
    let _ = fs::remove_dir_all(runtime);
  }

  #[test]
  fn a_running_splash_is_left_alone() {
    let runtime =
      std::env::temp_dir().join(format!("argvus-splash-running-{}", std::process::id()));
    let _ = fs::create_dir_all(&runtime);
    let environment = environment_under(&runtime);
    // This test process is alive, so the pid file names a running splash.
    fs::write(pid_file(&environment), format!("{}\n", std::process::id())).expect("pid file");
    let outcome = start_with(
      &environment,
      &runtime.join("no-such-splash"),
      &SplashRequest::session("frost"),
    );
    assert_eq!(outcome, StartOutcome::AlreadyRunning);
    let _ = fs::remove_dir_all(runtime);
  }

  #[test]
  fn stale_pid_file_is_removed_and_not_active() {
    let runtime = std::env::temp_dir().join(format!("argvus-splash-stale-{}", std::process::id()));
    let _ = fs::create_dir_all(&runtime);
    let environment = SplashEnvironment {
      values: BTreeMap::from([("XDG_RUNTIME_DIR".to_string(), runtime.display().to_string())]),
    };
    // A pid far above any real process: it cannot be alive.
    fs::write(pid_file(&environment), "2147483646\n").expect("pid file");
    assert!(!active(&environment));
    assert!(!pid_file(&environment).exists());
    let _ = fs::remove_dir_all(runtime);
  }

  #[test]
  fn wayland_socket_must_be_a_socket() {
    let runtime = std::env::temp_dir().join(format!("argvus-splash-socket-{}", std::process::id()));
    let _ = fs::create_dir_all(&runtime);
    let environment = SplashEnvironment {
      values: BTreeMap::from([
        ("XDG_RUNTIME_DIR".to_string(), runtime.display().to_string()),
        ("WAYLAND_DISPLAY".to_string(), "wayland-test".to_string()),
      ]),
    };
    assert!(!wayland_socket_ready(&environment), "no socket yet");
    fs::write(runtime.join("wayland-test"), "").expect("regular file");
    assert!(
      !wayland_socket_ready(&environment),
      "a regular file is not a socket"
    );
    let _ = fs::remove_dir_all(runtime);
  }
}
