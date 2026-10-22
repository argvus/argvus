//! Automatic privilege elevation through polkit's `pkexec`.
//!
//! Administrative operations (`name`/`avatar`/`groups` targeting another
//! user, and every password write) require effective root. Instead of asking
//! users to prefix commands with `sudo` or `pkexec`, the CLI re-executes
//! itself through `pkexec`, so the desktop polkit agent collects the
//! administrator authorization with the usual graphical prompt.
//!
//! The [`ELEVATED_ENV`] guard marks the elevated child, preventing infinite
//! recursion: if the child is still denied, the underlying error surfaces
//! instead of another elevation attempt.

use std::io::Write;
use std::os::unix::process::CommandExt as _;
use std::process::{Command, Stdio};

use argvus_accounts_core::passwd;
use argvus_accounts_core::{Error, Result};

/// Marks an already-elevated child process.
pub const ELEVATED_ENV: &str = "ARGVUS_ACCOUNTS_ELEVATED";

/// How a command requires privileges.
#[derive(Clone, Debug)]
pub enum Requirement {
  /// The operation always needs effective root (e.g. writing a password).
  Always,
  /// Root is only needed when acting on someone else's account; self
  /// operations keep using unprivileged tools (`chfn`, direct home writes).
  OtherUsersOnly {
    /// Target username of the operation.
    target: String,
  },
}

/// Elevates when required.
///
/// Returns normally when no elevation is needed (already root, already the
/// elevated child, or the requirement does not apply). When elevation is
/// required, this function replaces the process with `pkexec` and only
/// returns on failure (pkexec missing / exec refused).
pub fn ensure(requirement: &Requirement) -> Result<()> {
  if !elevation_needed(requirement)? {
    return Ok(());
  }
  exec_pkexec(None)
}

/// Like [`ensure`], but the elevated child reads `input` from its standard
/// input.
///
/// Used for secrets: the bytes travel through an anonymous pipe, so they never
/// appear in the argument vector or the environment of any process. The pipe
/// is filled before `exec`, so `input` must fit the pipe buffer; callers bound
/// it well below that.
pub fn ensure_with_input(requirement: &Requirement, input: &[u8]) -> Result<()> {
  if !elevation_needed(requirement)? {
    return Ok(());
  }
  let (reader, mut writer) =
    std::io::pipe().map_err(|err| Error::system(format!("failed to create pipe: {err}")))?;
  writer
    .write_all(input)
    .map_err(|err| Error::system(format!("failed to pass input to pkexec: {err}")))?;
  // Dropping the writer closes the pipe, so the child reads EOF after `input`.
  drop(writer);
  exec_pkexec(Some(Stdio::from(reader)))
}

/// Whether the requirement is unmet by the current process.
fn elevation_needed(requirement: &Requirement) -> Result<bool> {
  if passwd::effective_uid() == 0 || std::env::var_os(ELEVATED_ENV).is_some() {
    return Ok(false);
  }
  if let Requirement::OtherUsersOnly { target } = requirement {
    let me = passwd::current_user()?.username;
    if me == *target {
      return Ok(false);
    }
  }
  Ok(true)
}

/// Replaces the process with `pkexec`, running this same executable with the
/// original arguments. `stdin` is inherited unless a replacement is given.
fn exec_pkexec(stdin: Option<Stdio>) -> Result<()> {
  let program = locate_pkexec().ok_or_else(|| {
    Error::system("pkexec was not found; install 'polkit' or re-run the command with sudo")
  })?;
  let exe = std::env::current_exe()
    .map_err(|err| Error::system(format!("failed to resolve current executable: {err}")))?;

  eprintln!("argvus-accounts: requesting administrator privileges via polkit…");

  // pkexec sanitizes the inherited environment and rejects variables it
  // considers suspicious (e.g. TERM from the terminal).  Clear everything
  // and pass only what the polkit agent needs to receive the auth request.
  let mut cmd = Command::new(program);
  cmd
    .arg(exe)
    .args(std::env::args_os().skip(1))
    .env_clear()
    .env(ELEVATED_ENV, "1");

  if let Some(dbus) = std::env::var_os("DBUS_SESSION_BUS_ADDRESS") {
    cmd.env("DBUS_SESSION_BUS_ADDRESS", dbus);
  }
  if let Some(xdg) = std::env::var_os("XDG_RUNTIME_DIR") {
    cmd.env("XDG_RUNTIME_DIR", xdg);
  }
  if let Some(stdin) = stdin {
    cmd.stdin(stdin);
  }

  let err = cmd.exec(); // replaces this process; only returns on exec failure

  Err(Error::system(format!("failed to launch pkexec: {err}")))
}

fn locate_pkexec() -> Option<std::path::PathBuf> {
  let path = std::env::var_os("PATH")?;
  std::env::split_paths(&path)
    .map(|dir| dir.join("pkexec"))
    .find(|candidate| candidate.is_file())
}
