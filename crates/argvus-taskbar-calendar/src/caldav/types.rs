use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct CalDavAccount {
  pub id: Option<i64>,
  pub name: String,
  pub server_url: String,
  pub username: String,
  pub secret_lookup: String,
  pub sync_enabled: bool,
}

#[derive(Debug, Clone)]
pub struct CalendarCollection {
  pub href: String,
  pub display_name: String,
  pub ctag: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RemoteObject {
  pub href: String,
  pub etag: Option<String>,
  pub ics: String,
  pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub enum ConflictPolicy {
  PreserveBoth,
}

#[derive(Debug, Clone)]
pub struct SyncReport {
  pub pulled: usize,
  pub pushed: usize,
  pub conflicts: usize,
  pub finished_at: DateTime<Utc>,
}
