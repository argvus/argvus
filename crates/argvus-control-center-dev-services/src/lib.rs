//! Developer services page of the ARGVUS Control Center (Docker and Podman services).
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
mod backend;
mod model;
mod ui;

pub use model::{ContainerSummary, DevServicesSnapshot, ListeningPort};
pub use ui::DevServicesApp;
