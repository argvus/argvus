//! Services page of the ARGVUS Control Center (systemd units and their logs).
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
mod backend;
mod journal;
mod model;
mod ui;

pub use backend::list as list_units;
pub use model::{ServicePage, Unit};
pub use ui::ServicesApp;
