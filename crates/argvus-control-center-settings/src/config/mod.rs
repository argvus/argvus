//! Implements module declarations for the `config` subsystem in crate `argvus control center settings`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
pub mod apps;
pub mod fonts;
pub mod paths;
