//! Operating system names read from `/etc/os-release`.
use std::collections::HashMap;

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
  fn parses_os_release_values() {
    let parsed = parse_os_release("NAME=\"Arch Linux\"\nPRETTY_NAME=\"Arch Linux\"\n");
    assert_eq!(parsed.get("NAME").map(String::as_str), Some("Arch Linux"));
  }

  #[test]
  fn returns_empty_on_unknown_input() {
    let (pretty, distributor) = os_names("");
    assert!(pretty.is_empty());
    assert!(distributor.is_empty());
  }
}
