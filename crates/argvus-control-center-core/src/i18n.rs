//! Implements internationalization integration in crate `argvus control center core`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
pub use argvus_i18n::{Lang, na, tr};

/// Executes the `label` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn label(key: &str) -> &'static str {
  tr(Lang::detect(), key)
}
