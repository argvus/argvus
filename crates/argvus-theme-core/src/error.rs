//! Errors of theme loading, rendering and building.

use crate::template::TemplateError;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum ThemeError {
  #[error("{}: {source}", path.display())]
  Io {
    path: PathBuf,
    #[source]
    source: io::Error,
  },
  #[error("{}: {message}", path.display())]
  Invalid { path: PathBuf, message: String },
  #[error("{consumer} ({}): {error}", template.display())]
  Render {
    consumer: String,
    template: PathBuf,
    error: TemplateError,
  },
}

impl ThemeError {
  pub(crate) fn io(path: &Path, source: io::Error) -> Self {
    Self::Io {
      path: path.to_path_buf(),
      source,
    }
  }

  pub(crate) fn invalid(path: &Path, message: impl Into<String>) -> Self {
    Self::Invalid {
      path: path.to_path_buf(),
      message: message.into(),
    }
  }
}

/// Reads a whole text file, naming the path on failure.
pub(crate) fn read_text(path: &Path) -> Result<String, ThemeError> {
  std::fs::read_to_string(path).map_err(|source| ThemeError::io(path, source))
}
