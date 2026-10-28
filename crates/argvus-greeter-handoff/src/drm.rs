//! Raw libdrm bindings: the KMS structs, the C functions and the dumb buffers the surface is drawn into.

use super::*;

/// `DRM_FORMAT_XRGB8888`, the format of every dumb buffer this surface draws.
pub(super) const DRM_FORMAT_XRGB8888: u32 = 0x3432_5258;

pub(super) const DRM_IOCTL_MODE_CREATE_DUMB: libc::c_ulong = 0xc020_64b2;

pub(super) const DRM_IOCTL_MODE_MAP_DUMB: libc::c_ulong = 0xc010_64b3;

pub(super) const DRM_IOCTL_MODE_DESTROY_DUMB: libc::c_ulong = 0xc010_64b4;

#[repr(C)]
pub(super) struct ModeResources {
  pub(super) count_fbs: libc::c_int,
  pub(super) fbs: *mut u32,
  pub(super) count_crtcs: libc::c_int,
  pub(super) crtcs: *mut u32,
  pub(super) count_connectors: libc::c_int,
  pub(super) connectors: *mut u32,
  pub(super) count_encoders: libc::c_int,
  pub(super) encoders: *mut u32,
  pub(super) min_width: u32,
  pub(super) max_width: u32,
  pub(super) min_height: u32,
  pub(super) max_height: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(super) struct ModeInfo {
  pub(super) clock: u32,
  pub(super) hdisplay: u16,
  pub(super) hsync_start: u16,
  pub(super) hsync_end: u16,
  pub(super) htotal: u16,
  pub(super) hskew: u16,
  pub(super) vdisplay: u16,
  pub(super) vsync_start: u16,
  pub(super) vsync_end: u16,
  pub(super) vtotal: u16,
  pub(super) vscan: u16,
  pub(super) vrefresh: u32,
  pub(super) flags: u32,
  pub(super) mode_type: u32,
  pub(super) name: [libc::c_char; 32],
}

impl ModeInfo {
  /// Whether both modes drive the output with the same timings, so moving
  /// from one to the other needs no modeset.
  pub(super) fn same_timings(&self, other: &Self) -> bool {
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
pub(super) struct Connector {
  pub(super) connector_id: u32,
  pub(super) encoder_id: u32,
  pub(super) connector_type: u32,
  pub(super) connector_type_id: u32,
  pub(super) connection: libc::c_int,
  pub(super) mm_width: u32,
  pub(super) mm_height: u32,
  pub(super) subpixel: libc::c_int,
  pub(super) count_modes: libc::c_int,
  pub(super) modes: *mut ModeInfo,
  pub(super) count_props: libc::c_int,
  pub(super) props: *mut u32,
  pub(super) prop_values: *mut u64,
  pub(super) count_encoders: libc::c_int,
  pub(super) encoders: *mut u32,
}

#[repr(C)]
pub(super) struct Encoder {
  pub(super) encoder_id: u32,
  pub(super) encoder_type: u32,
  pub(super) crtc_id: u32,
  pub(super) possible_crtcs: u32,
  pub(super) possible_clones: u32,
}

#[repr(C)]
pub(super) struct Crtc {
  pub(super) crtc_id: u32,
  pub(super) buffer_id: u32,
  pub(super) x: u32,
  pub(super) y: u32,
  pub(super) width: u32,
  pub(super) height: u32,
  pub(super) mode_valid: libc::c_int,
  pub(super) mode: ModeInfo,
  pub(super) gamma_size: libc::c_int,
}

#[repr(C)]
pub(super) struct PlaneResources {
  pub(super) count_planes: u32,
  pub(super) planes: *mut u32,
}

#[repr(C)]
pub(super) struct Plane {
  pub(super) count_formats: u32,
  pub(super) formats: *mut u32,
  pub(super) plane_id: u32,
  pub(super) crtc_id: u32,
  pub(super) fb_id: u32,
  pub(super) crtc_x: u32,
  pub(super) crtc_y: u32,
  pub(super) x: u32,
  pub(super) y: u32,
  pub(super) possible_crtcs: u32,
  pub(super) gamma_size: u32,
}

#[repr(C)]
pub(super) struct CreateDumb {
  pub(super) height: u32,
  pub(super) width: u32,
  pub(super) bpp: u32,
  pub(super) flags: u32,
  pub(super) handle: u32,
  pub(super) pitch: u32,
  pub(super) size: u64,
}

#[repr(C)]
pub(super) struct MapDumb {
  pub(super) handle: u32,
  pub(super) pad: u32,
  pub(super) offset: u64,
}

#[repr(C)]
pub(super) struct DestroyDumb {
  pub(super) handle: u32,
}

#[link(name = "drm")]
unsafe extern "C" {
  pub(super) fn drmSetMaster(fd: libc::c_int) -> libc::c_int;
  pub(super) fn drmDropMaster(fd: libc::c_int) -> libc::c_int;
  pub(super) fn drmIoctl(
    fd: libc::c_int,
    request: libc::c_ulong,
    arg: *mut libc::c_void,
  ) -> libc::c_int;
  pub(super) fn drmModeGetResources(fd: libc::c_int) -> *mut ModeResources;
  pub(super) fn drmModeFreeResources(resources: *mut ModeResources);
  pub(super) fn drmModeGetConnector(fd: libc::c_int, connector_id: u32) -> *mut Connector;
  pub(super) fn drmModeGetConnectorCurrent(fd: libc::c_int, connector_id: u32) -> *mut Connector;
  pub(super) fn drmModeFreeConnector(connector: *mut Connector);
  pub(super) fn drmModeGetEncoder(fd: libc::c_int, encoder_id: u32) -> *mut Encoder;
  pub(super) fn drmModeFreeEncoder(encoder: *mut Encoder);
  pub(super) fn drmModeAddFB(
    fd: libc::c_int,
    width: u32,
    height: u32,
    depth: u8,
    bpp: u8,
    pitch: u32,
    handle: u32,
    buffer_id: *mut u32,
  ) -> libc::c_int;
  pub(super) fn drmModeRmFB(fd: libc::c_int, buffer_id: u32) -> libc::c_int;
  pub(super) fn drmModeGetCrtc(fd: libc::c_int, crtc_id: u32) -> *mut Crtc;
  pub(super) fn drmModeFreeCrtc(crtc: *mut Crtc);
  pub(super) fn drmModeSetCrtc(
    fd: libc::c_int,
    crtc_id: u32,
    buffer_id: u32,
    x: u32,
    y: u32,
    connectors: *mut u32,
    connector_count: libc::c_int,
    mode: *mut ModeInfo,
  ) -> libc::c_int;
  pub(super) fn drmModeSetCursor(
    fd: libc::c_int,
    crtc_id: u32,
    handle: u32,
    width: u32,
    height: u32,
  ) -> libc::c_int;
  pub(super) fn drmModeGetPlaneResources(fd: libc::c_int) -> *mut PlaneResources;
  pub(super) fn drmModeFreePlaneResources(resources: *mut PlaneResources);
  pub(super) fn drmModeGetPlane(fd: libc::c_int, plane_id: u32) -> *mut Plane;
  pub(super) fn drmModeFreePlane(plane: *mut Plane);
  pub(super) fn drmModeSetPlane(
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
  pub(super) fn drmModePageFlip(
    fd: libc::c_int,
    crtc_id: u32,
    buffer_id: u32,
    flags: u32,
    user_data: *mut libc::c_void,
  ) -> libc::c_int;
}

pub(super) struct DumbBuffer {
  pub(super) fd: RawFd,
  pub(super) handle: u32,
  pub(super) width: u32,
  pub(super) height: u32,
  pub(super) pitch: u32,
  pub(super) size: usize,
  pub(super) map: *mut u8,
  pub(super) framebuffer: u32,
}

impl DumbBuffer {
  pub(super) fn create(fd: RawFd, width: u32, height: u32) -> io::Result<Self> {
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
  pub(super) fn pixels(&mut self) -> &mut [u32] {
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

pub(super) fn drm_call<T>(fd: RawFd, request: libc::c_ulong, value: &mut T) -> io::Result<()> {
  let result = unsafe { drmIoctl(fd, request, value as *mut T as *mut libc::c_void) };
  if result == 0 {
    Ok(())
  } else {
    Err(io::Error::last_os_error())
  }
}

pub(super) fn destroy_dumb(fd: RawFd, handle: u32) {
  let mut request = DestroyDumb { handle };
  let _ = drm_call(fd, DRM_IOCTL_MODE_DESTROY_DUMB, &mut request);
}
