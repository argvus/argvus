//! Root module of crate `argvus tui`. It exposes public boundaries and composes internal responsibilities without duplicating domain rules.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
pub mod action_buttons;
pub mod buttons;
pub mod chrome;
pub mod components;
pub mod confirm;
pub mod hints;
pub mod icon_scale;
pub mod icons;
pub mod image;
pub mod menu;
pub mod page;
pub mod terminal;
pub mod text;

/// Defines the constant `MIN_WIDTH`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub const MIN_WIDTH: u16 = 60;
/// Defines the constant `MIN_HEIGHT`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub const MIN_HEIGHT: u16 = 18;
