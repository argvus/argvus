//! Readiness handshake for the graphical login transition.
//!
//! The login TUI cannot return immediately after greetd accepts a session:
//! its terminal wrapper would tear down the last visible frame first. This
//! module starts the GTK splash inside the still-running greeter compositor,
//! waits for a real frame-clock notification, and only then allows the TUI
//! guard to enter its handoff-preserving exit mode.

use anyhow::{Context, bail};
use std::{
  io::{self, Read},
  os::fd::RawFd,
  os::unix::process::CommandExt,
  process::{Child, Command, Stdio},
  time::{Duration, Instant},
};

const SPLASH: &str = "/usr/lib/argvus/loading-theme/splash";
const READY_FD: RawFd = 3;
const READY_TIMEOUT: Duration = Duration::from_secs(8);
const DRM_HANDOFF: &str = "/usr/lib/argvus/greeter/handoff";
const DRM_HANDOFF_SOCKET: &str = "/run/argvus-greeter/handoff.sock";
const DRM_HANDOFF_SOCKET_TIMEOUT: Duration = Duration::from_secs(2);

pub struct WaylandSplash {
  child: Child,
}

impl WaylandSplash {
  /// Starts the session splash and waits for its first rendered frame.
  pub fn start(theme: &str, accent: Option<&str>) -> anyhow::Result<Self> {
    if !std::path::Path::new(SPLASH).is_file() {
      bail!("session splash is not installed at {SPLASH}");
    }

    let mut pipe = [0; 2];
    if unsafe { libc::pipe(pipe.as_mut_ptr()) } != 0 {
      return Err(io::Error::last_os_error()).context("creating splash readiness pipe");
    }
    set_close_on_exec(pipe[0])?;
    set_close_on_exec(pipe[1])?;

    let write_fd = pipe[1];
    tracing::info!("T1 splash-spawn");
    let mut command = Command::new(SPLASH);
    command.args([
      "--session-handoff",
      "--spinner-only",
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
/// The helper waits for DRM master, which the greeter compositor holds until it
/// exits, and then draws. It is deliberately not owned by this process: it keeps
/// running after the greeter ends and stops only when the user session sends
/// `release` (see `argvus-start`). Nothing here blocks the login on failure.
pub struct DrmHandoff;

impl DrmHandoff {
  /// Starts the helper and waits until its command socket is listening.
  pub fn start(theme: &str) -> anyhow::Result<Self> {
    if !std::path::Path::new(DRM_HANDOFF).is_file() {
      bail!("DRM handoff surface is not installed at {DRM_HANDOFF}");
    }
    // A socket left by an earlier surface would make the readiness check pass
    // before this helper has bound its own socket.
    let _ = std::fs::remove_file(DRM_HANDOFF_SOCKET);

    let mut command = Command::new(DRM_HANDOFF);
    command
      .args(["--theme", theme])
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
