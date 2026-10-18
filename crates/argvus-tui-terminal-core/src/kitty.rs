//! The Kitty configuration the launcher writes: the source configuration with
//! its font lines replaced, and the runtime policy of the greeter and of the
//! Control Center appended to it.

/// Replaces the `font_family` and `font_size` lines of a Kitty configuration.
/// Every other line is kept as it is, and a line keeps the keyword followed by
/// blanks only.
pub fn rewrite_fonts(source: &str, family: &str, size: &str) -> String {
  let mut output = String::new();
  for line in source.lines() {
    let trimmed = line.trim_start();
    if let Some(rest) = keyword(trimmed, "font_family") {
      let _ = rest;
      output.push_str(&format!("font_family      {family}\n"));
    } else if keyword(trimmed, "font_size").is_some() {
      output.push_str(&format!("font_size        {size}\n"));
    } else {
      output.push_str(line);
      output.push('\n');
    }
  }
  output
}

/// The text after a keyword that is followed by at least one blank, or `None`.
fn keyword<'a>(line: &'a str, name: &str) -> Option<&'a str> {
  let rest = line.strip_prefix(name)?;
  rest.starts_with([' ', '\t']).then_some(rest)
}

/// The policy appended for the greeter: tabs hidden and tab keys disabled.
pub fn greeter_policy() -> &'static str {
  "\n# ARGVUS Greeter policy: this is a runtime-only override. The regular Kitty\n# configuration and all user terminal sessions keep their tab behavior.\ntab_bar_style hidden\nmap ctrl+alt+n no_op\nmap ctrl+shift+t no_op\nmap ctrl+q no_op\nmap alt+1 no_op\nmap alt+2 no_op\nmap alt+3 no_op\nmap alt+4 no_op\nmap alt+5 no_op\nmap ctrl+tab no_op\nmap ctrl+shift+tab no_op\n"
}

/// The policy appended for the Control Center: tab-less window with the given
/// background opacity, which wins because it comes last.
pub fn control_center_policy(opacity: &str) -> String {
  format!(
    "\n# ARGVUS Control Center policy: this is a runtime-only override. The\n# regular Kitty configuration and all user terminal sessions keep their tab\n# behavior and transparency.\nbackground_opacity {opacity}\ntab_bar_style hidden\nmap ctrl+alt+n no_op\nmap ctrl+shift+t no_op\nmap ctrl+q no_op\nmap alt+1 no_op\nmap alt+2 no_op\nmap alt+3 no_op\nmap alt+4 no_op\nmap alt+5 no_op\nmap ctrl+tab no_op\nmap ctrl+shift+tab no_op\n"
  )
}

#[cfg(test)]
mod tests {
  use super::{control_center_policy, greeter_policy, rewrite_fonts};

  #[test]
  fn only_the_font_lines_are_replaced() {
    let source = "font_family  Old\n  font_size 9\nfont_familyX keep\nbackground black\n";
    assert_eq!(
      rewrite_fonts(source, "New", "12"),
      "font_family      New\nfont_size        12\nfont_familyX keep\nbackground black\n"
    );
  }

  #[test]
  fn policies_end_with_the_tab_key_bindings() {
    assert!(greeter_policy().contains("tab_bar_style hidden\n"));
    assert!(control_center_policy("0.50").contains("background_opacity 0.50\n"));
  }
}
