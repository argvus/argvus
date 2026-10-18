//! Implements hostname configuration in crate `argvus control center settings`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use std::fs;
use std::process::Stdio;

use crate::error::SettingsError;

/// Executes the `current` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn current() -> String {
  command_stdout("hostnamectl", &["--static"])
    .or_else(|| fs::read_to_string("/etc/hostname").ok())
    .unwrap_or_default()
    .trim()
    .to_string()
}

/// Executes the `validate_hostname` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn validate_hostname(name: &str) -> Result<(), String> {
  let name = name.trim();
  if name.is_empty() || name.len() > 63 {
    return Err("hostname must be 1-63 characters".to_string());
  }
  if name.starts_with('-') || name.ends_with('-') {
    return Err("hostname cannot start or end with '-'".to_string());
  }
  if !name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-') {
    return Err("hostname can contain only letters, numbers and '-'".to_string());
  }
  Ok(())
}

/// Executes the `set` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn set(name: &str) -> Result<(), SettingsError> {
  validate_hostname(name).map_err(SettingsError::System)?;
  super::privileged::run(&["hostname", "set", name]).map(|_| ())
}

/// Executes the `command_stdout` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn command_stdout(command: &str, args: &[&str]) -> Option<String> {
  let output = argvus_control_center_core::process::command(command)
    .args(args)
    .stdin(Stdio::null())
    .output()
    .ok()?;
  output
    .status
    .success()
    .then(|| String::from_utf8_lossy(&output.stdout).to_string())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  /// Executes the `validates_hostname` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn validates_hostname() {
    assert!(validate_hostname("argvus-workstation").is_ok());
    assert!(validate_hostname("").is_err());
    assert!(validate_hostname("-bad").is_err());
    assert!(validate_hostname("bad_name").is_err());
  }
}
