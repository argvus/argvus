//! Implements error types and presentation in crate `argvus about`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
#[derive(Debug, thiserror::Error)]
pub enum AboutError {
  #[error("failed to open link: {0}")]
  OpenLink(#[source] std::io::Error),
}
