//! Isolated integration with system tools and APIs of the Hardware page.
use crate::model::*;
use argvus_control_center_core::{
  capabilities::Capabilities,
  process::{ProcessRequest, ProcessRunner, SystemProcessRunner},
  sanitize::terminal_text,
};
use std::{collections::HashSet, fs, path::Path};

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
    graphics: graphics(cap, &SystemProcessRunner),
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

fn read_dmi(name: &str) -> Option<String> {
  fs::read_to_string(format!("/sys/class/dmi/id/{name}"))
    .ok()
    .map(|v| terminal_text(v.trim()))
    .filter(|v: &String| !v.is_empty())
}
fn run(program: &str, args: &[&str]) -> String {
  run_with(&SystemProcessRunner, program, args)
}

fn run_with(runner: &impl ProcessRunner, program: &str, args: &[&str]) -> String {
  let output = runner
    .run(&ProcessRequest::new(program).args(args.iter().map(|v| (*v).to_string()).collect()))
    .ok();
  output
    .and_then(|v| String::from_utf8(v.stdout).ok())
    .map(|v| v.trim().into())
    .unwrap_or_else(|| "N/A".into())
}

trait RequestArgs {
  fn args(self, args: Vec<String>) -> Self;
}
impl RequestArgs for ProcessRequest {
  fn args(mut self, args: Vec<String>) -> Self {
    self.args = args;
    self
  }
}

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

fn gpus(cap: &Capabilities) -> Vec<GpuInfo> {
  gpus_at(cap, Path::new("/sys/class/drm"), &SystemProcessRunner)
}

fn gpus_at(cap: &Capabilities, drm: &Path, runner: &impl ProcessRunner) -> Vec<GpuInfo> {
  let mut out = Vec::new();
  if let Ok(entries) = fs::read_dir(drm) {
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
      if cap.has_lspci
        && let Some(pci) = gpu.pci.as_deref()
      {
        let value = run_with(runner, "lspci", &["-D", "-s", pci, "-nn"]);
        gpu.model = value
          .lines()
          .filter_map(parse_pci_gpu)
          .find(|(address, _)| address == pci)
          .map(|(_, model)| model);
      }
      gpu.render = find_render(&device);
      out.push(gpu);
    }
  }
  if out.is_empty() && cap.has_lspci {
    let text = run_with(runner, "lspci", &["-D", "-nnk"]);
    for (pci, model) in text.lines().filter_map(parse_pci_gpu) {
      out.push(GpuInfo {
        index: out.len(),
        pci: Some(pci),
        model: Some(model),
        ..Default::default()
      });
    }
  }
  out
}

fn parse_pci_gpu(line: &str) -> Option<(String, String)> {
  let (pci, description) = line.trim().split_once(char::is_whitespace)?;
  let parts: Vec<_> = pci.split([':', '.']).collect();
  let valid_pci = match parts.as_slice() {
    [bus, slot, function] => bus.len() == 2 && slot.len() == 2 && function.len() == 1,
    [domain, bus, slot, function] => {
      domain.len() == 4 && bus.len() == 2 && slot.len() == 2 && function.len() == 1
    }
    _ => false,
  };
  if !valid_pci
    || !parts
      .iter()
      .all(|part| part.bytes().all(|byte| byte.is_ascii_hexdigit()))
  {
    return None;
  }
  let (class, model) = description.trim().split_once(": ")?;
  if ![
    "VGA compatible controller",
    "3D controller",
    "Display controller",
  ]
  .iter()
  .any(|prefix| class.starts_with(prefix))
  {
    return None;
  }
  let model = model
    .split_whitespace()
    .filter(|word| {
      let Some(ids) = word
        .strip_prefix('[')
        .and_then(|word| word.strip_suffix(']'))
      else {
        return true;
      };
      !ids.split_once(':').is_some_and(|(vendor, device)| {
        vendor.len() == 4
          && device.len() == 4
          && vendor
            .bytes()
            .chain(device.bytes())
            .all(|byte| byte.is_ascii_hexdigit())
      })
    })
    .collect::<Vec<_>>()
    .join(" ");
  let model = terminal_text(&model);
  (!model.is_empty()).then(|| (pci.to_owned(), model))
}

