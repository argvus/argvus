//! Structured edits of ARGVUS CSS: a declaration inside a named block.
//!
//! The theme scripts used `sed` ranges such as
//! `/^window#waybar {/,/^}/s/border-radius: [0-9]*px;/border-radius: 0px;/`.
//! Each one is a block (from a selector line to the next line that starts with
//! `}`) and a single declaration rewritten inside it. This module names those
//! two operations and implements them with the regex engine, so the patterns
//! are written once and tested.

use regex::{NoExpand, Regex};
use std::sync::OnceLock;

/// A rounding declaration. Matches the value only when it is whole pixels, as
/// the scripts always did.
fn radius_pattern() -> &'static Regex {
  static PATTERN: OnceLock<Regex> = OnceLock::new();
  PATTERN.get_or_init(|| Regex::new(r"border-radius: [0-9]*px;").expect("valid pattern"))
}

/// A border declaration: everything up to the last semicolon on the line.
fn border_pattern() -> &'static Regex {
  static PATTERN: OnceLock<Regex> = OnceLock::new();
  PATTERN.get_or_init(|| Regex::new(r"border: .*;").expect("valid pattern"))
}

/// Which declaration a block edit rewrites.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Declaration {
  /// `border-radius: <n>px;` becomes `border-radius: <value>;`.
  Radius,
  /// `border: <anything>;` becomes `border: <value>;`.
  Border,
}

impl Declaration {
  /// Parses the name used on the command line.
  pub fn parse(name: &str) -> Option<Self> {
    match name {
      "radius" => Some(Self::Radius),
      "border" => Some(Self::Border),
      _ => None,
    }
  }

  fn pattern(self) -> &'static Regex {
    match self {
      Self::Radius => radius_pattern(),
      Self::Border => border_pattern(),
    }
  }

  fn property(self) -> &'static str {
    match self {
      Self::Radius => "border-radius",
      Self::Border => "border",
    }
  }
}

/// Rewrites `declaration` on every line of every block whose first line starts
/// with `block_start`, and leaves the rest of the text as it was.
///
/// A block starts at a line that starts with `block_start` and ends at the next
/// line that starts with `}`, which is included. The declaration is replaced
/// once per line, as `sed`'s `s///` without `g` did.
pub fn set_declaration_in_blocks(
  text: &str,
  block_start: &str,
  declaration: Declaration,
  value: &str,
) -> String {
  let pattern = declaration.pattern();
  let replacement = format!("{}: {value};", declaration.property());
  let mut inside = false;
  let mut output = String::with_capacity(text.len());
  for line in text.split_inclusive('\n') {
    let (content, ending) = match line.strip_suffix('\n') {
      Some(content) => (content, "\n"),
      None => (line, ""),
    };
    let starts_block = !inside && content.starts_with(block_start);
    if starts_block {
      inside = true;
    }
    let edited = if inside {
      pattern
        .replace(content, NoExpand(&replacement))
        .into_owned()
    } else {
      content.to_string()
    };
    // The end line belongs to the block, so it is edited before closing it.
    if inside && !starts_block && content.starts_with('}') {
      inside = false;
    }
    output.push_str(&edited);
    output.push_str(ending);
  }
  output
}

#[cfg(test)]
mod tests {
  use super::*;

  const CSS: &str = "window#waybar {\n  border: 1px solid red;\n  border-radius: 6px;\n}\n\nwindow#other {\n  border-radius: 9px;\n}\n";

  #[test]
  fn radius_changes_only_inside_the_named_block() {
    let edited = set_declaration_in_blocks(CSS, "window#waybar {", Declaration::Radius, "0px");
    assert!(edited.contains("  border-radius: 0px;\n}\n"));
    assert!(
      edited.contains("window#other {\n  border-radius: 9px;\n}"),
      "other block untouched"
    );
  }

  #[test]
  fn border_replaces_the_whole_declaration_value() {
    let edited = set_declaration_in_blocks(CSS, "window#waybar {", Declaration::Border, "none");
    assert!(edited.contains("  border: none;\n"));
    assert!(
      edited.contains("  border-radius: 6px;"),
      "other declarations untouched"
    );
  }

  #[test]
  fn radius_is_replaced_only_when_it_is_whole_pixels() {
    let text = "a {\n  border-radius: 1.5rem;\n}\n";
    assert_eq!(
      set_declaration_in_blocks(text, "a {", Declaration::Radius, "0px"),
      text
    );
  }

  #[test]
  fn missing_block_and_missing_final_newline_are_preserved() {
    assert_eq!(
      set_declaration_in_blocks(CSS, "nothing {", Declaration::Radius, "0px"),
      CSS
    );
    let text = "a {\n  border-radius: 2px;\n}";
    assert_eq!(
      set_declaration_in_blocks(text, "a {", Declaration::Radius, "0px"),
      "a {\n  border-radius: 0px;\n}"
    );
  }

  #[test]
  fn declaration_names_parse_from_the_command_line() {
    assert_eq!(Declaration::parse("radius"), Some(Declaration::Radius));
    assert_eq!(Declaration::parse("border"), Some(Declaration::Border));
    assert_eq!(Declaration::parse("margin"), None);
  }
}
