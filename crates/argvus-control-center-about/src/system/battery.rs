//! Reads the system batteries from `/sys/class/power_supply`.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use std::fs;
use std::path::Path;

use argvus_control_center_core::sanitize::terminal_text;

/// Directory where the kernel lists batteries, AC adapters and peripheral power supplies.
const POWER_SUPPLY_DIR: &str = "/sys/class/power_supply";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Charging state as reported by the kernel `status` attribute.
pub enum BatteryStatus {
  Charging,
  Discharging,
  Full,
  NotCharging,
  Unknown,
}

impl BatteryStatus {
  /// Maps the kernel `status` text; anything unexpected becomes `Unknown`.
  pub fn parse(value: &str) -> Self {
    match value.trim() {
      "Charging" => Self::Charging,
      "Discharging" => Self::Discharging,
      "Full" => Self::Full,
      "Not charging" => Self::NotCharging,
      _ => Self::Unknown,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Charge, state and wear of one system battery.
pub struct Battery {
  /// Kernel name of the supply, such as `BAT0`, `BAT1` or `CMB0`.
  pub name: String,
  pub percent: Option<u8>,
  pub status: BatteryStatus,
  /// Full capacity as a percentage of the design capacity.
  pub health: Option<u8>,
  pub cycles: Option<u64>,
}

/// Returns every battery that powers the computer, sorted by name.
///
/// Supplies are selected by `type`, not by name, because vendors use different
/// names (`BAT0`, `BAT1`, `CMB0`, `BATT`). Batteries of peripherals such as wireless
/// mice and headsets also have `type == Battery`, but report `scope == Device`.
pub fn batteries() -> Vec<Battery> {
  let Ok(entries) = fs::read_dir(POWER_SUPPLY_DIR) else {
    return Vec::new();
  };
  let mut batteries = entries
    .flatten()
    .filter(|entry| {
      let path = entry.path();
      is_system_battery(
        &read_text(&path, "type").unwrap_or_default(),
        read_text(&path, "scope").as_deref(),
      )
    })
    .map(|entry| read_battery(&entry.path(), &entry.file_name().to_string_lossy()))
    .collect::<Vec<_>>();
  batteries.sort_by(|left, right| left.name.cmp(&right.name));
  batteries
}

/// Tells whether a power supply is a battery of the computer itself.
pub fn is_system_battery(kind: &str, scope: Option<&str>) -> bool {
  kind.trim() == "Battery" && scope.map(str::trim) != Some("Device")
}

/// Computes the wear level from the full and design capacities, in the same unit.
///
/// Firmware reports energy (`energy_*`, µWh) or charge (`charge_*`, µAh); both pairs
/// give the same ratio. A new battery can exceed 100%, which is kept as reported.
pub fn health_percent(full: Option<u64>, design: Option<u64>) -> Option<u8> {
  let (full, design) = (full?, design?);
  if design == 0 {
    return None;
  }
  u8::try_from(full.saturating_mul(100) / design).ok()
}

/// Reads one power supply directory.
fn read_battery(path: &Path, name: &str) -> Battery {
  let full = read_number(path, "energy_full").or_else(|| read_number(path, "charge_full"));
  let design =
    read_number(path, "energy_full_design").or_else(|| read_number(path, "charge_full_design"));
  Battery {
    name: terminal_text(name),
    percent: read_number(path, "capacity").and_then(|value| u8::try_from(value).ok()),
    status: read_text(path, "status")
      .map(|value| BatteryStatus::parse(&value))
      .unwrap_or(BatteryStatus::Unknown),
    health: health_percent(full, design),
    // Many firmwares expose the attribute but leave it at 0 when they do not count cycles.
    cycles: read_number(path, "cycle_count").filter(|cycles| *cycles > 0),
  }
}

fn read_text(path: &Path, attribute: &str) -> Option<String> {
  fs::read_to_string(path.join(attribute))
    .ok()
    .map(|value| value.trim().to_string())
}

fn read_number(path: &Path, attribute: &str) -> Option<u64> {
  read_text(path, attribute)?.parse().ok()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parses_kernel_battery_status() {
    assert_eq!(
      BatteryStatus::parse("Discharging\n"),
      BatteryStatus::Discharging
    );
    assert_eq!(
      BatteryStatus::parse("Not charging"),
      BatteryStatus::NotCharging
    );
    assert_eq!(BatteryStatus::parse("Full"), BatteryStatus::Full);
    assert_eq!(BatteryStatus::parse("weird"), BatteryStatus::Unknown);
  }

  #[test]
  fn keeps_system_batteries_and_drops_peripherals() {
    assert!(is_system_battery("Battery\n", None));
    assert!(is_system_battery("Battery", Some("System")));
    assert!(!is_system_battery("Battery", Some("Device")));
    assert!(!is_system_battery("Mains", None));
    assert!(!is_system_battery("USB", None));
  }

  #[test]
  fn computes_health_from_full_and_design_capacity() {
    assert_eq!(health_percent(Some(45_000_000), Some(50_000_000)), Some(90));
    assert_eq!(
      health_percent(Some(52_000_000), Some(50_000_000)),
      Some(104)
    );
    assert_eq!(health_percent(Some(1), Some(0)), None);
    assert_eq!(health_percent(None, Some(50_000_000)), None);
  }
}
