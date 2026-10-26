//! Theme palettes, templates and builds for ARGVUS.
//!
//! A theme package ships data: a [`manifest::ThemeManifest`] with a palette of
//! named colours. Each component ships the templates of its own files and
//! registers them in the [`registry`]. A theme switch renders every template
//! into a build ([`build::render`]), writes it beside the previous builds and
//! points the `current` symlink at it ([`build::commit`]). Consumers read their
//! files through `current`, so a switch never changes a file under a reader.
//!
//! This crate holds no TUI code; `argvus-theme` resolves the TUI palette.

pub mod build;
pub mod cli;
pub mod color;
pub mod discovery;
pub mod error;
pub mod expr;
pub mod layout;
pub mod manifest;
pub mod package;
pub mod paths;
pub mod registry;
pub mod render;
pub mod template;

pub use cli::run_command;
pub use error::ThemeError;
