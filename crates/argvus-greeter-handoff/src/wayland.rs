//! Wayland mode: the login spinner as a layer-shell overlay, without GTK.
//!
//! The session starts this surface as soon as the user compositor publishes its
//! socket, and the greeter shows it before the greeter compositor exits. A
//! shared-memory client maps its first frame within milliseconds of
//! connecting, where the GTK splash needed about 300 ms, so the compositor's
//! first frames no longer show its background without the spinner.

use std::{
  fs::File,
  io::{self, Write},
  os::fd::{AsFd, AsRawFd, FromRawFd, OwnedFd},
  path::Path,
  sync::atomic::{AtomicBool, Ordering},
  thread,
};

use wayland_client::{
  Connection, Dispatch, QueueHandle,
  backend::WaylandError,
  delegate_noop,
  globals::{GlobalListContents, registry_queue_init},
  protocol::{
    wl_buffer, wl_callback, wl_compositor, wl_output, wl_region, wl_registry, wl_shm, wl_shm_pool,
    wl_surface,
  },
};
use wayland_protocols_wlr::layer_shell::v1::client::{zwlr_layer_shell_v1, zwlr_layer_surface_v1};

use crate::{Image, Palette, monotonic_millis, monotonic_nanos, paint_spinner, spinner};

/// The layer namespace of the session-loading splash. The Hyprland
/// configuration matches it to map the splash without an animation.
const NAMESPACE: &str = "argvus-loading-theme";
/// Longest wait for compositor events before checking for a stop request. A
/// signal interrupts the wait, so this only bounds a missed wake-up.
const POLL_TIMEOUT_MS: libc::c_int = 100;
/// Two buffers: one on screen, one being painted.
const BUFFERS_PER_SURFACE: usize = 2;

static STOP_REQUESTED: AtomicBool = AtomicBool::new(false);

/// Writes a login mark to stderr, which the session journal keeps, on the
/// `CLOCK_MONOTONIC` clock the greeter and the session logs use.
fn log_mark(event: &str) {
  eprintln!(
    "argvus-greeter-handoff: monotonic_ns={} {event}",
    monotonic_nanos()
  );
}

extern "C" fn request_stop(_signal: libc::c_int) {
  STOP_REQUESTED.store(true, Ordering::Relaxed);
}

/// Shows the spinner on every output of the compositor in `WAYLAND_DISPLAY`
/// until SIGTERM or SIGINT, or until the compositor goes away.
///
/// # Errors
///
/// Returns an error when it cannot connect or the compositor lacks a required
/// global, so the caller can retry on another socket.
pub fn run(palette: Palette, ready_fd: Option<i32>, retire_handoff: bool) -> io::Result<()> {
  for signal in [libc::SIGTERM, libc::SIGINT] {
    let handler = request_stop as extern "C" fn(libc::c_int);
    // SAFETY: the handler only stores to an atomic, which is async-signal-safe.
    unsafe {
      libc::signal(signal, handler as libc::sighandler_t);
    }
  }

  let connection = Connection::connect_to_env().map_err(io::Error::other)?;
  log_mark("connected");
  let (globals, mut queue) =
    registry_queue_init::<Splash>(&connection).map_err(io::Error::other)?;
  log_mark("registry ready");
  let qh = queue.handle();
  let compositor: wl_compositor::WlCompositor =
    globals.bind(&qh, 4..=6, ()).map_err(io::Error::other)?;
  let shm: wl_shm::WlShm = globals.bind(&qh, 1..=1, ()).map_err(io::Error::other)?;
  let layer_shell: zwlr_layer_shell_v1::ZwlrLayerShellV1 =
    globals.bind(&qh, 1..=4, ()).map_err(io::Error::other)?;
  let outputs: Vec<wl_output::WlOutput> = globals
    .contents()
    .clone_list()
    .into_iter()
    .filter(|global| global.interface == "wl_output")
    .map(|global| {
      globals
        .registry()
        .bind(global.name, global.version.min(4), &qh, ())
    })
    .collect();

  let mut splash = Splash {
    palette,
    compositor,
    shm,
    layer_shell,
    surfaces: Vec::new(),
    ready_fd,
    retire_handoff,
  };
  if outputs.is_empty() {
    splash.add_surface(None, &qh);
  }
  for output in &outputs {
    splash.add_surface(Some(output), &qh);
  }

  loop {
    if STOP_REQUESTED.load(Ordering::Relaxed) {
      log_mark("stop requested");
      return Ok(());
    }
    // After the surfaces exist, a broken connection means the compositor is
    // gone (the greeter compositor exits on purpose); that ends the splash.
    if queue.dispatch_pending(&mut splash).is_err() || splash.all_closed() {
      return Ok(());
    }
    match queue.flush() {
      Ok(()) => {}
      Err(WaylandError::Io(error)) if error.kind() == io::ErrorKind::WouldBlock => {}
      Err(_) => return Ok(()),
    }
    let Some(guard) = queue.prepare_read() else {
      continue;
    };
    let mut poll_fd = libc::pollfd {
      fd: guard.connection_fd().as_raw_fd(),
      events: libc::POLLIN,
      revents: 0,
    };
    // SAFETY: `poll_fd` is a valid pollfd for the duration of the call.
    let ready = unsafe { libc::poll(&mut poll_fd, 1, POLL_TIMEOUT_MS) };
    if ready > 0 && guard.read().is_err() {
      return Ok(());
    }
  }
}

