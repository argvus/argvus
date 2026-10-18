//! The CPU/GPU block: the temperature and the usage of each processor, as a
//! table with a bar of the usage.

use crate::gpu;
use crate::hardware;
use argvus_runtime::messages;
use argvus_widget_telemetry_core::cpu::usage_percent;
use argvus_widget_telemetry_core::output;
use argvus_widget_telemetry_core::sensors::cpu_celsius;
use std::thread;
use std::time::Duration;

/// The time between the two CPU samples that give the usage.
const SAMPLE_INTERVAL: Duration = Duration::from_millis(200);

/// The table of the block, with the header first. A figure that is not known
/// reads `N/A`.
pub fn table() -> String {
  let cpu_temperature = cpu_celsius(&hardware::hwmons()).map(|celsius| format!("{celsius}°C"));
  let cpu_usage = cpu_usage();
  let gpu_reading = gpu::reading().map(|(_, reading)| reading);
  let gpu_temperature = gpu_reading
    .as_ref()
    .and_then(|reading| reading.temperature)
    .map(|celsius| format!("{celsius}°C"));
  let gpu_usage = gpu_reading.as_ref().and_then(|reading| reading.usage);

  let header = format!(
    "{:<12} {:<7} {:<6} {}",
    messages::text("label.device", &[]),
    messages::text("label.temperature", &[]),
    messages::text("label.usage", &[]),
    "",
  );
  [
    header,
    row("CPU", cpu_temperature, cpu_usage.map(i64::from)),
    row("GPU", gpu_temperature, gpu_usage.map(i64::from)),
  ]
  .join("\n")
}

/// The CPU usage between two samples of `/proc/stat`.
fn cpu_usage() -> Option<u32> {
  let before = hardware::cpu_sample()?;
  thread::sleep(SAMPLE_INTERVAL);
  let after = hardware::cpu_sample()?;
  usage_percent(before, after).and_then(|percent| u32::try_from(percent).ok())
}

fn row(device: &str, temperature: Option<String>, usage: Option<i64>) -> String {
  let temperature = temperature.unwrap_or_else(|| "N/A".to_string());
  let (usage_text, bar) = match usage {
    Some(percent) => (
      format!("{percent}%"),
      output::bar(u64::try_from(percent).unwrap_or(0)),
    ),
    None => ("N/A".to_string(), output::bar(0)),
  };
  format!("{device:<12} {temperature:<7} {usage_text:<6} {bar}")
}
