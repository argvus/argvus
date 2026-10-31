//! Public onboarding entry point, hosted by the Control Center.
use crate::exec::{Failure, Outcome, find_on_path};
use argvus_welcome::state::StateStore;
use std::ffi::OsString;
use std::process::ExitCode;

pub fn run(arguments: Vec<OsString>) -> Outcome {
  let automatic = match arguments.as_slice() {
    [] => false,
    [argument] if argument == "--if-needed" => true,
    _ => return Err(Failure::new("usage: argvus welcome [--if-needed]")),
  };
  let store = StateStore::new().map_err(|error| Failure::new(error.to_string()))?;
  if automatic {
    let (progress, _) = store.load();
    if progress.opened || progress.completed || progress.auto_dismissed {
      store
        .consume_auto_start_marker()
        .map_err(|error| Failure::new(error.to_string()))?;
      return Ok(ExitCode::SUCCESS);
    }
  }
  if find_on_path("hyprctl").is_none() {
    return Err(Failure::new(
      "ARGVUS Welcome needs an active Hyprland graphical session.",
    ));
  }
  crate::control_center::run(vec![OsString::from("welcome")])
}
