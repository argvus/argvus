//! Window shadow parameters edited on the Shadow page (Borders > Shadow).
//!
//! The values are stored by `argvus-config` under `effects.shadow_*` and reach
//! the compositor through `argvus-hyprland`, which maps them to
//! `decoration.shadow`. This module owns their bounds, steps, display format,
//! color rules and parsing, so the page, the backend and the tests agree. The
//! bounds must match the schema in `argvus-config`.

/// Numeric shadow parameters, in the order the page lists them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShadowField {
  Range,
  RenderPower,
  OffsetX,
  OffsetY,
  Scale,
}

impl ShadowField {
  pub const ALL: [Self; 5] = [
    Self::Range,
    Self::RenderPower,
    Self::OffsetX,
    Self::OffsetY,
    Self::Scale,
  ];

  /// Name used in the `effects.shadow_<name>` config key and in the script
  /// arguments.
  pub const fn name(self) -> &'static str {
    match self {
      Self::Range => "range",
      Self::RenderPower => "render_power",
      Self::OffsetX => "offset_x",
      Self::OffsetY => "offset_y",
      Self::Scale => "scale",
    }
  }

  /// Translation key of the row label.
  pub const fn label_key(self) -> &'static str {
    match self {
      Self::Range => "control_center.shadow_range",
      Self::RenderPower => "control_center.shadow_render_power",
      Self::OffsetX => "control_center.shadow_offset_x",
      Self::OffsetY => "control_center.shadow_offset_y",
      Self::Scale => "control_center.shadow_scale",
    }
  }

  /// Value written when the config has no usable entry.
  pub const fn default_value(self) -> f64 {
    match self {
      Self::Range => 6.0,
      Self::RenderPower => 6.0,
      Self::OffsetX | Self::OffsetY => 0.0,
      Self::Scale => 1.0,
    }
  }

  /// Inclusive bounds accepted for this field.
  pub const fn bounds(self) -> (f64, f64) {
    match self {
      Self::Range => (0.0, 100.0),
      Self::RenderPower => (1.0, 10.0),
      Self::OffsetX | Self::OffsetY => (-100.0, 100.0),
      Self::Scale => (0.0, 1.0),
    }
  }

  /// Whether the field only takes whole numbers.
  pub const fn is_integer(self) -> bool {
    !matches!(self, Self::Scale)
  }

  /// Amount added by one `←/→` step.
  pub const fn step(self) -> f64 {
    match self {
      Self::Scale => 0.05,
      _ => 1.0,
    }
  }

  /// Display form on the page: whole numbers as-is, the scale with two decimals.
  pub fn format(self, value: f64) -> String {
    match self {
      Self::Scale => format!("{value:.2}"),
      _ => format!("{value}"),
    }
  }

  /// Clamps `value` to the bounds and rounds it to the precision of the step.
  pub fn normalize(self, value: f64) -> f64 {
    let (minimum, maximum) = self.bounds();
    let rounded = if self.is_integer() {
      value.round()
    } else {
      (value * 100.0).round() / 100.0
    };
    rounded.clamp(minimum, maximum)
  }

  /// Parses text typed in the prompt or read from the config. Returns `None`
  /// for anything that is not a number inside the bounds.
  pub fn parse(self, text: &str) -> Option<f64> {
    let value: f64 = text.trim().parse().ok()?;
    let (minimum, maximum) = self.bounds();
    let in_bounds = value.is_finite() && (minimum..=maximum).contains(&value);
    let shape_ok = !self.is_integer() || value.fract() == 0.0;
    (in_bounds && shape_ok).then_some(value)
  }
}

/// The numeric shadow parameters, indexed by [`ShadowField`] (same order as
/// `ShadowField::ALL`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShadowValues([f64; 5]);

impl Default for ShadowValues {
  fn default() -> Self {
    Self(ShadowField::ALL.map(ShadowField::default_value))
  }
}

impl ShadowValues {
  pub fn get(self, field: ShadowField) -> f64 {
    self.0[field as usize]
  }

  pub fn set(&mut self, field: ShadowField, value: f64) {
    self.0[field as usize] = field.normalize(value);
  }

  /// Moves `field` by `steps` of its own step size, within its bounds.
  pub fn adjust(&mut self, field: ShadowField, steps: i32) {
    let next = self.get(field) + f64::from(steps) * field.step();
    self.set(field, next);
  }
}

/// The shadow color. `FollowAccent` keeps the shadow on the accent of the active
/// theme with a `99` alpha, so it changes with the theme; `Custom` is an
/// `RRGGBBAA` value fixed by the user.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ShadowColor {
  #[default]
  FollowAccent,
  /// Eight lowercase hex digits, `rrggbbaa`.
  Custom(String),
}

/// Alpha of the accent-derived shadow, the `99` of the default `rgba(…99)`.
const ACCENT_SHADOW_ALPHA: &str = "99";

impl ShadowColor {
  /// Reads the color typed in the prompt or stored in the config. An empty text
  /// follows the accent; anything else must be eight hex digits.
  pub fn parse(text: &str) -> Option<Self> {
    let text = text.trim();
    if text.is_empty() {
      return Some(Self::FollowAccent);
    }
    let is_rgbaa = text.len() == 8 && text.bytes().all(|byte| byte.is_ascii_hexdigit());
    is_rgbaa.then(|| Self::Custom(text.to_ascii_lowercase()))
  }

