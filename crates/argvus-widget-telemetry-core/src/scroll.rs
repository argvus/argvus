//! The scroll window. When the enabled blocks are taller than the focused
//! monitor, the wheel slides a window over them. Heights are estimates in pixels
//! calibrated to the stylesheet, so the layout does not depend on the rendered
//! output of every block.

use crate::blocks::BLOCK_NAMES;

/// Height of a block's header, in pixels.
pub const HEADER_PX: u64 = 28;
/// Height of one text line of a block, in pixels.
pub const ROW_PX: u64 = 19;
/// Space between two blocks, in pixels.
pub const BLOCK_GAP_PX: u64 = 8;

/// Text lines a block renders, excluding its header.
pub fn block_lines(block: &str) -> u64 {
  match block {
    "system" => 9,
    "processes" => 6,
    "network" => 2,
    "keys" => 1,
    _ => 3,
  }
}

/// The height of a block: its header, its lines and the gap after it.
pub fn block_px(block: &str) -> u64 {
  HEADER_PX + block_lines(block) * ROW_PX + BLOCK_GAP_PX
}

/// The window of blocks that is shown, and where it can scroll.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Layout {
  pub visible: Vec<String>,
  pub first: Option<String>,
  pub last: Option<String>,
  pub offset: usize,
  pub max: usize,
  pub above: bool,
  pub below: bool,
}

/// The stored offset: the first line of the state file when it is a number,
/// otherwise zero.
pub fn parse_offset(text: Option<&str>) -> usize {
  text
    .and_then(|value| value.lines().next())
    .filter(|value| !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()))
    .and_then(|value| value.parse().ok())
    .unwrap_or(0)
}

/// The budget in pixels: a number, or `None` when the monitor could not answer.
pub fn parse_budget(text: Option<&str>) -> Option<u64> {
  let value = text?.trim();
  if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
    return None;
  }
  value.parse().ok()
}

/// Computes the window of the enabled blocks, in display order. The offset is
/// clamped so the list never scrolls past its last block, and everything is
/// shown when the enabled blocks fit the budget.
pub fn layout(enabled: &[String], budget: Option<u64>, stored_offset: usize) -> Layout {
  let items: Vec<(String, u64)> = enabled
    .iter()
    .filter(|block| BLOCK_NAMES.contains(&block.as_str()))
    .map(|block| (block.clone(), block_px(block)))
    .collect();
  let count = items.len();
  let total: u64 = items.iter().map(|(_, px)| px).sum();
  let budget = budget.unwrap_or(total);

  // The largest offset from which the rest of the list still fits the budget.
  let mut max = 0usize;
  let mut prefix = 0u64;
  for (_, px) in &items {
    if total - prefix <= budget {
      break;
    }
    prefix += px;
    max += 1;
  }
  if count == 0 {
    max = 0;
  } else if max >= count {
    max = count - 1;
  }
  let offset = stored_offset.min(max);

  let mut layout = Layout {
    offset,
    max,
    ..Layout::default()
  };
  let mut used = 0u64;
  let mut last_index: Option<usize> = None;
  for (index, (block, px)) in items.iter().enumerate() {
    if index < offset {
      continue;
    }
    if !layout.visible.is_empty() && used + px > budget {
      break;
    }
    if layout.first.is_none() {
      layout.first = Some(block.clone());
    }
    layout.visible.push(block.clone());
    layout.last = Some(block.clone());
    used += px;
    last_index = Some(index);
  }
  layout.above = offset > 0;
  layout.below = match last_index {
    Some(index) => index < count - 1,
    None => count > 0,
  };
  layout
}

#[cfg(test)]
mod tests {
  use super::{block_px, layout, parse_budget, parse_offset};

  fn names(items: &[&str]) -> Vec<String> {
    items.iter().map(|item| item.to_string()).collect()
  }

  #[test]
  fn everything_is_shown_when_the_blocks_fit() {
    let all = names(&["system", "keys"]);
    let window = layout(&all, Some(10_000), 0);
    assert_eq!(window.visible, all);
    assert!(!window.below && !window.above);
    assert_eq!(window.max, 0);
  }

  #[test]
  fn the_offset_is_clamped_to_the_last_block() {
    let all = names(&["system", "cpu_gpu", "keys"]);
    let budget = block_px("cpu_gpu") + block_px("keys");
    let window = layout(&all, Some(budget), 9);
    assert_eq!(window.offset, window.max);
    assert!(window.above);
    assert!(!window.below);
  }

  #[test]
  fn budget_and_offset_inputs_are_numbers_or_defaults() {
    assert_eq!(parse_budget(Some("800")), Some(800));
    assert_eq!(parse_budget(Some("-3")), None);
    assert_eq!(parse_offset(Some("abc")), 0);
    assert_eq!(parse_offset(Some("2\n")), 2);
  }
}
