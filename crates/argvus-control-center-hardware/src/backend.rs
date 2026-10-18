//! Implements isolated integration with system tools and APIs in crate `argvus control center hardware`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use crate::model::*;
use argvus_control_center_core::{
  capabilities::Capabilities,
  process::{ProcessRequest, ProcessRunner, SystemProcessRunner},
  sanitize::terminal_text,
};
use std::{collections::HashSet, fs, path::Path};

/// Executes the `collect` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn collect(cap: &Capabilities) -> HardwareSnapshot {
  let cpuinfo = fs::read_to_string("/proc/cpuinfo").unwrap_or_default();
  let meminfo = fs::read_to_string("/proc/meminfo").unwrap_or_default();
  let mut cpu = cpu(&cpuinfo);
  cpu.management = if cap.has_power_profiles_daemon {
    Some("power-profiles-daemon".into())
  } else if cap.has_tlp {
    Some("TLP".into())
  } else if cap.has_cpupower {
    Some("cpupower".into())
  } else {
    Some("sysfs cpufreq".into())
  };
  HardwareSnapshot {
    manufacturer: read_dmi("sys_vendor"),
    model: read_dmi("product_name"),
    chassis: read_dmi("chassis_type"),
    architecture: std::env::consts::ARCH.into(),
    kernel: run("uname", &["-r"]),
    cpu,
    memory: memory(&meminfo),
    gpus: gpus(cap),
    battery: battery(),
    energy: energy(cap),
    devices: devices(),
    boot: if Path::new("/sys/firmware/efi").exists() {
      "UEFI".into()
    } else {
      "BIOS/Legacy".into()
    },
    firmware: read_dmi("bios_version"),
    virtualization: virtualization(cap),
    software_rendering: std::env::var_os("LIBGL_ALWAYS_SOFTWARE").is_some_and(|v| v == "1"),
    is_laptop: detect_is_laptop(),
  }
}

/// Retrieves data for `detect_is_laptop` without mixing collection with TUI rendering. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn detect_is_laptop() -> bool {
  if let Ok(chassis) = fs::read_to_string("/sys/class/dmi/id/chassis_type")
    && let Ok(n) = chassis.trim().parse::<u8>()
    && matches!(n, 8..=14 | 30..=32)
  {
    return true;
  }
  fs::read_dir("/sys/class/power_supply")
    .ok()
    .and_then(|dir| {
      dir
        .flatten()
        .find(|e| e.file_name().to_string_lossy().starts_with("BAT"))
    })
    .is_some()
}

/// Retrieves data for `read_dmi` without mixing collection with TUI rendering. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn read_dmi(name: &str) -> Option<String> {
  fs::read_to_string(format!("/sys/class/dmi/id/{name}"))
    .ok()
    .map(|v| terminal_text(v.trim()))
    .filter(|v: &String| !v.is_empty())
}
/// Executes the `run` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn run(program: &str, args: &[&str]) -> String {
  let output = SystemProcessRunner
    .run(&ProcessRequest::new(program).args(args.iter().map(|v| (*v).to_string()).collect()))
    .ok();
  output
    .and_then(|v| String::from_utf8(v.stdout).ok())
    .map(|v| v.trim().into())
    .unwrap_or_else(|| "N/A".into())
}

/// Defines the `RequestArgs`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
trait RequestArgs {
  /// Executes the `args` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn args(self, args: Vec<String>) -> Self;
}
impl RequestArgs for ProcessRequest {
  /// Executes the `args` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn args(mut self, args: Vec<String>) -> Self {
    self.args = args;
    self
  }
}

