//! Shared building blocks of the ARGVUS Control Center pages: jobs, processes, privileges, navigation, search and configuration helpers.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
pub mod capabilities;
pub mod clipboard;
pub mod config;
pub mod format;
pub mod i18n;
pub mod jobs;
pub mod navigation;
pub mod pacman_conf;
pub mod paths;
pub mod picker;
pub mod privileged;
pub mod process;
pub mod sanitize;
pub mod search;
