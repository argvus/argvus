//! `monitor-*` subcommands: per-monitor state kept in the configuration store.

use super::*;

pub(super) fn monitor_set(args: &[String]) -> ExitCode {
  monitor_command(args, |store, args| {
    let [monitor, key, value] = args else {
      return Err(ConfigError::Invalid(
        "usage: argvus config monitor-set MONITOR KEY VALUE".into(),
      ));
    };
    monitor_state::set(store, monitor, key, value)
  })
}

pub(super) fn monitor_import(args: &[String]) -> ExitCode {
  monitor_command(args, |store, args| {
    let [file] = args else {
      return Err(ConfigError::Invalid(
        "usage: argvus config monitor-import STATE_FILE".into(),
      ));
    };
    monitor_state::import(store, Path::new(file))
  })
}

pub(super) fn monitor_primary(args: &[String]) -> ExitCode {
  monitor_command(args, |store, args| {
    let [monitor] = args else {
      return Err(ConfigError::Invalid(
        "usage: argvus config monitor-primary MONITOR".into(),
      ));
    };
    monitor_state::primary(store, monitor)
  })
}

pub(super) fn monitor_workspaces(args: &[String]) -> ExitCode {
  monitor_command(args, |store, args| {
    let [monitor, ids] = args else {
      return Err(ConfigError::Invalid(
        "usage: argvus config monitor-workspaces MONITOR IDS".into(),
      ));
    };
    monitor_state::workspaces(store, monitor, ids)
  })
}

pub(super) fn monitor_remove(args: &[String]) -> ExitCode {
  monitor_command(args, |store, args| {
    let [monitor] = args else {
      return Err(ConfigError::Invalid(
        "usage: argvus config monitor-remove MONITOR".into(),
      ));
    };
    monitor_state::remove(store, monitor)
  })
}

pub(super) fn monitor_sync(args: &[String]) -> ExitCode {
  monitor_command(args, |store, args| {
    let [file] = args else {
      return Err(ConfigError::Invalid(
        "usage: argvus config monitor-sync STATE_FILE".into(),
      ));
    };
    monitor_state::sync_legacy(store, Path::new(file))
  })
}

/// Runs one monitor command against the canonical store and prints its error,
/// so the display script can report it. The script keeps its own messages.
pub(super) fn monitor_command(
  arguments: &[String],
  operation: impl FnOnce(&ConfigStore, &[String]) -> ConfigResult<()>,
) -> ExitCode {
  let store = match ConfigStore::from_environment() {
    Ok(store) => store,
    Err(error) => {
      eprintln!("argvus-config: {error}");
      return ExitCode::from(1);
    }
  };
  match operation(&store, arguments) {
    Ok(()) => ExitCode::SUCCESS,
    Err(error) => {
      eprintln!("argvus-config: {error}");
      ExitCode::from(1)
    }
  }
}
