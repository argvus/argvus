//! Session subcommands run by the ARGVUS session: loading screen, prepare, config sync and reload, restart and the reload plan.

use super::theme_commands::cache_home;
use super::*;

/// The commands directory the session scripts use (`ARGVUS_COMMANDS_DIR`).
pub(super) fn commands_dir() -> PathBuf {
  env::var_os("ARGVUS_COMMANDS_DIR")
    .map(PathBuf::from)
    .unwrap_or_else(|| PathBuf::from("/usr/lib/argvus/commands"))
}

/// Runs the session-loading spinner until the compositor accepts it.
pub(super) fn session_loading() -> ExitCode {
  let Some(paths) = session_paths::SessionPaths::from_environment() else {
    eprintln!("argvus-config: configuration directory is unavailable");
    return ExitCode::from(1);
  };
  let runtime_dir = env::var_os("XDG_RUNTIME_DIR")
    .filter(|value| !value.is_empty())
    .map(PathBuf::from)
    .unwrap_or_else(|| {
      // SAFETY: getuid has no preconditions and cannot fail.
      let uid = unsafe { libc::getuid() };
      PathBuf::from(format!("/run/user/{uid}"))
    });
  let status = session_loading::run(&paths, &runtime_dir, &session_loading::spinner_binary());
  ExitCode::from(status.clamp(0, 255) as u8)
}

/// Runs the session preparation (the startup step that used to live in hypr-init.sh).
pub(super) fn session_prepare() -> ExitCode {
  let Some(paths) = session_paths::SessionPaths::from_environment() else {
    eprintln!("argvus-config: configuration directory is unavailable");
    return ExitCode::from(1);
  };
  let cache = match cache_home() {
    Ok(cache) => cache,
    Err(error) => {
      eprintln!("argvus-config: {error}");
      return ExitCode::from(1);
    }
  };
  let status = session_prepare::prepare(&paths, &commands_dir(), &cache);
  ExitCode::from(status.clamp(0, 255) as u8)
}

/// Synchronizes the Rofi, btop and yazi files without reloading anything.
/// Session preparation uses it; errors are ignored, as the shell ignored them.
pub(super) fn session_sync_config() -> ExitCode {
  let Some(paths) = session_paths::SessionPaths::from_environment() else {
    return ExitCode::SUCCESS;
  };
  let commands_dir = commands_dir();
  let _ = session_reload::sync_user_files(&session_reload::ReloadContext {
    paths: &paths,
    commands_dir: &commands_dir,
  });
  ExitCode::SUCCESS
}

/// Runs the runtime half of a configuration reload (see `session_reload`).
/// `--login` runs the reduced login variant (`session_reload::reload_at_login`).
pub(super) fn session_reload_config(arguments: &[String]) -> ExitCode {
  if arguments.iter().any(|argument| argument == "--login") {
    let status = session_reload::reload_at_login();
    return ExitCode::from(status.clamp(0, 255) as u8);
  }
  let Some(paths) = session_paths::SessionPaths::from_environment() else {
    eprintln!("argvus-config: configuration directory is unavailable");
    return ExitCode::from(1);
  };
  let commands_dir = commands_dir();
  let status = session_reload::reload_config(&session_reload::ReloadContext {
    paths: &paths,
    commands_dir: &commands_dir,
  });
  ExitCode::from(status.clamp(0, 255) as u8)
}

/// Restarts the user units that serve the named components.
///
/// Unknown components exit with 64 before any unit is touched, the code the
/// session script used before this moved to Rust.
pub(super) fn session_restart(components: &[String]) -> ExitCode {
  let names: Vec<&str> = components.iter().map(String::as_str).collect();
  let cache = match cache_home() {
    Ok(cache) => cache,
    Err(error) => {
      eprintln!("argvus-config: {error}");
      return ExitCode::from(1);
    }
  };
  let units = match session_units::units_for(&names, || session_units::telemetry_enabled(&cache)) {
    Ok(units) => units,
    Err(unknown) => {
      eprintln!("argvus-config: unknown session component: {}", unknown.0);
      return ExitCode::from(64);
    }
  };
  if units.is_empty() {
    return ExitCode::SUCCESS;
  }
  match std::process::Command::new("systemctl")
    .args(["--user", "restart"])
    .args(&units)
    .status()
  {
    Ok(status) if status.success() => ExitCode::SUCCESS,
    Ok(status) => ExitCode::from(status.code().unwrap_or(1).clamp(1, 255) as u8),
    Err(error) => {
      eprintln!("argvus-config: failed to run systemctl: {error}");
      ExitCode::from(1)
    }
  }
}

/// Prints the reload decision for the latest projection. A missing or
/// unreadable manifest means a full reload, as the session scripts assumed.
pub(super) fn print_session_plan(store: &ConfigStore) -> Result<(), Box<dyn std::error::Error>> {
  use crate::{project::ArgvusPaths, session_plan};
  let manifest_path = ArgvusPaths::from_store(store)?.projection_manifest;
  let manifest: Value = fs::read_to_string(&manifest_path)
    .ok()
    .and_then(|text| serde_json::from_str(&text).ok())
    .unwrap_or(Value::Null);
  print!(
    "{}",
    session_plan::render(&session_plan::decide_reload(&manifest))
  );
  Ok(())
}
