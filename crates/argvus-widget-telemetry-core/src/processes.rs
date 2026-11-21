//! Process rows and ranking by recent CPU and resident memory usage.

/// A line of `ps -eo pid,pcpu,pmem,comm --no-headers`.
#[derive(Debug, Clone, PartialEq)]
pub struct Process {
  pub pid: String,
  pub cpu: f64,
  pub memory: f64,
  pub command: String,
}

/// Reads the lines of `ps`. A line without the four fields is skipped.
pub fn parse_ps(text: &str) -> Vec<Process> {
  text
    .lines()
    .filter_map(|line| {
      let mut fields = line.split_whitespace();
      let pid = fields.next()?.to_string();
      let cpu: f64 = fields.next()?.parse().ok()?;
      let memory: f64 = fields.next()?.parse().ok()?;
      let command = fields.collect::<Vec<_>>().join(" ");
      if command.is_empty() {
        return None;
      }
      Some(Process {
        pid,
        cpu,
        memory,
        command,
      })
    })
    .collect()
}

/// Rank by the sum of CPU and memory percentages, then by CPU and PID.
pub fn busiest(mut processes: Vec<Process>, count: usize) -> Vec<Process> {
  processes.sort_by(|left, right| {
    (right.cpu + right.memory)
      .total_cmp(&(left.cpu + left.memory))
      .then_with(|| right.cpu.total_cmp(&left.cpu))
      .then_with(|| left.pid.cmp(&right.pid))
  });
  processes.truncate(count);
  processes
}

/// The command, cut to `max` characters with an ellipsis when it is longer.
pub fn truncate_command(command: &str, max: usize) -> String {
  if command.chars().count() <= max {
    return command.to_string();
  }
  let kept: String = command.chars().take(max.saturating_sub(3)).collect();
  format!("{kept}...")
}

#[cfg(test)]
mod tests {
  use super::{busiest, parse_ps, truncate_command};

  #[test]
  fn busiest_accounts_for_memory() {
    let text = "10 5.0 1.0 bash\n11 9.5 2.0 firefox\n12 5.0 0.5 zed\n";
    let rows = busiest(parse_ps(text), 2);
    let pids: Vec<&str> = rows.iter().map(|row| row.pid.as_str()).collect();
    assert_eq!(pids, ["11", "10"]);
  }

  #[test]
  fn command_keeps_its_spaces_and_is_cut() {
    let processes = parse_ps("1 0.0 0.0 Web Content  process\n");
    assert_eq!(processes[0].command, "Web Content process");
    assert_eq!(
      truncate_command("abcdefghijklmnopqrstuvwxyz", 20),
      "abcdefghijklmnopq..."
    );
    assert_eq!(truncate_command("short", 20), "short");
  }
}
