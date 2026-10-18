//! The theme and the accent of the monitor, and the btop theme file with the
//! accent applied.

/// The theme used when the selected one does not exist.
pub const DEFAULT_THEME: &str = "argvus-dark";

/// The btop theme keys that take the accent colour.
const ACCENT_KEYS: [&str; 14] = [
  "title",
  "hi_fg",
  "proc_misc",
  "cpu_box",
  "mem_box",
  "net_box",
  "proc_box",
  "temp_start",
  "cpu_mid",
  "free_mid",
  "available_mid",
  "download_mid",
  "upload_mid",
  "process_mid",
];

/// The selected theme: the name, and whether the theme file was found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Selection {
  Found(String),
  Missing { requested: String },
}

/// Chooses the theme: the canonical value first, then the fallback file, then
/// the default. A theme whose file does not exist falls back to the default.
pub fn select_theme(
  canonical: Option<&str>,
  fallback: Option<&str>,
  exists: impl Fn(&str) -> bool,
) -> Selection {
  let requested = canonical
    .filter(|value| !value.is_empty())
    .or(fallback.filter(|value| !value.is_empty()))
    .unwrap_or(DEFAULT_THEME);
  if exists(requested) {
    Selection::Found(requested.to_string())
  } else {
    Selection::Missing {
      requested: requested.to_string(),
    }
  }
}

/// The accent as `#RRGGBB` in upper case, or `None` when the text is not a
/// six-digit hex colour.
pub fn normalize_accent(text: &str) -> Option<String> {
  let digits = text.trim().strip_prefix('#')?;
  if digits.len() != 6 || !digits.bytes().all(|byte| byte.is_ascii_hexdigit()) {
    return None;
  }
  Some(format!("#{}", digits.to_ascii_uppercase()))
}

/// The text colour on top of an accent: black on light accents, white on dark.
pub fn foreground_for(accent: &str) -> &'static str {
  let digits = accent.trim_start_matches('#');
  let channel =
    |range: std::ops::Range<usize>| u32::from_str_radix(&digits[range], 16).unwrap_or(0);
  let luminance = (299 * channel(0..2) + 587 * channel(2..4) + 114 * channel(4..6)) / 1000;
  if luminance >= 128 {
    "#000000"
  } else {
    "#FFFFFF"
  }
}

/// Replaces the value of the first `key="..."` on a line with `value`. A line
/// without a quoted value is returned unchanged.
pub fn replace_quoted_value(line: &str, value: &str) -> String {
  let Some(start) = line.find("=\"") else {
    return line.to_string();
  };
  let content_start = start + 2;
  let Some(length) = line[content_start..].find('"') else {
    return line.to_string();
  };
  let end = content_start + length + 1;
  format!("{}=\"{value}\"{}", &line[..start], &line[end..])
}

/// The btop theme file with the accent applied. Without an accent the file is
/// returned as it is.
pub fn apply_accent(source: &str, accent: Option<&str>) -> String {
  let Some(accent) = accent else {
    return source.to_string();
  };
  let foreground = foreground_for(accent);
  let mut output = String::with_capacity(source.len());
  for line in source.lines() {
    let key = theme_key(line);
    let replaced = if key == "selected_fg" {
      replace_quoted_value(line, foreground)
    } else if ACCENT_KEYS.contains(&key) {
      replace_quoted_value(line, accent)
    } else {
      line.to_string()
    };
    output.push_str(&replaced);
    output.push('\n');
  }
  output
}

/// The key of a `theme[key]="value"` line: the text between `theme[` and the
/// first `]`. A line without that shape gives the text before its first `]`.
fn theme_key(line: &str) -> &str {
  let rest = line.strip_prefix("theme[").unwrap_or(line);
  rest.split(']').next().unwrap_or("")
}

#[cfg(test)]
mod tests {
  use super::{
    Selection, apply_accent, foreground_for, normalize_accent, replace_quoted_value, select_theme,
  };

  #[test]
  fn theme_prefers_the_canonical_value_then_the_file() {
    let exists = |name: &str| name != "missing";
    assert_eq!(
      select_theme(Some("dracula"), Some("nord"), exists),
      Selection::Found("dracula".into())
    );
    assert_eq!(
      select_theme(None, Some("nord"), exists),
      Selection::Found("nord".into())
    );
    assert_eq!(
      select_theme(Some(""), None, exists),
      Selection::Found("argvus-dark".into())
    );
    assert_eq!(
      select_theme(Some("missing"), None, exists),
      Selection::Missing {
        requested: "missing".into()
      }
    );
  }

  #[test]
  fn accent_must_be_six_hex_digits_and_is_upper_case() {
    assert_eq!(normalize_accent(" #3590bd "), Some("#3590BD".into()));
    assert_eq!(normalize_accent("3590BD"), None);
    assert_eq!(normalize_accent("#3590B"), None);
    assert_eq!(normalize_accent("null"), None);
  }

  #[test]
  fn foreground_contrasts_with_the_accent() {
    assert_eq!(foreground_for("#FFFFFF"), "#000000");
    assert_eq!(foreground_for("#3590BD"), "#FFFFFF");
    assert_eq!(foreground_for("#F0E68C"), "#000000");
  }

  #[test]
  fn quoted_value_is_replaced_once() {
    assert_eq!(
      replace_quoted_value("theme[title]=\"#111111\"", "#3590BD"),
      "theme[title]=\"#3590BD\""
    );
    assert_eq!(
      replace_quoted_value("theme[title]=none", "#3590BD"),
      "theme[title]=none"
    );
  }

  #[test]
  fn accent_changes_only_the_accent_keys() {
    let source =
      "theme[main_bg]=\"#000000\"\ntheme[title]=\"#111111\"\ntheme[selected_fg]=\"#222222\"\n";
    assert_eq!(
      apply_accent(source, Some("#3590BD")),
      "theme[main_bg]=\"#000000\"\ntheme[title]=\"#3590BD\"\ntheme[selected_fg]=\"#FFFFFF\"\n"
    );
    assert_eq!(apply_accent(source, None), source);
  }
}
