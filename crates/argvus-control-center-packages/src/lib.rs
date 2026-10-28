//! Packages page of the ARGVUS Control Center (pacman, AUR and repositories).
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
pub mod backend;
pub mod model;
pub mod ui;
pub use model::{PackageSource, PackageTask, PackagesPage};
pub use ui::PackagesApp;
