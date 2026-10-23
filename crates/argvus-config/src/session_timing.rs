//! Duration of each session step, appended to the session log.
//!
//! Login latency is spread over many best-effort steps that never fail, so the
//! only reliable way to find the slow one is to time each of them where it runs.
//! Each step adds one line to `session.log`, next to the entries the session
//! scripts already write there.

use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::Path;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use crate::project::argvus_state_root;

/// File name, under the ARGVUS state directory, that receives the timings.
const SESSION_LOG_FILE: &str = "session.log";

/// Runs `step` and records how long it took under `label`.
///
/// The timing is best effort: a missing state directory or an unwritable log
/// never changes the result of the step.
pub fn timed<T>(label: &str, step: impl FnOnce() -> T) -> T {
  let started = Instant::now();
  let value = step();
  let elapsed_millis = started.elapsed().as_millis();
  if let Ok(state_root) = argvus_state_root() {
    let _ = append_timing(&state_root.join(SESSION_LOG_FILE), label, elapsed_millis);
  }
  value
}

/// Appends one `timing` entry to the log at `log_path`, creating it when missing.
fn append_timing(log_path: &Path, label: &str, elapsed_millis: u128) -> io::Result<()> {
  if let Some(parent) = log_path.parent() {
    std::fs::create_dir_all(parent)?;
  }
  let mut log = OpenOptions::new()
    .create(true)
    .append(true)
    .open(log_path)?;
  // Epoch milliseconds let the entries be lined up with the systemd journal,
  // which uses wall-clock time as well.
  let epoch_millis = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map(|duration| duration.as_millis())
    .unwrap_or_default();
  writeln!(log, "{epoch_millis} timing {label} {elapsed_millis}ms")
}

#[cfg(test)]
mod tests {
  use super::append_timing;
  use std::fs;

  #[test]
  fn appends_one_timing_entry_per_step() {
    let directory =
      std::env::temp_dir().join(format!("argvus-session-timing-{}", std::process::id()));
    let log_path = directory.join("session.log");
    let _ = fs::remove_dir_all(&directory);

    append_timing(&log_path, "first-step", 12).unwrap();
    append_timing(&log_path, "second-step", 0).unwrap();

    let log_text = fs::read_to_string(&log_path).unwrap();
    let _ = fs::remove_dir_all(&directory);
    let entries: Vec<&str> = log_text.lines().collect();
    assert_eq!(entries.len(), 2);
    assert!(entries[0].ends_with(" timing first-step 12ms"));
    assert!(entries[1].ends_with(" timing second-step 0ms"));
  }
}
