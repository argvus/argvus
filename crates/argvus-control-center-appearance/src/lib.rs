//! Root module of crate `argvus control center appearance`. It exposes public boundaries and composes internal responsibilities without duplicating domain rules.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
mod animations;
mod backend;
mod blur;
mod icons;
mod model;
mod profile;
mod ui;

pub use model::{
  ACCENTS, AppearancePage, AppearanceState, ControlPanelCard, ControlPanelCards, CustomTheme,
  PromptGoal, TaskbarPosition, TaskbarUtilityGroupMode, WallpaperCollection, WallpaperEntry,
  WallpaperMode, WidgetTelemetryBlock, WidgetTelemetryBlocks,
};
pub use ui::AppearanceApp;

#[cfg(test)]
mod test_support {
  use std::sync::{Mutex, MutexGuard, PoisonError};

  /// Serializes every test that repoints `HOME` or the XDG directories.
  ///
  /// `std::env` is process-global and the test harness runs tests on parallel
  /// threads, so a per-module lock would not be enough: two modules mutating
  /// `HOME` under different locks would still interleave and produce
  /// failures that depend on scheduling. One crate-wide lock is the only way
  /// these tests are deterministic.
  pub static ENV_LOCK: Mutex<()> = Mutex::new(());

  /// Acquires [`ENV_LOCK`], tolerating a mutex poisoned by an unrelated failure
  /// so one broken test does not cascade into every other test.
  pub fn env_lock() -> MutexGuard<'static, ()> {
    ENV_LOCK.lock().unwrap_or_else(PoisonError::into_inner)
  }
}
