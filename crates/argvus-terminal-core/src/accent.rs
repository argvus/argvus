//! The accent colour of a theme when the user has not chosen one: the value the
//! theme declares in its manifest, or the built-in table for the bundled themes.

/// The accent of a theme that declares none.
pub const FALLBACK_ACCENT: &str = "#3590bd";

/// The theme identifier of a theme name: the `-float` variant shares the accent
/// of its base theme.
pub fn theme_id(theme: &str) -> &str {
  theme.strip_suffix("-float").unwrap_or(theme)
}

/// The top-level `accent = "#RRGGBB"` of a `theme.toml`, read up to its first
/// table header. The colour must be followed by the closing quote.
pub fn from_manifest(text: &str) -> Option<String> {
  for line in text.lines() {
    if line.starts_with('[') {
      break;
    }
    let Some(rest) = line.strip_prefix("accent = \"#") else {
      continue;
    };
    let digits = rest.as_bytes();
    if digits.len() > 6 && digits[6] == b'"' && digits[..6].iter().all(u8::is_ascii_hexdigit) {
      return Some(format!("#{rest}", rest = &rest[..6]));
    }
  }
  None
}

/// The built-in accent of a bundled theme, or the fallback accent.
pub fn built_in(theme_id: &str) -> &'static str {
  match theme_id {
    "one-dark" => "#61AFEF",
    "dracula" => "#BD93F9",
    "argvus-dark" => "#3590bd",
    "silver-dark" => "#595959",
    "argvus-light" => "#181818",
    "github-light" => "#0969DA",
    "one-light" => "#4078F2",
    "everforest-light" => "#3A94C5",
    "solarized-light" => "#268BD2",
    "frost" => "#0969DA",
    "gruvbox-light" => "#458588",
    "slate-dark" => "#7391a5",
    "universe" => "#eeeeee",
    "gruvbox-high-dark" => "#D79921",
    "gruvbox-dark" => "#D4BE98",
    "tokyo-night" => "#7AA2F7",
    "solitude" => "#798186",
    "sunset" => "#E2BE8A",
    "matrix" => "#82FB9C",
    "monokai-dark" => "#78DCE8",
    _ => FALLBACK_ACCENT,
  }
}

#[cfg(test)]
mod tests {
  use super::{built_in, from_manifest, theme_id};

  #[test]
  fn float_variant_shares_the_base_accent() {
    assert_eq!(theme_id("dracula-float"), "dracula");
    assert_eq!(theme_id("dracula"), "dracula");
    assert_eq!(built_in(theme_id("dracula-float")), "#BD93F9");
    assert_eq!(built_in("unknown-theme"), "#3590bd");
  }

  #[test]
  fn manifest_accent_stops_at_the_first_table() {
    let manifest = "name = \"x\"\naccent = \"#AABBCC\"\n[colors]\naccent = \"#000000\"\n";
    assert_eq!(from_manifest(manifest).as_deref(), Some("#AABBCC"));
    assert_eq!(from_manifest("[colors]\naccent = \"#000000\"\n"), None);
    assert_eq!(from_manifest("accent = \"blue\"\n"), None);
    assert_eq!(from_manifest("accent = \"#AABBCCDD\"\n"), None);
  }
}
