//! GPG keys listed and changed through `argvus gpg` (the GPG keys page).
//!
//! The launcher owns every write: it creates keys, exports public keys and
//! imports them. This module only calls its CLI, parses the TSV it prints and
//! checks the generate form before anything runs.
//!
//! A passphrase typed in the page is sent to the launcher on stdin, never as a
//! process argument, so it does not appear in `ps` or in shell history. The
//! page does not choose the pinentry: gpg-agent uses pinentry-gtk, which the
//! launcher configures.
use std::io::Write;
use std::process::Stdio;

use argvus_control_center_core::process::command;

use super::date::{CalendarDate, DateOrder};

/// Key algorithm offered by the generate form.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum KeyAlgorithm {
  #[default]
  Rsa,
  /// DSA signing key with an ElGamal encryption subkey.
  Dsa,
}

impl KeyAlgorithm {
  /// The value `argvus gpg generate --algo` accepts.
  pub fn as_arg(self) -> &'static str {
    match self {
      Self::Rsa => "rsa",
      Self::Dsa => "dsa",
    }
  }

  /// The other algorithm; the form's algorithm row cycles between the two.
  pub fn toggled(self) -> Self {
    match self {
      Self::Rsa => Self::Dsa,
      Self::Dsa => Self::Rsa,
    }
  }

  /// The key sizes this algorithm offers, in the order the strength row cycles
  /// them. DSA stops at 3072 bits because gpg cannot create larger DSA keys.
  pub fn key_sizes(self) -> &'static [u16] {
    match self {
      Self::Rsa => &[1024, 2048, 4096],
      Self::Dsa => &[1024, 2048, 3072],
    }
  }

  /// The size a new form starts with for this algorithm.
  pub fn default_key_size(self) -> u16 {
    match self {
      Self::Rsa => 4096,
      Self::Dsa => 2048,
    }
  }
}

/// Values of the generate form. A passphrase is set only when
/// `protect_with_passphrase` is on; turning it off makes the key unprotected,
/// which the page confirms first.
#[derive(Clone, PartialEq, Eq)]
pub struct GpgForm {
  pub full_name: String,
  pub email: String,
  pub comment: String,
  pub algorithm: KeyAlgorithm,
  pub key_size: u16,
  pub never_expires: bool,
  /// The expiry date as typed, in the layout of the user's language.
  pub expiry: String,
  pub protect_with_passphrase: bool,
  pub passphrase: String,
  pub passphrase_confirm: String,
}

// The passphrase fields are left out of Debug, so a debug log never shows them.
impl std::fmt::Debug for GpgForm {
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    formatter
      .debug_struct("GpgForm")
      .field("full_name", &self.full_name)
      .field("email", &self.email)
      .field("comment", &self.comment)
      .field("algorithm", &self.algorithm)
      .field("key_size", &self.key_size)
      .field("never_expires", &self.never_expires)
      .field("expiry", &self.expiry)
      .field("protect_with_passphrase", &self.protect_with_passphrase)
      .finish_non_exhaustive()
  }
}

impl GpgForm {
  /// An empty form that expires two years from today, written in the date
  /// layout of `order`. Clearing the form returns to this state.
  pub fn new(order: DateOrder) -> Self {
    Self {
      full_name: String::new(),
      email: String::new(),
      comment: String::new(),
      algorithm: KeyAlgorithm::default(),
      key_size: KeyAlgorithm::default().default_key_size(),
      never_expires: false,
      expiry: CalendarDate::today().add_years(2).display(order),
      protect_with_passphrase: true,
      passphrase: String::new(),
      passphrase_confirm: String::new(),
    }
  }

  /// The user ID gpg stores: `Name (comment) <email>`, with the comment only when
  /// one is typed. The launcher checks the length and control characters.
  pub fn identity(&self) -> String {
    let name = self.full_name.trim();
    let comment = self.comment.trim();
    let email = self.email.trim();
    if comment.is_empty() {
      format!("{name} <{email}>")
    } else {
      format!("{name} ({comment}) <{email}>")
    }
  }

  /// The next key size in the list of the current algorithm. A size the list no
  /// longer has, after the algorithm changed, starts again from the default.
  pub fn next_key_size(&mut self) {
    let sizes = self.algorithm.key_sizes();
    self.key_size = match sizes.iter().position(|size| *size == self.key_size) {
      Some(position) => sizes[(position + 1) % sizes.len()],
      None => self.algorithm.default_key_size(),
    };
  }

