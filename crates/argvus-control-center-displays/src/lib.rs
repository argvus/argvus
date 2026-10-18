//! Root module of crate `argvus control center displays`. It exposes public boundaries and composes internal responsibilities without duplicating domain rules.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
mod backend;
mod model;
mod ui;

pub use model::{
  DisplayPage, DisplayState, Mode, Monitor, MonitorInfo, MonitorProfile, MonitorSetting,
  PersistedConfig, PersistedMonitor, PromptGoal,
};
pub use ui::DisplaysApp;
