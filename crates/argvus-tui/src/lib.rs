//! Shared Ratatui widgets, menus, dialogs and chrome for the ARGVUS terminal interfaces.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
pub mod action_buttons;
pub mod buttons;
pub mod chrome;
pub mod components;
pub mod confirm;
pub mod filter_search;
pub mod hints;
pub mod icon_scale;
pub mod icons;
pub mod image;
pub mod menu;
pub mod page;
pub mod scroll;
pub mod terminal;
pub mod terminal_view;
pub mod text;
pub mod text_editor;

pub const MIN_WIDTH: u16 = 60;
pub const MIN_HEIGHT: u16 = 18;
