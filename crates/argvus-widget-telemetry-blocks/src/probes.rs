//! The probes the taskbar runs: the CPU temperature and the GPU temperature.
//! Each prints the degrees, or nothing, and exits 0 when the value exists. The
//! `--available` and `--available-temp` forms only report whether it exists.

use crate::gpu;
use crate::hardware;
use argvus_runtime::messages;
use argvus_runtime::process::program_exists;
use argvus_widget_telemetry_core::gpu::maximum_temperature;
use argvus_widget_telemetry_core::sensors::cpu_celsius;
use std::process::ExitCode;

/// `cpu-temp [--available]`.
pub fn cpu_temp(arguments: &[String]) -> Option<ExitCode> {
  let available_only = match arguments {
    [] => false,
    [flag] if flag == "--available" => true,
    _ => return None,
  };
  Some(report(
    cpu_celsius(&hardware::hwmons()).map(|celsius| celsius.to_string()),
    available_only,
  ))
}

/// `gpu --temp | --available-temp | --present | --available`.
pub fn gpu_temp(arguments: &[String]) -> Option<ExitCode> {
  match arguments {
    [flag] if flag == "--present" => Some(report_presence(gpu::present())),
    [flag] if flag == "--available" => Some(report_presence(gpu_panel_available(
      gpu::present(),
      program_exists("btop"),
    ))),
    [flag] if flag == "--available-temp" => Some(report(
      maximum_temperature(gpu::temperatures()).map(|celsius| celsius.to_string()),
      true,
    )),
    [flag] if flag == "--temp" => {
      let temperature = gpu_temperature_text(
        &gpu::temperatures(),
        &messages::text("gpu.temperature.unavailable", &[]),
      );
      println!("{temperature}");
      Some(ExitCode::SUCCESS)
    }
    _ => None,
  }
}

fn gpu_panel_available(has_supported_gpu: bool, has_btop: bool) -> bool {
  has_supported_gpu && has_btop
}

fn gpu_temperature_text(temperatures: &[Option<i64>], unavailable: &str) -> String {
  maximum_temperature(temperatures.iter().copied())
    .map(|celsius| format!("{celsius}°C"))
    .unwrap_or_else(|| unavailable.to_string())
}

/// Reports a capability using the same success/failure contract as sensors.
fn report_presence(is_available: bool) -> ExitCode {
  if is_available {
    ExitCode::SUCCESS
  } else {
    ExitCode::from(1)
  }
}

/// Prints a value and exits 0, or exits 1 without output when there is none.
fn report(value: Option<String>, available_only: bool) -> ExitCode {
  match value {
    Some(value) => {
      if !available_only {
        println!("{value}");
      }
      ExitCode::SUCCESS
    }
    None => ExitCode::from(1),
  }
}

#[cfg(test)]
mod tests {
  use super::{gpu_panel_available, gpu_temperature_text};

  #[test]
  fn panel_requires_a_supported_gpu_and_btop() {
    assert!(gpu_panel_available(true, true));
    assert!(!gpu_panel_available(false, true));
    assert!(!gpu_panel_available(true, false));
  }

  #[test]
  fn temperature_uses_maximum_or_localized_unavailable_state() {
    assert_eq!(
      gpu_temperature_text(&[Some(45), None, Some(68)], "Indisponível"),
      "68°C"
    );
    assert_eq!(
      gpu_temperature_text(&[None, Some(250)], "Indisponível"),
      "Indisponível"
    );
  }
}
