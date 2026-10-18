//! Implements resource resolution in crate `argvus theme`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use std::collections::HashMap;

use ratatui::style::Color;

use super::Theme;
use super::fallback;
use super::loader::{self, Loader};
use super::parser::{self, Rgba};

/// Executes the `resolve` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn resolve(loader: &Loader) -> Theme {
  let mut definitions: Vec<(String, String)> = fallback::definitions()
    .into_iter()
    .map(|(name, value)| (name.to_string(), value.to_string()))
    .collect();
  for source in loader.sources() {
    if let Some(css) = loader::read_recursive(&source, &mut Vec::new()) {
      definitions.extend(parser::extract_define_colors(&css));
    }
  }
  let palette = resolve_palette(&definitions);
  let bg = get(&palette, "argvus_bg", fallback::BG);
  let fg = get(&palette, "argvus_fg", fallback::FG);
  let surface_alt = palette
    .get("argvus_surface_alt")
    .or_else(|| palette.get("argvus_surface"))
    .copied()
    .unwrap_or(fallback::SURFACE_ALT);
  let accent = loader
    .accent_override()
    .unwrap_or_else(|| get(&palette, "argvus_accent", fallback::ACCENT));
  let muted = get(&palette, "argvus_muted", fallback::MUTED);
  let border = composite(get(&palette, "argvus_border", fallback::BORDER), bg);
  let focus = composite(get(&palette, "argvus_focus", fallback::FOCUS), bg);
  let accent_alpha = composite(
    loader
      .accent_override()
      .map(|_| Rgba { a: 115, ..accent })
      .unwrap_or_else(|| get(&palette, "argvus_accent_alpha", Rgba { a: 115, ..accent })),
    bg,
  );
  let danger = get(
    &palette,
    "argvus_danger",
    if luminance(bg) > 0.5 {
      Rgba::opaque(0x9b, 0x2f, 0x3b)
    } else {
      Rgba::opaque(0xd0, 0x5f, 0x5f)
    },
  );
  let light = luminance(bg) > luminance(fg);
  let selected_foreground = if luminance(accent) > 0.5 {
    bg
  } else {
    Rgba::opaque(255, 255, 255)
  };
  Theme {
    name: loader.active_name(),
    is_light: light,
    background: color(bg),
    foreground: color(fg),
    accent: color(accent),
    tab_active: color(accent),
    tab_inactive: color(muted),
    selected_background: color(accent),
    selected_foreground: color(selected_foreground),
    border: color(border),
    border_active: color(focus),
    muted: color(muted),
    link: color(if loader.active_name().starts_with("silver-dark") {
      Rgba::opaque(255, 255, 255)
    } else {
      accent
    }),
    surface: color(surface_alt),
    accent_alpha: color(accent_alpha),
    success: color(if light {
      Rgba::opaque(0x2f, 0x81, 0x32)
    } else {
      Rgba::opaque(0x4f, 0xae, 0x4d)
    }),
    warning: color(if light {
      Rgba::opaque(0x8a, 0x7a, 0x22)
    } else {
      Rgba::opaque(0xb3, 0xa3, 0x52)
    }),
    error: color(danger),
  }
}

/// Executes the `resolve_palette` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn resolve_palette(definitions: &[(String, String)]) -> HashMap<String, Rgba> {
  let mut palette = HashMap::new();
  for (name, value) in definitions {
    let parsed = value
      .strip_prefix('@')
      .and_then(|reference| palette.get(reference).copied())
      .or_else(|| parser::parse_color(value));
    if let Some(parsed) = parsed {
      palette.insert(name.clone(), parsed);
    }
  }
  palette
}

/// Executes the `get` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn get(palette: &HashMap<String, Rgba>, name: &str, fallback: Rgba) -> Rgba {
  palette.get(name).copied().unwrap_or(fallback)
}

/// Executes the `color` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn color(value: Rgba) -> Color {
  Color::Rgb(value.r, value.g, value.b)
}

/// Executes the `composite` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn composite(front: Rgba, back: Rgba) -> Rgba {
  let alpha = front.a as f32 / 255.0;
  let blend = |a: u8, b: u8| (a as f32 * alpha + b as f32 * (1.0 - alpha)).round() as u8;
  Rgba::opaque(
    blend(front.r, back.r),
    blend(front.g, back.g),
    blend(front.b, back.b),
  )
}

/// Executes the `luminance` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn luminance(color: Rgba) -> f32 {
  (0.299 * color.r as f32 + 0.587 * color.g as f32 + 0.114 * color.b as f32) / 255.0
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  /// Executes the `resolves_references_and_later_overrides` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn resolves_references_and_later_overrides() {
    let palette = resolve_palette(&[
      ("base".into(), "#010203".into()),
      ("copy".into(), "@base".into()),
      ("base".into(), "#040506".into()),
    ]);
    assert_eq!(palette.get("copy"), Some(&Rgba::opaque(1, 2, 3)));
    assert_eq!(palette.get("base"), Some(&Rgba::opaque(4, 5, 6)));
  }

  #[test]
  /// Executes the `missing_files_still_produce_safe_theme` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn missing_files_still_produce_safe_theme() {
    let theme = resolve(&Loader::new());
    assert_ne!(theme.background, theme.foreground);
    assert!(!theme.name.is_empty());
  }
}