fn graphics(cap: &Capabilities, runner: &impl ProcessRunner) -> GraphicsInfo {
  let probe = |program: &str, arg: &str| {
    runner
      .run(
        &ProcessRequest::new(program)
          .arg(arg)
          .env("LC_ALL", "C.UTF-8")
          .timeout(std::time::Duration::from_secs(10))
          .output_limit(1024 * 1024),
      )
      .ok()
      .filter(|output| output.status == Some(0) && !output.timed_out && !output.cancelled)
      .map(|output| terminal_text(&String::from_utf8_lossy(&output.stdout)))
  };
  GraphicsInfo {
    opengl_renderer: cap
      .has_glxinfo
      .then(|| probe("glxinfo", "-B"))
      .flatten()
      .and_then(|text| {
        parse_values(&text, "OpenGL renderer string")
          .into_iter()
          .next()
      }),
    vulkan_devices: cap
      .has_vulkaninfo
      .then(|| probe("vulkaninfo", "--summary"))
      .flatten()
      .map(|text| parse_values(&text, "deviceName"))
      .unwrap_or_default(),
  }
}

fn parse_values(input: &str, key: &str) -> Vec<String> {
  input
    .lines()
    .filter_map(|line| {
      let (name, value) = line.split_once([':', '='])?;
      let value = value.trim();
      (name.trim() == key && !value.is_empty()).then(|| value.to_owned())
    })
    .collect()
}

