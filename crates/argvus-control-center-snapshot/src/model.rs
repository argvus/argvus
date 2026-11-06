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

/// Origin of a retention value returned by Snapper.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SettingOrigin {
  Explicit,
  /// Legacy wire name for the built-in fallback when get-config omits a key.
  SnapperDefault,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetentionValue {
  pub value: String,
  pub origin: SettingOrigin,
}

/// Known settings are typed by validation and unknown keys remain diagnostic-only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetentionConfig {
  pub values: std::collections::BTreeMap<String, RetentionValue>,
  pub unknown: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CleanupAlgorithm {
  Number,
  Timeline,
  EmptyPrePost,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimerState {
  Missing,
  Disabled,
  EnabledInactive,
  Active,
  Unavailable(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapperTimers {
  pub timeline: TimerState,
  pub cleanup: TimerState,
}

impl CleanupAlgorithm {
  pub fn as_snapper_arg(self) -> &'static str {
    match self {
      Self::Number => "number",
      Self::Timeline => "timeline",
      Self::EmptyPrePost => "empty-pre-post",
    }
  }
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
  OutputLimit,
  PackageUpdateBusy,
  RestoreBusy,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Comparison {
  pub unsupported: usize,
  pub truncated: bool,
  pub total_seen: usize,
  pub origin: u32,
  pub destination: u32,
  pub changes: Vec<ChangedPath>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeKind {
  Created,
  Removed,
  Modified,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(into = "WireChangedPath", try_from = "WireChangedPath")]
pub struct ChangedPath {
  pub path: String,
  pub indicators: String,
  pub kind: ChangeKind,
  pub content: bool,
  pub file_type: bool,
  pub permissions: bool,
  pub owner: bool,
  pub group: bool,
  pub xattrs: bool,
  pub acl: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileDiff {
  Text(String),
  Binary,
  Truncated,
}

/// Metadata used to detect a destination change between review and confirmation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestoreFileIdentity {
  pub device: u64,
  pub inode: u64,
  pub size: u64,
  pub modified_seconds: i64,
  pub modified_nanoseconds: i64,
  pub changed_seconds: i64,
  pub changed_nanoseconds: i64,
  pub owner: u32,
  pub group: u32,
  pub mode: u32,
  pub links: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestoreReview {
  pub source_snapshot: u32,
  pub path: String,
  pub destination: Option<RestoreFileIdentity>,
  pub diff: FileDiff,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestoreResult {
  pub source_snapshot: u32,
  pub safety_snapshot: u32,
  pub path: String,
}

#[derive(Serialize, Deserialize)]
struct WireChangedPath {
  path: String,
  indicators: String,
}
impl From<ChangedPath> for WireChangedPath {
  fn from(change: ChangedPath) -> Self {
    Self {
      path: change.path,
      indicators: change.indicators,
    }
  }
}
impl TryFrom<WireChangedPath> for ChangedPath {
  type Error = String;
  fn try_from(change: WireChangedPath) -> Result<Self, Self::Error> {
    crate::backend::parse_status(format!("{} {}", change.indicators, change.path).as_bytes())
      .map_err(|error| error.to_string())?
      .pop()
      .ok_or_else(|| "empty change".into())
  }
}
