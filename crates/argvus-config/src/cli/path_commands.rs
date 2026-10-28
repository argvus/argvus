//! Path subcommands used by the shell helpers: the live copy of a file and its writable location.

use super::*;

/// `read-path RELATIVE`: prints the file a reader sees for a packaged config
/// path, with the same cascade as the shell's `paths_config`. The packaged
/// file is printed when no copy exists.
pub(super) fn read_path(arguments: &[String]) -> ExitCode {
  let [relative] = arguments else {
    eprintln!("usage: argvus config read-path RELATIVE");
    return ExitCode::from(64);
  };
  let Some(paths) = session_paths::SessionPaths::from_environment() else {
    eprintln!("argvus-config: cannot resolve the configuration directory");
    return ExitCode::from(1);
  };
  println!("{}", paths.read_config(relative).display());
  ExitCode::SUCCESS
}

/// `mutable-path RELATIVE`: prints the user copy of a packaged file, creating it
/// first when needed. The shell helper `paths_ensure_generated_copy` delegates here.
pub(super) fn mutable_path(arguments: &[String]) -> ExitCode {
  let [relative] = arguments else {
    eprintln!("usage: argvus config mutable-path RELATIVE");
    return ExitCode::from(64);
  };
  let Some(paths) = session_paths::SessionPaths::from_environment() else {
    eprintln!("argvus-config: cannot resolve the configuration directory");
    return ExitCode::from(1);
  };
  match user_copy::ensure_user_copy(&paths, relative) {
    Ok(path) => {
      println!("{}", path.display());
      ExitCode::SUCCESS
    }
    Err(error) => {
      eprintln!("argvus-config: {relative}: {error}");
      ExitCode::from(1)
    }
  }
}
