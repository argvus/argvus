//! Implements domain state and models consumed by the UI in crate `argvus control center diagnostics`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use argvus_control_center_storage::StorageSnapshot;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Defines `DiagnosticPage`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub enum DiagnosticPage {
  Home,
  Summary,
  Services,
  Boot,
  Graphics,
  Network,
  Audio,
  Bluetooth,
  Storage,
  Packages,
  Argvus,
  Detail(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Default)]
/// Defines `Severity`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub enum Severity {
  Error,
  Warning,
  #[default]
  Unknown,
  Info,
  Ok,
}

#[derive(Debug, Clone, Default)]
/// Represents `DiagnosticCheck`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct DiagnosticCheck {
  pub id: String,
  pub category: String,
  pub severity: Severity,
  pub title: String,
  pub summary: String,
  pub details: String,
  pub remediation_hint: Option<String>,
  pub route: Option<String>,
}

#[derive(Debug, Clone, Default)]
/// Represents `DiagnosticFacts`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct DiagnosticFacts {
  pub storage: StorageSnapshot,
  pub package_updates: Option<usize>,
  pub package_lock: bool,
  pub failed_system_units: Option<usize>,
  pub failed_user_units: Option<usize>,
  pub nvidia_present: Option<bool>,
  pub nvidia_module_loaded: Option<bool>,
  pub opengl_renderer: Option<String>,
  pub vulkan_available: Option<bool>,
}
