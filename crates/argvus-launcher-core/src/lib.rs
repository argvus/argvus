//! Local universal search. Preferences live in argvus-config; indexes are derived.
mod engine;
mod files;
mod providers;
pub mod search;
pub mod settings;

pub use engine::run_command;

/// Clears derived data and pauses indexing until explicitly resumed.
pub fn clear_index() -> Result<(), String> {
  let mut settings = settings::Settings::load()?;
  let previous = settings.clone();
  settings.index_paused = true;
  settings.save_changes(&previous)?;
  files::clear()
}

/// Count and timestamp from the rebuildable cache, for the settings page.
pub fn index_status() -> (usize, u64) {
  let index = files::load();
  (index.files.len(), index.scanned)
}

/// Invalidates the name cache. Current preferences decide whether it is rebuilt.
pub fn rebuild_index() -> Result<(), String> {
  files::clear()
}
