//! Root module of crate `argvus control center dev services`. It exposes public boundaries and composes internal responsibilities without duplicating domain rules.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
mod backend;
mod model;
mod ui;

pub use model::{ContainerSummary, DevServicesSnapshot, ListeningPort};
pub use ui::DevServicesApp;
