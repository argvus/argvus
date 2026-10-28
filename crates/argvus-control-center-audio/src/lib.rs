//! Audio page of the ARGVUS Control Center (outputs, inputs and volume).
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
pub mod backend;
pub mod model;
pub mod ui;
pub use model::AudioPage;
pub use ui::AudioApp;
