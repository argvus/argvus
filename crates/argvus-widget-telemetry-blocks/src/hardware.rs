//! Reads the kernel interfaces the blocks use: the hardware monitor sensors, the
//! display cards, the loaded modules and the CPU counters. Only the files are
//! read here; the rules that use them are in the core crate.

use argvus_runtime::process::read_text;
use argvus_widget_telemetry_core::cpu::{self, Sample};
use argvus_widget_telemetry_core::gpu::{self, HwmonTemperatures, recognized_vendor};
use argvus_widget_telemetry_core::sensors::{Hwmon, Temperature};
use std::fs;
use std::path::{Path, PathBuf};

/// A display card and its PCI vendor id, when sysfs reports one.
pub struct Card {
  pub device: PathBuf,
  pub bus_id: Option<String>,
  pub vendor: Option<String>,
}

/// The entries of a directory, sorted by name as the shell globs were.
fn sorted_entries(directory: &str) -> Vec<PathBuf> {
  let mut entries: Vec<PathBuf> = fs::read_dir(directory)
    .map(|entries| {
      entries
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .collect()
    })
    .unwrap_or_default();
  entries.sort();
  entries
}

/// The text of a sysfs attribute, without the line break that ends it.
fn attribute(path: &Path) -> Option<String> {
  read_text(path).map(|text| text.trim_end_matches('\n').to_string())
}

/// The file stem of a `tempN_input` or `tempN_label` file.
fn temperature_stem(file_name: &str) -> Option<&str> {
  let stem = file_name
    .strip_suffix("_input")
    .or_else(|| file_name.strip_suffix("_label"))?;
  let index = stem.strip_prefix("temp")?;
  (!index.is_empty() && index.bytes().all(|byte| byte.is_ascii_digit())).then_some(stem)
}

/// Every hardware monitor chip, with its temperature sensors.
pub fn hwmons() -> Vec<Hwmon> {
  sorted_entries("/sys/class/hwmon")
    .into_iter()
    .filter_map(|directory| {
      let name = attribute(&directory.join("name"))?;
      let device = fs::canonicalize(directory.join("device")).ok();
      let mut stems: Vec<String> = sorted_entries(directory.to_str()?)
        .iter()
        .filter_map(|path| path.file_name()?.to_str().and_then(temperature_stem))
        .map(String::from)
        .collect();
      stems.sort();
      stems.dedup();
      let temperatures = stems
        .into_iter()
        .map(|stem| Temperature {
          label: attribute(&directory.join(format!("{stem}_label"))),
          millidegrees: attribute(&directory.join(format!("{stem}_input")))
            .and_then(|value| value.trim().parse().ok()),
          stem,
        })
        .collect();
      Some(Hwmon {
        name,
        device,
        temperatures,
      })
    })
    .collect()
}

/// The display cards, in card order. A card is `cardN`, not a connector.
pub fn cards() -> Vec<Card> {
  sorted_entries("/sys/class/drm")
    .into_iter()
    .filter(|path| {
      path
        .file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| name.strip_prefix("card"))
        .is_some_and(|index| !index.is_empty() && index.bytes().all(|byte| byte.is_ascii_digit()))
    })
    .map(|card| {
      let device = card.join("device");
      let vendor = attribute(&device.join("vendor")).map(|vendor| vendor.trim().to_string());
      let device = fs::canonicalize(&device).unwrap_or(device);
      let bus_id = device
        .file_name()
        .and_then(|name| name.to_str())
        .map(String::from);
      Card {
        device,
        bus_id,
        vendor,
      }
    })
    .collect()
}

/// Whether sysfs currently exposes a supported display GPU.
pub fn has_supported_gpu(cards: &[Card]) -> bool {
  cards
    .iter()
    .any(|card| recognized_vendor(card.vendor.as_deref()).is_some())
}

/// The highest valid sensor reading associated with each supported GPU.
pub fn gpu_temperatures(cards: &[Card], hwmons: &[Hwmon]) -> Vec<Option<i64>> {
  let devices = cards
    .iter()
    .filter(|card| recognized_vendor(card.vendor.as_deref()).is_some())
    .map(|card| card.device.clone())
    .collect::<Vec<_>>();
  let hwmon_temperatures: Vec<HwmonTemperatures> = hwmons
    .iter()
    .filter_map(|hwmon| {
      Some((
        hwmon.device.clone()?,
        hwmon
          .temperatures
          .iter()
          .map(|temperature| temperature.celsius())
          .collect(),
      ))
    })
    .collect();
  cards
    .iter()
    .filter(|card| recognized_vendor(card.vendor.as_deref()).is_some())
    .map(|card| gpu::device_temperature(&card.device, &devices, &hwmon_temperatures))
    .collect()
}

/// The names of the kernel modules that are loaded.
pub fn loaded_modules() -> Vec<String> {
  read_text(Path::new("/proc/modules"))
    .unwrap_or_default()
    .lines()
    .filter_map(|line| line.split_whitespace().next().map(String::from))
    .collect()
}

/// The aggregate CPU counters, now.
pub fn cpu_sample() -> Option<Sample> {
  cpu::parse_stat(&read_text(Path::new("/proc/stat"))?)
}
