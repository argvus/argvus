//! `argvus-keys`: SSH key manager of ARGVUS. The public command is `argvus keys`.
//!
//! Usage:
//!   argvus-keys list                                  print keys as TSV
//!   argvus-keys generate NAME [--type ed25519|rsa] [--comment C] [--passphrase]
//!   argvus-keys load NAME                             add the key to the agent
//!   argvus-keys unload NAME                           remove the key from the agent
//!   argvus-keys unload-all                            remove every key from the agent
//!   argvus-keys delete NAME                           delete the key pair from disk
//!
//! `generate-run` and `load-run` are the steps a terminal runs for a key with a
//! passphrase. They are not meant to be typed by hand.

mod gpg_cli;
mod messages;

use argvus_keys_core::store::{self, KeyError};
use argvus_keys_core::{KeyType, valid_comment, valid_name};
use messages::{print_launcher, report_error};
use std::io::{IsTerminal, Write};
use std::process::{Command, ExitCode, Stdio};

/// Runs one command of this crate. `arguments` are the words after the command
/// name. The `argvus` dispatcher and the binary both call this function.
pub fn run_command(arguments: Vec<String>) -> ExitCode {
  prepare_environment();
  run(&arguments)
}

/// The agent socket is exported for `ssh-add`, because Hyprland does not carry
/// the session variable into the terminals it opens.
fn prepare_environment() {
  if std::env::var_os("SSH_AUTH_SOCK").is_none()
    && let Some(socket) = store::agent_socket()
  {
    // SAFETY: called once at startup, before any other thread exists.
    unsafe { std::env::set_var("SSH_AUTH_SOCK", socket) };
  }
  // Every file this command creates is private to the user.
  // SAFETY: umask only changes the creation mask of this process.
  unsafe { libc::umask(0o077) };
}

fn run(arguments: &[String]) -> ExitCode {
  let Some(command) = arguments.first().map(String::as_str) else {
    return help();
  };
  let rest = &arguments[1..];
  match command {
    "" | "-h" | "--help" | "help" => help(),
    "-V" | "--version" => {
      println!("argvus-keys {}", env!("CARGO_PKG_VERSION"));
      ExitCode::SUCCESS
    }
    "list" => list(),
    "gpg" => gpg_cli::run(rest),
    "generate" => generate(rest),
    "generate-run" => generate_run(rest),
    "load" => load(rest),
    "load-run" => load_run(rest),
    "unload" => unload(rest),
    "unload-all" => unload_all(),
    "delete" => delete(rest),
    _ => usage(),
  }
}

fn help() -> ExitCode {
  print_launcher("help.keys", &[], messages::Stream::Stdout);
  ExitCode::SUCCESS
}

fn usage() -> ExitCode {
  print_launcher("error.keys_usage", &[], messages::Stream::Stderr);
  ExitCode::from(1)
}

/// Reports a key error for a shortcut run, where stderr is not visible.
fn fail(error: &KeyError) -> ExitCode {
  report_error(&messages::key_error(error));
  ExitCode::from(1)
}

fn list() -> ExitCode {
  for row in store::list(&store::ssh_dir()) {
    println!("{}", row.tsv());
  }
  ExitCode::SUCCESS
}

/// `generate NAME [--type T] [--comment C] [--passphrase]`.
fn generate(arguments: &[String]) -> ExitCode {
  let mut name: Option<&str> = None;
  let mut key_type = "ed25519";
  let mut comment: Option<&str> = None;
  let mut passphrase = false;
  let mut index = 0;
  while index < arguments.len() {
    match arguments[index].as_str() {
      "--type" => {
        let Some(value) = arguments.get(index + 1) else {
          return usage();
        };
        key_type = value;
        index += 2;
      }
      "--comment" => {
        let Some(value) = arguments.get(index + 1) else {
          return usage();
        };
        comment = Some(value);
        index += 2;
      }
      "--passphrase" => {
        passphrase = true;
        index += 1;
      }
      flag if flag.starts_with('-') => return usage(),
      value => {
        if name.is_some() {
          return usage();
        }
        name = Some(value);
        index += 1;
      }
    }
  }
  let Some(name) = name else {
    return usage();
  };
  if !valid_name(name) {
    return fail(&KeyError::InvalidName(name.into()));
  }
  let Some(key_type) = KeyType::parse(key_type) else {
    return fail(&KeyError::InvalidType(key_type.into()));
  };
  let comment = comment.unwrap_or(name);
  if !valid_comment(comment) {
    return usage();
  }
  let dir = store::ssh_dir();
  if dir.join(name).exists() || dir.join(format!("{name}.pub")).exists() {
    return fail(&KeyError::Exists(name.into()));
  }
  if passphrase {
    // The passphrase is asked where ssh-keygen runs: here when a terminal is
    // attached, otherwise in a new ARGVUS terminal.
    if std::io::stdin().is_terminal() {
      return match store::create(&dir, name, key_type, comment, true) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => fail(&error),
      };
    }
    return open_terminal(&format!(
      "generate-run {name} --type {} --comment {comment}",
      key_type.as_str()
    ));
  }
  match store::create(&dir, name, key_type, comment, false) {
    Ok(()) => {
      print_launcher(
        "message.key_generated",
        &[format!("name={name}")],
        messages::Stream::Stdout,
      );
      ExitCode::SUCCESS
    }
    Err(error) => fail(&error),
  }
}

