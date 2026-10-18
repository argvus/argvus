//! `argvus`: the public command dispatcher of the ARGVUS desktop. Each subcommand
//! is served by a private executable, a packaged script or an internal route. The
//! dispatcher replaces its own process with the target, so arguments, stdio,
//! signals and the exit status pass through unchanged.

mod apps;
mod commands;
mod control_center;
mod copy;
mod dispatch;
mod exec;
mod paths;
mod script;
mod setup;
mod usage;

use std::ffi::OsString;
use std::process::ExitCode;

fn main() -> ExitCode {
  let arguments: Vec<OsString> = std::env::args_os().skip(1).collect();
  match dispatch::run(&arguments) {
    Ok(exit_code) => exit_code,
    Err(failure) => {
      if !failure.message().is_empty() {
        eprintln!("{}", failure.message());
      }
      ExitCode::from(failure.code())
    }
  }
}
