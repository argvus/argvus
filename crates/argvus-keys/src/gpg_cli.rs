//! The `gpg` subcommand of `argvus keys`: the command line of the GPG keys. The
//! arguments are read as the shell script read them, and every outcome is
//! printed through the messages of argvus-i18n.

use crate::messages::{Stream, print_launcher, report_error_as, text_launcher};
use argvus_keys_core::gpg_ops::{self, Failure, Output};
use std::io::{BufRead, Write};
use std::process::ExitCode;

/// Runs `argvus keys gpg ARGUMENTS`.
pub fn run(arguments: &[String]) -> ExitCode {
  let Some(command) = arguments.first().map(String::as_str) else {
    return help();
  };
  let rest = &arguments[1..];
  match command {
    "" | "-h" | "--help" | "help" => help(),
    "list" => show(gpg_ops::list()),
    "status" => show(gpg_ops::status()),
    "generate" => generate(rest),
    "export-public" => export_public(rest),
    "import" => match rest {
      [file] => show(gpg_ops::import(file)),
      _ => usage(),
    },
    "import-private" => match rest {
      [file] => show(gpg_ops::import_private(file)),
      _ => usage(),
    },
    "export-private" => export_private(rest),
    "delete" => delete(rest),
    "edit" => edit(rest),
    "keyserver" => keyserver(rest),
    _ => usage(),
  }
}

fn help() -> ExitCode {
  print_launcher("help.gpg", &[], Stream::Stdout);
  ExitCode::SUCCESS
}

fn usage() -> ExitCode {
  print_launcher("error.gpg_usage", &[], Stream::Stderr);
  ExitCode::from(1)
}

/// Prints the outcome of a command: its text, its message, or the failure.
fn show(outcome: Result<Output, Failure>) -> ExitCode {
  match outcome {
    Ok(Output::Text(text)) => {
      print!("{text}");
      let _ = std::io::stdout().flush();
      ExitCode::SUCCESS
    }
    Ok(Output::Message(key, arguments)) => {
      print_launcher(key, &arguments, Stream::Stdout);
      ExitCode::SUCCESS
    }
    Ok(Output::Nothing) => ExitCode::SUCCESS,
    Err(failure) => {
      report_error_as("argvus-gpg", &text_launcher(failure.key, &failure.args));
      ExitCode::from(1)
    }
  }
}

/// `generate UID [--algo rsa|dsa] [--bits N] [--expire X] [--no-passphrase] [--passphrase-stdin] [--yes]`.
fn generate(arguments: &[String]) -> ExitCode {
  let mut options = gpg_ops::GenerateOptions {
    uid: None,
    algo: "rsa".into(),
    bits: None,
    expire: "2y".into(),
    no_passphrase: false,
    passphrase_stdin: false,
    confirmed: false,
  };
  let mut index = 0;
  while index < arguments.len() {
    let argument = arguments[index].as_str();
    match argument {
      "--algo" | "--bits" | "--expire" => {
        let Some(value) = arguments.get(index + 1) else {
          return usage();
        };
        match argument {
          "--algo" => options.algo = value.clone(),
          "--bits" => options.bits = Some(value.clone()),
          _ => options.expire = value.clone(),
        }
        index += 2;
        continue;
      }
      "--no-passphrase" => options.no_passphrase = true,
      "--passphrase-stdin" => options.passphrase_stdin = true,
      "--yes" => options.confirmed = true,
      flag if flag.starts_with('-') => return usage(),
      value => {
        if options.uid.is_some() {
          return usage();
        }
        options.uid = Some(value.to_string());
      }
    }
    index += 1;
  }
  if options.uid.is_none() {
    return usage();
  }
  // The two ways to protect a key cannot be combined.
  if options.no_passphrase && options.passphrase_stdin {
    return usage();
  }
  // The first line of stdin is the passphrase; it never comes from an argument.
  let stdin_line = || {
    let mut line = String::new();
    let _ = std::io::stdin().lock().read_line(&mut line);
    line.strip_suffix('\n').unwrap_or(&line).to_string()
  };
  show(gpg_ops::generate(options, stdin_line))
}

