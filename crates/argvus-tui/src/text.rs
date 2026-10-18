//! Implements text measurement and truncation in crate `argvus tui`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// Number of terminal columns `text` occupies when rendered. Emoji and other
/// wide glyphs take 2 columns while some combining codepoints take 0, so this
/// is not the same as `text.chars().count()`.
pub fn display_width(text: &str) -> usize {
  UnicodeWidthStr::width(text)
}

/// Keeps the leading characters of `text` while the accumulated display width
/// stays within `width` columns. A trailing codepoint that would overflow is
/// dropped entirely. Does not append an ellipsis.
pub fn truncate_to_width(text: &str, width: usize) -> String {
  let mut out = String::new();
  let mut columns = 0usize;
  for character in text.chars() {
    let character_width = UnicodeWidthChar::width(character).unwrap_or(0);
    if columns + character_width > width {
      break;
    }
    columns += character_width;
    out.push(character);
  }
  out
}

/// Truncates `text` with a trailing "…" so the result fits within `width`
/// terminal columns (the ellipsis itself occupies one column).
pub fn ellipsize(text: &str, width: usize) -> String {
  if display_width(text) <= width {
    return text.to_string();
  }
  if width == 0 {
    return String::new();
  }
  if width == 1 {
    return "…".to_string();
  }
  let mut out = truncate_to_width(text, width - 1);
  out.push('…');
  out
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  /// Executes the `display_width_counts_visual_columns_not_chars` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn display_width_counts_visual_columns_not_chars() {
    assert_eq!(display_width("NO"), 2);
    assert_eq!(display_width("🌍"), 2);
    assert_eq!(display_width("🗂️"), 2);
    assert_eq!("🗂️".chars().count(), 2);
    assert_eq!(display_width("abc"), 3);
    assert_eq!(display_width("áudio"), 5);
  }

  #[test]
  /// Executes the `truncate_keeps_whole_codepoints_and_respects_width` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn truncate_keeps_whole_codepoints_and_respects_width() {
    assert_eq!(truncate_to_width("abcdef", 3), "abc");
    assert_eq!(truncate_to_width("abcde", 10), "abcde");
    assert_eq!(truncate_to_width("ab🌍cd", 3), "ab");
    assert_eq!(truncate_to_width("ab🌍cd", 4), "ab🌍");
    assert_eq!(truncate_to_width("ab🌍cd", 5), "ab🌍c");
  }

  #[test]
  /// Executes the `ellipsize_reserves_a_column_for_the_marker` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn ellipsize_reserves_a_column_for_the_marker() {
    assert_eq!(ellipsize("short", 10), "short");
    assert_eq!(ellipsize("abcdef", 3), "ab…");
    assert_eq!(ellipsize("🌍🌍🌍", 3), "🌍…");
    assert_eq!(ellipsize("anything", 0), "");
    assert_eq!(ellipsize("anything", 1), "…");
  }
}
