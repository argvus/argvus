//! Process supervision and compositor readiness checks.

use std::process::Command;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Action {
  Lock,
  Suspend,
  LogOut,
  Reboot,
  Shutdown,
}

impl Action {
  pub const ALL: [Self; 5] = [
    Self::Lock,
    Self::Suspend,
    Self::LogOut,
    Self::Reboot,
    Self::Shutdown,
  ];

  pub fn requires_confirmation(self) -> bool {
    matches!(self, Self::LogOut | Self::Reboot | Self::Shutdown)
  }

  /// Fixed command and arguments; no user-controlled text is passed to a shell.
  pub fn command(self) -> (&'static str, &'static [&'static str]) {
    match self {
      Self::Lock => ("argvus", &["lock"]),
      Self::Suspend => ("systemctl", &["suspend"]),
      Self::LogOut => ("hyprshutdown", &[]),
      Self::Reboot => ("hyprshutdown", &["--post-cmd", "systemctl reboot"]),
      Self::Shutdown => ("hyprshutdown", &["--post-cmd", "systemctl poweroff"]),
    }
  }
}

pub enum Handoff {
  Ready,
  Finished,
}

pub fn run(action: Action, ready: impl FnOnce()) -> std::io::Result<Handoff> {
  let (program, args) = action.command();
  if action == Action::Lock || action == Action::Suspend {
    lock_session()?;
    if action == Action::Lock {
      return Ok(Handoff::Finished);
    }
  }
  let mut command = Command::new(program);
  command.args(args);
  if action.requires_confirmation() {
    if shutdown_surface_exists(None) {
      return Err(std::io::Error::other("hyprshutdown is already active"));
    }
    command.arg("--no-fork");
  }
  let mut child = command.spawn()?;
  if action.requires_confirmation() {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(4);
    while std::time::Instant::now() < deadline {
      if let Some(status) = child.try_wait()? {
        return finish(status);
      }
      if shutdown_surface_exists(Some(child.id())) {
        break;
      }
      std::thread::sleep(std::time::Duration::from_millis(75));
    }
    ready();
  }
  finish(child.wait()?)
}

fn finish(status: std::process::ExitStatus) -> std::io::Result<Handoff> {
  if status.success() {
    Ok(Handoff::Finished)
  } else {
    Err(std::io::Error::other(status.to_string()))
  }
}

fn compositor_locked() -> std::io::Result<bool> {
  let result = Command::new("hyprctl").arg("locked").output()?;
  match (result.status.success(), result.stdout.trim_ascii()) {
    (true, b"true") => Ok(true),
    (true, b"false") => Ok(false),
    _ => Err(std::io::Error::other(
      "could not query the compositor lock state",
    )),
  }
}

fn lock_session() -> std::io::Result<()> {
  if compositor_locked()? {
    return Ok(());
  }
  let (program, args) = Action::Lock.command();
  let mut locker = Command::new(program).args(args).spawn()?;
  let deadline = std::time::Instant::now() + std::time::Duration::from_secs(8);
  let result = loop {
    match compositor_locked() {
      Ok(true) => break Ok(()),
      Ok(false) => {}
      Err(error) => break Err(error),
    }
    match locker.try_wait() {
      Ok(None) => {}
      Ok(Some(status)) => {
        return Err(std::io::Error::other(format!(
          "locker exited before locking: {status}"
        )));
      }
      Err(error) => break Err(error),
    }
    if std::time::Instant::now() >= deadline {
      break Err(std::io::Error::other(
        "session lock was not confirmed before timeout",
      ));
    }
    std::thread::sleep(std::time::Duration::from_millis(100));
  };
  std::thread::spawn(move || {
    let _ = locker.wait();
  });
  result
}

fn shutdown_surface_exists(pid: Option<u32>) -> bool {
  let Ok(output) = Command::new("hyprctl").args(["layers", "-j"]).output() else {
    return false;
  };
  let Ok(layers) = serde_json::from_slice::<serde_json::Value>(&output.stdout) else {
    return false;
  };
  layers
    .as_object()
    .into_iter()
    .flat_map(|outputs| outputs.values())
    .filter_map(|monitor| monitor["levels"].as_object())
    .flat_map(|levels| levels.values())
    .filter_map(serde_json::Value::as_array)
    .flatten()
    .any(|layer| layer["namespace"] == "hyprshutdown" && pid.is_none_or(|pid| layer["pid"] == pid))
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::os::unix::process::ExitStatusExt;

  #[test]
  fn rejected_commands_are_not_reported_as_finished() {
    let rejected = std::process::ExitStatus::from_raw(1 << 8);
    assert!(finish(rejected).is_err());
    let accepted = std::process::ExitStatus::from_raw(0);
    assert!(matches!(finish(accepted), Ok(Handoff::Finished)));
  }
}