struct Splash {
  palette: Palette,
  compositor: wl_compositor::WlCompositor,
  shm: wl_shm::WlShm,
  layer_shell: zwlr_layer_shell_v1::ZwlrLayerShellV1,
  surfaces: Vec<SplashSurface>,
  ready_fd: Option<i32>,
  /// Sends `retire` to the DRM handoff surface once every output shows a
  /// frame; cleared after it is sent.
  retire_handoff: bool,
}

struct SplashSurface {
  surface: wl_surface::WlSurface,
  layer: zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
  /// Size in surface coordinates from the last configure; zero until then.
  size: (u32, u32),
  scale: i32,
  buffers: Vec<ShmBuffer>,
  frame_pending: bool,
  /// Both buffers were busy when a frame was due; draw on the next release.
  waiting_for_buffer: bool,
  presented: bool,
  closed: bool,
}

impl Splash {
  fn add_surface(&mut self, output: Option<&wl_output::WlOutput>, qh: &QueueHandle<Self>) {
    let index = self.surfaces.len();
    let surface = self.compositor.create_surface(qh, index);
    let layer = self.layer_shell.get_layer_surface(
      &surface,
      output,
      zwlr_layer_shell_v1::Layer::Overlay,
      NAMESPACE.to_owned(),
      qh,
      index,
    );
    layer.set_anchor(
      zwlr_layer_surface_v1::Anchor::Top
        | zwlr_layer_surface_v1::Anchor::Bottom
        | zwlr_layer_surface_v1::Anchor::Left
        | zwlr_layer_surface_v1::Anchor::Right,
    );
    // Cover the whole output, including the area of exclusive panels.
    layer.set_exclusive_zone(-1);
    layer.set_keyboard_interactivity(zwlr_layer_surface_v1::KeyboardInteractivity::None);
    layer.set_size(0, 0);
    surface.commit();
    self.surfaces.push(SplashSurface {
      surface,
      layer,
      size: (0, 0),
      scale: 1,
      buffers: Vec::new(),
      frame_pending: false,
      waiting_for_buffer: false,
      presented: false,
      closed: false,
    });
  }

  fn all_closed(&self) -> bool {
    self.surfaces.iter().all(|surface| surface.closed)
  }

  /// Paints and commits the next frame of surface `index`, unless a frame is
  /// already pending or no buffer is free.
  fn draw(&mut self, index: usize, qh: &QueueHandle<Self>) {
    let palette = self.palette;
    let shm = &self.shm;
    let Some(state) = self.surfaces.get_mut(index) else {
      return;
    };
    if state.closed || state.frame_pending || state.size.0 == 0 || state.size.1 == 0 {
      return;
    }
    let scale = state.scale.max(1) as u32;
    let (width, height) = (state.size.0 * scale, state.size.1 * scale);
    if state
      .buffers
      .first()
      .is_none_or(|buffer| buffer.width != width || buffer.height != height)
    {
      state.buffers.clear();
      for slot in 0..BUFFERS_PER_SURFACE {
        match ShmBuffer::new(shm, qh, width, height, (index, slot)) {
          Ok(buffer) => state.buffers.push(buffer),
          Err(error) => {
            eprintln!("argvus-greeter-handoff: cannot allocate a splash buffer: {error}");
            state.buffers.clear();
            return;
          }
        }
      }
      state.surface.set_buffer_scale(scale as i32);
    }

    let SplashSurface {
      surface,
      buffers,
      frame_pending,
      waiting_for_buffer,
      presented,
      ..
    } = state;
    let Some(buffer) = buffers.iter_mut().find(|buffer| !buffer.busy) else {
      *waiting_for_buffer = true;
      return;
    };
    let image = Image {
      width: width as usize,
      height: height as usize,
      stride: width as usize,
    };
    let full = !buffer.painted;
    if full {
      buffer.pixels().fill(0xff00_0000 | palette.background);
      buffer.painted = true;
    }
    let area = paint_spinner(
      buffer.pixels(),
      image,
      &palette,
      &spinner::frame(monotonic_millis()),
    );
    buffer.busy = true;
    surface.attach(Some(&buffer.buffer), 0, 0);
    if full {
      surface.damage_buffer(0, 0, width as i32, height as i32);
    } else {
      surface.damage_buffer(
        area.x as i32,
        area.y as i32,
        area.width as i32,
        area.height as i32,
      );
    }
    surface.frame(qh, index);
    *frame_pending = true;
    surface.commit();
    if index == 0 && !*presented && full {
      log_mark("first commit");
    }
  }

