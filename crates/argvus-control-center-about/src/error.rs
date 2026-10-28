//! Error types and presentation of the About page.
#[derive(Debug, thiserror::Error)]
pub enum AboutError {
  #[error("failed to open link: {0}")]
  OpenLink(#[source] std::io::Error),
}
