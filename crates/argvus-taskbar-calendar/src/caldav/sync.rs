use chrono::Utc;

use crate::error::Result;

use super::{CalDavClient, SyncReport};

pub struct CalDavSyncService {
  client: CalDavClient,
}

impl CalDavSyncService {
  pub fn new(client: CalDavClient) -> Self {
    Self { client }
  }

  pub async fn initial_sync(&self, collection_url: &str) -> Result<SyncReport> {
    let remote = self.client.list_objects(collection_url).await?;
    Ok(SyncReport {
      pulled: remote.len(),
      pushed: 0,
      conflicts: 0,
      finished_at: Utc::now(),
    })
  }
}
