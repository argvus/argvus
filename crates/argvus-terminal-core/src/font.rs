//! The font of each profile. The desktop stores the chosen families and sizes in
//! `generated/fonts.conf`; a missing key falls back to the earlier name, then to
//! the built-in default.

/// The font of a profile: a family and a size.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Font {
  pub family: String,
  pub size: String,
}

/// The value of the first `key=value` line of the fonts file that names `key`.
/// An empty value counts as missing, as the launcher reads it.
pub fn state_value(fonts: Option<&str>, key: &str) -> Option<String> {
  let prefix = format!("{key}=");
  fonts?
    .lines()
    .find_map(|line| line.strip_prefix(prefix.as_str()))
    .filter(|value| !value.is_empty())
    .map(String::from)
}

/// The family and size of a font profile, read from the fonts file.
pub fn for_profile(profile: &str, fonts: Option<&str>) -> Font {
  let (family_keys, size_keys, default_size) = if profile == "apps" {
    (
      ["apps_family", "default_family"],
      ["apps_size", "default_size"],
      "12",
    )
  } else {
    (
      ["terminal_family", "monospace_family"],
      ["terminal_size", "monospace_size"],
      "13",
    )
  };
  let family = state_value(fonts, family_keys[0])
    .or_else(|| state_value(fonts, family_keys[1]))
    .unwrap_or_else(|| "IBM Plex Mono".to_string());
  let size = state_value(fonts, size_keys[0])
    .or_else(|| state_value(fonts, size_keys[1]))
    .unwrap_or_else(|| default_size.to_string());
  Font { family, size }
}

#[cfg(test)]
mod tests {
  use super::{for_profile, state_value};

  #[test]
  fn profile_uses_its_keys_then_the_earlier_names() {
    let fonts = "monospace_family=JetBrains Mono\nmonospace_size=14\napps_family=\n";
    assert_eq!(
      for_profile("terminal", Some(fonts)).family,
      "JetBrains Mono"
    );
    assert_eq!(for_profile("terminal", Some(fonts)).size, "14");
    assert_eq!(for_profile("apps", Some(fonts)).family, "IBM Plex Mono");
    assert_eq!(for_profile("apps", Some(fonts)).size, "12");
    assert_eq!(for_profile("terminal", None).size, "13");
  }

  #[test]
  fn empty_value_is_missing() {
    assert_eq!(state_value(Some("terminal_size=\n"), "terminal_size"), None);
  }
}
