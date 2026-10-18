//! Implements shared filesystem path resolution in crate `argvus control center settings`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
pub use argvus_control_center_core::paths::{
  argvus_config_home, argvus_data_home, config_home, fonts_file, home, system_config_root,
};
