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

/// A `border` or `border-bottom` declaration with a whole-pixel width, as in
/// `border: 1px solid @th-decorate;`. The width is the only part replaced.
fn border_width_pattern() -> &'static Regex {
  static PATTERN: OnceLock<Regex> = OnceLock::new();
  PATTERN
    .get_or_init(|| Regex::new(r"(border(?:-bottom)?: )[0-9]+px solid ").expect("valid pattern"))
}

/// A `border` declaration set to `none`, which has no width or colour of its own.
fn border_none_pattern() -> &'static Regex {
  static PATTERN: OnceLock<Regex> = OnceLock::new();
  PATTERN.get_or_init(|| Regex::new(r"(border(?:-bottom)?: )none;").expect("valid pattern"))
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
  edit_block_lines(text, block_start, false, |content| {
    pattern
      .replace(content, NoExpand(&replacement))
      .into_owned()
  })
}

/// Sets the border width of every `border` and `border-bottom` declaration in
/// the named blocks, keeping each colour. A `none` border takes `@th-decorate`,
/// the accent colour of the taskbar window, as its colour. A width of `0` turns
/// the border off.
pub fn set_border_width_in_blocks(text: &str, block_start: &str, width: u32) -> String {
  let solid = format!("${{1}}{width}px solid ");
  let none = format!("${{1}}{width}px solid @th-decorate;");
  edit_block_lines(text, block_start, true, |content| {
    let edited = border_width_pattern().replace_all(content, solid.as_str());
    border_none_pattern()
      .replace_all(&edited, none.as_str())
      .into_owned()
  })
}

/// The border width of the first `border` declaration in the named block: the
/// pixel count of `border: <n>px solid …`, or `0` for `border: none`. `None`
/// when the block has no such declaration.
pub fn border_width_in_block(text: &str, block_start: &str) -> Option<u32> {
  let width = Regex::new(r"^\s*border: ([0-9]+)px solid ").expect("valid pattern");
  let none = Regex::new(r"^\s*border: none;").expect("valid pattern");
  let mut inside = false;
  for line in text.lines() {
    let starts_block = !inside && line.starts_with(block_start);
    if starts_block {
      inside = true;
    }
    if inside {
      if let Some(captures) = width.captures(line) {
        return captures[1].parse().ok();
      }
      if none.is_match(line) {
        return Some(0);
      }
    }
    if inside && !starts_block && line.starts_with('}') {
      inside = false;
    }
  }
  None
}

/// Applies `edit` to every line of every block whose first line starts with
/// `block_start`, and leaves the rest of the text as it was. A block ends at the
/// next line that starts with `}`, which is included in the edit.
///
/// With `rule_start`, a match counts only when the line before it is blank, a
/// closing `}` or the start of the text. A selector such as `#mpris,` also
/// appears inside longer selector lists, and only the rule's own line opens a
/// block.
fn edit_block_lines(
  text: &str,
  block_start: &str,
  rule_start: bool,
  edit: impl Fn(&str) -> String,
) -> String {
  let mut inside = false;
  let mut previous = "";
  let mut output = String::with_capacity(text.len());
  for line in text.split_inclusive('\n') {
    let (content, ending) = match line.strip_suffix('\n') {
      Some(content) => (content, "\n"),
      None => (line, ""),
    };
    let after_break = !rule_start || previous.trim().is_empty() || previous.ends_with('}');
    let starts_block = !inside && after_break && content.starts_with(block_start);
    previous = content;
    if starts_block {
      inside = true;
    }
    let edited = if inside {
      edit(content)
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

  const TASKBAR: &str = "window#waybar {\n  border: 1px solid @th-decorate;\n}\n\n#mpris,\n#custom-spotify-mpris {\n  border: 1px solid @th-mpris-border;\n}\n\n#mpris:hover,\n#custom-spotify-mpris:hover {\n  border-bottom: 1px solid @th-decorate;\n}\n";

  #[test]
  fn border_width_keeps_each_colour() {
    let edited = set_border_width_in_blocks(TASKBAR, "window#waybar {", 3);
    assert!(edited.contains("  border: 3px solid @th-decorate;\n"));
    let edited = set_border_width_in_blocks(TASKBAR, "#mpris,", 2);
    assert!(edited.contains("  border: 2px solid @th-mpris-border;\n"));
    let edited = set_border_width_in_blocks(TASKBAR, "#mpris:hover,", 0);
    assert!(edited.contains("  border-bottom: 0px solid @th-decorate;\n"));
  }

  #[test]
  fn a_selector_inside_a_longer_list_does_not_open_a_block() {
    let text = "#language,\n#mpris,\n#cpu {\n  border-bottom: 1px solid @th-border-bottom;\n}\n\n#mpris,\n#custom-spotify-mpris {\n  border: 1px solid @th-mpris-border;\n}\n";
    let edited = set_border_width_in_blocks(text, "#mpris,", 3);
    assert!(edited.contains("  border-bottom: 1px solid @th-border-bottom;\n"));
    assert!(edited.contains("  border: 3px solid @th-mpris-border;\n"));
  }

  #[test]
  fn border_width_replaces_none_with_the_accent_colour() {
    let text = "window#waybar {\n  border: none;\n}\n";
    assert_eq!(
      set_border_width_in_blocks(text, "window#waybar {", 1),
      "window#waybar {\n  border: 1px solid @th-decorate;\n}\n"
    );
  }

  #[test]
  fn border_width_is_read_from_the_taskbar_window() {
    assert_eq!(border_width_in_block(TASKBAR, "window#waybar {"), Some(1));
    let off = set_border_width_in_blocks(TASKBAR, "window#waybar {", 0);
    assert_eq!(border_width_in_block(&off, "window#waybar {"), Some(0));
    let none = "window#waybar {\n  border: none;\n}\n";
    assert_eq!(border_width_in_block(none, "window#waybar {"), Some(0));
    assert_eq!(border_width_in_block(TASKBAR, "nothing {"), None);
  }
}
