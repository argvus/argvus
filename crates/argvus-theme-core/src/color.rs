//! Colours of a theme palette and the text forms the consumer formats expect.

/// An 8-bit RGBA colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
  pub red: u8,
  pub green: u8,
  pub blue: u8,
  pub alpha: u8,
}

impl Color {
  const WHITE: Self = Self::opaque(255, 255, 255);
  const BLACK: Self = Self::opaque(0, 0, 0);

  pub const fn opaque(red: u8, green: u8, blue: u8) -> Self {
    Self {
      red,
      green,
      blue,
      alpha: 255,
    }
  }

  /// Parses `#RGB`, `#RRGGBB` or `#RRGGBBAA`. Anything else is `None`.
  pub fn parse(text: &str) -> Option<Self> {
    let digits = text.strip_prefix('#')?;
    if !digits.bytes().all(|byte| byte.is_ascii_hexdigit()) {
      return None;
    }
    let channel = |index: usize| u8::from_str_radix(&digits[index..index + 2], 16).ok();
    match digits.len() {
      3 => {
        let expanded: String = digits.chars().flat_map(|digit| [digit, digit]).collect();
        Self::parse(&format!("#{expanded}"))
      }
      6 => Some(Self::opaque(channel(0)?, channel(2)?, channel(4)?)),
      8 => Some(Self {
        red: channel(0)?,
        green: channel(2)?,
        blue: channel(4)?,
        alpha: channel(6)?,
      }),
      _ => None,
    }
  }

  /// The same colour with an opacity between `0.0` and `1.0`.
  pub fn with_opacity(self, opacity: f64) -> Self {
    Self {
      alpha: unit_to_byte(opacity),
      ..self
    }
  }

  /// Blends towards `other`: `weight` `0.0` keeps `self`, `1.0` gives `other`.
  pub fn mix(self, other: Self, weight: f64) -> Self {
    let weight = weight.clamp(0.0, 1.0);
    let blend = |from: u8, to: u8| {
      let value = f64::from(from) + (f64::from(to) - f64::from(from)) * weight;
      // The value stays within 0..=255 because both ends do.
      value.round() as u8
    };
    Self {
      red: blend(self.red, other.red),
      green: blend(self.green, other.green),
      blue: blend(self.blue, other.blue),
      alpha: blend(self.alpha, other.alpha),
    }
  }

  /// Blends towards white by `amount`, keeping the opacity.
  pub fn lighten(self, amount: f64) -> Self {
    Self {
      alpha: self.alpha,
      ..self.mix(Self::WHITE.with_opacity(self.opacity()), amount)
    }
  }

  /// Blends towards black by `amount`, keeping the opacity.
  pub fn darken(self, amount: f64) -> Self {
    Self {
      alpha: self.alpha,
      ..self.mix(Self::BLACK.with_opacity(self.opacity()), amount)
    }
  }

  /// Black or white, whichever reads on this colour: the integer luminance
  /// rule `argvus config accent` has always used (black from 128 up).
  pub fn contrast(self) -> Self {
    let luminance =
      (299 * u32::from(self.red) + 587 * u32::from(self.green) + 114 * u32::from(self.blue)) / 1000;
    if luminance >= 128 {
      Self::BLACK
    } else {
      Self::WHITE
    }
  }

  pub fn opacity(self) -> f64 {
    f64::from(self.alpha) / 255.0
  }

  /// `#RRGGBB`.
  pub fn hex(self) -> String {
    format!("#{:02X}{:02X}{:02X}", self.red, self.green, self.blue)
  }

  /// `#RRGGBBAA`.
  pub fn hex_with_alpha(self) -> String {
    format!("{}{:02X}", self.hex(), self.alpha)
  }

  /// `#AARRGGBB`, the order Qt and QML use.
  pub fn argb_hex(self) -> String {
    format!(
      "#{:02X}{:02X}{:02X}{:02X}",
      self.alpha, self.red, self.green, self.blue
    )
  }

  /// `RRGGBB` without the `#`.
  pub fn bare_hex(self) -> String {
    format!("{:02X}{:02X}{:02X}", self.red, self.green, self.blue)
  }

  /// `rgb(r, g, b)`.
  pub fn css_rgb(self) -> String {
    format!("rgb({}, {}, {})", self.red, self.green, self.blue)
  }

  /// `rgba(r, g, b, a)` with the opacity as a decimal.
  pub fn css_rgba(self) -> String {
    format!(
      "rgba({}, {}, {}, {})",
      self.red,
      self.green,
      self.blue,
      opacity_text(self.alpha)
    )
  }

  /// `rgba(r, g, b, NN%)`, the opacity as a whole percentage (rofi).
  pub fn css_rgba_percent(self) -> String {
    format!(
      "rgba({}, {}, {}, {}%)",
      self.red,
      self.green,
      self.blue,
      (self.opacity() * 100.0).round()
    )
  }

  /// `rgb(rrggbb)`, the hyprlock colour form.
  pub fn hex_rgb_function(self) -> String {
    format!("rgb({:02x}{:02x}{:02x})", self.red, self.green, self.blue)
  }

