//! The memory block: the RAM and swap table of the widget.

use argvus_runtime::messages;
use argvus_runtime::process::read_text;
use argvus_widget_telemetry_core::memory::{self, Memory};
use argvus_widget_telemetry_core::output;
use std::path::Path;

/// The table of the block, one line per row, with the header first.
pub fn table() -> String {
  let meminfo = read_text(Path::new("/proc/meminfo")).unwrap_or_default();
  let figures = memory::from_meminfo(&meminfo);
  [
    row(
      &messages::text("label.type", &[]),
      &messages::text("label.used", &[]),
      &messages::text("label.percent", &[]),
      "",
    ),
    figure_row("RAM", &figures, Kind::Ram),
    figure_row("Swap", &figures, Kind::Swap),
  ]
  .join("\n")
}

enum Kind {
  Ram,
  Swap,
}

fn figure_row(name: &str, figures: &Memory, kind: Kind) -> String {
  let (used, total, percent) = match kind {
    Kind::Ram => (&figures.ram_used, &figures.ram_total, &figures.ram_percent),
    Kind::Swap => (
      &figures.swap_used,
      &figures.swap_total,
      &figures.swap_percent,
    ),
  };
  row(
    name,
    &format!("{used}/{total}G"),
    &format!("{percent}%"),
    &output::bar(percent.parse().unwrap_or(0)),
  )
}

fn row(kind: &str, used: &str, percent: &str, bar: &str) -> String {
  format!("{kind:<7} {used:<13} {percent:<7} {bar}")
}
