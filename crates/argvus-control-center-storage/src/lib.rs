//! Storage page of the ARGVUS Control Center (disks, partitions and usage).
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
pub mod backend;
pub mod model;
mod ui;

pub use model::{StoragePage, StorageSnapshot};
pub use ui::StorageApp;
