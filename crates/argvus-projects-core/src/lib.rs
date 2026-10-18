//! Project switcher for ARGVUS: the project list, the workspaces the projects open in,
//! and the rules of the dispatch and picker layers.

pub mod projects;

pub use projects::{MAX_PROJECTS, lua_string, resolve, workspace_name};
