//! Reads how the machine booted: firmware, boot mode, Secure Boot and virtualization.
use std::fs;
use std::path::Path;

use argvus_control_center_core::process;
use argvus_control_center_core::sanitize::terminal_text;

use super::board::read_dmi;

/// Directory that exists only when the kernel was started by UEFI firmware.
const EFI_DIR: &str = "/sys/firmware/efi";
/// EFI variable set by bootloaders that implement the Boot Loader Interface
/// (systemd-boot, Limine and others) with their name and version.
const LOADER_INFO_VAR: &str =
  "/sys/firmware/efi/efivars/LoaderInfo-4a67b082-0a4c-41cf-b6c7-440b29bb8c4f";
/// GRUB installs its modules and configuration in one of these directories.
const GRUB_DIRS: [&str; 3] = ["/boot/grub", "/efi/grub", "/boot/efi/grub"];
/// EFI variable holding the Secure Boot state (global variable GUID).
const SECURE_BOOT_VAR: &str =
  "/sys/firmware/efi/efivars/SecureBoot-8be4df61-93ca-11d2-aa0d-00e098032b8c";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Firmware interface that started the kernel.
pub enum BootMode {
  Uefi,
  Legacy,
}

/// Returns the firmware vendor, version and release date, such as `American Megatrends 1.40 (03/15/2024)`.
pub fn firmware() -> Option<String> {
  firmware_label(
    read_dmi("bios_vendor").as_deref(),
    read_dmi("bios_version").as_deref(),
    read_dmi("bios_date").as_deref(),
  )
}

/// Joins the firmware fields that are present; the date goes in parentheses.
pub fn firmware_label(
  vendor: Option<&str>,
  version: Option<&str>,
  date: Option<&str>,
) -> Option<String> {
  let name = [vendor, version]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(" ");
  match (name.is_empty(), date) {
    (true, None) => None,
    (true, Some(date)) => Some(date.to_string()),
    (false, None) => Some(name),
    (false, Some(date)) => Some(format!("{name} ({date})")),
  }
}

/// Detects the boot mode from the presence of the EFI sysfs directory.
pub fn boot_mode() -> BootMode {
  if Path::new(EFI_DIR).exists() {
    BootMode::Uefi
  } else {
    BootMode::Legacy
  }
}

/// Returns the Secure Boot state, or `None` when the firmware does not expose it (Legacy boot included).
pub fn secure_boot() -> Option<bool> {
  fs::read(SECURE_BOOT_VAR)
    .ok()
    .and_then(|bytes| secure_boot_from_efivar(&bytes))
}

/// Decodes an efivarfs file: 4 attribute bytes followed by the 1-byte value.
pub fn secure_boot_from_efivar(bytes: &[u8]) -> Option<bool> {
  bytes.get(4).map(|value| *value == 1)
}

/// Identifies the bootloader without privileges, such as `systemd-boot 258.1` or `GRUB 2.12`.
///
/// Bootloaders that implement the Boot Loader Interface publish their name and version in
/// `LoaderInfo`. GRUB does not, so it is recognized by its directory and reported with the
/// version of the installed `grub` package. Anything else is unknown.
pub fn bootloader(grub_version: Option<&str>) -> Option<String> {
  if let Some(info) = fs::read(LOADER_INFO_VAR)
    .ok()
    .and_then(|bytes| loader_info_from_efivar(&bytes))
  {
    return Some(info);
  }
  GRUB_DIRS
    .iter()
    .any(|dir| Path::new(dir).is_dir())
    .then(|| match grub_version {
      Some(version) => format!("GRUB {version}"),
      None => "GRUB".to_string(),
    })
}

/// Decodes `LoaderInfo`: 4 attribute bytes, then a NUL-terminated UTF-16LE string.
pub fn loader_info_from_efivar(bytes: &[u8]) -> Option<String> {
  let units = bytes
    .get(4..)?
    .chunks_exact(2)
    .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
    .take_while(|unit| *unit != 0)
    .collect::<Vec<_>>();
  let text = String::from_utf16(&units).ok()?;
  let text = terminal_text(text.trim());
  (!text.is_empty()).then_some(text)
}

/// Returns the hypervisor reported by `systemd-detect-virt`, such as `kvm` or `none`.
///
/// The tool exits with status 1 on bare metal while still printing `none`, so the exit
/// status is ignored and only the printed value is used.
pub fn virtualization() -> Option<String> {
  let output = process::command("systemd-detect-virt").output().ok()?;
  parse_detect_virt(&String::from_utf8_lossy(&output.stdout))
}

/// Takes the first non-empty line printed by `systemd-detect-virt`.
pub fn parse_detect_virt(output: &str) -> Option<String> {
  output
    .lines()
    .map(str::trim)
    .find(|line| !line.is_empty())
    .map(str::to_string)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn firmware_label_joins_vendor_version_and_date() {
    let label = firmware_label(Some("AMI"), Some("1.40"), Some("03/15/2024"));
    assert_eq!(label.as_deref(), Some("AMI 1.40 (03/15/2024)"));
  }

  #[test]
  fn firmware_label_skips_missing_fields() {
    assert_eq!(
      firmware_label(None, Some("1.40"), None).as_deref(),
      Some("1.40")
    );
    assert_eq!(firmware_label(None, None, None), None);
  }

  #[test]
  fn decodes_secure_boot_state_after_attribute_bytes() {
    assert_eq!(secure_boot_from_efivar(&[6, 0, 0, 0, 1]), Some(true));
    assert_eq!(secure_boot_from_efivar(&[6, 0, 0, 0, 0]), Some(false));
    assert_eq!(secure_boot_from_efivar(&[6, 0, 0]), None);
  }

  /// Encodes a `LoaderInfo` efivar: attribute bytes, UTF-16LE text and the NUL terminator.
  fn loader_info(text: &str) -> Vec<u8> {
    let mut bytes = vec![6, 0, 0, 0];
    for unit in text.encode_utf16().chain([0]) {
      bytes.extend(unit.to_le_bytes());
    }
    bytes
  }

  #[test]
  fn decodes_bootloader_name_and_version() {
    assert_eq!(
      loader_info_from_efivar(&loader_info("systemd-boot 258.1-1-arch")).as_deref(),
      Some("systemd-boot 258.1-1-arch")
    );
    assert_eq!(
      loader_info_from_efivar(&loader_info("Limine 9.3.0")).as_deref(),
      Some("Limine 9.3.0")
    );
  }

  #[test]
  fn rejects_empty_or_truncated_loader_info() {
    assert_eq!(loader_info_from_efivar(&loader_info("")), None);
    assert_eq!(loader_info_from_efivar(&[6, 0]), None);
  }

  #[test]
  fn reads_hypervisor_name_from_detect_virt() {
    assert_eq!(parse_detect_virt("kvm\n").as_deref(), Some("kvm"));
    assert_eq!(parse_detect_virt("none\n").as_deref(), Some("none"));
    assert_eq!(parse_detect_virt(""), None);
  }
}
