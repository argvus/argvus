//! The public subcommand table of `argvus`. The first argument selects one route,
//! and the remaining arguments are forwarded to it unchanged.

use crate::commands::{
  GAME_SNAKE, LOADING_THEME_BINARY, PrivateCommand, REMOVABLE_DEVICES, ScriptRoute,
  TASKBAR_CALENDAR, required_executable,
};
use crate::exec::{Failure, Outcome, is_executable, replace_process};
use crate::{apps, control_center, script, setup, usage};
use std::ffi::OsString;
use std::io::{self, IsTerminal};
use std::path::Path;
use std::process::{Command, ExitCode};

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Routes the command line. Routes that replace the process only return on
/// failure, so their result is always an error.
pub fn run(arguments: &[OsString]) -> Outcome {
  let Some(first_argument) = arguments.first() else {
    print!("{}", usage::TEXT);
    return Ok(ExitCode::SUCCESS);
  };
  let rest = &arguments[1..];
  let Some(subcommand) = first_argument.to_str() else {
    return Err(unknown_command(&first_argument.to_string_lossy()));
  };

  match subcommand {
    "" | "--help" | "-h" | "help" => {
      print!("{}", usage::TEXT);
      Ok(ExitCode::SUCCESS)
    }
    "--version" | "-V" | "version" => {
      println!("ARGVUS {VERSION}");
      Ok(ExitCode::SUCCESS)
    }
    // The desktop session starts `tty` without forwarding arguments: the shell
    // route never passed them on, and the session depends on that.
    "--start-desktop" => exec_private(PrivateCommand::Tty, &[]),
    "--setup" | "setup" => setup::run(rest),
    "--system-monitor" | "system-monitor" => {
      Ok(argvus_system_monitor::run_command(owned_arguments(rest)?))
    }
    "--control-center" | "control-center" => control_center::run(rest.to_vec()),
    "tui-terminal" => Ok(argvus_tui_terminal::run_command(owned_arguments(rest)?)),
    "--terminal" | "terminal" => Ok(argvus_terminal::run_command(owned_arguments(rest)?)),
    "projects" => Ok(argvus_projects::run_command(owned_arguments(rest)?)),
    "widget-telemetry" => widget_telemetry(rest),
    "loading-theme" => exec_program(Path::new(LOADING_THEME_BINARY), rest),
    "network" | "networkctl" => Ok(argvus_network_commands::networkctl::run_command(
      "argvus-networkctl",
      owned_arguments(rest)?,
    )),
    "bluetooth" | "bluetoothctl" => Ok(argvus_network_commands::bluetoothctl::run_command(
      "argvus-bluetoothctl",
      owned_arguments(rest)?,
    )),
    // Compatibility names for the callers moved in a later step: they run the same
    // in-process commands as the subcommands above.
    "widget-telemetry-toggle" => Ok(argvus_widget_telemetry_toggle::run_command(
      owned_arguments(rest)?,
    )),
    "widget-telemetry-block" => Ok(argvus_widget_telemetry_blocks::run_command(
      owned_arguments(rest)?,
    )),
    "--spf" | "spf" | "--superfile" | "superfile" => apps::run_spf(rest),
    "--yazy" | "yazy" | "--yazi" | "yazi" => apps::run_yazi(rest),
    "--about" | "about" => control_center::run(with_leading("--about", rest)),
    "--calendar" | "calendar" => exec_named(TASKBAR_CALENDAR, &default_arguments(rest, "toggle")),
    "--game-snake" | "game-snake" => game_snake(rest),
    "--default-apps" | "default-apps" => control_center::run(with_leading("apps", rest)),
    "--removable-devices" | "removable-devices" | "--storage" | "storage" => {
      exec_named(REMOVABLE_DEVICES, &default_arguments(rest, "menu"))
    }
    "--displays" | "displays" => control_center::run(with_leading("displays", rest)),
    "--network-interfaces" => control_center::run(with_leading_pair("network", "interfaces", rest)),
    // Theme and effect transactions belong to argvus-config; these routes keep
    // the names the keybindings and the callers already use.
    "theme-switch" => run_config(&with_leading("theme-apply", rest)),
    "effects-toggle" => run_config(&with_leading("effects", rest)),
    "config" => run_config(rest),
    "gpg" => Ok(argvus_keys::run_command(command_words("gpg", rest)?)),
    "keys" => Ok(argvus_keys::run_command(owned_arguments(rest)?)),
    // Runs in-process: the i18n commands are a library of the workspace.
    "i18n" => Ok(argvus_i18n_cli::run(&owned_arguments(rest)?)),
    other => {
      if let Some(route) = ScriptRoute::from_name(other) {
        return script::run(route, rest);
      }
      let command = PrivateCommand::from_subcommand(other).ok_or_else(|| unknown_command(other))?;
      exec_private(command, rest)
    }
  }
}

fn unknown_command(subcommand: &str) -> Failure {
  Failure::new(format!(
    "unknown command: {subcommand} (see 'argvus --help')"
  ))
}

/// Replaces this process with a private command from the table.
fn exec_private(command: PrivateCommand, arguments: &[OsString]) -> Outcome {
  exec_named(command.executable_name(), arguments)
}

