//! Readiness handshake for the graphical login transition.
//!
//! The login TUI cannot return immediately after greetd accepts a session:
//! its terminal wrapper would tear down the last visible frame first. This
//! module starts the spinner surface (the Wayland mode of the handoff helper)
//! inside the still-running greeter compositor, waits for its first presented
//! frame, and only then allows the TUI guard to enter its handoff-preserving
//! exit mode.

use anyhow::{Context, bail};
use std::{
  fs::OpenOptions,
  io::{self, BufRead, BufReader, Read, Write},
  os::fd::RawFd,
  os::unix::{net::UnixStream, process::CommandExt},
  process::{Child, Command, Stdio},
  time::{Duration, Instant},
};

const READY_FD: RawFd = 3;
const READY_TIMEOUT: Duration = Duration::from_secs(8);
const DRM_HANDOFF: &str = "/usr/lib/argvus/greeter/handoff";
const DRM_HANDOFF_SOCKET: &str = "/run/argvus-greeter/handoff.sock";
const DRM_HANDOFF_SOCKET_TIMEOUT: Duration = Duration::from_secs(2);
/// How long the greeter waits for the DRM handoff surface to show its first
/// frame after leaving the VT.
const DRM_HANDOFF_DRAWING_TIMEOUT: Duration = Duration::from_secs(2);
/// The empty virtual terminal the seat switches to while the greeter
/// compositor exits. It is above the autovt range (1-6 by default), so logind
/// starts no getty there.
const HANDOFF_VT: u32 = 12;
const ARGVUS: &str = "/usr/bin/argvus";
/// Login marks on `CLOCK_MONOTONIC`, shared with the DRM handoff surface. The
/// journal and the session log use the same clock.
const TIMING_LOG: &str = "/run/argvus-greeter/timing.log";

/// The greeter compositor and the `start-hyprland` watchdog that started it.
#[derive(Debug, PartialEq, Eq)]
struct CompositorProcesses {
  compositor: libc::pid_t,
  watchdog: Option<libc::pid_t>,
}

/// Ends the greeter session without waiting for its compositor. Call it only
/// after the DRM handoff surface took the display (`leave_vt` and
/// `wait_for_handoff_frame`).
///
/// greetd starts the user session only when the greeter session command ends.
/// That command is the `start-hyprland` watchdog, which waits for Hyprland.
/// The compositor's teardown hands its devices back to logind, and on the
/// NVIDIA driver logind then spends about half a second closing the DRM client,
/// answering nothing else, the user session's CreateSession included. That
/// happens whether the compositor exits or is killed.
///
/// So only the watchdog is stopped now: greetd moves on at once and a stopped
/// watchdog cannot relaunch Hyprland. The compositor, inactive and without DRM
/// master, is asked to exit only once the user session is on screen
/// (`defer_greeter_compositor_exit`), when logind is no longer on the login
/// path. Under greetd the compositor shares the foreground process group of the
/// greeter's terminal, so the hangup when greetd's session worker exits can end
/// it before that; the deferred request then finds no compositor. Without a
/// watchdog, the compositor is asked to exit now.
pub fn end_greeter_compositor() {
  let Some(watchdog) = compositor_processes().and_then(|processes| {
    // A controlling process that exits hangs up its terminal's foreground
    // process group. Hyprland does not handle SIGHUP, so if it is in that
    // group it would exit at once; the mark records whether that can happen.
    let watchdog = processes.watchdog?;
    mark(&format!(
      "greeter.watchdog-stop-requested {}",
      hangup_exposure(watchdog, processes.compositor)
    ));
    Some(watchdog)
  }) else {
    request_greeter_compositor_exit();
    return;
  };
  if !send_kill(watchdog) {
    tracing::warn!(watchdog, "could not stop the greeter compositor watchdog");
    request_greeter_compositor_exit();
    return;
  }
  defer_greeter_compositor_exit();
}

