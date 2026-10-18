//! Root module of crate `argvus control center audio`. It exposes public boundaries and composes internal responsibilities without duplicating domain rules.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
pub mod backend;
pub mod model;
pub mod ui;
pub use model::AudioPage;
pub use ui::AudioApp;
