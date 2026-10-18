//! Process helpers for the dispatcher routes: the failure that ends a run, the
//! `exec` replacement and the executable lookup.

use std::env;
use std::ffi::OsString;
use std::fmt;
use std::fs;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

/// Status used when a route cannot start its program, the same status the
/// shell `exec` reports.
const EXEC_FAILED_CODE: u8 = 126;

/// Ends the dispatcher. An empty message means the child already printed its
/// own output and only its exit status is returned.
#[derive(Debug)]
pub struct Failure {
  message: String,
  code: u8,
}

/// The result of a route. Routes that replace the process only return on
/// failure, so a successful outcome is always a child that finished normally.
pub type Outcome = Result<ExitCode, Failure>;

impl Failure {
  /// A dispatcher error, printed as `argvus: <message>` with status 1.
  pub fn new(message: impl fmt::Display) -> Self {
    Self {
      message: format!("argvus: {message}"),
      code: 1,
    }
  }

  /// A `setup` error, printed as `argvus --setup: <message>` with status 1.
  pub fn setup(message: impl fmt::Display) -> Self {
    Self {
      message: format!("argvus --setup: {message}"),
      code: 1,
    }
  }

  /// A dispatcher error with its own exit status, printed as `argvus: <message>`.
  pub fn with_code(message: impl fmt::Display, code: u8) -> Self {
    Self {
      message: format!("argvus: {message}"),
      code,
    }
  }

  /// The status of a child that failed. The child already reported the error,
  /// so nothing is printed.
  pub fn exit_status(code: u8) -> Self {
    Self {
      message: String::new(),
      code,
    }
  }

  /// The message printed on stderr. Empty when the child reported its own error.
  pub fn message(&self) -> &str {
    &self.message
  }

  /// The exit status of the dispatcher.
  pub fn code(&self) -> u8 {
    self.code
  }
}

/// Replaces this process with `command`. It returns only when the program cannot
/// be started, so the caller gets the failure to report.
pub fn replace_process(command: &mut Command) -> Failure {
  let program = command.get_program().to_string_lossy().into_owned();
  let error = command.exec();
  Failure {
    message: format!("argvus: cannot run {program}: {error}"),
    code: EXEC_FAILED_CODE,
  }
}

/// Reads an environment variable, treating an empty value as unset. This matches
/// the `${VAR:-default}` expansion used throughout the shell dispatcher.
pub fn env_nonempty(name: &str) -> Option<OsString> {
  env::var_os(name).filter(|value| !value.is_empty())
}

/// Whether `path` is a regular file with at least one execute bit set.
pub fn is_executable(path: &Path) -> bool {
  fs::metadata(path)
    .is_ok_and(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
}

/// Finds `name` in the directories of `PATH`, in order, like `command -v`.
pub fn find_on_path(name: &str) -> Option<PathBuf> {
  let search_path = env::var_os("PATH")?;
  env::split_paths(&search_path)
    .map(|directory| directory.join(name))
    .find(|candidate| is_executable(candidate))
}

/// Converts an I/O error on `path` into a dispatcher failure.
pub fn io_failure(path: &Path, error: io::Error) -> Failure {
  Failure::new(format!("{}: {error}", path.display()))
}
