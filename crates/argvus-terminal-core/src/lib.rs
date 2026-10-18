//! Rules of the ARGVUS terminal launcher: the kitty configuration it generates,
//! the transparency and the accent it applies, the fonts of each profile, and
//! the command line. Nothing here reads files or starts processes.

pub mod accent;
pub mod command;
pub mod font;
pub mod kitty;
pub mod profile;
pub mod transparency;

/// Whitespace as the shell's `[[:space:]]` class names it.
pub fn is_blank(character: char) -> bool {
  matches!(character, ' ' | '\t' | '\r' | '\u{b}' | '\u{c}')
}