  /// Switches the algorithm and keeps the size when the new algorithm offers it.
  pub fn toggle_algorithm(&mut self) {
    self.algorithm = self.algorithm.toggled();
    if !self.algorithm.key_sizes().contains(&self.key_size) {
      self.key_size = self.algorithm.default_key_size();
    }
  }

  /// Empties every field and returns to the defaults, keeping the date layout.
  pub fn clear(&mut self, order: DateOrder) {
    *self = Self::new(order);
  }

  /// Checks the form and returns the expiry argument for gpg: `0` for no expiry,
  /// or the ISO date. A date must be real and after today.
  pub fn expiry_argument(&self, order: DateOrder) -> Result<String, FormError> {
    expiry_argument(self.never_expires, &self.expiry, order)
  }

  /// Checks every field the generate form needs, before the launcher runs.
  pub fn validate(&self, order: DateOrder) -> Result<(), FormError> {
    if self.full_name.trim().is_empty() {
      return Err(FormError::NameRequired);
    }
    if !is_plausible_email(self.email.trim()) {
      return Err(FormError::EmailInvalid);
    }
    self.expiry_argument(order)?;
    if self.protect_with_passphrase {
      if self.passphrase.is_empty() {
        return Err(FormError::PassphraseEmpty);
      }
      // A newline would end the passphrase early on the launcher's stdin.
      if self.passphrase.contains('\n') || self.passphrase.contains('\r') {
        return Err(FormError::PassphraseInvalid);
      }
      if self.passphrase != self.passphrase_confirm {
        return Err(FormError::PassphraseMismatch);
      }
    }
    Ok(())
  }
}

/// The expiry argument gpg takes for a typed date, shared by the generate form
/// and the edit page: `0` for no expiry, or the ISO date. A date must be real
/// and after today.
pub fn expiry_argument(never: bool, expiry: &str, order: DateOrder) -> Result<String, FormError> {
  if never {
    return Ok("0".into());
  }
  let date = CalendarDate::parse(expiry, order).ok_or(FormError::DateInvalid)?;
  if date <= CalendarDate::today() {
    return Err(FormError::DateNotFuture);
  }
  Ok(date.iso())
}

/// A problem with the generate form, shown to the user before anything runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormError {
  NameRequired,
  EmailInvalid,
  DateInvalid,
  DateNotFuture,
  PassphraseEmpty,
  PassphraseInvalid,
  PassphraseMismatch,
}

/// A deliberately loose check: one `@` with text on both sides and no spaces.
/// gpg accepts any user ID, so this only catches typos before the key is made.
fn is_plausible_email(email: &str) -> bool {
  match email.split_once('@') {
    Some((local, domain)) => {
      !local.is_empty() && !domain.is_empty() && !email.contains(char::is_whitespace)
    }
    None => false,
  }
}

/// One primary key of the keyring, as `argvus gpg list` reports it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpgKeyEntry {
  pub fingerprint: String,
  pub key_id: String,
  pub key_type: String,
  /// A date such as `2028-10-12`, or `never`.
  pub expires: String,
  /// Whether the secret key is in the keyring. Only public keys can be
  /// exported from the page, so this is informative.
  pub secret: bool,
  pub validity: String,
  pub user_id: String,
}

/// Where the pinentry setting comes from, as `argvus gpg status` reports it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PinentryOrigin {
  /// Written by ARGVUS, between its markers in `gpg-agent.conf`.
  Argvus,
  /// Written by the user, outside the ARGVUS block.
  User,
  /// Not set; gpg uses its built-in default.
  #[default]
  Default,
}

/// The pinentry program gpg-agent uses, if one is set. The page shows it only;
/// ARGVUS fixes it to pinentry-gtk.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Pinentry {
  pub program: Option<String>,
  pub origin: PinentryOrigin,
}

/// The keyring and the pinentry. `available` is false when `argvus gpg` is not
/// installed, so the page can say so instead of showing an empty list.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GpgKeys {
  pub available: bool,
  pub entries: Vec<GpgKeyEntry>,
  pub pinentry: Pinentry,
  /// Whether the installed launcher is older than [`REQUIRED_LAUNCHER_API`].
  /// The page then offers no key action, because the launcher would answer
  /// with its usage text.
  pub outdated: bool,
}