fn read_path(path: &Path, name: &str) -> Option<String> {
  fs::read_to_string(path.join(name))
    .ok()
    .map(|v| terminal_text(v.trim()))
}
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
fn battery() -> Option<BatteryInfo> {
  let root = Path::new("/sys/class/power_supply");
  let entry = fs::read_dir(root)
    .ok()?
    .flatten()
    .find(|candidate| candidate.file_name().to_string_lossy().starts_with("BAT"))?;
  let battery_path = entry.path();
  let full =
    read_num(&battery_path, "energy_full").or_else(|| read_num(&battery_path, "charge_full"));
  let design = read_num(&battery_path, "energy_full_design")
    .or_else(|| read_num(&battery_path, "charge_full_design"));
  Some(BatteryInfo {
    device: entry.file_name().to_string_lossy().into(),
    manufacturer: read_path(&battery_path, "manufacturer"),
    model: read_path(&battery_path, "model_name"),
    percent: read_num(&battery_path, "capacity").and_then(|candidate| u8::try_from(candidate).ok()),
    status: read_path(&battery_path, "status"),
    full_capacity: full,
    design_capacity: design,
    cycles: read_num(&battery_path, "cycle_count"),
    energy_now: read_num(&battery_path, "energy_now")
      .or_else(|| read_num(&battery_path, "charge_now")),
    power_now: read_num(&battery_path, "power_now")
      .or_else(|| read_num(&battery_path, "current_now")),
    health: full
      .zip(design)
      .filter(|(_, d)| *d > 0)
      .map(|(f, d)| f as f32 / d as f32 * 100.0),
    ac_online: ac_online(),
  })
}
fn read_num(p: &Path, n: &str) -> Option<u64> {
  fs::read_to_string(p.join(n)).ok()?.trim().parse().ok()
}
fn ac_online() -> Option<bool> {
  let root = Path::new("/sys/class/power_supply");
  fs::read_dir(root)
    .ok()?
    .flatten()
    .filter(|v| v.file_name().to_string_lossy().starts_with("AC"))
    .find_map(|v| read_num(&v.path(), "online").map(|n| n == 1))
}
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
  fn parses_meminfo() {
    let m = memory("MemTotal: 100 kB\nMemAvailable: 40 kB\nSwapTotal: 20 kB\nSwapFree: 5 kB");
    assert_eq!(m.used_kib, Some(60));
    assert_eq!(m.swap_free_kib, Some(5));
  }
  #[test]
  fn parses_cpu() {
    let c = cpu("vendor_id: GenuineIntel\nmodel name: Test\nprocessor: 0\nprocessor: 1\n");
    assert_eq!(c.threads, Some(2));
    assert_eq!(c.model.as_deref(), Some("Test"));
  }
  #[test]
  fn parses_power_profiles_keeping_only_actual_profiles() {
    let output = "* performance:\n    CpuDriver:\tintel_pstate\n  balanced:\n    CpuDriver:\tintel_pstate\n    PlatformDriver:\tplaceholder\n  power-saver:\n    CpuDriver:\tintel_pstate\n    PlatformDriver:\tplaceholder\n";
    assert_eq!(
      parse_profiles(output),
      vec!["performance", "balanced", "power-saver"]
    );
  }
  #[test]
  fn parses_power_profiles_with_degraded_and_space_drivers() {
    let output = "* balanced:\n    CpuDriver:\tintel_pstate\n    Degraded:\tno\n  performance:\n    CpuDriver:  intel_pstate\n";
    assert_eq!(parse_profiles(output), vec!["balanced", "performance"]);
  }
  struct GraphicsFixture;
  impl ProcessRunner for GraphicsFixture {
    fn run(
      &self,
      request: &ProcessRequest,
    ) -> Result<
      argvus_control_center_core::process::ProcessOutput,
      argvus_control_center_core::process::ProcessError,
    > {
      let stdout = match request.program.as_str() {
        "lspci" => {
          "0000:00:02.0 VGA compatible controller [0300]: Intel Corporation Integrated Graphics [8086:1234]\n0000:01:00.0 3D controller [0302]: NVIDIA Corporation GM107M [GeForce 940MX] [10de:179c] (rev a2)\n"
        }
        "glxinfo" => "OpenGL renderer string: Intel Integrated Graphics\n",
        "vulkaninfo" => {
          "deviceName = Intel Integrated Graphics\n deviceName = llvmpipe (LLVM 20.1.8, 256 bits)\n"
        }
        other => panic!("unexpected probe {other}"),
      };
      Ok(argvus_control_center_core::process::ProcessOutput {
        stdout: stdout.as_bytes().to_vec(),
        stderr: vec![],
        status: Some(0),
        timed_out: false,
        cancelled: false,
      })
    }
  }

  fn graphics_capabilities() -> Capabilities {
    Capabilities {
      has_lspci: true,
      has_glxinfo: true,
      has_vulkaninfo: true,
      ..Capabilities::default()
    }
  }

  #[test]
  fn p8_pci_models_do_not_include_address_or_class() {
    let devices = gpus_at(
      &graphics_capabilities(),
      Path::new("/dev/null"),
      &GraphicsFixture,
    );
    assert_eq!(devices.len(), 2);
    assert_eq!(
      devices[0].model.as_deref(),
      Some("Intel Corporation Integrated Graphics")
    );
    assert_eq!(
      devices[1].model.as_deref(),
      Some("NVIDIA Corporation GM107M [GeForce 940MX] (rev a2)")
    );
  }

  #[test]
  fn p8_session_renderers_are_not_copied_to_each_gpu() {
    let devices = gpus_at(
      &graphics_capabilities(),
      Path::new("/dev/null"),
      &GraphicsFixture,
    );
    assert_eq!(devices.len(), 2);
    assert!(
      devices
        .iter()
        .all(|gpu| gpu.opengl.is_none() && gpu.vulkan.is_none())
    );
  }
  #[test]
  fn p8_parses_short_pci_addresses_and_preserves_model_brackets() {
    let (pci, model) = parse_pci_gpu("01:00.0 VGA compatible controller [0300]: NVIDIA Corporation GM107M [GeForce 940MX] [10de:179c] (rev a2)").unwrap();
    assert_eq!(pci, "01:00.0");
    assert_eq!(model, "NVIDIA Corporation GM107M [GeForce 940MX] (rev a2)");
    for line in [
      "Kernel driver in use: nvidia",
      "0000:01:00.0 Audio device [0403]: NVIDIA Audio [10de:1234]",
      "garbage VGA compatible controller: GPU",
      "0000:01:00.0 3D controller [0302]: ",
    ] {
      assert!(parse_pci_gpu(line).is_none(), "{line}");
    }
  }

  #[test]
  fn p8_global_graphics_keeps_all_vulkan_names_including_software() {
    let info = graphics(&graphics_capabilities(), &GraphicsFixture);
    assert_eq!(
      info.opengl_renderer.as_deref(),
      Some("Intel Integrated Graphics")
    );
    assert_eq!(
      info.vulkan_devices,
      [
        "Intel Integrated Graphics",
        "llvmpipe (LLVM 20.1.8, 256 bits)"
      ]
    );
  }

  #[test]
  fn p8_failed_graphics_probes_do_not_report_stdout_as_success() {
    struct Failed;
    impl ProcessRunner for Failed {
      fn run(
        &self,
        request: &ProcessRequest,
      ) -> Result<
        argvus_control_center_core::process::ProcessOutput,
        argvus_control_center_core::process::ProcessError,
      > {
        assert!(request.timeout.is_some());
        assert!(request.output_limit.is_some());
        Ok(argvus_control_center_core::process::ProcessOutput {
          stdout: b"OpenGL renderer string: wrong\ndeviceName = wrong\n".to_vec(),
          stderr: vec![],
          status: Some(1),
          timed_out: false,
          cancelled: false,
        })
      }
    }
    let info = graphics(&graphics_capabilities(), &Failed);
    assert!(info.opengl_renderer.is_none());
    assert!(info.vulkan_devices.is_empty());
  }
}