/// Waits for the DRM handoff surface to leave (it removes its socket once the
/// user compositor replaced it, or after its own timeouts), then for
/// `DEFERRED_EXIT_DELAY_SECONDS`, and asks the greeter compositor to exit.
/// `$1` is the socket and `$2` the delay. The wait for the socket is bounded
/// to 10 seconds.
const DEFERRED_EXIT_SCRIPT: &str = r#"socket="$1"
delay="$2"
polls=0
while [ -S "$socket" ] && [ "$polls" -lt 200 ]; do
  sleep 0.05
  polls=$((polls + 1))
done
sleep "$delay"
exec /usr/bin/argvus config hyprland exit"#;

/// Seconds between the handoff surface leaving and the greeter compositor's
/// exit, so the user session's own startup does not meet a busy logind.
const DEFERRED_EXIT_DELAY_SECONDS: &str = "3";

/// Asks the greeter compositor to exit once the user session is on screen,
/// from a process of its own: the greeter may end before that, and its own
/// process group, so the hangup of the greeter's terminal does not reach it.
/// When it cannot start, the compositor is asked to exit now.
fn defer_greeter_compositor_exit() {
  let spawned = Command::new("/bin/sh")
    .args([
      "-c",
      DEFERRED_EXIT_SCRIPT,
      "argvus-greeter-exit",
      DRM_HANDOFF_SOCKET,
      DEFERRED_EXIT_DELAY_SECONDS,
    ])
    .stdin(Stdio::null())
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .process_group(0)
    .spawn();
  match spawned {
    Ok(mut deferred) => {
      mark("greeter.compositor-exit-deferred");
      // Reaped while the greeter lives; the process keeps running after it.
      std::thread::spawn(move || {
        let _ = deferred.wait();
      });
    }
    Err(error) => {
      tracing::warn!(%error, "could not defer the greeter compositor exit");
      request_greeter_compositor_exit();
    }
  }
}

/// Whether ending `watchdog` can hang up `compositor`, from `/proc`, as
/// `session-leader=<bool> tty=<n> compositor-in-foreground=<bool>`.
fn hangup_exposure(watchdog: libc::pid_t, compositor: libc::pid_t) -> String {
  let watchdog_stat = std::fs::read_to_string(format!("/proc/{watchdog}/stat")).unwrap_or_default();
  let compositor_stat =
    std::fs::read_to_string(format!("/proc/{compositor}/stat")).unwrap_or_default();
  let watchdog_fields = stat_fields(&watchdog_stat);
  let compositor_fields = stat_fields(&compositor_stat);
  let field = |fields: &[&str], index: usize| fields.get(index).copied().unwrap_or("?").to_owned();
  let session_leader = field(&watchdog_fields, STAT_SESSION) == watchdog.to_string();
  let tty = field(&watchdog_fields, STAT_TTY);
  let in_foreground =
    field(&compositor_fields, STAT_PROCESS_GROUP) == field(&watchdog_fields, STAT_FOREGROUND_GROUP);
  format!("session-leader={session_leader} tty={tty} compositor-in-foreground={in_foreground}")
}

/// Sends SIGKILL to `pid`. Returns whether the signal was delivered.
fn send_kill(pid: libc::pid_t) -> bool {
  // SAFETY: kill(2) only takes integers; the PID was checked to be a process
  // of this user (`compositor_processes`).
  unsafe { libc::kill(pid, libc::SIGKILL) == 0 }
}

/// The greeter compositor of this session, from the lock file Hyprland writes
/// in its instance directory, and its watchdog when the parent is one.
///
/// Each PID is checked to be the expected program of this user: a stale lock
/// can name a PID that another process reuses, and greetd's session worker
/// (root) is the parent when Hyprland started without the watchdog.
fn compositor_processes() -> Option<CompositorProcesses> {
  let runtime_dir = std::env::var_os("XDG_RUNTIME_DIR")?;
  let signature = std::env::var("HYPRLAND_INSTANCE_SIGNATURE")
    .ok()
    .filter(|signature| is_instance_signature(signature))?;
  let lock = std::path::Path::new(&runtime_dir)
    .join("hypr")
    .join(signature)
    .join("hyprland.lock");
  let compositor = lock_pid(&std::fs::read_to_string(lock).ok()?)?;
  // SAFETY: getuid(2) cannot fail and has no side effects.
  let user = unsafe { libc::getuid() };
  if !is_user_process(compositor, user, &["Hyprland", "hyprland"]) {
    return None;
  }
  let watchdog = std::fs::read_to_string(format!("/proc/{compositor}/stat"))
    .ok()
    .and_then(|stat| stat_parent_pid(&stat))
    .filter(|&parent| is_user_process(parent, user, &["start-hyprland"]));
  Some(CompositorProcesses {
    compositor,
    watchdog,
  })
}

