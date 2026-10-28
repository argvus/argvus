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

mod key_edits;
mod private_files;
mod runner;
mod transfer;

pub use key_edits::{edit_expire, edit_passwd, edit_revoke, edit_sign, edit_trust};
pub use runner::{Failure, Outcome, Output};
use runner::{Gpg, Secret, ensure_pinentry, find_program, gnupg_home, require_gpg};
pub use transfer::{export_private, export_public, import, import_private, keyserver};

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
