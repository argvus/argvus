//! Reads RAM and swap totals and usage from `/proc/meminfo`.

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
/// Memory and swap sizes in KiB, as `/proc/meminfo` reports them.
pub struct MemoryUsage {
  pub total_kib: u64,
  /// Memory the kernel can give to new programs without swapping (`MemAvailable`).
  pub available_kib: Option<u64>,
  pub swap_total_kib: Option<u64>,
  pub swap_free_kib: Option<u64>,
}

impl MemoryUsage {
  /// RAM in use: everything that is not available to new programs.
  pub fn used_kib(&self) -> Option<u64> {
    self
      .available_kib
      .map(|available| self.total_kib.saturating_sub(available))
  }

  /// Swap in use, including zram devices; `None` when swap is not reported.
  pub fn swap_used_kib(&self) -> Option<u64> {
    let total = self.swap_total_kib?;
    Some(total.saturating_sub(self.swap_free_kib.unwrap_or(total)))
  }
}

/// Parses `/proc/meminfo`; `None` without `MemTotal`.
pub fn memory_usage(input: &str) -> Option<MemoryUsage> {
  let value = |name: &str| {
    input.lines().find_map(|line| {
      let (key, value) = line.split_once(':')?;
      (key == name)
        .then(|| value.split_whitespace().next()?.parse::<u64>().ok())
        .flatten()
    })
  };
  Some(MemoryUsage {
    total_kib: value("MemTotal")?,
    available_kib: value("MemAvailable"),
    swap_total_kib: value("SwapTotal"),
    swap_free_kib: value("SwapFree"),
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn reads_memory_and_swap_usage() {
    let meminfo = "MemTotal:       16000000 kB\nMemAvailable:   10000000 kB\n\
SwapTotal:       8000000 kB\nSwapFree:        6000000 kB\n";
    let usage = memory_usage(meminfo).expect("usage");
    assert_eq!(usage.used_kib(), Some(6_000_000));
    assert_eq!(usage.swap_used_kib(), Some(2_000_000));
  }

  #[test]
  fn reports_zero_swap_when_disabled() {
    let usage =
      memory_usage("MemTotal: 8000000 kB\nSwapTotal: 0 kB\nSwapFree: 0 kB\n").expect("usage");
    assert_eq!(usage.swap_total_kib, Some(0));
    assert_eq!(usage.used_kib(), None);
  }

  #[test]
  fn returns_none_without_memtotal() {
    assert_eq!(memory_usage("MemAvailable: 123 kB\n"), None);
  }
}