  /// The eight-digit value the compositor receives for the given accent
  /// (`#RRGGBB`), as shown on the page.
  pub fn effective(&self, accent: &str) -> String {
    match self {
      Self::FollowAccent => format!(
        "{}{ACCENT_SHADOW_ALPHA}",
        accent.trim_start_matches('#').to_ascii_lowercase()
      ),
      Self::Custom(color) => color.clone(),
    }
  }

  /// The text stored by `argvus-config` (`""` follows the accent).
  pub fn stored(&self) -> &str {
    match self {
      Self::FollowAccent => "",
      Self::Custom(color) => color,
    }
  }
}

/// What the Shadow page edits: the global enable switch, the sharp corners
/// switch, the color and the numeric parameters.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ShadowSettings {
  pub enabled: bool,
  pub sharp: bool,
  pub color: ShadowColor,
  pub values: ShadowValues,
}

impl ShadowSettings {
  /// `name=value` pairs for `argvus config effects shadow-settings set`, without
  /// the master switch (`enabled`), which is sent separately.
  pub fn script_arguments(&self) -> Vec<String> {
    let mut arguments = vec![
      format!("sharp={}", self.sharp),
      format!("color={}", self.color.stored()),
    ];
    arguments.extend(
      ShadowField::ALL
        .iter()
        .map(|field| format!("{}={}", field.name(), field.format(self.values.get(*field)))),
    );
    arguments
  }

  /// Reads `name=value` lines from `argvus config effects shadow-settings get`.
  /// Missing or invalid entries keep their defaults.
  pub fn from_script_output(output: &str) -> Self {
    let mut settings = Self::default();
    for line in output.lines() {
      let Some((name, raw)) = line.split_once('=') else {
        continue;
      };
      match name {
        "enabled" => settings.enabled = raw == "true",
        "sharp" => settings.sharp = raw == "true",
        "color" => {
          if let Some(color) = ShadowColor::parse(raw) {
            settings.color = color;
          }
        }
        _ => {
          let Some(field) = ShadowField::ALL
            .into_iter()
            .find(|field| field.name() == name)
          else {
            continue;
          };
          if let Some(value) = field.parse(raw) {
            settings.values.set(field, value);
          }
        }
      }
    }
    settings
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn defaults_match_the_values_shown_on_the_page() {
    let settings = ShadowSettings::default();
    assert!(!settings.enabled);
    assert!(!settings.sharp);
    assert_eq!(settings.color, ShadowColor::FollowAccent);
    assert_eq!(
      ShadowField::Range.format(settings.values.get(ShadowField::Range)),
      "6"
    );
    assert_eq!(
      ShadowField::RenderPower.format(settings.values.get(ShadowField::RenderPower)),
      "6"
    );
    assert_eq!(
      ShadowField::Scale.format(settings.values.get(ShadowField::Scale)),
      "1.00"
    );
  }

  #[test]
  fn accent_color_gets_the_default_alpha() {
    assert_eq!(ShadowColor::FollowAccent.effective("#3590bd"), "3590bd99");
    assert_eq!(
      ShadowColor::Custom("00000099".into()).effective("#3590bd"),
      "00000099"
    );
  }

  #[test]
  fn color_parse_accepts_empty_or_rrggbbaa_only() {
    assert_eq!(ShadowColor::parse(""), Some(ShadowColor::FollowAccent));
    assert_eq!(
      ShadowColor::parse("aabbcc11"),
      Some(ShadowColor::Custom("aabbcc11".into()))
    );
    assert_eq!(ShadowColor::parse("aabbcc"), None);
    assert_eq!(ShadowColor::parse("aabbccgg"), None);
  }

  #[test]
  fn adjust_steps_by_field_and_stays_in_bounds() {
    let mut values = ShadowValues::default();
    values.adjust(ShadowField::Range, -10);
    assert_eq!(values.get(ShadowField::Range), 0.0);
    values.adjust(ShadowField::OffsetX, 300);
    assert_eq!(values.get(ShadowField::OffsetX), 100.0);
    values.adjust(ShadowField::Scale, -1);
    assert!((values.get(ShadowField::Scale) - 0.95).abs() < 1e-9);
  }

  #[test]
  fn parse_rejects_out_of_range_and_fractional_integers() {
    assert_eq!(ShadowField::RenderPower.parse("0"), None);
    assert_eq!(ShadowField::Range.parse("2.5"), None);
    assert_eq!(ShadowField::OffsetY.parse("-12"), Some(-12.0));
    assert_eq!(ShadowField::Scale.parse("1.5"), None);
  }

  #[test]
  fn script_round_trip_keeps_the_settings() {
    let settings = ShadowSettings {
      enabled: false,
      sharp: true,
      color: ShadowColor::Custom("aabbcc11".into()),
      values: {
        let mut values = ShadowValues::default();
        values.set(ShadowField::Range, 12.0);
        values.set(ShadowField::Scale, 0.5);
        values
      },
    };
    let output = format!("enabled=false\n{}", settings.script_arguments().join("\n"));
    assert_eq!(ShadowSettings::from_script_output(&output), settings);
  }

  #[test]
  fn invalid_script_entries_fall_back_to_defaults() {
    let settings = ShadowSettings::from_script_output("range=abc\ncolor=zz\nunknown=3\n");
    assert_eq!(settings, ShadowSettings::default());
  }
}
