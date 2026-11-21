//! The processes block: the five busiest processes by CPU and memory.

use std::fs;
use std::thread;
use std::time::{Duration, Instant};

use argvus_runtime::messages;
use argvus_runtime::process::output;
use argvus_widget_telemetry_core::processes::{Process, busiest, parse_ps, truncate_command};

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
  let rows = busiest(sample(), LISTED_PROCESSES)
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

/// Process identity and CPU ticks from Linux /proc/<pid>/stat.
fn cpu_ticks(pid: &str) -> Option<(u64, u64)> {
  let stat = fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
  let (_, fields) = stat.rsplit_once(") ")?;
  let mut fields = fields.split_whitespace();
  let user: u64 = fields.nth(11)?.parse().ok()?;
  let system: u64 = fields.next()?.parse().ok()?;
  let start = fields.nth(6)?.parse().ok()?;
  Some((start, user.checked_add(system)?))
}

fn sample() -> Vec<Process> {
  let Some(hz) = output("getconf", &["CLK_TCK"])
    .and_then(|value| value.trim().parse::<u32>().ok())
    .filter(|&value| value > 0)
  else {
    return Vec::new();
  };
  let listing = output("ps", &["-eo", "pid,pcpu,pmem,comm", "--no-headers"]).unwrap_or_default();
  let own_pid = std::process::id().to_string();
  let baseline: Vec<_> = parse_ps(&listing)
    .into_iter()
    .filter(|process| process.pid != own_pid)
    .filter_map(|process| {
      let (start, ticks) = cpu_ticks(&process.pid)?;
      Some((process, start, ticks, Instant::now()))
    })
    .collect();
  thread::sleep(Duration::from_millis(250));
  baseline
    .into_iter()
    .filter_map(|(mut process, start, ticks, sampled)| {
      let (current_start, current_ticks) = cpu_ticks(&process.pid)?;
      if current_start != start {
        return None;
      }
      let delta = current_ticks.checked_sub(ticks)?;
      process.cpu = 100.0 * delta as f64 / (f64::from(hz) * sampled.elapsed().as_secs_f64());
      Some(process)
    })
    .collect()
}
