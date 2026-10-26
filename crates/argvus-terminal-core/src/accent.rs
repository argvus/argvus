//! The accent colour of a theme when the user has not chosen one.

/// The accent of a theme that declares none.
pub const FALLBACK_ACCENT: &str = "#3590bd";

/// The theme identifier of a theme name: the `-float` variant shares the accent
/// of its base theme.
pub fn theme_id(theme: &str) -> &str {
  theme.strip_suffix("-float").unwrap_or(theme)
}

#[cfg(test)]
mod tests {
  use super::theme_id;

  #[test]
  fn float_variant_shares_the_base_theme() {
    assert_eq!(theme_id("dracula-float"), "dracula");
    assert_eq!(theme_id("dracula"), "dracula");
  }
}
