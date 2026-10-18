//! The report of the scroll window, as the toggle prints it with `layout`: one
//! `key=value` line per field. The block commands read it to decide what to show.

/// The window as the report describes it.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Report {
  /// The blocks inside the window, in display order.
  pub visible: Vec<String>,
  pub first: Option<String>,
  pub last: Option<String>,
  /// Content is scrolled out of view above the window.
  pub above: bool,
  /// Content is scrolled out of view below the window.
  pub below: bool,
}

/// Reads the report text. A missing field stays empty, so a toggle that failed
/// gives an empty window.
pub fn parse(text: &str) -> Report {
  let mut report = Report::default();
  for line in text.lines() {
    let Some((key, value)) = line.split_once('=') else {
      continue;
    };
    match key {
      "visible" => report.visible = value.split_whitespace().map(String::from).collect(),
      "first" => report.first = non_empty(value),
      "last" => report.last = non_empty(value),
      "above" => report.above = value == "1",
      "below" => report.below = value == "1",
      _ => {}
    }
  }
  report
}

fn non_empty(value: &str) -> Option<String> {
  if value.is_empty() {
    None
  } else {
    Some(value.to_string())
  }
}

#[cfg(test)]
mod tests {
  use super::parse;

  #[test]
  fn reads_the_window_fields() {
    let report =
      parse("visible=memory keys\nfirst=memory\nlast=keys\noffset=2\nmax=4\nabove=1\nbelow=0\n");
    assert_eq!(report.visible, ["memory", "keys"]);
    assert_eq!(report.first.as_deref(), Some("memory"));
    assert_eq!(report.last.as_deref(), Some("keys"));
    assert!(report.above);
    assert!(!report.below);
  }

  #[test]
  fn empty_report_hides_everything() {
    let report = parse("");
    assert!(report.visible.is_empty());
    assert_eq!(report.first, None);
    assert!(!report.above);
  }
}
