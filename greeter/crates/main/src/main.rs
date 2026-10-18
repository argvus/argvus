//! Process entry point for the Argvus greetd frontend.
//!
//! This module deliberately keeps startup orchestration small. Discovery and
//! policy live in dedicated modules so the binary can be audited as a login
//! component without mixing terminal rendering, authentication, and system
//! configuration concerns.

mod app;
mod config;
mod greetd;
mod handoff;
mod i18n;
mod power;
mod session;
mod ui;
mod users;

use anyhow::Context;
use std::{
  env,
  fs::{self, File, OpenOptions},
  path::PathBuf,
};
use tracing_subscriber::{EnvFilter, fmt};

/// Returns a monotonic timestamp suitable for correlating login-transition events.
pub(crate) fn monotonic_ns() -> u128 {
  let mut value = libc::timespec {
    tv_sec: 0,
    tv_nsec: 0,
  };
  // CLOCK_MONOTONIC has no relation to wall-clock time and is therefore safe
  // for measuring a transition even when NTP adjusts the system clock.
  if unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut value) } == 0 {
    value.tv_sec as u128 * 1_000_000_000 + value.tv_nsec as u128
  } else {
    0
  }
}

fn main() -> anyhow::Result<()> {
  // Logging is initialized before any discovery work so configuration and
  // environment failures remain visible when the greeter runs under greetd.
  init_logging();

  // These objects are collected once at startup. The login UI then owns the
  // immutable discovery snapshot while only the selected indices change.
  let config = config::Config::load().context("failed to load greeter configuration")?;
  let users = users::discover_users().context("failed to discover local users")?;
  let sessions = session::discover_sessions(&config.session.default)
    .context("failed to discover graphical sessions")?;
  let i18n = i18n::load().context("failed to load greeter translations")?;

  app::run(config, users, sessions, i18n)
}

fn init_logging() {
  // Respect RUST_LOG when operators need diagnostics, while keeping normal
  // login screens quiet by default. Logs must never share Kitty's stdout or
  // stderr because those streams are the Ratatui drawing surface.
  let filter = EnvFilter::try_from_default_env()
    .unwrap_or_else(|_| EnvFilter::new("argvus_greeter=info,warn"));

  match open_log_file() {
    Some(file) => fmt()
      .with_env_filter(filter)
      .with_target(false)
      .compact()
      .with_writer(file)
      .init(),
    None => fmt()
      .with_env_filter(filter)
      .with_target(false)
      .compact()
      // If runtime storage is unavailable, discard diagnostics rather than
      // corrupting the interactive login surface with terminal log lines.
      .with_writer(std::io::sink)
      .init(),
  }
}

/// Opens the private diagnostic log used by the pre-authentication process.
///
/// Runtime state is preferred because it is session-scoped. The state-home
/// fallback keeps development launches diagnosable when greetd does not set
/// `XDG_RUNTIME_DIR`; the final temporary path avoids writing to the terminal.
fn open_log_file() -> Option<File> {
  let state_dir = env::var_os("XDG_RUNTIME_DIR")
    .map(PathBuf::from)
    .map(|path| path.join("argvus-greeter"))
    .or_else(|| {
      env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .map(|path| path.join("argvus-greeter"))
    })
    .unwrap_or_else(|| env::temp_dir().join(format!("argvus-greeter-{}", std::process::id())));

  fs::create_dir_all(&state_dir).ok()?;
  OpenOptions::new()
    .create(true)
    .append(true)
    .open(state_dir.join("greeter.log"))
    .ok()
}
