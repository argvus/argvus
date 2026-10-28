//! Boot page of the ARGVUS Control Center (GRUB and systemd-boot settings).
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
pub mod backend;
pub mod model;
pub mod ui;

pub use model::BootPage;
pub use ui::BootApp;