/// Executes the `cpu` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn cpu(input: &str) -> CpuInfo {
  let mut out = CpuInfo::default();
  let mut processors = 0;
  for line in input.lines() {
    let Some((key, value)) = line.split_once(':') else {
      continue;
    };
    let value = value.trim();
    match key.trim() {
      "vendor_id" => {
        out.vendor.get_or_insert(value.into());
      }
      "model name" => {
        out.model.get_or_insert(value.into());
      }
      "physical id" => {}
      "processor" => processors += 1,
      "cpu MHz" => out.current_mhz = value.parse::<f64>().ok().map(|v| v as u64),
      _ => {}
    };
  }
  out.threads = (processors > 0).then_some(processors);
  let topology = (0..processors)
    .filter_map(|index| {
      let root = Path::new("/sys/devices/system/cpu").join(format!("cpu{index}/topology"));
      let package = read_path(&root, "physical_package_id")?;
      let core = read_path(&root, "core_id")?;
      Some((package, core))
    })
    .collect::<HashSet<_>>();
  out.physical_cores = (!topology.is_empty())
    .then_some(topology.len() as u32)
    .or(out.threads);
  out.sockets = if topology.is_empty() {
    None
  } else {
    Some(
      topology
        .iter()
        .map(|(package, _)| package)
        .collect::<HashSet<_>>()
        .len() as u32,
    )
  };
  let root = Path::new("/sys/devices/system/cpu/cpu0/cpufreq");
  out.governor = read_path(root, "scaling_governor");
  out.governors = read_path(root, "scaling_available_governors")
    .map(|v| v.split_whitespace().map(str::to_owned).collect())
    .unwrap_or_default();
  out.driver = read_path(root, "scaling_driver");
  out.min_mhz = read_num(root, "cpuinfo_min_freq").map(|v| v / 1000);
  out.max_mhz = read_num(root, "cpuinfo_max_freq").map(|v| v / 1000);
  out.current_mhz = read_num(root, "scaling_cur_freq")
    .map(|v| v / 1000)
    .or(out.current_mhz);
  out
}

/// Executes the `memory` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn memory(input: &str) -> MemoryInfo {
  let mut m = MemoryInfo::default();
  for line in input.lines() {
    let Some((k, v)) = line.split_once(':') else {
      continue;
    };
    let n = v.split_whitespace().next().and_then(|v| v.parse().ok());
    match k {
      "MemTotal" => m.total_kib = n,
      "MemAvailable" => m.available_kib = n,
      "SwapTotal" => m.swap_total_kib = n,
      "SwapFree" => m.swap_free_kib = n,
      _ => {}
    }
  }
  m.used_kib = m
    .total_kib
    .zip(m.available_kib)
    .map(|(t, a)| t.saturating_sub(a));
  m
}

/// Executes the `gpus` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn gpus(cap: &Capabilities) -> Vec<GpuInfo> {
  let mut out = Vec::new();
  if let Ok(entries) = fs::read_dir("/sys/class/drm") {
    for entry in entries.flatten() {
      let name = entry.file_name().to_string_lossy().into_owned();
      if !name.starts_with("card") || name.contains('-') {
        continue;
      };
      let device = entry.path().join("device");
      let driver = fs::read_link(device.join("driver"))
        .ok()
        .and_then(|p| p.file_name().map(|v| v.to_string_lossy().into_owned()));
      let pci = fs::canonicalize(&device)
        .ok()
        .and_then(|p| p.file_name().map(|v| v.to_string_lossy().into_owned()));
      let mut gpu = GpuInfo {
        index: out.len(),
        pci,
        vendor_id: read_path(&device, "vendor"),
        device_id: read_path(&device, "device"),
        vendor: read_path(&device, "vendor").map(|v| {
          if v == "0x10de" {
            "NVIDIA".into()
          } else if v == "0x1002" {
            "AMD".into()
          } else if v == "0x8086" {
            "Intel".into()
          } else {
            v
          }
        }),
        driver: driver.clone(),
        module: driver,
        drm: Some(format!("/dev/dri/{name}")),
        render: None,
        driver_status: Some("loaded".into()),
        ..Default::default()
      };
      if cap.has_lspci {
        let value = run(
          "lspci",
          &["-s", gpu.pci.as_deref().unwrap_or_default(), "-nn"],
        );
        gpu.model = value
          .split_once(':')
          .map(|(_, value)| value.split(" [").next().unwrap_or(value).trim().into());
      }
      gpu.render = find_render(&device);
      out.push(gpu);
    }
  }
  if out.is_empty() && cap.has_lspci {
    let text = run("lspci", &["-D", "-nnk"]);
    for line in text.lines().filter(|v| {
      v.contains("VGA") || v.contains("3D controller") || v.contains("Display controller")
    }) {
      out.push(GpuInfo {
        index: out.len(),
        model: line.split_once(':').map(|(_, v)| v.trim().into()),
        ..Default::default()
      });
    }
  }
  let opengl = if cap.has_glxinfo {
    parse_value(&run("glxinfo", &["-B"]), "OpenGL renderer string")
  } else {
    None
  };
  let vulkan = if cap.has_vulkaninfo {
    parse_value(&run("vulkaninfo", &["--summary"]), "deviceName")
  } else {
    None
  };
  for gpu in &mut out {
    gpu.opengl = opengl.clone();
    gpu.vulkan = vulkan.clone();
  }
  out
}

