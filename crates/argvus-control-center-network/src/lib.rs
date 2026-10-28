//! Network page of the ARGVUS Control Center (NetworkManager connections and profiles).
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
pub mod backend;
pub mod model;
pub mod profile;
pub mod ui;

pub use model::NetworkPage;
pub use ui::NetworkApp;
