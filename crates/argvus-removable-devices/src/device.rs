use std::cmp::Ordering;
use std::collections::HashMap;

use crate::config::Config;
use crate::filesystem;
use crate::text;
use crate::udisks::{Enumerated, RawBlock, RawDrive};

#[derive(Debug, Clone, Default)]
pub struct Device {
  pub object_path: String,
  pub drive_path: String,
  pub block: String,
  pub name: String,
  pub vendor: String,
  pub model: String,
  pub filesystem: String,
  pub mount_point: String,
  pub cleartext: String,
  pub capacity: u64,
  pub used: u64,
  pub free: u64,
  pub mounted: bool,
  pub read_only: bool,
  pub encrypted: bool,
  pub locked: bool,
  pub removable: bool,
  pub ejectable: bool,
  pub can_power_off: bool,
  pub is_optical: bool,
  pub hidden: bool,
  pub insertion_seq: u64,
  pub mount_time: i64,
}

fn basename(path: &str) -> &str {
  match path.rfind('/') {
    Some(slash) => &path[slash + 1..],
    None => path,
  }
}

// A partition that lives on the EFI system partition path is hidden by default.
fn is_efi_system(block: &RawBlock, mount_point: &str) -> bool {
  if !mount_point.is_empty() && mount_point.to_lowercase().contains("/boot/efi") {
    return true;
  }
  if block.id_type.eq_ignore_ascii_case("vfat") && block.partition_type.eq_ignore_ascii_case("0xef")
  {
    return true;
  }
  false
}

fn is_removable_drive(drive: Option<&RawDrive>) -> bool {
  match drive {
    Some(drive) => drive.removable || drive.media_removable || drive.ejectable || drive.optical,
    None => false,
  }
}

/// Only blocks that carry a filesystem or an encrypted container can become
/// devices; partition tables, unlocked cleartext and loop devices are skipped.
fn is_candidate_block(block: &RawBlock) -> bool {
  if block.is_partition_table || block.is_cleartext || block.is_loop || block.device.is_empty() {
    return false;
  }
  block.has_filesystem || block.has_encrypted
}

pub fn build_devices(
  raw: &Enumerated,
  cfg: &Config,
  mount_time: &mut HashMap<String, i64>,
  insertion_seq: &mut HashMap<String, u64>,
  next_insertion: &mut u64,
) -> Vec<Device> {
  let mut drives: HashMap<String, &RawDrive> = HashMap::new();
  for drive in &raw.drives {
    drives.insert(drive.object_path.clone(), drive);
  }

  let mut devices: Vec<Device> = Vec::with_capacity(raw.blocks.len());

  for block in &raw.blocks {
    if !is_candidate_block(block) {
      continue;
    }
    let drive = if block.drive.is_empty() {
      None
    } else {
      drives.get(&block.drive).copied()
    };
    if !is_removable_drive(drive) {
      continue;
    }

    let mut device = device_from_block(block, drive);
    // Hidden devices are dropped before their insertion order and mount time
    // are recorded, so showing them later starts them fresh.
    if device.hidden && !cfg.show_hidden {
      continue;
    }
    device.name = display_name(block, &device);
    device.insertion_seq = record_insertion(&device, insertion_seq, next_insertion);
    device.mount_time = record_mount_time(&device, mount_time);

    devices.push(device);
  }

  sort_devices(&mut devices, cfg.sort.as_str());
  devices
}

