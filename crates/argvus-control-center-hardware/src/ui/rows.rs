//! Rows and labels of the Hardware Overview page lists.

use super::*;

impl HardwareApp {
  pub(super) fn rows(&self) -> Vec<String> {
    match self.page {
      HardwarePage::Home => self.home_rows(),
      HardwarePage::Cpu => self
        .snapshot
        .cpu
        .governors
        .iter()
        .map(|gov| {
          let badge = if self.snapshot.cpu.governor.as_deref() == Some(gov) {
            format!("   ★ {}", tr(self.lang, "control_center.active"))
          } else {
            String::new()
          };
          format!("{gov}{badge}")
        })
        .collect(),
      HardwarePage::Gpu => self
        .snapshot
        .gpus
        .iter()
        .map(|gpu| {
          format!(
            "GPU {}  {}  {}",
            gpu.index,
            gpu
              .model
              .as_deref()
              .unwrap_or(tr(self.lang, "control_center.unavailable_6a8fc3")),
            gpu
              .driver
              .as_deref()
              .unwrap_or(tr(self.lang, "control_center.no_driver")),
          )
        })
        .collect(),
      HardwarePage::Power => self
        .snapshot
        .energy
        .profiles
        .iter()
        .map(|prof| {
          let badge = if self.snapshot.energy.profile.as_deref() == Some(prof) {
            format!("   ★ {}", tr(self.lang, "control_center.active"))
          } else {
            String::new()
          };
          format!("{prof}{badge}")
        })
        .collect(),
      HardwarePage::Devices => self
        .snapshot
        .devices
        .iter()
        .map(|dev| format!("[{}] {}", dev.kind, dev.name))
        .collect(),
      _ => vec![],
    }
  }

  pub(super) fn home_rows(&self) -> Vec<String> {
    let summary_label = tr(self.lang, "control_center.summary");
    let cpu_label = "CPU";
    let gpu_label = "GPU";
    let memory_label = tr(self.lang, "control_center.memory");
    let power_label = tr(self.lang, "control_center.power");
    let devices_label = tr(self.lang, "control_center.devices");

    let cpu_str = self
      .snapshot
      .cpu
      .model
      .clone()
      .unwrap_or_else(|| na(self.lang));

    let gpu_str = if self.snapshot.gpus.is_empty() {
      tr(self.lang, "control_center.no_gpu").into()
    } else {
      format!("{} GPU(s)", self.snapshot.gpus.len())
    };

    let mem_str = format_kib(self.snapshot.memory.total_kib);

    let power_str = self
      .snapshot
      .energy
      .profile
      .clone()
      .unwrap_or_else(|| tr(self.lang, "control_center.default").into());

    let dev_str = format!(
      "{} {}",
      self.snapshot.devices.len(),
      tr(self.lang, "control_center.devices_a41e65")
    );

    vec![
      format!(
        "{} {}  ·  {} · {}",
        AppConfig::icon(argvus_tui::icons::MONITOR),
        summary_label,
        self.snapshot.architecture,
        self.snapshot.kernel
      ),
      format!(
        "{} {}  ·  {} · {}",
        AppConfig::icon(argvus_tui::icons::MEMORY),
        cpu_label,
        cpu_str,
        self.snapshot.cpu.governor.as_deref().unwrap_or("—")
      ),
      format!(
        "{} {}  ·  {}",
        AppConfig::icon(argvus_tui::icons::GPU),
        gpu_label,
        gpu_str
      ),
      format!(
        "{} {}  ·  {}",
        AppConfig::icon(argvus_tui::icons::POWER),
        memory_label,
        mem_str
      ),
      format!(
        "{} {}  ·  {}",
        AppConfig::icon(argvus_tui::icons::BATTERY),
        power_label,
        power_str
      ),
      format!(
        "{} {}  ·  {}",
        AppConfig::icon(argvus_tui::icons::ETHERNET),
        devices_label,
        dev_str
      ),
    ]
  }
}