/// Converts input data into `parse_value` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn parse_value(input: &str, key: &str) -> Option<String> {
  input.lines().find_map(|line| {
    line
      .split_once(':')
      .filter(|(name, _)| name.trim() == key)
      .map(|(_, value)| value.trim().into())
  })
}
/// Retrieves data for `read_path` without mixing collection with TUI rendering. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn read_path(path: &Path, name: &str) -> Option<String> {
  fs::read_to_string(path.join(name))
    .ok()
    .map(|v| terminal_text(v.trim()))
}
/// Executes the `find_render` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn find_render(card: &Path) -> Option<String> {
  fs::read_dir("/dev/dri")
    .ok()?
    .flatten()
    .map(|v| v.path())
    .find(|p| {
      p.file_name()
        .is_some_and(|n| n.to_string_lossy().starts_with("renderD"))
        && fs::canonicalize(format!(
          "/sys/class/drm/{}",
          p.file_name().unwrap().to_string_lossy()
        ))
        .ok()
        .is_some_and(|path| path.starts_with(card))
    })
    .map(|p| p.display().to_string())
}
/// Executes the `battery` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn battery() -> Option<BatteryInfo> {
  let root = Path::new("/sys/class/power_supply");
  let entry = fs::read_dir(root)
    .ok()?
    .flatten()
    .find(|v| v.file_name().to_string_lossy().starts_with("BAT"))?;
  let p = entry.path();
  let full = read_num(&p, "energy_full").or_else(|| read_num(&p, "charge_full"));
  let design = read_num(&p, "energy_full_design").or_else(|| read_num(&p, "charge_full_design"));
  Some(BatteryInfo {
    device: entry.file_name().to_string_lossy().into(),
    manufacturer: read_path(&p, "manufacturer"),
    model: read_path(&p, "model_name"),
    percent: read_num(&p, "capacity").and_then(|v| u8::try_from(v).ok()),
    status: read_path(&p, "status"),
    full_capacity: full,
    design_capacity: design,
    cycles: read_num(&p, "cycle_count"),
    energy_now: read_num(&p, "energy_now").or_else(|| read_num(&p, "charge_now")),
    power_now: read_num(&p, "power_now").or_else(|| read_num(&p, "current_now")),
    health: full
      .zip(design)
      .filter(|(_, d)| *d > 0)
      .map(|(f, d)| f as f32 / d as f32 * 100.0),
    ac_online: ac_online(),
  })
}
/// Retrieves data for `read_num` without mixing collection with TUI rendering. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn read_num(p: &Path, n: &str) -> Option<u64> {
  fs::read_to_string(p.join(n)).ok()?.trim().parse().ok()
}
/// Executes the `ac_online` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn ac_online() -> Option<bool> {
  let root = Path::new("/sys/class/power_supply");
  fs::read_dir(root)
    .ok()?
    .flatten()
    .filter(|v| v.file_name().to_string_lossy().starts_with("AC"))
    .find_map(|v| read_num(&v.path(), "online").map(|n| n == 1))
}
/// Executes the `energy` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn energy(cap: &Capabilities) -> EnergyInfo {
  let profiles = if cap.has_power_profiles_daemon {
    parse_profiles(&run("powerprofilesctl", &["list"]))
  } else {
    Vec::new()
  };
  let profile = cap
    .has_power_profiles_daemon
    .then(|| run("powerprofilesctl", &["get"]));
  EnergyInfo {
    backend: if cap.has_power_profiles_daemon {
      Some("power-profiles-daemon".into())
    } else if cap.has_tlp {
      Some("TLP".into())
    } else if cap.has_cpupower {
      Some("cpupower".into())
    } else {
      Some("sysfs cpufreq".into())
    },
    profile,
    profiles,
    tlp_active: cap
      .has_tlp
      .then(|| run("tlp-stat", &["-s"]).contains("enabled")),
  }
}