/// The launcher command-set version this page needs. `argvus gpg status` prints
/// it as its first line (`api<TAB>N`). Keep it equal to `GPG_API` in
/// `argvus-keys gpg` (the `argvus gpg` route), which is the other side of the contract.
pub const REQUIRED_LAUNCHER_API: u32 = 2;

/// Reads the version from `argvus gpg status`. A status without the line comes
/// from a launcher that predates the version, so it counts as version 1.
pub fn parse_api(status: &str) -> u32 {
  status
    .lines()
    .find_map(|line| {
      let fields: Vec<&str> = line.split('\t').collect();
      match fields.as_slice() {
        ["api", version] => version.parse().ok(),
        _ => None,
      }
    })
    .unwrap_or(1)
}

/// Loads the keyring and the pinentry, after checking that the launcher
/// command is installed.
pub fn load() -> GpgKeys {
  let entries = match command("argvus").args(["gpg", "list"]).output() {
    Ok(output) if output.status.success() => parse_list(&String::from_utf8_lossy(&output.stdout)),
    _ => return GpgKeys::default(),
  };
  let status = load_status();
  GpgKeys {
    available: true,
    entries,
    pinentry: parse_status(&status),
    outdated: parse_api(&status) < REQUIRED_LAUNCHER_API,
  }
}

/// The output of `argvus gpg status`, or empty when it cannot run. Empty counts
/// as an outdated launcher, which is the safe answer.
fn load_status() -> String {
  match command("argvus").args(["gpg", "status"]).output() {
    Ok(output) if output.status.success() => String::from_utf8_lossy(&output.stdout).into_owned(),
    _ => String::new(),
  }
}

/// Parses the TSV printed by `argvus gpg list`:
/// `fingerprint, key id, type, expires, secret, validity, user id`.
/// Lines with another field count are skipped, so a launcher that prints extra
/// columns in the future cannot break the page.
pub fn parse_list(listing: &str) -> Vec<GpgKeyEntry> {
  listing.lines().filter_map(parse_line).collect()
}

fn parse_line(line: &str) -> Option<GpgKeyEntry> {
  let fields: Vec<&str> = line.split('\t').collect();
  let [
    fingerprint,
    key_id,
    key_type,
    expires,
    secret,
    validity,
    user_id,
  ] = fields.as_slice()
  else {
    return None;
  };
  if fingerprint.is_empty() {
    return None;
  }
  Some(GpgKeyEntry {
    fingerprint: (*fingerprint).to_owned(),
    key_id: (*key_id).to_owned(),
    key_type: (*key_type).to_owned(),
    expires: (*expires).to_owned(),
    secret: *secret == "yes",
    validity: (*validity).to_owned(),
    user_id: (*user_id).to_owned(),
  })
}

/// Parses the `pinentry` line printed by `argvus gpg status`:
/// `pinentry, program or none, argvus|user|default`. Any other shape keeps the
/// default, so the page never claims a pinentry it cannot read.
pub fn parse_status(status: &str) -> Pinentry {
  for line in status.lines() {
    let fields: Vec<&str> = line.split('\t').collect();
    let ["pinentry", program, origin] = fields.as_slice() else {
      continue;
    };
    let origin = match *origin {
      "argvus" => PinentryOrigin::Argvus,
      "user" => PinentryOrigin::User,
      _ => PinentryOrigin::Default,
    };
    let program = (*program != "none").then(|| (*program).to_owned());
    return Pinentry { program, origin };
  }
  Pinentry::default()
}

/// Replaces a leading `~` with the home directory, since the launcher gets the
/// path as an argument and no shell expands it.
pub fn expand_home(path: &str, home: &str) -> String {
  match path.strip_prefix('~') {
    Some("") => home.to_owned(),
    Some(rest) if rest.starts_with('/') => format!("{home}{rest}"),
    _ => path.to_owned(),
  }
}

/// What `argvus gpg generate` needs: the arguments, and the passphrase that goes
/// to its stdin when the key is protected.
#[derive(Debug, PartialEq, Eq)]
pub struct GenerateRequest {
  pub args: Vec<String>,
  pub passphrase: Option<String>,
}

