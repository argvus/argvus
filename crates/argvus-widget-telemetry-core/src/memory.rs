//! The memory block: RAM and swap usage from `/proc/meminfo`. Each figure is
//! rounded to one decimal before the next one is computed, as the script did.

/// The figures of the memory block, in GiB, with the percentages.
#[derive(Debug, Clone, PartialEq)]
pub struct Memory {
  pub ram_total: String,
  pub ram_used: String,
  pub ram_percent: String,
  pub swap_total: String,
  pub swap_used: String,
  pub swap_percent: String,
}

/// The value of a `/proc/meminfo` key in kB, from the first line that names it.
fn kilobytes(meminfo: &str, key: &str) -> f64 {
  meminfo
    .lines()
    .find(|line| line.contains(key))
    .and_then(|line| line.split_whitespace().nth(1))
    .and_then(|value| value.parse::<f64>().ok())
    .unwrap_or(0.0)
}

/// One figure in GiB, with one decimal, as text.
fn gibibytes(kb: f64) -> String {
  format!("{:.1}", kb / 1024.0 / 1024.0)
}

/// Reads the memory figures from the text of `/proc/meminfo`.
pub fn from_meminfo(meminfo: &str) -> Memory {
  let ram_total = gibibytes(kilobytes(meminfo, "MemTotal"));
  let ram_available = gibibytes(kilobytes(meminfo, "MemAvailable"));
  let swap_total = gibibytes(kilobytes(meminfo, "SwapTotal"));
  let swap_free = gibibytes(kilobytes(meminfo, "SwapFree"));
  let ram_used = format!("{:.1}", number(&ram_total) - number(&ram_available));
  let ram_percent = percent(&ram_used, &ram_total);
  let swap_used = format!("{:.1}", number(&swap_total) - number(&swap_free));
  let swap_percent = if swap_total == "0.0" {
    "0".to_string()
  } else {
    percent(&swap_used, &swap_total)
  };
  Memory {
    ram_total,
    ram_used,
    ram_percent,
    swap_total,
    swap_used,
    swap_percent,
  }
}

fn number(text: &str) -> f64 {
  text.parse().unwrap_or(0.0)
}

fn percent(used: &str, total: &str) -> String {
  format!("{:.0}", number(used) / number(total) * 100.0)
}

#[cfg(test)]
mod tests {
  use super::from_meminfo;

  #[test]
  fn figures_are_gibibytes_with_rounded_percentages() {
    let meminfo =
      "MemTotal: 16777216 kB\nMemAvailable: 8388608 kB\nSwapTotal: 0 kB\nSwapFree: 0 kB\n";
    let memory = from_meminfo(meminfo);
    assert_eq!(memory.ram_total, "16.0");
    assert_eq!(memory.ram_used, "8.0");
    assert_eq!(memory.ram_percent, "50");
    assert_eq!(memory.swap_percent, "0");
  }
}
