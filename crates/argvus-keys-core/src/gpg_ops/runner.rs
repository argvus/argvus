//! Runs gpg with the ARGVUS options: the result types, the GnuPG home, the binary lookup and the pinentry check.

use super::private_files::{restrict_dir, write_private_atomic};
use super::*;

/// A refused or failed command: the message key and its arguments.
#[derive(Debug, PartialEq, Eq)]
pub struct Failure {
  pub key: &'static str,
  pub args: Vec<String>,
}

impl Failure {
  pub(super) fn new(key: &'static str, args: &[(&str, &str)]) -> Self {
    Self {
      key,
      args: args
        .iter()
        .map(|(name, value)| format!("{name}={value}"))
        .collect(),
    }
  }
}

/// What a successful command shows.
#[derive(Debug, PartialEq, Eq)]
pub enum Output {
  /// Text written to standard output as it is.
  Text(String),
  /// A message key and its arguments.
  Message(&'static str, Vec<String>),
  /// Nothing to show.
  Nothing,
}

pub type Outcome = Result<Output, Failure>;

/// How the passphrase of a secret-key step is supplied.
#[derive(Clone, PartialEq, Eq)]
pub(super) enum Secret {
  /// gpg asks on its pinentry.
  Ask,
  /// The empty passphrase, in loopback mode: no pinentry is asked.
  Empty,
  /// The passphrase read from stdin, given to gpg on its file descriptor 0.
  Stdin(String),
}

/// The state of one command: the gnupg home and the passphrase mode.
pub(super) struct Gpg {
  pub(super) home: PathBuf,
  pub(super) secret: Secret,
  /// gpg's own messages are shown when the script does not redirect them.
  pub(super) show_stderr: bool,
}

pub(super) fn gnupg_home() -> PathBuf {
  if let Some(home) = env::var_os("GNUPGHOME").filter(|value| !value.is_empty()) {
    return PathBuf::from(home);
  }
  PathBuf::from(env::var_os("HOME").unwrap_or_default()).join(".gnupg")
}

/// The result of one gpg run: whether it succeeded and what it printed.
pub(super) struct Run {
  pub(super) success: bool,
  pub(super) stdout: String,
}

impl Gpg {
  pub(super) fn new(secret: Secret) -> Self {
    Self {
      home: gnupg_home(),
      secret,
      show_stderr: false,
    }
  }

  /// Runs a step with the passphrase mode of this command. `stdin` is the
  /// text written to a command that reads its input from gpg.
  pub(super) fn run(&self, arguments: &[&str], stdin: Option<&str>) -> Run {
    let mut command = Command::new("gpg");
    let mut pipe_stdin = stdin.map(str::to_string);
    match &self.secret {
      Secret::Ask => {}
      Secret::Empty => {
        command.args(["--batch", "--pinentry-mode", "loopback", "--passphrase", ""]);
      }
      Secret::Stdin(passphrase) => {
        command.args([
          "--batch",
          "--pinentry-mode",
          "loopback",
          "--passphrase-fd",
          "0",
        ]);
        pipe_stdin = Some(format!("{passphrase}\n"));
      }
    }
    let stderr = if self.show_stderr {
      Stdio::inherit()
    } else {
      Stdio::null()
    };
    command
      .args(arguments)
      .stdout(Stdio::piped())
      .stderr(stderr);
    if pipe_stdin.is_some() {
      command.stdin(Stdio::piped());
    }
    let Ok(mut child) = command.spawn() else {
      return Run {
        success: false,
        stdout: String::new(),
      };
    };
    if let (Some(text), Some(mut input)) = (pipe_stdin, child.stdin.take()) {
      let _ = input.write_all(text.as_bytes());
    }
    let mut stdout = String::new();
    if let Some(mut output) = child.stdout.take() {
      let _ = output.read_to_string(&mut stdout);
    }
    let success = child.wait().map(|status| status.success()).unwrap_or(false);
    Run { success, stdout }
  }

