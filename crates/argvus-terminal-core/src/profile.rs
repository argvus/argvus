//! The font profiles and the names of their generated configuration files.

/// The font profiles the launcher accepts.
pub const PROFILES: [&str; 2] = ["terminal", "apps"];

/// Whether a font profile is one the launcher knows.
pub fn is_known(profile: &str) -> bool {
  PROFILES.contains(&profile)
}

/// The generated kitty configuration of a profile, relative to the cache.
/// The terminal profile has the plain name; the others carry their profile.
pub fn base_file(profile: &str) -> String {
  match profile {
    "terminal" => "argvus-terminal/kitty.conf".to_string(),
    other => format!("argvus-terminal/kitty-{other}.conf"),
  }
}

/// The generated configuration of the TUI of a profile, relative to the cache.
pub fn tui_file(profile: &str) -> String {
  match profile {
    "terminal" => "argvus-terminal/kitty-tui.conf".to_string(),
    other => format!("argvus-terminal/kitty-tui-{other}.conf"),
  }
}

#[cfg(test)]
mod tests {
  use super::{base_file, is_known, tui_file};

  #[test]
  fn profiles_name_their_files() {
    assert!(is_known("apps"));
    assert!(!is_known("editor"));
    assert_eq!(base_file("terminal"), "argvus-terminal/kitty.conf");
    assert_eq!(base_file("apps"), "argvus-terminal/kitty-apps.conf");
    assert_eq!(tui_file("terminal"), "argvus-terminal/kitty-tui.conf");
    assert_eq!(tui_file("apps"), "argvus-terminal/kitty-tui-apps.conf");
  }
}
