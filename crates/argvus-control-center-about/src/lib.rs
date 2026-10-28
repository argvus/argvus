//! About page of the ARGVUS Control Center (system, version and environment information).
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
pub mod app;
pub mod error;
pub mod image;
pub mod pages;
pub mod system;
pub mod ui;

mod i18n {
  pub use argvus_i18n::*;
}

mod theme {
  pub use argvus_theme::*;
}

pub use app::{App, Tab};
pub use error::AboutError;