/// `export-public ID FILE [--force]`; `-` as FILE writes to standard output.
fn export_public(arguments: &[String]) -> ExitCode {
  let [id, file, options @ ..] = arguments else {
    return usage();
  };
  let mut force = false;
  for option in options {
    match option.as_str() {
      "--force" => force = true,
      _ => return usage(),
    }
  }
  show(gpg_ops::export_public(id, file, force))
}

/// `export-private FINGERPRINT FILE [--force] [--yes]`.
fn export_private(arguments: &[String]) -> ExitCode {
  let [fingerprint, file, options @ ..] = arguments else {
    return usage();
  };
  let mut force = false;
  let mut confirmed = false;
  for option in options {
    match option.as_str() {
      "--force" => force = true,
      "--yes" => confirmed = true,
      _ => return usage(),
    }
  }
  show(gpg_ops::export_private(fingerprint, file, force, confirmed))
}

/// `delete FINGERPRINT [--yes]`.
fn delete(arguments: &[String]) -> ExitCode {
  let [fingerprint, options @ ..] = arguments else {
    return usage();
  };
  let mut confirmed = false;
  for option in options {
    match option.as_str() {
      "--yes" => confirmed = true,
      _ => return usage(),
    }
  }
  show(gpg_ops::delete(fingerprint, confirmed))
}

/// `edit ACTION ...`: expire, passwd, sign, revoke or trust.
fn edit(arguments: &[String]) -> ExitCode {
  let Some((action, rest)) = arguments.split_first() else {
    return usage();
  };
  match action.as_str() {
    "expire" => match rest {
      [fingerprint, expire] => show(gpg_ops::edit_expire(fingerprint, expire)),
      _ => usage(),
    },
    "passwd" => match rest {
      [fingerprint] => show(gpg_ops::edit_passwd(fingerprint)),
      _ => usage(),
    },
    "sign" => {
      let [fingerprint, signer, options @ ..] = rest else {
        return usage();
      };
      let Some(confirmed) = confirmation(options) else {
        return usage();
      };
      show(gpg_ops::edit_sign(fingerprint, signer, confirmed))
    }
    "revoke" => {
      let [fingerprint, options @ ..] = rest else {
        return usage();
      };
      let mut reason = String::new();
      let mut confirmed = false;
      let mut index = 0;
      while index < options.len() {
        match options[index].as_str() {
          "--reason" => {
            let Some(value) = options.get(index + 1) else {
              return usage();
            };
            reason = value.clone();
            index += 2;
            continue;
          }
          "--yes" => confirmed = true,
          _ => return usage(),
        }
        index += 1;
      }
      show(gpg_ops::edit_revoke(fingerprint, &reason, confirmed))
    }
    "trust" => match rest {
      [fingerprint, level] => show(gpg_ops::edit_trust(fingerprint, level)),
      _ => usage(),
    },
    _ => usage(),
  }
}

/// The `--yes` options of a command: any number of them, and nothing else.
fn confirmation(options: &[String]) -> Option<bool> {
  let mut confirmed = false;
  for option in options {
    match option.as_str() {
      "--yes" => confirmed = true,
      _ => return None,
    }
  }
  Some(confirmed)
}

/// `keyserver send|recv FINGERPRINT [--yes]`.
fn keyserver(arguments: &[String]) -> ExitCode {
  let [action, fingerprint, options @ ..] = arguments else {
    return usage();
  };
  let mut confirmed = false;
  for option in options {
    match option.as_str() {
      "--yes" => confirmed = true,
      _ => return usage(),
    }
  }
  show(gpg_ops::keyserver(action, fingerprint, confirmed))
}
