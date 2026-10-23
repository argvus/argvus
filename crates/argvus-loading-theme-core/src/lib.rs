//! Pure theme palette and spinner helpers shared by the splash surfaces: the
//! GTK overlay and the greeter's DRM handoff surface.

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

/// Converts a `#rgb` or `#rrggbb` color to `0xrrggbb`. The alpha of a
/// `#rrggbbaa` color is ignored, because every splash surface is opaque.
pub fn hex_to_rgb(value: &str) -> Option<u32> {
  let digits = value.strip_prefix('#')?;
  if !digits
    .chars()
    .all(|character| character.is_ascii_hexdigit())
  {
    return None;
  }
  match digits.len() {
    3 => {
      let expanded: String = digits.chars().flat_map(|digit| [digit, digit]).collect();
      u32::from_str_radix(&expanded, 16).ok()
    }
    6 | 8 => u32::from_str_radix(&digits[..6], 16).ok(),
    _ => None,
  }
}

/// The loading spinner, shared by every surface of the login handoff.
///
/// The greeter splash, the DRM handoff surface and the session overlay are
/// three processes on two display paths. They draw this same spinner, in
/// physical pixels, and take its phase from `CLOCK_MONOTONIC`, so the switch
/// from one process to the next does not change what is on screen.
pub mod spinner {
  /// Number of dots on the circle.
  pub const DOTS: usize = 12;
  /// Distance from the center to each dot, in physical pixels.
  pub const ORBIT_RADIUS: f64 = 28.0;
  /// Radius of one dot, in physical pixels.
  pub const DOT_RADIUS: f64 = 5.0;
  /// How long each dot stays highlighted.
  pub const STEP_MILLIS: u64 = 90;
  /// Color of the dots that are not highlighted.
  pub const IDLE_RGB: u32 = 0x40_4040;
  /// Opacity of the dots that are not highlighted.
  pub const IDLE_ALPHA: f64 = 0.4;
  /// Side of the square that holds the whole spinner, in physical pixels.
  pub const EXTENT: f64 = 2.0 * (ORBIT_RADIUS + DOT_RADIUS);

  /// The highlighted dot at `monotonic_millis` (milliseconds of `CLOCK_MONOTONIC`).
  pub fn active_dot(monotonic_millis: u64) -> usize {
    ((monotonic_millis / STEP_MILLIS) % DOTS as u64) as usize
  }

  /// The idle dot color over an opaque `background`, for surfaces without
  /// alpha blending (the DRM handoff buffer). It equals what a compositor shows
  /// for `IDLE_RGB` at `IDLE_ALPHA` over the same background.
  pub fn idle_rgb_over(background: u32) -> u32 {
    let channel = |shift: u32| {
      let base = f64::from((background >> shift) & 0xff);
      let idle = f64::from((IDLE_RGB >> shift) & 0xff);
      ((base + (idle - base) * IDLE_ALPHA).round() as u32) << shift
    };
    channel(16) | channel(8) | channel(0)
  }

  /// Offset of dot `index` from the spinner center, in physical pixels.
  pub fn dot_offset(index: usize) -> (f64, f64) {
    let angle = std::f64::consts::TAU * (index % DOTS) as f64 / DOTS as f64;
    (angle.cos() * ORBIT_RADIUS, angle.sin() * ORBIT_RADIUS)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn converts_short_long_and_alpha_hex_colors() {
    assert_eq!(hex_to_rgb("#191b27"), Some(0x19_1b27));
    assert_eq!(hex_to_rgb("#FFF"), Some(0xff_ffff));
    assert_eq!(hex_to_rgb("#11223344"), Some(0x11_2233));
    assert_eq!(hex_to_rgb("191b27"), None);
    assert_eq!(hex_to_rgb("#12345"), None);
  }

  #[test]
  fn spinner_phase_follows_the_shared_clock() {
    assert_eq!(spinner::active_dot(0), 0);
    assert_eq!(spinner::active_dot(spinner::STEP_MILLIS - 1), 0);
    assert_eq!(spinner::active_dot(spinner::STEP_MILLIS), 1);
    assert_eq!(
      spinner::active_dot(spinner::STEP_MILLIS * spinner::DOTS as u64),
      0
    );
  }

  #[test]
  fn idle_dot_blends_toward_its_color() {
    assert_eq!(spinner::idle_rgb_over(spinner::IDLE_RGB), spinner::IDLE_RGB);
    // 0x19 + (0x40 - 0x19) * 0.4 = 0x28.6 -> 0x29
    assert_eq!(spinner::idle_rgb_over(0x19_1919), 0x29_2929);
  }

  #[test]
  fn spinner_dots_lie_on_the_orbit() {
    for index in 0..spinner::DOTS {
      let (x, y) = spinner::dot_offset(index);
      assert!(((x * x + y * y).sqrt() - spinner::ORBIT_RADIUS).abs() < 1e-9);
    }
    let (first_x, first_y) = spinner::dot_offset(0);
    assert!((first_x - spinner::ORBIT_RADIUS).abs() < 1e-9 && first_y.abs() < 1e-9);
  }

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
