//! Implements domain state and models consumed by the UI in crate `argvus control center services`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServicePage {
  Home,
  System,
  User,
  Failed,
  Logs,
  Detail,
  LogDetail(usize),
}
#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// Represents `Unit`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct Unit {
  pub name: String,
  pub description: String,
  pub load: String,
  pub active: String,
  pub sub: String,
  pub file_state: String,
  pub main_pid: Option<u32>,
  pub fragment: Option<String>,
  pub scope: String,
}
impl Unit {
  /// Executes the `failed` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn failed(&self) -> bool {
    self.active == "failed"
  }
  /// Executes the `valid_name` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn valid_name(name: &str) -> bool {
    !name.is_empty()
      && !name.chars().any(|c| c.is_control() || c.is_whitespace())
      && name.ends_with(".service")
      && name.len() < 256
  }
}
