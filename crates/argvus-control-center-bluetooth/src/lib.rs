//! Root module of crate `argvus control center bluetooth`. It exposes public boundaries and composes internal responsibilities without duplicating domain rules.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
pub mod agent;
pub mod backend;
pub mod model;
pub mod ui;
pub use model::BluetoothPage;
pub use ui::BluetoothApp;
