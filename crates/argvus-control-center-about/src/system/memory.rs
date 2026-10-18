//! Implements `memory` responsibilities in crate `argvus about`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
pub fn memory_info(input: &str) -> Option<String> {
  let kib = input.lines().find_map(|line| {
    let (key, value) = line.split_once(':')?;
    if key == "MemTotal" {
      value.split_whitespace().next()?.parse::<u64>().ok()
    } else {
      None
    }
  })?;
  Some(format!("{:.1} GiB", kib as f64 / 1024.0 / 1024.0))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  /// Executes the `formats_memory_as_gib` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn formats_memory_as_gib() {
    assert_eq!(
      memory_info("MemTotal:       8388608 kB\n").as_deref(),
      Some("8.0 GiB")
    );
  }

  #[test]
  /// Executes the `returns_none_without_memtotal` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn returns_none_without_memtotal() {
    assert_eq!(memory_info("MemAvailable: 123 kB\n"), None);
  }
}
