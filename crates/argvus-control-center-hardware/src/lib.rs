//! Root module of crate `argvus control center hardware`. It exposes public boundaries and composes internal responsibilities without duplicating domain rules.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
mod backend;
mod model;
mod ui;

pub use model::{HardwarePage, HardwareSnapshot};
pub use ui::HardwareApp;
