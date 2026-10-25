//! Implements module declarations for the `system` subsystem in crate `argvus about`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use std::collections::HashMap;
use std::fs;

pub mod battery;
pub mod board;
pub mod boot;
pub mod cpu;
pub mod displays;
pub mod gpu;
pub mod hyprland;
pub mod memory;
pub mod os;
pub mod packages;
pub mod session;
pub mod storage;

use battery::Battery;
use boot::BootMode;
use displays::Display;
use memory::MemoryUsage;
use packages::PackageCounts;
use storage::Storage;

/// Packages whose versions the page shows, queried together in one `pacman -Q` call.
const VERSIONED_PACKAGES: [&str; 9] = [
  "argvus",
  "hyprland",
  "wayland",
  "mesa",
  "pipewire",
  "quickshell",
  "greetd",
  "systemd",
  "grub",
];

#[derive(Debug, Clone, Default)]
/// Facts shown on the System & Hardware tab. `None` means the value could not be read;
/// the page renders it as "N/A" in the interface language.
pub struct SystemInfo {
  // Boot
  pub firmware: Option<String>,
  pub boot_mode: Option<BootMode>,
  pub secure_boot: Option<bool>,
  pub virtualization: Option<String>,
  pub bootloader: Option<String>,
  // System
  pub argvus: Option<String>,
  pub hostname: Option<String>,
  pub os_name: Option<String>,
  pub distributor: Option<String>,
  pub kernel: Option<String>,
  pub architecture: Option<String>,
  pub hyprland: Option<String>,
  pub wayland: Option<String>,
  pub gpu_drivers: Option<String>,
  pub gtk: Option<String>,
  /// Active monitors; `None` outside a Hyprland session.
  pub displays: Option<Vec<Display>>,
  pub pipewire: Option<String>,
  pub quickshell: Option<String>,
  pub greetd: Option<String>,
  pub systemd: Option<String>,
  pub packages: Option<PackageCounts>,
  pub uptime_seconds: Option<u64>,
  pub locale: Option<String>,
  // Hardware
  /// System batteries; empty on machines without one.
  pub batteries: Vec<Battery>,
  pub cpu: Option<String>,
  pub cpu_max_mhz: Option<u64>,
  pub threads: Option<usize>,
  pub cores: Option<usize>,
  pub gpus: Option<String>,
  pub memory: Option<MemoryUsage>,
  pub storage: Option<Storage>,
  pub model: Option<String>,
  pub motherboard: Option<String>,
}

impl SystemInfo {
  /// Reads every value from sysfs, procfs, efivarfs and a few short-lived commands
  /// (`pacman`, `uname`, `hyprctl`, `lspci`, `systemd-detect-virt`).
  pub fn gather() -> Self {
    // Counting foreign packages makes pacman read every sync database, the slowest probe
    // by far; it runs on its own thread while the others proceed.
    std::thread::scope(|scope| {
      let package_counts = scope.spawn(packages::package_counts);
      let mut info = Self::gather_without_package_counts();
      info.packages = package_counts.join().ok().flatten();
      info
    })
  }

  /// Reads everything but the package counts, which [`SystemInfo::gather`] runs in parallel.
  fn gather_without_package_counts() -> Self {
    let os_release = read_to_string("/etc/os-release").unwrap_or_default();
    let (os_name, distributor) = os::os_names(&os_release);
    let cpuinfo = read_to_string("/proc/cpuinfo").unwrap_or_default();
    let mut packages = parse_pacman_versions(&command_stdout("pacman", &pacman_query_args()));
    let (kernel, architecture) = parse_uname(command_output("uname", &["-r", "-m"]).as_deref());
    let boot_mode = boot::boot_mode();
    let mesa = packages.remove("mesa");

    Self {
      firmware: boot::firmware(),
      boot_mode: Some(boot_mode),
      secure_boot: if boot_mode == BootMode::Uefi {
        boot::secure_boot()
      } else {
        None
      },
      virtualization: boot::virtualization(),
      bootloader: boot::bootloader(packages.get("grub").map(String::as_str)),
      argvus: packages.remove("argvus"),
      hostname: hostname(),
      os_name: non_empty(os_name),
      distributor: non_empty(distributor),
      kernel,
      architecture: architecture.or_else(|| Some(std::env::consts::ARCH.to_string())),
      // The running compositor is authoritative; the package is the fallback outside a session.
      hyprland: hyprland::running_version().or_else(|| packages.remove("hyprland")),
      wayland: packages.remove("wayland"),
      gpu_drivers: gpu::gpu_drivers(&gpu::gpu_driver_names(), mesa.as_deref()),
      gtk: gtk_version(),
      displays: displays::displays(),
      pipewire: packages.remove("pipewire"),
      quickshell: packages.remove("quickshell"),
      greetd: packages.remove("greetd"),
      systemd: packages.remove("systemd"),
      packages: None,
      uptime_seconds: session::uptime_seconds(),
      locale: session::system_locale(),
      batteries: battery::batteries(),
      cpu: cpu::cpu_model(&cpuinfo),
      cpu_max_mhz: cpu::max_frequency_mhz(),
      threads: cpu::threads(&cpuinfo),
      cores: cpu::cores(&cpuinfo),
      gpus: gpu::gpu_info(command_output("lspci", &[]).as_deref()),
      memory: memory::memory_usage(&read_to_string("/proc/meminfo").unwrap_or_default()),
      storage: storage::root_storage(),
      model: board::computer_model(),
      motherboard: board::motherboard(),
    }
  }
}