  /// `rgba(rrggbbaa)`, the Hyprland colour form.
  pub fn hyprland(self) -> String {
    format!(
      "rgba({:02x}{:02x}{:02x}{:02x})",
      self.red, self.green, self.blue, self.alpha
    )
  }

  /// `0xAARRGGBB`, the Hyprland and Hyprtoolkit integer form.
  pub fn argb_integer(self) -> String {
    format!(
      "0x{:02X}{:02X}{:02X}{:02X}",
      self.alpha, self.red, self.green, self.blue
    )
  }
}

/// The shortest decimal that reads back as the same alpha byte: `0.88`, not `0.878`.
fn opacity_text(alpha: u8) -> String {
  let opacity = f64::from(alpha) / 255.0;
  (1..=4)
    .map(|places| format!("{opacity:.places$}"))
    .find(|text| {
      text
        .parse::<f64>()
        .is_ok_and(|value| unit_to_byte(value) == alpha)
    })
    .map_or_else(|| format_decimal(opacity), |text| trim_decimal(&text))
}

fn trim_decimal(text: &str) -> String {
  if text.contains('.') {
    text.trim_end_matches('0').trim_end_matches('.').to_owned()
  } else {
    text.to_owned()
  }
}

fn unit_to_byte(unit: f64) -> u8 {
  // Clamped to 0..=1 first, so the product stays within 0..=255.
  (unit.clamp(0.0, 1.0) * 255.0).round() as u8
}

/// A decimal with at most three places and no trailing zeros: `0.5`, `1`, `0.133`.
pub fn format_decimal(value: f64) -> String {
  let text = format!("{value:.3}");
  let text = text.trim_end_matches('0').trim_end_matches('.');
  if text == "-0" {
    "0".to_owned()
  } else {
    text.to_owned()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parses_short_long_and_alpha_forms() {
    assert_eq!(Color::parse("#fff"), Some(Color::opaque(255, 255, 255)));
    assert_eq!(Color::parse("#E2BE8A"), Some(Color::opaque(226, 190, 138)));
    assert_eq!(
      Color::parse("#E2BE8A22").map(|color| color.alpha),
      Some(0x22)
    );
  }

  #[test]
  fn rejects_malformed_colours() {
    for text in ["E2BE8A", "#E2BE8", "#GGGGGG", "#", "", "#E2BE8A2"] {
      assert_eq!(Color::parse(text), None, "{text}");
    }
  }

  #[test]
  fn formats_every_consumer_form() {
    let color = Color::parse("#E2BE8A22").unwrap();
    assert_eq!(color.hex(), "#E2BE8A");
    assert_eq!(color.hex_with_alpha(), "#E2BE8A22");
    assert_eq!(color.argb_hex(), "#22E2BE8A");
    assert_eq!(color.bare_hex(), "E2BE8A");
    assert_eq!(color.css_rgb(), "rgb(226, 190, 138)");
    assert_eq!(color.css_rgba(), "rgba(226, 190, 138, 0.133)");
    for (alpha, text) in [
      (224, "0.88"),
      (115, "0.45"),
      (255, "1"),
      (0, "0"),
      (34, "0.133"),
    ] {
      assert_eq!(
        color.with_opacity(f64::from(alpha) / 255.0).css_rgba(),
        format!("rgba(226, 190, 138, {text})")
      );
    }
    assert_eq!(color.hyprland(), "rgba(e2be8a22)");
    assert_eq!(color.css_rgba_percent(), "rgba(226, 190, 138, 13%)");
    assert_eq!(color.hex_rgb_function(), "rgb(e2be8a)");
    assert_eq!(color.argb_integer(), "0x22E2BE8A");
  }

  #[test]
  fn opacity_mix_lighten_and_darken() {
    let color = Color::opaque(100, 100, 100);
    assert_eq!(color.with_opacity(0.5).alpha, 128);
    assert_eq!(
      color.mix(Color::opaque(200, 0, 100), 0.5),
      Color::opaque(150, 50, 100)
    );
    assert_eq!(color.lighten(1.0), Color::opaque(255, 255, 255));
    assert_eq!(color.darken(1.0), Color::opaque(0, 0, 0));
    assert_eq!(color.with_opacity(0.5).darken(0.5).alpha, 128);
  }

  #[test]
  fn contrast_follows_the_accent_rule() {
    assert_eq!(Color::parse("#E2BE8A").unwrap().contrast(), Color::BLACK);
    assert_eq!(Color::parse("#3590BD").unwrap().contrast(), Color::WHITE);
    assert_eq!(Color::opaque(128, 128, 128).contrast(), Color::BLACK);
    assert_eq!(Color::opaque(127, 128, 128).contrast(), Color::WHITE);
  }

  #[test]
  fn decimals_drop_trailing_zeros() {
    assert_eq!(format_decimal(1.0), "1");
    assert_eq!(format_decimal(0.5), "0.5");
    assert_eq!(format_decimal(0.0), "0");
    assert_eq!(format_decimal(12.25), "12.25");
  }
}
