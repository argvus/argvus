//! The processes block: the five busiest processes by CPU.

use argvus_runtime::messages;
use argvus_runtime::process::output;
use argvus_widget_telemetry_core::processes::{busiest, parse_ps, truncate_command};

/// How many processes the block lists.
const LISTED_PROCESSES: usize = 5;
/// The longest command shown; a longer one is cut with an ellipsis.
const COMMAND_WIDTH: usize = 20;

/// The table of the block, one line per process, with the header first.
pub fn table() -> String {
  let header = format!(
    "{:<7} {:<5} {:<5} {}",
    messages::text("label.pid", &[]),
    messages::text("label.cpu_percent", &[]),
    messages::text("label.memory_percent", &[]),
    messages::text("label.command", &[]),
  );
  let listing = output("ps", &["-eo", "pid,pcpu,pmem,comm", "--no-headers"]).unwrap_or_default();
  let rows = busiest(parse_ps(&listing), LISTED_PROCESSES)
    .into_iter()
    .map(|process| {
      format!(
        "{:<7} {:<5.1} {:<5.1} {}",
        process.pid,
        process.cpu,
        process.memory,
        truncate_command(&process.command, COMMAND_WIDTH),
      )
    });
  std::iter::once(header)
    .chain(rows)
    .collect::<Vec<_>>()
    .join("\n")
}
