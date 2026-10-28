//! Direct DRM/KMS transition surface used while greetd changes compositors.
//!
//! The normal ARGVUS splash is a Wayland client and therefore cannot render
//! while the greeter compositor has already exited and the user compositor has
//! not acquired the display yet. This small process owns a DRM dumb buffer for
//! that exact interval and releases it through a private handoff socket.

use std::{
  fs::{self, OpenOptions},
  io::{self, BufRead, BufReader, Write},
  os::fd::{AsRawFd, IntoRawFd, RawFd},
  os::unix::fs::PermissionsExt,
  os::unix::net::{UnixListener, UnixStream},
  path::{Path, PathBuf},
  thread,
  time::{Duration, Instant},
};

use argvus_loading_theme_core::{
  DEFAULT_ACCENT, DEFAULT_BACKGROUND, hex_to_rgb, spinner, theme_colors,
};
use clap::Parser;

mod control;
mod drawing;
mod drm;
mod outputs;
mod overlay;
mod timing;
mod wayland;

use control::{handle_command, prepare_socket, send_command};
use drawing::{Palette, draw_background, draw_spinner};
use drm::{ModeInfo, drmDropMaster, drmModePageFlip, drmModeSetCrtc, drmModeSetCursor};
use outputs::{
  PreparedSurface, Released, acquire_card, crtc_mode, crtc_state, open_observers, prepare_surface,
  wait_for_replacement,
};
use overlay::{hold_overlay, show_overlay};
use timing::{mark, monotonic_millis};

const DEFAULT_SOCKET: &str = "/run/argvus-greeter/handoff.sock";
/// Login marks on `CLOCK_MONOTONIC`, shared with the greeter. The journal and
/// the session log use the same clock, so one login can be timed end to end.
const TIMING_LOG: &str = "/run/argvus-greeter/timing.log";
const DEFAULT_THEME: &str = "argvus-dark";
/// How often the surface redraws the spinner and checks for commands.
const POLL_DELAY: Duration = Duration::from_millis(10);
/// How long the last frame stays on screen after a release while the user
/// compositor takes over the output.
const REPLACEMENT_TIMEOUT: Duration = Duration::from_secs(10);
/// How long the overlay stays over the user compositor once it has replaced
/// the primary plane, when the session spinner never sends `retire`.
const RETIRE_TIMEOUT: Duration = Duration::from_secs(3);
/// Interval between the attempts to become DRM master. Each attempt is one
/// `open`, and every millisecond the output spends between two masters is a
/// frame that nobody draws.
const MASTER_POLL: Duration = Duration::from_millis(1);
const ACK_TIMEOUT: Duration = Duration::from_secs(20);
/// How long the surface waits for one client to send its command line. The
/// socket is open to every local user, so a client that connects and stays
/// silent must not stop the surface from serving the session's `release`.
const CLIENT_TIMEOUT: Duration = Duration::from_secs(1);

#[derive(Debug, Parser)]
#[command(name = "argvus-greeter-handoff")]
struct Cli {
  /// Unix socket used by argvus-session to update the theme and release DRM.
  #[arg(long, default_value = DEFAULT_SOCKET)]
  socket: PathBuf,
  /// Initial theme used before the authenticated session publishes its theme.
  #[arg(long, default_value = DEFAULT_THEME)]
  theme: String,
  /// The user's accent (`#rrggbb`), as the greeter passes it to the GTK splash.
  #[arg(long)]
  accent: Option<String>,
  /// Client mode: sends one command (`release` or `theme=NAME`) to the running
  /// surface, waits for its acknowledgement and exits. Without a running
  /// surface this does nothing and succeeds.
  #[arg(long, value_name = "COMMAND")]
  send: Option<String>,
  /// Wayland mode: draws the same spinner as a layer-shell overlay on the
  /// compositor of `WAYLAND_DISPLAY`, until SIGTERM or until the compositor
  /// goes away. Fails when it cannot connect, so the caller can retry.
  #[arg(long, conflicts_with = "send")]
  wayland: bool,
  /// Wayland mode: writes `READY` to this inherited descriptor once every
  /// output has shown its first frame.
  #[arg(long, requires = "wayland")]
  ready_fd: Option<i32>,
  /// Wayland mode: once every output has shown its first frame, sends
  /// `retire` to the DRM handoff surface, which then removes the overlay that
  /// hid the compositor's first frames.
  #[arg(long, requires = "wayland")]
  retire_handoff: bool,
}