/// Builds a device from its UDisks block and drive: identity, lock state,
/// mount point, drive capabilities, filesystem, usage and visibility.
fn device_from_block(block: &RawBlock, drive: Option<&RawDrive>) -> Device {
  let mut device = Device {
    object_path: block.object_path.clone(),
    drive_path: block.drive.clone(),
    block: block.device.clone(),
    encrypted: block.has_encrypted,
    locked: block.has_encrypted && (block.cleartext.is_empty() || block.cleartext == "/"),
    cleartext: String::new(),
    ..Device::default()
  };
  if !device.locked {
    device.cleartext = block.cleartext.clone();
  }

  if let Some(mount_point) = block
    .mount_points
    .iter()
    .find(|mount_point| !mount_point.is_empty())
  {
    device.mount_point = mount_point.clone();
  }
  device.mounted = !device.mount_point.is_empty();

  if let Some(drive) = drive {
    device.vendor = text::trim_line_whitespace(&drive.vendor).to_string();
    device.model = text::trim_line_whitespace(&drive.model).to_string();
    device.removable = drive.removable || drive.media_removable;
    device.ejectable = drive.ejectable;
    device.is_optical = drive.optical;
  }
  device.can_power_off = !device.drive_path.is_empty() && (device.removable || device.ejectable);

  device.filesystem = block.id_type.clone();
  if device.encrypted && device.filesystem.is_empty() {
    device.filesystem = "crypto_LUKS".to_string();
  }

  device.read_only = block.read_only;
  if device.mounted {
    if let Some(usage) = filesystem::statvfs_usage(&device.mount_point) {
      device.capacity = usage.total;
      device.used = usage.used;
      device.free = usage.free;
      device.read_only = device.read_only || usage.read_only;
    }
  } else {
    device.capacity = block.size;
  }

  device.hidden = block.hint_ignore || is_efi_system(block, &device.mount_point);
  device
}

/// The filesystem label wins, then the UDisks name hint, then vendor and
/// model, and finally the block device file name.
fn display_name(block: &RawBlock, device: &Device) -> String {
  if !block.id_label.is_empty() {
    return block.id_label.clone();
  }
  if !block.hint_name.is_empty() {
    return block.hint_name.clone();
  }
  match (device.vendor.is_empty(), device.model.is_empty()) {
    (false, false) => format!("{} {}", device.vendor, device.model),
    (false, true) => device.vendor.clone(),
    (true, false) => device.model.clone(),
    (true, true) => basename(&device.block).to_string(),
  }
}

/// Returns the insertion order of the device, assigning the next number the
/// first time the object path is seen.
fn record_insertion(
  device: &Device,
  insertion_seq: &mut HashMap<String, u64>,
  next_insertion: &mut u64,
) -> u64 {
  *insertion_seq
    .entry(device.object_path.clone())
    .or_insert_with(|| {
      let assigned = *next_insertion;
      *next_insertion += 1;
      assigned
    })
}

/// Keeps the first time a device was seen mounted; an unmounted device resets
/// its entry to 0.
fn record_mount_time(device: &Device, mount_time: &mut HashMap<String, i64>) -> i64 {
  if !device.mounted {
    mount_time.insert(device.object_path.clone(), 0);
    return 0;
  }
  let now = now_epoch();
  let mounted_since = mount_time.entry(device.object_path.clone()).or_insert(0);
  if *mounted_since <= 0 {
    *mounted_since = now;
  }
  *mounted_since
}

/// Sorts by the configured mode: `name`, `size`, `insertion`, or by default
/// mounted devices first, the most recently mounted on top.
fn sort_devices(devices: &mut [Device], mode: &str) {
  devices.sort_by(|left, right| match mode {
    "name" => left.name.to_lowercase().cmp(&right.name.to_lowercase()),
    "size" => {
      if left.capacity != right.capacity {
        return right.capacity.cmp(&left.capacity);
      }
      left.insertion_seq.cmp(&right.insertion_seq)
    }
    "insertion" => left.insertion_seq.cmp(&right.insertion_seq),
    _ => {
      if left.mounted != right.mounted {
        return if left.mounted {
          Ordering::Less
        } else {
          Ordering::Greater
        };
      }
      if left.mounted && right.mounted && left.mount_time != right.mount_time {
        return right.mount_time.cmp(&left.mount_time);
      }
      left.insertion_seq.cmp(&right.insertion_seq)
    }
  });
}

fn now_epoch() -> i64 {
  std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .map(|elapsed| elapsed.as_secs() as i64)
    .unwrap_or(0)
}
