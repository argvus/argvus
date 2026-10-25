//! Reads the computer and motherboard identification exposed by DMI/SMBIOS.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use std::fs;

use argvus_control_center_core::sanitize::terminal_text;

/// Values firmware vendors leave in DMI fields they never filled in.
const DMI_PLACEHOLDERS: [&str; 8] = [
  "to be filled by o.e.m.",
  "default string",
  "system product name",
  "system manufacturer",
  "not applicable",
  "not specified",
  "none",
  "o.e.m.",
];

/// Returns `<sys_vendor> <product_name>`, the model of the computer.
pub fn computer_model() -> Option<String> {
  vendor_and_name(
    read_dmi("sys_vendor").as_deref(),
    read_dmi("product_name").as_deref(),
  )
}

/// Returns `<board_vendor> <board_name>`, the motherboard.
pub fn motherboard() -> Option<String> {
  vendor_and_name(
    read_dmi("board_vendor").as_deref(),
    read_dmi("board_name").as_deref(),
  )
}

/// Joins a vendor and a name, dropping placeholders and a vendor repeated in the name.
pub fn vendor_and_name(vendor: Option<&str>, name: Option<&str>) -> Option<String> {
  let vendor = vendor.filter(|value| is_meaningful(value));
  let name = name.filter(|value| is_meaningful(value));
  match (vendor, name) {
    (Some(vendor), Some(name)) if name.starts_with(vendor) => Some(name.to_string()),
    (Some(vendor), Some(name)) => Some(format!("{vendor} {name}")),
    (Some(only), None) | (None, Some(only)) => Some(only.to_string()),
    (None, None) => None,
  }
}

/// Reads one field of `/sys/class/dmi/id`, sanitized for the terminal.
pub(super) fn read_dmi(field: &str) -> Option<String> {
  fs::read_to_string(format!("/sys/class/dmi/id/{field}"))
    .ok()
    .map(|value| terminal_text(value.trim()))
    .filter(|value| !value.is_empty())
}

/// Tells whether a DMI value carries real information.
fn is_meaningful(value: &str) -> bool {
  let value = value.trim();
  !value.is_empty() && !DMI_PLACEHOLDERS.contains(&value.to_ascii_lowercase().as_str())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn joins_vendor_and_board_name() {
    let board = vendor_and_name(Some("ASUSTeK COMPUTER INC."), Some("PRIME B550M-A"));
    assert_eq!(
      board.as_deref(),
      Some("ASUSTeK COMPUTER INC. PRIME B550M-A")
    );
  }

  #[test]
  fn drops_oem_placeholders() {
    let model = vendor_and_name(Some("To Be Filled By O.E.M."), Some("B450M Pro4"));
    assert_eq!(model.as_deref(), Some("B450M Pro4"));
    assert_eq!(
      vendor_and_name(Some("Default string"), Some("Default string")),
      None
    );
  }

  #[test]
  fn does_not_repeat_vendor_already_in_name() {
    let model = vendor_and_name(Some("LENOVO"), Some("LENOVO 21AH"));
    assert_eq!(model.as_deref(), Some("LENOVO 21AH"));
  }
}
