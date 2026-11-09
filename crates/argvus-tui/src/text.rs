//! Text measurement and truncation.
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

/// Number of terminal columns `text` occupies when rendered. Emoji and other
/// wide glyphs take 2 columns while some combining codepoints take 0, so this
/// is not the same as `text.chars().count()`.
pub fn display_width(text: &str) -> usize {
  UnicodeWidthStr::width(text)
}

/// Pads by terminal cells rather than Unicode scalar count.
pub fn pad_to_width(text: &str, width: usize) -> String {
  format!(
    "{text}{}",
    " ".repeat(width.saturating_sub(display_width(text)))
  )
}

/// Fits a table cell to exactly `width` terminal columns, with safe truncation.
pub fn fit_to_width(text: &str, width: usize) -> String {
  pad_to_width(&ellipsize(text, width), width)
}

/// Hard wraps complete graphemes, preserving explicit newlines. A grapheme
/// wider than the entire area is represented by a replacement character.
pub fn wrap_to_width(text: &str, width: usize) -> Vec<String> {
  let width = width.max(1);
  let mut lines = Vec::new();
  let mut current = String::new();
  let mut columns = 0;
  for grapheme in text.graphemes(true) {
    if matches!(grapheme, "\n" | "\r\n") {
      lines.push(std::mem::take(&mut current));
      columns = 0;
      continue;
    }
    let grapheme = if display_width(grapheme) > width {
      "�"
    } else {
      grapheme
    };
    let next_width = display_width(grapheme);
    if columns + next_width > width {
      lines.push(std::mem::take(&mut current));
      columns = 0;
    }
    current.push_str(grapheme);
    columns += next_width;
  }
  lines.push(current);
  lines
}

/// Previous extended grapheme boundary, expressed as an index into `Vec<char>`.
pub fn previous_grapheme_boundary(characters: &[char], cursor: usize) -> usize {
  let text: String = characters.iter().collect();
  let mut position = 0;
  for grapheme in text.graphemes(true) {
    let next = position + grapheme.chars().count();
    if next >= cursor {
      return position;
    }
    position = next;
  }
  position
}

/// Next extended grapheme boundary, expressed as an index into `Vec<char>`.
pub fn next_grapheme_boundary(characters: &[char], cursor: usize) -> usize {
  let text: String = characters.iter().collect();
  let mut position = 0;
  for grapheme in text.graphemes(true) {
    position += grapheme.chars().count();
    if position > cursor {
      return position;
    }
  }
  position
}

/// Character index at or before a visual column, without splitting a grapheme.
pub fn character_index_at_width(characters: &[char], column: usize) -> usize {
  let text: String = characters.iter().collect();
  let mut position = 0;
  let mut width = 0;
  for grapheme in text.graphemes(true) {
    width += display_width(grapheme);
    if width > column {
      break;
    }
    position += grapheme.chars().count();
  }
  position
}

/// Keeps whole leading graphemes within `width` terminal columns.
/// Does not append an ellipsis.
pub fn truncate_to_width(text: &str, width: usize) -> String {
  let mut out = String::new();
  let mut columns = 0usize;
  for grapheme in text.graphemes(true) {
    let grapheme_width = display_width(grapheme);
    if columns + grapheme_width > width {
      break;
    }
    columns += grapheme_width;
    out.push_str(grapheme);
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
  fn wraps_multilingual_text_in_cells_without_splitting_graphemes() {
    for text in [
      "日本語中文한국어",
      "Русский язык",
      "Français e\u{301}",
      "👩‍💻👩‍💻",
    ] {
      for width in 2..12 {
        let lines = wrap_to_width(text, width);
        assert!(lines.iter().all(|line| display_width(line) <= width));
        assert_eq!(lines.concat(), text);
      }
    }
    assert_eq!(wrap_to_width("a\nb", 2), ["a", "b"]);
    assert_eq!(wrap_to_width("日", 1), ["�"]);
    assert_eq!(pad_to_width("日本", 6), "日本  ");
  }

  #[test]
  fn display_width_counts_visual_columns_not_chars() {
    assert_eq!(display_width("NO"), 2);
    assert_eq!(display_width("🌍"), 2);
    assert_eq!(display_width("🗂️"), 2);
    assert_eq!("🗂️".chars().count(), 2);
    assert_eq!(display_width("abc"), 3);
    assert_eq!(display_width("áudio"), 5);
  }

  #[test]
  fn truncate_keeps_whole_codepoints_and_respects_width() {
    assert_eq!(truncate_to_width("abcdef", 3), "abc");
    assert_eq!(truncate_to_width("abcde", 10), "abcde");
    assert_eq!(truncate_to_width("ab🌍cd", 3), "ab");
    assert_eq!(truncate_to_width("ab🌍cd", 4), "ab🌍");
    assert_eq!(truncate_to_width("ab🌍cd", 5), "ab🌍c");
    assert_eq!(truncate_to_width("日本語", 3), "日");
    assert_eq!(truncate_to_width("e\u{301}x", 1), "e\u{301}");
    assert_eq!(truncate_to_width("👩‍💻x", 2), "👩‍💻");
  }

  #[test]
  fn ellipsize_reserves_a_column_for_the_marker() {
    assert_eq!(ellipsize("short", 10), "short");
    assert_eq!(ellipsize("abcdef", 3), "ab…");
    assert_eq!(ellipsize("🌍🌍🌍", 3), "🌍…");
    assert_eq!(ellipsize("anything", 0), "");
    assert_eq!(ellipsize("anything", 1), "…");
  }
}
