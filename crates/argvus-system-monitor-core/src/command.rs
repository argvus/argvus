//! The command line of the monitor. Only the first argument chooses a module
//! or a mode; the rest is passed to btop.

/// The boxes of the full monitor.
pub const ALL_BOXES: &str = "cpu mem net proc";

/// What the command line asks for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
  Help,
  PrintTheme,
  PrintConfig,
  /// Builds the theme file of a theme; without a name, the active one.
  Apply(Option<String>),
  /// Runs btop with the boxes of the module and the remaining arguments.
  Run {
    panel: bool,
    shown_boxes: &'static str,
    arguments: Vec<String>,
  },
}

/// Reads the command line, without the program name.
pub fn parse(arguments: &[String]) -> Command {
  let rest = |skip: usize| arguments.iter().skip(skip).cloned().collect::<Vec<_>>();
  match arguments.first().map(String::as_str) {
    Some("-h" | "--help") => Command::Help,
    Some("--print-theme") => Command::PrintTheme,
    Some("--print-config") => Command::PrintConfig,
    Some("--apply") => Command::Apply(arguments.get(1).filter(|theme| !theme.is_empty()).cloned()),
    Some("--panel") => parse_panel(arguments),
    Some("--mem") => run(false, "mem", rest(1)),
    Some("--cpu") => run(false, "cpu", rest(1)),
    Some("--net") => run(false, "net", rest(1)),
    Some("--proc") => run(false, "proc", rest(1)),
    _ => run(false, ALL_BOXES, rest(0)),
  }
}

fn parse_panel(arguments: &[String]) -> Command {
  let module = arguments.get(1).map(String::as_str);
  let (shown_boxes, argument_start) = match module {
    Some("--mem") => ("mem", 2),
    Some("--cpu") => ("cpu", 2),
    Some("--net") => ("net", 2),
    Some("--proc") => ("proc", 2),
    _ => (ALL_BOXES, 1),
  };
  run(
    true,
    shown_boxes,
    arguments.iter().skip(argument_start).cloned().collect(),
  )
}

fn run(panel: bool, shown_boxes: &'static str, arguments: Vec<String>) -> Command {
  Command::Run {
    panel,
    shown_boxes,
    arguments,
  }
}

/// Whether btop is given its own configuration or themes directory. Such a run
/// uses the paths the user gave, and the monitor writes no configuration.
pub fn has_explicit_paths(arguments: &[String]) -> bool {
  arguments.iter().any(|argument| {
    matches!(argument.as_str(), "-c" | "--config" | "--themes-dir")
      || argument.starts_with("--config=")
      || argument.starts_with("--themes-dir=")
  })
}

#[cfg(test)]
mod tests {
  use super::{ALL_BOXES, Command, has_explicit_paths, parse};

  fn words(text: &[&str]) -> Vec<String> {
    text.iter().map(|word| word.to_string()).collect()
  }

  #[test]
  fn module_flag_is_only_read_first() {
    assert_eq!(
      parse(&words(&["--mem", "--utf-force"])),
      Command::Run {
        panel: false,
        shown_boxes: "mem",
        arguments: words(&["--utf-force"]),
      }
    );
    assert_eq!(
      parse(&words(&["--utf-force", "--mem"])),
      Command::Run {
        panel: false,
        shown_boxes: ALL_BOXES,
        arguments: words(&["--utf-force", "--mem"]),
      }
    );
  }

  #[test]
  fn modes_are_recognised() {
    assert_eq!(parse(&words(&["--print-theme"])), Command::PrintTheme);
    assert_eq!(parse(&words(&["--apply"])), Command::Apply(None));
    assert_eq!(
      parse(&words(&["--apply", "nord"])),
      Command::Apply(Some("nord".into()))
    );
    assert_eq!(parse(&words(&["-h"])), Command::Help);
  }

  #[test]
  fn explicit_btop_paths_are_detected() {
    assert!(has_explicit_paths(&words(&["--config=/tmp/x.conf"])));
    assert!(has_explicit_paths(&words(&["-c", "x"])));
    assert!(!has_explicit_paths(&words(&["--utf-force"])));
  }

  #[test]
  fn panel_mode_reads_module_before_forwarding_btop_arguments() {
    assert_eq!(
      parse(&words(&["--panel", "--cpu", "--utf-force"])),
      Command::Run {
        panel: true,
        shown_boxes: "cpu",
        arguments: words(&["--utf-force"]),
      }
    );
    assert_eq!(
      parse(&words(&["--panel", "--mem"])),
      Command::Run {
        panel: true,
        shown_boxes: "mem",
        arguments: words(&[]),
      }
    );
  }
}
