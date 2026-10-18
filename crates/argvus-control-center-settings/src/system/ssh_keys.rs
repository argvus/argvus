//! SSH key pairs listed and changed through `argvus-keys` (the SSH keys page).
//!
//! The launcher owns every write: it creates, deletes and loads the files under
//! `~/.ssh` and talks to the ssh-agent. This module only calls its CLI and
//! parses the TSV listing. Passphrases never pass through here: loading and
//! passphrase-protected generation open an ARGVUS terminal where the tool
//! prompts for the secret, so it is never a process argument.
use argvus_control_center_core::process::command;

use super::clipboard;
use std::fs;
use std::path::PathBuf;

/// Key algorithm offered by the generate form.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum KeyType {
  #[default]
  Ed25519,
  Rsa,
}

impl KeyType {
  /// The value `argvus-keys generate --type` accepts.
  pub fn as_arg(self) -> &'static str {
    match self {
      Self::Ed25519 => "ed25519",
      Self::Rsa => "rsa",
    }
  }

  /// The other algorithm; the form's type row cycles between the two.
  pub fn toggled(self) -> Self {
    match self {
      Self::Ed25519 => Self::Rsa,
      Self::Rsa => Self::Ed25519,
    }
  }
}

/// Whether a key is held by the ssh-agent. `Unknown` means the agent could not
/// be reached, so no state can be claimed for any key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentState {
  Loaded,
  NotLoaded,
  Unknown,
}

/// One key pair found in `~/.ssh`, as reported by `argvus-keys list`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SshKeyEntry {
  pub name: String,
  pub key_type: String,
  pub fingerprint: String,
  pub comment: String,
  pub has_passphrase: bool,
  pub agent: AgentState,
}

/// The loaded key pairs. `available` is false when `argvus-keys` is not
/// installed, so the page can say so instead of showing an empty list.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SshKeys {
  pub available: bool,
  pub entries: Vec<SshKeyEntry>,
}

/// Loads the key pairs, after checking that the launcher command is installed.
pub fn load() -> SshKeys {
  match command("argvus").args(["keys", "list"]).output() {
    Ok(output) if output.status.success() => SshKeys {
      available: true,
      entries: parse_list(&String::from_utf8_lossy(&output.stdout)),
    },
    _ => SshKeys::default(),
  }
}

/// Parses the TSV printed by `argvus-keys list`:
/// `name, type, fingerprint, comment, passphrase, loaded`.
/// Lines with another field count are skipped, so a launcher that prints
/// extra columns in the future cannot break the page.
pub fn parse_list(listing: &str) -> Vec<SshKeyEntry> {
  listing.lines().filter_map(parse_line).collect()
}

fn parse_line(line: &str) -> Option<SshKeyEntry> {
  let fields: Vec<&str> = line.split('\t').collect();
  let [name, key_type, fingerprint, comment, passphrase, agent] = fields.as_slice() else {
    return None;
  };
  if name.is_empty() {
    return None;
  }
  Some(SshKeyEntry {
    name: (*name).to_owned(),
    key_type: (*key_type).to_owned(),
    fingerprint: (*fingerprint).to_owned(),
    comment: (*comment).to_owned(),
    has_passphrase: *passphrase == "yes",
    agent: parse_agent_state(agent),
  })
}

fn parse_agent_state(value: &str) -> AgentState {
  match value {
    "yes" => AgentState::Loaded,
    "no" => AgentState::NotLoaded,
    _ => AgentState::Unknown,
  }
}

/// Checks a key name before it reaches the CLI. It follows the launcher's rule:
/// the name becomes a file name under `~/.ssh`, so only letters, digits, dot,
/// hyphen and underscore are accepted, without a leading dot or a `.pub` suffix.
pub fn is_valid_name(name: &str) -> bool {
  !name.is_empty()
    && !name.starts_with('.')
    && !name.ends_with(".pub")
    && name
      .chars()
      .all(|character| character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | '_'))
}

/// Generates a pair without a passphrase. Passphrase-protected generation is
/// only available from the CLI, where the terminal asks for the secret.
pub fn generate(name: &str, key_type: KeyType) -> Result<(), String> {
  require_valid_name(name)?;
  run(&["generate", name, "--type", key_type.as_arg()])
}

/// Adds the key to the agent. The launcher opens a terminal, where ssh-add asks
/// for the passphrase, so this returns as soon as the terminal is requested.
pub fn load_key(name: &str) -> Result<(), String> {
  require_valid_name(name)?;
  run(&["load", name])
}

/// Removes one key from the agent. The key file stays on disk.
pub fn unload(name: &str) -> Result<(), String> {
  require_valid_name(name)?;
  run(&["unload", name])
}

/// Removes every key from the agent. The key files stay on disk.
pub fn unload_all() -> Result<(), String> {
  run(&["unload-all"])
}

