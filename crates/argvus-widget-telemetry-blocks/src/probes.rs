//! The probes the taskbar runs: the CPU temperature and the GPU temperature.
//! Each prints the degrees, or nothing, and exits 0 when the value exists. The
//! `--available` and `--available-temp` forms only report whether it exists.

use crate::gpu;
use crate::hardware;
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

/// `gpu --temp | --available-temp`.
pub fn gpu_temp(arguments: &[String]) -> Option<ExitCode> {
  let available_only = match arguments {
    [flag] if flag == "--temp" => false,
    [flag] if flag == "--available-temp" => true,
    _ => return None,
  };
  let temperature = gpu::reading()
    .and_then(|(_, reading)| reading.temperature)
    .map(|celsius| format!("{celsius}°C"));
  Some(report(temperature, available_only))
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