/// Parses `powerprofilesctl list` output, keeping only the profile headings
/// (e.g. `performance`, `balanced`, `power-saver`). Driver sublines such as
/// `CpuDriver:intel_pstate` and `PlatformDriver:placeholder` are dropped.
fn parse_profiles(output: &str) -> Vec<String> {
  output
    .lines()
    .filter_map(|line| {
      let name = line
        .trim()
        .trim_start_matches('*')
        .trim()
        .trim_end_matches(':')
        .trim();
      (!name.is_empty() && !name.chars().any(char::is_whitespace) && !name.contains(':'))
        .then(|| name.into())
    })
    .collect()
}
/// Executes the `devices` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn devices() -> Vec<DeviceInfo> {
  let mut devices = Vec::new();
  for (root, kind, limit) in [
    ("/sys/bus/pci/devices", "PCI", 100),
    ("/sys/bus/usb/devices", "USB", 100),
  ] {
    for entry in fs::read_dir(root)
      .ok()
      .into_iter()
      .flatten()
      .flatten()
      .take(limit)
    {
      let path = entry.path();
      let name = terminal_text(&entry.file_name().to_string_lossy());
      let driver = fs::read_link(path.join("driver"))
        .ok()
        .and_then(|p| p.file_name().map(|v| terminal_text(&v.to_string_lossy())));
      devices.push(DeviceInfo {
        name,
        kind: kind.into(),
        vendor: read_path(&path, "vendor"),
        product: read_path(&path, "device").or_else(|| read_path(&path, "product")),
        driver,
        bus: Some(kind.into()),
        path: Some(path.display().to_string()),
        status: None,
      });
    }
  }
  if let Ok(entries) = fs::read_dir("/sys/class/drm") {
    devices.extend(entries.flatten().filter_map(|entry| {
      let name = entry.file_name().to_string_lossy().into_owned();
      name.starts_with("card").then(|| DeviceInfo {
        name,
        kind: "DRM".into(),
        vendor: None,
        product: None,
        driver: None,
        bus: Some("DRM".into()),
        path: Some(entry.path().display().to_string()),
        status: None,
      })
    }));
  }
  devices
}
/// Executes the `virtualization` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn virtualization(cap: &Capabilities) -> Option<String> {
  if !cap.is_virtual_machine {
    return None;
  }
  Some(run("systemd-detect-virt", &[]))
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  /// Converts input data into `parses_meminfo` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn parses_meminfo() {
    let m = memory("MemTotal: 100 kB\nMemAvailable: 40 kB\nSwapTotal: 20 kB\nSwapFree: 5 kB");
    assert_eq!(m.used_kib, Some(60));
    assert_eq!(m.swap_free_kib, Some(5));
  }
  #[test]
  /// Converts input data into `parses_cpu` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn parses_cpu() {
    let c = cpu("vendor_id: GenuineIntel\nmodel name: Test\nprocessor: 0\nprocessor: 1\n");
    assert_eq!(c.threads, Some(2));
    assert_eq!(c.model.as_deref(), Some("Test"));
  }
  #[test]
  /// Converts input data into `parses_power_profiles_keeping_only_actual_profiles` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn parses_power_profiles_keeping_only_actual_profiles() {
    let output = "* performance:\n    CpuDriver:\tintel_pstate\n  balanced:\n    CpuDriver:\tintel_pstate\n    PlatformDriver:\tplaceholder\n  power-saver:\n    CpuDriver:\tintel_pstate\n    PlatformDriver:\tplaceholder\n";
    assert_eq!(
      parse_profiles(output),
      vec!["performance", "balanced", "power-saver"]
    );
  }
  #[test]
  /// Converts input data into `parses_power_profiles_with_degraded_and_space_drivers` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn parses_power_profiles_with_degraded_and_space_drivers() {
    let output = "* balanced:\n    CpuDriver:\tintel_pstate\n    Degraded:\tno\n  performance:\n    CpuDriver:  intel_pstate\n";
    assert_eq!(parse_profiles(output), vec!["balanced", "performance"]);
  }
}
