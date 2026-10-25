//! Implements `gpu` responsibilities in crate `argvus about`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use std::fs;
use std::path::Path;

/// Kernel drivers whose userspace graphics stack is Mesa, so Mesa's version is the meaningful one.
const MESA_DRIVERS: [&str; 6] = ["amdgpu", "radeon", "i915", "xe", "nouveau", "virtio_gpu"];

/// Executes the `gpu_info` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn gpu_info(lspci_output: Option<&str>) -> Option<String> {
  if let Some(output) = lspci_output {
    let gpus = output
      .lines()
      .filter(|line| {
        line.contains("VGA compatible controller")
          || line.contains("3D controller")
          || line.contains("Display controller")
      })
      .filter_map(|line| line.split_once(": ").map(|(_, value)| value.to_string()))
      .collect::<Vec<_>>();
    if !gpus.is_empty() {
      return Some(gpus.join("\n"));
    }
  }

  let drm = Path::new("/sys/class/drm");
  if let Ok(entries) = fs::read_dir(drm) {
    let names = entries
      .flatten()
      .filter_map(|entry| entry.file_name().into_string().ok())
      .collect::<Vec<_>>()
      .join("\n");
    if let Some(cards) = cards_from_drm(&names) {
      return Some(cards);
    }
  }

  None
}

/// Executes the `cards_from_drm` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn cards_from_drm(drm_output: &str) -> Option<String> {
  let cards = drm_output
    .lines()
    .filter(|name| name.starts_with("card") && !name.contains('-'))
    .collect::<Vec<_>>();
  (!cards.is_empty()).then(|| cards.join(", "))
}

/// Lists the kernel driver bound to each DRM card (`/sys/class/drm/cardN/device/driver`), without repeats.
pub fn gpu_driver_names() -> Vec<String> {
  let Ok(entries) = fs::read_dir("/sys/class/drm") else {
    return Vec::new();
  };
  let mut cards = entries
    .flatten()
    .filter(|entry| {
      let name = entry.file_name();
      let name = name.to_string_lossy();
      name.starts_with("card") && !name.contains('-')
    })
    .collect::<Vec<_>>();
  cards.sort_by_key(|entry| entry.file_name());

  let mut drivers = Vec::new();
  for card in cards {
    let driver = fs::read_link(card.path().join("device/driver"))
      .ok()
      .and_then(|link| {
        link
          .file_name()
          .map(|name| name.to_string_lossy().into_owned())
      });
    if let Some(driver) = driver
      && !drivers.contains(&driver)
    {
      drivers.push(driver);
    }
  }
  drivers
}

/// Describes each driver with its version: the module version for out-of-tree drivers
/// such as NVIDIA (`/sys/module/<driver>/version`), or Mesa's version for in-kernel ones.
pub fn gpu_drivers(drivers: &[String], mesa_version: Option<&str>) -> Option<String> {
  let lines = drivers
    .iter()
    .map(|driver| {
      let module_version = fs::read_to_string(format!("/sys/module/{driver}/version"))
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
      describe_driver(driver, module_version.as_deref(), mesa_version)
    })
    .collect::<Vec<_>>();
  (!lines.is_empty()).then(|| lines.join("\n"))
}

/// Formats one driver line, such as `nvidia 580.82.07` or `amdgpu (Mesa 25.2.3)`.
pub fn describe_driver(
  driver: &str,
  module_version: Option<&str>,
  mesa_version: Option<&str>,
) -> String {
  match (module_version, mesa_version) {
    (Some(version), _) => format!("{driver} {version}"),
    (None, Some(mesa)) if MESA_DRIVERS.contains(&driver) => format!("{driver} (Mesa {mesa})"),
    _ => driver.to_string(),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  /// Defines the constant `LSPCI`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
  const LSPCI: &str = r#"00:02.0 VGA compatible controller: Intel Corporation Alder Lake
01:00.0 3D controller: NVIDIA Corporation GA107"#;

  #[test]
  /// Converts input data into `parses_lspci_graphics_lines` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn parses_lspci_graphics_lines() {
    assert_eq!(
      gpu_info(Some(LSPCI)).as_deref(),
      Some("Intel Corporation Alder Lake\nNVIDIA Corporation GA107")
    );
  }

  #[test]
  /// Executes the `falls_back_to_drm_cards` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn falls_back_to_drm_cards() {
    assert_eq!(
      cards_from_drm("card0\ncard1\n"),
      Some("card0, card1".into())
    );
    assert_eq!(cards_from_drm("card1-eDP-1\n"), None);
  }

  #[test]
  fn proprietary_driver_uses_module_version() {
    let line = describe_driver("nvidia", Some("580.82.07"), Some("25.2.3"));
    assert_eq!(line, "nvidia 580.82.07");
  }

  #[test]
  fn in_kernel_driver_uses_mesa_version() {
    assert_eq!(
      describe_driver("amdgpu", None, Some("25.2.3")),
      "amdgpu (Mesa 25.2.3)"
    );
    assert_eq!(describe_driver("i915", None, None), "i915");
  }

  #[test]
  fn unknown_driver_without_version_shows_only_its_name() {
    assert_eq!(
      describe_driver("simpledrm", None, Some("25.2.3")),
      "simpledrm"
    );
  }
}
