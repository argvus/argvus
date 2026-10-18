//! The GPU reading: the driver of the machine and its usage, temperature and
//! name. nvidia-smi is asked for the proprietary driver; the other drivers are
//! read from sysfs.

use crate::hardware::{self, Card};
use argvus_runtime::messages;
use argvus_runtime::process::{output_any, program_exists, read_text, run_quiet};
use argvus_widget_telemetry_core::gpu::{self, Driver, Reading, VENDOR_AMD, VENDOR_NVIDIA};
use argvus_widget_telemetry_core::sensors::chip_celsius;

/// The GPU of the machine, when its driver is known.
pub fn reading() -> Option<(Driver, Reading)> {
  let cards = hardware::cards();
  let vendors: Vec<&str> = cards
    .iter()
    .filter_map(|card| card.vendor.as_deref())
    .collect();
  let nvidia_smi_answers = program_exists("nvidia-smi") && run_quiet("nvidia-smi", &["-L"]);
  let modules = hardware::loaded_modules();
  let loaded: Vec<&str> = modules.iter().map(String::as_str).collect();
  let driver = gpu::detect(nvidia_smi_answers, &loaded, &vendors);
  let reading = match driver {
    Driver::NvidiaProprietary => nvidia_proprietary(),
    Driver::NvidiaNouveau => nvidia_nouveau(&cards),
    Driver::Amd | Driver::AmdRadeon => amd(&cards),
    Driver::Unknown => return None,
  };
  Some((driver, reading))
}

/// The NVIDIA proprietary driver, read through its first GPU.
fn nvidia_proprietary() -> Reading {
  let csv = output_any(
    "nvidia-smi",
    &[
      "--query-gpu=utilization.gpu,temperature.gpu,name",
      "--format=csv,noheader,nounits",
    ],
  );
  let first_line = csv.lines().next().unwrap_or_default();
  gpu::parse_nvidia_csv(first_line).unwrap_or(Reading {
    usage: None,
    temperature: None,
    name: "NVIDIA".to_string(),
  })
}

/// The nouveau driver: its temperature comes from the hardware monitor, and the
/// driver does not report the usage.
fn nvidia_nouveau(cards: &[Card]) -> Reading {
  let temperature = chip_celsius(&hardware::hwmons(), &["nouveau"], &["temp1", "temp2"]);
  let product = cards
    .iter()
    .find(|card| card.vendor.as_deref() == Some(VENDOR_NVIDIA))
    .and_then(|card| read_text(&card.device.join("product_name")));
  Reading {
    usage: None,
    temperature,
    name: gpu::nouveau_name(product.as_deref()),
  }
}

/// The AMD drivers: the usage is the busy percent of the card, the temperature
/// comes from its hardware monitor.
fn amd(cards: &[Card]) -> Reading {
  let amd_cards = cards
    .iter()
    .filter(|card| card.vendor.as_deref() == Some(VENDOR_AMD));
  let usage = amd_cards.clone().find_map(|card| {
    read_text(&card.device.join("gpu_busy_percent"))?
      .trim()
      .parse()
      .ok()
  });
  // The model comes from the first AMD card, as the shell read it.
  let model = amd_cards.clone().next().and_then(|card| {
    ["product_name", "label"]
      .iter()
      .find_map(|file| read_text(&card.device.join(file)))
  });
  let default_name = messages::text("gpu.name.amd", &[]);
  Reading {
    usage,
    temperature: chip_celsius(
      &hardware::hwmons(),
      &["amdgpu", "radeon"],
      &["temp2", "temp1"],
    ),
    name: gpu::amd_name(model.as_deref(), &default_name),
  }
}
