//! Hyprland blur parameters edited on the Blur page.
//!
//! The values are stored by `argvus-config` under `effects.blur_*` and reach
//! the compositor through `argvus-hyprland`. This module owns their bounds,
//! steps, display format and parsing, so the page, the backend and the tests
//! agree. The bounds must match the schema in `argvus-config`.

/// Editable blur parameters, in the order the page lists them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlurField {
  Size,
  Passes,
  Brightness,
  Noise,
  Contrast,
  Vibrancy,
  VibrancyDarkness,
}

impl BlurField {
  pub const ALL: [Self; 7] = [
    Self::Size,
    Self::Passes,
    Self::Brightness,
    Self::Noise,
    Self::Contrast,
    Self::Vibrancy,
    Self::VibrancyDarkness,
  ];

  /// Name used in the `effects.blur_<name>` config key and in the script
  /// arguments.
  pub const fn name(self) -> &'static str {
    match self {
      Self::Size => "size",
      Self::Passes => "passes",
      Self::Brightness => "brightness",
      Self::Noise => "noise",
      Self::Contrast => "contrast",
      Self::Vibrancy => "vibrancy",
      Self::VibrancyDarkness => "vibrancy_darkness",
    }
  }

  /// Translation key of the row label.
  pub const fn label_key(self) -> &'static str {
    match self {
      Self::Size => "control_center.blur_size",
      Self::Passes => "control_center.blur_passes",
      Self::Brightness => "control_center.blur_brightness",
      Self::Noise => "control_center.blur_noise",
      Self::Contrast => "control_center.blur_contrast",
      Self::Vibrancy => "control_center.blur_vibrancy",
      Self::VibrancyDarkness => "control_center.blur_vibrancy_darkness",
    }
  }

  /// Value written when the config has no usable entry.
  pub const fn default_value(self) -> f64 {
    match self {
      Self::Size => 6.0,
      Self::Passes => 2.0,
      Self::Brightness => 1.0,
      Self::Noise => 0.0,
      Self::Contrast => 0.9,
      Self::Vibrancy => 0.1,
      Self::VibrancyDarkness => 0.0,
    }
  }

  /// Inclusive bounds accepted for this field.
  pub const fn bounds(self) -> (f64, f64) {
    match self {
      Self::Size => (1.0, 64.0),
      Self::Passes => (1.0, 8.0),
      Self::Brightness | Self::Contrast => (0.0, 2.0),
      Self::Noise | Self::Vibrancy | Self::VibrancyDarkness => (0.0, 1.0),
    }
  }

  /// Whether the field only takes whole numbers.
  pub const fn is_integer(self) -> bool {
    matches!(self, Self::Size | Self::Passes)
  }

  /// Amount added by one `←/→` step.
  pub const fn step(self) -> f64 {
    match self {
      Self::Size | Self::Passes => 1.0,
      Self::Brightness | Self::Contrast => 0.05,
      Self::Noise | Self::Vibrancy | Self::VibrancyDarkness => 0.01,
    }
  }

  /// Display form on the page: whole numbers as-is, noise with two decimals and
  /// contrast/vibrancy with six.
  pub fn format(self, value: f64) -> String {
    match self {
      Self::Size | Self::Passes | Self::Brightness => format!("{value}"),
      Self::Noise => format!("{value:.2}"),
      Self::Contrast | Self::Vibrancy | Self::VibrancyDarkness => format!("{value:.6}"),
    }
  }

  /// Clamps `value` to the bounds and rounds it to the precision of the step,
  /// so repeated `←/→` presses do not accumulate floating-point drift.
  pub fn normalize(self, value: f64) -> f64 {
    let (minimum, maximum) = self.bounds();
    let rounded = if self.is_integer() {
      value.round()
    } else {
      (value * 1000.0).round() / 1000.0
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

/// The blur parameters, indexed by [`BlurField`] (same order as `BlurField::ALL`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlurValues([f64; 7]);

impl Default for BlurValues {
  fn default() -> Self {
    Self(BlurField::ALL.map(BlurField::default_value))
  }
}

impl BlurValues {
  pub fn get(self, field: BlurField) -> f64 {
    self.0[field as usize]
  }

  pub fn set(&mut self, field: BlurField, value: f64) {
    self.0[field as usize] = field.normalize(value);
  }

  /// Moves `field` by `steps` of its own step size, within its bounds.
  pub fn adjust(&mut self, field: BlurField, steps: i32) {
    let next = self.get(field) + f64::from(steps) * field.step();
    self.set(field, next);
  }

  /// `name=value` pairs for `argvus effects-toggle blur-settings set`.
  pub fn script_arguments(self) -> Vec<String> {
    BlurField::ALL
      .iter()
      .map(|field| format!("{}={}", field.name(), field.format(self.get(*field))))
      .collect()
  }

  /// Reads `name=value` lines from `argvus effects-toggle blur-settings get`.
  /// Missing or invalid entries keep their defaults.
  pub fn from_script_output(output: &str) -> Self {
    let mut values = Self::default();
    for line in output.lines() {
      let Some((name, raw)) = line.split_once('=') else {
        continue;
      };
      let Some(field) = BlurField::ALL
        .into_iter()
        .find(|field| field.name() == name)
      else {
        continue;
      };
      if let Some(value) = field.parse(raw) {
        values.set(field, value);
      }
    }
    values
  }
}

/// What the Blur page edits: the global enable switch and the parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlurSettings {
  pub enabled: bool,
  pub values: BlurValues,
}

impl Default for BlurSettings {
  fn default() -> Self {
    Self {
      enabled: true,
      values: BlurValues::default(),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn defaults_match_the_values_shown_on_the_page() {
    let values = BlurValues::default();
    assert_eq!(BlurField::Size.format(values.get(BlurField::Size)), "6");
    assert_eq!(BlurField::Passes.format(values.get(BlurField::Passes)), "2");
    assert_eq!(
      BlurField::Brightness.format(values.get(BlurField::Brightness)),
      "1"
    );
    assert_eq!(
      BlurField::Noise.format(values.get(BlurField::Noise)),
      "0.00"
    );
    assert_eq!(
      BlurField::Contrast.format(values.get(BlurField::Contrast)),
      "0.900000"
    );
    assert_eq!(
      BlurField::Vibrancy.format(values.get(BlurField::Vibrancy)),
      "0.100000"
    );
  }

  #[test]
  fn adjust_steps_by_field_and_stays_in_bounds() {
    let mut values = BlurValues::default();
    values.adjust(BlurField::Size, -10);
    assert_eq!(values.get(BlurField::Size), 1.0);
    values.adjust(BlurField::Size, 100);
    assert_eq!(values.get(BlurField::Size), 64.0);

    values.adjust(BlurField::Contrast, 1);
    assert!((values.get(BlurField::Contrast) - 0.95).abs() < 1e-9);
    values.adjust(BlurField::Noise, 1);
    assert_eq!(values.get(BlurField::Noise), 0.01);
  }

  #[test]
  fn parse_rejects_out_of_range_and_fractional_integers() {
    assert_eq!(BlurField::Size.parse("0"), None);
    assert_eq!(BlurField::Size.parse("65"), None);
    assert_eq!(BlurField::Passes.parse("2.5"), None);
    assert_eq!(BlurField::Contrast.parse("2.5"), None);
    assert_eq!(BlurField::Contrast.parse(" 0.75 "), Some(0.75));
    assert_eq!(BlurField::Noise.parse("nan"), None);
  }

  #[test]
  fn script_round_trip_keeps_the_values() {
    let mut values = BlurValues::default();
    values.set(BlurField::Size, 12.0);
    values.set(BlurField::Brightness, 0.8);
    let output = values.script_arguments().join("\n");
    assert_eq!(BlurValues::from_script_output(&output), values);
  }

  #[test]
  fn invalid_script_entries_fall_back_to_defaults() {
    let values = BlurValues::from_script_output("size=abc\npasses=99\nunknown=3\n");
    assert_eq!(values, BlurValues::default());
  }
}
