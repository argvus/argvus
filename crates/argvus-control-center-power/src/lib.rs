//! Power page of the ARGVUS Control Center (profiles, battery and sleep).
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
mod backend;
mod hypridle;
mod model;
mod ui;

pub use model::{LidContext, PowerBehavior, PowerButtonBehavior, PowerPage, PowerState};
pub use ui::PowerApp;
