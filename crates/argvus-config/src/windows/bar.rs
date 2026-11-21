use super::*;
use crate::ConfigStore;
use std::process::Command;

pub(super) enum PositionMode {
  Save,
  Apply,
}

pub(super) fn position(position: &str, mode: PositionMode) -> ConfigResult<()> {
  if !matches!(position, "top" | "bottom") {
    return Err(invalid("position must be top or bottom"));
  }
  let store = ConfigStore::from_environment()?;
  let document = store.update("/layout/taskbar/position", json!(position))?;
  crate::project::project_layout_only(&store, &document)?;
  if matches!(mode, PositionMode::Apply) && std::env::var("ARGVUS_NO_RUNTIME").as_deref() != Ok("1")
  {
    let status = Command::new("argvus")
      .args(["sessionctl", "bars-presentation"])
      .status()?;
    if !status.success() {
      return Err(invalid(
        "taskbar position saved but presentation could not be applied",
      ));
    }
  }
  Ok(())
}

/// Activates the Floating surface in the session's shared shell.
pub(super) fn run() -> ConfigResult<()> {
  let status = Command::new("systemctl")
    .args(["--user", "start", "argvus-window-bar.service"])
    .status()?;
  if !status.success() {
    return Err(invalid("could not start the window bar"));
  }
  Ok(())
}