/// Builds `pacman -Q <package>...` arguments for every versioned package.
fn pacman_query_args() -> Vec<&'static str> {
  std::iter::once("-Q").chain(VERSIONED_PACKAGES).collect()
}

/// Maps package names to upstream versions from `pacman -Q` lines such as
/// `mesa 1:25.2.3-1` (shown as `25.2.3`).
///
/// Missing packages are reported on stderr, so only the installed ones appear here.
pub fn parse_pacman_versions(output: &str) -> HashMap<String, String> {
  output
    .lines()
    .filter_map(|line| {
      let mut fields = line.split_whitespace();
      let name = fields.next()?.to_string();
      Some((name, upstream_version(fields.next()?).to_string()))
    })
    .collect()
}

/// Drops the pacman epoch (`1:`) and package release (`-1`) from a version.
fn upstream_version(version: &str) -> &str {
  let without_epoch = version.split_once(':').map_or(version, |(_, rest)| rest);
  without_epoch
    .rsplit_once('-')
    .map_or(without_epoch, |(upstream, _)| upstream)
}

/// Splits `uname -r -m` output (`6.18.57-2-lts x86_64`) into kernel release and machine.
fn parse_uname(output: Option<&str>) -> (Option<String>, Option<String>) {
  let mut fields = output.unwrap_or_default().split_whitespace();
  (
    fields.next().map(str::to_string),
    fields.next().map(str::to_string),
  )
}

/// Returns the GTK version from the first pkg-config file found.
fn gtk_version() -> Option<String> {
  ["gtk4", "gtk4.0", "gtk+-3.0"]
    .iter()
    .find_map(|name| pc_file_version(name))
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

/// Turns an empty string into `None`.
fn non_empty(value: String) -> Option<String> {
  (!value.is_empty()).then_some(value)
}

/// Retrieves data for `read_to_string` without mixing collection with TUI rendering. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn read_to_string(path: &str) -> Option<String> {
  fs::read_to_string(path).ok()
}

/// Returns the trimmed stdout of a command that exited successfully.
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

/// Returns stdout whatever the exit status, for tools that report partial results
/// with a failing status (`pacman -Q` when one of the packages is not installed).
pub(crate) fn command_stdout(command: &str, args: &[&str]) -> String {
  argvus_control_center_core::process::command(command)
    .args(args)
    .output()
    .map(|output| String::from_utf8_lossy(&output.stdout).into_owned())
    .unwrap_or_default()
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
  fn parses_versions_of_installed_packages_only() {
    let versions = parse_pacman_versions("argvus 0.4.0-1\nmesa 1:25.2.3-2\n");
    assert_eq!(versions.get("argvus").map(String::as_str), Some("0.4.0"));
    assert_eq!(versions.get("mesa").map(String::as_str), Some("25.2.3"));
    assert_eq!(versions.get("wayland"), None);
  }

  #[test]
  fn queries_every_versioned_package_in_one_call() {
    assert_eq!(
      pacman_query_args(),
      [
        "-Q",
        "argvus",
        "hyprland",
        "wayland",
        "mesa",
        "pipewire",
        "quickshell",
        "greetd",
        "systemd",
        "grub"
      ]
    );
  }

  #[test]
  fn splits_kernel_release_and_machine() {
    let (kernel, machine) = parse_uname(Some("6.18.57-2-lts x86_64"));
    assert_eq!(kernel.as_deref(), Some("6.18.57-2-lts"));
    assert_eq!(machine.as_deref(), Some("x86_64"));
    assert_eq!(parse_uname(None), (None, None));
  }
}
