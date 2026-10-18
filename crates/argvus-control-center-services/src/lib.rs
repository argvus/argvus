//! Root module of crate `argvus control center services`. It exposes public boundaries and composes internal responsibilities without duplicating domain rules.
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
