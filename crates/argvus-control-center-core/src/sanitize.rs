//! Safe normalization of external output for the TUI.
#[derive(Clone, Copy)]
enum State {
  Ground,
  Escape,
  EscapeIntermediate,
  Csi,
  StringControl { osc: bool },
  StringEscape { osc: bool },
}

/// Removes terminal controls and spoofing format characters, retaining text layout.
/// Unterminated OSC strings recover at newline; other control strings require ST.
pub fn terminal_text(input: &str) -> String {
  let mut output = String::with_capacity(input.len());
  let mut state = State::Ground;
  for character in input.chars() {
    if character == '\n' {
      if matches!(
        state,
        State::StringControl { osc: false } | State::StringEscape { osc: false }
      ) {
        continue;
      }
      output.push(character);
      state = State::Ground;
      continue;
    }
    if character == '\u{1b}' {
      state = match state {
        State::StringControl { osc } | State::StringEscape { osc } => State::StringEscape { osc },
        _ => State::Escape,
      };
      continue;
    }
    state = match state {
      State::Ground => match character {
        '\u{9b}' => State::Csi,
        '\u{9d}' => State::StringControl { osc: true },
        '\u{90}' | '\u{98}' | '\u{9e}' | '\u{9f}' => State::StringControl { osc: false },
        _ => {
          push_visible(&mut output, character);
          State::Ground
        }
      },
      State::Escape => match character {
        '[' => State::Csi,
        ']' => State::StringControl { osc: true },
        'P' | 'X' | '^' | '_' => State::StringControl { osc: false },
        '\u{20}'..='\u{2f}' => State::EscapeIntermediate,
        _ => State::Ground,
      },
      State::EscapeIntermediate => {
        if ('\u{30}'..='\u{7e}').contains(&character) {
          State::Ground
        } else {
          State::EscapeIntermediate
        }
      }
      State::Csi => {
        if ('\u{40}'..='\u{7e}').contains(&character) {
          State::Ground
        } else if ('\u{20}'..='\u{3f}').contains(&character) {
          State::Csi
        } else {
          push_visible(&mut output, character);
          State::Ground
        }
      }
      State::StringControl { osc } => {
        if character == '\u{9c}' || (osc && character == '\u{7}') {
          State::Ground
        } else {
          State::StringControl { osc }
        }
      }
      State::StringEscape { osc } => {
        if character == '\\' || character == '\u{9c}' || (osc && character == '\u{7}') {
          State::Ground
        } else {
          State::StringControl { osc }
        }
      }
    };
  }
  output
}

fn push_visible(output: &mut String, character: char) {
  let spoofing_format = matches!(character, '\u{061c}' | '\u{200b}'..='\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2060}' | '\u{2066}'..='\u{206f}' | '\u{feff}');
  if character == '\t' || (!character.is_control() && !spoofing_format) {
    output.push(character);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn removes_ansi_and_control_characters_but_keeps_layout() {
    assert_eq!(
      terminal_text("ok\u{1b}[31m red\u{1b}[0m\nnext\tline\u{7}"),
      "ok red\nnext\tline"
    );
  }
  #[test]
  fn csi_accepts_all_ascii_final_bytes() {
    for final_byte in ['~', '@', '`', '}', 'm'] {
      assert_eq!(
        terminal_text(&format!("before\x1b[1;2{final_byte}after")),
        "beforeafter"
      );
    }
  }

  #[test]
  fn string_controls_hide_payloads() {
    assert_eq!(
      terminal_text("before\x1bPmulti\nline payload\x1b\\after"),
      "beforeafter"
    );
    for introducer in [']', 'P', '_', '^', 'X'] {
      assert_eq!(
        terminal_text(&format!("before\x1b{introducer}hidden payload\x1b\\after")),
        "beforeafter"
      );
    }
  }

  #[test]
  fn unterminated_control_strings_recover_at_newline() {
    assert_eq!(
      terminal_text("before\x1b]malformed title\nvisible next line"),
      "before\nvisible next line"
    );
  }

  #[test]
  fn removes_bidi_and_invisible_format_controls() {
    assert_eq!(
      terminal_text("pa\u{202e}ck\u{2066}age\u{2069}\u{200b}\u{200f}\u{feff}"),
      "package"
    );
  }

  #[test]
  fn c1_controls_and_character_set_escapes_are_removed() {
    assert_eq!(
      terminal_text("a\u{9b}1~b\u{9d}hidden\u{9c}c\x1b(Bd"),
      "abcd"
    );
    for control in ['\u{90}', '\u{98}', '\u{9e}', '\u{9f}'] {
      assert_eq!(terminal_text(&format!("a{control}hidden\u{9c}b")), "ab");
    }
  }

  #[test]
  fn preserves_unicode_text_and_layout_and_is_idempotent() {
    let input = "módulo\t日本語\nالعربية\x1b[31m color\x1b[0m";
    let clean = terminal_text(input);
    assert_eq!(clean, "módulo\t日本語\nالعربية color");
    assert_eq!(terminal_text(&clean), clean);
    assert_eq!(terminal_text("visible\x1b]unterminated"), "visible");
  }
}
