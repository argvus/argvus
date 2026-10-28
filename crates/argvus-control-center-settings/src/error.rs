//! Error types and presentation of the Settings pages.
#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
  #[error("default applications: {0}")]
  DefaultApps(String),
  #[error("fonts: {0}")]
  Fonts(String),
  #[error("font discovery: {0}")]
  FontDiscovery(String),
  #[error("system settings: {0}")]
  System(String),
}
