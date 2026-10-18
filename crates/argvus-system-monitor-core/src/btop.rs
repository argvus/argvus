//! The btop configuration the monitor generates from the ARGVUS settings: the
//! colour theme and the boxes that are shown.

/// The keys of the btop configuration that the monitor sets.
const THEME_KEY: &str = "color_theme";
const BOXES_KEY: &str = "shown_boxes";

/// The btop configuration with the theme and the boxes set. A key that the
/// configuration lacks is appended.
pub fn render_config(source: &str, theme: &str, shown_boxes: &str) -> String {
  let mut output = String::with_capacity(source.len() + 64);
  for line in source.lines() {
    if is_setting(line, THEME_KEY) {
      output.push_str(&format!("{THEME_KEY} = \"{theme}\""));
    } else if is_setting(line, BOXES_KEY) {
      output.push_str(&format!("{BOXES_KEY} = \"{shown_boxes}\""));
    } else {
      output.push_str(line);
    }
    output.push('\n');
  }
  if !has_setting(&output, THEME_KEY) {
    output.push_str(&format!("\n{THEME_KEY} = \"{theme}\"\n"));
  }
  if !has_setting(&output, BOXES_KEY) {
    output.push_str(&format!("{BOXES_KEY} = \"{shown_boxes}\"\n"));
  }
  output
}

/// Whether a line assigns `key`, after optional blanks.
fn is_setting(line: &str, key: &str) -> bool {
  let trimmed = line.trim_start_matches([' ', '\t']);
  trimmed
    .strip_prefix(key)
    .map(|rest| rest.trim_start_matches([' ', '\t']))
    .is_some_and(|rest| rest.starts_with('='))
}

fn has_setting(text: &str, key: &str) -> bool {
  text.lines().any(|line| is_setting(line, key))
}

#[cfg(test)]
mod tests {
  use super::render_config;

  #[test]
  fn existing_settings_are_replaced() {
    let source = "# comment\n  color_theme = \"old\"\nshown_boxes = \"cpu\"\n";
    assert_eq!(
      render_config(source, "argvus-dark", "mem"),
      "# comment\ncolor_theme = \"argvus-dark\"\nshown_boxes = \"mem\"\n"
    );
  }

  #[test]
  fn missing_settings_are_appended() {
    assert_eq!(
      render_config("update_ms = 2000\n", "argvus-light", "cpu mem"),
      "update_ms = 2000\n\ncolor_theme = \"argvus-light\"\nshown_boxes = \"cpu mem\"\n"
    );
  }
}
