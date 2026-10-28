//! Font discovery and configuration of the Settings pages.
use std::collections::BTreeSet;

use crate::error::SettingsError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontEntry {
  pub family: String,
  pub style: String,
}

impl FontEntry {
  pub fn display_name(&self) -> String {
    if self.style.eq_ignore_ascii_case("regular") || self.style.is_empty() {
      self.family.clone()
    } else {
      format!("{} {}", self.family, self.style)
    }
  }
}

pub fn list() -> Result<Vec<FontEntry>, SettingsError> {
  let output = argvus_control_center_core::process::command("fc-list")
    .args(["--format", "%{family}\t%{style}\n"])
    .output()
    .map_err(|error| SettingsError::FontDiscovery(format!("fontconfig unavailable: {error}")))?;
  if !output.status.success() {
    return Err(SettingsError::FontDiscovery(format!(
      "fc-list exited with {}",
      output.status
    )));
  }

  let mut entries = BTreeSet::new();
  for line in String::from_utf8_lossy(&output.stdout).lines() {
    let (families, styles) = line.split_once('\t').unwrap_or((line, "Regular"));
    let style = styles.split(',').next().unwrap_or("Regular").trim();
    for family in families.split(',') {
      let family = family.trim();
      if !family.is_empty() {
        entries.insert((family.to_string(), style.to_string()));
      }
    }
  }
  if entries.is_empty() {
    return Err(SettingsError::FontDiscovery(
      "fontconfig returned no font families".to_string(),
    ));
  }
  Ok(
    entries
      .into_iter()
      .map(|(family, style)| FontEntry { family, style })
      .collect(),
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn regular_style_uses_family_only() {
    let font = FontEntry {
      family: "Noto Sans".into(),
      style: "Regular".into(),
    };
    assert_eq!(font.display_name(), "Noto Sans");
  }

  #[test]
  fn named_style_is_visible() {
    let font = FontEntry {
      family: "Noto Sans".into(),
      style: "Bold".into(),
    };
    assert_eq!(font.display_name(), "Noto Sans Bold");
  }
}
