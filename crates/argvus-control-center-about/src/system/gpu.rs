//! Implements `gpu` responsibilities in crate `argvus about`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use std::fs;
use std::path::Path;

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
}
