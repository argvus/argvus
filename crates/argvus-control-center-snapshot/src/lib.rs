//! Root Btrfs snapshot management through Snapper, with an injectable process boundary.
pub mod backend;
pub mod model;
mod ui;
pub use ui::SnapshotApp;
