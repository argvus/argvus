//! Running the programs the telemetry commands depend on. A program that is
//! missing or fails is an ordinary outcome for the widget, so none of these
//! helpers panic or print.

use std::env;
use std::io::Read;
use std::path::Path;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

/// The pause between checks of a program that runs under a time limit.
const POLL: Duration = Duration::from_millis(20);

/// Whether a program exists in one of the directories of `PATH`.
pub fn program_exists(program: &str) -> bool {
  env::var_os("PATH")
    .map(|paths| env::split_paths(&paths).any(|directory| directory.join(program).is_file()))
    .unwrap_or(false)
}

/// Runs a program with its output discarded, and reports whether it succeeded.
pub fn run_quiet(program: &str, arguments: &[&str]) -> bool {
  Command::new(program)
    .args(arguments)
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .status()
    .map(|status| status.success())
    .unwrap_or(false)
}

/// The standard output of a program that succeeded, or `None` when it did not
/// run or failed.
pub fn output(program: &str, arguments: &[&str]) -> Option<String> {
  let output = Command::new(program)
    .args(arguments)
    .stderr(Stdio::null())
    .output()
    .ok()?;
  output
    .status
    .success()
    .then(|| String::from_utf8_lossy(&output.stdout).into_owned())
}

/// The standard output of a program, whatever its status; empty when it does
/// not run. Used where the shell read output with `$(...)` and ignored failure.
pub fn output_any(program: &str, arguments: &[&str]) -> String {
  output_with_environment(program, arguments, &[])
}

/// Like `output_any`, with extra environment variables for the program.
pub fn output_with_environment(
  program: &str,
  arguments: &[&str],
  environment: &[(&str, &str)],
) -> String {
  Command::new(program)
    .args(arguments)
    .envs(environment.iter().copied())
    .stderr(Stdio::null())
    .output()
    .map(|output| String::from_utf8_lossy(&output.stdout).into_owned())
    .unwrap_or_default()
}

/// What a program printed and how it ended, when it ended in time.
pub struct Finished {
  pub stdout: String,
  pub status: ExitStatus,
}

/// Runs a program and waits at most `limit`. A program that is still running at
/// the limit is killed, and `None` is returned.
pub fn run_with_limit(program: &str, arguments: &[&str], limit: Duration) -> Option<Finished> {
  let mut child: Child = Command::new(program)
    .args(arguments)
    .stdout(Stdio::piped())
    .stderr(Stdio::null())
    .spawn()
    .ok()?;
  let deadline = Instant::now() + limit;
  loop {
    match child.try_wait().ok()? {
      Some(status) => {
        let mut stdout = String::new();
        if let Some(mut pipe) = child.stdout.take() {
          let _ = pipe.read_to_string(&mut stdout);
        }
        return Some(Finished { stdout, status });
      }
      None if Instant::now() >= deadline => {
        let _ = child.kill();
        let _ = child.wait();
        return None;
      }
      None => thread::sleep(POLL),
    }
  }
}

/// Reads a file as text, or `None` when it cannot be read.
pub fn read_text(path: &Path) -> Option<String> {
  std::fs::read_to_string(path).ok()
}

/// Shows a desktop notification, when `notify-send` is installed. A failure to
/// show it is not an error for the command.
pub fn notify(title: &str, body: &str) {
  if program_exists("notify-send") {
    let _ = Command::new("notify-send")
      .args([title, body])
      .stdout(Stdio::null())
      .stderr(Stdio::null())
      .status();
  }
}
