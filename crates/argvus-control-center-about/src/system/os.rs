//! Implements `os` responsibilities in crate `argvus about`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use std::collections::HashMap;

/// Converts input data into `parse_os_release` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn parse_os_release(input: &str) -> HashMap<String, String> {
  input
    .lines()
    .filter_map(|line| {
      let (key, value) = line.split_once('=')?;
      let value = value.trim().trim_matches('"').replace("\\\"", "\"");
      Some((key.to_string(), value))
    })
    .collect()
}

/// Executes the `os_names` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn os_names(input: &str) -> (String, String) {
  let release = parse_os_release(input);
  let pretty = release
    .get("PRETTY_NAME")
    .or_else(|| release.get("NAME"))
    .cloned()
    .unwrap_or_default();
  let distributor = release.get("NAME").cloned().unwrap_or_default();
  (pretty, distributor)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  /// Converts input data into `parses_os_release_values` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn parses_os_release_values() {
    let parsed = parse_os_release("NAME=\"Arch Linux\"\nPRETTY_NAME=\"Arch Linux\"\n");
    assert_eq!(parsed.get("NAME").map(String::as_str), Some("Arch Linux"));
  }

  #[test]
  /// Executes the `returns_empty_on_unknown_input` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn returns_empty_on_unknown_input() {
    let (pretty, distributor) = os_names("");
    assert!(pretty.is_empty());
    assert!(distributor.is_empty());
  }
}
