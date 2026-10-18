//! Implements safe fallback rendering in crate `argvus about`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use image::imageops;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};

/// Defines the constant `HALF_BLOCK`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
const HALF_BLOCK: char = '\u{2580}'; // ▲ upper half block ▀
/// Defines the constant `ASCII_RAMP`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
const ASCII_RAMP: [char; 10] = [' ', '.', ':', '-', '=', '+', '*', '#', '%', '@'];

/// Executes the `half_blocks` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn half_blocks(
  source: &image::RgbaImage,
  cols: usize,
  rows: usize,
  background: Color,
) -> Vec<Line<'static>> {
  if cols == 0 || rows == 0 {
    return Vec::new();
  }
  let resized = imageops::resize(
    source,
    cols as u32,
    (rows * 2) as u32,
    imageops::FilterType::Triangle,
  );
  let mut lines = Vec::with_capacity(rows);
  for row in 0..rows {
    let mut spans = Vec::with_capacity(cols);
    for col in 0..cols {
      let top = resized.get_pixel(col as u32, (row * 2) as u32).0;
      let bottom = resized.get_pixel(col as u32, (row * 2 + 1) as u32).0;
      let (top, bottom) = (blend(top, background), blend(bottom, background));
      if top == background && bottom == background {
        spans.push(Span::styled(" ".to_string(), style(bottom)));
      } else {
        spans.push(Span::styled(HALF_BLOCK.to_string(), style(bottom).fg(top)));
      }
    }
    lines.push(Line::from(spans));
  }
  lines
}

/// Executes the `ascii` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn ascii(
  source: &image::RgbaImage,
  cols: usize,
  rows: usize,
  accent: Color,
) -> Vec<Line<'static>> {
  if cols == 0 || rows == 0 {
    return Vec::new();
  }
  let resized = imageops::resize(
    source,
    cols as u32,
    rows as u32,
    imageops::FilterType::Triangle,
  );
  let mut lines = Vec::with_capacity(rows);
  for row in 0..rows {
    let mut text = String::with_capacity(cols);
    for col in 0..cols {
      let pixel = resized.get_pixel(col as u32, row as u32).0;
      let coverage = pixel[3] as f32 / 255.0;
      let index =
        ((coverage * (ASCII_RAMP.len() - 1) as f32).round() as usize).min(ASCII_RAMP.len() - 1);
      text.push(ASCII_RAMP[index]);
    }
    lines.push(Line::styled(text, Style::new().fg(accent)));
  }
  lines
}

/// Executes the `blend` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn blend(pixel: [u8; 4], background: Color) -> Color {
  let (br, bg, bb) = rgb_tuple(background);
  let alpha = pixel[3] as f32 / 255.0;
  if alpha >= 1.0 {
    return Color::Rgb(pixel[0], pixel[1], pixel[2]);
  }
  if alpha <= 0.0 {
    return background;
  }
  let mix =
    |front: u8, back: u8| (front as f32 * alpha + back as f32 * (1.0 - alpha)).round() as u8;
  Color::Rgb(mix(pixel[0], br), mix(pixel[1], bg), mix(pixel[2], bb))
}

/// Executes the `style` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn style(bg: Color) -> Style {
  Style::new().bg(bg)
}

/// Executes the `rgb_tuple` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn rgb_tuple(color: Color) -> (u8, u8, u8) {
  match color {
    Color::Rgb(r, g, b) => (r, g, b),
    Color::Reset => (0, 0, 0),
    Color::Black => (0, 0, 0),
    Color::White => (255, 255, 255),
    Color::Gray => (128, 128, 128),
    Color::DarkGray => (64, 64, 64),
    Color::Red => (255, 0, 0),
    Color::Green => (0, 255, 0),
    Color::Yellow => (255, 255, 0),
    Color::Blue => (0, 0, 255),
    Color::Magenta => (255, 0, 255),
    Color::Cyan => (0, 255, 255),
    Color::LightRed => (255, 128, 128),
    Color::LightGreen => (128, 255, 128),
    Color::LightYellow => (255, 255, 128),
    Color::LightBlue => (128, 128, 255),
    Color::LightMagenta => (255, 128, 255),
    Color::LightCyan => (128, 255, 255),
    Color::Indexed(_) => (128, 128, 128),
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use image::Rgba;

  /// Executes the `solid_square` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn solid_square(size: u32, color: [u8; 4]) -> image::RgbaImage {
    let mut image = image::RgbaImage::new(size, size);
    for x in 0..size {
      for y in 0..size {
        image.put_pixel(x, y, Rgba(color));
      }
    }
    image
  }

  #[test]
  /// Executes the `half_blocks_emit_full_lines` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn half_blocks_emit_full_lines() {
    let logo = solid_square(4, [0x35, 0x90, 0xbd, 255]);
    let lines = half_blocks(&logo, 4, 3, Color::Rgb(0x11, 0x13, 0x16));
    assert_eq!(lines.len(), 3);
    assert!(lines[0].width() >= 4);
  }

  #[test]
  /// Executes the `ascii_emits_ramp_for_opaque_logo` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn ascii_emits_ramp_for_opaque_logo() {
    let logo = solid_square(4, [0x35, 0x90, 0xbd, 255]);
    let lines = ascii(&logo, 4, 3, Color::Rgb(0x35, 0x90, 0xbd));
    assert_eq!(lines.len(), 3);
    assert!(lines[0].to_string().contains('@'));
  }

  #[test]
  /// Executes the `empty_input_yields_no_lines` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn empty_input_yields_no_lines() {
    let logo = solid_square(2, [0, 0, 0, 0]);
    assert!(
      half_blocks(&logo, 4, 2, Color::Rgb(0x11, 0x13, 0x16))
        .iter()
        .all(|line| line.width() == 4)
    );
  }
}
