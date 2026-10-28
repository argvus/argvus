//! Domain state and models consumed by the UI of the Diagnostics page.
use argvus_control_center_storage::StorageSnapshot;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
pub enum Severity {
  Error,
  Warning,
  #[default]
  Unknown,
  Info,
  Ok,
}

#[derive(Debug, Clone, Default)]
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
