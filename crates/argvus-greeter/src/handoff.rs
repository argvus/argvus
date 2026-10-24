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

/// Asks the greeter compositor to exit through its IPC socket. A failure is
/// logged; greetd still ends the greeter session after its own timeout.
pub fn exit_greeter_compositor() {
  mark("greeter.compositor-exit-requested");
  let status = Command::new(ARGVUS)
    .args(["config", "hyprland", "exit"])
    .stdin(Stdio::null())
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .status();
  if !matches!(status, Ok(status) if status.success()) {
    tracing::warn!(
      ?status,
      "the greeter compositor did not accept the exit request"
    );
  }
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
}