/// Whether `pid` runs one of `names` as `user`.
fn is_user_process(pid: libc::pid_t, user: libc::uid_t, names: &[&str]) -> bool {
  let name = std::fs::read_to_string(format!("/proc/{pid}/comm")).unwrap_or_default();
  let status = std::fs::read_to_string(format!("/proc/{pid}/status")).unwrap_or_default();
  names.contains(&name.trim()) && status_real_uid(&status) == Some(user)
}

/// A Hyprland instance signature names a directory, so it may hold only the
/// characters Hyprland uses (hex digits, digits and `_`).
fn is_instance_signature(signature: &str) -> bool {
  !signature.is_empty()
    && signature
      .chars()
      .all(|character| character.is_ascii_alphanumeric() || character == '_')
}

/// The compositor PID: the first line of `hyprland.lock`.
fn lock_pid(lock: &str) -> Option<libc::pid_t> {
  lock
    .lines()
    .next()?
    .trim()
    .parse()
    .ok()
    .filter(|&pid| pid > 1)
}

/// Indexes into `stat_fields`, which start at the state (field 3 of proc(5)).
const STAT_PARENT: usize = 1;
const STAT_PROCESS_GROUP: usize = 2;
const STAT_SESSION: usize = 3;
const STAT_TTY: usize = 4;
const STAT_FOREGROUND_GROUP: usize = 5;

/// The fields of `/proc/<pid>/stat` after the command name. The name in
/// parentheses can hold spaces and parentheses, so they start after its last `)`.
fn stat_fields(stat: &str) -> Vec<&str> {
  stat
    .rsplit_once(')')
    .map(|(_, fields)| fields.split_whitespace().collect())
    .unwrap_or_default()
}

/// The parent PID from `/proc/<pid>/stat`.
fn stat_parent_pid(stat: &str) -> Option<libc::pid_t> {
  stat_fields(stat)
    .get(STAT_PARENT)?
    .parse()
    .ok()
    .filter(|&pid| pid > 1)
}

/// The real UID from `/proc/<pid>/status`.
fn status_real_uid(status: &str) -> Option<libc::uid_t> {
  status
    .lines()
    .find_map(|line| line.strip_prefix("Uid:"))?
    .split_whitespace()
    .next()?
    .parse()
    .ok()
}

/// Asks the greeter compositor to exit through its IPC socket. A failure is
/// logged; greetd still ends the greeter session after its own timeout.
pub fn exit_greeter_compositor() {
  mark("greeter.compositor-exit-requested");
  let status = exit_request_command().status();
  if !matches!(status, Ok(status) if status.success()) {
    tracing::warn!(
      ?status,
      "the greeter compositor did not accept the exit request"
    );
  }
}

/// Asks the greeter compositor to exit without waiting for the answer. Hyprland
/// answers the IPC request only after its teardown, about half a second later,
/// so a thread reaps the request instead of the login waiting for it.
fn request_greeter_compositor_exit() {
  mark("greeter.compositor-exit-requested");
  match exit_request_command().spawn() {
    Ok(mut request) => {
      std::thread::spawn(move || {
        let _ = request.wait();
      });
    }
    Err(error) => {
      tracing::warn!(%error, "could not ask the greeter compositor to exit");
    }
  }
}

/// `argvus config hyprland exit`, which asks the compositor of this session
/// to exit through its IPC socket.
fn exit_request_command() -> Command {
  let mut command = Command::new(ARGVUS);
  command
    .args(["config", "hyprland", "exit"])
    .stdin(Stdio::null())
    .stdout(Stdio::null())
    .stderr(Stdio::null());
  command
}

