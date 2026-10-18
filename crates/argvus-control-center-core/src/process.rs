//! Implements controlled external-process execution in crate `argvus control center core`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! The runner receives the program and each argument separately; it never passes the
//! request through a shell. This boundary matters because backends may receive
//! names, filters, and paths from the interface. Timeout handling is also centralized
//! here so a missing or stalled tool cannot block the
//! TUI rendering loop.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use std::ffi::OsStr;
use std::io::{self, BufRead, BufReader};
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};
use std::sync::{
  Arc, Mutex,
  atomic::{AtomicBool, Ordering},
};
use std::thread;
use std::time::{Duration, Instant};

/// Path that test builds start instead of the requested program. It never
/// exists, so spawning fails with `NotFound` before anything runs.
#[cfg(all(feature = "testing", not(test)))]
const DISABLED_PROGRAM: &str = "/nonexistent/argvus-control-center-test-process";

/// Builds the [`Command`] for an external program. Every process the
/// Control Center starts goes through here, directly or through
/// [`SystemProcessRunner`], so the `testing` feature can guarantee that no
/// test runs a system tool (`pkexec`, `pacman`, `systemctl`, `hyprctl`,
/// `argvus-config`...): the spawn fails as if the tool were not installed,
/// which every caller already handles. Without the feature this is
/// `Command::new(program)`.
pub fn command(program: impl AsRef<OsStr>) -> Command {
  #[cfg(all(feature = "testing", not(test)))]
  let program = {
    let _ = program;
    DISABLED_PROGRAM
  };
  Command::new(program)
}

#[derive(Debug, Clone)]
/// Represents `ProcessRequest`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct ProcessRequest {
  /// Executable name resolved by the system, without shell interpolation.
  pub program: String,
  /// Structured arguments in the same order in which they are passed to the process.
  pub args: Vec<String>,
  /// Environment overrides passed directly to the child process.
  pub env: Vec<(String, String)>,
  /// Optional deadline for terminating the operation and avoiding an indefinite wait.
  pub timeout: Option<Duration>,
}

impl ProcessRequest {
  /// Constructs `new` with this module's expected initial state. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn new(program: impl Into<String>) -> Self {
    Self {
      program: program.into(),
      args: Vec::new(),
      env: Vec::new(),
      timeout: None,
    }
  }

  /// Executes the `arg` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn arg(mut self, value: impl Into<String>) -> Self {
    self.args.push(value.into());
    self
  }

  /// Adds an environment override without invoking a shell.
  pub fn env(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
    self.env.push((name.into(), value.into()));
    self
  }

  /// Executes the `timeout` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn timeout(mut self, timeout: Duration) -> Self {
    self.timeout = Some(timeout);
    self
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Represents `ProcessOutput`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct ProcessOutput {
  /// Standard output preserved as bytes so each backend can choose its decoding.
  pub stdout: Vec<u8>,
  /// Error output kept separate from data output for diagnosis without polluting the UI.
  pub stderr: Vec<u8>,
  /// Exit code when the process finished normally.
  pub status: Option<i32>,
  /// Indicates that the runner terminated the process after the configured deadline.
  pub timed_out: bool,
}

#[derive(Debug)]
/// Defines `ProcessError`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub enum ProcessError {
  Io(io::Error),
  EmptyProgram,
}

impl std::fmt::Display for ProcessError {
  /// Executes the `fmt` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Io(error) => write!(formatter, "process failed: {error}"),
      Self::EmptyProgram => formatter.write_str("process program cannot be empty"),
    }
  }
}

impl std::error::Error for ProcessError {}

#[derive(Debug, Default)]
/// Represents `LiveProcess`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct LiveProcess {
  /// State shared by stdout/stderr readers and the rendering thread.
  inner: Arc<LiveInner>,
}
#[derive(Debug, Default)]
/// Represents `LiveInner`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
struct LiveInner {
  /// Lines already received, kept in order for incremental display.
  lines: Mutex<Vec<String>>,
  /// Atomic signal that avoids blocking readers during shutdown.
  finished: AtomicBool,
}

