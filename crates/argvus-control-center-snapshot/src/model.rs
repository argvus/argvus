//! Validated snapshot state shared by the UI and the restricted helper.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Snapshot {
  pub number: u32,
  pub kind: String,
  pub date: String,
  pub description: String,
  pub cleanup: String,
  pub active: bool,
  pub default: bool,
}

impl Snapshot {
  pub fn deletable(&self) -> bool {
    self.number > 0 && !self.active && !self.default
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Integration {
  NonBtrfs(String),
  MissingSnapper,
  NoConfig,
  ChooseConfig(Vec<String>),
  Ready {
    config: String,
    snapshots: Vec<Snapshot>,
  },
}

/// Stable error kinds allow translated UI text without matching arbitrary stderr.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorKind {
  Detection,
  Permission,
  Authentication,
  Timeout,
  Incompatible,
  Invalid,
  Snapper,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotError {
  pub kind: ErrorKind,
  pub detail: String,
}
impl SnapshotError {
  pub fn new(kind: ErrorKind, detail: impl Into<String>) -> Self {
    Self {
      kind,
      detail: detail.into(),
    }
  }
}
impl std::fmt::Display for SnapshotError {
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(formatter, "{:?}: {}", self.kind, self.detail)
  }
}
impl std::error::Error for SnapshotError {}

pub type SnapshotResult<T> = Result<T, SnapshotError>;
