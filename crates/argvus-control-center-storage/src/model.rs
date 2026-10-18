//! Implements domain state and models consumed by the UI in crate `argvus control center storage`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoragePage {
  Home,
  Summary,
  Disks,
  DiskDetails(usize),
  Partitions,
  PartitionDetails(usize),
  Filesystems,
  FilesystemDetails(usize),
  Mounts,
  MountDetails(usize),
  Smart,
  SmartDetails(usize),
  Usage,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// Represents `StorageSnapshot`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct StorageSnapshot {
  pub devices: Vec<StorageDevice>,
  pub filesystems: Vec<Filesystem>,
  pub mounts: Vec<Mount>,
  pub swap: Vec<SwapDevice>,
  pub smart: Vec<SmartStatus>,
  pub smart_available: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// Represents `StorageDevice`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct StorageDevice {
  pub name: String,
  pub kind: String,
  pub model: Option<String>,
  pub vendor: Option<String>,
  pub serial: Option<String>,
  pub size_bytes: Option<u64>,
  pub transport: Option<String>,
  pub rotational: Option<bool>,
  pub removable: Option<bool>,
  pub driver: Option<String>,
  pub parent: Option<String>,
  pub uuid: Option<String>,
  pub fstype: Option<String>,
  pub label: Option<String>,
  pub mountpoint: Option<String>,
  pub readonly: Option<bool>,
  pub children: Vec<StorageDevice>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// Represents `Filesystem`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct Filesystem {
  pub source: String,
  pub fstype: String,
  pub uuid: Option<String>,
  pub label: Option<String>,
  pub mountpoint: Option<String>,
  pub total_bytes: Option<u64>,
  pub used_bytes: Option<u64>,
  pub available_bytes: Option<u64>,
  pub readonly: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// Represents `Mount`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct Mount {
  pub target: String,
  pub source: String,
  pub fstype: String,
  pub options: Vec<String>,
  pub readonly: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// Represents `SwapDevice`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct SwapDevice {
  pub source: String,
  pub kind: String,
  pub total_bytes: u64,
  pub used_bytes: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// Represents `SmartStatus`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct SmartStatus {
  pub device: String,
  pub health: SmartHealth,
  pub temperature_c: Option<i64>,
  pub power_on_hours: Option<u64>,
  pub power_cycles: Option<u64>,
  pub percentage_used: Option<u8>,
  pub critical_warning: Option<u64>,
  pub details: Vec<(String, String)>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
/// Defines `SmartHealth`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub enum SmartHealth {
  Passed,
  Warning,
  Failed,
  Unsupported,
  #[default]
  Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Defines `UsageLevel`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub enum UsageLevel {
  Ok,
  Warning,
  Critical,
}

/// Executes the `usage_level` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn usage_level(total: u64, available: u64) -> UsageLevel {
  if total == 0 {
    return UsageLevel::Ok;
  }
  let used = total.saturating_sub(available);
  let percent = used.saturating_mul(100) / total;
  if percent >= 90 {
    UsageLevel::Critical
  } else if percent >= 80 {
    UsageLevel::Warning
  } else {
    UsageLevel::Ok
  }
}