/// `generate-run NAME --type T --comment C`, in the terminal of a passphrase key.
fn generate_run(arguments: &[String]) -> ExitCode {
  let [name, type_flag, key_type, comment_flag, comment] = arguments else {
    return usage();
  };
  if type_flag != "--type" || comment_flag != "--comment" {
    return usage();
  }
  let Some(key_type) = KeyType::parse(key_type) else {
    return usage();
  };
  if !valid_name(name) || !valid_comment(comment) {
    return usage();
  }
  let dir = store::ssh_dir();
  let status = match store::create(&dir, name, key_type, comment, true) {
    Ok(()) => 0,
    Err(KeyError::Command(_, code)) => code,
    Err(error) => {
      report_error(&messages::key_error(&error));
      1
    }
  };
  wait_for_enter();
  ExitCode::from(status as u8)
}

/// `load NAME`: a key without a passphrase loads here; a key with one loads
/// here when a terminal is attached, and otherwise in a new ARGVUS terminal.
fn load(arguments: &[String]) -> ExitCode {
  let [name] = arguments else {
    return usage();
  };
  let private = match store::private_key(&store::ssh_dir(), name) {
    Ok(private) => private,
    Err(error) => return fail(&error),
  };
  if !store::has_passphrase(&private) {
    return match store::load_without_prompt(&private) {
      Ok(0) => ExitCode::SUCCESS,
      Ok(code) => ExitCode::from(code as u8),
      Err(error) => fail(&error),
    };
  }
  if std::io::stdin().is_terminal() {
    return match store::load_with_prompt(&private) {
      Ok(0) => ExitCode::SUCCESS,
      Ok(code) => ExitCode::from(code as u8),
      Err(error) => fail(&error),
    };
  }
  open_terminal(&format!("load-run {name}"))
}

/// `load-run NAME`, in the terminal of a key with a passphrase.
fn load_run(arguments: &[String]) -> ExitCode {
  let [name] = arguments else {
    return usage();
  };
  let private = match store::private_key(&store::ssh_dir(), name) {
    Ok(private) => private,
    Err(error) => return fail(&error),
  };
  let status = store::load_with_prompt(&private).unwrap_or(1);
  wait_for_enter();
  ExitCode::from(status as u8)
}

fn unload(arguments: &[String]) -> ExitCode {
  let [name] = arguments else {
    return usage();
  };
  match store::unload(&store::ssh_dir(), name) {
    Ok(0) => {
      print_launcher(
        "message.key_unloaded",
        &[format!("name={name}")],
        messages::Stream::Stdout,
      );
      ExitCode::SUCCESS
    }
    Ok(_) => ExitCode::from(1),
    Err(error) => fail(&error),
  }
}

fn unload_all() -> ExitCode {
  match store::unload_all() {
    Ok(0) => {
      print_launcher("message.keys_unloaded_all", &[], messages::Stream::Stdout);
      ExitCode::SUCCESS
    }
    _ => ExitCode::from(1),
  }
}

fn delete(arguments: &[String]) -> ExitCode {
  let [name] = arguments else {
    return usage();
  };
  match store::delete(&store::ssh_dir(), name) {
    Ok(()) => {
      print_launcher(
        "message.key_deleted",
        &[format!("name={name}")],
        messages::Stream::Stdout,
      );
      ExitCode::SUCCESS
    }
    Err(error) => fail(&error),
  }
}

/// Runs an `argvus keys` subcommand in an ARGVUS terminal. Only the validated
/// names and types are interpolated, so the command needs no quoting beyond the
/// Lua string itself.
fn open_terminal(subcommand: &str) -> ExitCode {
  let command = format!("argvus terminal --class argvus-keys argvus keys {subcommand}");
  let lua = format!(
    "hl.dsp.exec_cmd({})",
    argvus_keys_core::keys::lua_string(&command)
  );
  let output = Command::new("hyprctl")
    .args(["dispatch", &lua])
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .output();
  match output {
    Ok(output) if output.status.success() => ExitCode::SUCCESS,
    Ok(output) => {
      let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
      text.push_str(&String::from_utf8_lossy(&output.stderr));
      report_error(text.trim_end());
      ExitCode::from(1)
    }
    Err(error) => {
      report_error(&error.to_string());
      ExitCode::from(1)
    }
  }
}

/// Pauses after an interactive step, so its result stays readable in the
/// terminal window.
fn wait_for_enter() {
  println!();
  print_launcher("message.keys_session_ended", &[], messages::Stream::Stdout);
  let mut line = String::new();
  let _ = std::io::stdin().read_line(&mut line);
  let _ = std::io::stdout().flush();
}
