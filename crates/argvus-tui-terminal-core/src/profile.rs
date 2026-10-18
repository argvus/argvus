//! The font profile of a window: the terminal font, or the apps font for file
//! managers. The class and the command decide it, as the launcher always did.

/// The two font profiles of the launcher.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontProfile {
  Terminal,
  Apps,
}

impl FontProfile {
  /// The name used in the Kitty profile file and the terminal backend.
  pub fn name(self) -> &'static str {
    match self {
      Self::Terminal => "terminal",
      Self::Apps => "apps",
    }
  }
}

/// The result of the selection: the font profile and whether the window is a
/// file manager, which takes its size from the terminal font.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Selection {
  pub font: FontProfile,
  pub file_manager: bool,
}

/// Arguments that name a file manager even when the class does not.
const FILE_MANAGER_ARGUMENTS: [&str; 8] = [
  "--spf",
  "spf",
  "--superfile",
  "superfile",
  "--yazy",
  "yazy",
  "--yazi",
  "yazi",
];

/// Selects the font profile from the window class and the command arguments.
pub fn select(class: &str, arguments: &[String]) -> Selection {
  let class_is_file_manager = class == "argvus-file-manager"
    || class.contains("superfile")
    || class.contains("yazi")
    || class.contains("spf");
  let argument_is_file_manager = arguments
    .iter()
    .any(|argument| FILE_MANAGER_ARGUMENTS.contains(&argument.as_str()));
  if class_is_file_manager || argument_is_file_manager {
    Selection {
      font: FontProfile::Apps,
      file_manager: true,
    }
  } else {
    Selection {
      font: FontProfile::Terminal,
      file_manager: false,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::{FontProfile, select};

  fn words(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| value.to_string()).collect()
  }

  #[test]
  fn a_plain_window_uses_the_terminal_font() {
    let selection = select("argvus-tui", &words(&["htop"]));
    assert_eq!(selection.font, FontProfile::Terminal);
    assert!(!selection.file_manager);
  }

  #[test]
  fn a_file_manager_class_or_argument_uses_the_apps_font() {
    assert!(select("argvus-file-manager", &[]).file_manager);
    assert!(select("my-yazi-window", &[]).file_manager);
    assert_eq!(
      select("argvus-tui", &words(&["--spf"])).font,
      FontProfile::Apps
    );
  }
}
