//! Implements domain state and models consumed by the UI in crate `argvus control center audio`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioPage {
  Home,
  Summary,
  Output,
  Input,
  Devices,
}
#[derive(Debug, Clone, Default, PartialEq)]
/// Represents `AudioSnapshot`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct AudioSnapshot {
  pub available: bool,
  pub backend: String,
  pub outputs: Vec<AudioDevice>,
  pub inputs: Vec<AudioDevice>,
  pub default_output: Option<u32>,
  pub default_input: Option<u32>,
}
#[derive(Debug, Clone, Default, PartialEq)]
/// Represents `AudioDevice`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct AudioDevice {
  pub id: u32,
  pub name: String,
  pub description: String,
  pub direction: String,
  pub volume: Option<u8>,
  pub muted: bool,
  pub is_default: bool,
}
