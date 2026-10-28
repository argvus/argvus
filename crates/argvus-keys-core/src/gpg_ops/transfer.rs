//! Key transfer: public and private import and export, and the keyserver commands.

use super::private_files::{absolute, private_temp, readable};
use super::runner::{Failure, Gpg, Outcome, Output, Secret, require_gpg};
use super::*;

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

/// A listing line that starts a secret key packet.
pub(super) fn is_secret_packet(line: &str) -> bool {
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
