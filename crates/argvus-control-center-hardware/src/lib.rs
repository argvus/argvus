//! Hardware page of the ARGVUS Control Center (detected devices and their details).
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
mod backend;
mod model;
mod ui;

pub use model::{HardwarePage, HardwareSnapshot};
pub use ui::HardwareApp;
