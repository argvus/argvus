//! DRM card, connector and CRTC discovery, and the watch that detects when another client replaces the surface.

use super::drawing::{Palette, draw_background, draw_spinner};
use super::drm::{Connector, DumbBuffer, ModeInfo};
use super::drm::{
  drmModeFreeConnector, drmModeFreeCrtc, drmModeFreeEncoder, drmModeFreeResources,
  drmModeGetConnector, drmModeGetConnectorCurrent, drmModeGetCrtc, drmModeGetEncoder,
  drmModeGetResources, drmSetMaster,
};
use super::timing::{mark, monotonic_millis};
use super::*;

/// What the surface keeps after a `release` until the user compositor shows
/// its own first frame.
pub(super) struct Released {
  pub(super) stream: UnixStream,
  pub(super) buffer: DumbBuffer,
  pub(super) crtc_id: u32,
}

/// Waits until the CRTC no longer scans out `buffer`, which means the user
/// compositor committed its own frame, or until `REPLACEMENT_TIMEOUT`, and keeps
/// the spinner animating meanwhile. Writing to the buffer needs no DRM master:
/// it stays scanned out until the compositor's first modeset, and the squares
/// disappear only for the moment the compositor needs to serve its clients.
/// Returns whether the compositor replaced the frame before the timeout.
pub(super) fn wait_for_replacement(
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
pub(super) fn crtc_mode(fd: RawFd, crtc_id: u32) -> Option<ModeInfo> {
  let crtc = unsafe { drmModeGetCrtc(fd, crtc_id) };
  if crtc.is_null() {
    return None;
  }
  let mode = unsafe { ((*crtc).mode_valid != 0).then_some((*crtc).mode) };
  unsafe { drmModeFreeCrtc(crtc) };
  mode
}

/// The framebuffer the CRTC scans out and whether it has a mode set.
pub(super) fn crtc_state(fd: RawFd, crtc_id: u32) -> Option<(u32, bool)> {
  let crtc = unsafe { drmModeGetCrtc(fd, crtc_id) };
  if crtc.is_null() {
    return None;
  }
  let state = unsafe { ((*crtc).buffer_id, (*crtc).mode_valid != 0) };
  unsafe { drmModeFreeCrtc(crtc) };
  Some(state)
}

/// One CRTC as the handoff sees it while it waits for DRM master.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct CrtcSample {
  pub(super) crtc_id: u32,
  pub(super) framebuffer: u32,
  pub(super) mode_valid: bool,
}

/// Remembers the last state of every CRTC, so the wait logs only the changes.
#[derive(Debug, Default)]
pub(super) struct CrtcWatch {
  pub(super) last: Vec<CrtcSample>,
}

impl CrtcWatch {
  /// The samples of `current` that differ from the previous call.
  pub(super) fn changes(&mut self, current: Vec<CrtcSample>) -> Vec<CrtcSample> {
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
pub(super) fn crtc_samples(fd: RawFd) -> Vec<CrtcSample> {
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
pub(super) struct Observer {
  pub(super) name: String,
  pub(super) file: fs::File,
  pub(super) watch: CrtcWatch,
}

/// The DRM cards, in name order, and one observer on each.
///
/// When the greeter compositor's descriptor is the last one to close, the
/// kernel restores the fbdev console mode, which blanks the output before this
/// process can draw. The observers keep the device open, and they read the
/// CRTCs, so the timing log shows whether the output stayed on between the two
/// masters.
pub(super) fn open_observers() -> io::Result<(Vec<PathBuf>, Vec<Observer>)> {
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
pub(super) fn acquire_card(cards: &[PathBuf], observers: &mut Vec<Observer>) -> io::Result<RawFd> {
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
pub(super) struct PreparedSurface {
  pub(super) connector_id: u32,
  pub(super) crtc_id: u32,
  pub(super) mode: ModeInfo,
  pub(super) buffer: DumbBuffer,
}

/// Finds the output and paints a buffer for it with the background and the
/// spinner. None of this needs DRM master.
pub(super) fn prepare_surface(fd: RawFd, palette: &Palette) -> io::Result<PreparedSurface> {
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

/// The first connected output, its CRTC and its preferred mode. The cached
/// connector state comes first: the compositor that just left has probed the
/// connectors, and probing again takes tens of milliseconds while the output
/// waits for a frame.
pub(super) fn select_output(fd: RawFd) -> io::Result<(u32, u32, ModeInfo)> {
  find_output(fd, drmModeGetConnectorCurrent).or_else(|_| find_output(fd, drmModeGetConnector))
}

pub(super) fn find_output(
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