pub(super) fn detail_lines(app: &HardwareApp) -> Vec<Line<'static>> {
  let snapshot = &app.snapshot;
  let mut lines = Vec::new();
  let mut add = |key: &str, value: String| {
    lines.push(Line::from(format!(
      "{:<20} {value}",
      field_label(app.lang, key)
    )))
  };
  match app.page {
    HardwarePage::Summary => {
      add(
        "Manufacturer",
        snapshot
          .manufacturer
          .clone()
          .unwrap_or_else(|| na(app.lang)),
      );
      add(
        "Model",
        snapshot.model.clone().unwrap_or_else(|| na(app.lang)),
      );
      add(
        "Chassis",
        snapshot.chassis.clone().unwrap_or_else(|| na(app.lang)),
      );
      add("Architecture", snapshot.architecture.clone());
      add("Kernel", snapshot.kernel.clone());
      add(
        "CPU",
        snapshot.cpu.model.clone().unwrap_or_else(|| na(app.lang)),
      );
      add(
        "Firmware",
        snapshot.firmware.clone().unwrap_or_else(|| na(app.lang)),
      );
      add(
        "Threads",
        snapshot
          .cpu
          .threads
          .map(|v| v.to_string())
          .unwrap_or_else(|| na(app.lang)),
      );
      add(
        "Physical cores",
        snapshot
          .cpu
          .physical_cores
          .map(|v| v.to_string())
          .unwrap_or_else(|| na(app.lang)),
      );
      add(
        "Sockets",
        snapshot
          .cpu
          .sockets
          .map(|v| v.to_string())
          .unwrap_or_else(|| na(app.lang)),
      );
      add("GPU(s)", snapshot.gpus.len().to_string());
      add("RAM", format_kib(snapshot.memory.total_kib));
      add("Boot", snapshot.boot.clone());
      add(
        "Virtualization",
        snapshot
          .virtualization
          .clone()
          .unwrap_or_else(|| tr(app.lang, "control_center.none_detected").into()),
      );
      add(
        "Battery",
        if snapshot.battery.is_some() {
          tr(app.lang, "control_center.present").into()
        } else {
          tr(app.lang, "control_center.none_detected").into()
        },
      );
    }
    HardwarePage::Cpu => {
      add(
        "Vendor",
        snapshot.cpu.vendor.clone().unwrap_or_else(|| na(app.lang)),
      );
      add(
        "Model",
        snapshot.cpu.model.clone().unwrap_or_else(|| na(app.lang)),
      );
      add(
        "Threads",
        snapshot
          .cpu
          .threads
          .map(|v| v.to_string())
          .unwrap_or_else(|| na(app.lang)),
      );
      add(
        "Current",
        snapshot
          .cpu
          .current_mhz
          .map(|v| format!("{v} MHz"))
          .unwrap_or_else(|| na(app.lang)),
      );
      add(
        "Min/Max",
        format!(
          "{} / {} MHz",
          snapshot
            .cpu
            .min_mhz
            .map(|v| v.to_string())
            .unwrap_or_else(|| na(app.lang)),
          snapshot
            .cpu
            .max_mhz
            .map(|v| v.to_string())
            .unwrap_or_else(|| na(app.lang))
        ),
      );
      add(
        "Governor",
        snapshot
          .cpu
          .governor
          .clone()
          .unwrap_or_else(|| na(app.lang)),
      );
      add(
        "Available",
        if snapshot.cpu.governors.is_empty() {
          na(app.lang)
        } else {
          snapshot.cpu.governors.join(", ")
        },
      );
      add(
        "Scaling driver",
        snapshot.cpu.driver.clone().unwrap_or_else(|| na(app.lang)),
      );
      add(
        "Management",
        snapshot
          .cpu
          .management
          .clone()
          .unwrap_or_else(|| na(app.lang)),
      );
    }
    HardwarePage::Gpu | HardwarePage::GpuDetail(_) => {
      let gpus: Vec<&crate::model::GpuInfo> = match app.page {
        HardwarePage::GpuDetail(index) => snapshot.gpus.get(index).into_iter().collect(),
        _ => snapshot.gpus.iter().collect(),
      };
      for gpu in gpus {
        add(
          &format!("GPU {}", gpu.index),
          gpu
            .model
            .clone()
            .or(gpu.vendor.clone())
            .unwrap_or_else(|| na(app.lang)),
        );
        add("PCI", gpu.pci.clone().unwrap_or_else(|| na(app.lang)));
        add(
          "Vendor ID",
          gpu.vendor_id.clone().unwrap_or_else(|| na(app.lang)),
        );
        add(
          "Device ID",
          gpu.device_id.clone().unwrap_or_else(|| na(app.lang)),
        );
        add("Driver", gpu.driver.clone().unwrap_or_else(|| na(app.lang)));
        add("Module", gpu.module.clone().unwrap_or_else(|| na(app.lang)));
        add(
          "Driver status",
          gpu.driver_status.clone().unwrap_or_else(|| na(app.lang)),
        );
        add("DRM", gpu.drm.clone().unwrap_or_else(|| na(app.lang)));
        add("Render", gpu.render.clone().unwrap_or_else(|| na(app.lang)));
        add("OpenGL", gpu.opengl.clone().unwrap_or_else(|| na(app.lang)));
        add("Vulkan", gpu.vulkan.clone().unwrap_or_else(|| na(app.lang)));
      }
      if snapshot
        .gpus
        .iter()
        .any(|g| g.driver.as_deref() == Some("vmwgfx"))
        && snapshot.virtualization.is_some()
      {
        add(
          "Note",
          tr(
            app.lang,
            "control_center.vmwgfx_detected_in_virtual_machine",
          )
          .into(),
        );
      }
      add(
        "OpenGL software",
        if snapshot.software_rendering {
          tr(app.lang, "control_center.active_fb4e81").into()
        } else {
          tr(app.lang, "control_center.inactive").into()
        },
      );
    }
    HardwarePage::Memory => {
      add("Total", format_kib(snapshot.memory.total_kib));
      add("Used", format_kib(snapshot.memory.used_kib));
      add("Available", format_kib(snapshot.memory.available_kib));
      add("Swap total", format_kib(snapshot.memory.swap_total_kib));
      add("Swap free", format_kib(snapshot.memory.swap_free_kib));
    }
    HardwarePage::Power => {
      add(
        "Backend",
        snapshot
          .energy
          .backend
          .clone()
          .unwrap_or_else(|| na(app.lang)),
      );
      add(
        "Profile",
        snapshot
          .energy
          .profile
          .clone()
          .unwrap_or_else(|| na(app.lang)),
      );
      if snapshot.energy.profiles.is_empty() {
        add(
          "Profiles",
          tr(app.lang, "control_center.no_profile_available").into(),
        );
      }
      if let Some(battery) = &snapshot.battery {
        add("Battery", battery.device.clone());
        if let Some(manufacturer) = &battery.manufacturer {
          add("Manufacturer", manufacturer.clone());
        }
        if let Some(model) = &battery.model {
          add("Model", model.clone());
        }
        add(
          "Status",
          battery.status.clone().unwrap_or_else(|| na(app.lang)),
        );
        add(
          "Capacity",
          battery
            .percent
            .map(|v| format!("{v}%"))
            .unwrap_or_else(|| na(app.lang)),
        );
        add(
          "Health",
          battery
            .health
            .map(|v| format!("{v:.1}%"))
            .unwrap_or_else(|| na(app.lang)),
        );
        if let Some(ac) = battery.ac_online {
          add(
            "AC",
            if ac {
              tr(app.lang, "control_center.connected_cbc626").into()
            } else {
              tr(app.lang, "control_center.disconnected_53344a").into()
            },
          );
        }
      } else {
        add(
          "Battery",
          tr(app.lang, "control_center.no_battery_detected").into(),
        );
      }
    }
    HardwarePage::Devices => {
      for device in &snapshot.devices {
        add(&device.kind, device.name.clone());
      }
      if snapshot.devices.is_empty() {
        add(
          "Devices",
          tr(app.lang, "control_center.no_device_detected").into(),
        );
      }
    }
    HardwarePage::DeviceDetail(index) => {
      if let Some(device) = snapshot.devices.get(index) {
        add("Name", device.name.clone());
        add("Type", device.kind.clone());
        add(
          "Vendor",
          device.vendor.clone().unwrap_or_else(|| na(app.lang)),
        );
        add(
          "Driver",
          device.driver.clone().unwrap_or_else(|| na(app.lang)),
        );
        add(
          "Product",
          device.product.clone().unwrap_or_else(|| na(app.lang)),
        );
        add("Bus", device.bus.clone().unwrap_or_else(|| na(app.lang)));
        add("Path", device.path.clone().unwrap_or_else(|| na(app.lang)));
      }
    }
    HardwarePage::Home => {}
  }
  lines
}

