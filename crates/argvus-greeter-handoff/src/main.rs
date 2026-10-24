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

mod wayland;

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
/// `DRM_FORMAT_XRGB8888`, the format of every dumb buffer this surface draws.
const DRM_FORMAT_XRGB8888: u32 = 0x3432_5258;
/// Interval between the attempts to become DRM master. Each attempt is one
/// `open`, and every millisecond the output spends between two masters is a
/// frame that nobody draws.
const MASTER_POLL: Duration = Duration::from_millis(1);
const ACK_TIMEOUT: Duration = Duration::from_secs(20);
/// How long the surface waits for one client to send its command line. The
/// socket is open to every local user, so a client that connects and stays
/// silent must not stop the surface from serving the session's `release`.
const CLIENT_TIMEOUT: Duration = Duration::from_secs(1);
const DRM_IOCTL_MODE_CREATE_DUMB: libc::c_ulong = 0xc020_64b2;
const DRM_IOCTL_MODE_MAP_DUMB: libc::c_ulong = 0xc010_64b3;
const DRM_IOCTL_MODE_DESTROY_DUMB: libc::c_ulong = 0xc010_64b4;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Palette {
  background: u32,
  accent: u32,
  /// The user's accent, kept when a `theme=` command changes the theme.
  user_accent: Option<u32>,
}

impl Palette {
  /// The colors of `theme` from the table the GTK splash uses, with the user's
  /// accent when there is one, so both surfaces paint the same colors.
  fn new(theme: &str, user_accent: Option<u32>) -> Self {
    let (background, theme_accent) = theme_colors(theme)
      .map(|(background, _, accent)| (background, accent))
      .unwrap_or((DEFAULT_BACKGROUND, DEFAULT_ACCENT));
    let background = hex_to_rgb(background).unwrap_or_default();
    let accent = user_accent
      .or_else(|| hex_to_rgb(theme_accent))
      .unwrap_or_default();
    Self {
      background,
      accent,
      user_accent,
    }
  }

  fn with_theme(&self, theme: &str) -> Self {
    Self::new(theme, self.user_accent)
  }
}

