//! Describes the disk and file system that hold the root (`/`) mount.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use std::fs;
use std::path::Path;

use argvus_control_center_core::sanitize::terminal_text;

/// Block devices as the kernel exposes them, one directory per disk, partition or mapper.
const SYS_BLOCK: &str = "/sys/class/block";
/// Bound on the device-mapper chain walk (LUKS on LVM on RAID stays far below it).
const MAX_STACK_DEPTH: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Kind of the physical disk under `/`.
pub enum DiskKind {
  Nvme,
  Ssd,
  Hdd,
  /// eMMC or SD card (`mmcblk*`).
  Mmc,
  /// Virtual disks and anything whose kind the kernel does not report reliably.
  Unknown,
}

impl DiskKind {
  /// Short technical label shown next to the model, `None` when unknown.
  pub fn label(self) -> Option<&'static str> {
    match self {
      Self::Nvme => Some("NVMe"),
      Self::Ssd => Some("SSD"),
      Self::Hdd => Some("HDD"),
      Self::Mmc => Some("eMMC/SD"),
      Self::Unknown => None,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Root file system and the physical disk under it.
pub struct Storage {
  /// Disk model, or the block device name when the disk reports none (`vda`).
  pub model: Option<String>,
  pub kind: DiskKind,
  pub filesystem: String,
  pub total_bytes: Option<u64>,
  pub used_bytes: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// The `/` line of `/proc/self/mountinfo`.
pub struct RootMount {
  /// Mount source, such as `/dev/nvme0n1p2`, `/dev/mapper/root` or a ZFS dataset.
  pub source: String,
  pub filesystem: String,
}

/// Reads the root mount, its disk and its usage; `None` when `/` cannot be found.
pub fn root_storage() -> Option<Storage> {
  let mount = root_mount(&fs::read_to_string("/proc/self/mountinfo").ok()?)?;
  let disk = block_name(&mount.source).and_then(|name| physical_disk(&name, 0));
  let (total_bytes, used_bytes) = root_usage();
  Some(Storage {
    model: disk.as_deref().map(disk_model),
    kind: disk.as_deref().map_or(DiskKind::Unknown, |name| {
      disk_kind(name, read_attribute(name, "queue/rotational").as_deref())
    }),
    filesystem: terminal_text(&mount.filesystem),
    total_bytes,
    used_bytes,
  })
}

/// Finds the mount of `/`. When `/` is mounted more than once, the last line is the
/// visible one.
pub fn root_mount(mountinfo: &str) -> Option<RootMount> {
  mountinfo.lines().rev().find_map(|line| {
    // Fields: id parent major:minor root mount-point options [optional...] - fstype source super
    let (before, after) = line.split_once(" - ")?;
    let mount_point = before.split_whitespace().nth(4)?;
    if mount_point != "/" {
      return None;
    }
    let mut fields = after.split_whitespace();
    let filesystem = fields.next()?.to_string();
    let source = unescape_mountinfo(fields.next()?);
    Some(RootMount { source, filesystem })
  })
}

/// Classifies a disk by its name and `queue/rotational`.
///
/// Virtual disks (`vd*`, `xvd*`) report `rotational = 1` whatever backs them, so they
/// stay unknown instead of being shown as hard disks.
pub fn disk_kind(name: &str, rotational: Option<&str>) -> DiskKind {
  if name.starts_with("nvme") {
    DiskKind::Nvme
  } else if name.starts_with("mmcblk") {
    DiskKind::Mmc
  } else if name.starts_with("vd") || name.starts_with("xvd") {
    DiskKind::Unknown
  } else {
    match rotational.map(str::trim) {
      Some("0") => DiskKind::Ssd,
      Some("1") => DiskKind::Hdd,
      _ => DiskKind::Unknown,
    }
  }
}

/// Resolves a mount source to its block device name (`/dev/mapper/root` → `dm-0`).
/// Sources that are not device paths (ZFS datasets, `overlay`) have none.
fn block_name(source: &str) -> Option<String> {
  if !source.starts_with("/dev/") {
    return None;
  }
  let device = fs::canonicalize(source).ok()?;
  let name = device.file_name()?.to_string_lossy().into_owned();
  Path::new(SYS_BLOCK).join(&name).exists().then_some(name)
}

/// Walks from a partition or a device-mapper/md device down to the physical disk.
fn physical_disk(name: &str, depth: usize) -> Option<String> {
  let device = Path::new(SYS_BLOCK).join(name);
  if depth < MAX_STACK_DEPTH
    && let Ok(entries) = fs::read_dir(device.join("slaves"))
  {
    // A mapper over several disks (RAID, multi-disk LVM) is shown by its first member.
    let mut slaves = entries
      .flatten()
      .map(|entry| entry.file_name().to_string_lossy().into_owned())
      .collect::<Vec<_>>();
    slaves.sort();
    if let Some(first) = slaves.first() {
      return physical_disk(first, depth + 1);
    }
  }
  if device.join("partition").exists() {
    // A partition's sysfs directory sits inside the directory of its disk.
    let canonical = fs::canonicalize(&device).ok()?;
    return Some(
      canonical
        .parent()?
        .file_name()?
        .to_string_lossy()
        .into_owned(),
    );
  }
  Some(name.to_string())
}

/// Returns the disk model (`device/model`, or `device/name` on eMMC/SD), else the device name.
fn disk_model(name: &str) -> String {
  read_attribute(name, "device/model")
    .or_else(|| read_attribute(name, "device/name"))
    .unwrap_or_else(|| name.to_string())
}

fn read_attribute(name: &str, attribute: &str) -> Option<String> {
  fs::read_to_string(Path::new(SYS_BLOCK).join(name).join(attribute))
    .ok()
    .map(|value| terminal_text(value.trim()))
    .filter(|value| !value.is_empty())
}

/// Size and used space of the root file system, from `statvfs("/")`.
fn root_usage() -> (Option<u64>, Option<u64>) {
  let Ok(stats) = rustix::fs::statvfs("/") else {
    return (None, None);
  };
  let block = stats.f_frsize;
  let total = stats.f_blocks.saturating_mul(block);
  let used = stats
    .f_blocks
    .saturating_sub(stats.f_bfree)
    .saturating_mul(block);
  (Some(total), Some(used))
}

/// Decodes the octal escapes mountinfo uses for spaces, tabs and backslashes (`\040`).
fn unescape_mountinfo(value: &str) -> String {
  let bytes = value.as_bytes();
  let mut decoded = Vec::with_capacity(bytes.len());
  let mut index = 0;
  while index < bytes.len() {
    let escape = bytes.get(index + 1..index + 4).filter(|digits| {
      bytes[index] == b'\\' && digits.iter().all(|digit| (b'0'..=b'7').contains(digit))
    });
    match escape {
      Some(digits) => {
        let octal = std::str::from_utf8(digits).unwrap_or_default();
        decoded.push(u8::from_str_radix(octal, 8).unwrap_or(b'?'));
        index += 4;
      }
      None => {
        decoded.push(bytes[index]);
        index += 1;
      }
    }
  }
  String::from_utf8_lossy(&decoded).into_owned()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn finds_btrfs_root_by_mount_point_not_device_number() {
    // Btrfs reports an anonymous device number (0:32); the source names the real device.
    let mountinfo = "\
22 1 0:21 / /proc rw,nosuid shared:5 - proc proc rw\n\
25 1 0:32 /@ / rw,relatime shared:1 - btrfs /dev/mapper/cryptroot rw,ssd,subvol=/@\n\
26 25 0:32 /@home /home rw,relatime shared:2 - btrfs /dev/mapper/cryptroot rw\n";
    assert_eq!(
      root_mount(mountinfo),
      Some(RootMount {
        source: "/dev/mapper/cryptroot".into(),
        filesystem: "btrfs".into(),
      })
    );
  }

  #[test]
  fn uses_the_last_root_mount_and_optional_fields() {
    let mountinfo = "\
1 0 8:1 / / rw - ext4 /dev/sda1 rw\n\
2 1 259:2 / / rw,noatime shared:1 master:2 - xfs /dev/nvme0n1p2 rw\n";
    let mount = root_mount(mountinfo).expect("root");
    assert_eq!(mount.source, "/dev/nvme0n1p2");
    assert_eq!(mount.filesystem, "xfs");
    assert_eq!(root_mount("1 0 0:1 / /home rw - ext4 /dev/sda2 rw\n"), None);
  }

  #[test]
  fn classifies_disks_without_trusting_virtual_rotational_flag() {
    assert_eq!(disk_kind("nvme0n1", Some("0")), DiskKind::Nvme);
    assert_eq!(disk_kind("sda", Some("0\n")), DiskKind::Ssd);
    assert_eq!(disk_kind("sdb", Some("1")), DiskKind::Hdd);
    assert_eq!(disk_kind("mmcblk0", Some("0")), DiskKind::Mmc);
    assert_eq!(disk_kind("vda", Some("1")), DiskKind::Unknown);
    assert_eq!(disk_kind("sdc", None), DiskKind::Unknown);
  }

  #[test]
  fn decodes_mountinfo_octal_escapes() {
    assert_eq!(unescape_mountinfo("/dev/disk\\040one"), "/dev/disk one");
    assert_eq!(unescape_mountinfo("/dev/sda1"), "/dev/sda1");
    assert_eq!(unescape_mountinfo("trailing\\"), "trailing\\");
  }
}
