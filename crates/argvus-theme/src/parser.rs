//! Implements resource parsing in crate `argvus theme`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgba {
  pub r: u8,
  pub g: u8,
  pub b: u8,
  pub a: u8,
}

impl Rgba {
  /// Executes the const function documented in this module. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
  pub const fn opaque(r: u8, g: u8, b: u8) -> Self {
    Self { r, g, b, a: 255 }
  }
}

/// Converts input data into `parse_color` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn parse_color(input: &str) -> Option<Rgba> {
  let input = input.trim();
  if let Some(hex) = input.strip_prefix('#') {
    return parse_hex(hex);
  }
  for (prefix, rgba) in [("rgba(", true), ("rgb(", false)] {
    if let Some(inner) = input
      .strip_prefix(prefix)
      .and_then(|value| value.strip_suffix(')'))
    {
      let mut values = inner.split([',', ' ']).filter(|value| !value.is_empty());
      let r = values.next()?.parse::<f32>().ok()?;
      let g = values.next()?.parse::<f32>().ok()?;
      let b = values.next()?.parse::<f32>().ok()?;
      if [r, g, b]
        .into_iter()
        .any(|value| !(0.0..=255.0).contains(&value))
      {
        return None;
      }
      let alpha = if rgba {
        let value = values.next()?.parse::<f32>().ok()?;
        if value <= 1.0 { value * 255.0 } else { value }
      } else {
        255.0
      };
      return Some(Rgba {
        r: r.round() as u8,
        g: g.round() as u8,
        b: b.round() as u8,
        a: alpha.clamp(0.0, 255.0).round() as u8,
      });
    }
  }
  None
}

/// Converts input data into `parse_hex` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn parse_hex(hex: &str) -> Option<Rgba> {
  match hex.len() {
    3 => Some(Rgba::opaque(
      u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?,
      u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?,
      u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?,
    )),
    6 | 8 => Some(Rgba {
      r: u8::from_str_radix(&hex[0..2], 16).ok()?,
      g: u8::from_str_radix(&hex[2..4], 16).ok()?,
      b: u8::from_str_radix(&hex[4..6], 16).ok()?,
      a: if hex.len() == 8 {
        u8::from_str_radix(&hex[6..8], 16).ok()?
      } else {
        255
      },
    }),
    _ => None,
  }
}

/// Executes the `extract_define_colors` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn extract_define_colors(css: &str) -> Vec<(String, String)> {
  let mut output = Vec::new();
  let mut rest = css;
  while let Some(start) = rest.find("@define-color") {
    let after = &rest[start + "@define-color".len()..];
    let Some((declaration, next)) = after.split_once(';') else {
      break;
    };
    let declaration = declaration.trim();
    if let Some((name, value)) = declaration.split_once(char::is_whitespace) {
      let value = value.trim();
      if !name.is_empty() && !value.is_empty() {
        output.push((name.to_string(), value.to_string()));
      }
    }
    rest = next;
  }
  output
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  /// Converts input data into `parses_supported_css_colors` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn parses_supported_css_colors() {
    assert_eq!(parse_color("#123"), Some(Rgba::opaque(0x11, 0x22, 0x33)));
    assert_eq!(parse_color("#112233"), Some(Rgba::opaque(0x11, 0x22, 0x33)));
    assert_eq!(parse_color("rgb(1, 2, 3)"), Some(Rgba::opaque(1, 2, 3)));
    assert_eq!(parse_color("rgba(1, 2, 3, 0.5)").map(|c| c.a), Some(128));
  }

  #[test]
  /// Executes the `extracts_palette_declarations` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn extracts_palette_declarations() {
    let defs = extract_define_colors(
      "@define-color argvus_bg #111316; @define-color argvus_border rgba(223, 229, 234, 0.22);",
    );
    assert_eq!(defs.len(), 2);
    assert_eq!(defs[1].1, "rgba(223, 229, 234, 0.22)");
  }
}
