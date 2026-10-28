//! ARGVUS Control Center: Home, navigation, search, CLI routes and the page shell.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
pub mod app;
pub mod cli;
pub mod config_app;
pub mod event;
pub mod home;
pub mod hyprland;
pub mod settings_hub;
pub mod ui;
