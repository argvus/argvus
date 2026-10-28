//! Edits of an existing key: expiry, passphrase, signature, revocation and owner trust.

use super::runner::{Failure, Gpg, Outcome, Output, Secret, ensure_pinentry, require_gpg};
use super::*;

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
