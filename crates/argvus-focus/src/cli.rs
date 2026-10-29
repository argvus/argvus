//! Command-line options, parsed by the primary instance for every launch.

use std::ffi::OsString;

use argvus_taskbar_popup::PopupAnchor;
use clap::Parser;

#[derive(Debug, Parser, PartialEq, Eq)]
#[command(
  name = "argvus focus",
  about = "ARGVUS Focus: Pomodoro and focus sessions",
  version
)]
pub struct Cli {
  /// Hide the window if it is visible, show it otherwise.
  #[arg(long)]
  pub toggle: bool,
  /// Start or resume the current phase.
  #[arg(long, conflicts_with = "pause")]
  pub start: bool,
  /// Pause the running phase.
  #[arg(long)]
  pub pause: bool,
  /// Click position (desktop coordinates) passed by the taskbar, so the popup
  /// opens next to the taskbar icon.
  #[arg(long, requires = "y", allow_hyphen_values = true)]
  pub x: Option<i32>,
  #[arg(long, requires = "x", allow_hyphen_values = true)]
  pub y: Option<i32>,
  /// Bottom edge of the bar that was clicked.
  #[arg(long, requires = "x", allow_hyphen_values = true)]
  pub bar_bottom: Option<i32>,
  /// Right edge of the bar that was clicked.
  #[arg(long, requires = "bar_bottom", allow_hyphen_values = true)]
  pub bar_right: Option<i32>,
}

impl Cli {
  /// Where the popup opens, when the taskbar passed the click position.
  pub fn anchor(&self) -> Option<PopupAnchor> {
    Some(PopupAnchor {
      x: self.x?,
      y: self.y?,
      bar_bottom: self.bar_bottom,
      bar_right: self.bar_right,
    })
  }

  pub fn parse_arguments(arguments: Vec<OsString>) -> Result<Self, clap::Error> {
    Self::try_parse_from(arguments)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn parse(arguments: &[&str]) -> Result<Cli, clap::Error> {
    Cli::parse_arguments(
      std::iter::once("argvus-focus")
        .chain(arguments.iter().copied())
        .map(OsString::from)
        .collect(),
    )
  }

  #[test]
  fn no_arguments_just_show_the_window() {
    assert_eq!(
      parse(&[]).unwrap(),
      Cli {
        toggle: false,
        start: false,
        pause: false,
        x: None,
        y: None,
        bar_bottom: None,
        bar_right: None,
      }
    );
  }

  #[test]
  fn start_and_pause_conflict() {
    assert!(parse(&["--start", "--pause"]).is_err());
    assert!(parse(&["--toggle", "--start"]).unwrap().start);
  }

  #[test]
  fn taskbar_click_position_becomes_an_anchor() {
    let cli = parse(&[
      "--toggle",
      "--x",
      "1700",
      "--y",
      "-4",
      "--bar-bottom",
      "46",
      "--bar-right",
      "1902",
    ])
    .unwrap();
    assert_eq!(
      cli.anchor(),
      Some(PopupAnchor {
        x: 1700,
        y: -4,
        bar_bottom: Some(46),
        bar_right: Some(1902)
      })
    );
    assert_eq!(parse(&["--toggle"]).unwrap().anchor(), None);
    assert!(parse(&["--x", "10"]).is_err(), "x needs y");
    assert!(parse(&["--x", "1", "--y", "2", "--bar-right", "9"]).is_err());
  }

  #[test]
  fn unknown_options_are_rejected() {
    assert!(parse(&["--bogus"]).is_err());
  }
}
