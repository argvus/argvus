//! Shell commands launched by the module: detached launches, captured runs,
//! quoting and desktop notifications.

use std::io::Write;
use std::process::{Command, Stdio};

// Launch a command without waiting (fire and forget), detached from the
// terminal. A thread reaps the child, so the long-running `watch` does not
// collect zombies. Returns the pid, or -1 on failure.
pub fn fork_exec(command: &str) -> i32 {
  use std::os::unix::process::CommandExt;
  match Command::new("/bin/sh")
    .arg("-c")
    .arg(command)
    .stdin(Stdio::null())
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .process_group(0)
    .spawn()
  {
    Ok(mut child) => {
      let pid = child.id() as i32;
      std::thread::spawn(move || {
        let _ = child.wait();
      });
      pid
    }
    Err(_) => -1,
  }
}

// Run a command, feeding `input` lines to its stdin, capturing stdout+stderr.
// Returns Some((stdout, stderr)) on success (exit code 0), None otherwise.
pub fn run_capture(command: &str, input: &[String]) -> Option<(String, String)> {
  let mut child = Command::new("/bin/sh")
    .arg("-c")
    .arg(command)
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .ok()?;
  {
    let mut stdin = child.stdin.take()?;
    for line in input {
      if stdin.write_all(line.as_bytes()).is_err() {
        break;
      }
      if stdin.write_all(b"\n").is_err() {
        break;
      }
    }
  }
  let output = child.wait_with_output().ok()?;
  if !output.status.success() {
    return None;
  }
  Some((
    String::from_utf8_lossy(&output.stdout).into_owned(),
    String::from_utf8_lossy(&output.stderr).into_owned(),
  ))
}

// Desktop notification via notify-send (best effort).
pub fn notify(summary: &str, body: &str, critical: bool) {
  let mut command = String::from("notify-send --app-name=argvus-removable-devices ");
  command += if critical {
    "-u critical "
  } else {
    "-u normal "
  };
  command += &shell_quote(summary);
  if !body.is_empty() {
    command.push(' ');
    command += &shell_quote(body);
  }
  fork_exec(&command);
}

/// Quotes `text` as one POSIX shell word, so it can be appended to a command
/// line that runs through `/bin/sh -c`.
pub fn shell_quote(text: &str) -> String {
  let mut quoted = String::from("'");
  for character in text.chars() {
    if character == '\'' {
      quoted.push_str("'\\''");
    } else {
      quoted.push(character);
    }
  }
  quoted.push('\'');
  quoted
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn launched_commands_are_reaped() {
    let pid = fork_exec("exit 0");
    assert!(pid > 0);
    let status = std::path::PathBuf::from(format!("/proc/{pid}"));
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while status.exists() && std::time::Instant::now() < deadline {
      std::thread::sleep(std::time::Duration::from_millis(20));
    }
    assert!(!status.exists(), "child {pid} was left as a zombie");
  }

  #[test]
  fn shell_quotes() {
    assert_eq!(shell_quote("abc"), "'abc'");
    assert_eq!(shell_quote("a'b"), "'a'\\''b'");
  }
}