  /// Runs a step whose output the script discards, and reports its status.
  /// The output is not a pipe here, as the script's `>/dev/null` made it.
  pub(super) fn quiet(&self, arguments: &[&str], stdin: Option<&str>) -> bool {
    let mut command = Command::new("gpg");
    let mut pipe_stdin = stdin.map(str::to_string);
    match &self.secret {
      Secret::Ask => {}
      Secret::Empty => {
        command.args(["--batch", "--pinentry-mode", "loopback", "--passphrase", ""]);
      }
      Secret::Stdin(passphrase) => {
        command.args([
          "--batch",
          "--pinentry-mode",
          "loopback",
          "--passphrase-fd",
          "0",
        ]);
        pipe_stdin = Some(format!("{passphrase}\n"));
      }
    }
    let stderr = if self.show_stderr {
      Stdio::inherit()
    } else {
      Stdio::null()
    };
    command.args(arguments).stdout(Stdio::null()).stderr(stderr);
    if pipe_stdin.is_some() {
      command.stdin(Stdio::piped());
    }
    let Ok(mut child) = command.spawn() else {
      return false;
    };
    if let (Some(text), Some(mut input)) = (pipe_stdin, child.stdin.take()) {
      let _ = input.write_all(text.as_bytes());
    }
    child.wait().map(|status| status.success()).unwrap_or(false)
  }

  /// Runs gpg with no passphrase handling and returns its listing, or `None`.
  pub(super) fn listing(&self, arguments: &[&str]) -> Option<String> {
    let run = self.run(arguments, None);
    run.success.then_some(run.stdout)
  }
}

/// Fails when `gpg` is not installed.
pub(super) fn require_gpg() -> Result<(), Failure> {
  let found = env::var_os("PATH")
    .map(|paths| env::split_paths(&paths).any(|dir| dir.join("gpg").is_file()))
    .unwrap_or(false);
  if found {
    Ok(())
  } else {
    Err(Failure::new("error.gpg_missing", &[]))
  }
}

/// The path of a program on `PATH`.
pub(super) fn find_program(program: &str) -> Option<PathBuf> {
  env::var_os("PATH").and_then(|paths| {
    env::split_paths(&paths)
      .map(|dir| dir.join(program))
      .find(|candidate| candidate.is_file())
  })
}

/// Makes gpg-agent use the ARGVUS pinentry. The line lives in the ARGVUS block;
/// the user's own lines are kept, and a pinentry the user wrote outside the
/// block is refused. The agent is restarted only when the block changes.
pub(super) fn ensure_pinentry(gpg: &Gpg) -> Result<(), Failure> {
  let program = find_program(PINENTRY_BIN)
    .ok_or_else(|| Failure::new("error.gpg_pinentry_missing", &[("bin", PINENTRY_BIN)]))?;
  let program = program.to_string_lossy().into_owned();
  let conf_path = gpg::agent_conf(&gpg.home);
  let existing = fs::read_to_string(&conf_path).ok();
  if let Some(conf) = &existing {
    if gpg::user_pinentry(conf).is_some() {
      return Err(Failure::new("error.gpg_pinentry_conflict", &[]));
    }
    if gpg::managed_pinentry(conf).as_deref() == Some(program.as_str()) {
      return Ok(());
    }
  }
  if !gpg.home.is_dir() {
    fs::create_dir_all(&gpg.home).map_err(io_failure)?;
    restrict_dir(&gpg.home);
  }
  let text = gpg::with_managed_pinentry(existing.as_deref().unwrap_or(""), &program);
  if existing.is_some() {
    fs::copy(&conf_path, conf_path.with_extension("conf.bak")).map_err(io_failure)?;
  }
  write_private_atomic(&conf_path, text.as_bytes())?;
  // gpg-agent reads its configuration once, so it restarts with the new pinentry.
  let _ = Command::new("gpgconf")
    .args(["--kill", "gpg-agent"])
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .status();
  Ok(())
}

/// A file operation of the keyring failed. The shell reported such failures as
/// a generic action failure.
pub(super) fn io_failure(_error: std::io::Error) -> Failure {
  Failure::new("error.gpg_action_failed", &[])
}
