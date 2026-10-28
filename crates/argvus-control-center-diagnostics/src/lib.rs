//! Diagnostics page of the ARGVUS Control Center (health checks of the desktop).
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
mod backend;
mod model;
mod ui;

pub use model::{DiagnosticCheck, DiagnosticFacts, DiagnosticPage, Severity};
pub use ui::DiagnosticsApp;
