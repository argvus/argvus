//! Text helpers for the menu and the Waybar output: byte sizes and trimming.

pub fn human_bytes(bytes: u64) -> String {
  const UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];
  let mut value = bytes as f64;
  let mut unit = 0;
  while value >= 1000.0 && unit < 5 {
    value /= 1000.0;
    unit += 1;
  }
  if unit == 0 {
    format!("{} B", bytes)
  } else {
    format!("{:.1} {}", value, UNITS[unit])
  }
}

/// Trims spaces, tabs and line breaks only. Unlike `str::trim`, other Unicode
/// whitespace is kept, so a menu selection is compared exactly as the menu
/// command printed it.
pub fn trim_line_whitespace(text: &str) -> &str {
  text.trim_matches(|character| matches!(character, ' ' | '\t' | '\r' | '\n'))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn human_bytes_values() {
    assert_eq!(human_bytes(0), "0 B");
    assert_eq!(human_bytes(512), "512 B");
    assert_eq!(human_bytes(1024), "1.0 KB");
    assert_eq!(human_bytes(512 * 1024 * 1024), "536.9 MB");
  }

  #[test]
  fn trim_whitespace() {
    assert_eq!(trim_line_whitespace("  foo \n"), "foo");
    assert_eq!(trim_line_whitespace("\t\r\n"), "");
  }
}