#[repr(C)]
struct ModeResources {
  count_fbs: libc::c_int,
  fbs: *mut u32,
  count_crtcs: libc::c_int,
  crtcs: *mut u32,
  count_connectors: libc::c_int,
  connectors: *mut u32,
  count_encoders: libc::c_int,
  encoders: *mut u32,
  min_width: u32,
  max_width: u32,
  min_height: u32,
  max_height: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct ModeInfo {
  clock: u32,
  hdisplay: u16,
  hsync_start: u16,
  hsync_end: u16,
  htotal: u16,
  hskew: u16,
  vdisplay: u16,
  vsync_start: u16,
  vsync_end: u16,
  vtotal: u16,
  vscan: u16,
  vrefresh: u32,
  flags: u32,
  mode_type: u32,
  name: [libc::c_char; 32],
}

impl ModeInfo {
  /// Whether both modes drive the output with the same timings, so moving
  /// from one to the other needs no modeset.
  fn same_timings(&self, other: &Self) -> bool {
    (
      self.clock,
      self.hdisplay,
      self.hsync_start,
      self.hsync_end,
      self.htotal,
      self.vdisplay,
      self.vsync_start,
      self.vsync_end,
      self.vtotal,
      self.flags,
    ) == (
      other.clock,
      other.hdisplay,
      other.hsync_start,
      other.hsync_end,
      other.htotal,
      other.vdisplay,
      other.vsync_start,
      other.vsync_end,
      other.vtotal,
      other.flags,
    )
  }
}

#[repr(C)]
struct Connector {
  connector_id: u32,
  encoder_id: u32,
  connector_type: u32,
  connector_type_id: u32,
  connection: libc::c_int,
  mm_width: u32,
  mm_height: u32,
  subpixel: libc::c_int,
  count_modes: libc::c_int,
  modes: *mut ModeInfo,
  count_props: libc::c_int,
  props: *mut u32,
  prop_values: *mut u64,
  count_encoders: libc::c_int,
  encoders: *mut u32,
}

#[repr(C)]
struct Encoder {
  encoder_id: u32,
  encoder_type: u32,
  crtc_id: u32,
  possible_crtcs: u32,
  possible_clones: u32,
}

#[repr(C)]
struct Crtc {
  crtc_id: u32,
  buffer_id: u32,
  x: u32,
  y: u32,
  width: u32,
  height: u32,
  mode_valid: libc::c_int,
  mode: ModeInfo,
  gamma_size: libc::c_int,
}

#[repr(C)]
struct PlaneResources {
  count_planes: u32,
  planes: *mut u32,
}

#[repr(C)]
struct Plane {
  count_formats: u32,
  formats: *mut u32,
  plane_id: u32,
  crtc_id: u32,
  fb_id: u32,
  crtc_x: u32,
  crtc_y: u32,
  x: u32,
  y: u32,
  possible_crtcs: u32,
  gamma_size: u32,
}

#[repr(C)]
struct CreateDumb {
  height: u32,
  width: u32,
  bpp: u32,
  flags: u32,
  handle: u32,
  pitch: u32,
  size: u64,
}

#[repr(C)]
struct MapDumb {
  handle: u32,
  pad: u32,
  offset: u64,
}

#[repr(C)]
struct DestroyDumb {
  handle: u32,
}

#[link(name = "drm")]
unsafe extern "C" {
  fn drmSetMaster(fd: libc::c_int) -> libc::c_int;
  fn drmDropMaster(fd: libc::c_int) -> libc::c_int;
  fn drmIoctl(fd: libc::c_int, request: libc::c_ulong, arg: *mut libc::c_void) -> libc::c_int;
  fn drmModeGetResources(fd: libc::c_int) -> *mut ModeResources;
  fn drmModeFreeResources(resources: *mut ModeResources);
  fn drmModeGetConnector(fd: libc::c_int, connector_id: u32) -> *mut Connector;
  fn drmModeGetConnectorCurrent(fd: libc::c_int, connector_id: u32) -> *mut Connector;
  fn drmModeFreeConnector(connector: *mut Connector);
  fn drmModeGetEncoder(fd: libc::c_int, encoder_id: u32) -> *mut Encoder;
  fn drmModeFreeEncoder(encoder: *mut Encoder);
  fn drmModeAddFB(
    fd: libc::c_int,
    width: u32,
    height: u32,
    depth: u8,
    bpp: u8,
    pitch: u32,
    handle: u32,
    buffer_id: *mut u32,
  ) -> libc::c_int;
  fn drmModeRmFB(fd: libc::c_int, buffer_id: u32) -> libc::c_int;
  fn drmModeGetCrtc(fd: libc::c_int, crtc_id: u32) -> *mut Crtc;
  fn drmModeFreeCrtc(crtc: *mut Crtc);
  fn drmModeSetCrtc(
    fd: libc::c_int,
    crtc_id: u32,
    buffer_id: u32,
    x: u32,
    y: u32,
    connectors: *mut u32,
    connector_count: libc::c_int,
    mode: *mut ModeInfo,
  ) -> libc::c_int;
  fn drmModeSetCursor(
    fd: libc::c_int,
    crtc_id: u32,
    handle: u32,
    width: u32,
    height: u32,
  ) -> libc::c_int;
  fn drmModeGetPlaneResources(fd: libc::c_int) -> *mut PlaneResources;
  fn drmModeFreePlaneResources(resources: *mut PlaneResources);
  fn drmModeGetPlane(fd: libc::c_int, plane_id: u32) -> *mut Plane;
  fn drmModeFreePlane(plane: *mut Plane);
  fn drmModeSetPlane(
    fd: libc::c_int,
    plane_id: u32,
    crtc_id: u32,
    fb_id: u32,
    flags: u32,
    crtc_x: i32,
    crtc_y: i32,
    crtc_w: u32,
    crtc_h: u32,
    src_x: u32,
    src_y: u32,
    src_w: u32,
    src_h: u32,
  ) -> libc::c_int;
  fn drmModePageFlip(
    fd: libc::c_int,
    crtc_id: u32,
    buffer_id: u32,
    flags: u32,
    user_data: *mut libc::c_void,
  ) -> libc::c_int;
}

struct DumbBuffer {
  fd: RawFd,
  handle: u32,
  width: u32,
  height: u32,
  pitch: u32,
  size: usize,
  map: *mut u8,
  framebuffer: u32,
}

impl DumbBuffer {
  fn create(fd: RawFd, width: u32, height: u32) -> io::Result<Self> {
    let mut request = CreateDumb {
      height,
      width,
      bpp: 32,
      flags: 0,
      handle: 0,
      pitch: 0,
      size: 0,
    };
    drm_call(fd, DRM_IOCTL_MODE_CREATE_DUMB, &mut request)?;

    let mut framebuffer = 0;
    let add_fb = unsafe {
      drmModeAddFB(
        fd,
        width,
        height,
        24,
        32,
        request.pitch,
        request.handle,
        &mut framebuffer,
      )
    };
    if add_fb != 0 {
      destroy_dumb(fd, request.handle);
      return Err(io::Error::last_os_error());
    }

    let mut map_request = MapDumb {
      handle: request.handle,
      pad: 0,
      offset: 0,
    };
    if let Err(error) = drm_call(fd, DRM_IOCTL_MODE_MAP_DUMB, &mut map_request) {
      unsafe { drmModeRmFB(fd, framebuffer) };
      destroy_dumb(fd, request.handle);
      return Err(error);
    }

    let mapped = unsafe {
      libc::mmap(
        std::ptr::null_mut(),
        request.size as usize,
        libc::PROT_READ | libc::PROT_WRITE,
        libc::MAP_SHARED,
        fd,
        map_request.offset as libc::off_t,
      )
    };
    if mapped == libc::MAP_FAILED {
      unsafe { drmModeRmFB(fd, framebuffer) };
      destroy_dumb(fd, request.handle);
      return Err(io::Error::last_os_error());
    }

    Ok(Self {
      fd,
      handle: request.handle,
      width,
      height,
      pitch: request.pitch,
      size: request.size as usize,
      map: mapped.cast(),
      framebuffer,
    })
  }
}

impl DumbBuffer {
  /// The mapped buffer as XRGB8888 pixels, `pitch / 4` per row.
  fn pixels(&mut self) -> &mut [u32] {
    // SAFETY: `map` is the live mmap of this dumb buffer, `size` bytes long and
    // aligned for u32. It stays mapped until `drop`, and the exclusive borrow of
    // `self` prevents another slice over it.
    unsafe { std::slice::from_raw_parts_mut(self.map.cast::<u32>(), self.size / 4) }
  }
}

impl Drop for DumbBuffer {
  fn drop(&mut self) {
    unsafe {
      libc::munmap(self.map.cast(), self.size);
      drmModeRmFB(self.fd, self.framebuffer);
    }
    destroy_dumb(self.fd, self.handle);
  }
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

/// A copy of the last frame on an overlay plane of the output's CRTC. The user
/// compositor only drives the primary and cursor planes, so the overlay stays
/// on screen through its first modeset. Dropping it removes the framebuffer,
/// and the kernel turns off that plane alone; that needs no DRM master.
struct Overlay {
  buffer: DumbBuffer,
}

/// Puts the released frame on a free overlay plane. Returns `None`, and the
/// surface falls back to the primary plane alone, when the CRTC has no free
/// overlay plane for XRGB8888 or the driver refuses it.
fn show_overlay(fd: RawFd, released: &Released, palette: &Palette) -> Option<Overlay> {
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
struct OverlayPlane {
  plane_id: u32,
  crtc_id: u32,
  possible_crtcs: u32,
  xrgb8888: bool,
}

/// The first plane that can show an XRGB8888 buffer on the CRTC at
/// `crtc_index` and that no CRTC uses yet.
fn free_overlay_plane(planes: &[OverlayPlane], crtc_index: usize) -> Option<u32> {
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
fn overlay_planes(fd: RawFd) -> Vec<OverlayPlane> {
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
fn crtc_index(fd: RawFd, crtc_id: u32) -> Option<usize> {
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
fn overlay_expired(elapsed: Duration, since_replaced: Option<Duration>) -> Option<&'static str> {
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
fn hold_overlay(
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

/// What the surface keeps after a `release` until the user compositor shows
/// its own first frame.
struct Released {
  stream: UnixStream,
  buffer: DumbBuffer,
  crtc_id: u32,
}

/// Waits until the CRTC no longer scans out `buffer`, which means the user
/// compositor committed its own frame, or until `REPLACEMENT_TIMEOUT`, and keeps
/// the spinner animating meanwhile. Writing to the buffer needs no DRM master:
/// it stays scanned out until the compositor's first modeset, and the squares
/// disappear only for the moment the compositor needs to serve its clients.
/// Returns whether the compositor replaced the frame before the timeout.
fn wait_for_replacement(
  fd: RawFd,
  crtc_id: u32,
  buffer: &mut DumbBuffer,
  palette: &Palette,
) -> bool {
  let framebuffer = buffer.framebuffer;
  let deadline = Instant::now() + REPLACEMENT_TIMEOUT;
  while Instant::now() < deadline {
    let Some((scanned_out, _)) = crtc_state(fd, crtc_id) else {
      return true;
    };
    if scanned_out != framebuffer {
      return true;
    }
    draw_spinner(buffer, palette, &spinner::frame(monotonic_millis()));
    thread::sleep(POLL_DELAY);
  }
  false
}

/// The mode the CRTC drives, when it has one.
fn crtc_mode(fd: RawFd, crtc_id: u32) -> Option<ModeInfo> {
  let crtc = unsafe { drmModeGetCrtc(fd, crtc_id) };
  if crtc.is_null() {
    return None;
  }
  let mode = unsafe { ((*crtc).mode_valid != 0).then_some((*crtc).mode) };
  unsafe { drmModeFreeCrtc(crtc) };
  mode
}

/// The framebuffer the CRTC scans out and whether it has a mode set.
fn crtc_state(fd: RawFd, crtc_id: u32) -> Option<(u32, bool)> {
  let crtc = unsafe { drmModeGetCrtc(fd, crtc_id) };
  if crtc.is_null() {
    return None;
  }
  let state = unsafe { ((*crtc).buffer_id, (*crtc).mode_valid != 0) };
  unsafe { drmModeFreeCrtc(crtc) };
  Some(state)
}

/// Appends `event` to `TIMING_LOG`. Timing is diagnostic, so a failure to
/// write never affects the surface.
fn mark(event: &str) {
  let line = format!("monotonic_ns={} mark {event}\n", monotonic_nanos());
  if let Ok(mut file) = OpenOptions::new()
    .create(true)
    .append(true)
    .open(TIMING_LOG)
  {
    let _ = file.write_all(line.as_bytes());
  }
}

/// Milliseconds of `CLOCK_MONOTONIC`, the clock the GTK splash also reads, so
/// both surfaces show the same spinner frame at the same instant.
fn monotonic_millis() -> u64 {
  monotonic_nanos() / 1_000_000
}

fn monotonic_nanos() -> u64 {
  let mut now = libc::timespec {
    tv_sec: 0,
    tv_nsec: 0,
  };
  // SAFETY: `now` is a valid, writable timespec for the duration of the call.
  if unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut now) } != 0 {
    return 0;
  }
  let seconds = u64::try_from(now.tv_sec).unwrap_or_default();
  let nanoseconds = u64::try_from(now.tv_nsec).unwrap_or_default();
  seconds * 1_000_000_000 + nanoseconds
}

/// Sends one command to the running surface and waits for its acknowledgement.
/// A missing socket means no surface is running, which is not an error.
fn send_command(socket: &Path, command: &str) -> io::Result<()> {
  let mut stream = match UnixStream::connect(socket) {
    Ok(stream) => stream,
    Err(error)
      if matches!(
        error.kind(),
        io::ErrorKind::NotFound | io::ErrorKind::ConnectionRefused
      ) =>
    {
      return Ok(());
    }
    Err(error) => return Err(error),
  };
  stream.set_read_timeout(Some(ACK_TIMEOUT))?;
  writeln!(stream, "{command}")?;
  let mut acknowledgement = String::new();
  BufReader::new(&mut stream).read_line(&mut acknowledgement)?;
  if acknowledgement.trim().is_empty() {
    return Err(io::Error::new(
      io::ErrorKind::UnexpectedEof,
      "handoff surface closed without acknowledging the command",
    ));
  }
  Ok(())
}

fn prepare_socket(path: &Path) -> io::Result<UnixListener> {
  if let Some(parent) = path.parent() {
    fs::create_dir_all(parent)?;
  }
  let _ = fs::remove_file(path);
  let listener = UnixListener::bind(path)?;
  fs::set_permissions(path, fs::Permissions::from_mode(0o666))?;
  listener.set_nonblocking(true)?;
  Ok(listener)
}

/// One CRTC as the handoff sees it while it waits for DRM master.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CrtcSample {
  crtc_id: u32,
  framebuffer: u32,
  mode_valid: bool,
}

/// Remembers the last state of every CRTC, so the wait logs only the changes.
#[derive(Debug, Default)]
struct CrtcWatch {
  last: Vec<CrtcSample>,
}

impl CrtcWatch {
  /// The samples of `current` that differ from the previous call.
  fn changes(&mut self, current: Vec<CrtcSample>) -> Vec<CrtcSample> {
    let changed = current
      .iter()
      .filter(|sample| !self.last.contains(sample))
      .copied()
      .collect();
    self.last = current;
    changed
  }
}

/// The framebuffer and mode state of every CRTC of `fd`. Reading it needs no
/// DRM master.
fn crtc_samples(fd: RawFd) -> Vec<CrtcSample> {
  let resources = unsafe { drmModeGetResources(fd) };
  if resources.is_null() {
    return Vec::new();
  }
  let resources_ref = unsafe { &*resources };
  let samples = (0..resources_ref.count_crtcs)
    .filter_map(|index| {
      let crtc_id = unsafe { *resources_ref.crtcs.add(index as usize) };
      crtc_state(fd, crtc_id).map(|(framebuffer, mode_valid)| CrtcSample {
        crtc_id,
        framebuffer,
        mode_valid,
      })
    })
    .collect();
  unsafe { drmModeFreeResources(resources) };
  samples
}

/// A descriptor on one DRM card, held open for the whole wait for DRM master.
struct Observer {
  name: String,
  file: fs::File,
  watch: CrtcWatch,
}

/// The DRM cards, in name order, and one observer on each.
///
/// When the greeter compositor's descriptor is the last one to close, the
/// kernel restores the fbdev console mode, which blanks the output before this
/// process can draw. The observers keep the device open, and they read the
/// CRTCs, so the timing log shows whether the output stayed on between the two
/// masters.
fn open_observers() -> io::Result<(Vec<PathBuf>, Vec<Observer>)> {
  let mut cards = fs::read_dir("/dev/dri")?
    .filter_map(Result::ok)
    .map(|entry| entry.path())
    .filter(|path| {
      path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with("card"))
    })
    .collect::<Vec<_>>();
  cards.sort();
  let observers = cards
    .iter()
    .filter_map(|card| {
      let file = OpenOptions::new().read(true).write(true).open(card).ok()?;
      let name = card.file_name()?.to_string_lossy().into_owned();
      Some(Observer {
        name,
        file,
        watch: CrtcWatch::default(),
      })
    })
    .collect();
  Ok((cards, observers))
}

/// Waits until a descriptor on one of `cards` becomes DRM master and returns
/// it. An observer that becomes master is taken out of `observers`; its
/// descriptor is returned without being closed.
fn acquire_card(cards: &[PathBuf], observers: &mut Vec<Observer>) -> io::Result<RawFd> {
  let deadline = Instant::now() + Duration::from_secs(15);
  loop {
    for observer in observers.iter_mut() {
      for sample in observer
        .watch
        .changes(crtc_samples(observer.file.as_raw_fd()))
      {
        mark(&format!(
          "handoff.crtc card={} crtc={} fb={} mode_valid={}",
          observer.name, sample.crtc_id, sample.framebuffer, sample.mode_valid
        ));
      }
    }
    // An observer opened while no process held DRM master became master
    // itself, and then no new descriptor can.
    if let Some(index) = observers
      .iter()
      .position(|observer| unsafe { drmSetMaster(observer.file.as_raw_fd()) } == 0)
    {
      return Ok(observers.swap_remove(index).file.into_raw_fd());
    }
    for card in cards {
      let Ok(file) = OpenOptions::new().read(true).write(true).open(card) else {
        continue;
      };
      let fd = file.as_raw_fd();
      let result = unsafe { drmSetMaster(fd) };
      if result == 0 {
        return Ok(file.into_raw_fd());
      }
    }
    if Instant::now() >= deadline {
      return Err(io::Error::new(
        io::ErrorKind::TimedOut,
        "DRM device did not become available",
      ));
    }
    thread::sleep(MASTER_POLL);
  }
}

/// The output the surface drives and its first frame, already painted.
struct PreparedSurface {
  connector_id: u32,
  crtc_id: u32,
  mode: ModeInfo,
  buffer: DumbBuffer,
}

/// Finds the output and paints a buffer for it with the background and the
/// spinner. None of this needs DRM master.
fn prepare_surface(fd: RawFd, palette: &Palette) -> io::Result<PreparedSurface> {
  let (connector_id, crtc_id, mode) = select_output(fd)?;
  let mut buffer = DumbBuffer::create(fd, mode.hdisplay as u32, mode.vdisplay as u32)?;
  // A new dumb buffer is black. Paint it before the modeset, or the output
  // shows that black for a moment between two spinner frames.
  draw_background(&mut buffer, palette);
  draw_spinner(&mut buffer, palette, &spinner::frame(monotonic_millis()));
  Ok(PreparedSurface {
    connector_id,
    crtc_id,
    mode,
    buffer,
  })
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

/// The first connected output, its CRTC and its preferred mode. The cached
/// connector state comes first: the compositor that just left has probed the
/// connectors, and probing again takes tens of milliseconds while the output
/// waits for a frame.
fn select_output(fd: RawFd) -> io::Result<(u32, u32, ModeInfo)> {
  find_output(fd, drmModeGetConnectorCurrent).or_else(|_| find_output(fd, drmModeGetConnector))
}

fn find_output(
  fd: RawFd,
  get_connector: unsafe extern "C" fn(libc::c_int, u32) -> *mut Connector,
) -> io::Result<(u32, u32, ModeInfo)> {
  let resources = unsafe { drmModeGetResources(fd) };
  if resources.is_null() {
    return Err(io::Error::last_os_error());
  }

  let result = (|| unsafe {
    let resources_ref = &*resources;
    for index in 0..resources_ref.count_connectors {
      let connector_id = *resources_ref.connectors.add(index as usize);
      let connector = get_connector(fd, connector_id);
      if connector.is_null() {
        continue;
      }
      let connector_ref = &*connector;
      if connector_ref.connection != 1 || connector_ref.count_modes == 0 {
        drmModeFreeConnector(connector);
        continue;
      }

      let encoder = if connector_ref.encoder_id != 0 {
        drmModeGetEncoder(fd, connector_ref.encoder_id)
      } else {
        std::ptr::null_mut()
      };
      if encoder.is_null() {
        drmModeFreeConnector(connector);
        continue;
      }
      let encoder_ref = &*encoder;
      let mode = *connector_ref.modes;
      let output = (connector_id, encoder_ref.crtc_id, mode);
      drmModeFreeEncoder(encoder);
      drmModeFreeConnector(connector);
      return Ok(output);
    }
    Err(io::Error::new(
      io::ErrorKind::NotFound,
      "no connected KMS output found",
    ))
  })();
  unsafe { drmModeFreeResources(resources) };
  result
}

/// A request a client sends to the surface, one per connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Command<'a> {
  /// Changes the theme of the frame.
  Theme(&'a str),
  /// Answered once the surface draws, so a client can wait for its first frame.
  Ping,
  /// Drops DRM master for the user compositor.
  Release,
  /// The session spinner is on screen; the overlay can go.
  Retire,
  Unknown,
}

fn parse_command(line: &str) -> Command<'_> {
  match line.trim() {
    "ping" => Command::Ping,
    "release" => Command::Release,
    "retire" => Command::Retire,
    other => other
      .strip_prefix("theme=")
      .map_or(Command::Unknown, Command::Theme),
  }
}

/// Reads the request line of one client. The socket is open to every local
/// user, so the read gives up after `CLIENT_TIMEOUT`.
fn read_command(stream: &mut UnixStream) -> io::Result<String> {
  stream.set_read_timeout(Some(CLIENT_TIMEOUT))?;
  stream.set_write_timeout(Some(CLIENT_TIMEOUT))?;
  let mut command = String::new();
  BufReader::new(&mut *stream).read_line(&mut command)?;
  Ok(command)
}

/// Handles one client request while the surface owns the output. A release
/// returns its stream still open, so the acknowledgement can be written after
/// DRM master has been dropped.
fn handle_command(mut stream: UnixStream, palette: &mut Palette) -> io::Result<Option<UnixStream>> {
  let line = read_command(&mut stream)?;
  match parse_command(&line) {
    Command::Theme(theme) => *palette = palette.with_theme(theme),
    Command::Release => return Ok(Some(stream)),
    // Nothing is on an overlay before the release.
    Command::Ping | Command::Retire => {}
    Command::Unknown => {
      stream.write_all(b"error\n")?;
      return Ok(None);
    }
  }
  stream.write_all(b"ok\n")?;
  Ok(None)
}

/// Paints the whole buffer with the background color.
fn draw_background(buffer: &mut DumbBuffer, palette: &Palette) {
  buffer.pixels().fill(0xff00_0000 | palette.background);
}

/// Draws the shared spinner (`argvus_loading_theme_core::spinner`) at the
/// center of the output.
fn draw_spinner(buffer: &mut DumbBuffer, palette: &Palette, frame: &spinner::Frame) {
  let image = Image {
    width: buffer.width as usize,
    height: buffer.height as usize,
    stride: (buffer.pitch / 4) as usize,
  };
  paint_spinner(buffer.pixels(), image, palette, frame);
}

/// The layout of an XRGB8888 image: its size and the pixels per row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Image {
  width: usize,
  height: usize,
  stride: usize,
}

/// A rectangle of pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Area {
  x: usize,
  y: usize,
  width: usize,
  height: usize,
}

/// Paints the spinner at the center of `pixels` and returns the area it wrote.
/// Each pixel of that area is written once with its final color, so a
/// scanned-out buffer never shows a half-drawn frame, and the area covers every
/// position of the animation, so it also erases the previous frame.
fn paint_spinner(
  pixels: &mut [u32],
  image: Image,
  palette: &Palette,
  frame: &spinner::Frame,
) -> Area {
  let center_x = image.width as f64 / 2.0;
  let center_y = image.height as f64 / 2.0;
  // One extra pixel on each side for the anti-aliased edge of the squares.
  let half_side = (spinner::EXTENT / 2.0).ceil() as usize + 1;
  let first_x = (center_x as usize).saturating_sub(half_side);
  let last_x = (center_x as usize + half_side).min(image.width.saturating_sub(1));
  let first_y = (center_y as usize).saturating_sub(half_side);
  let last_y = (center_y as usize + half_side).min(image.height.saturating_sub(1));
  for y in first_y..=last_y {
    for x in first_x..=last_x {
      let color = spinner_pixel(palette, frame, x as f64 - center_x, y as f64 - center_y);
      pixels[y * image.stride + x] = 0xff00_0000 | color;
    }
  }
  Area {
    x: first_x,
    y: first_y,
    width: last_x + 1 - first_x,
    height: last_y + 1 - first_y,
  }
}

/// The color of the one-pixel square whose top-left corner is at (`left`,
/// `top`) from the spinner center. Each square covers the pixel in proportion to
/// the area they share, which gives the same anti-aliased edge cairo draws in
/// the GTK splash.
fn spinner_pixel(palette: &Palette, frame: &spinner::Frame, left: f64, top: f64) -> u32 {
  let mut color = palette.background;
  for square in frame.squares {
    let coverage = overlap(square.x, square.x + spinner::SQUARE, left, left + 1.0)
      * overlap(square.y, square.y + spinner::SQUARE, top, top + 1.0);
    if coverage > 0.0 {
      color = mix_rgb(color, palette.accent, coverage);
    }
  }
  color
}

/// Length of the overlap between the intervals [`start`, `end`] and [`from`, `to`].
fn overlap(start: f64, end: f64, from: f64, to: f64) -> f64 {
  (end.min(to) - start.max(from)).max(0.0)
}

/// `base` moved toward `over` by `amount` (0 keeps `base`, 1 gives `over`).
fn mix_rgb(base: u32, over: u32, amount: f64) -> u32 {
  let channel = |shift: u32| {
    let from = f64::from((base >> shift) & 0xff);
    let to = f64::from((over >> shift) & 0xff);
    ((from + (to - from) * amount).round() as u32) << shift
  };
  channel(16) | channel(8) | channel(0)
}

fn drm_call<T>(fd: RawFd, request: libc::c_ulong, value: &mut T) -> io::Result<()> {
  let result = unsafe { drmIoctl(fd, request, value as *mut T as *mut libc::c_void) };
  if result == 0 {
    Ok(())
  } else {
    Err(io::Error::last_os_error())
  }
}

fn destroy_dumb(fd: RawFd, handle: u32) {
  let mut request = DestroyDumb { handle };
  let _ = drm_call(fd, DRM_IOCTL_MODE_DESTROY_DUMB, &mut request);
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn palette_uses_the_splash_theme_table() {
    assert_eq!(Palette::new("ARGVUS Light", None).background, 0xffffff);
    assert_eq!(Palette::new("tokyo-night", None).background, 0x1a1b26);
    assert_eq!(Palette::new("tokyo-night", None).accent, 0x7aa2f7);
    assert_eq!(Palette::new("solitude", None).background, 0x101315);
    assert_eq!(Palette::new("solitude", None).accent, 0x798186);
    assert_eq!(Palette::new("argvus-dark", None).background, 0x191b27);
  }

  #[test]
  fn unknown_themes_use_the_splash_defaults() {
    let palette = Palette::new("custom", None);
    assert_eq!(Some(palette.background), hex_to_rgb(DEFAULT_BACKGROUND));
    assert_eq!(Some(palette.accent), hex_to_rgb(DEFAULT_ACCENT));
  }

  #[test]
  fn the_user_accent_survives_a_theme_change() {
    let palette = Palette::new("argvus-dark", Some(0x12_3456));
    assert_eq!(palette.accent, 0x12_3456);
    let changed = palette.with_theme("tokyo-night");
    assert_eq!(changed.background, 0x1a1b26);
    assert_eq!(changed.accent, 0x12_3456);
  }

  #[test]
  fn spinner_pixels_show_the_squares_and_the_background() {
    let palette = Palette::new("argvus-dark", Some(0xff_0000));
    let rest = spinner::frame(0);
    // The top-left square covers this pixel completely.
    assert_eq!(spinner_pixel(&palette, &rest, -20.0, -20.0), 0xff_0000);
    assert_eq!(
      spinner_pixel(&palette, &rest, 25.0, 25.0),
      palette.background
    );
    // A pixel that straddles the left edge of the top-left square is half covered.
    assert_eq!(
      spinner_pixel(&palette, &rest, -20.5, -20.0),
      mix_rgb(palette.background, 0xff_0000, 0.5)
    );
  }

  fn mode(clock: u32, vrefresh: u32) -> ModeInfo {
    ModeInfo {
      clock,
      hdisplay: 1920,
      hsync_start: 2008,
      hsync_end: 2052,
      htotal: 2200,
      hskew: 0,
      vdisplay: 1080,
      vsync_start: 1084,
      vsync_end: 1089,
      vtotal: 1125,
      vscan: 0,
      vrefresh,
      flags: 5,
      mode_type: 0,
      name: [0; 32],
    }
  }

  #[test]
  fn modes_with_the_same_timings_need_no_modeset() {
    let mut named = mode(185_580, 75);
    named.mode_type = 0x48;
    named.name[0] = b'1' as libc::c_char;
    assert!(mode(185_580, 75).same_timings(&named));
    assert!(!mode(185_580, 75).same_timings(&mode(148_500, 60)));
  }

  #[test]
  fn commands_are_parsed_from_their_line() {
    assert_eq!(parse_command("ping\n"), Command::Ping);
    assert_eq!(parse_command("release\n"), Command::Release);
    assert_eq!(parse_command("retire"), Command::Retire);
    assert_eq!(parse_command("theme=nord\n"), Command::Theme("nord"));
    assert_eq!(parse_command("reboot"), Command::Unknown);
  }

  #[test]
  fn the_overlay_is_a_free_xrgb_plane_of_the_crtc() {
    let plane = |plane_id, crtc_id, possible_crtcs, xrgb8888| OverlayPlane {
      plane_id,
      crtc_id,
      possible_crtcs,
      xrgb8888,
    };
    let planes = [
      plane(58, 0, 0b010, true),
      plane(68, 57, 0b001, true),
      plane(78, 0, 0b001, false),
      plane(88, 0, 0b001, true),
    ];
    assert_eq!(free_overlay_plane(&planes, 0), Some(88));
    assert_eq!(free_overlay_plane(&planes, 1), Some(58));
    assert_eq!(free_overlay_plane(&planes, 2), None);
    assert_eq!(free_overlay_plane(&planes, 40), None);
  }

  #[test]
  fn the_overlay_leaves_on_its_own_only_after_a_timeout() {
    assert_eq!(overlay_expired(Duration::from_secs(1), None), None);
    assert_eq!(
      overlay_expired(Duration::from_secs(5), Some(Duration::from_secs(1))),
      None
    );
    assert_eq!(
      overlay_expired(Duration::from_secs(5), Some(RETIRE_TIMEOUT)),
      Some("handoff.retire-timeout")
    );
    assert_eq!(
      overlay_expired(REPLACEMENT_TIMEOUT, None),
      Some("handoff.replacement-timeout")
    );
  }

  #[test]
  fn the_crtc_watch_reports_only_changes() {
    let on = CrtcSample {
      crtc_id: 57,
      framebuffer: 111,
      mode_valid: true,
    };
    let off = CrtcSample {
      framebuffer: 0,
      mode_valid: false,
      ..on
    };
    let mut watch = CrtcWatch::default();
    assert_eq!(watch.changes(vec![on]), vec![on]);
    assert!(watch.changes(vec![on]).is_empty());
    assert_eq!(watch.changes(vec![off]), vec![off]);
  }

  /// Binds a socket in `directory` and serves one request through
  /// `handle_command`, returning the socket path so a client can connect.
  fn serve_once(
    directory: &Path,
    palette_theme: &'static str,
  ) -> (PathBuf, thread::JoinHandle<()>) {
    let socket = directory.join("handoff.sock");
    let listener = UnixListener::bind(&socket).expect("bind test socket");
    let handle = thread::spawn(move || {
      let (stream, _) = listener.accept().expect("accept test client");
      let mut palette = Palette::new(palette_theme, None);
      if let Ok(Some(mut release)) = handle_command(stream, &mut palette) {
        // Mirrors main: the acknowledgement follows the DRM release.
        release
          .write_all(b"released\n")
          .expect("acknowledge release");
      }
    });
    (socket, handle)
  }

  fn test_directory(name: &str) -> PathBuf {
    let directory =
      std::env::temp_dir().join(format!("argvus-handoff-test-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&directory);
    fs::create_dir_all(&directory).expect("create test directory");
    directory
  }

  #[test]
  fn a_silent_client_cannot_stall_the_surface() {
    let directory = test_directory("silent");
    let socket = directory.join("handoff.sock");
    let listener = UnixListener::bind(&socket).expect("bind test socket");
    let _client = UnixStream::connect(&socket).expect("connect silent client");
    let (stream, _) = listener.accept().expect("accept silent client");
    let started = Instant::now();
    let mut palette = Palette::new("argvus-dark", None);
    assert!(handle_command(stream, &mut palette).is_err());
    assert!(started.elapsed() < CLIENT_TIMEOUT + Duration::from_secs(1));
    let _ = fs::remove_dir_all(&directory);
  }

  #[test]
  fn sending_without_a_running_surface_succeeds() {
    let directory = test_directory("absent");
    assert!(send_command(&directory.join("missing.sock"), "release").is_ok());
    let _ = fs::remove_dir_all(&directory);
  }

  #[test]
  fn release_is_acknowledged_by_the_surface() {
    let directory = test_directory("release");
    let (socket, server) = serve_once(&directory, "argvus-dark");
    assert!(send_command(&socket, "release").is_ok());
    server.join().expect("surface thread");
    let _ = fs::remove_dir_all(&directory);
  }

  #[test]
  fn theme_update_is_acknowledged_and_keeps_the_surface_running() {
    let directory = test_directory("theme");
    let (socket, server) = serve_once(&directory, "argvus-dark");
    assert!(send_command(&socket, "theme=tokyo-night").is_ok());
    server.join().expect("surface thread");
    let _ = fs::remove_dir_all(&directory);
  }
}
