//! Root module of crate `argvus control center storage`. It exposes public boundaries and composes internal responsibilities without duplicating domain rules.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
pub mod backend;
pub mod model;
mod ui;

pub use model::{StoragePage, StorageSnapshot};
pub use ui::StorageApp;