/// Replaces this process with an executable at a fixed path, such as the loading
/// overlay, which other packages launch from the same place.
fn exec_program(program: &Path, arguments: &[OsString]) -> Outcome {
  if !is_executable(program) {
    return Err(Failure::new(format!(
      "{} is not installed",
      program.display()
    )));
  }
  Err(replace_process(Command::new(program).args(arguments)))
}

/// Replaces this process with a named private executable.
fn exec_named(name: &str, arguments: &[OsString]) -> Outcome {
  let executable = required_executable(name)?;
  Err(replace_process(Command::new(executable).args(arguments)))
}

/// Runs `argvus-config` in-process with `arguments`. The configuration commands are a
/// library of the workspace, so they need no executable and no PATH lookup.
fn run_config(arguments: &[OsString]) -> Outcome {
  Ok(argvus_config_core::cli::run_command(owned_arguments(
    arguments,
  )?))
}

/// Starts the game directly in a terminal, or inside the terminal wrapper when
/// the dispatcher has no terminal of its own.
fn game_snake(arguments: &[OsString]) -> Outcome {
  let game = required_executable(GAME_SNAKE)?;
  if io::stdin().is_terminal() && io::stdout().is_terminal() {
    return Err(replace_process(Command::new(game).args(arguments)));
  }
  let argvus = std::env::current_exe()
    .map_err(|error| Failure::new(format!("cannot locate argvus: {error}")))?;
  Err(replace_process(
    Command::new(argvus)
      .arg("tui-terminal")
      .args([
        "--class",
        "argvus-game-snake",
        "--term",
        "kitty",
        "--profile",
        "control-center",
        "--",
      ])
      .arg(game)
      .args(arguments),
  ))
}

/// Replaces the arguments with `default` when none were given.
fn default_arguments(arguments: &[OsString], default: &str) -> Vec<OsString> {
  if arguments.is_empty() {
    vec![OsString::from(default)]
  } else {
    arguments.to_vec()
  }
}

/// The words of an in-process command: the command name, then `arguments`.
fn command_words(command: &str, arguments: &[OsString]) -> Result<Vec<String>, Failure> {
  Ok(
    std::iter::once(command.to_owned())
      .chain(owned_arguments(arguments)?)
      .collect(),
  )
}

/// `argvus widget-telemetry block|toggle ...`: the Waybar block and the toggle.
fn widget_telemetry(arguments: &[OsString]) -> Outcome {
  match arguments.first().and_then(|argument| argument.to_str()) {
    Some("block") => Ok(argvus_widget_telemetry_blocks::run_command(
      owned_arguments(&arguments[1..])?,
    )),
    Some("toggle") => Ok(argvus_widget_telemetry_toggle::run_command(
      owned_arguments(&arguments[1..])?,
    )),
    _ => {
      eprintln!("usage: argvus widget-telemetry <block|toggle> [arguments...]");
      Ok(ExitCode::from(2))
    }
  }
}

/// The arguments of an in-process command as strings. A non-UTF-8 argument (a
/// file name in a legacy encoding, for example) is refused rather than altered,
/// so a command never acts on a different path than the one it was given.
fn owned_arguments(arguments: &[OsString]) -> Result<Vec<String>, Failure> {
  arguments
    .iter()
    .map(|argument| {
      argument.to_str().map(str::to_owned).ok_or_else(|| {
        Failure::with_code(
          format!(
            "argument is not valid UTF-8: {}",
            argument.to_string_lossy()
          ),
          2,
        )
      })
    })
    .collect()
}

fn with_leading(first: &str, arguments: &[OsString]) -> Vec<OsString> {
  std::iter::once(OsString::from(first))
    .chain(arguments.iter().cloned())
    .collect()
}

fn with_leading_pair(first: &str, second: &str, arguments: &[OsString]) -> Vec<OsString> {
  [OsString::from(first), OsString::from(second)]
    .into_iter()
    .chain(arguments.iter().cloned())
    .collect()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn default_arguments_apply_only_when_empty() {
    assert_eq!(
      default_arguments(&[], "toggle"),
      vec![OsString::from("toggle")]
    );
    let given = [OsString::from("show")];
    assert_eq!(default_arguments(&given, "toggle"), given.to_vec());
  }

  #[test]
  fn non_utf8_arguments_are_refused_instead_of_altered() {
    use std::os::unix::ffi::OsStringExt as _;
    let invalid = OsString::from_vec(b"caf\xe9.txt".to_vec());
    let failure = owned_arguments(&[OsString::from("open"), invalid]).unwrap_err();
    assert_eq!(failure.code(), 2);
    assert!(failure.message().contains("not valid UTF-8"));
    assert_eq!(
      owned_arguments(&[OsString::from("café.txt")]).unwrap(),
      vec!["café.txt".to_owned()]
    );
  }

  #[test]
  fn leading_arguments_precede_the_forwarded_ones() {
    let forwarded = [OsString::from("list")];
    assert_eq!(
      with_leading("gpg", &forwarded),
      vec![OsString::from("gpg"), OsString::from("list")]
    );
  }
}
