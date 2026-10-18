//! Implements `session` responsibilities in crate `argvus about`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use std::env;

/// Executes the `session_name_from` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn session_name_from(kind: Option<&str>, desktop: Option<&str>) -> Option<String> {
  match (kind, desktop) {
    (Some(kind), Some(desktop)) if !desktop.is_empty() => Some(format!("{kind} ({desktop})")),
    (Some(kind), _) => Some(kind.to_string()),
    (_, Some(desktop)) if !desktop.is_empty() => Some(desktop.to_string()),
    _ => None,
  }
}

/// Executes the `session_name` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn session_name() -> Option<String> {
  let kind = env::var("XDG_SESSION_TYPE").ok();
  let desktop = env::var("XDG_CURRENT_DESKTOP")
    .or_else(|_| env::var("XDG_SESSION_DESKTOP"))
    .ok();
  session_name_from(kind.as_deref(), desktop.as_deref())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  /// Executes the `combines_session_type_and_desktop` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn combines_session_type_and_desktop() {
    assert_eq!(
      session_name_from(Some("wayland"), Some("Hyprland")).as_deref(),
      Some("wayland (Hyprland)")
    );
  }

  #[test]
  /// Processes `handles_partial_values` in this module's event flow. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn handles_partial_values() {
    assert_eq!(
      session_name_from(Some("wayland"), None).as_deref(),
      Some("wayland")
    );
    assert_eq!(
      session_name_from(None, Some("Hyprland")).as_deref(),
      Some("Hyprland")
    );
    assert_eq!(session_name_from(None, None), None);
  }
}