impl LiveProcess {
  /// Constructs `new` with this module's expected initial state. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn new() -> Self {
    Self::default()
  }

  /// Executes the `push_line` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn push_line(&self, line: &str) {
    if let Ok(mut lines) = self.inner.lines.lock() {
      lines.push(line.to_owned());
    }
  }

  /// Executes the `output` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn output(&self) -> String {
    self
      .inner
      .lines
      .lock()
      .map(|lines| lines.join("\n"))
      .unwrap_or_default()
  }

  /// Checks the condition represented by `is_finished` using only the state available to the module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn is_finished(&self) -> bool {
    self.inner.finished.load(Ordering::Acquire)
  }

  /// Executes the `finish` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn finish(&self) {
    self.inner.finished.store(true, Ordering::Release);
  }
}

impl Clone for LiveProcess {
  /// Executes the `clone` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn clone(&self) -> Self {
    Self {
      inner: Arc::clone(&self.inner),
    }
  }
}

/// Defines the `ProcessRunner`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub trait ProcessRunner: Send + Sync {
  /// Executes the `run` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn run(&self, request: &ProcessRequest) -> Result<ProcessOutput, ProcessError>;

  /// Executes the `run_live` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn run_live(
    &self,
    request: &ProcessRequest,
    _live: &LiveProcess,
  ) -> Result<ProcessOutput, ProcessError> {
    self.run(request)
  }
}

#[derive(Debug, Default, Clone, Copy)]
/// Represents `SystemProcessRunner`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct SystemProcessRunner;

impl ProcessRunner for SystemProcessRunner {
  /// Executes the `run` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn run(&self, request: &ProcessRequest) -> Result<ProcessOutput, ProcessError> {
    let _measurement = ProcessMeasurement::new(request);
    if request.program.trim().is_empty() {
      return Err(ProcessError::EmptyProgram);
    }
    let mut child = command(&request.program)
      .process_group(0)
      .args(&request.args)
      .envs(request.env.iter().map(|(name, value)| (name, value)))
      .stdin(Stdio::null())
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .spawn()
      .map_err(ProcessError::Io)?;
    let stdout = child
      .stdout
      .take()
      .ok_or_else(|| ProcessError::Io(io::Error::other("stdout was not captured")))?;
    let stderr = child
      .stderr
      .take()
      .ok_or_else(|| ProcessError::Io(io::Error::other("stderr was not captured")))?;
    // Drain both pipes on dedicated threads so commands with large output
    // (e.g. `pacman -Sl`, which emits hundreds of KB) do not fill the pipe
    // buffer while the parent polls `try_wait`, which would block the child
    // until the timeout kills it.
    let stdout_reader = std::thread::spawn(|| read_pipe(stdout));
    let stderr_reader = std::thread::spawn(|| read_pipe(stderr));
    // An absolute deadline prevents polling delays from accumulating on each iteration.
    let deadline = request.timeout.map(|timeout| Instant::now() + timeout);
    loop {
      match child.try_wait().map_err(ProcessError::Io)? {
        Some(status) => {
          if request.timeout.is_some() {
            terminate_descendants(child.id());
          }
          let stdout = stdout_reader.join().unwrap_or_default();
          let stderr = stderr_reader.join().unwrap_or_default();
          return Ok(ProcessOutput {
            stdout,
            stderr,
            status: status.code(),
            timed_out: false,
          });
        }
        None
          if crate::jobs::current_cancelled()
            || deadline.is_some_and(|value| Instant::now() >= value) =>
        {
          terminate_descendants(child.id());
          let _ = child.kill();
          let _ = child.wait();
          let stdout = stdout_reader.join().unwrap_or_default();
          let stderr = stderr_reader.join().unwrap_or_default();
          return Ok(ProcessOutput {
            stdout,
            stderr,
            status: None,
            timed_out: true,
          });
        }
        None => thread::sleep(Duration::from_millis(10)),
      }
    }
  }

  /// Executes the `run_live` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn run_live(
    &self,
    request: &ProcessRequest,
    live: &LiveProcess,
  ) -> Result<ProcessOutput, ProcessError> {
    let _measurement = ProcessMeasurement::new(request);
    if request.program.trim().is_empty() {
      return Err(ProcessError::EmptyProgram);
    }
    let mut child = command(&request.program)
      .process_group(0)
      .args(&request.args)
      .envs(request.env.iter().map(|(name, value)| (name, value)))
      .stdin(Stdio::null())
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .spawn()
      .map_err(ProcessError::Io)?;
    let stdout = child
      .stdout
      .take()
      .ok_or_else(|| ProcessError::Io(io::Error::other("stdout was not captured")))?;
    let stderr = child
      .stderr
      .take()
      .ok_or_else(|| ProcessError::Io(io::Error::other("stderr was not captured")))?;
    let live_stdout = live.clone();
    let stdout_reader =
      std::thread::spawn(move || read_pipe_live(BufReader::new(stdout), &live_stdout));
    let live_stderr = live.clone();
    let stderr_reader =
      std::thread::spawn(move || read_pipe_live(BufReader::new(stderr), &live_stderr));
    let deadline = request.timeout.map(|timeout| Instant::now() + timeout);
    let output = loop {
      match child.try_wait().map_err(ProcessError::Io)? {
        Some(status) => {
          if request.timeout.is_some() {
            terminate_descendants(child.id());
          }
          let stdout = stdout_reader.join().unwrap_or_default();
          let stderr = stderr_reader.join().unwrap_or_default();
          break Ok(ProcessOutput {
            stdout,
            stderr,
            status: status.code(),
            timed_out: false,
          });
        }
        None
          if crate::jobs::current_cancelled()
            || deadline.is_some_and(|value| Instant::now() >= value) =>
        {
          terminate_descendants(child.id());
          let _ = child.kill();
          let _ = child.wait();
          let stdout = stdout_reader.join().unwrap_or_default();
          let stderr = stderr_reader.join().unwrap_or_default();
          break Ok(ProcessOutput {
            stdout,
            stderr,
            status: None,
            timed_out: true,
          });
        }
        None => thread::sleep(Duration::from_millis(10)),
      }
    };
    live.finish();
    output
  }
}

