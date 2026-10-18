//! Implements module declarations for the `system` subsystem in crate `argvus control center settings`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
pub mod clipboard;
pub mod command;
pub mod date;
pub mod fonts;
pub mod gpg_keys;
pub mod host;
pub mod input;
pub mod keybindings;
pub mod keyboard;
pub mod locale;
pub mod privileged;
pub mod projects;
pub mod ratbag;
pub mod snippets;
pub mod ssh_keys;
pub mod time;
pub mod window_rules;