fn main() -> io::Result<()> {
  let cli = Cli::parse();
  if let Some(command) = cli.send.as_deref() {
    return send_command(&cli.socket, command);
  }
  if cli.wayland {
    let palette = Palette::new(&cli.theme, cli.accent.as_deref().and_then(hex_to_rgb));
    return wayland::run(palette, cli.ready_fd, cli.retire_handoff);
  }
  let listener = prepare_socket(&cli.socket)?;
  let user_accent = cli.accent.as_deref().and_then(hex_to_rgb);
  let mut palette = Palette::new(&cli.theme, user_accent);
  mark("handoff.waiting-for-master");
  let (cards, mut observers) = open_observers()?;
  // Finding the output and painting the first frame need no DRM master, so
  // they happen before the wait and the output gets its frame as soon as the
  // greeter compositor lets go of it. The buffer belongs to the observer's
  // descriptor, which stays open until this process exits.
  let prepared = observers
    .iter()
    .find_map(|observer| prepare_surface(observer.file.as_raw_fd(), &palette).ok());
  if let Some(prepared) = &prepared {
    mark(&format!("handoff.prepared crtc={}", prepared.crtc_id));
  }
  let fd = acquire_card(&cards, &mut observers)?;
  mark("handoff.master-acquired");

  let result = run_surface(fd, prepared, &listener, &mut palette);
  // The overlay can only be set while this process is still DRM master.
  let overlay = result
    .as_ref()
    .ok()
    .and_then(|released| show_overlay(fd, released, &palette));
  unsafe {
    drmDropMaster(fd);
  }
  mark("handoff.master-dropped");
  let outcome = match result {
    Ok(mut released) => {
      // The release is acknowledged only after DRM master has been dropped, so
      // the caller can start the user compositor without racing this process.
      let _ = released.stream.write_all(b"released\n");
      match overlay {
        // The compositor's first modeset scans out a buffer it has not drawn
        // yet. The overlay stays above it, with the spinner still animating,
        // until the session spinner is on screen.
        Some(mut overlay) => {
          hold_overlay(fd, &listener, &released, &mut overlay, &palette);
          drop(overlay);
        }
        // Destroying the framebuffer now would make the kernel turn the output
        // off until the compositor's first modeset. The buffer stays on the
        // output, with the spinner still animating, until the compositor
        // replaces it.
        None => {
          if wait_for_replacement(fd, released.crtc_id, &mut released.buffer, &palette) {
            mark("handoff.replaced-by-compositor");
          } else {
            mark("handoff.replacement-timeout");
          }
        }
      }
      drop(released.buffer);
      Ok(())
    }
    Err(error) => Err(error),
  };
  let _ = fs::remove_file(&cli.socket);
  unsafe {
    libc::close(fd);
  }
  outcome
}

fn run_surface(
  fd: RawFd,
  prepared: Option<PreparedSurface>,
  listener: &UnixListener,
  palette: &mut Palette,
) -> io::Result<Released> {
  let PreparedSurface {
    connector_id,
    crtc_id,
    mode,
    mut buffer,
  } = match prepared {
    Some(prepared) => prepared,
    None => prepare_surface(fd, palette)?,
  };
  // Whether the greeter compositor left its frame on the output or turned the
  // output off decides if the monitor has to sync again at this modeset.
  match crtc_state(fd, crtc_id) {
    Some((framebuffer, mode_valid)) => mark(&format!(
      "handoff.output-found crtc={crtc_id} framebuffer={framebuffer} mode_valid={mode_valid}"
    )),
    None => mark(&format!(
      "handoff.output-found crtc={crtc_id} state=unknown"
    )),
  }
  let mut connector = connector_id;
  // The frame was painted before the wait; bring the spinner to this instant.
  draw_spinner(&mut buffer, palette, &spinner::frame(monotonic_millis()));
  // When the greeter compositor left its frame on the output in the same mode,
  // a page flip swaps the image without a modeset. A modeset can make the
  // monitor resynchronise, which shows as a black flash even with identical
  // timings. A page flip needs a framebuffer to replace: with an empty primary
  // plane the kernel refuses it (EBUSY), so that case goes to the modeset.
  let mut flipped = false;
  let primary_has_frame = crtc_state(fd, crtc_id).is_some_and(|(framebuffer, _)| framebuffer != 0);
  if primary_has_frame && crtc_mode(fd, crtc_id).is_some_and(|current| current.same_timings(&mode))
  {
    // libdrm returns the negated errno.
    match unsafe { drmModePageFlip(fd, crtc_id, buffer.framebuffer, 0, std::ptr::null_mut()) } {
      0 => flipped = true,
      error => mark(&format!("handoff.page-flip-failed errno={}", -error)),
    }
  }
  if flipped {
    mark("handoff.page-flip");
  } else {
    let set_result = unsafe {
      drmModeSetCrtc(
        fd,
        crtc_id,
        buffer.framebuffer,
        0,
        0,
        &mut connector,
        1,
        &mode as *const ModeInfo as *mut ModeInfo,
      )
    };
    if set_result != 0 {
      return Err(io::Error::last_os_error());
    }
    mark("handoff.modeset");
  }

  mark("handoff.first-frame");
  // The greeter compositor's cursor stays on its plane until that compositor
  // exits; the surface shows no cursor.
  unsafe { drmModeSetCursor(fd, crtc_id, 0, 0, 0) };
  loop {
    while let Ok((stream, _)) = listener.accept() {
      let previous = *palette;
      match handle_command(stream, palette) {
        Ok(Some(stream)) => {
          mark("handoff.release-received");
          return Ok(Released {
            stream,
            buffer,
            crtc_id,
          });
        }
        Ok(None) => {}
        // A client that disconnects early or sends garbage must not end the surface.
        Err(_) => {}
      }
      if *palette != previous {
        draw_background(&mut buffer, palette);
      }
    }
    draw_spinner(&mut buffer, palette, &spinner::frame(monotonic_millis()));
    thread::sleep(POLL_DELAY);
  }
}

#[cfg(test)]
mod tests;
