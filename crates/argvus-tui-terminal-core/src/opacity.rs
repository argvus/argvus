//! The Kitty background opacity of the Control Center. Transparency takes
//! effect only when both transparency and blur are enabled.

/// The opacity for the given effect settings, printed with two decimals.
/// `transparency_value` is a percentage; a missing or out-of-range value is 50.
pub fn background_opacity(
  transparency_enabled: &str,
  transparency_value: &str,
  blur_enabled: &str,
) -> String {
  if transparency_enabled != "true" || blur_enabled != "true" {
    return "1.00".to_string();
  }
  let value = if transparency_value.is_empty()
    || !transparency_value.bytes().all(|byte| byte.is_ascii_digit())
  {
    50
  } else {
    transparency_value.parse::<u32>().unwrap_or(50)
  };
  let value = if value <= 100 { value } else { 50 };
  format!("{:.2}", 1.0 - f64::from(value) / 100.0)
}

#[cfg(test)]
mod tests {
  use super::background_opacity;

  #[test]
  fn transparency_needs_both_effects() {
    assert_eq!(background_opacity("true", "50", "true"), "0.50");
    assert_eq!(background_opacity("true", "50", "false"), "1.00");
    assert_eq!(background_opacity("false", "50", "true"), "1.00");
  }

  #[test]
  fn a_bad_value_is_fifty_percent() {
    assert_eq!(background_opacity("true", "", "true"), "0.50");
    assert_eq!(background_opacity("true", "abc", "true"), "0.50");
    assert_eq!(background_opacity("true", "150", "true"), "0.50");
    assert_eq!(background_opacity("true", "20", "true"), "0.80");
  }
}
