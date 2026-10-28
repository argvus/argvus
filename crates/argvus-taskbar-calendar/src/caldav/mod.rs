//! CalDAV foundation: HTTP client, discovery, credentials and sync service.
//!
//! The module is compiled but not yet wired into the calendar UI or the
//! reminder service (see `docs/internal/argvus-taskbar-calendar/wiki.md`), so
//! its items and re-exports are unused until that integration lands.
#![allow(dead_code, unused_imports)]

pub mod client;
pub mod credentials;
pub mod discovery;
pub mod sync;
pub mod types;

pub use client::*;
pub use credentials::*;
pub use discovery::*;
pub use sync::*;
pub use types::*;