/// Retrieves data for `read_pipe` without mixing collection with TUI rendering. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn terminate_descendants(pid: u32) {
  if let Ok(pid) = i32::try_from(pid) {
    // Each managed child owns its process group; never target our own group.
    unsafe {
      libc::kill(-pid, libc::SIGKILL);
    }
  }
}

/// Opt-in timings omit arguments, output and environment (which may contain secrets).
struct ProcessMeasurement {
  started: Instant,
  program: String,
  log: Option<std::ffi::OsString>,
}

impl ProcessMeasurement {
  fn new(request: &ProcessRequest) -> Self {
    Self {
      started: Instant::now(),
      program: std::path::Path::new(&request.program)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned(),
      log: std::env::var_os("ARGVUS_PERFORMANCE_LOG"),
    }
  }
}

impl Drop for ProcessMeasurement {
  fn drop(&mut self) {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    if let Some(path) = &self.log
      && let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
    {
      let _ = writeln!(
        file,
        "program={} elapsed_ms={} cancelled={}",
        crate::sanitize::terminal_text(&self.program),
        self.started.elapsed().as_millis(),
        crate::jobs::current_cancelled()
      );
    }
  }
}

fn read_pipe(mut pipe: impl std::io::Read) -> Vec<u8> {
  let mut output = Vec::new();
  let _ = pipe.read_to_end(&mut output);
  output
}

