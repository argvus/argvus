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

const DEFAULT_SOCKET: &str = "/run/argvus-greeter/handoff.sock";
const DEFAULT_THEME: &str = "argvus-dark";
/// How often the surface redraws the spinner and checks for commands.
const POLL_DELAY: Duration = Duration::from_millis(10);
/// How long the last frame stays on screen after a release while the user
/// compositor takes over the output.
const REPLACEMENT_TIMEOUT: Duration = Duration::from_secs(10);
/// Interval between the checks for the compositor's first frame.
const REPLACEMENT_POLL: Duration = Duration::from_millis(20);
/// Interval between the attempts to become DRM master.
const MASTER_POLL: Duration = Duration::from_millis(10);
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
  let listener = prepare_socket(&cli.socket)?;
  let user_accent = cli.accent.as_deref().and_then(hex_to_rgb);
  let mut palette = Palette::new(&cli.theme, user_accent);
  let fd = acquire_card()?;

  let result = run_surface(fd, &listener, &mut palette);
  unsafe {
    drmDropMaster(fd);
  }
  let _ = fs::remove_file(&cli.socket);
  let outcome = match result {
    Ok(mut released) => {
      // The release is acknowledged only after DRM master has been dropped, so
      // the caller can start the user compositor without racing this process.
      let _ = released.stream.write_all(b"released\n");
      // Destroying the framebuffer now would make the kernel turn the output
      // off until the compositor's first modeset: a black flash between the
      // two spinners. The last frame stays until the compositor replaces it.
      wait_for_replacement(fd, released.crtc_id, released.buffer.framebuffer);
      drop(released.buffer);
      Ok(())
    }
    Err(error) => Err(error),
  };
  unsafe {
    libc::close(fd);
  }
  outcome
}

/// What the surface keeps after a `release` until the user compositor shows
/// its own first frame.
struct Released {
  stream: UnixStream,
  buffer: DumbBuffer,
  crtc_id: u32,
}

/// Waits until the CRTC no longer scans out `framebuffer`, which means the
/// user compositor committed its own frame, or until `REPLACEMENT_TIMEOUT`.
fn wait_for_replacement(fd: RawFd, crtc_id: u32, framebuffer: u32) {
  let deadline = Instant::now() + REPLACEMENT_TIMEOUT;
  while Instant::now() < deadline {
    let crtc = unsafe { drmModeGetCrtc(fd, crtc_id) };
    if crtc.is_null() {
      return;
    }
    let scanned_out = unsafe { (*crtc).buffer_id };
    unsafe { drmModeFreeCrtc(crtc) };
    if scanned_out != framebuffer {
      return;
    }
    thread::sleep(REPLACEMENT_POLL);
  }
}

/// Milliseconds of `CLOCK_MONOTONIC`, the clock the GTK splash also reads, so
/// both surfaces show the same spinner frame at the same instant.
fn monotonic_millis() -> u64 {
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
  seconds * 1000 + nanoseconds / 1_000_000
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

fn acquire_card() -> io::Result<RawFd> {
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

  let deadline = Instant::now() + Duration::from_secs(15);
  loop {
    for card in &cards {
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

fn run_surface(fd: RawFd, listener: &UnixListener, palette: &mut Palette) -> io::Result<Released> {
  let (connector_id, crtc_id, mode) = select_output(fd)?;
  let mut connector = connector_id;
  let mut buffer = DumbBuffer::create(fd, mode.hdisplay as u32, mode.vdisplay as u32)?;
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

  draw_background(&mut buffer, palette);
  loop {
    while let Ok((stream, _)) = listener.accept() {
      let previous = *palette;
      match handle_command(stream, palette) {
        Ok(Some(stream)) => {
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

fn select_output(fd: RawFd) -> io::Result<(u32, u32, ModeInfo)> {
  let resources = unsafe { drmModeGetResources(fd) };
  if resources.is_null() {
    return Err(io::Error::last_os_error());
  }

  let result = (|| unsafe {
    let resources_ref = &*resources;
    for index in 0..resources_ref.count_connectors {
      let connector_id = *resources_ref.connectors.add(index as usize);
      let connector = drmModeGetConnector(fd, connector_id);
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

/// Handles one client request. A release returns its stream still open, so the
/// acknowledgement can be written after DRM master has been dropped.
fn handle_command(mut stream: UnixStream, palette: &mut Palette) -> io::Result<Option<UnixStream>> {
  stream.set_read_timeout(Some(CLIENT_TIMEOUT))?;
  stream.set_write_timeout(Some(CLIENT_TIMEOUT))?;
  let mut command = String::new();
  BufReader::new(&mut stream).read_line(&mut command)?;
  let command = command.trim();
  if let Some(theme) = command.strip_prefix("theme=") {
    *palette = palette.with_theme(theme);
    stream.write_all(b"ok\n")?;
    return Ok(None);
  }
  if command == "release" {
    return Ok(Some(stream));
  }
  stream.write_all(b"error\n")?;
  Ok(None)
}

/// Paints the whole buffer with the background color.
fn draw_background(buffer: &mut DumbBuffer, palette: &Palette) {
  buffer.pixels().fill(0xff00_0000 | palette.background);
}

/// Draws the shared spinner (`argvus_loading_theme_core::spinner`) at the
/// center of the output. Each pixel of the spinner area is written once with
/// its final color, so the scanned-out buffer never shows a half-drawn frame.
fn draw_spinner(buffer: &mut DumbBuffer, palette: &Palette, frame: &spinner::Frame) {
  let width = buffer.width as i64;
  let height = buffer.height as i64;
  let stride = i64::from(buffer.pitch / 4);
  let center_x = width as f64 / 2.0;
  let center_y = height as f64 / 2.0;
  // One extra pixel on each side for the anti-aliased edge of the squares.
  let half_side = (spinner::EXTENT / 2.0).ceil() as i64 + 1;
  let first_x = (center_x as i64 - half_side).max(0);
  let last_x = (center_x as i64 + half_side).min(width - 1);
  let first_y = (center_y as i64 - half_side).max(0);
  let last_y = (center_y as i64 + half_side).min(height - 1);
  let pixels = buffer.pixels();
  for y in first_y..=last_y {
    for x in first_x..=last_x {
      let color = spinner_pixel(palette, frame, x as f64 - center_x, y as f64 - center_y);
      pixels[(y * stride + x) as usize] = 0xff00_0000 | color;
    }
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
