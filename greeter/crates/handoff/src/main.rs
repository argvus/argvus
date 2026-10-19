//! Direct DRM/KMS transition surface used while greetd changes compositors.
//!
//! The normal ARGVUS splash is a Wayland client and therefore cannot render
//! while the greeter compositor has already exited and the user compositor has
//! not acquired the display yet. This small process owns a DRM dumb buffer for
//! that exact interval and releases it through a private handoff socket.

use std::{
  fs::{self, OpenOptions},
  io::{self, BufRead, BufReader},
  os::fd::{AsRawFd, IntoRawFd, RawFd},
  os::unix::fs::PermissionsExt,
  os::unix::net::{UnixListener, UnixStream},
  path::{Path, PathBuf},
  thread,
  time::{Duration, Instant},
};

use clap::Parser;

const DEFAULT_SOCKET: &str = "/run/argvus-greeter/handoff.sock";
const DEFAULT_THEME: &str = "argvus-dark";
const FRAME_DELAY: Duration = Duration::from_millis(90);
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
}

#[derive(Clone, Copy)]
struct Palette {
  background: u32,
  accent: u32,
}

impl Palette {
  fn from_theme(theme: &str) -> Self {
    let normalized = theme.trim().to_ascii_lowercase().replace(['_', ' '], "-");
    let normalized = normalized.strip_prefix("argvus-").unwrap_or(&normalized);
    let normalized = normalized.strip_suffix("-float").unwrap_or(normalized);

    let (background, accent) = match normalized {
      "dark" | "dark-aether" => (0x191b27, 0x3590bd),
      "light" | "light-veil" => (0xffffff, 0x000000),
      "dracula" => (0x282a36, 0xbd93f9),
      "onedark" | "one-dark" => (0x282c34, 0x61afef),
      "dark-silver" | "silver-dark" => (0x595959, 0x333647),
      "dark-slate" | "slate-dark" => (0x3b4352, 0x7391a5),
      "dark-universe" | "universe" => (0x000000, 0xffffff),
      "dark-gruvbox-high" | "gruvbox-high-dark" => (0x282828, 0xD79921),
      "dark-gruvbox" | "gruvbox-dark" => (0x282828, 0xD4BE98),
      "dark-rose-pine" | "rose-pine" => (0x191724, 0xc4a7e7),
      "dark-tokio-night" | "dark-tokyo-night" | "tokyo-night" => (0x1a1b26, 0x7aa2f7),
      "github-light" => (0xffffff, 0x0969da),
      "one-light" => (0xfafafa, 0x4078f2),
      "everforest-light" | "everforest-light-float" => (0xfdf6e3, 0x3a94c5),
      "solarized-light" | "light-solarized" => (0xfdf6e3, 0x268bd2),
      "catppuccin-latte" | "light-catppuccin-latte" | "dark-catppuccin-latte" => {
        (0xeff1f5, 0x1e66f5)
      }
      "gruvbox-light" | "light-gruvbox" => (0xfbf1c7, 0x458588),
      "solitude" => (0x101315, 0x798186),
      "dark-sunset" | "sunset" => (0x0f0f0f, 0xe2be8a),
      "matrix" => (0x0b0c16, 0x82fb9c),
      "monokai-dark" => (0x2d2a2e, 0x78dce8),
      "dark-matrix" => (0x0b0c16, 0x82fb9c),
      "dark-monokai" => (0x2d2a2e, 0x78dce8),
      _ => (0x191b27, 0x3590bd),
    };

    Self { background, accent }
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
      pitch: request.pitch,
      size: request.size as usize,
      map: mapped.cast(),
      framebuffer,
    })
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
  let listener = prepare_socket(&cli.socket)?;
  let mut palette = Palette::from_theme(&cli.theme);
  let fd = acquire_card()?;

  let result = run_surface(fd, &listener, &mut palette);
  unsafe {
    drmDropMaster(fd);
    libc::close(fd);
  }
  let _ = fs::remove_file(&cli.socket);
  result
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
    thread::sleep(Duration::from_millis(50));
  }
}

