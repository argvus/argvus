use crate::error::Result;

use super::{CalDavClient, CalendarCollection};

pub async fn discover(client: &CalDavClient) -> Result<Vec<CalendarCollection>> {
  client.discover_collections().await
}
