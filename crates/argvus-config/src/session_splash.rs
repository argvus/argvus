//! Loading splash shown while a configuration reload runs.
//!
//! The splash is an optional cover. It starts once the compositor accepts
//! connections, waits for the splash's own readiness file, and records its pid
//! so the reload that started it can stop it. Failing to start the splash never
//! fails the reload.

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::{FileTypeExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread::sleep;
use std::time::Duration;

/// Which flow started a splash. Each owner records its pid in its own file, so
/// a flow stops only the splash it started.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Owner {
  /// The session splash, started by `argvus-sessionctl` during a reload.
  Session,
  /// The overlay of a theme switch.
  Transition,
}

impl Owner {
  fn pid_file_name(self) -> &'static str {
    match self {
      Owner::Session => "argvus-loading-theme.pid",
      Owner::Transition => "argvus-loading-theme-transition.pid",
    }
  }
}

/// Every owner, so that a second splash is never started over a running one.
const OWNERS: [Owner; 2] = [Owner::Session, Owner::Transition];
/// The kernel keeps only the first 15 bytes of a process name in `/proc/<pid>/comm`.
const COMM_LENGTH: usize = 15;
/// How long `stop` waits for a splash to exit after SIGTERM before SIGKILL.
const EXIT_WAIT_STEPS: u32 = 100;
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

/// Where a splash started by `owner` records its pid.
pub fn pid_file(environment: &SplashEnvironment, owner: Owner) -> PathBuf {
  environment.runtime_dir().join(owner.pid_file_name())
}

fn read_pid(path: &Path) -> Option<i32> {
  let text = fs::read_to_string(path).ok()?;
  let first = text.lines().next()?;
  if first.is_empty() || !first.bytes().all(|byte| byte.is_ascii_digit()) {
    return None;
  }
  first.parse::<i32>().ok().filter(|pid| *pid > 0)
}

/// Whether a process with this pid exists. A zombie still counts: it has
/// exited but nobody has reaped it yet.
fn process_alive(pid: i32) -> bool {
  // SAFETY: signal 0 only checks that the process exists and sends nothing.
  unsafe { libc::kill(pid, 0) == 0 }
}

/// Whether `pid` runs a process named `name`. The name is compared with the
/// kernel's truncated `comm`. It is only a fallback for pid files written before
/// the start time was recorded: a script's `comm` is its interpreter's name, so
/// the name does not identify a splash that is a shell script.
fn is_named(pid: i32, name: &str) -> bool {
  let expected: String = name.chars().take(COMM_LENGTH).collect();
  fs::read_to_string(format!("/proc/{pid}/comm"))
    .map(|comm| comm.trim_end() == expected)
    .unwrap_or(false)
}

/// The kernel's start time of `pid` (field 22 of `/proc/<pid>/stat`, in clock
/// ticks since boot). A recycled pid has a different start time.
pub(crate) fn process_start_time(pid: i32) -> Option<u64> {
  let stat = fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
  // The command name is in parentheses and may contain spaces, so the fields are
  // counted from the last closing parenthesis.
  let after_name = stat.rsplit_once(')')?.1;
  // After `)` the first field is the state, which is field 3; start time is field 22.
  after_name.split_whitespace().nth(19)?.parse().ok()
}

/// The start time recorded on the pid file's second line, when present.
fn read_start_time(path: &Path) -> Option<u64> {
  fs::read_to_string(path)
    .ok()?
    .lines()
    .nth(1)?
    .trim()
    .parse()
    .ok()
}

/// Waits for `pid` to end, reaping it when it is a child of this process, and
/// returns whether it ended within `steps` polls of 50 ms.
fn wait_until_exited(pid: i32, steps: u32) -> bool {
  for _ in 0..=steps {
    let mut status: libc::c_int = 0;
    // SAFETY: WNOHANG reads the state of `pid` without blocking, and reaps it
    // only when it is a child of this process.
    let reaped = unsafe { libc::waitpid(pid, &mut status, libc::WNOHANG) };
    if reaped == pid {
      return true;
    }
    let not_our_child =
      reaped == -1 && std::io::Error::last_os_error().raw_os_error() == Some(libc::ECHILD);
    if not_our_child && !process_alive(pid) {
      return true;
    }
    sleep(Duration::from_millis(50));
  }
  false
}

