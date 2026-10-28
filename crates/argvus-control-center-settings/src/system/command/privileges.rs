//! Root checks: re-running through pkexec and the effective user.

use super::*;

pub(super) fn ensure_root(args: &[String]) -> Result<(), String> {
  if effective_uid() == 0 || std::env::var_os(ELEVATED_ENV).is_some() {
    return Ok(());
  }
  let pkexec = locate_pkexec().ok_or_else(|| "pkexec was not found; install polkit".to_string())?;
  let exe = std::env::current_exe().map_err(|error| error.to_string())?;
  let mut command = argvus_control_center_core::process::command(pkexec);
  command
    .arg(exe)
    .arg("system-settings")
    .args(args)
    .env_clear()
    .env(ELEVATED_ENV, "1");
  if let Some(value) = std::env::var_os("PATH") {
    command.env("PATH", value);
  } else {
    command.env(
      "PATH",
      "/usr/local/sbin:/usr/local/bin:/usr/bin:/usr/sbin:/sbin:/bin",
    );
  }
  if let Some(value) = std::env::var_os("DBUS_SESSION_BUS_ADDRESS") {
    command.env("DBUS_SESSION_BUS_ADDRESS", value);
  }
  if let Some(value) = std::env::var_os("DBUS_SYSTEM_BUS_ADDRESS") {
    command.env("DBUS_SYSTEM_BUS_ADDRESS", value);
  }
  if let Some(value) = std::env::var_os("XDG_RUNTIME_DIR") {
    command.env("XDG_RUNTIME_DIR", value);
  }
  let error = command.exec();
  Err(format!("failed to launch pkexec: {error}"))
}

pub(super) fn effective_uid() -> u32 {
  fs::read_to_string("/proc/self/status")
    .ok()
    .and_then(|contents| {
      contents.lines().find_map(|line| {
        line
          .strip_prefix("Uid:")
          .and_then(|rest| rest.split_whitespace().nth(1))
          .and_then(|value| value.parse().ok())
      })
    })
    .unwrap_or(1)
}

pub(super) fn locate_pkexec() -> Option<PathBuf> {
  std::env::var_os("PATH").and_then(|path| {
    std::env::split_paths(&path)
      .map(|dir| dir.join("pkexec"))
      .find(|candidate| candidate.is_file())
  })
}
