//! GPU slot selection for the supported btop version.

use std::fmt::Write;

/// The number of GPU boxes supported by btop 1.4.7.
pub const MAX_GPU_BOXES: usize = 6;

/// Counts supported display-class devices, capped to btop's six GPU slots.
pub fn supported_gpu_count(devices: impl IntoIterator<Item = (String, String)>) -> usize {
  devices
    .into_iter()
    .filter(|(vendor, class)| {
      matches!(vendor.as_str(), "0x10de" | "0x1002" | "0x8086") && class.starts_with("0x03")
    })
    .count()
    .min(MAX_GPU_BOXES)
}

/// The `shown_boxes` value for the detected number of GPUs.
pub fn shown_boxes(count: usize) -> Option<String> {
  let count = count.min(MAX_GPU_BOXES);
  if count == 0 {
    return None;
  }
  let mut boxes = String::with_capacity(count * 5);
  for index in 0..count {
    if index > 0 {
      boxes.push(' ');
    }
    let _ = write!(boxes, "gpu{index}");
  }
  Some(boxes)
}

#[cfg(test)]
mod tests {
  use super::{MAX_GPU_BOXES, shown_boxes, supported_gpu_count};

  #[test]
  fn selects_all_recognized_vendors_and_ignores_other_pci_devices() {
    assert_eq!(
      supported_gpu_count([
        ("0x8086".into(), "0x030000".into()),
        ("0x10de".into(), "0x030200".into()),
        ("0x1002".into(), "0x030000".into()),
        ("0x1234".into(), "0x030000".into()),
        ("0x8086".into(), "0x060000".into()),
      ]),
      3
    );
  }

  #[test]
  fn configures_zero_one_multiple_and_the_supported_maximum_slots() {
    assert_eq!(shown_boxes(0), None);
    assert_eq!(shown_boxes(1).as_deref(), Some("gpu0"));
    assert_eq!(shown_boxes(3).as_deref(), Some("gpu0 gpu1 gpu2"));
    assert_eq!(
      shown_boxes(9).as_deref(),
      Some("gpu0 gpu1 gpu2 gpu3 gpu4 gpu5")
    );
    assert_eq!(MAX_GPU_BOXES, 6);
  }
}