/// Checks the form and builds the generate request. Nothing runs here, so the
/// rules are testable without the launcher.
pub fn generate_request(form: &GpgForm, order: DateOrder) -> Result<GenerateRequest, FormError> {
  form.validate(order)?;
  let mut args = vec![
    "generate".to_owned(),
    form.identity(),
    "--algo".to_owned(),
    form.algorithm.as_arg().to_owned(),
    "--bits".to_owned(),
    form.key_size.to_string(),
    "--expire".to_owned(),
    form.expiry_argument(order)?,
  ];
  if form.protect_with_passphrase {
    args.push("--passphrase-stdin".to_owned());
    return Ok(GenerateRequest {
      args,
      passphrase: Some(form.passphrase.clone()),
    });
  }
  args.extend(["--no-passphrase".to_owned(), "--yes".to_owned()]);
  Ok(GenerateRequest {
    args,
    passphrase: None,
  })
}

/// Generates a key from the form. A protected key gets its passphrase on stdin,
/// and the launcher hands it to gpg, which never asks the pinentry for it.
pub fn generate(form: &GpgForm, order: DateOrder) -> Result<(), String> {
  let request = generate_request(form, order).map_err(|error| format!("{error:?}"))?;
  let args: Vec<&str> = request.args.iter().map(String::as_str).collect();
  run_with_input(&args, request.passphrase.as_deref())
}

/// Writes the public key of one fingerprint to a file. The launcher refuses to
/// replace an existing file, so the page never overwrites one.
pub fn export_public(fingerprint: &str, path: &str) -> Result<(), String> {
  run(&["export-public", fingerprint, path])
}

/// Imports public keys from a file. The launcher refuses files that hold a
/// private key.
pub fn import(path: &str) -> Result<(), String> {
  run(&["import", path])
}

/// The reason of a revocation, as the launcher names it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RevokeReason {
  None,
  Compromised,
  Superseded,
  Unused,
}

impl RevokeReason {
  /// The word `argvus gpg edit revoke --reason` accepts.
  pub fn as_arg(self) -> &'static str {
    match self {
      Self::None => "none",
      Self::Compromised => "compromised",
      Self::Superseded => "superseded",
      Self::Unused => "unused",
    }
  }

  /// Every reason, in the order the revoke page lists them.
  pub const ALL: [RevokeReason; 4] = [
    Self::None,
    Self::Compromised,
    Self::Superseded,
    Self::Unused,
  ];
}

/// How far a key is trusted, as `argvus gpg edit trust` takes it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrustLevel {
  None,
  Marginal,
  Full,
  Ultimate,
}

impl TrustLevel {
  /// The word `argvus gpg edit trust` accepts.
  pub fn as_arg(self) -> &'static str {
    match self {
      Self::None => "none",
      Self::Marginal => "marginal",
      Self::Full => "full",
      Self::Ultimate => "ultimate",
    }
  }

  /// Every level, in the order the trust rows list them.
  pub const ALL: [TrustLevel; 4] = [Self::None, Self::Marginal, Self::Full, Self::Ultimate];
}

/// Which kind of key an import brings in. The choice decides the launcher command
/// and the messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpgImportKind {
  Public,
  Private,
}

/// Imports a file that holds a private key. The launcher refuses files without one.
pub fn import_private(path: &str) -> Result<(), String> {
  run(&import_private_args(path))
}

pub fn import_private_args(path: &str) -> Vec<&str> {
  vec!["import-private", path]
}

/// Writes the private key of one fingerprint to a file, mode 600. The launcher
/// requires `--yes`, which the page passes after its own confirmation.
pub fn export_private(fingerprint: &str, path: &str) -> Result<(), String> {
  run(&export_private_args(fingerprint, path))
}

pub fn export_private_args<'a>(fingerprint: &'a str, path: &'a str) -> Vec<&'a str> {
  vec!["export-private", fingerprint, path, "--yes"]
}

/// Deletes a key and its secret part, by full fingerprint. Requires `--yes`.
pub fn delete(fingerprint: &str) -> Result<(), String> {
  run(&delete_args(fingerprint))
}

pub fn delete_args(fingerprint: &str) -> Vec<&str> {
  vec!["delete", fingerprint, "--yes"]
}

