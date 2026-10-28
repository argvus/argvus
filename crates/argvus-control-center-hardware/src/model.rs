//! Domain state and models consumed by the UI of the Hardware page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HardwarePage {
  Home,
  Summary,
  Cpu,
  Gpu,
  GpuDetail(usize),
  Memory,
  Power,
  Devices,
  DeviceDetail(usize),
}

#[derive(Debug, Clone, Default)]
pub struct HardwareSnapshot {
  pub manufacturer: Option<String>,
  pub model: Option<String>,
  pub chassis: Option<String>,
  pub architecture: String,
  pub kernel: String,
  pub cpu: CpuInfo,
  pub memory: MemoryInfo,
  pub gpus: Vec<GpuInfo>,
  pub battery: Option<BatteryInfo>,
  pub energy: EnergyInfo,
  pub devices: Vec<DeviceInfo>,
  pub boot: String,
  pub firmware: Option<String>,
  pub virtualization: Option<String>,
  pub software_rendering: bool,
  pub is_laptop: bool,
}

#[derive(Debug, Clone, Default)]
pub struct CpuInfo {
  pub vendor: Option<String>,
  pub model: Option<String>,
  pub sockets: Option<u32>,
  pub physical_cores: Option<u32>,
  pub threads: Option<u32>,
  pub current_mhz: Option<u64>,
  pub min_mhz: Option<u64>,
  pub max_mhz: Option<u64>,
  pub governor: Option<String>,
  pub governors: Vec<String>,
  pub driver: Option<String>,
  pub management: Option<String>,
}
#[derive(Debug, Clone, Default)]
pub struct MemoryInfo {
  pub total_kib: Option<u64>,
  pub available_kib: Option<u64>,
  pub used_kib: Option<u64>,
  pub swap_total_kib: Option<u64>,
  pub swap_free_kib: Option<u64>,
}
#[derive(Debug, Clone, Default)]
pub struct GpuInfo {
  pub index: usize,
  pub model: Option<String>,
  pub pci: Option<String>,
  pub vendor_id: Option<String>,
  pub device_id: Option<String>,
  pub vendor: Option<String>,
  pub driver: Option<String>,
  pub module: Option<String>,
  pub drm: Option<String>,
  pub render: Option<String>,
  pub opengl: Option<String>,
  pub vulkan: Option<String>,
  pub driver_status: Option<String>,
}
#[derive(Debug, Clone, Default)]
pub struct BatteryInfo {
  pub device: String,
  pub manufacturer: Option<String>,
  pub model: Option<String>,
  pub percent: Option<u8>,
  pub status: Option<String>,
  pub full_capacity: Option<u64>,
  pub design_capacity: Option<u64>,
  pub cycles: Option<u64>,
  pub energy_now: Option<u64>,
  pub power_now: Option<u64>,
  pub health: Option<f32>,
  pub ac_online: Option<bool>,
}
#[derive(Debug, Clone, Default)]
pub struct EnergyInfo {
  pub backend: Option<String>,
  pub profile: Option<String>,
  pub profiles: Vec<String>,
  pub tlp_active: Option<bool>,
}
#[derive(Debug, Clone, Default)]
pub struct DeviceInfo {
  pub name: String,
  pub kind: String,
  pub vendor: Option<String>,
  pub product: Option<String>,
  pub driver: Option<String>,
  pub bus: Option<String>,
  pub path: Option<String>,
  pub status: Option<String>,
}

pub fn format_kib(value: Option<u64>) -> String {
  value
    .map(|v| format!("{:.1} GiB", v as f64 / 1_048_576.0))
    .unwrap_or_else(|| "N/A".into())
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn formats_missing_memory_safely() {
    assert_eq!(format_kib(None), "N/A");
  }
}
