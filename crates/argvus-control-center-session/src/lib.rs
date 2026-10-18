//! Root module of crate `argvus control center session`. It exposes public boundaries and composes internal responsibilities without duplicating domain rules.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
mod backend;
mod journal;
mod model;
mod ui;

pub use model::{AutostartEntry, Component, ComponentStatus, DiagnosticsEntry, SessionPage};
pub use ui::SessionApp;