/// Sets the expiry of a key and its subkeys. `expiry` is the argument from
/// [`expiry_argument`]: `0` or an ISO date.
pub fn set_expiry(fingerprint: &str, expiry: &str) -> Result<(), String> {
  run(&expire_args(fingerprint, expiry))
}

pub fn expire_args<'a>(fingerprint: &'a str, expiry: &'a str) -> Vec<&'a str> {
  vec!["edit", "expire", fingerprint, expiry]
}

/// Changes the passphrase; the pinentry window asks for the old and new one.
pub fn change_passphrase(fingerprint: &str) -> Result<(), String> {
  run(&passwd_args(fingerprint))
}

pub fn passwd_args(fingerprint: &str) -> Vec<&str> {
  vec!["edit", "passwd", fingerprint]
}

/// Certifies a key with the secret key of `signer`. Requires `--yes`.
pub fn sign_key(fingerprint: &str, signer: &str) -> Result<(), String> {
  run(&sign_args(fingerprint, signer))
}

pub fn sign_args<'a>(fingerprint: &'a str, signer: &'a str) -> Vec<&'a str> {
  vec!["edit", "sign", fingerprint, signer, "--yes"]
}

/// Revokes a key with a reason. Requires `--yes`.
pub fn revoke(fingerprint: &str, reason: RevokeReason) -> Result<(), String> {
  run(&revoke_args(fingerprint, reason))
}

pub fn revoke_args(fingerprint: &str, reason: RevokeReason) -> Vec<&str> {
  vec![
    "edit",
    "revoke",
    fingerprint,
    "--reason",
    reason.as_arg(),
    "--yes",
  ]
}

/// Sets the trust of a key.
pub fn set_trust(fingerprint: &str, level: TrustLevel) -> Result<(), String> {
  run(&trust_args(fingerprint, level))
}

pub fn trust_args(fingerprint: &str, level: TrustLevel) -> Vec<&str> {
  vec!["edit", "trust", fingerprint, level.as_arg()]
}

/// Publishes a key to the keyserver gpg is configured with. Requires `--yes`.
pub fn keyserver_send(fingerprint: &str) -> Result<(), String> {
  run(&keyserver_send_args(fingerprint))
}

pub fn keyserver_send_args(fingerprint: &str) -> Vec<&str> {
  vec!["keyserver", "send", fingerprint, "--yes"]
}

/// Fetches a key from the keyserver gpg is configured with.
pub fn keyserver_recv(fingerprint: &str) -> Result<(), String> {
  run(&keyserver_recv_args(fingerprint))
}

pub fn keyserver_recv_args(fingerprint: &str) -> Vec<&str> {
  vec!["keyserver", "recv", fingerprint]
}

/// The armored public key of one fingerprint, read from the launcher's standard
/// output so the page can copy it without writing a file.
pub fn public_key(fingerprint: &str) -> Result<String, String> {
  let output = command("argvus")
    .args(["gpg", "export-public", fingerprint, "-"])
    .output()
    .map_err(|error| format!("failed to run argvus-gpg: {error}"))?;
  if !output.status.success() {
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
    return Err(if stderr.is_empty() {
      "argvus-gpg failed".into()
    } else {
      stderr
    });
  }
  Ok(
    String::from_utf8_lossy(&output.stdout)
      .trim_end()
      .to_owned(),
  )
}

fn run(args: &[&str]) -> Result<(), String> {
  run_with_input(args, None)
}

