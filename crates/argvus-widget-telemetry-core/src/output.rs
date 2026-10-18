//! The text the Waybar modules print: the JSON of a custom module, and the
//! progress bar of the usage blocks. The JSON escapes what the shell helpers
//! left unescaped, so a value with a quote cannot break the module.

/// The width, in cells, of the progress bars.
pub const BAR_SIZE: u32 = 8;

/// `{"text":"..."}` for a multi-line text. The text is printed with its line
/// breaks written as `\n`; trailing line breaks are dropped.
pub fn json_text(text: &str) -> String {
  let mut escaped = String::with_capacity(text.len());
  for character in text.trim_end_matches('\n').chars() {
    match character {
      '"' => escaped.push_str("\\\""),
      '\\' => escaped.push_str("\\\\"),
      '\n' => escaped.push_str("\\n"),
      other => escaped.push(other),
    }
  }
  format!("{{\"text\":\"{escaped}\"}}")
}

/// A bar of `BAR_SIZE` cells filled for `percent`, clamped to 0..=100.
pub fn bar(percent: u64) -> String {
  let filled = (percent.min(100) * u64::from(BAR_SIZE) / 100) as usize;
  let empty = BAR_SIZE as usize - filled;
  format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}

#[cfg(test)]
mod tests {
  use super::{bar, json_text};

  #[test]
  fn json_keeps_the_line_breaks_as_escapes() {
    assert_eq!(json_text("a\nb\n"), "{\"text\":\"a\\nb\"}");
    assert_eq!(json_text(""), "{\"text\":\"\"}");
  }

  #[test]
  fn json_escapes_quotes_and_backslashes() {
    assert_eq!(
      json_text("say \"hi\" \\"),
      "{\"text\":\"say \\\"hi\\\" \\\\\"}"
    );
  }

  #[test]
  fn bars_fill_by_percent() {
    assert_eq!(bar(50), "████░░░░");
    assert_eq!(bar(0), "░░░░░░░░");
    assert_eq!(bar(100), "████████");
    assert_eq!(bar(250), "████████");
  }
}