/// Why a public key could not be copied, so the page can name the cause in the
/// user's language.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CopyError {
  /// The `.pub` file is missing, unreadable or the name is not a valid pair.
  Unreadable,
  /// `wl-copy` is not installed or did not take the text.
  Clipboard,
}

/// Copies the public key of a pair to the Wayland clipboard. The public file is
/// read here rather than through the launcher, which has no read command: it
/// holds no secret, and the copy changes nothing under `~/.ssh`.
pub fn copy_public_key(name: &str) -> Result<(), CopyError> {
  if !is_valid_name(name) {
    return Err(CopyError::Unreadable);
  }
  let path = ssh_dir()
    .ok_or(CopyError::Unreadable)?
    .join(format!("{name}.pub"));
  let public_key = fs::read_to_string(path).map_err(|_| CopyError::Unreadable)?;
  clipboard::copy(public_key.trim_end()).map_err(|_| CopyError::Clipboard)
}

/// The directory the launcher reads: `ARGVUS_SSH_DIR` when set, else `~/.ssh`.
fn ssh_dir() -> Option<PathBuf> {
  std::env::var_os("ARGVUS_SSH_DIR")
    .filter(|dir| !dir.is_empty())
    .map(PathBuf::from)
    .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".ssh")))
}

/// Deletes the pair from `~/.ssh`. The launcher refuses while the key is
/// loaded in the agent, so the agent never keeps a key whose file is gone.
pub fn delete(name: &str) -> Result<(), String> {
  require_valid_name(name)?;
  run(&["delete", name])
}

fn require_valid_name(name: &str) -> Result<(), String> {
  if is_valid_name(name) {
    Ok(())
  } else {
    Err(format!("invalid key name: {name}"))
  }
}

fn run(args: &[&str]) -> Result<(), String> {
  let output = command("argvus")
    .arg("keys")
    .args(args)
    .output()
    .map_err(|error| format!("failed to run argvus-keys: {error}"))?;
  if output.status.success() {
    return Ok(());
  }
  let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
  Err(if stderr.is_empty() {
    "argvus-keys failed".into()
  } else {
    stderr
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parses_each_listed_key_pair() {
    let listing = "alpha\ted25519\tSHA256:abc\talpha@host\tno\tyes\n\
                   beta\trsa\tSHA256:def\t\tyes\tno\n";
    let entries = parse_list(listing);
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].name, "alpha");
    assert_eq!(entries[0].key_type, "ed25519");
    assert_eq!(entries[0].fingerprint, "SHA256:abc");
    assert_eq!(entries[0].comment, "alpha@host");
    assert!(!entries[0].has_passphrase);
    assert_eq!(entries[0].agent, AgentState::Loaded);
    assert!(entries[1].has_passphrase);
    assert_eq!(entries[1].agent, AgentState::NotLoaded);
    assert_eq!(entries[1].comment, "");
  }

  #[test]
  fn unknown_agent_column_is_not_claimed_as_loaded() {
    let entries = parse_list("alpha\ted25519\tSHA256:abc\talpha\tno\tunknown\n");
    assert_eq!(entries[0].agent, AgentState::Unknown);
  }

  #[test]
  fn skips_lines_with_another_column_count() {
    let entries = parse_list("alpha\ted25519\tSHA256:abc\n\n");
    assert!(entries.is_empty());
  }

  #[test]
  fn skips_lines_without_a_name() {
    let entries = parse_list("\ted25519\tSHA256:abc\tcomment\tno\tno\n");
    assert!(entries.is_empty());
  }

  #[test]
  fn accepts_names_the_launcher_accepts() {
    assert!(is_valid_name("work-2026.key_1"));
  }

  #[test]
  fn rejects_names_that_would_escape_or_shadow_a_file() {
    assert!(!is_valid_name(""));
    assert!(!is_valid_name("../escape"));
    assert!(!is_valid_name(".hidden"));
    assert!(!is_valid_name("work.pub"));
    assert!(!is_valid_name("with space"));
    assert!(!is_valid_name("sub/dir"));
  }

  #[test]
  fn refuses_a_traversal_name_before_running_the_launcher() {
    // Validation happens before any process starts, so this stays a pure check.
    assert!(generate("../escape", KeyType::Ed25519).is_err());
    assert!(delete("work.pub").is_err());
    assert_eq!(copy_public_key("../escape"), Err(CopyError::Unreadable));
  }

  #[test]
  fn key_type_cycles_between_the_two_supported_algorithms() {
    assert_eq!(KeyType::default(), KeyType::Ed25519);
    assert_eq!(KeyType::Ed25519.toggled(), KeyType::Rsa);
    assert_eq!(KeyType::Rsa.toggled(), KeyType::Ed25519);
    assert_eq!(KeyType::Rsa.as_arg(), "rsa");
  }
}
