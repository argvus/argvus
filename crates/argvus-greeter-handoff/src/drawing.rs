//! Software drawing of the background and the loading spinner into a dumb buffer.

use super::drm::DumbBuffer;
use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Palette {
  pub(super) background: u32,
  pub(super) accent: u32,
  /// The user's accent, kept when a `theme=` command changes the theme.
  pub(super) user_accent: Option<u32>,
}

impl Palette {
  /// The colors of `theme` from the table the GTK splash uses, with the user's
  /// accent when there is one, so both surfaces paint the same colors.
  pub(super) fn new(theme: &str, user_accent: Option<u32>) -> Self {
    let (background, theme_accent) = theme_colors(theme)
      .map(|(background, _, accent)| (background, accent))
      .unwrap_or((DEFAULT_BACKGROUND, DEFAULT_ACCENT));
    let background = hex_to_rgb(background).unwrap_or_default();
    let accent = user_accent
      .or_else(|| hex_to_rgb(theme_accent))
      .unwrap_or_default();
    Self {
      background,
      accent,
      user_accent,
    }
  }

  pub(super) fn with_theme(&self, theme: &str) -> Self {
    Self::new(theme, self.user_accent)
  }
}

/// Paints the whole buffer with the background color.
pub(super) fn draw_background(buffer: &mut DumbBuffer, palette: &Palette) {
  buffer.pixels().fill(0xff00_0000 | palette.background);
}

/// Draws the shared spinner (`argvus_loading_theme_core::spinner`) at the
/// center of the output.
pub(super) fn draw_spinner(buffer: &mut DumbBuffer, palette: &Palette, frame: &spinner::Frame) {
  let image = Image {
    width: buffer.width as usize,
    height: buffer.height as usize,
    stride: (buffer.pitch / 4) as usize,
  };
  paint_spinner(buffer.pixels(), image, palette, frame);
}

/// The layout of an XRGB8888 image: its size and the pixels per row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Image {
  pub(super) width: usize,
  pub(super) height: usize,
  pub(super) stride: usize,
}

/// A rectangle of pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Area {
  pub(super) x: usize,
  pub(super) y: usize,
  pub(super) width: usize,
  pub(super) height: usize,
}

/// Paints the spinner at the center of `pixels` and returns the area it wrote.
/// Each pixel of that area is written once with its final color, so a
/// scanned-out buffer never shows a half-drawn frame, and the area covers every
/// position of the animation, so it also erases the previous frame.
pub(super) fn paint_spinner(
  pixels: &mut [u32],
  image: Image,
  palette: &Palette,
  frame: &spinner::Frame,
) -> Area {
  let center_x = image.width as f64 / 2.0;
  let center_y = image.height as f64 / 2.0;
  // One extra pixel on each side for the anti-aliased edge of the squares.
  let half_side = (spinner::EXTENT / 2.0).ceil() as usize + 1;
  let first_x = (center_x as usize).saturating_sub(half_side);
  let last_x = (center_x as usize + half_side).min(image.width.saturating_sub(1));
  let first_y = (center_y as usize).saturating_sub(half_side);
  let last_y = (center_y as usize + half_side).min(image.height.saturating_sub(1));
  for y in first_y..=last_y {
    for x in first_x..=last_x {
      let color = spinner_pixel(palette, frame, x as f64 - center_x, y as f64 - center_y);
      pixels[y * image.stride + x] = 0xff00_0000 | color;
    }
  }
  Area {
    x: first_x,
    y: first_y,
    width: last_x + 1 - first_x,
    height: last_y + 1 - first_y,
  }
}

/// The color of the one-pixel square whose top-left corner is at (`left`,
/// `top`) from the spinner center. Each square covers the pixel in proportion to
/// the area they share, which gives the same anti-aliased edge cairo draws in
/// the GTK splash.
pub(super) fn spinner_pixel(palette: &Palette, frame: &spinner::Frame, left: f64, top: f64) -> u32 {
  let mut color = palette.background;
  for square in frame.squares {
    let coverage = overlap(square.x, square.x + spinner::SQUARE, left, left + 1.0)
      * overlap(square.y, square.y + spinner::SQUARE, top, top + 1.0);
    if coverage > 0.0 {
      color = mix_rgb(color, palette.accent, coverage);
    }
  }
  color
}

/// Length of the overlap between the intervals [`start`, `end`] and [`from`, `to`].
pub(super) fn overlap(start: f64, end: f64, from: f64, to: f64) -> f64 {
  (end.min(to) - start.max(from)).max(0.0)
}

/// `base` moved toward `over` by `amount` (0 keeps `base`, 1 gives `over`).
pub(super) fn mix_rgb(base: u32, over: u32, amount: f64) -> u32 {
  let channel = |shift: u32| {
    let from = f64::from((base >> shift) & 0xff);
    let to = f64::from((over >> shift) & 0xff);
    ((from + (to - from) * amount).round() as u32) << shift
  };
  channel(16) | channel(8) | channel(0)
}