  /// Writes `READY` once every surface still open has shown a frame, and then
  /// lets the DRM handoff surface remove the overlay above this compositor.
  fn report_ready_when_presented(&mut self) {
    if !self
      .surfaces
      .iter()
      .all(|surface| surface.closed || surface.presented)
    {
      return;
    }
    if std::mem::take(&mut self.retire_handoff) {
      log_mark("retiring the DRM handoff surface");
      // Off the event loop, so a slow acknowledgement cannot stall the
      // animation. Without a running surface there is nothing to retire.
      thread::spawn(|| {
        let _ = crate::send_command(Path::new(crate::DEFAULT_SOCKET), "retire");
      });
    }
    let Some(fd) = self.ready_fd.take() else {
      return;
    };
    if fd >= 0 {
      // SAFETY: the caller hands this descriptor to the process for this
      // single message; it is taken once and closed when the file drops.
      let mut file = unsafe { File::from_raw_fd(fd) };
      let _ = file.write_all(b"READY\n");
    }
  }
}

/// One `wl_shm` buffer in its own memfd, mapped for painting.
struct ShmBuffer {
  buffer: wl_buffer::WlBuffer,
  map: *mut u32,
  pixels: usize,
  width: u32,
  height: u32,
  busy: bool,
  /// The background has been painted; later frames only repaint the spinner.
  painted: bool,
}

impl ShmBuffer {
  fn new(
    shm: &wl_shm::WlShm,
    qh: &QueueHandle<Splash>,
    width: u32,
    height: u32,
    key: (usize, usize),
  ) -> io::Result<Self> {
    let stride = width as usize * 4;
    let size = stride * height as usize;
    // SAFETY: the name is a valid C string and the flags are valid.
    let raw = unsafe { libc::memfd_create(c"argvus-splash".as_ptr(), libc::MFD_CLOEXEC) };
    if raw < 0 {
      return Err(io::Error::last_os_error());
    }
    // SAFETY: memfd_create returned a new descriptor that nothing else owns.
    let fd = unsafe { OwnedFd::from_raw_fd(raw) };
    // SAFETY: `fd` is a valid memfd.
    if unsafe { libc::ftruncate(fd.as_raw_fd(), size as libc::off_t) } != 0 {
      return Err(io::Error::last_os_error());
    }
    // SAFETY: maps `size` bytes of the memfd, which has exactly that length.
    let map = unsafe {
      libc::mmap(
        std::ptr::null_mut(),
        size,
        libc::PROT_READ | libc::PROT_WRITE,
        libc::MAP_SHARED,
        fd.as_raw_fd(),
        0,
      )
    };
    if map == libc::MAP_FAILED {
      return Err(io::Error::last_os_error());
    }
    let pool = shm.create_pool(fd.as_fd(), size as i32, qh, ());
    let buffer = pool.create_buffer(
      0,
      width as i32,
      height as i32,
      stride as i32,
      wl_shm::Format::Xrgb8888,
      qh,
      key,
    );
    // The buffer keeps the memory alive in the compositor.
    pool.destroy();
    Ok(Self {
      buffer,
      map: map.cast(),
      pixels: size / 4,
      width,
      height,
      busy: false,
      painted: false,
    })
  }

  fn pixels(&mut self) -> &mut [u32] {
    // SAFETY: `map` is a live, u32-aligned mapping of `pixels` u32 values that
    // stays mapped until drop; the exclusive borrow prevents another slice.
    unsafe { std::slice::from_raw_parts_mut(self.map, self.pixels) }
  }
}