/// Runs `argvus gpg` with `input` written to its stdin, then closes it. Only
/// passphrases use this, so the secret stays out of the argument list.
fn run_with_input(args: &[&str], input: Option<&str>) -> Result<(), String> {
  let mut child = command("argvus")
    .arg("gpg")
    .args(args)
    .stdin(if input.is_some() {
      Stdio::piped()
    } else {
      Stdio::null()
    })
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .map_err(|error| format!("failed to run argvus-gpg: {error}"))?;
  if let (Some(input), Some(mut stdin)) = (input, child.stdin.take()) {
    // A launcher that stops reading early closes the pipe; the exit status below
    // then reports the failure, so this write error is not the one to show.
    let _ = stdin.write_all(input.as_bytes());
    let _ = stdin.write_all(b"\n");
  }
  let output = child
    .wait_with_output()
    .map_err(|error| format!("failed to wait for argvus-gpg: {error}"))?;
  if output.status.success() {
    return Ok(());
  }
  let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
  Err(if stderr.is_empty() {
    "argvus-gpg failed".into()
  } else {
    stderr
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  const FP_ALICE: &str = "AAAA111122223333AAAA111122223333AAAAAAAA";
  const FP_BOB: &str = "CCCCCCCCCCCCCCCCDDDDDDDDDDDDDDDDEEEEEEEE";

  fn filled_form() -> GpgForm {
    GpgForm {
      full_name: "Alice Example".into(),
      email: "alice@example.com".into(),
      expiry: "2099/05/03".into(),
      passphrase: "correct horse".into(),
      passphrase_confirm: "correct horse".into(),
      ..GpgForm::new(DateOrder::YearFirst)
    }
  }

  #[test]
  fn parses_each_listed_primary_key() {
    let listing = format!(
      "{FP_ALICE}\tAAAA111122223333\ted25519\tnever\tyes\tu\tAlice <alice@example.com>\n\
       {FP_BOB}\tCCCCCCCCCCCCCCCC\trsa\t2030-03-17\tno\te\tBob: Test <bob@example.com>\n"
    );
    let entries = parse_list(&listing);
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].fingerprint, FP_ALICE);
    assert_eq!(entries[0].key_type, "ed25519");
    assert_eq!(entries[0].expires, "never");
    assert!(entries[0].secret);
    assert_eq!(entries[1].expires, "2030-03-17");
    assert!(!entries[1].secret);
    assert_eq!(entries[1].validity, "e");
    assert_eq!(entries[1].user_id, "Bob: Test <bob@example.com>");
  }

  #[test]
  fn skips_lines_with_another_column_count() {
    let entries = parse_list("AAAA\tAAAA\ted25519\n\n");
    assert!(entries.is_empty());
  }

  #[test]
  fn skips_lines_without_a_fingerprint() {
    let entries = parse_list("\tAAAA\ted25519\tnever\tyes\tu\tAlice\n");
    assert!(entries.is_empty());
  }

  #[test]
  fn reads_the_pinentry_line_and_its_origin() {
    let managed = parse_status("gnupg\tyes\npinentry\t/usr/bin/pinentry-gtk\targvus\n");
    assert_eq!(managed.program.as_deref(), Some("/usr/bin/pinentry-gtk"));
    assert_eq!(managed.origin, PinentryOrigin::Argvus);

    let user = parse_status("pinentry\t/usr/bin/pinentry-curses\tuser\n");
    assert_eq!(user.origin, PinentryOrigin::User);

    let none = parse_status("pinentry\tnone\tdefault\n");
    assert_eq!(none, Pinentry::default());
  }

  #[test]
  fn an_unreadable_status_keeps_the_default_pinentry() {
    assert_eq!(parse_status("garbage\n"), Pinentry::default());
  }

  #[test]
  fn expands_a_leading_tilde_only() {
    assert_eq!(expand_home("~/key.asc", "/home/u"), "/home/u/key.asc");
    assert_eq!(expand_home("~", "/home/u"), "/home/u");
    assert_eq!(expand_home("/tmp/key.asc", "/home/u"), "/tmp/key.asc");
    assert_eq!(expand_home("~other/key.asc", "/home/u"), "~other/key.asc");
  }

  #[test]
  fn the_api_line_gives_the_launcher_version() {
    assert_eq!(parse_api("api\t2\ngnupg\tyes\n"), 2);
    assert_eq!(REQUIRED_LAUNCHER_API, 2);
  }

  #[test]
  fn a_status_without_the_api_line_counts_as_the_first_launcher() {
    assert_eq!(parse_api("gnupg\tyes\npinentry\tnone\tdefault\n"), 1);
    assert_eq!(parse_api(""), 1);
  }

  #[test]
  fn an_unreadable_api_line_counts_as_outdated() {
    assert_eq!(parse_api("api\tnew\n"), 1);
  }

  // These vectors are the other side of the contract with `argvus gpg`; the
  // argvus-keys' gpg command checks the same subcommands and argument counts.
  #[test]
  fn edit_actions_send_the_words_the_launcher_parses() {
    let fp = FP_ALICE;
    assert_eq!(passwd_args(fp), ["edit", "passwd", fp]);
    assert_eq!(
      trust_args(fp, TrustLevel::Marginal),
      ["edit", "trust", fp, "marginal"]
    );
    assert_eq!(
      trust_args(fp, TrustLevel::Full),
      ["edit", "trust", fp, "full"]
    );
    assert_eq!(
      trust_args(fp, TrustLevel::Ultimate),
      ["edit", "trust", fp, "ultimate"]
    );
    assert_eq!(
      revoke_args(fp, RevokeReason::Unused),
      ["edit", "revoke", fp, "--reason", "unused", "--yes"]
    );
    assert_eq!(sign_args(fp, FP_BOB), ["edit", "sign", fp, FP_BOB, "--yes"]);
    assert_eq!(
      expire_args(fp, "2031-06-30"),
      ["edit", "expire", fp, "2031-06-30"]
    );
  }

  #[test]
  fn keyserver_and_delete_actions_send_the_words_the_launcher_parses() {
    let fp = FP_ALICE;
    assert_eq!(keyserver_recv_args(fp), ["keyserver", "recv", fp]);
    assert_eq!(keyserver_send_args(fp), ["keyserver", "send", fp, "--yes"]);
    assert_eq!(delete_args(fp), ["delete", fp, "--yes"]);
    assert_eq!(
      export_private_args(fp, "/tmp/k.asc"),
      ["export-private", fp, "/tmp/k.asc", "--yes"]
    );
    assert_eq!(
      import_private_args("/tmp/k.asc"),
      ["import-private", "/tmp/k.asc"]
    );
  }

  #[test]
  fn revoke_reasons_and_trust_levels_use_the_launcher_words() {
    assert_eq!(RevokeReason::None.as_arg(), "none");
    assert_eq!(RevokeReason::Compromised.as_arg(), "compromised");
    assert_eq!(RevokeReason::Superseded.as_arg(), "superseded");
    assert_eq!(RevokeReason::Unused.as_arg(), "unused");
    assert_eq!(TrustLevel::Marginal.as_arg(), "marginal");
    assert_eq!(TrustLevel::Ultimate.as_arg(), "ultimate");
    assert_eq!(RevokeReason::ALL.len(), 4);
    assert_eq!(TrustLevel::ALL.len(), 4);
  }

  #[test]
  fn an_edit_that_never_expires_sends_zero_whatever_the_text() {
    assert_eq!(
      expiry_argument(true, "not a date", DateOrder::YearFirst),
      Ok("0".into())
    );
    assert_eq!(
      expiry_argument(false, "2099/01/31", DateOrder::YearFirst),
      Ok("2099-01-31".into())
    );
  }

  #[test]
  fn the_identity_puts_the_comment_between_name_and_email() {
    let mut form = filled_form();
    assert_eq!(form.identity(), "Alice Example <alice@example.com>");
    form.comment = " work ".into();
    assert_eq!(form.identity(), "Alice Example (work) <alice@example.com>");
  }

  #[test]
  fn the_algorithm_and_size_rows_cycle_within_the_algorithm() {
    let mut form = GpgForm::new(DateOrder::YearFirst);
    assert_eq!((form.algorithm, form.key_size), (KeyAlgorithm::Rsa, 4096));
    form.next_key_size();
    assert_eq!(form.key_size, 1024);
    form.next_key_size();
    assert_eq!(form.key_size, 2048);

    form.toggle_algorithm();
    assert_eq!((form.algorithm, form.key_size), (KeyAlgorithm::Dsa, 2048));
    form.next_key_size();
    assert_eq!(form.key_size, 3072);
    form.next_key_size();
    assert_eq!(form.key_size, 1024);
  }

  #[test]
  fn switching_to_dsa_moves_a_size_dsa_does_not_offer_to_its_default() {
    let mut form = GpgForm::new(DateOrder::YearFirst);
    assert_eq!(form.key_size, 4096);
    form.toggle_algorithm();
    assert_eq!(form.key_size, KeyAlgorithm::Dsa.default_key_size());
  }

  #[test]
  fn the_default_form_asks_for_a_passphrase_and_expires_in_two_years() {
    let form = GpgForm::new(DateOrder::DayFirst);
    assert!(form.protect_with_passphrase);
    assert!(!form.never_expires);
    let expected = CalendarDate::today()
      .add_years(2)
      .display(DateOrder::DayFirst);
    assert_eq!(form.expiry, expected);
  }

  #[test]
  fn clearing_the_form_restores_the_defaults() {
    let mut form = filled_form();
    form.clear(DateOrder::YearFirst);
    assert_eq!(form, GpgForm::new(DateOrder::YearFirst));
  }

  #[test]
  fn never_expires_passes_zero_and_ignores_the_date() {
    let form = GpgForm {
      never_expires: true,
      expiry: "not a date".into(),
      ..filled_form()
    };
    assert_eq!(form.expiry_argument(DateOrder::YearFirst), Ok("0".into()));
  }

  #[test]
  fn a_typed_date_becomes_iso_for_gpg() {
    let form = GpgForm {
      expiry: "03/05/2099".into(),
      ..filled_form()
    };
    assert_eq!(
      form.expiry_argument(DateOrder::DayFirst),
      Ok("2099-05-03".into())
    );
  }

  #[test]
  fn a_past_or_impossible_date_is_refused() {
    let past = GpgForm {
      expiry: "2001/01/01".into(),
      ..filled_form()
    };
    assert_eq!(
      past.expiry_argument(DateOrder::YearFirst),
      Err(FormError::DateNotFuture)
    );
    let impossible = GpgForm {
      expiry: "2099/02/30".into(),
      ..filled_form()
    };
    assert_eq!(
      impossible.expiry_argument(DateOrder::YearFirst),
      Err(FormError::DateInvalid)
    );
  }

  #[test]
  fn the_form_needs_a_name_and_a_plausible_email() {
    let no_name = GpgForm {
      full_name: "  ".into(),
      ..filled_form()
    };
    assert_eq!(
      no_name.validate(DateOrder::YearFirst),
      Err(FormError::NameRequired)
    );
    let bad_email = GpgForm {
      email: "alice at example".into(),
      ..filled_form()
    };
    assert_eq!(
      bad_email.validate(DateOrder::YearFirst),
      Err(FormError::EmailInvalid)
    );
  }

  #[test]
  fn a_protected_key_needs_matching_passphrases_without_newlines() {
    let empty = GpgForm {
      passphrase: String::new(),
      passphrase_confirm: String::new(),
      ..filled_form()
    };
    assert_eq!(
      empty.validate(DateOrder::YearFirst),
      Err(FormError::PassphraseEmpty)
    );
    let mismatch = GpgForm {
      passphrase_confirm: "other".into(),
      ..filled_form()
    };
    assert_eq!(
      mismatch.validate(DateOrder::YearFirst),
      Err(FormError::PassphraseMismatch)
    );
    let multiline = GpgForm {
      passphrase: "a\nb".into(),
      passphrase_confirm: "a\nb".into(),
      ..filled_form()
    };
    assert_eq!(
      multiline.validate(DateOrder::YearFirst),
      Err(FormError::PassphraseInvalid)
    );
  }

  #[test]
  fn a_protected_key_sends_the_passphrase_on_stdin_and_never_in_arguments() {
    let request = generate_request(&filled_form(), DateOrder::YearFirst).expect("valid form");
    assert!(request.args.contains(&"--passphrase-stdin".to_owned()));
    assert!(
      request
        .args
        .iter()
        .all(|arg| !arg.contains("correct horse"))
    );
    assert_eq!(request.passphrase.as_deref(), Some("correct horse"));
  }

  #[test]
  fn an_unprotected_key_asks_the_launcher_for_confirmation_and_sends_no_passphrase() {
    let form = GpgForm {
      protect_with_passphrase: false,
      ..filled_form()
    };
    let request = generate_request(&form, DateOrder::YearFirst).expect("valid form");
    assert!(request.args.contains(&"--no-passphrase".to_owned()));
    assert!(request.args.contains(&"--yes".to_owned()));
    assert_eq!(request.passphrase, None);
  }

  #[test]
  fn generation_arguments_carry_the_algorithm_size_and_expiry() {
    let form = GpgForm {
      algorithm: KeyAlgorithm::Dsa,
      key_size: 2048,
      ..filled_form()
    };
    let request = generate_request(&form, DateOrder::YearFirst).expect("valid form");
    assert_eq!(
      request.args,
      [
        "generate",
        "Alice Example <alice@example.com>",
        "--algo",
        "dsa",
        "--bits",
        "2048",
        "--expire",
        "2099-05-03",
        "--passphrase-stdin",
      ]
    );
  }
}
