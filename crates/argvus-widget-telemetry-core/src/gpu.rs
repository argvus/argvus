//! The GPU of the CPU/GPU block and of the taskbar: which driver the machine
//! uses, and the parsing of what the driver reports. The reading itself comes
//! from sysfs and nvidia-smi, in the binary.

/// The PCI vendor id of NVIDIA cards, as sysfs writes it.
pub const VENDOR_NVIDIA: &str = "0x10de";
/// The PCI vendor id of AMD cards, as sysfs writes it.
pub const VENDOR_AMD: &str = "0x1002";

/// The kernel driver that serves the GPU.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Driver {
  NvidiaProprietary,
  NvidiaNouveau,
  Amd,
  AmdRadeon,
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
  use super::{Driver, Reading, amd_name, detect, nouveau_name, parse_nvidia_csv};

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
    assert_eq!(detect(false, &[], &["0x8086"]), Driver::Unknown);
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
}
