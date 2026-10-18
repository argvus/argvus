//! The GPG commands. Each one checks its arguments with the rules of `gpg`, runs
//! gpg, and returns what the user sees: text, a message key, or a failure key.
//!
//! Passphrases never reach an argument list. A passphrase given with
//! `--passphrase-stdin` is written to gpg's stdin, and each step gets its own
//! copy. Without it, gpg asks on its pinentry.

use crate::gpg::{
  self, GPG_API, PINENTRY_BIN, created_fingerprint, parse_fingerprints, parse_key_listing,
  parse_subkey_fingerprints,
};
use std::env;
use std::fs;
use std::io::{ErrorKind, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// A refused or failed command: the message key and its arguments.
#[derive(Debug, PartialEq, Eq)]
pub struct Failure {
  pub key: &'static str,
  pub args: Vec<String>,
}

impl Failure {
  fn new(key: &'static str, args: &[(&str, &str)]) -> Self {
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
enum Secret {
  /// gpg asks on its pinentry.
  Ask,
  /// The empty passphrase, in loopback mode: no pinentry is asked.
  Empty,
  /// The passphrase read from stdin, given to gpg on its file descriptor 0.
  Stdin(String),
}

/// The state of one command: the gnupg home and the passphrase mode.
struct Gpg {
  home: PathBuf,
  secret: Secret,
  /// gpg's own messages are shown when the script does not redirect them.
  show_stderr: bool,
}

fn gnupg_home() -> PathBuf {
  if let Some(home) = env::var_os("GNUPGHOME").filter(|value| !value.is_empty()) {
    return PathBuf::from(home);
  }
  PathBuf::from(env::var_os("HOME").unwrap_or_default()).join(".gnupg")
}

/// The result of one gpg run: whether it succeeded and what it printed.
struct Run {
  success: bool,
  stdout: String,
}

impl Gpg {
  fn new(secret: Secret) -> Self {
    Self {
      home: gnupg_home(),
      secret,
      show_stderr: false,
    }
  }

  /// Runs a step with the passphrase mode of this command. `stdin` is the
  /// text written to a command that reads its input from gpg.
  fn run(&self, arguments: &[&str], stdin: Option<&str>) -> Run {
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
  fn quiet(&self, arguments: &[&str], stdin: Option<&str>) -> bool {
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
  fn listing(&self, arguments: &[&str]) -> Option<String> {
    let run = self.run(arguments, None);
    run.success.then_some(run.stdout)
  }
}

/// Fails when `gpg` is not installed.
fn require_gpg() -> Result<(), Failure> {
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
fn find_program(program: &str) -> Option<PathBuf> {
  env::var_os("PATH").and_then(|paths| {
    env::split_paths(&paths)
      .map(|dir| dir.join(program))
      .find(|candidate| candidate.is_file())
  })
}

/// Makes gpg-agent use the ARGVUS pinentry. The line lives in the ARGVUS block;
/// the user's own lines are kept, and a pinentry the user wrote outside the
/// block is refused. The agent is restarted only when the block changes.
fn ensure_pinentry(gpg: &Gpg) -> Result<(), Failure> {
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
fn io_failure(_error: std::io::Error) -> Failure {
  Failure::new("error.gpg_action_failed", &[])
}

#[cfg(unix)]
fn restrict_dir(dir: &Path) {
  use std::os::unix::fs::PermissionsExt;
  let _ = fs::set_permissions(dir, fs::Permissions::from_mode(0o700));
}

/// Writes a file through a private temporary file in the same directory, renamed
/// into place when it is complete.
fn write_private_atomic(target: &Path, data: &[u8]) -> Result<(), Failure> {
  let dir = target.parent().unwrap_or(Path::new("."));
  let temp = private_temp(dir, ".gpg-agent.conf")?;
  fs::write(&temp, data).map_err(io_failure)?;
  fs::rename(&temp, target).map_err(io_failure)
}

/// A new private file in `dir` with the given prefix, created exclusively.
fn private_temp(dir: &Path, prefix: &str) -> Result<PathBuf, Failure> {
  use std::os::unix::fs::OpenOptionsExt;
  for attempt in 0..100u32 {
    let stamp = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .map(|duration| duration.as_nanos())
      .unwrap_or(0);
    let candidate = dir.join(format!("{prefix}.{}{stamp}{attempt}", std::process::id()));
    match fs::OpenOptions::new()
      .write(true)
      .create_new(true)
      .mode(0o600)
      .open(&candidate)
    {
      Ok(_) => return Ok(candidate),
      Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
      Err(error) => return Err(io_failure(error)),
    }
  }
  Err(Failure::new("error.gpg_export_failed", &[]))
}

/// The absolute form of a path the user typed.
fn absolute(path: &str) -> String {
  if path.starts_with('/') {
    return path.to_string();
  }
  let current = env::current_dir().unwrap_or_default();
  current.join(path).to_string_lossy().into_owned()
}

fn require_yes(confirmed: bool) -> Result<(), Failure> {
  if confirmed {
    Ok(())
  } else {
    Err(Failure::new("error.gpg_confirm_action", &[]))
  }
}

/// Lists the keys: fingerprint, key id, type, expiry, secret, validity, user id.
pub fn list() -> Outcome {
  let gpg = Gpg::new(Secret::Ask);
  require_gpg()?;
  let secret = gpg
    .listing(&["--with-colons", "--fixed-list-mode", "--list-secret-keys"])
    .map(|text| parse_fingerprints(&text))
    .unwrap_or_default();
  let keys = gpg
    .listing(&["--with-colons", "--fixed-list-mode", "--list-keys"])
    .unwrap_or_default();
  let mut text = String::new();
  for row in parse_key_listing(&keys, &secret) {
    text.push_str(&row.tsv());
    text.push('\n');
  }
  Ok(Output::Text(text))
}

/// The state of the pinentry and of gpg, as tab separated lines.
pub fn status() -> Outcome {
  let mut text = format!("api\t{GPG_API}\n");
  text.push_str(if find_program("gpg").is_some() {
    "gnupg\tyes\n"
  } else {
    "gnupg\tno\n"
  });
  let conf = fs::read_to_string(gpg::agent_conf(&gnupg_home())).ok();
  match conf {
    None => text.push_str("pinentry\tnone\tdefault\n"),
    Some(conf) => {
      if let Some(managed) = gpg::managed_pinentry(&conf) {
        text.push_str(&format!("pinentry\t{managed}\targvus\n"));
      } else if let Some(user) = gpg::user_pinentry(&conf) {
        text.push_str(&format!("pinentry\t{user}\tuser\n"));
      } else {
        text.push_str("pinentry\tnone\tdefault\n");
      }
    }
  }
  Ok(Output::Text(text))
}

/// Options of `generate`.
pub struct GenerateOptions {
  pub uid: Option<String>,
  pub algo: String,
  pub bits: Option<String>,
  pub expire: String,
  pub no_passphrase: bool,
  pub passphrase_stdin: bool,
  pub confirmed: bool,
}

/// Creates a primary key with certify and sign usage, and adds an encryption
/// subkey: a primary key with those usages cannot encrypt by itself.
pub fn generate(options: GenerateOptions, stdin_line: impl FnOnce() -> String) -> Outcome {
  let uid = options.uid.as_deref().unwrap_or("");
  if !gpg::valid_uid(uid) {
    return Err(Failure::new("error.gpg_invalid_uid", &[]));
  }
  if !gpg::valid_algo(&options.algo) {
    return Err(Failure::new(
      "error.gpg_bad_algo",
      &[("algo", &options.algo)],
    ));
  }
  let bits = match options.bits {
    Some(bits) => bits,
    None => gpg::default_bits(&options.algo).unwrap_or("").to_string(),
  };
  if !gpg::valid_key_bits(&options.algo, &bits) {
    return Err(Failure::new(
      "error.gpg_bad_bits",
      &[("algo", &options.algo), ("bits", &bits)],
    ));
  }
  if !gpg::valid_expire(&options.expire) {
    return Err(Failure::new(
      "error.gpg_bad_expire",
      &[("expire", &options.expire)],
    ));
  }
  if options.no_passphrase && !options.confirmed {
    return Err(Failure::new("error.gpg_confirm_no_passphrase", &[]));
  }
  require_gpg()?;
  let secret = if options.no_passphrase {
    Secret::Empty
  } else if options.passphrase_stdin {
    // The first line is the passphrase. A missing or empty line is refused,
    // because it would make an unprotected key by accident.
    let passphrase = stdin_line();
    if passphrase.is_empty() {
      return Err(Failure::new("error.gpg_passphrase_empty", &[]));
    }
    Secret::Stdin(passphrase)
  } else {
    Secret::Ask
  };
  let mut gpg = Gpg::new(secret);
  gpg.show_stderr = true;
  ensure_pinentry(&gpg)?;

  let key_spec = format!("{}{bits}", options.algo);
  let status = gpg.run(
    &[
      "--status-fd",
      "1",
      "--quick-gen-key",
      uid,
      &key_spec,
      "default",
      &options.expire,
    ],
    None,
  );
  if !status.success {
    return Err(Failure::new("error.gpg_generate_failed", &[]));
  }
  let fingerprint = created_fingerprint(&status.stdout)
    .ok_or_else(|| Failure::new("error.gpg_generate_failed", &[]))?;
  let subkey = gpg::encryption_subkey_algo(&options.algo, &bits).unwrap_or_default();
  if !gpg.quiet(
    &[
      "--quick-add-key",
      &fingerprint,
      &subkey,
      "encr",
      &options.expire,
    ],
    None,
  ) {
    return Err(Failure::new(
      "error.gpg_subkey_failed",
      &[("fingerprint", &fingerprint)],
    ));
  }
  Ok(Output::Message(
    "message.gpg_generated",
    vec![format!("fingerprint={fingerprint}")],
  ))
}

/// Writes the public key to a file (or to standard output with `-`).
pub fn export_public(id: &str, file: &str, force: bool) -> Outcome {
  if !gpg::valid_id(id) {
    return Err(Failure::new("error.gpg_invalid_id", &[("id", id)]));
  }
  require_gpg()?;
  let gpg = Gpg::new(Secret::Ask);
  if file == "-" {
    let run = gpg.run(&["--batch", "--armor", "--export", id], None);
    let key = run.stdout.trim_end_matches('\n');
    if key.is_empty() {
      return Err(Failure::new("error.gpg_no_key", &[("id", id)]));
    }
    return Ok(Output::Text(format!("{key}\n")));
  }
  let file = absolute(file);
  let dir = Path::new(&file)
    .parent()
    .unwrap_or(Path::new("/"))
    .to_path_buf();
  if !dir.is_dir() {
    return Err(Failure::new("error.gpg_no_dir", &[("file", &file)]));
  }
  if Path::new(&file).exists() && !force {
    return Err(Failure::new("error.gpg_file_exists", &[("file", &file)]));
  }
  let temp = private_temp(&dir, ".argvus-gpg-export")
    .map_err(|_| Failure::new("error.gpg_export_failed", &[("file", &file)]))?;
  let run = gpg.run(&["--batch", "--armor", "--export", id], None);
  let stored = if run.success {
    fs::write(&temp, run.stdout.as_bytes()).is_ok()
  } else {
    false
  };
  if !stored {
    let _ = fs::remove_file(&temp);
    return Err(Failure::new("error.gpg_export_failed", &[("file", &file)]));
  }
  if fs::metadata(&temp)
    .map(|meta| meta.len() == 0)
    .unwrap_or(true)
  {
    let _ = fs::remove_file(&temp);
    return Err(Failure::new("error.gpg_no_key", &[("id", id)]));
  }
  fs::rename(&temp, &file)
    .map_err(|_| Failure::new("error.gpg_export_failed", &[("file", &file)]))?;
  Ok(Output::Message(
    "message.gpg_exported",
    vec![format!("file={file}")],
  ))
}

/// Imports public keys from a file. A file with a private key is refused.
pub fn import(file: &str) -> Outcome {
  if !readable(file) {
    return Err(Failure::new("error.gpg_not_readable", &[("file", file)]));
  }
  require_gpg()?;
  let file = absolute(file);
  let gpg = Gpg::new(Secret::Ask);
  let packets = gpg.run(&["--batch", "--list-packets", &file], None);
  if packets.stdout.lines().any(is_secret_packet) {
    return Err(Failure::new(
      "error.gpg_private_refused",
      &[("file", &file)],
    ));
  }
  if !gpg.quiet(&["--batch", "--import", &file], None) {
    return Err(Failure::new("error.gpg_import_failed", &[("file", &file)]));
  }
  Ok(Output::Message(
    "message.gpg_imported",
    vec![format!("file={file}")],
  ))
}

/// Imports a file with a private key. The key keeps its passphrase.
pub fn import_private(file: &str) -> Outcome {
  if !readable(file) {
    return Err(Failure::new("error.gpg_not_readable", &[("file", file)]));
  }
  require_gpg()?;
  let file = absolute(file);
  let gpg = Gpg::new(Secret::Ask);
  let packets = gpg.run(&["--batch", "--list-packets", &file], None);
  if !packets.stdout.lines().any(is_secret_packet) {
    return Err(Failure::new("error.gpg_not_private", &[("file", &file)]));
  }
  if !gpg.quiet(&["--no-tty", "--batch", "--import", &file], None) {
    return Err(Failure::new("error.gpg_import_failed", &[("file", &file)]));
  }
  Ok(Output::Message(
    "message.gpg_private_imported",
    vec![format!("file={file}")],
  ))
}

fn readable(file: &str) -> bool {
  Path::new(file).is_file() && fs::File::open(file).is_ok()
}

/// A listing line that starts a secret key packet.
fn is_secret_packet(line: &str) -> bool {
  line.contains(":secret key packet:") || line.contains(":secret sub key packet:")
}

/// Exports the private key of one fingerprint, armored, with mode 600.
pub fn export_private(fingerprint: &str, file: &str, force: bool, confirmed: bool) -> Outcome {
  if !gpg::valid_fingerprint(fingerprint) {
    return Err(Failure::new(
      "error.gpg_invalid_fingerprint",
      &[("fingerprint", fingerprint)],
    ));
  }
  require_yes(confirmed)?;
  require_gpg()?;
  let file = absolute(file);
  let dir = Path::new(&file)
    .parent()
    .unwrap_or(Path::new("/"))
    .to_path_buf();
  if !dir.is_dir() {
    return Err(Failure::new("error.gpg_no_dir", &[("file", &file)]));
  }
  if Path::new(&file).exists() && !force {
    return Err(Failure::new("error.gpg_file_exists", &[("file", &file)]));
  }
  let gpg = Gpg::new(Secret::Ask);
  let temp = private_temp(&dir, ".argvus-gpg-export")
    .map_err(|_| Failure::new("error.gpg_export_failed", &[("file", &file)]))?;
  let run = gpg.run(
    &[
      "--no-tty",
      "--batch",
      "--armor",
      "--export-secret-keys",
      fingerprint,
    ],
    None,
  );
  if !run.success || fs::write(&temp, run.stdout.as_bytes()).is_err() {
    let _ = fs::remove_file(&temp);
    return Err(Failure::new("error.gpg_export_failed", &[("file", &file)]));
  }
  if fs::metadata(&temp)
    .map(|meta| meta.len() == 0)
    .unwrap_or(true)
  {
    let _ = fs::remove_file(&temp);
    return Err(Failure::new(
      "error.gpg_no_secret_key",
      &[("fingerprint", fingerprint)],
    ));
  }
  fs::rename(&temp, &file)
    .map_err(|_| Failure::new("error.gpg_export_failed", &[("file", &file)]))?;
  Ok(Output::Message(
    "message.gpg_private_exported",
    vec![format!("file={file}")],
  ))
}

/// Deletes a key by its full fingerprint, with its secret part when it has one.
pub fn delete(fingerprint: &str, confirmed: bool) -> Outcome {
  if !gpg::valid_fingerprint(fingerprint) {
    return Err(Failure::new(
      "error.gpg_invalid_fingerprint",
      &[("fingerprint", fingerprint)],
    ));
  }
  require_yes(confirmed)?;
  require_gpg()?;
  let gpg = Gpg::new(Secret::Ask);
  let has_secret = gpg.quiet(&["--batch", "--list-secret-keys", fingerprint], None);
  let action_failed = || Failure::new("error.gpg_action_failed", &[("fingerprint", fingerprint)]);
  let removed = if has_secret {
    gpg.quiet(
      &[
        "--no-tty",
        "--batch",
        "--yes",
        "--delete-secret-and-public-key",
        fingerprint,
      ],
      None,
    )
  } else {
    gpg.quiet(
      &["--no-tty", "--batch", "--yes", "--delete-keys", fingerprint],
      None,
    )
  };
  if !removed {
    return Err(action_failed());
  }
  Ok(Output::Message(
    "message.gpg_deleted",
    vec![format!("fingerprint={fingerprint}")],
  ))
}

/// Changes the expiry of the primary key and of each subkey.
pub fn edit_expire(fingerprint: &str, expire: &str) -> Outcome {
  if !gpg::valid_fingerprint(fingerprint) {
    return Err(Failure::new(
      "error.gpg_invalid_fingerprint",
      &[("fingerprint", fingerprint)],
    ));
  }
  if !gpg::valid_expire(expire) {
    return Err(Failure::new("error.gpg_bad_expire", &[("expire", expire)]));
  }
  require_gpg()?;
  let gpg = Gpg::new(Secret::Ask);
  let action_failed = || Failure::new("error.gpg_action_failed", &[("fingerprint", fingerprint)]);
  if !gpg.quiet(
    &[
      "--no-tty",
      "--batch",
      "--yes",
      "--quick-set-expire",
      fingerprint,
      expire,
    ],
    None,
  ) {
    return Err(action_failed());
  }
  // gpg sets a subkey's expiry only when its fingerprint is named.
  let listing = gpg
    .listing(&[
      "--batch",
      "--with-colons",
      "--fixed-list-mode",
      "--list-keys",
      fingerprint,
    ])
    .unwrap_or_default();
  for subkey in parse_subkey_fingerprints(&listing) {
    if !gpg.quiet(
      &[
        "--no-tty",
        "--batch",
        "--yes",
        "--quick-set-expire",
        fingerprint,
        expire,
        &subkey,
      ],
      None,
    ) {
      return Err(action_failed());
    }
  }
  Ok(Output::Message(
    "message.gpg_expire_set",
    vec![format!("fingerprint={fingerprint}")],
  ))
}

/// Changes the passphrase of a key. gpg asks the old and the new one itself.
pub fn edit_passwd(fingerprint: &str) -> Outcome {
  if !gpg::valid_fingerprint(fingerprint) {
    return Err(Failure::new(
      "error.gpg_invalid_fingerprint",
      &[("fingerprint", fingerprint)],
    ));
  }
  require_gpg()?;
  let gpg = Gpg::new(Secret::Ask);
  ensure_pinentry(&gpg)?;
  if !gpg.quiet(&["--no-tty", "--change-passphrase", fingerprint], None) {
    return Err(Failure::new(
      "error.gpg_action_failed",
      &[("fingerprint", fingerprint)],
    ));
  }
  Ok(Output::Message(
    "message.gpg_passphrase_changed",
    vec![format!("fingerprint={fingerprint}")],
  ))
}

/// Certifies a key with a secret key of this keyring.
pub fn edit_sign(fingerprint: &str, signer: &str, confirmed: bool) -> Outcome {
  if !gpg::valid_fingerprint(fingerprint) {
    return Err(Failure::new(
      "error.gpg_invalid_fingerprint",
      &[("fingerprint", fingerprint)],
    ));
  }
  if !gpg::valid_fingerprint(signer) {
    return Err(Failure::new(
      "error.gpg_invalid_fingerprint",
      &[("fingerprint", signer)],
    ));
  }
  require_yes(confirmed)?;
  require_gpg()?;
  let gpg = Gpg::new(Secret::Ask);
  if !gpg.quiet(&["--batch", "--list-secret-keys", signer], None) {
    return Err(Failure::new(
      "error.gpg_signer_not_secret",
      &[("fingerprint", signer)],
    ));
  }
  ensure_pinentry(&gpg)?;
  if !gpg.quiet(
    &[
      "--no-tty",
      "--batch",
      "--yes",
      "--local-user",
      signer,
      "--quick-sign-key",
      fingerprint,
    ],
    None,
  ) {
    return Err(Failure::new(
      "error.gpg_action_failed",
      &[("fingerprint", fingerprint)],
    ));
  }
  Ok(Output::Message(
    "message.gpg_signed",
    vec![format!("fingerprint={fingerprint}")],
  ))
}

/// Revokes a key with a reason. The answers go to gpg's `--command-fd`.
pub fn edit_revoke(fingerprint: &str, reason: &str, confirmed: bool) -> Outcome {
  if !gpg::valid_fingerprint(fingerprint) {
    return Err(Failure::new(
      "error.gpg_invalid_fingerprint",
      &[("fingerprint", fingerprint)],
    ));
  }
  let code = gpg::revocation_code(reason)
    .ok_or_else(|| Failure::new("error.gpg_bad_reason", &[("reason", reason)]))?;
  require_yes(confirmed)?;
  require_gpg()?;
  let gpg = Gpg::new(Secret::Ask);
  ensure_pinentry(&gpg)?;
  let answers = format!("revkey\ny\n{code}\n\ny\nsave\n");
  if !gpg.quiet(
    &[
      "--no-tty",
      "--yes",
      "--pinentry-mode",
      "ask",
      "--command-fd",
      "0",
      "--edit-key",
      fingerprint,
    ],
    Some(&answers),
  ) {
    return Err(Failure::new(
      "error.gpg_action_failed",
      &[("fingerprint", fingerprint)],
    ));
  }
  Ok(Output::Message(
    "message.gpg_revoked",
    vec![format!("fingerprint={fingerprint}")],
  ))
}

/// Sets how far a key is trusted. Ultimate asks gpg for one more confirmation.
pub fn edit_trust(fingerprint: &str, level: &str) -> Outcome {
  if !gpg::valid_fingerprint(fingerprint) {
    return Err(Failure::new(
      "error.gpg_invalid_fingerprint",
      &[("fingerprint", fingerprint)],
    ));
  }
  let value = gpg::trust_value(level)
    .ok_or_else(|| Failure::new("error.gpg_bad_trust", &[("level", level)]))?;
  require_gpg()?;
  let gpg = Gpg::new(Secret::Ask);
  let answers = if value == "5" {
    format!("trust\n{value}\ny\nquit\n")
  } else {
    format!("trust\n{value}\nquit\n")
  };
  if !gpg.quiet(
    &[
      "--no-tty",
      "--yes",
      "--command-fd",
      "0",
      "--edit-key",
      fingerprint,
    ],
    Some(&answers),
  ) {
    return Err(Failure::new(
      "error.gpg_action_failed",
      &[("fingerprint", fingerprint)],
    ));
  }
  Ok(Output::Message(
    "message.gpg_trust_set",
    vec![
      format!("fingerprint={fingerprint}"),
      format!("level={level}"),
    ],
  ))
}

/// Sends a key to the keyserver (publishing it, so it needs confirmation) or
/// fetches one from it.
pub fn keyserver(action: &str, fingerprint: &str, confirmed: bool) -> Outcome {
  if !gpg::valid_fingerprint(fingerprint) {
    return Err(Failure::new(
      "error.gpg_invalid_fingerprint",
      &[("fingerprint", fingerprint)],
    ));
  }
  require_gpg()?;
  let gpg = Gpg::new(Secret::Ask);
  let failed = || {
    Failure::new(
      "error.gpg_keyserver_failed",
      &[("fingerprint", fingerprint)],
    )
  };
  match action {
    "send" => {
      require_yes(confirmed)?;
      if !gpg.quiet(&["--no-tty", "--batch", "--send-keys", fingerprint], None) {
        return Err(failed());
      }
      Ok(Output::Message(
        "message.gpg_sent",
        vec![format!("fingerprint={fingerprint}")],
      ))
    }
    "recv" => {
      if !gpg.quiet(&["--no-tty", "--batch", "--recv-keys", fingerprint], None) {
        return Err(failed());
      }
      Ok(Output::Message(
        "message.gpg_received",
        vec![format!("fingerprint={fingerprint}")],
      ))
    }
    _ => Err(Failure::new("error.gpg_usage", &[])),
  }
}
