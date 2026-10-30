//! The temperature sensors of the hardware monitor (hwmon) interface. The files
//! are read by the binary; the rules that choose a sensor live here.

use std::path::PathBuf;

/// The chip drivers whose sensors describe the CPU.
const CPU_DRIVERS: [&str; 3] = ["coretemp", "k10temp", "zenpower"];
/// The sensor labels that name the CPU package, in the order they are preferred.
const CPU_LABELS: [&str; 4] = ["Package id 0", "Tctl", "Tdie", "Package"];

/// One `tempN` sensor of a chip: its label, if the chip names it, and its
/// reading in millidegrees Celsius.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Temperature {
  /// The file stem of the sensor, such as `temp1`.
  pub stem: String,
  pub label: Option<String>,
  pub millidegrees: Option<i64>,
}

impl Temperature {
  /// The reading in whole degrees Celsius, rounded half to even as printf does.
  pub fn celsius(&self) -> Option<i64> {
    self
      .millidegrees
      .map(|millidegrees| (millidegrees as f64 / 1000.0).round_ties_even() as i64)
  }
}

/// A hardware monitor chip and its temperature sensors, in name order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hwmon {
  pub name: String,
  /// Resolved device link, for safe association with a GPU.
  pub device: Option<PathBuf>,
  pub temperatures: Vec<Temperature>,
}

/// The CPU temperature in whole degrees. The CPU chips are read first: their
/// package sensor, then their first sensor. Any other sensor is the fallback.
pub fn cpu_celsius(hwmons: &[Hwmon]) -> Option<i64> {
  for hwmon in hwmons
    .iter()
    .filter(|hwmon| CPU_DRIVERS.contains(&hwmon.name.as_str()))
  {
    let labelled = hwmon.temperatures.iter().find(|temperature| {
      temperature
        .label
        .as_deref()
        .is_some_and(|label| CPU_LABELS.contains(&label))
    });
    if let Some(celsius) = labelled.and_then(Temperature::celsius) {
      return Some(celsius);
    }
    let first = hwmon
      .temperatures
      .iter()
      .find(|temperature| temperature.stem == "temp1");
    if let Some(celsius) = first.and_then(Temperature::celsius) {
      return Some(celsius);
    }
  }
  hwmons
    .iter()
    .flat_map(|hwmon| hwmon.temperatures.iter())
    .find_map(Temperature::celsius)
}

/// The temperature of the first chip named in `names` that reports one. Its
/// sensors are tried in the order of `stems`.
pub fn chip_celsius(hwmons: &[Hwmon], names: &[&str], stems: &[&str]) -> Option<i64> {
  hwmons
    .iter()
    .filter(|hwmon| names.contains(&hwmon.name.as_str()))
    .find_map(|hwmon| {
      stems.iter().find_map(|stem| {
        hwmon
          .temperatures
          .iter()
          .find(|temperature| temperature.stem == *stem)
          .and_then(Temperature::celsius)
      })
    })
}

#[cfg(test)]
mod tests {
  use super::{Hwmon, Temperature, chip_celsius, cpu_celsius};

  fn sensor(stem: &str, label: Option<&str>, millidegrees: Option<i64>) -> Temperature {
    Temperature {
      stem: stem.to_string(),
      label: label.map(String::from),
      millidegrees,
    }
  }

  #[test]
  fn cpu_prefers_the_package_label() {
    let hwmons = vec![
      Hwmon {
        name: "acpitz".into(),
        device: None,
        temperatures: vec![sensor("temp1", None, Some(90_000))],
      },
      Hwmon {
        name: "coretemp".into(),
        device: None,
        temperatures: vec![
          sensor("temp1", Some("Core 0"), Some(51_000)),
          sensor("temp2", Some("Package id 0"), Some(62_400)),
        ],
      },
    ];
    assert_eq!(cpu_celsius(&hwmons), Some(62));
  }

  #[test]
  fn cpu_falls_back_to_the_first_sensor_then_to_any_sensor() {
    let cpu_without_label = vec![Hwmon {
      name: "k10temp".into(),
      device: None,
      temperatures: vec![sensor("temp1", None, Some(45_500))],
    }];
    assert_eq!(cpu_celsius(&cpu_without_label), Some(46));

    let other_chip = vec![Hwmon {
      name: "nvme".into(),
      device: None,
      temperatures: vec![sensor("temp1", None, Some(38_000))],
    }];
    assert_eq!(cpu_celsius(&other_chip), Some(38));
    assert_eq!(cpu_celsius(&[]), None);
  }

  #[test]
  fn chip_reads_the_first_available_stem() {
    let hwmons = vec![Hwmon {
      name: "amdgpu".into(),
      device: None,
      temperatures: vec![sensor("temp1", None, Some(55_000))],
    }];
    assert_eq!(
      chip_celsius(&hwmons, &["amdgpu"], &["temp2", "temp1"]),
      Some(55)
    );
    assert_eq!(chip_celsius(&hwmons, &["radeon"], &["temp1"]), None);
  }
}