pub(super) fn field_label(lang: Lang, key: &str) -> String {
  let label = match key {
    "Manufacturer" => "control_center.hardware_manufacturer",
    "Model" => "control_center.hardware_model",
    "Chassis" => "control_center.hardware_chassis",
    "Architecture" => "control_center.hardware_architecture",
    "Kernel" => "control_center.hardware_kernel",
    "CPU" => "control_center.hardware_cpu",
    "Threads" => "control_center.hardware_threads",
    "GPU(s)" => "control_center.hardware_gpu_s",
    "RAM" => "control_center.hardware_ram",
    "Boot" => "control_center.hardware_boot",
    "Virtualization" => "control_center.hardware_virtualization",
    "Battery" => "control_center.hardware_battery",
    "Firmware" => "control_center.hardware_firmware",
    "AC" => "control_center.hardware_ac_power",
    "Management" => "control_center.hardware_management",
    "Vendor" => "control_center.hardware_vendor",
    "Current" => "control_center.hardware_current",
    "Min/Max" => "control_center.hardware_min_max",
    "Governor" => "control_center.hardware_governor",
    "Available" => "control_center.hardware_available",
    "Scaling driver" => "control_center.hardware_scaling_driver",
    "Vendor ID" => "control_center.hardware_vendor_id",
    "Device ID" => "control_center.hardware_device_id",
    "Driver" => "control_center.hardware_driver",
    "DRM" => "control_center.hardware_drm",
    "Render" => "control_center.hardware_render",
    "OpenGL software" => "control_center.hardware_opengl_software",
    "Total" => "control_center.hardware_total",
    "Used" => "control_center.hardware_used",
    "Swap total" => "control_center.hardware_swap_total",
    "Swap free" => "control_center.hardware_swap_free",
    "Backend" => "control_center.hardware_backend",
    "Profile" => "control_center.hardware_profile",
    "Profiles" => "control_center.hardware_profiles",
    "Status" => "control_center.hardware_status",
    "Capacity" => "control_center.hardware_capacity",
    "Health" => "control_center.hardware_health",
    "Name" => "control_center.hardware_name",
    "Type" => "control_center.hardware_type",
    "Product" => "control_center.hardware_product",
    "Bus" => "control_center.hardware_bus",
    "Path" => "control_center.hardware_path",
    "Module" => "control_center.hardware_module",
    "Driver status" => "control_center.hardware_driver_status",
    "Devices" => "control_center.hardware_devices",
    _ => key,
  };
  tr(lang, label).into()
}
