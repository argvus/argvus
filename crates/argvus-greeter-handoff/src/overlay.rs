//! Overlay plane that keeps the spinner on screen while the user compositor takes over the display.

use super::control::{Command, parse_command, read_command};
use super::drawing::{Palette, draw_background, draw_spinner};
use super::drm::{DRM_FORMAT_XRGB8888, DumbBuffer};
use super::drm::{
  drmModeFreePlane, drmModeFreePlaneResources, drmModeFreeResources, drmModeGetPlane,
  drmModeGetPlaneResources, drmModeGetResources, drmModeSetPlane,
};
use super::outputs::{Released, crtc_state};
use super::timing::{mark, monotonic_millis};
use super::*;

/// A copy of the last frame on an overlay plane of the output's CRTC. The user
/// compositor only drives the primary and cursor planes, so the overlay stays
/// on screen through its first modeset. Dropping it removes the framebuffer,
/// and the kernel turns off that plane alone; that needs no DRM master.
pub(super) struct Overlay {
  pub(super) buffer: DumbBuffer,
}

/// Puts the released frame on a free overlay plane. Returns `None`, and the
/// surface falls back to the primary plane alone, when the CRTC has no free
/// overlay plane for XRGB8888 or the driver refuses it.
pub(super) fn show_overlay(fd: RawFd, released: &Released, palette: &Palette) -> Option<Overlay> {
  let Some(plane_id) = crtc_index(fd, released.crtc_id)
    .and_then(|index| free_overlay_plane(&overlay_planes(fd), index))
  else {
    mark("handoff.overlay-unavailable reason=no-free-plane");
    return None;
  };
  let (width, height) = (released.buffer.width, released.buffer.height);
  let mut buffer = match DumbBuffer::create(fd, width, height) {
    Ok(buffer) => buffer,
    Err(error) => {
      mark(&format!(
        "handoff.overlay-unavailable errno={}",
        error.raw_os_error().unwrap_or_default()
      ));
      return None;
    }
  };
  draw_background(&mut buffer, palette);
  draw_spinner(&mut buffer, palette, &spinner::frame(monotonic_millis()));
  // Source coordinates are 16.16 fixed point.
  let result = unsafe {
    drmModeSetPlane(
      fd,
      plane_id,
      released.crtc_id,
      buffer.framebuffer,
      0,
      0,
      0,
      width,
      height,
      0,
      0,
      width << 16,
      height << 16,
    )
  };
  if result != 0 {
    // libdrm returns the negated errno.
    mark(&format!("handoff.overlay-unavailable errno={}", -result));
    return None;
  }
  mark(&format!("handoff.overlay plane={plane_id}"));
  Some(Overlay { buffer })
}

/// One overlay plane as `drmModeGetPlane` reports it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct OverlayPlane {
  pub(super) plane_id: u32,
  pub(super) crtc_id: u32,
  pub(super) possible_crtcs: u32,
  pub(super) xrgb8888: bool,
}

/// The first plane that can show an XRGB8888 buffer on the CRTC at
/// `crtc_index` and that no CRTC uses yet.
pub(super) fn free_overlay_plane(planes: &[OverlayPlane], crtc_index: usize) -> Option<u32> {
  planes
    .iter()
    .find(|plane| {
      plane.crtc_id == 0
        && plane.xrgb8888
        && crtc_index < 32
        && plane.possible_crtcs & (1 << crtc_index) != 0
    })
    .map(|plane| plane.plane_id)
}

/// The planes of `fd`. Without `DRM_CLIENT_CAP_UNIVERSAL_PLANES`, which this
/// surface never sets, the kernel lists only the overlay planes.
pub(super) fn overlay_planes(fd: RawFd) -> Vec<OverlayPlane> {
  let resources = unsafe { drmModeGetPlaneResources(fd) };
  if resources.is_null() {
    return Vec::new();
  }
  let resources_ref = unsafe { &*resources };
  let planes = (0..resources_ref.count_planes as usize)
    .filter_map(|index| {
      let plane_id = unsafe { *resources_ref.planes.add(index) };
      let plane = unsafe { drmModeGetPlane(fd, plane_id) };
      if plane.is_null() {
        return None;
      }
      let plane_ref = unsafe { &*plane };
      let formats =
        unsafe { std::slice::from_raw_parts(plane_ref.formats, plane_ref.count_formats as usize) };
      let info = OverlayPlane {
        plane_id,
        crtc_id: plane_ref.crtc_id,
        possible_crtcs: plane_ref.possible_crtcs,
        xrgb8888: formats.contains(&DRM_FORMAT_XRGB8888),
      };
      unsafe { drmModeFreePlane(plane) };
      Some(info)
    })
    .collect();
  unsafe { drmModeFreePlaneResources(resources) };
  planes
}

/// The position of `crtc_id` in the device's CRTC list, which is the bit
/// `possible_crtcs` uses for it.
pub(super) fn crtc_index(fd: RawFd, crtc_id: u32) -> Option<usize> {
  let resources = unsafe { drmModeGetResources(fd) };
  if resources.is_null() {
    return None;
  }
  let resources_ref = unsafe { &*resources };
  let index = (0..resources_ref.count_crtcs as usize)
    .find(|&index| unsafe { *resources_ref.crtcs.add(index) } == crtc_id);
  unsafe { drmModeFreeResources(resources) };
  index
}

/// Why the overlay leaves the screen without a `retire`, if it has to.
pub(super) fn overlay_expired(
  elapsed: Duration,
  since_replaced: Option<Duration>,
) -> Option<&'static str> {
  match since_replaced {
    Some(since) if since >= RETIRE_TIMEOUT => Some("handoff.retire-timeout"),
    None if elapsed >= REPLACEMENT_TIMEOUT => Some("handoff.replacement-timeout"),
    _ => None,
  }
}

/// Keeps the overlay on screen and its spinner animating until the session
/// spinner sends `retire`, which it does once its own first frame is on
/// screen, or until `overlay_expired`. Writing to the buffer needs no DRM
/// master.
pub(super) fn hold_overlay(
  fd: RawFd,
  listener: &UnixListener,
  released: &Released,
  overlay: &mut Overlay,
  palette: &Palette,
) {
  let started = Instant::now();
  let mut replaced_at = None;
  loop {
    while let Ok((mut stream, _)) = listener.accept() {
      let Ok(line) = read_command(&mut stream) else {
        continue;
      };
      let command = parse_command(&line);
      let _ = stream.write_all(match command {
        Command::Unknown => b"error\n".as_slice(),
        // Already released; a repeated request gets the same answer.
        Command::Release => b"released\n".as_slice(),
        Command::Theme(_) | Command::Ping | Command::Retire => b"ok\n".as_slice(),
      });
      if command == Command::Retire {
        mark("handoff.retired");
        return;
      }
    }
    if replaced_at.is_none()
      && crtc_state(fd, released.crtc_id).map(|(framebuffer, _)| framebuffer)
        != Some(released.buffer.framebuffer)
    {
      mark("handoff.replaced-by-compositor");
      replaced_at = Some(Instant::now());
    }
    if let Some(event) = overlay_expired(started.elapsed(), replaced_at.map(|at| at.elapsed())) {
      mark(event);
      return;
    }
    draw_spinner(
      &mut overlay.buffer,
      palette,
      &spinner::frame(monotonic_millis()),
    );
    thread::sleep(POLL_DELAY);
  }
}
