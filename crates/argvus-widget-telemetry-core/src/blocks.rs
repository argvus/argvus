//! The telemetry blocks: their names, their state (enabled or disabled), and
//! the order they are shown in. Stored in the blocks file as `name=state` lines
//! and a `__order__=` line with the comma separated order.

/// Every block the widget knows, in the order a fresh install shows them.
pub const BLOCK_NAMES: [&str; 8] = [
  "system",
  "cpu_gpu",
  "memory",
  "storage",
  "processes",
  "network",
  "dev_dashboard",
  "keys",
];

/// The key of the order line in the blocks file.
pub const ORDER_KEY: &str = "__order__";

/// Whether a name is one of the blocks.
pub fn is_block(name: &str) -> bool {
  BLOCK_NAMES.contains(&name)
}

/// The state of a block as stored. `None` for an unknown or missing state.
pub fn parse_state(value: &str) -> Option<bool> {
  match value {
    "enabled" => Some(true),
    "disabled" => Some(false),
    _ => None,
  }
}

pub fn state_name(enabled: bool) -> &'static str {
  if enabled { "enabled" } else { "disabled" }
}

/// The stored blocks: state per block and the stored order line.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Blocks {
  /// The `name=state` lines, in the file's order, with their values.
  states: Vec<(String, String)>,
  /// The value of the order line, if the file had one.
  order: Option<String>,
}

impl Blocks {
  /// Reads the text of a blocks file. Only the first line of each key counts,
  /// as the script's `sed -n 1p` did.
  pub fn parse(text: &str) -> Self {
    let mut blocks = Self::default();
    for line in text.lines() {
      let Some((key, value)) = line.split_once('=') else {
        continue;
      };
      if key == ORDER_KEY {
        if blocks.order.is_none() {
          blocks.order = Some(value.to_string());
        }
        continue;
      }
      if !blocks.states.iter().any(|(name, _)| name == key) {
        blocks.states.push((key.to_string(), value.to_string()));
      }
    }
    blocks
  }

  /// The state of a block. A missing or invalid state means enabled, so an
  /// upgrade keeps the complete legacy widget.
  pub fn enabled(&self, block: &str) -> bool {
    self
      .states
      .iter()
      .find(|(name, _)| name == block)
      .and_then(|(_, value)| parse_state(value))
      .unwrap_or(true)
  }

  /// The display order: the stored order, without names that are no longer
  /// blocks, then every block the stored order does not have, in default order.
  pub fn order(&self) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    for name in self.order.as_deref().unwrap_or("").split(',') {
      if is_block(name) && !result.iter().any(|item| item == name) {
        result.push(name.to_string());
      }
    }
    for name in BLOCK_NAMES {
      if !result.iter().any(|item| item == name) {
        result.push(name.to_string());
      }
    }
    result
  }
}

/// Whether a comma separated order names every block exactly once.
pub fn valid_order(order: &str) -> bool {
  let names: Vec<&str> = order.split(',').collect();
  if names.len() != BLOCK_NAMES.len() {
    return false;
  }
  let mut seen: Vec<&str> = Vec::new();
  for name in names {
    if !is_block(name) || seen.contains(&name) {
      return false;
    }
    seen.push(name);
  }
  true
}

/// The blocks file with the states written in the given order.
pub fn render_in_order(order: &str, states: &[(&str, bool)]) -> String {
  let mut text = format!("{ORDER_KEY}={order}\n");
  for (block, enabled) in states {
    text.push_str(&format!("{block}={}\n", state_name(*enabled)));
  }
  text
}

/// The blocks file for a state of every block and an order.
pub fn render(order: &str, states: &[(&str, bool)]) -> String {
  let mut text = format!("{ORDER_KEY}={order}\n");
  for block in BLOCK_NAMES {
    let enabled = states
      .iter()
      .find(|(name, _)| *name == block)
      .map(|(_, enabled)| *enabled)
      .unwrap_or(true);
    text.push_str(&format!("{block}={}\n", state_name(enabled)));
  }
  text
}

#[cfg(test)]
mod tests {
  use super::{Blocks, render, valid_order};

  #[test]
  fn missing_state_means_enabled_and_order_is_reconciled() {
    let blocks = Blocks::parse("__order__=keys,bogus,system,keys\nsystem=disabled\n");
    assert!(!blocks.enabled("system"));
    assert!(blocks.enabled("memory"));
    let order = blocks.order();
    assert_eq!(&order[..2], &["keys".to_string(), "system".to_string()]);
    assert_eq!(order.len(), 8);
  }

  #[test]
  fn order_must_name_every_block_once() {
    assert!(valid_order(
      "system,cpu_gpu,memory,storage,processes,network,dev_dashboard,keys"
    ));
    assert!(!valid_order(
      "system,system,memory,storage,processes,network,dev_dashboard,keys"
    ));
    assert!(!valid_order("system"));
  }

  #[test]
  fn rendered_file_lists_the_order_then_every_block() {
    let text = render("keys,system", &[("system", false)]);
    assert!(text.starts_with("__order__=keys,system\nsystem=disabled\ncpu_gpu=enabled\n"));
  }
}
