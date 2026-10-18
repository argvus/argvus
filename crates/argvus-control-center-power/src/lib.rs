//! Root module of crate `argvus control center power`. It exposes public boundaries and composes internal responsibilities without duplicating domain rules.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
mod backend;
mod hypridle;
mod model;
mod ui;

pub use model::{LidContext, PowerBehavior, PowerButtonBehavior, PowerPage, PowerState};
pub use ui::PowerApp;
