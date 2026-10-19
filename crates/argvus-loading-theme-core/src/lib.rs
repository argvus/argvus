//! Pure theme palette helpers shared by the splash application.

pub const DEFAULT_BACKGROUND: &str = "#101218";
pub const DEFAULT_FOREGROUND: &str = "#f4f4f4";
pub const DEFAULT_ACCENT: &str = "#7aa2f7";

pub fn theme_colors(theme: &str) -> Option<(&'static str, &'static str, &'static str)> {
  let normalized = theme.trim().to_ascii_lowercase().replace(['_', ' '], "-");
  let normalized = normalized.strip_prefix("argvus-").unwrap_or(&normalized);
  let normalized = normalized.strip_suffix("-float").unwrap_or(normalized);
  match normalized {
    "onedark" | "one-dark" => Some(("#282C34", "#ABB2BF", "#61AFEF")),
    "dracula" => Some(("#282A36", "#F8F8F2", "#BD93F9")),
    "dark" | "dark-aether" => Some(("#191b27", "#3590bd", DEFAULT_ACCENT)),
    "silver-dark" | "dark-silver" => Some(("#595959", "#121518", "#121518")),
    "rose-pine" | "dark-rose-pine" => Some(("#191724", "#E0DEF4", "#C4A7E7")),
    "slate-dark" | "dark-slate" => Some(("#3b4352", "#7391a5", DEFAULT_ACCENT)),
    "universe" | "dark-universe" => Some(("#000000", "#ffffff", DEFAULT_ACCENT)),
    "gruvbox-high-dark" | "dark-gruvbox-high" => Some(("#282828", "#EBDBB2", "#D79921")),
    "gruvbox-dark" | "dark-gruvbox" => Some(("#282828", "#EBDBB2", "#D4BE98")),
    "light" | "light-veil" => Some(("#ffffff", "#000000", DEFAULT_ACCENT)),
    "github-light" => Some(("#FFFFFF", "#1F2328", "#0969DA")),
    "one-light" => Some(("#FAFAFA", "#383A42", "#4078F2")),
    "everforest-light" => Some(("#FDF6E3", "#5C6A72", "#3A94C5")),
    "solarized-light" | "light-solarized" => Some(("#FDF6E3", "#657B83", "#268BD2")),
    "tokyo-night" | "dark-tokio-night" => Some(("#1A1B26", "#C0CAF5", "#7AA2F7")),
    "solitude" | "dark-solitude" => Some(("#101315", "#CACCCC", "#798186")),
    "sunset" | "dark-sunset" => Some(("#0F0F0F", "#EADCCC", "#E2BE8A")),
    "matrix" | "dark-matrix" => Some(("#0B0C16", "#DDF7FF", "#82FB9C")),
    "monokai-dark" | "dark-monokai" => Some(("#2D2A2E", "#FCFCFA", "#78DCE8")),
    "catppuccin-latte" | "light-catppuccin-latte" | "dark-catppuccin-latte" => {
      Some(("#EFF1F5", "#4C4F69", "#1E66F5"))
    }
    "gruvbox-light" | "light-gruvbox" => Some(("#FBF1C7", "#3C3836", "#458588")),
    _ => None,
  }
}

pub fn valid_color(value: &str, fallback: &str) -> String {
  let valid = matches!(value.len(), 4 | 7 | 9)
    && value.starts_with('#')
    && value[1..]
      .chars()
      .all(|character| character.is_ascii_hexdigit());
  if valid {
    value.to_owned()
  } else {
    fallback.to_owned()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn accepts_supported_hex_colors() {
    assert_eq!(valid_color("#abc", DEFAULT_BACKGROUND), "#abc");
    assert_eq!(valid_color("#AABBCC", DEFAULT_BACKGROUND), "#AABBCC");
    assert_eq!(valid_color("#AABBCCDD", DEFAULT_BACKGROUND), "#AABBCCDD");
  }

  #[test]
  fn rejects_css_injection_and_bad_lengths() {
    assert_eq!(valid_color("red", DEFAULT_BACKGROUND), DEFAULT_BACKGROUND);
    assert_eq!(
      valid_color("#123456;", DEFAULT_BACKGROUND),
      DEFAULT_BACKGROUND
    );
    assert_eq!(
      valid_color("#12345", DEFAULT_BACKGROUND),
      DEFAULT_BACKGROUND
    );
  }

  #[test]
  fn maps_supported_theme_names() {
    assert_eq!(
      theme_colors("ARGVUS Dark"),
      Some(("#191b27", "#3590bd", DEFAULT_ACCENT))
    );
    assert_eq!(
      theme_colors("silver-dark-float"),
      Some(("#595959", "#121518", "#121518"))
    );
    assert_eq!(
      theme_colors("ARGVUS Light"),
      Some(("#ffffff", "#000000", DEFAULT_ACCENT))
    );
    assert_eq!(
      theme_colors("solitude"),
      Some(("#101315", "#CACCCC", "#798186"))
    );
    assert_eq!(
      theme_colors("sunset"),
      Some(("#0F0F0F", "#EADCCC", "#E2BE8A"))
    );
    assert_eq!(
      theme_colors("matrix"),
      Some(("#0B0C16", "#DDF7FF", "#82FB9C"))
    );
    assert_eq!(
      theme_colors("monokai-dark"),
      Some(("#2D2A2E", "#FCFCFA", "#78DCE8"))
    );
    assert_eq!(
      theme_colors("ARGVUS GitHub Light"),
      Some(("#FFFFFF", "#1F2328", "#0969DA"))
    );
    assert_eq!(
      theme_colors("ARGVUS Solarized Light"),
      Some(("#FDF6E3", "#657B83", "#268BD2"))
    );
    assert_eq!(
      theme_colors("solarized-light"),
      Some(("#FDF6E3", "#657B83", "#268BD2"))
    );
    assert_eq!(
      theme_colors("ARGVUS Everforest Light Float"),
      Some(("#FDF6E3", "#5C6A72", "#3A94C5"))
    );
    assert_eq!(
      theme_colors("ARGVUS Catppuccin Latte"),
      Some(("#EFF1F5", "#4C4F69", "#1E66F5"))
    );
    assert_eq!(
      theme_colors("catppuccin-latte-float"),
      Some(("#EFF1F5", "#4C4F69", "#1E66F5"))
    );
    assert_eq!(
      theme_colors("argvus-light-catppuccin-latte-float"),
      Some(("#EFF1F5", "#4C4F69", "#1E66F5"))
    );
    assert_eq!(
      theme_colors("ARGVUS One Dark"),
      Some(("#282C34", "#ABB2BF", "#61AFEF"))
    );
    assert_eq!(
      theme_colors("ARGVUS One Light Float"),
      Some(("#FAFAFA", "#383A42", "#4078F2"))
    );
    assert_eq!(
      theme_colors("ARGVUS Tokyo Night"),
      Some(("#1A1B26", "#C0CAF5", "#7AA2F7"))
    );
    assert_eq!(
      theme_colors("tokyo-night-float"),
      Some(("#1A1B26", "#C0CAF5", "#7AA2F7"))
    );
  }

  #[test]
  fn maps_light_gruvbox_theme_names() {
    assert_eq!(
      theme_colors("ARGVUS Light Gruvbox Float"),
      Some(("#FBF1C7", "#3C3836", "#458588"))
    );
  }
}
