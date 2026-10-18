//! Human-readable formatting shared by the Control Center domain pages.

/// Formats a byte count with the largest binary unit that keeps it at or above
/// one, using one decimal for TiB and GiB and whole numbers below that.
pub fn human_bytes(bytes: u64) -> String {
  /// Defines the constant `KIB`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
  const KIB: u64 = 1024;
  /// Defines the constant `MIB`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
  const MIB: u64 = 1024 * 1024;
  /// Defines the constant `GIB`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
  const GIB: u64 = 1024 * 1024 * 1024;
  /// Defines the constant `TIB`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
  const TIB: u64 = 1024 * 1024 * 1024 * 1024;
  if bytes >= TIB {
    format!("{:.1} TiB", bytes as f64 / TIB as f64)
  } else if bytes >= GIB {
    format!("{:.1} GiB", bytes as f64 / GIB as f64)
  } else if bytes >= MIB {
    format!("{:.0} MiB", bytes as f64 / MIB as f64)
  } else if bytes >= KIB {
    format!("{:.0} KiB", bytes as f64 / KIB as f64)
  } else {
    format!("{bytes} B")
  }
}

#[cfg(test)]
mod tests {
  use super::human_bytes;

  #[test]
  fn picks_the_largest_unit_that_fits() {
    assert_eq!(human_bytes(0), "0 B");
    assert_eq!(human_bytes(1023), "1023 B");
    assert_eq!(human_bytes(1024), "1 KiB");
    assert_eq!(human_bytes(5 * 1024 * 1024), "5 MiB");
    assert_eq!(human_bytes(3 * 1024 * 1024 * 1024 / 2), "1.5 GiB");
    assert_eq!(human_bytes(2 * 1024 * 1024 * 1024 * 1024), "2.0 TiB");
  }
}
