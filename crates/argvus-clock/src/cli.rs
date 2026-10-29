//! Command-line options, parsed by the primary instance for every launch.

use std::ffi::OsString;

use argvus_taskbar_popup::PopupAnchor;
use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum PageArg {
  Clock,
  World,
  Stopwatch,
  Timer,
  Alarms,
}

#[derive(Debug, Parser, PartialEq, Eq)]
#[command(
  name = "argvus clock",
  about = "ARGVUS Clock: clock, world clocks, stopwatch, timer and alarms",
  version
)]
pub struct Cli {
  /// Run without opening the window, to keep alarms active (used when
  /// starting with the session).
  #[arg(long, conflicts_with = "toggle")]
  pub background: bool,
  /// Hide the window if it is visible, show it otherwise (taskbar clock).
  #[arg(long)]
  pub toggle: bool,
  /// Page to show.
  #[arg(long, value_enum)]
  pub page: Option<PageArg>,
  /// Click position (desktop coordinates) passed by the taskbar, so the popup
  /// opens next to the taskbar clock.
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
      std::iter::once("argvus-clock")
        .chain(arguments.iter().copied())
        .map(OsString::from)
        .collect(),
    )
  }

  #[test]
  fn page_and_toggle_parse() {
    let cli = parse(&["--toggle", "--page", "alarms"]).unwrap();
    assert!(cli.toggle);
    assert_eq!(cli.page, Some(PageArg::Alarms));
  }

  #[test]
  fn background_conflicts_with_toggle() {
    assert!(parse(&["--background", "--toggle"]).is_err());
    assert!(parse(&["--background"]).unwrap().background);
  }

  #[test]
  fn taskbar_click_position_becomes_an_anchor() {
    let cli = parse(&[
      "--toggle",
      "--x",
      "1700",
      "--y",
      "14",
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
        y: 14,
        bar_bottom: Some(46),
        bar_right: Some(1902)
      })
    );
    assert_eq!(parse(&["--toggle"]).unwrap().anchor(), None);
    assert!(parse(&["--y", "10"]).is_err(), "y needs x");
  }

  #[test]
  fn unknown_page_is_rejected() {
    assert!(parse(&["--page", "calendar"]).is_err());
  }
}