/// Switches the seat to `HANDOFF_VT`, which deactivates the greeter session.
///
/// logind then takes DRM master from the greeter compositor without that
/// compositor touching the outputs, so its last frame stays on screen until the
/// DRM handoff surface replaces it. An inactive compositor also leaves the
/// outputs on when it exits, where an active one turns them off and the monitor
/// loses its signal. greetd switches back to its own VT for the user session.
pub fn leave_vt() -> anyhow::Result<()> {
  mark("greeter.vt-switch-requested");
  let connection = zbus::blocking::Connection::system().context("connecting to the system bus")?;
  // Not `seat/self`: Kitty moves the greeter into a systemd scope of its own,
  // outside the logind session, so logind cannot resolve the caller's seat.
  let seat_path = seat_object_path(std::env::var("XDG_SEAT").ok().as_deref());
  let seat = zbus::blocking::Proxy::new(
    &connection,
    "org.freedesktop.login1",
    seat_path.as_str(),
    "org.freedesktop.login1.Seat",
  )
  .with_context(|| format!("reaching the logind seat {seat_path}"))?;
  seat
    .call::<_, _, ()>("SwitchTo", &(HANDOFF_VT))
    .with_context(|| format!("switching to VT {HANDOFF_VT}"))?;
  Ok(())
}

/// The logind object of `seat`, from the session's `XDG_SEAT`, or of `seat0`
/// when it is unset or not a valid object path element.
fn seat_object_path(seat: Option<&str>) -> String {
  let seat = seat
    .filter(|seat| {
      !seat.is_empty()
        && seat
          .chars()
          .all(|character| character.is_ascii_alphanumeric() || character == '_')
    })
    .unwrap_or("seat0");
  format!("/org/freedesktop/login1/seat/{seat}")
}

/// Waits until the DRM handoff surface draws. The surface answers `ping` only
/// once it shows its first frame, which needs the DRM master that the greeter
/// compositor loses when the greeter leaves the VT.
pub fn wait_for_handoff_frame() -> anyhow::Result<()> {
  let mut stream =
    UnixStream::connect(DRM_HANDOFF_SOCKET).context("connecting to the DRM handoff surface")?;
  stream.set_read_timeout(Some(DRM_HANDOFF_DRAWING_TIMEOUT))?;
  stream.set_write_timeout(Some(DRM_HANDOFF_DRAWING_TIMEOUT))?;
  stream.write_all(b"ping\n")?;
  let mut answer = String::new();
  BufReader::new(&mut stream)
    .read_line(&mut answer)
    .context("waiting for the DRM handoff surface to draw")?;
  if answer.trim() != "ok" {
    bail!("the DRM handoff surface answered {:?}", answer.trim());
  }
  mark("greeter.handoff-drawing");
  Ok(())
}

/// Appends `event` to `TIMING_LOG`. Timing is diagnostic, so a failure to
/// write never affects the login.
pub fn mark(event: &str) {
  let line = format!(
    "monotonic_ns={} mark {event}\n",
    argvus_system::monotonic_ns()
  );
  if let Ok(mut file) = OpenOptions::new()
    .create(true)
    .append(true)
    .open(TIMING_LOG)
  {
    let _ = file.write_all(line.as_bytes());
  }
}

pub struct WaylandSplash {
  child: Child,
}

