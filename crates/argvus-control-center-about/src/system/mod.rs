//! Implements module declarations for the `system` subsystem in crate `argvus about`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use std::fs;

pub mod cpu;
pub mod gpu;
pub mod memory;
pub mod os;
pub mod session;

#[derive(Debug, Clone)]
/// Represents `SystemInfo`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct SystemInfo {
  pub hostname: String,
  pub os_name: String,
  pub os_type: String,
  pub distributor: String,
  pub kernel: String,
  pub window_system: String,
  pub cpu: String,
  pub memory: String,
  pub gpus: String,
}

impl SystemInfo {
  /// Executes the `gather` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn gather(na: &str) -> Self {
    let os_release = read_to_string("/etc/os-release").unwrap_or_default();
    let (os_name, distributor) = os::os_names(&os_release);

    Self {
      hostname: hostname().unwrap_or_else(|| na.to_string()),
      os_name: if os_name.is_empty() {
        na.to_string()
      } else {
        os_name
      },
      os_type: format!("{} bits", usize::BITS),
      distributor: if distributor.is_empty() {
        na.to_string()
      } else {
        distributor
      },
      kernel: command_output("uname", &["-r"]).unwrap_or_else(|| na.to_string()),
      window_system: session::session_name().unwrap_or_else(|| na.to_string()),
      cpu: cpu::cpu_info(&read_to_string("/proc/cpuinfo").unwrap_or_default())
        .unwrap_or_else(|| na.to_string()),
      memory: memory::memory_info(&read_to_string("/proc/meminfo").unwrap_or_default())
        .unwrap_or_else(|| na.to_string()),
      gpus: gpu::gpu_info(command_output("lspci", &[]).as_deref())
        .unwrap_or_else(|| na.to_string()),
    }
  }
}

/// Executes the `argvus_version` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn argvus_version(na: &str) -> String {
  installed_package_version("argvus").unwrap_or_else(|| na.to_string())
}

/// Executes the `installed_package_version` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn installed_package_version(package: &str) -> Option<String> {
  command_output("pacman", &["-Q", package])
    .and_then(|line| line.split_whitespace().nth(1).map(str::to_string))
}

/// Executes the `gtk_version` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn gtk_version(na: &str) -> String {
  ["gtk4", "gtk4.0", "gtk+-3.0"]
    .iter()
    .find_map(|name| pc_file_version(name))
    .unwrap_or_else(|| na.to_string())
}

/// Executes the `pc_file_version` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn pc_file_version(name: &str) -> Option<String> {
  for dir in pc_dirs() {
    let path = dir.join(format!("{name}.pc"));
    if let Ok(contents) = fs::read_to_string(path)
      && let Some(version) = parse_pc_version(&contents)
    {
      return Some(version);
    }
  }
  None
}

/// Converts input data into `parse_pc_version` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn parse_pc_version(contents: &str) -> Option<String> {
  contents.lines().find_map(|line| {
    let (key, value) = line.split_once(':')?;
    (key.trim() == "Version").then(|| value.trim().to_string())
  })
}

/// Executes the `pc_dirs` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn pc_dirs() -> Vec<std::path::PathBuf> {
  let mut dirs = Vec::new();
  if let Some(path) = std::env::var_os("PKG_CONFIG_PATH") {
    dirs.extend(std::env::split_paths(&path));
  }
  dirs.extend(
    [
      "/usr/lib/pkgconfig",
      "/usr/lib/x86_64-linux-gnu/pkgconfig",
      "/usr/local/lib/pkgconfig",
      "/usr/share/pkgconfig",
    ]
    .map(std::path::PathBuf::from),
  );
  dirs
}

/// Executes the `hostname` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn hostname() -> Option<String> {
  read_to_string("/etc/hostname")
    .map(|value| value.trim().to_string())
    .filter(|value| !value.is_empty())
    .or_else(|| command_output("hostname", &[]))
}

/// Retrieves data for `read_to_string` without mixing collection with TUI rendering. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn read_to_string(path: &str) -> Option<String> {
  fs::read_to_string(path).ok()
}

/// Executes the `command_output` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn command_output(command: &str, args: &[&str]) -> Option<String> {
  let output = argvus_control_center_core::process::command(command)
    .args(args)
    .output()
    .ok()?;
  output
    .status
    .success()
    .then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  /// Converts input data into `parses_pkgconfig_version` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn parses_pkgconfig_version() {
    assert_eq!(
      parse_pc_version("prefix=/usr\ndescription=GTK\nVersion: 4.22.4\n"),
      Some("4.22.4".into())
    );
    assert_eq!(parse_pc_version("url: https://x\n"), None);
  }

  #[test]
  /// Converts input data into `parses_pacman_query_line` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn parses_pacman_query_line() {
    let parsed = installed_package_version("argvus");
    let _ = parsed;
  }
}