impl Drop for ShmBuffer {
  fn drop(&mut self) {
    self.buffer.destroy();
    // SAFETY: unmaps the mapping created in `new`, which nothing uses anymore.
    unsafe {
      libc::munmap(self.map.cast(), self.pixels * 4);
    }
  }
}

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for Splash {
  fn event(
    _: &mut Self,
    _: &wl_registry::WlRegistry,
    _: wl_registry::Event,
    _: &GlobalListContents,
    _: &Connection,
    _: &QueueHandle<Self>,
  ) {
    // Outputs that appear later are not covered; the splash is short-lived.
  }
}

impl Dispatch<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1, usize> for Splash {
  fn event(
    splash: &mut Self,
    layer: &zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
    event: zwlr_layer_surface_v1::Event,
    index: &usize,
    _: &Connection,
    qh: &QueueHandle<Self>,
  ) {
    match event {
      zwlr_layer_surface_v1::Event::Configure {
        serial,
        width,
        height,
      } => {
        layer.ack_configure(serial);
        let Some(state) = splash.surfaces.get_mut(*index) else {
          return;
        };
        if *index == 0 && state.size == (0, 0) {
          log_mark("first configure");
        }
        state.size = (width, height);
        // The splash is opaque, so the compositor can skip what lies below.
        let region = splash.compositor.create_region(qh, ());
        region.add(0, 0, width as i32, height as i32);
        state.surface.set_opaque_region(Some(&region));
        region.destroy();
        splash.draw(*index, qh);
      }
      zwlr_layer_surface_v1::Event::Closed => {
        if let Some(state) = splash.surfaces.get_mut(*index) {
          state.closed = true;
          state.buffers.clear();
          state.layer.destroy();
          state.surface.destroy();
        }
        splash.report_ready_when_presented();
      }
      _ => {}
    }
  }
}

impl Dispatch<wl_surface::WlSurface, usize> for Splash {
  fn event(
    splash: &mut Self,
    _: &wl_surface::WlSurface,
    event: wl_surface::Event,
    index: &usize,
    _: &Connection,
    qh: &QueueHandle<Self>,
  ) {
    if let wl_surface::Event::PreferredBufferScale { factor } = event
      && let Some(state) = splash.surfaces.get_mut(*index)
      && factor > 0
      && factor != state.scale
    {
      // The next frame allocates buffers at the new scale.
      state.scale = factor;
      splash.draw(*index, qh);
    }
  }
}

impl Dispatch<wl_callback::WlCallback, usize> for Splash {
  fn event(
    splash: &mut Self,
    _: &wl_callback::WlCallback,
    event: wl_callback::Event,
    index: &usize,
    _: &Connection,
    qh: &QueueHandle<Self>,
  ) {
    let wl_callback::Event::Done { .. } = event else {
      return;
    };
    let Some(state) = splash.surfaces.get_mut(*index) else {
      return;
    };
    state.frame_pending = false;
    if !state.presented {
      state.presented = true;
      if *index == 0 {
        log_mark("first layer-shell frame callback");
      }
      splash.report_ready_when_presented();
    }
    splash.draw(*index, qh);
  }
}

impl Dispatch<wl_buffer::WlBuffer, (usize, usize)> for Splash {
  fn event(
    splash: &mut Self,
    _: &wl_buffer::WlBuffer,
    event: wl_buffer::Event,
    key: &(usize, usize),
    _: &Connection,
    qh: &QueueHandle<Self>,
  ) {
    let wl_buffer::Event::Release = event else {
      return;
    };
    let (index, slot) = *key;
    let Some(state) = splash.surfaces.get_mut(index) else {
      return;
    };
    if let Some(buffer) = state.buffers.get_mut(slot) {
      buffer.busy = false;
    }
    if state.waiting_for_buffer {
      state.waiting_for_buffer = false;
      splash.draw(index, qh);
    }
  }
}

delegate_noop!(Splash: wl_compositor::WlCompositor);
delegate_noop!(Splash: wl_shm_pool::WlShmPool);
delegate_noop!(Splash: wl_region::WlRegion);
delegate_noop!(Splash: zwlr_layer_shell_v1::ZwlrLayerShellV1);
delegate_noop!(Splash: ignore wl_shm::WlShm);
delegate_noop!(Splash: ignore wl_output::WlOutput);
