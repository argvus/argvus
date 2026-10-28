//! Domain state and models consumed by the UI of the Audio page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioPage {
  Home,
  Summary,
  Output,
  Input,
  Devices,
}
#[derive(Debug, Clone, Default, PartialEq)]
pub struct AudioSnapshot {
  pub available: bool,
  pub backend: String,
  pub outputs: Vec<AudioDevice>,
  pub inputs: Vec<AudioDevice>,
  pub default_output: Option<u32>,
  pub default_input: Option<u32>,
}
#[derive(Debug, Clone, Default, PartialEq)]
pub struct AudioDevice {
  pub id: u32,
  pub name: String,
  pub description: String,
  pub direction: String,
  pub volume: Option<u8>,
  pub muted: bool,
  pub is_default: bool,
}