fn run_surface(fd: RawFd, listener: &UnixListener, palette: &mut Palette) -> io::Result<()> {
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

  let mut frame = 0usize;
  loop {
    while let Ok((stream, _)) = listener.accept() {
      if handle_command(stream, palette)? {
        return Ok(());
      }
    }
    draw_frame(&mut buffer, palette, frame);
    frame = frame.wrapping_add(1);
    thread::sleep(FRAME_DELAY);
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

fn handle_command(mut stream: UnixStream, palette: &mut Palette) -> io::Result<bool> {
  let mut command = String::new();
  BufReader::new(&mut stream).read_line(&mut command)?;
  let command = command.trim();
  if let Some(theme) = command.strip_prefix("theme=") {
    *palette = Palette::from_theme(theme);
    return Ok(false);
  }
  Ok(command == "release")
}

fn draw_frame(buffer: &mut DumbBuffer, palette: &Palette, frame: usize) {
  let width = (buffer.pitch / 4) as i32;
  let height = (buffer.size as u32 / buffer.pitch) as i32;
  let pixels = unsafe { std::slice::from_raw_parts_mut(buffer.map.cast::<u32>(), buffer.size / 4) };
  pixels.fill(0xff00_0000 | palette.background);

  let cx = width / 2;
  let cy = height / 2;
  let radius = 28i32;
  let active = frame % 12;
  for spoke in 0..12 {
    let angle = std::f32::consts::TAU * spoke as f32 / 12.0;
    let x = cx + (angle.cos() * radius as f32) as i32;
    let y = cy + (angle.sin() * radius as f32) as i32;
    let color = if spoke == active {
      0xff00_0000 | palette.accent
    } else {
      0x6640_4040
    };
    draw_disc(pixels, width, height, x, y, 5, color);
  }
}

fn draw_disc(
  pixels: &mut [u32],
  width: i32,
  height: i32,
  cx: i32,
  cy: i32,
  radius: i32,
  color: u32,
) {
  for y in (cy - radius)..=(cy + radius) {
    for x in (cx - radius)..=(cx + radius) {
      if x < 0 || y < 0 || x >= width || y >= height {
        continue;
      }
      let dx = x - cx;
      let dy = y - cy;
      if dx * dx + dy * dy <= radius * radius {
        pixels[(y * width + x) as usize] = color;
      }
    }
  }
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
  fn supported_themes_map_to_expected_colors() {
    assert_eq!(Palette::from_theme("ARGVUS Light").background, 0xffffff);
    assert_eq!(Palette::from_theme("universe-float").accent, 0xffffff);
    assert_eq!(Palette::from_theme("tokyo-night").background, 0x1a1b26);
    assert_eq!(Palette::from_theme("tokyo-night").accent, 0x7aa2f7);
    assert_eq!(Palette::from_theme("solitude").background, 0x101315);
    assert_eq!(Palette::from_theme("solitude").accent, 0x798186);
    assert_eq!(Palette::from_theme("sunset").background, 0x0f0f0f);
    assert_eq!(Palette::from_theme("sunset").accent, 0xe2be8a);
    assert_eq!(Palette::from_theme("matrix").background, 0x0b0c16);
    assert_eq!(Palette::from_theme("matrix").accent, 0x82fb9c);
    assert_eq!(Palette::from_theme("monokai-dark").background, 0x2d2a2e);
    assert_eq!(Palette::from_theme("monokai-dark").accent, 0x78dce8);
  }

  #[test]
  fn unknown_themes_fall_back_to_aether() {
    assert_eq!(Palette::from_theme("custom").background, 0x191b27);
    assert_eq!(Palette::from_theme("custom").accent, 0x3590bd);
  }
}
