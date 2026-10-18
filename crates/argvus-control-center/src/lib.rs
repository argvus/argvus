//! Root module of crate `argvus control center`. It exposes public boundaries and composes internal responsibilities without duplicating domain rules.
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
