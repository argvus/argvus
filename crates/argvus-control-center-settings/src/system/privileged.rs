//! Typed privileged-operation integration of the Settings pages.
use std::process::Stdio;

use crate::error::SettingsError;

pub fn run(args: &[&str]) -> Result<String, SettingsError> {
  // The executable is the running Control Center binary, which now lives in
  // the private commands directory and is not on PATH.
  let executable = std::env::current_exe()
    .map(|path| path.to_string_lossy().into_owned())
    .unwrap_or_else(|_| "argvus-control-center".into());
  let output = argvus_control_center_core::process::command(executable)
    .arg("system-settings")
    .args(args)
    .stdin(Stdio::null())
    .output()
    .map_err(|error| SettingsError::System(format!("argvus-control-center: {error}")))?;
  if output.status.success() {
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
  } else {
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    Err(SettingsError::System(if stderr.is_empty() {
      format!(
        "argvus-control-center system-settings exited with {}",
        output.status
      )
    } else {
      stderr
    }))
  }
}