impl WaylandSplash {
  /// Starts the session splash and waits for its first rendered frame.
  pub fn start(theme: &str, accent: Option<&str>) -> anyhow::Result<Self> {
    if !std::path::Path::new(DRM_HANDOFF).is_file() {
      bail!("session splash is not installed at {DRM_HANDOFF}");
    }

    let mut pipe = [0; 2];
    if unsafe { libc::pipe(pipe.as_mut_ptr()) } != 0 {
      return Err(io::Error::last_os_error()).context("creating splash readiness pipe");
    }
    set_close_on_exec(pipe[0])?;
    set_close_on_exec(pipe[1])?;

    let write_fd = pipe[1];
    tracing::info!("T1 splash-spawn");
    let mut command = Command::new(DRM_HANDOFF);
    command.args([
      "--wayland",
      "--theme",
      theme,
      "--ready-fd",
      &READY_FD.to_string(),
    ]);
    if let Some(accent) = accent {
      command.args(["--accent", accent]);
    }
    command
      .stdin(Stdio::null())
      .stdout(Stdio::null())
      .stderr(Stdio::null());
    // SAFETY: pre_exec runs between fork and exec. It only duplicates the
    // already-created readiness descriptor and closes the inherited copies.
    unsafe {
      command.pre_exec(move || {
        if libc::dup2(write_fd, READY_FD) < 0 {
          return Err(io::Error::last_os_error());
        }
        if write_fd != READY_FD {
          libc::close(write_fd);
        }
        Ok(())
      });
    }
    let child = command.spawn().context("starting the session splash")?;
    unsafe {
      libc::close(pipe[1]);
    }

    if let Err(error) = wait_for_ready(pipe[0], child.id()) {
      let mut child = child;
      let _ = child.kill();
      let _ = child.wait();
      return Err(error);
    }
    tracing::info!(pid = child.id(), "T3 splash-first-frame and splash-ready");
    Ok(Self { child })
  }

  /// Disarms child cleanup after the greeter has handed the compositor off.
  pub fn detach(self) {
    let _ = self.child.id();
    std::mem::forget(self);
  }
}

impl Drop for WaylandSplash {
  fn drop(&mut self) {
    let _ = self.child.kill();
    let _ = self.child.wait();
  }
}

/// The DRM surface that keeps a frame on screen while the greeter compositor
/// exits and the user compositor has not taken the display yet.
///
/// The helper waits for DRM master, which the greeter compositor holds until
/// the greeter leaves the VT (`leave_vt`) or the compositor exits, and then
/// draws. It is deliberately not owned by this process: it keeps
/// running after the greeter ends and stops only when the user session sends
/// `release` (see `argvus-start`). Nothing here blocks the login on failure.
pub struct DrmHandoff;

impl DrmHandoff {
  /// Starts the helper and waits until its command socket is listening. The
  /// theme and accent are the ones given to the GTK splash, so both surfaces
  /// paint the same spinner in the same colors.
  pub fn start(theme: &str, accent: Option<&str>) -> anyhow::Result<Self> {
    if !std::path::Path::new(DRM_HANDOFF).is_file() {
      bail!("DRM handoff surface is not installed at {DRM_HANDOFF}");
    }
    // A socket left by an earlier surface would make the readiness check pass
    // before this helper has bound its own socket.
    let _ = std::fs::remove_file(DRM_HANDOFF_SOCKET);

    let mut command = Command::new(DRM_HANDOFF);
    command.args(["--theme", theme]);
    if let Some(accent) = accent {
      command.args(["--accent", accent]);
    }
    command
      .stdin(Stdio::null())
      .stdout(Stdio::null())
      .stderr(Stdio::null())
      // Own process group, so the greeter session teardown does not signal it.
      .process_group(0);
    let child = command
      .spawn()
      .context("starting the DRM handoff surface")?;

    let deadline = Instant::now() + DRM_HANDOFF_SOCKET_TIMEOUT;
    while !std::path::Path::new(DRM_HANDOFF_SOCKET).exists() {
      if Instant::now() >= deadline {
        bail!(
          "DRM handoff surface did not open its socket (pid {})",
          child.id()
        );
      }
      std::thread::sleep(Duration::from_millis(20));
    }
    tracing::info!(pid = child.id(), "DRM handoff surface is listening");
    // Dropping the Child does not stop the process; it is intentionally left running.
    Ok(Self)
  }
}

fn set_close_on_exec(fd: RawFd) -> io::Result<()> {
  let flags = unsafe { libc::fcntl(fd, libc::F_GETFD) };
  if flags < 0 {
    return Err(io::Error::last_os_error());
  }
  if unsafe { libc::fcntl(fd, libc::F_SETFD, flags | libc::FD_CLOEXEC) } < 0 {
    return Err(io::Error::last_os_error());
  }
  Ok(())
}

