//! Discovery of the calendar collections of a CalDAV account.

use crate::error::Result;

use super::{CalDavClient, CalendarCollection};

pub async fn discover(client: &CalDavClient) -> Result<Vec<CalendarCollection>> {
  client.discover_collections().await
}