/// Ends the splash `pid`: SIGTERM so it can clean up, then SIGKILL when it
/// ignores that for the whole wait.
fn terminate(pid: i32) {
  // SAFETY: the caller has checked that `pid` is the splash it started.
  unsafe {
    libc::kill(pid, libc::SIGTERM);
  }
  if !wait_until_exited(pid, EXIT_WAIT_STEPS) {
    // SAFETY: as above; SIGKILL cannot be ignored.
    unsafe {
      libc::kill(pid, libc::SIGKILL);
    }
    wait_until_exited(pid, EXIT_WAIT_STEPS / 10);
  }
}

/// The pid recorded for `owner`, when that splash is still running. A stale
/// pid file is removed, as the session script did.
fn running_pid(environment: &SplashEnvironment, owner: Owner) -> Option<i32> {
  let path = pid_file(environment, owner);
  match read_pid(&path) {
    Some(pid) if process_alive(pid) => Some(pid),
    _ => {
      let _ = fs::remove_file(&path);
      None
    }
  }
}

/// Whether a splash started by any owner is still running.
pub fn active(environment: &SplashEnvironment) -> bool {
  OWNERS
    .iter()
    .any(|owner| running_pid(environment, *owner).is_some())
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

/// One splash start. The session splash waits for the compositor, shows an
/// opaque spinner from its first frame and outlives the helper that starts it.
/// The overlay of a theme switch already runs inside a live session, so it does
/// not wait, fades in and passes the theme's colours.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SplashRequest<'a> {
  pub theme: &'a str,
  pub colors: Option<SplashColors>,
  pub await_compositor: bool,
  pub spinner_only: bool,
  pub detached: bool,
  pub owner: Owner,
}

impl<'a> SplashRequest<'a> {
  /// The request the session reload makes.
  pub fn session(theme: &'a str) -> Self {
    Self {
      theme,
      colors: None,
      await_compositor: true,
      spinner_only: true,
      detached: true,
      owner: Owner::Session,
    }
  }
}

