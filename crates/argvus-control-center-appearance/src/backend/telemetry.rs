//! Widget telemetry: the enabled state, its blocks and their order.

use super::commands::read_first_or_default;
use super::*;

pub(super) fn telemetry_state() -> bool {
  if let Ok(output) = SystemProcessRunner.run(
    &ProcessRequest::new("env")
      .arg("ARGVUS_MACHINE_OUTPUT=1")
      .arg("argvus")
      .arg("widget-telemetry-toggle")
      .arg("status")
      .timeout(Duration::from_secs(3)),
  ) && output.status.is_none_or(|status| status == 0)
  {
    match terminal_text(&String::from_utf8_lossy(&output.stdout)).trim() {
      "enabled" => return true,
      "disabled" => return false,
      _ => {}
    }
  }
  let cache = cache_home().join("argvus").join("waybar");
  let candidates = [
    cache.join("widget-telemetry-state"),
    cache.join("desktop-telemetry-state"),
    cache.join("sysinfo-state"),
  ];
  for candidate in candidates {
    match read_first_or_default(&candidate).as_deref() {
      Some("enabled") => return true,
      Some("disabled") => return false,
      _ => {}
    }
  }
  false
}

/// Maps the widget package's stable command-line identifier back to its
/// enum variant, shared by the block-state and display-order readers.
pub(super) fn telemetry_block_from_key(key: &str) -> Option<WidgetTelemetryBlock> {
  Some(match key {
    "system" => WidgetTelemetryBlock::System,
    "cpu_gpu" => WidgetTelemetryBlock::CpuGpu,
    "memory" => WidgetTelemetryBlock::Memory,
    "storage" => WidgetTelemetryBlock::Storage,
    "processes" => WidgetTelemetryBlock::Processes,
    "network" => WidgetTelemetryBlock::Network,
    "keys" => WidgetTelemetryBlock::Shortcuts,
    "dev_dashboard" => WidgetTelemetryBlock::DevDashboard,
    _ => return None,
  })
}

/// Runs `argvus-widget-telemetry-toggle blocks status` once, whose output is
/// shared by [`telemetry_blocks`] and [`telemetry_order`].
pub(super) fn telemetry_blocks_status() -> Option<String> {
  let output = SystemProcessRunner
    .run(
      &ProcessRequest::new("env")
        .arg("ARGVUS_MACHINE_OUTPUT=1")
        .arg("argvus")
        .arg("widget-telemetry-toggle")
        .arg("blocks")
        .arg("status")
        .timeout(Duration::from_secs(3)),
    )
    .ok()?;
  if output.status.is_some_and(|status| status != 0) {
    return None;
  }
  Some(terminal_text(&String::from_utf8_lossy(&output.stdout)))
}

/// Reads the widget-owned machine format while retaining enabled defaults when
/// an older package does not yet expose block preferences.
pub(super) fn telemetry_blocks(status: &str) -> WidgetTelemetryBlocks {
  let mut blocks = WidgetTelemetryBlocks::default();
  for line in status.lines() {
    let Some((key, value)) = line.split_once('=') else {
      continue;
    };
    let Some(block) = telemetry_block_from_key(key.trim()) else {
      continue;
    };
    match value.trim() {
      "enabled" => blocks.set(block, true),
      "disabled" => blocks.set(block, false),
      _ => {}
    }
  }
  blocks
}

/// Reads the persisted display order, appending any block missing from it
/// (an older package install, before this preference existed) at the end.
pub(super) fn telemetry_order(status: &str) -> Vec<WidgetTelemetryBlock> {
  let mut order = Vec::new();
  if let Some(value) = status
    .lines()
    .find_map(|line| line.strip_prefix("__order__="))
  {
    for key in value.split(',') {
      if let Some(block) = telemetry_block_from_key(key.trim())
        && !order.contains(&block)
      {
        order.push(block);
      }
    }
  }
  for block in WidgetTelemetryBlock::ALL {
    if !order.contains(&block) {
      order.push(block);
    }
  }
  order
}

/// The widget switch and the raw `blocks status` output.
pub(super) struct TelemetryReading {
  pub(super) enabled: bool,
  pub(super) blocks_status: String,
}

pub(super) fn read_telemetry() -> TelemetryReading {
  std::thread::scope(|scope| {
    let blocks_status = scope.spawn(telemetry_blocks_status);
    let enabled = telemetry_state();
    TelemetryReading {
      enabled,
      blocks_status: blocks_status.join().ok().flatten().unwrap_or_default(),
    }
  })
}

pub fn apply_widget_telemetry(
  enabled: bool,
  blocks: &WidgetTelemetryBlocks,
  order: &[WidgetTelemetryBlock],
) -> Result<(), String> {
  let values = WidgetTelemetryBlock::ALL.map(|block| {
    if blocks.enabled(block) {
      "enabled"
    } else {
      "disabled"
    }
  });
  let order = order
    .iter()
    .map(|block| block.key())
    .collect::<Vec<_>>()
    .join(",");
  let mut args = vec!["apply-state", if enabled { "enabled" } else { "disabled" }];
  args.extend(values);
  args.push(&order);
  let mut request = ProcessRequest::new("argvus").arg("widget-telemetry-toggle");
  for argument in args {
    request = request.arg(argument);
  }
  let output = SystemProcessRunner
    .run(&request)
    .map_err(|error| error.to_string())?;
  if output.status.is_none_or(|status| status != 0) {
    return Err(terminal_text(&String::from_utf8_lossy(&output.stderr)).to_string());
  }
  Ok(())
}
