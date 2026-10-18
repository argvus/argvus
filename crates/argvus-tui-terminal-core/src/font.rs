//! The font values of the generated fonts file. Each value has a fallback
//! chain; a key with an empty value counts as missing, as the launcher did.

use crate::profile::{FontProfile, Selection};

/// The generated fonts file as `key=value` lines.
pub struct FontState {
  text: Option<String>,
}

impl FontState {
  /// The state of a file that may be missing; a missing file has no values.
  pub fn new(text: Option<String>) -> Self {
    Self { text }
  }

  /// The first `key=value` line of the file. An empty value is no value.
  pub fn value(&self, key: &str) -> Option<String> {
    let prefix = format!("{key}=");
    self
      .text
      .as_deref()?
      .lines()
      .find_map(|line| line.strip_prefix(&prefix))
      .filter(|value| !value.is_empty())
      .map(str::to_string)
  }

  fn first(&self, keys: &[&str], fallback: &str) -> String {
    keys
      .iter()
      .find_map(|key| self.value(key))
      .unwrap_or_else(|| fallback.to_string())
  }

  /// The font family of a profile.
  pub fn family(&self, font: FontProfile) -> String {
    match font {
      FontProfile::Apps => self.first(&["apps_family", "default_family"], "IBM Plex Mono"),
      FontProfile::Terminal => {
        self.first(&["terminal_family", "monospace_family"], "IBM Plex Mono")
      }
    }
  }

  /// The font size of a selection. A file manager uses the terminal size.
  pub fn size(&self, selection: Selection) -> String {
    match (selection.font, selection.file_manager) {
      (FontProfile::Apps, false) => self.first(&["apps_size", "default_size"], "12"),
      (FontProfile::Apps, true) | (FontProfile::Terminal, _) => {
        self.first(&["terminal_size", "monospace_size"], "13")
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::FontState;
  use crate::profile::{FontProfile, Selection};

  fn state(text: &str) -> FontState {
    FontState::new(Some(text.to_string()))
  }

  #[test]
  fn values_fall_back_when_missing_or_empty() {
    let fonts = state("monospace_family=Fira\nterminal_family=\n");
    assert_eq!(fonts.family(FontProfile::Terminal), "Fira");
    assert_eq!(fonts.family(FontProfile::Apps), "IBM Plex Mono");
    assert_eq!(
      FontState::new(None).family(FontProfile::Terminal),
      "IBM Plex Mono"
    );
  }

  #[test]
  fn sizes_follow_the_selection() {
    let fonts = state("default_size=11\nmonospace_size=14\n");
    let apps = Selection {
      font: FontProfile::Apps,
      file_manager: false,
    };
    let manager = Selection {
      font: FontProfile::Apps,
      file_manager: true,
    };
    assert_eq!(fonts.size(apps), "11");
    assert_eq!(fonts.size(manager), "14");
    assert_eq!(FontState::new(None).size(apps), "12");
  }
}
