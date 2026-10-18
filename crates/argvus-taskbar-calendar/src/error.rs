use thiserror::Error;

pub type Result<T> = std::result::Result<T, ArgvusError>;

#[derive(Debug, Error)]
pub enum ArgvusError {
  #[error("configuration error: {0}")]
  Configuration(String),
  #[error("database error: {0}")]
  Database(#[from] rusqlite::Error),
  #[error("iCalendar error: {0}")]
  Ics(String),
  #[error("CalDAV error: {0}")]
  #[allow(dead_code)]
  CalDav(String),
  #[error("network error: {0}")]
  Network(#[from] reqwest::Error),
  #[error("notification error: {0}")]
  Notification(String),
  #[error("I/O error: {0}")]
  Io(#[from] std::io::Error),
  #[error("serialization error: {0}")]
  Serialization(String),
}
