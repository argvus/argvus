//! The command line of the launcher. The first argument chooses a mode; without
//! one, the options before `--` (or before the first other argument) belong to
//! the launcher and the rest go to kitty.

use crate::profile;

/// What the command line asks for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
  Help,
  /// Regenerates the configurations of a theme; without one, the active theme.
  Apply(Option<String>),
  PrintConfig(Option<String>),
  PrintTuiConfig(Option<String>),
  Launch(Launch),
}

/// A launch of kitty.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Launch {
  pub class: String,
  pub font_profile: String,
  pub tui: bool,
  pub kitty_arguments: Vec<String>,
}

/// A command line the launcher rejects, with the message key and its argument.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
  ClassRequiresValue,
  FontProfileRequiresValue,
  UnsupportedFontProfile(String),
}

/// Reads the command line, without the program name.
pub fn parse(arguments: &[String]) -> Result<Mode, Error> {
  let theme = || arguments.get(1).filter(|theme| !theme.is_empty()).cloned();
  match arguments.first().map(String::as_str) {
    Some("--help" | "-h") => Ok(Mode::Help),
    Some("--apply") => Ok(Mode::Apply(theme())),
    Some("--print-config") => Ok(Mode::PrintConfig(theme())),
    Some("--print-tui-config") => Ok(Mode::PrintTuiConfig(theme())),
    _ => parse_launch(arguments).map(Mode::Launch),
  }
}

fn parse_launch(arguments: &[String]) -> Result<Launch, Error> {
  let mut launch = Launch {
    class: "kitty".to_string(),
    font_profile: "terminal".to_string(),
    tui: false,
    kitty_arguments: Vec::new(),
  };
  let mut index = 0;
  while let Some(argument) = arguments.get(index) {
    match argument.as_str() {
      "--class" => {
        launch.class = arguments
          .get(index + 1)
          .ok_or(Error::ClassRequiresValue)?
          .clone();
        index += 2;
      }
      "--font-profile" => {
        launch.font_profile = arguments
          .get(index + 1)
          .ok_or(Error::FontProfileRequiresValue)?
          .clone();
        index += 2;
      }
      "--tui" => {
        launch.tui = true;
        index += 1;
      }
      "--" => {
        index += 1;
        break;
      }
      other => {
        if let Some(class) = other.strip_prefix("--class=") {
          launch.class = class.to_string();
        } else if let Some(profile) = other.strip_prefix("--font-profile=") {
          launch.font_profile = profile.to_string();
        } else {
          break;
        }
        index += 1;
      }
    }
  }
  if !profile::is_known(&launch.font_profile) {
    return Err(Error::UnsupportedFontProfile(launch.font_profile));
  }
  launch.kitty_arguments = arguments[index..].to_vec();
  Ok(launch)
}

#[cfg(test)]
mod tests {
  use super::{Error, Launch, Mode, parse};

  fn words(text: &[&str]) -> Vec<String> {
    text.iter().map(|word| word.to_string()).collect()
  }

  #[test]
  fn modes_take_an_optional_theme() {
    assert_eq!(parse(&words(&["--apply"])), Ok(Mode::Apply(None)));
    assert_eq!(parse(&words(&["--apply", ""])), Ok(Mode::Apply(None)));
    assert_eq!(
      parse(&words(&["--print-tui-config", "dracula"])),
      Ok(Mode::PrintTuiConfig(Some("dracula".into())))
    );
  }

  #[test]
  fn launch_reads_its_options_and_passes_the_rest() {
    assert_eq!(
      parse(&words(&[
        "--class",
        "argvus-keys",
        "--tui",
        "--",
        "-e",
        "zsh"
      ])),
      Ok(Mode::Launch(Launch {
        class: "argvus-keys".into(),
        font_profile: "terminal".into(),
        tui: true,
        kitty_arguments: words(&["-e", "zsh"]),
      }))
    );
    assert_eq!(
      parse(&words(&["--font-profile=apps", "--class=x", "htop"])),
      Ok(Mode::Launch(Launch {
        class: "x".into(),
        font_profile: "apps".into(),
        tui: false,
        kitty_arguments: words(&["htop"]),
      }))
    );
  }

  #[test]
  fn launch_rejects_missing_values_and_unknown_profiles() {
    assert_eq!(parse(&words(&["--class"])), Err(Error::ClassRequiresValue));
    assert_eq!(
      parse(&words(&["--font-profile"])),
      Err(Error::FontProfileRequiresValue)
    );
    assert_eq!(
      parse(&words(&["--font-profile", "editor"])),
      Err(Error::UnsupportedFontProfile("editor".into()))
    );
  }
}
