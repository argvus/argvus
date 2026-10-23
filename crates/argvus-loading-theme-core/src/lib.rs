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
/// three processes on two display paths. They draw this same animation, in
/// physical pixels, and take its phase from `CLOCK_MONOTONIC`, so the switch
/// from one process to the next does not change what is on screen.
///
/// The animation is a four-square loader: a box grows from `REST` to `OPEN`,
/// each square moves to the next corner, and the box shrinks back. The
/// keyframes and the easing follow a CSS loader with a 1.5 s cycle.
pub mod spinner {
  /// Length of one full cycle, in milliseconds.
  pub const CYCLE_MILLIS: u64 = 1500;
  /// Side of each square, in physical pixels.
  pub const SQUARE: f64 = 21.0;
  /// Side of the box while the squares rest in its corners.
  pub const REST: f64 = 40.0;
  /// Side of the box while the squares travel to the next corners.
  pub const OPEN: f64 = 60.0;
  /// Side of the square that holds the whole spinner, in physical pixels.
  pub const EXTENT: f64 = OPEN;

  /// Cycle progress at which the box stops growing.
  const EXPAND_END: f64 = 0.33;
  /// Cycle progress at which the squares reach the next corners.
  const MOVE_END: f64 = 0.66;
  /// Control points of the `cubic-bezier(0.3, 1, 0, 1)` applied to each segment.
  const EASE_X1: f64 = 0.3;
  const EASE_Y1: f64 = 1.0;
  const EASE_X2: f64 = 0.0;
  const EASE_Y2: f64 = 1.0;

  /// Corner fractions (x, y) of the box that each square occupies at rest:
  /// top-left, top-right, bottom-right, bottom-left.
  const START: [(f64, f64); 4] = [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)];
  /// Corner fractions after the squares have moved one corner clockwise.
  const END: [(f64, f64); 4] = [(1.0, 0.0), (1.0, 1.0), (0.0, 1.0), (0.0, 0.0)];

  /// Top-left corner of one square, relative to the spinner center.
  #[derive(Debug, Clone, Copy, PartialEq)]
  pub struct Square {
    pub x: f64,
    pub y: f64,
  }

  /// Position of the four squares at one instant, relative to the spinner center.
  #[derive(Debug, Clone, Copy, PartialEq)]
  pub struct Frame {
    pub squares: [Square; 4],
  }

  /// The frame at `monotonic_millis` (milliseconds of `CLOCK_MONOTONIC`).
  pub fn frame(monotonic_millis: u64) -> Frame {
    let progress = (monotonic_millis % CYCLE_MILLIS) as f64 / CYCLE_MILLIS as f64;
    let (container, corners) = if progress < EXPAND_END {
      let eased = ease(progress / EXPAND_END);
      (lerp(REST, OPEN, eased), START)
    } else if progress < MOVE_END {
      let eased = ease((progress - EXPAND_END) / (MOVE_END - EXPAND_END));
      (OPEN, blend_corners(START, END, eased))
    } else {
      let eased = ease((progress - MOVE_END) / (1.0 - MOVE_END));
      (lerp(OPEN, REST, eased), END)
    };
    let free = container - SQUARE;
    let squares = corners.map(|(fraction_x, fraction_y)| Square {
      x: -container / 2.0 + fraction_x * free,
      y: -container / 2.0 + fraction_y * free,
    });
    Frame { squares }
  }

  /// Eased progress of `cubic-bezier(0.3, 1, 0, 1)` at linear progress `x`.
  pub fn ease(x: f64) -> f64 {
    let x = x.clamp(0.0, 1.0);
    let (mut low, mut high) = (0.0, 1.0);
    for _ in 0..60 {
      let middle = (low + high) / 2.0;
      if bezier_axis(middle, EASE_X1, EASE_X2) < x {
        low = middle;
      } else {
        high = middle;
      }
    }
    bezier_axis((low + high) / 2.0, EASE_Y1, EASE_Y2)
  }

  fn bezier_axis(t: f64, first: f64, second: f64) -> f64 {
    let inverse = 1.0 - t;
    3.0 * inverse * inverse * t * first + 3.0 * inverse * t * t * second + t * t * t
  }

  fn lerp(from: f64, to: f64, amount: f64) -> f64 {
    from + (to - from) * amount
  }

  fn blend_corners(from: [(f64, f64); 4], to: [(f64, f64); 4], amount: f64) -> [(f64, f64); 4] {
    std::array::from_fn(|index| {
      (
        lerp(from[index].0, to[index].0, amount),
        lerp(from[index].1, to[index].1, amount),
      )
    })
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
  fn spinner_cycle_repeats_on_the_shared_clock() {
    assert_eq!(spinner::frame(0), spinner::frame(spinner::CYCLE_MILLIS));
    assert_eq!(
      spinner::frame(333),
      spinner::frame(spinner::CYCLE_MILLIS + 333)
    );
  }

  #[test]
  fn spinner_starts_at_rest_in_the_corners() {
    let frame = spinner::frame(0);
    assert_eq!(frame.squares[0], spinner::Square { x: -20.0, y: -20.0 });
    assert_eq!(frame.squares[2], spinner::Square { x: -1.0, y: -1.0 });
  }

  #[test]
  fn spinner_moves_each_square_to_the_next_corner() {
    // 66 % of the cycle: the box is open and the squares have arrived.
    let frame = spinner::frame(990);
    let first = frame.squares[0];
    assert!((first.x - 9.0).abs() < 1e-6);
    assert!((first.y + 30.0).abs() < 1e-6);
  }

  #[test]
  fn spinner_stays_inside_its_extent() {
    let half = spinner::EXTENT / 2.0;
    for millis in (0..spinner::CYCLE_MILLIS).step_by(5) {
      for square in spinner::frame(millis).squares {
        assert!(square.x >= -half - 1e-9 && square.y >= -half - 1e-9);
        assert!(square.x + spinner::SQUARE <= half + 1e-9);
        assert!(square.y + spinner::SQUARE <= half + 1e-9);
      }
    }
  }

  #[test]
  fn spinner_stays_centered_while_it_animates() {
    for millis in (0..spinner::CYCLE_MILLIS).step_by(7) {
      let squares = spinner::frame(millis).squares;
      let mean_x: f64 = squares
        .iter()
        .map(|square| square.x + spinner::SQUARE / 2.0)
        .sum::<f64>()
        / 4.0;
      let mean_y: f64 = squares
        .iter()
        .map(|square| square.y + spinner::SQUARE / 2.0)
        .sum::<f64>()
        / 4.0;
      assert!(mean_x.abs() < 1e-9 && mean_y.abs() < 1e-9);
    }
  }

  #[test]
  fn spinner_easing_hits_both_ends() {
    assert!(spinner::ease(0.0).abs() < 1e-9);
    assert!((spinner::ease(1.0) - 1.0).abs() < 1e-9);
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
