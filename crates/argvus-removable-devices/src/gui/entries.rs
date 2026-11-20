//! Menu rows built from the devices: titles, state words, action labels and tooltips.

use crate::{actions::ActionKind, device::Device, i18n, text};

pub(crate) struct DeviceEntry {
  pub(crate) kind: ActionKind,
  pub(crate) device: Device,
}

// Both GTK menus use the same device capabilities.
pub(crate) fn device_entries(d: &Device) -> Vec<DeviceEntry> {
  let can_mount = !(d.mounted || (d.encrypted && d.locked));
  let mut out = Vec::new();
  if !d.encrypted || !d.locked {
    out.push(DeviceEntry {
      kind: ActionKind::Open,
      device: d.clone(),
    });
  }
  if can_mount {
    out.push(DeviceEntry {
      kind: ActionKind::Mount,
      device: d.clone(),
    });
  }
  if d.mounted {
    out.push(DeviceEntry {
      kind: ActionKind::Unmount,
      device: d.clone(),
    });
  }
  if d.encrypted && d.locked {
    out.push(DeviceEntry {
      kind: ActionKind::Unlock,
      device: d.clone(),
    });
  }
  if d.encrypted && !d.locked {
    out.push(DeviceEntry {
      kind: ActionKind::Lock,
      device: d.clone(),
    });
  }
  if d.ejectable {
    out.push(DeviceEntry {
      kind: ActionKind::Eject,
      device: d.clone(),
    });
  }
  if d.can_power_off {
    out.push(DeviceEntry {
      kind: ActionKind::PowerOff,
      device: d.clone(),
    });
  }
  if d.mounted {
    out.push(DeviceEntry {
      kind: ActionKind::Copy,
      device: d.clone(),
    });
  }
  out
}

pub(crate) fn action_label(kind: ActionKind) -> &'static str {
  let key = match kind {
    ActionKind::Open => "device.open",
    ActionKind::Mount => "device.mount",
    ActionKind::Unmount => "device.unmount",
    ActionKind::Eject => "device.eject",
    ActionKind::PowerOff => "device.power_off",
    ActionKind::Lock => "device.lock",
    ActionKind::Unlock => "device.unlock",
    ActionKind::Copy => "device.copy_path",
  };
  i18n::tr(key)
}

pub(crate) fn action_tooltip(kind: ActionKind) -> &'static str {
  let key = match kind {
    ActionKind::Open => "device.open_in_the_configured_file_manager",
    ActionKind::Mount => "device.mount_this_device",
    ActionKind::Unmount => "device.unmount_this_device",
    ActionKind::Eject => "device.unmount_and_eject_this_device",
    ActionKind::PowerOff => "device.safely_power_off_this_drive",
    ActionKind::Lock => "device.lock_this_encrypted_volume",
    ActionKind::Unlock => "device.unlock_this_encrypted_volume",
    ActionKind::Copy => "device.copy_the_mount_path",
  };
  i18n::tr(key)
}

pub(crate) fn device_title(d: &Device) -> String {
  let key = state_words(d.encrypted && d.locked, d.mounted);
  let state = i18n::tr(key);
  if d.capacity > 0 {
    format!("{} · {} · {}", d.name, text::human_bytes(d.capacity), state)
  } else {
    format!("{} · {}", d.name, state)
  }
}

pub(crate) fn state_words(locked: bool, mounted: bool) -> &'static str {
  if locked {
    "device.locked"
  } else if mounted {
    "device.mounted"
  } else {
    "device.available"
  }
}
