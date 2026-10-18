//! The key store: the key pairs under the SSH directory and the running agent.
//! This is the only module that runs `ssh-keygen` and `ssh-add`.

use crate::keys::{
  KeyInfo, KeyRow, KeyType, Loaded, parse_agent_fingerprints, parse_key_info, valid_comment,
  valid_name,
};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Why a key operation was refused. The interface turns each one into its
/// message.
#[derive(Debug, thiserror::Error)]
pub enum KeyError {
  #[error("invalid key name: {0}")]
  InvalidName(String),
  #[error("invalid key type: {0}")]
  InvalidType(String),
  #[error("invalid comment")]
  InvalidComment,
  #[error("key already exists: {0}")]
  Exists(String),
  #[error("key not found: {0}")]
  NotFound(String),
  #[error("public key missing: {0}")]
  NoPublic(String),
  #[error("public key unreadable: {0}")]
  Unreadable(String),
  #[error("key is loaded in the agent: {0}")]
  InUse(String),
  #[error("{0} failed with status {1}")]
  Command(&'static str, i32),
  #[error("{0}")]
  Io(#[from] io::Error),
}

/// The SSH directory: `ARGVUS_SSH_DIR` when set, otherwise `~/.ssh`.
pub fn ssh_dir() -> PathBuf {
  if let Some(dir) = std::env::var_os("ARGVUS_SSH_DIR").filter(|value| !value.is_empty()) {
    return PathBuf::from(dir);
  }
  PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".ssh")
}

/// The agent socket. Hyprland does not carry the session variable into the
/// terminals it opens, so the socket of the ARGVUS service is the fallback.
pub fn agent_socket() -> Option<PathBuf> {
  if let Some(socket) = std::env::var_os("SSH_AUTH_SOCK").filter(|value| !value.is_empty()) {
    return Some(PathBuf::from(socket));
  }
  let runtime = std::env::var_os("XDG_RUNTIME_DIR")
    .filter(|value| !value.is_empty())
    .map(PathBuf::from)
    .unwrap_or_else(|| PathBuf::from(format!("/run/user/{}", unsafe { libc::getuid() })));
  Some(runtime.join("ssh-agent.socket"))
}

/// Runs a command with its output discarded, and reports its status code.
fn run_quiet(program: &str, arguments: &[&str], stdin_null: bool) -> io::Result<Option<i32>> {
  let mut command = Command::new(program);
  command
    .args(arguments)
    .stdout(Stdio::null())
    .stderr(Stdio::null());
  if stdin_null {
    command.stdin(Stdio::null());
  }
  Ok(command.status()?.code())
}

/// The status `ssh-add -l` returns when the agent cannot be reached. A reachable
/// agent with no key returns 1, and that is an empty list, not an error.
pub const AGENT_UNREACHABLE: i32 = 2;

/// The fingerprints of the agent. `Err` carries the status of `ssh-add -l` when
/// it did not list the keys.
pub fn agent_fingerprints() -> Result<Vec<String>, i32> {
  let output = Command::new("ssh-add")
    .arg("-l")
    .stderr(Stdio::null())
    .output()
    .map_err(|_| AGENT_UNREACHABLE)?;
  match output.status.code() {
    Some(0) => Ok(parse_agent_fingerprints(&String::from_utf8_lossy(
      &output.stdout,
    ))),
    Some(code) => Err(code),
    None => Err(AGENT_UNREACHABLE),
  }
}

/// The public key information of a `.pub` file, read with `ssh-keygen -lf`.
fn key_info(public: &Path) -> Option<KeyInfo> {
  let output = Command::new("ssh-keygen")
    .arg("-lf")
    .arg(public)
    .stderr(Stdio::null())
    .output()
    .ok()?;
  if !output.status.success() {
    return None;
  }
  parse_key_info(&String::from_utf8_lossy(&output.stdout))
}

/// True when the private key cannot be read without a passphrase. The empty
/// passphrase is only a probe, never a secret, and stdin is closed so nothing
/// can prompt.
pub fn has_passphrase(private: &Path) -> bool {
  run_quiet(
    "ssh-keygen",
    &["-y", "-P", "", "-f", &private.to_string_lossy()],
    true,
  )
  .map(|code| code != Some(0))
  .unwrap_or(true)
}

/// The key pairs of a directory, in name order, with their agent state.
pub fn list(dir: &Path) -> Vec<KeyRow> {
  let agent = agent_fingerprints();
  let mut names: Vec<String> = fs::read_dir(dir)
    .map(|entries| {
      entries
        .flatten()
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter_map(|file| file.strip_suffix(".pub").map(str::to_string))
        .collect()
    })
    .unwrap_or_default();
  names.sort();
  names
    .into_iter()
    .filter(|name| valid_name(name) && dir.join(name).is_file())
    .filter_map(|name| {
      let info = key_info(&dir.join(format!("{name}.pub")))?;
      let loaded = match &agent {
        Err(AGENT_UNREACHABLE) => Loaded::Unknown,
        Err(_) => Loaded::No,
        Ok(fingerprints) if fingerprints.contains(&info.fingerprint) => Loaded::Yes,
        Ok(_) => Loaded::No,
      };
      Some(KeyRow {
        passphrase: has_passphrase(&dir.join(&name)),
        name,
        key_type: info.key_type,
        fingerprint: info.fingerprint,
        comment: info.comment,
        loaded,
      })
    })
    .collect()
}

/// Creates a key pair. Without `passphrase` the pair is created with an empty
/// passphrase; with it, `ssh-keygen` asks on the terminal it runs in.
pub fn create(
  dir: &Path,
  name: &str,
  key_type: KeyType,
  comment: &str,
  passphrase: bool,
) -> Result<(), KeyError> {
  if !valid_name(name) {
    return Err(KeyError::InvalidName(name.into()));
  }
  if !valid_comment(comment) {
    return Err(KeyError::InvalidComment);
  }
  prepare_dir(dir)?;
  let private = dir.join(name);
  let mut command = Command::new("ssh-keygen");
  command.arg("-q");
  if key_type == KeyType::Rsa {
    command.args(["-t", "rsa", "-b", "4096"]);
  } else {
    command.args(["-t", "ed25519"]);
  }
  command.arg("-f").arg(&private).arg("-C").arg(comment);
  if !passphrase {
    command.args(["-N", ""]);
  }
  let status = command.status()?;
  match status.code() {
    Some(0) => Ok(()),
    code => Err(KeyError::Command("ssh-keygen", code.unwrap_or(1))),
  }
}

/// Creates the SSH directory with the mode `ssh` expects, when it is missing.
pub fn prepare_dir(dir: &Path) -> Result<(), KeyError> {
  if !dir.is_dir() {
    fs::create_dir_all(dir)?;
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(dir, fs::Permissions::from_mode(0o700))?;
  }
  Ok(())
}

/// The private key of a name, which must exist.
pub fn private_key(dir: &Path, name: &str) -> Result<PathBuf, KeyError> {
  if !valid_name(name) {
    return Err(KeyError::InvalidName(name.into()));
  }
  let private = dir.join(name);
  if !private.is_file() {
    return Err(KeyError::NotFound(name.into()));
  }
  Ok(private)
}

/// Loads a key into the agent without a passphrase prompt. A key with a
/// passphrase must be loaded from a terminal, so the caller checks first.
pub fn load_without_prompt(private: &Path) -> Result<i32, KeyError> {
  let status = Command::new("ssh-add")
    .arg(private)
    .stdin(Stdio::null())
    .status()?;
  Ok(status.code().unwrap_or(1))
}

/// Loads a key into the agent, letting `ssh-add` ask for a passphrase on the
/// terminal it runs in.
pub fn load_with_prompt(private: &Path) -> Result<i32, KeyError> {
  let status = Command::new("ssh-add").arg(private).status()?;
  Ok(status.code().unwrap_or(1))
}

/// Removes a key from the agent by its public file.
pub fn unload(dir: &Path, name: &str) -> Result<i32, KeyError> {
  if !valid_name(name) {
    return Err(KeyError::InvalidName(name.into()));
  }
  let public = dir.join(format!("{name}.pub"));
  if !public.is_file() {
    return Err(KeyError::NoPublic(name.into()));
  }
  let status = Command::new("ssh-add").arg("-d").arg(&public).status()?;
  Ok(status.code().unwrap_or(1))
}

/// Removes every key from the agent.
pub fn unload_all() -> Result<i32, KeyError> {
  let status = Command::new("ssh-add").arg("-D").status()?;
  Ok(status.code().unwrap_or(1))
}

/// Deletes a key pair. A key the agent holds is not deleted.
pub fn delete(dir: &Path, name: &str) -> Result<(), KeyError> {
  let private = private_key(dir, name)?;
  let public = dir.join(format!("{name}.pub"));
  if !public.is_file() {
    return Err(KeyError::NoPublic(name.into()));
  }
  let info = key_info(&public).ok_or_else(|| KeyError::Unreadable(name.into()))?;
  if let Ok(fingerprints) = agent_fingerprints()
    && fingerprints.contains(&info.fingerprint)
  {
    return Err(KeyError::InUse(name.into()));
  }
  fs::remove_file(&private)?;
  fs::remove_file(&public)?;
  Ok(())
}
