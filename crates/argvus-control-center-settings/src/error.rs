//! Implements error types and presentation in crate `argvus control center settings`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
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
