//! Implements `capabilities` responsibilities in crate `argvus control center core`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// Represents `Capabilities`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct Capabilities {
  pub has_lspci: bool,
  pub has_glxinfo: bool,
  pub has_eglinfo: bool,
  pub has_vulkaninfo: bool,
  pub has_systemd: bool,
  pub has_journal: bool,
  pub has_loginctl: bool,
  pub has_hypridle: bool,
  pub has_hyprctl: bool,
  pub has_sessionctl: bool,
  pub has_networkmanager: bool,
  pub has_nmcli: bool,
  pub has_iwd: bool,
  pub has_resolvectl: bool,
  pub has_ip: bool,
  pub has_wireguard_tools: bool,
  pub has_bluetooth: bool,
  pub has_bluetoothctl: bool,
  pub has_pipewire: bool,
  pub has_wpctl: bool,
  pub has_pactl: bool,
  pub has_wireplumber: bool,
  pub has_power_profiles_daemon: bool,
  pub has_tlp: bool,
  pub has_cpupower: bool,
  pub has_reflector: bool,
  pub has_pacman: bool,
  pub has_smartctl: bool,
  pub has_lsblk: bool,
  pub has_findmnt: bool,
  pub has_swapon: bool,
  pub has_nvme: bool,
  pub has_lvm: bool,
  pub has_cryptsetup: bool,
  pub has_plymouth: bool,
  pub has_mkinitcpio: bool,
  pub has_systemd_boot: bool,
  pub has_grub: bool,
  pub has_btrfs: bool,
  pub has_snapper: bool,
  pub has_paru: bool,
  pub has_yay: bool,
  pub is_uefi: bool,
  pub has_battery: bool,
  pub is_virtual_machine: bool,
  pub has_podman: bool,
  pub has_docker: bool,
}

impl Capabilities {
  /// Retrieves data for `detect` without mixing collection with TUI rendering. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn detect() -> Self {
    Self {
      has_lspci: executable("lspci"),
      has_glxinfo: executable("glxinfo"),
      has_eglinfo: executable("eglinfo"),
      has_vulkaninfo: executable("vulkaninfo"),
      has_systemd: executable("systemctl"),
      has_journal: executable("journalctl"),
      has_loginctl: executable("loginctl"),
      has_hypridle: executable("hypridle"),
      has_hyprctl: executable("hyprctl"),
      has_sessionctl: executable("argvus"),
      has_networkmanager: executable("nmcli"),
      has_nmcli: executable("nmcli"),
      has_iwd: executable("iwctl"),
      has_resolvectl: executable("resolvectl"),
      has_ip: executable("ip"),
      has_wireguard_tools: executable("wg"),
      has_bluetooth: bluetooth_adapter_present(),
      has_bluetoothctl: executable("bluetoothctl"),
      has_pipewire: executable("wpctl") || executable("pipewire"),
      has_wpctl: executable("wpctl"),
      has_pactl: executable("pactl"),
      has_wireplumber: executable("wireplumber") || executable("wpctl"),
      has_power_profiles_daemon: executable("powerprofilesctl"),
      has_tlp: executable("tlp-stat"),
      has_cpupower: executable("cpupower"),
      has_reflector: executable("reflector"),
      has_pacman: executable("pacman"),
      has_smartctl: executable("smartctl"),
      has_lsblk: executable("lsblk"),
      has_findmnt: executable("findmnt"),
      has_swapon: executable("swapon"),
      has_nvme: executable("nvme"),
      has_lvm: executable("lvs") || executable("pvs"),
      has_cryptsetup: executable("cryptsetup"),
      has_plymouth: executable("plymouth") || path_exists("/usr/lib/plymouth"),
      has_mkinitcpio: executable("mkinitcpio"),
      has_systemd_boot: executable("bootctl") && path_exists("/sys/firmware/efi"),
      has_grub: executable("grub-mkconfig") || path_exists("/etc/default/grub"),
      has_btrfs: executable("btrfs"),
      has_snapper: executable("snapper"),
      has_paru: executable("paru"),
      has_yay: executable("yay"),
      is_uefi: path_exists("/sys/firmware/efi"),
      has_battery: battery_present(),
      is_virtual_machine: virtual_machine(),
      has_podman: executable("podman"),
      has_docker: executable("docker"),
    }
  }
}

/// Executes the `virtual_machine` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn virtual_machine() -> bool {
  [
    "/sys/class/dmi/id/product_name",
    "/sys/class/dmi/id/sys_vendor",
  ]
  .into_iter()
  .filter_map(|path| std::fs::read_to_string(path).ok())
  .map(|value| value.to_ascii_lowercase())
  .any(|value| {
    ["virtualbox", "vmware", "qemu", "kvm", "microsoft", "xen"]
      .iter()
      .any(|name| value.contains(name))
  })
}

/// Executes the `executable` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn executable(name: &str) -> bool {
  resolve_executable(name).is_some()
}

/// Returns the absolute path of an executable, searching first the inherited
/// `PATH` and then the standard system binary directories. Desktop sessions
/// (Hyprland autostarts, systemd units, etc.) sometimes launch the UI without
/// `/usr/bin` on `PATH`, so capability detection and command execution must
/// not rely solely on the inherited environment. This mirrors the equivalent
/// fix applied in argvus-accounts.
pub fn resolve_executable(name: &str) -> Option<PathBuf> {
  std::env::var_os("PATH")
    .map(|path| std::env::split_paths(&path).collect::<Vec<_>>())
    .into_iter()
    .flatten()
    .chain(STANDARD_BIN_DIRS.iter().map(PathBuf::from))
    .map(|dir| dir.join(name))
    .find(|candidate| candidate.is_file())
}

/// Mirrors the `PATH` assembled by `/etc/profile` on Arch so binaries in
/// `/usr/bin` are found even when the launching session stripped `PATH`.
const STANDARD_BIN_DIRS: &[&str] = &[
  "/usr/local/sbin",
  "/usr/local/bin",
  "/usr/bin",
  "/usr/sbin",
  "/bin",
  "/sbin",
];

/// Executes the `path_exists` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn path_exists(path: &str) -> bool {
  Path::new(path).exists()
}

/// Executes the `battery_present` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn battery_present() -> bool {
  std::fs::read_dir("/sys/class/power_supply")
    .ok()
    .into_iter()
    .flatten()
    .filter_map(Result::ok)
    .any(|entry| entry.file_name().to_string_lossy().starts_with("BAT"))
}

/// Executes the `bluetooth_adapter_present` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn bluetooth_adapter_present() -> bool {
  std::fs::read_dir("/sys/class/bluetooth")
    .ok()
    .into_iter()
    .flatten()
    .filter_map(Result::ok)
    .any(|entry| {
      let name = entry.file_name().to_string_lossy().into_owned();
      name.starts_with("hci") && entry.path().is_dir()
    })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  /// Executes the `default_capabilities_are_safe_and_false` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn default_capabilities_are_safe_and_false() {
    let capabilities = Capabilities::default();
    assert!(!capabilities.has_battery);
    assert!(!capabilities.is_uefi);
  }
}
