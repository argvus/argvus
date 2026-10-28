//! Domain state and models consumed by the UI of the Services page.
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
  pub fn failed(&self) -> bool {
    self.active == "failed"
  }
  pub fn valid_name(name: &str) -> bool {
    !name.is_empty()
      && !name.chars().any(|c| c.is_control() || c.is_whitespace())
      && name.ends_with(".service")
      && name.len() < 256
  }
}
