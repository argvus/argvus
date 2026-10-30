//! Rules of the ARGVUS system monitor: which btop theme and accent apply, the
//! theme and configuration files btop reads, and how the command line selects
//! the boxes. Nothing here reads files or starts processes.

pub mod btop;
pub mod command;
pub mod gpu;
pub mod theme;
