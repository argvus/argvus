//! Session page of the ARGVUS Control Center (autostart, idle and the session journal).
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
mod backend;
mod journal;
mod model;
mod ui;

pub use model::{AutostartEntry, Component, ComponentStatus, DiagnosticsEntry, SessionPage};
pub use ui::SessionApp;
