//! Window deformation parameters. Bounds follow Hyprland v0.57.0's
//! `src/config/values/ConfigValues.cpp`, including intensity's maximum of 2.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WobbleField {
  Stiffness,
  Damping,
  Mass,
  Intensity,
  Mesh,
}

impl WobbleField {
  pub const ALL: [Self; 5] = [
    Self::Stiffness,
    Self::Damping,
    Self::Mass,
    Self::Intensity,
    Self::Mesh,
  ];

  pub const fn name(self) -> &'static str {
    match self {
      Self::Stiffness => "stiffness",
      Self::Damping => "damping",
      Self::Mass => "mass",
      Self::Intensity => "intensity",
      Self::Mesh => "mesh",
    }
  }

  pub const fn label_key(self) -> &'static str {
    match self {
      Self::Stiffness => "control_center.wobble_stiffness",
      Self::Damping => "control_center.wobble_damping",
      Self::Mass => "control_center.wobble_mass",
      Self::Intensity => "control_center.wobble_intensity",
      Self::Mesh => "control_center.wobble_mesh",
    }
  }

  pub const fn bounds(self) -> (f64, f64) {
    match self {
      Self::Stiffness | Self::Mass => (0.0001, 1000.0),
      Self::Damping => (0.0, 1000.0),
      Self::Intensity => (0.0, 2.0),
      Self::Mesh => (2.0, 32.0),
    }
  }

  pub const fn default_value(self) -> f64 {
    match self {
      Self::Stiffness => 100.0,
      Self::Damping => 10.0,
      Self::Mass | Self::Intensity => 1.0,
      Self::Mesh => 8.0,
    }
  }

  pub const fn step(self) -> f64 {
    match self {
      Self::Stiffness => 10.0,
      Self::Damping | Self::Mesh => 1.0,
      Self::Mass | Self::Intensity => 0.1,
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WobbleSettings {
  pub enabled: bool,
  pub values: [f64; 5],
}

impl Default for WobbleSettings {
  fn default() -> Self {
    Self {
      enabled: false,
      values: WobbleField::ALL.map(WobbleField::default_value),
    }
  }
}