/// Retrieves data for `read_pipe_live` without mixing collection with TUI rendering. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn read_pipe_live(mut pipe: impl BufRead, live: &LiveProcess) -> Vec<u8> {
  let mut output = Vec::new();
  let mut line = String::new();
  loop {
    line.clear();
    match pipe.read_line(&mut line) {
      Ok(0) => break,
      Ok(_) => {
        let trimmed = line.trim_end_matches(['\n', '\r']);
        if !trimmed.is_empty() {
          live.push_line(trimmed);
        }
        output.extend_from_slice(line.as_bytes());
      }
      Err(_) => break,
    }
  }
  output
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn timeout_closes_inherited_pipes() {
    let started = Instant::now();
    let output = SystemProcessRunner
      .run(
        &ProcessRequest::new("sh")
          .arg("-c")
          .arg("sleep 30 & wait")
          .timeout(Duration::from_millis(80)),
      )
      .unwrap();
    assert!(output.timed_out);
    assert!(started.elapsed() < Duration::from_millis(580));
  }

  #[test]
  fn completed_parent_does_not_wait_for_background_pipe() {
    let started = Instant::now();
    let output = SystemProcessRunner
      .run(
        &ProcessRequest::new("sh")
          .arg("-c")
          .arg("sleep 30 & exit 0")
          .timeout(Duration::from_secs(1)),
      )
      .unwrap();
    assert_eq!(output.status, Some(0));
    assert!(started.elapsed() < Duration::from_millis(500));
  }

  #[test]
  fn cancelled_query_terminates_process_tree() {
    let manager = crate::jobs::JobManager::default();
    let job = manager.spawn(|_| {
      SystemProcessRunner
        .run(
          &ProcessRequest::new("sh")
            .arg("-c")
            .arg("sleep 30 & wait")
            .timeout(Duration::from_secs(5)),
        )
        .map_err(|e| e.to_string())
    });
    std::thread::sleep(Duration::from_millis(40));
    job.cancel();
    let start = Instant::now();
    loop {
      if let crate::jobs::JobState::Finished(result) = job.try_state() {
        assert!(result.unwrap().timed_out);
        break;
      }
      assert!(start.elapsed() < Duration::from_millis(500));
      std::thread::sleep(Duration::from_millis(5));
    }
  }

  #[test]
  /// Executes the `builds_arguments_without_a_shell` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn builds_arguments_without_a_shell() {
    let request = ProcessRequest::new("printf")
      .arg("%s")
      .arg("hello;not-a-command");
    let output = SystemProcessRunner.run(&request).unwrap();
    assert_eq!(output.stdout, b"hello;not-a-command");
  }

  #[test]
  fn passes_environment_without_a_shell() {
    let output = SystemProcessRunner
      .run(
        &ProcessRequest::new("sh")
          .arg("-c")
          .arg("printf '%s' \"$ARGVUS_TEST_VALUE\"")
          .env("ARGVUS_TEST_VALUE", "static-mode"),
      )
      .unwrap();
    assert_eq!(output.stdout, b"static-mode");
  }

  #[test]
  /// Executes the `rejects_empty_program` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn rejects_empty_program() {
    assert!(matches!(
      SystemProcessRunner.run(&ProcessRequest::new(" ")),
      Err(ProcessError::EmptyProgram)
    ));
  }

  #[test]
  /// Executes the `drains_large_stdout_without_deadlocking` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn drains_large_stdout_without_deadlocking() {
    let request = ProcessRequest::new("head")
      .arg("-c")
      .arg("500000")
      .arg("/dev/zero")
      .timeout(Duration::from_secs(30));
    let output = SystemProcessRunner.run(&request).unwrap();
    assert_eq!(output.stdout.len(), 500_000);
    assert_eq!(output.status, Some(0));
    assert!(!output.timed_out);
  }

  #[test]
  /// Executes the `run_live_streams_lines_into_the_sink` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn run_live_streams_lines_into_the_sink() {
    let live = LiveProcess::new();
    let request = ProcessRequest::new("printf").arg("%s").arg("alpha\nbeta\n");
    let output = SystemProcessRunner.run_live(&request, &live).unwrap();
    assert_eq!(output.stdout, b"alpha\nbeta\n");
    assert_eq!(output.status, Some(0));
    assert_eq!(live.output(), "alpha\nbeta");
    assert!(live.is_finished());
  }

  #[test]
  /// Executes the `run_live_captures_stderr_but_only_streams_non_empty_lines` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn run_live_captures_stderr_but_only_streams_non_empty_lines() {
    let live = LiveProcess::new();
    let request = ProcessRequest::new("sh")
      .arg("-c")
      .arg("printf 'oops\\n' >&2")
      .timeout(Duration::from_secs(30));
    let output = SystemProcessRunner.run_live(&request, &live).unwrap();
    assert_eq!(output.stderr, b"oops\n");
    assert_eq!(output.stdout, Vec::<u8>::new());
    assert_eq!(live.output(), "oops");
    assert!(live.is_finished());
  }

  #[test]
  /// Executes the `run_live_default_falls_back_to_buffered_run` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn run_live_default_falls_back_to_buffered_run() {
    #[derive(Debug, Default)]
    /// Represents `RecordingRunner`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
    struct RecordingRunner(std::sync::Mutex<Option<ProcessRequest>>);
    impl ProcessRunner for RecordingRunner {
      /// Executes the `run` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
      fn run(&self, request: &ProcessRequest) -> Result<ProcessOutput, ProcessError> {
        *self.0.lock().unwrap() = Some(request.clone());
        Ok(ProcessOutput {
          stdout: b"recorded".to_vec(),
          stderr: Vec::new(),
          status: Some(0),
          timed_out: false,
        })
      }
    }
    let runner = RecordingRunner::default();
    let live = LiveProcess::new();
    let request = ProcessRequest::new("echo").arg("hello");
    let output = runner.run_live(&request, &live).unwrap();
    assert_eq!(output.stdout, b"recorded");
    assert!(live.output().is_empty());
    assert!(!live.is_finished());
    assert_eq!(runner.0.lock().unwrap().clone().unwrap().args, ["hello"]);
  }
}