fn wait_for_ready(fd: RawFd, pid: u32) -> anyhow::Result<()> {
  let mut pollfd = libc::pollfd {
    fd,
    events: libc::POLLIN,
    revents: 0,
  };
  let timeout = READY_TIMEOUT.as_millis().min(i32::MAX as u128) as i32;
  let result = unsafe { libc::poll(&mut pollfd, 1, timeout) };
  if result <= 0 || pollfd.revents & libc::POLLIN == 0 {
    unsafe { libc::close(fd) };
    bail!("session splash did not report a rendered frame (pid {pid})");
  }
  let mut file = unsafe { std::fs::File::from_raw_fd(fd) };
  let mut ready = Vec::with_capacity(6);
  while ready.len() < 6 {
    let mut byte = [0_u8; 1];
    let bytes = file.read(&mut byte).context("reading splash readiness")?;
    if bytes == 0 {
      break;
    }
    ready.push(byte[0]);
  }
  if ready.as_slice() != b"READY\n" {
    bail!("session splash returned an invalid readiness marker");
  }
  Ok(())
}

use std::os::fd::FromRawFd;

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn the_seat_object_comes_from_xdg_seat() {
    assert_eq!(
      seat_object_path(Some("seat1")),
      "/org/freedesktop/login1/seat/seat1"
    );
    assert_eq!(seat_object_path(None), "/org/freedesktop/login1/seat/seat0");
    assert_eq!(
      seat_object_path(Some("")),
      "/org/freedesktop/login1/seat/seat0"
    );
    assert_eq!(
      seat_object_path(Some("../x")),
      "/org/freedesktop/login1/seat/seat0"
    );
  }

  #[test]
  fn the_compositor_pid_is_the_first_line_of_the_lock() {
    assert_eq!(lock_pid("691029\nwayland-1\n"), Some(691029));
    assert_eq!(lock_pid(" 42 \n"), Some(42));
    assert_eq!(lock_pid(""), None);
    assert_eq!(lock_pid("wayland-1\n"), None);
    // PID 1 and below are never a greeter compositor.
    assert_eq!(lock_pid("1\n"), None);
    assert_eq!(lock_pid("-5\n"), None);
  }

  #[test]
  fn the_parent_pid_is_read_after_the_command_name() {
    let stat = "691029 (Hyprland) S 691026 691026 691026 0 -1 4194560";
    assert_eq!(stat_parent_pid(stat), Some(691026));
    // A command name with spaces and parentheses does not shift the fields.
    let stat = "77 (a) b) c) S 70 77 77 0 -1";
    assert_eq!(stat_parent_pid(stat), Some(70));
    assert_eq!(stat_parent_pid("77 (init) S 1 1 1"), None);
    assert_eq!(stat_parent_pid("garbage"), None);
  }

  #[test]
  fn the_stat_fields_name_the_process_group_session_and_terminal() {
    let fields = stat_fields("691026 (start-hyprland) S 690990 691026 691026 1025 691026 4194560");
    assert_eq!(fields[STAT_PARENT], "690990");
    assert_eq!(fields[STAT_PROCESS_GROUP], "691026");
    assert_eq!(fields[STAT_SESSION], "691026");
    assert_eq!(fields[STAT_TTY], "1025");
    assert_eq!(fields[STAT_FOREGROUND_GROUP], "691026");
    assert!(stat_fields("garbage").is_empty());
  }

  #[test]
  fn the_real_uid_is_the_first_uid_field() {
    let status = "Name:\tHyprland\nUid:\t935\t935\t935\t935\nGid:\t935\n";
    assert_eq!(status_real_uid(status), Some(935));
    assert_eq!(status_real_uid("Name:\tx\n"), None);
  }

  #[test]
  fn an_instance_signature_cannot_leave_the_hypr_directory() {
    assert!(is_instance_signature(
      "efb50993780079460b0cbed1363e2166a2de1d9f_1793200274_1827100966"
    ));
    assert!(!is_instance_signature(""));
    assert!(!is_instance_signature("../935"));
    assert!(!is_instance_signature("a/b"));
  }
}
