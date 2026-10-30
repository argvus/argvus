//! The GPU of the CPU/GPU block and of the taskbar: which driver the machine
//! uses, and the parsing of what the driver reports. The reading itself comes
//! from sysfs and nvidia-smi, in the binary.

use std::path::PathBuf;

/// The PCI vendor id of NVIDIA cards, as sysfs writes it.
pub const VENDOR_NVIDIA: &str = "0x10de";
/// The PCI vendor id of AMD cards, as sysfs writes it.
pub const VENDOR_AMD: &str = "0x1002";
/// The PCI vendor id of Intel graphics devices.
pub const VENDOR_INTEL: &str = "0x8086";

/// A supported PCI graphics vendor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Vendor {
  Nvidia,
  Amd,
  Intel,
}

/// The PCI vendor of a graphics device supported by the taskbar monitor.
pub fn recognized_vendor(vendor: Option<&str>) -> Option<Vendor> {
  match vendor? {
    VENDOR_NVIDIA => Some(Vendor::Nvidia),
    VENDOR_AMD => Some(Vendor::Amd),
    VENDOR_INTEL => Some(Vendor::Intel),
    _ => None,
  }
}

/// A sysfs-associated hwmon device and its temperature values in Celsius.
pub type HwmonTemperatures = (PathBuf, Vec<Option<i64>>);

/// Returns the highest valid temperature belonging to exactly one GPU path.
/// An unrelated or ambiguously linked hwmon device is never attributed.
pub fn device_temperature(
  device: &PathBuf,
  gpu_devices: &[PathBuf],
  hwmons: &[HwmonTemperatures],
) -> Option<i64> {
  if gpu_devices.iter().filter(|path| *path == device).count() != 1 {
    return None;
  }
  hwmons
    .iter()
    .filter(|(path, _)| path == device)
    .flat_map(|(_, readings)| readings.iter().copied().flatten())
    .filter(|temperature| valid_temperature(*temperature))
    .max()
}

/// Whether a Celsius temperature is in a plausible hardware sensor range.
pub fn valid_temperature(temperature: i64) -> bool {
  (-40..=200).contains(&temperature)
}

/// The highest valid temperature among the supplied GPU readings.
pub fn maximum_temperature(temperatures: impl IntoIterator<Item = Option<i64>>) -> Option<i64> {
  temperatures
    .into_iter()
    .flatten()
    .filter(|temperature| valid_temperature(*temperature))
    .max()
}

/// Converts sysfs and NVML spellings of a PCI address into a comparable form.
pub fn normalize_pci_bus_id(bus_id: &str) -> Option<String> {
  let normalized = bus_id.trim().to_ascii_lowercase();
  let normalized = normalized
    .strip_prefix("00000000:")
    .map(|bus_id| format!("0000:{bus_id}"))
    .unwrap_or(normalized);
  let parts = normalized.split([':', '.']).collect::<Vec<_>>();
  (parts.len() == 4
    && parts
      .iter()
      .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_hexdigit())))
  .then_some(normalized)
}

/// Parses the `pci.bus_id,temperature.gpu` rows returned by nvidia-smi.
pub fn parse_nvidia_temperatures(output: &str) -> Vec<(String, Option<i64>)> {
  output
    .lines()
    .filter_map(|line| {
      let (bus_id, temperature) = line.split_once(',')?;
      let bus_id = normalize_pci_bus_id(bus_id)?;
      let temperature = temperature.trim().parse::<i64>().ok();
      Some((
        bus_id,
        temperature.filter(|value| valid_temperature(*value)),
      ))
    })
    .collect()
}

/// The kernel driver that serves the GPU.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Driver {
  NvidiaProprietary,
  NvidiaNouveau,
  Amd,
  AmdRadeon,
  Intel,
  Unknown,
}

/// The figures of one GPU. A figure that is not known is `None`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reading {
  pub usage: Option<u32>,
  pub temperature: Option<i64>,
  pub name: String,
}

/// The driver of the machine: nvidia-smi answers first, then the loaded kernel
/// modules, then the vendor of the display cards in their order.
pub fn detect(nvidia_smi_answers: bool, loaded_modules: &[&str], card_vendors: &[&str]) -> Driver {
  if nvidia_smi_answers {
    return Driver::NvidiaProprietary;
  }
  for (module, driver) in [
    ("nouveau", Driver::NvidiaNouveau),
    ("amdgpu", Driver::Amd),
    ("radeon", Driver::AmdRadeon),
  ] {
    if loaded_modules.contains(&module) {
      return driver;
    }
  }
  for vendor in card_vendors {
    match *vendor {
      VENDOR_NVIDIA => return Driver::NvidiaNouveau,
      VENDOR_AMD => return Driver::Amd,
      _ => {}
    }
  }
  if card_vendors.contains(&VENDOR_INTEL) {
    return Driver::Intel;
  }
  Driver::Unknown
}

/// The first line of `nvidia-smi --query-gpu=utilization.gpu,temperature.gpu,name
/// --format=csv,noheader,nounits`. A value nvidia-smi does not report, such as
/// `[N/A]`, is unknown.
pub fn parse_nvidia_csv(line: &str) -> Option<Reading> {
  let mut fields = line.split(',');
  let usage = fields.next()?.trim().parse().ok();
  let temperature = fields.next()?.trim().parse().ok();
  let name = fields.next()?.trim().to_string();
  Some(Reading {
    usage,
    temperature,
    name,
  })
}

