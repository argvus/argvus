//! The CPU usage of the CPU/GPU block, from the aggregate line of `/proc/stat`.
//! Two samples give the share of the time that was not idle between them.

/// One reading of the aggregate CPU counters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sample {
  idle: u64,
  total: u64,
}

/// Reads the `cpu` line of `/proc/stat`. The idle counter is its fourth value.
pub fn parse_stat(text: &str) -> Option<Sample> {
  let line = text
    .lines()
    .find(|line| line.split_whitespace().next() == Some("cpu"))?;
  let values: Vec<u64> = line
    .split_whitespace()
    .skip(1)
    .map(str::parse)
    .collect::<Result<_, _>>()
    .ok()?;
  Some(Sample {
    idle: *values.get(3)?,
    total: values.iter().sum(),
  })
}

/// The busy share between two samples, in whole percent, truncated. Without
/// time between the samples there is no share.
pub fn usage_percent(before: Sample, after: Sample) -> Option<i64> {
  let total = after.total.checked_sub(before.total)?;
  let idle = after.idle.checked_sub(before.idle)?;
  if total == 0 {
    return None;
  }
  Some(((1.0 - idle as f64 / total as f64) * 100.0) as i64)
}

#[cfg(test)]
mod tests {
  use super::{Sample, parse_stat, usage_percent};

  #[test]
  fn busy_share_is_the_non_idle_time() {
    let before = parse_stat("cpu  100 0 100 800 0 0 0 0 0 0\n").unwrap();
    let after = parse_stat("cpu  150 0 150 900 0 0 0 0 0 0\n").unwrap();
    assert_eq!(usage_percent(before, after), Some(50));
  }

  #[test]
  fn no_elapsed_time_has_no_share() {
    let sample = Sample { idle: 1, total: 2 };
    assert_eq!(usage_percent(sample, sample), None);
    assert_eq!(parse_stat("intr 1\n"), None);
  }
}
