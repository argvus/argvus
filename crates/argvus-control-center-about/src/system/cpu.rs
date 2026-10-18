//! Implements `cpu` responsibilities in crate `argvus about`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
pub fn cpu_info(input: &str) -> Option<String> {
  let model = input
    .lines()
    .find_map(|line| {
      line
        .split_once(':')
        .filter(|(key, _)| key.trim() == "model name")
    })
    .map(|(_, value)| value.trim().to_string())?;
  let count = input
    .lines()
    .filter(|line| line.starts_with("processor"))
    .count();
  Some(if count > 1 {
    format!("{model} x {count}")
  } else {
    model
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  /// Converts input data into `parses_cpu_model_and_count` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn parses_cpu_model_and_count() {
    let cpu = "processor\t: 0\nmodel name\t: Example CPU\nprocessor\t: 1\n";
    assert_eq!(cpu_info(cpu).as_deref(), Some("Example CPU x 2"));
  }

  #[test]
  /// Processes `handles_single_core` in this module's event flow. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn handles_single_core() {
    let cpu = "model name\t: Example CPU\n";
    assert_eq!(cpu_info(cpu).as_deref(), Some("Example CPU"));
  }

  #[test]
  /// Executes the `returns_none_without_model` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn returns_none_without_model() {
    assert_eq!(cpu_info("processor\t: 0\n"), None);
  }
}
