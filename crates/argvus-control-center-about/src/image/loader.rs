//! Implements resource loading in crate `argvus about`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use image::{Rgba, RgbaImage};

/// Defines the constant `MAX_DIMENSION`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub const MAX_DIMENSION: u32 = 512;

/// Retrieves data for `load_svg` without mixing collection with TUI rendering. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn load_svg(path: &std::path::Path) -> Option<RgbaImage> {
  let data = std::fs::read(path).ok()?;
  let tree = resvg::usvg::Tree::from_data(&data, &resvg::usvg::Options::default()).ok()?;
  let size = tree.size();
  if size.width() <= 0.0 || size.height() <= 0.0 {
    return None;
  }

  let scale = (MAX_DIMENSION as f32 / size.width().max(size.height())).min(1.0);
  let width = (size.width() * scale).round().max(1.0) as u32;
  let height = (size.height() * scale).round().max(1.0) as u32;

  let mut pixmap = resvg::tiny_skia::Pixmap::new(width, height)?;
  let transform = resvg::tiny_skia::Transform::from_scale(scale, scale);
  resvg::render(&tree, transform, &mut pixmap.as_mut());

  unpremultiply(&pixmap)
}

/// Executes the `unpremultiply` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn unpremultiply(pixmap: &resvg::tiny_skia::Pixmap) -> Option<RgbaImage> {
  let mut image = RgbaImage::new(pixmap.width(), pixmap.height());
  let data = pixmap.data();
  for (index, pixel) in data.chunks_exact(4).enumerate() {
    let (r, g, b, a) = (pixel[0], pixel[1], pixel[2], pixel[3]);
    let (r, g, b) = if a == 0 {
      (0, 0, 0)
    } else {
      multiply_back(r, a)
        .zip(multiply_back(g, a))
        .zip(multiply_back(b, a))
        .map(|((r, g), b)| (r, g, b))
        .unwrap_or((0, 0, 0))
    };
    image.put_pixel(
      (index % pixmap.width() as usize) as u32,
      (index / pixmap.width() as usize) as u32,
      Rgba([r, g, b, a]),
    );
  }
  Some(image)
}

/// Executes the `multiply_back` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn multiply_back(channel: u8, alpha: u8) -> Option<u8> {
  if alpha == 0 {
    return Some(0);
  }
  u16::from(channel).checked_mul(255).map(|value| {
    let half = u16::from(alpha) / 2;
    ((value + half) / u16::from(alpha)).min(255) as u8
  })
}

/// Executes the `find_logo_path` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn find_logo_path() -> Option<std::path::PathBuf> {
  let mut candidates = vec![std::path::PathBuf::from(
    "/usr/share/argvus/svg/ARGVUS-logo.svg",
  )];
  if let Ok(current_dir) = std::env::current_dir() {
    candidates
      .extend([current_dir.join("../argvus-branding/src/usr/share/argvus/svg/ARGVUS-logo.svg")]);
  }
  candidates.iter().find(|&path| path.exists()).cloned()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  /// Executes the `multiplies_back_premultiplied_alpha` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn multiplies_back_premultiplied_alpha() {
    assert_eq!(multiply_back(0x80, 0xff), Some(0x80));
    assert_eq!(multiply_back(0x40, 0x80), Some(0x80));
    assert_eq!(multiply_back(0x00, 0x00), Some(0x00));
  }
}
