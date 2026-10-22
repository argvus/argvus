//! The background opacity of the terminal. With transparency off, or with no
//! value, the terminal keeps the theme's own background.

/// What the launcher knows about the transparency switch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transparency {
  /// Whether `argvus config` answered for the settings.
  pub argvus_present: bool,
  /// `true`, `false`, or another word from the configuration; empty when unknown.
  pub enabled: String,
  /// The transparency percent; empty when unknown.
  pub value: String,
}

/// The `background_opacity` the terminal gets, or `None` when the theme keeps its
/// own. Transparency is on unless the setting says otherwise, on virtual
/// machines and low-power sessions too.
pub fn opacity(state: &Transparency) -> Option<String> {
  let explicit = state.argvus_present
    && matches!(state.enabled.as_str(), "true" | "false")
    && state.value.bytes().all(|byte| byte.is_ascii_digit());
  if explicit {
    return Some(if state.enabled == "true" {
      let percent: f64 = state.value.parse().unwrap_or(0.0);
      format!("{:.2}", 1.0 - percent / 100.0)
    } else {
      "1.0".to_string()
    });
  }
  if enabled_by_default(state) {
    None
  } else {
    Some("1.0".to_string())
  }
}

fn enabled_by_default(state: &Transparency) -> bool {
  match state.enabled.as_str() {
    "true" | "enabled" => true,
    "false" | "disabled" => false,
    _ => true,
  }
}

#[cfg(test)]
mod tests {
  use super::{Transparency, opacity};

  fn state(enabled: &str, value: &str) -> Transparency {
    Transparency {
      argvus_present: true,
      enabled: enabled.into(),
      value: value.into(),
    }
  }

  #[test]
  fn enabled_transparency_uses_the_percent() {
    assert_eq!(opacity(&state("true", "25")).as_deref(), Some("0.75"));
    assert_eq!(opacity(&state("false", "25")).as_deref(), Some("1.0"));
  }

  #[test]
  fn unknown_setting_keeps_transparency_on() {
    assert_eq!(opacity(&state("", "")), None);
    assert_eq!(opacity(&state("enabled", "x")), None);
  }
}
