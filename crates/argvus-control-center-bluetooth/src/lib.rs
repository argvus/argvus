//! Bluetooth page of the ARGVUS Control Center (adapters, pairing and devices).
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
pub mod agent;
pub mod backend;
pub mod model;
pub mod ui;
pub use model::BluetoothPage;
pub use ui::BluetoothApp;
