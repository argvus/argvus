//! Implements safe normalization of external output for the TUI in crate `argvus control center core`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
pub fn terminal_text(input: &str) -> String {
  let mut output = String::new();
  let mut chars = input.chars().peekable();
  while let Some(character) = chars.next() {
    if character == '\u{1b}' {
      if chars.peek() == Some(&'[') {
        chars.next();
        for next in chars.by_ref() {
          if next.is_ascii_alphabetic() {
            break;
          }
        }
      } else if chars.peek() == Some(&']') {
        chars.next();
        while let Some(next) = chars.next() {
          if next == '\u{7}' {
            break;
          }
          if next == '\u{1b}' && chars.next() == Some('\\') {
            break;
          }
        }
      }
      continue;
    }
    if character == '\n' || character == '\t' || !character.is_control() {
      output.push(character);
    }
  }
  output
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  /// Executes the `removes_ansi_and_control_characters_but_keeps_layout` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn removes_ansi_and_control_characters_but_keeps_layout() {
    assert_eq!(
      terminal_text("ok\u{1b}[31m red\u{1b}[0m\nnext\tline\u{7}"),
      "ok red\nnext\tline"
    );
  }
}