/// The arguments the splash binary receives for `request`.
fn splash_arguments(request: &SplashRequest, ready_file: &Path) -> Vec<OsString> {
  let mut arguments: Vec<OsString> = vec!["--theme".into(), request.theme.into()];
  if let Some(colors) = &request.colors {
    arguments.extend([
      "--background".into(),
      colors.background.as_str().into(),
      "--foreground".into(),
      colors.foreground.as_str().into(),
      "--accent".into(),
      colors.accent.as_str().into(),
    ]);
  }
  if request.spinner_only {
    arguments.push("--spinner-only".into());
  }
  if request.detached {
    arguments.push("--detached".into());
  }
  arguments.extend(["--ready-file".into(), ready_file.into()]);
  arguments
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
  // stdout is not inherited: the caller reads it to learn whether this call
  // started the splash, and the splash lives on after the caller returns.
  // A pipe held open by the splash would block that read until the splash
  // exits. stderr stays attached to the unit's log.
  let Ok(mut child) = Command::new(binary)
    .args(splash_arguments(request, &ready_file))
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
    // The child was spawned by this process, so `terminate` also reaps it.
    terminate(child.id() as i32);
    return StartOutcome::NotReady;
  }
  // The splash outlives this process; its pid is what `stop` will signal. Line
  // two is its start time, which identifies the splash even when the binary is a
  // script, whose process name is its interpreter's.
  let record = match process_start_time(child.id() as i32) {
    Some(start_time) => format!("{}\n{start_time}\n", child.id()),
    None => format!("{}\n", child.id()),
  };
  if fs::write(pid_file(environment, request.owner), record).is_ok() {
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

/// Stops the splash that `owner` started, if it is still running.
///
/// The pid file is removed first, so a failed stop does not leave a stale
/// record behind. The pid is signalled only while `/proc/<pid>/comm` still names
/// the splash binary, so a recycled pid is left alone. The splash gets SIGTERM,
/// then SIGKILL if it ignores that.
pub fn stop(environment: &SplashEnvironment, owner: Owner) {
  let binary_name = crate::session_loading::splash_binary()
    .file_name()
    .map(|name| name.to_string_lossy().into_owned())
    .unwrap_or_default();
  stop_named(environment, owner, &binary_name);
}

/// `stop` for a splash whose process name is `name`.
fn stop_named(environment: &SplashEnvironment, owner: Owner, name: &str) {
  let path = pid_file(environment, owner);
  let pid = read_pid(&path);
  let start_time = read_start_time(&path);
  let _ = fs::remove_file(&path);
  let Some(pid) = pid else {
    return;
  };
  let is_the_splash = match start_time {
    Some(recorded) => process_start_time(pid) == Some(recorded),
    None => is_named(pid, name),
  };
  if !is_the_splash {
    // The recorded pid is gone or now belongs to another process.
    return;
  }
  terminate(pid);
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
        theme: "github-light",
        colors: None,
        await_compositor: false,
        spinner_only: true,
        detached: true,
        owner: Owner::Session,
      },
    );
    assert_eq!(outcome, StartOutcome::Unavailable);
    assert!(!pid_file(&environment, Owner::Session).exists());
    let _ = fs::remove_dir_all(runtime);
  }

  #[test]
  fn a_running_splash_is_left_alone() {
    let runtime =
      std::env::temp_dir().join(format!("argvus-splash-running-{}", std::process::id()));
    let _ = fs::create_dir_all(&runtime);
    let environment = environment_under(&runtime);
    // This test process is alive, so the pid file names a running splash.
    fs::write(
      pid_file(&environment, Owner::Session),
      format!("{}\n", std::process::id()),
    )
    .expect("pid file");
    let outcome = start_with(
      &environment,
      &runtime.join("no-such-splash"),
      &SplashRequest::session("github-light"),
    );
    assert_eq!(outcome, StartOutcome::AlreadyRunning);
    let _ = fs::remove_dir_all(runtime);
  }

  #[test]
  fn session_splash_is_opaque_and_detached_while_the_overlay_is_not() {
    let ready = Path::new("/run/ready");
    let session = splash_arguments(&SplashRequest::session("github-light"), ready);
    assert!(session.contains(&OsString::from("--spinner-only")));
    assert!(session.contains(&OsString::from("--detached")));

    let overlay = SplashRequest {
      theme: "github-light",
      colors: None,
      await_compositor: false,
      spinner_only: false,
      detached: false,
      owner: Owner::Transition,
    };
    let arguments = splash_arguments(&overlay, ready);
    assert!(!arguments.contains(&OsString::from("--spinner-only")));
    assert!(!arguments.contains(&OsString::from("--detached")));
  }

  #[test]
  fn stale_pid_file_is_removed_and_not_active() {
    let runtime = std::env::temp_dir().join(format!("argvus-splash-stale-{}", std::process::id()));
    let _ = fs::create_dir_all(&runtime);
    let environment = SplashEnvironment {
      values: BTreeMap::from([("XDG_RUNTIME_DIR".to_string(), runtime.display().to_string())]),
    };
    // A pid far above any real process: it cannot be alive.
    fs::write(pid_file(&environment, Owner::Session), "2147483646\n").expect("pid file");
    assert!(!active(&environment));
    assert!(!pid_file(&environment, Owner::Session).exists());
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

  /// Copies `source` to `directory/name`, so the kernel reports `name` as the
  /// process name, and starts it with `arguments`.
  fn spawn_named(
    directory: &Path,
    source: &str,
    name: &str,
    arguments: &[&str],
  ) -> std::process::Child {
    let program = directory.join(name);
    fs::copy(source, &program).expect("copy test program");
    // Another test may fork while this copy is still open for writing; the
    // child then holds the descriptor, and exec fails with ETXTBSY until it closes.
    for _ in 0..50 {
      match Command::new(&program)
        .args(arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
      {
        Ok(child) => {
          wait_for_exec(child.id(), name);
          return child;
        }
        Err(error) if error.raw_os_error() == Some(libc::ETXTBSY) => {
          sleep(Duration::from_millis(20));
        }
        Err(error) => panic!("spawn test program: {error}"),
      }
    }
    panic!("spawn test program: still busy after retries");
  }

  /// Waits until `pid` runs a program named `name`. Identifying a splash by its
  /// name is only meaningful once the exec has replaced the test-binary image.
  fn wait_for_exec(pid: u32, name: &str) {
    let expected: String = name.chars().take(COMM_LENGTH).collect();
    for _ in 0..400 {
      let comm = fs::read_to_string(format!("/proc/{pid}/comm")).unwrap_or_default();
      if comm.trim_end() == expected {
        return;
      }
      sleep(Duration::from_millis(5));
    }
    panic!("child {pid} did not exec {name} in time");
  }

  /// Records `pid` the way the session does: the pid, then its start time, which
  /// identifies the process even after an exec changes its name.
  fn write_recorded_pid(environment: &SplashEnvironment, owner: Owner, pid: u32) {
    let start_time = process_start_time(pid as i32).expect("start time of the child");
    fs::write(
      pid_file(environment, owner),
      format!("{pid}\n{start_time}\n"),
    )
    .expect("pid file");
  }

  fn test_runtime(label: &str) -> PathBuf {
    let runtime =
      std::env::temp_dir().join(format!("argvus-splash-{label}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&runtime);
    fs::create_dir_all(&runtime).expect("runtime dir");
    runtime
  }

  #[test]
  fn stop_reaps_the_splash_instead_of_waiting_for_a_zombie() {
    let runtime = test_runtime("reap");
    let environment = environment_under(&runtime);
    let mut child = spawn_named(&runtime, "/bin/sleep", "splash-test", &["60"]);
    write_recorded_pid(&environment, Owner::Transition, child.id());

    let started = std::time::Instant::now();
    stop_named(&environment, Owner::Transition, "splash-test");
    // Without reaping, `process_alive` keeps seeing the zombie and the wait runs for five seconds.
    assert!(
      // The splash sleeps for 60 s, so this bound only has to rule out a
      // five-second wait; it leaves room for a loaded machine.
      started.elapsed() < Duration::from_secs(10),
      "{:?}",
      started.elapsed()
    );
    assert!(!pid_file(&environment, Owner::Transition).exists());
    assert!(
      child.try_wait().is_err(),
      "stop did not reap the splash child"
    );
    let _ = fs::remove_dir_all(runtime);
  }

  #[test]
  fn legacy_pid_file_is_identified_by_process_name() {
    let runtime = test_runtime("legacy");
    let environment = environment_under(&runtime);
    let mut child = spawn_named(&runtime, "/bin/sleep", "splash-test", &["60"]);
    // A pid file written before the start time was recorded holds only the pid.
    fs::write(
      pid_file(&environment, Owner::Transition),
      format!("{}\n", child.id()),
    )
    .expect("pid file");

    stop_named(&environment, Owner::Transition, "splash-test");
    assert!(
      child.try_wait().is_err(),
      "stop did not reap the legacy splash child"
    );
    let _ = fs::remove_dir_all(runtime);
  }

  #[test]
  fn stop_does_not_signal_a_pid_that_is_not_the_splash() {
    let runtime = test_runtime("identity");
    let environment = environment_under(&runtime);
    let mut other = spawn_named(&runtime, "/bin/sleep", "other-process", &["60"]);
    fs::write(
      pid_file(&environment, Owner::Transition),
      format!("{}\n", other.id()),
    )
    .expect("pid file");

    stop_named(&environment, Owner::Transition, "splash-test");
    assert!(
      process_alive(other.id() as i32),
      "a recycled pid must survive"
    );
    assert!(!pid_file(&environment, Owner::Transition).exists());
    let _ = other.kill();
    let _ = other.wait();
    let _ = fs::remove_dir_all(runtime);
  }

  #[test]
  fn stop_escalates_to_sigkill_when_sigterm_is_ignored() {
    let runtime = test_runtime("sigkill");
    let environment = environment_under(&runtime);
    let mut child = spawn_named(
      &runtime,
      "/bin/sh",
      "splash-test",
      &["-c", "trap '' TERM; sleep 60"],
    );
    write_recorded_pid(&environment, Owner::Transition, child.id());

    stop_named(&environment, Owner::Transition, "splash-test");
    assert!(!process_alive(child.id() as i32), "SIGKILL ends the splash");
    let _ = child.kill();
    let _ = child.wait();
    let _ = fs::remove_dir_all(runtime);
  }

  #[test]
  fn a_recorded_start_time_identifies_a_script_splash() {
    // A script runs under its interpreter, so its process name is not the binary
    // name. The recorded start time is what identifies it.
    let runtime = test_runtime("start-time");
    let environment = environment_under(&runtime);
    let splash = spawn_named(&runtime, "/bin/sleep", "interpreter-name", &["60"]);
    let start_time = process_start_time(splash.id() as i32).expect("start time");
    fs::write(
      pid_file(&environment, Owner::Transition),
      format!("{}\n{start_time}\n", splash.id()),
    )
    .expect("pid file");

    stop_named(&environment, Owner::Transition, "splash-test");
    // `stop` reaped the child, so it no longer exists as a process at all.
    assert!(
      !process_alive(splash.id() as i32),
      "the recorded splash was stopped"
    );
    // `stop` already reaped the child; this only satisfies the Child handle.
    let mut splash = splash;
    let _ = splash.wait();
    let _ = fs::remove_dir_all(runtime);
  }

  #[test]
  fn a_recorded_start_time_that_no_longer_matches_is_left_alone() {
    let runtime = test_runtime("start-mismatch");
    let environment = environment_under(&runtime);
    let mut other = spawn_named(&runtime, "/bin/sleep", "other-process", &["60"]);
    let start_time = process_start_time(other.id() as i32).expect("start time");
    // The same pid with a different start time is a recycled pid.
    fs::write(
      pid_file(&environment, Owner::Transition),
      format!("{}\n{}\n", other.id(), start_time + 1),
    )
    .expect("pid file");

    stop_named(&environment, Owner::Transition, "splash-test");
    assert!(
      process_alive(other.id() as i32),
      "a recycled pid must survive"
    );
    let _ = other.kill();
    let _ = other.wait();
    let _ = fs::remove_dir_all(runtime);
  }

  #[test]
  fn each_owner_stops_only_its_own_splash() {
    let runtime = test_runtime("owners");
    let environment = environment_under(&runtime);
    let mut session = spawn_named(&runtime, "/bin/sleep", "splash-test", &["60"]);
    fs::write(
      pid_file(&environment, Owner::Session),
      format!("{}\n", session.id()),
    )
    .expect("pid file");

    stop_named(&environment, Owner::Transition, "splash-test");
    assert!(
      process_alive(session.id() as i32),
      "the session splash is not the transition's"
    );
    assert!(pid_file(&environment, Owner::Session).exists());
    let _ = session.kill();
    let _ = session.wait();
    let _ = fs::remove_dir_all(runtime);
  }

  #[test]
  fn a_transition_overlay_never_starts_over_a_session_splash() {
    let runtime = test_runtime("shared-guard");
    let environment = environment_under(&runtime);
    let mut session = spawn_named(&runtime, "/bin/sleep", "splash-test", &["60"]);
    fs::write(
      pid_file(&environment, Owner::Session),
      format!("{}\n", session.id()),
    )
    .expect("pid file");

    let outcome = start_with(
      &environment,
      &runtime.join("no-such-splash"),
      &SplashRequest {
        theme: "github-light",
        colors: None,
        await_compositor: false,
        spinner_only: false,
        detached: false,
        owner: Owner::Transition,
      },
    );
    assert_eq!(outcome, StartOutcome::AlreadyRunning);
    let _ = session.kill();
    let _ = session.wait();
    let _ = fs::remove_dir_all(runtime);
  }
}