/// The name of a nouveau card, from its product name when sysfs has one.
pub fn nouveau_name(product: Option<&str>) -> String {
  match product.map(str::trim).filter(|product| !product.is_empty()) {
    Some(product) => format!("NVIDIA {product} (nouveau)"),
    None => "NVIDIA (nouveau)".to_string(),
  }
}

/// The name of an AMD card, from its product name or its label.
pub fn amd_name(model: Option<&str>, default: &str) -> String {
  match model.map(str::trim).filter(|model| !model.is_empty()) {
    Some(model) => format!("AMD {model}"),
    None => default.to_string(),
  }
}

#[cfg(test)]
mod tests {
  use super::{
    Driver, HwmonTemperatures, Reading, Vendor, amd_name, detect, device_temperature,
    maximum_temperature, normalize_pci_bus_id, nouveau_name, parse_nvidia_csv,
    parse_nvidia_temperatures, recognized_vendor,
  };
  use std::path::PathBuf;

  #[test]
  fn detection_follows_the_documented_order() {
    assert_eq!(detect(true, &["amdgpu"], &[]), Driver::NvidiaProprietary);
    assert_eq!(
      detect(false, &["radeon", "amdgpu"], &["0x10de"]),
      Driver::Amd
    );
    assert_eq!(
      detect(false, &[], &["0x8086", "0x10de"]),
      Driver::NvidiaNouveau
    );
    assert_eq!(detect(false, &[], &["0x8086"]), Driver::Intel);
  }

  #[test]
  fn nvidia_csv_reads_three_fields() {
    assert_eq!(
      parse_nvidia_csv("37, 52, NVIDIA GeForce RTX 3060"),
      Some(Reading {
        usage: Some(37),
        temperature: Some(52),
        name: "NVIDIA GeForce RTX 3060".into(),
      })
    );
    let not_reported = parse_nvidia_csv("[N/A], 52, RTX").unwrap();
    assert_eq!(not_reported.usage, None);
  }

  #[test]
  fn names_fall_back_when_sysfs_has_none() {
    assert_eq!(nouveau_name(Some("GA106")), "NVIDIA GA106 (nouveau)");
    assert_eq!(nouveau_name(None), "NVIDIA (nouveau)");
    assert_eq!(amd_name(Some("Navi 33"), "AMD GPU"), "AMD Navi 33");
    assert_eq!(amd_name(Some(" "), "AMD GPU"), "AMD GPU");
  }

  #[test]
  fn recognizes_supported_gpu_vendors() {
    assert_eq!(recognized_vendor(Some("0x10de")), Some(Vendor::Nvidia));
    assert_eq!(recognized_vendor(Some("0x1002")), Some(Vendor::Amd));
    assert_eq!(recognized_vendor(Some("0x8086")), Some(Vendor::Intel));
    assert_eq!(recognized_vendor(Some("0x1234")), None);
    assert_eq!(recognized_vendor(None), None);
  }

  #[test]
  fn parses_nvidia_rows_for_multiple_devices_and_ignores_invalid_values() {
    assert_eq!(
      parse_nvidia_temperatures(
        "00000000:01:00.0, 47\n00000000:02:00.0, 71\nnot-a-bus, 50\n00000000:03:00.0, N/A\n"
      ),
      vec![
        ("0000:01:00.0".into(), Some(47)),
        ("0000:02:00.0".into(), Some(71)),
        ("0000:03:00.0".into(), None),
      ]
    );
    assert_eq!(
      normalize_pci_bus_id("0000:01:00.0"),
      Some("0000:01:00.0".into())
    );
    assert_eq!(normalize_pci_bus_id("broken"), None);
    assert!(parse_nvidia_temperatures("").is_empty());
  }

  #[test]
  fn aggregates_only_valid_gpu_temperatures() {
    assert_eq!(
      maximum_temperature([Some(58), Some(74), None, Some(201), Some(-41)]),
      Some(74)
    );
    assert_eq!(maximum_temperature([None, Some(201)]), None);
  }

  #[test]
  fn hwmon_temperature_requires_a_unique_exact_device_link() {
    let gpu = PathBuf::from("/sys/devices/pci0000:00/0000:01:00.0");
    let other = PathBuf::from("/sys/devices/pci0000:00/0000:02:00.0");
    let hwmons: Vec<HwmonTemperatures> = vec![
      (gpu.clone(), vec![Some(53), Some(69), Some(250)]),
      (PathBuf::from("/sys/devices/platform/acpi"), vec![Some(99)]),
    ];
    assert_eq!(
      device_temperature(&gpu, &[gpu.clone(), other.clone()], &hwmons),
      Some(69)
    );
    assert_eq!(
      device_temperature(&other, &[gpu.clone(), other.clone()], &hwmons),
      None
    );
    assert_eq!(
      device_temperature(&gpu, &[gpu.clone(), gpu.clone()], &hwmons),
      None
    );
    assert_eq!(device_temperature(&gpu, &[], &hwmons), None);
    assert_eq!(
      device_temperature(&gpu, std::slice::from_ref(&gpu), &[]),
      None
    );
  }
}
