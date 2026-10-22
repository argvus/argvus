//! Command dispatch and presentation logic.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use argvus_accounts_core::metadata;
use argvus_accounts_core::passwd;
use argvus_accounts_core::password;
use argvus_accounts_core::{AccountManager, Error, Result};

use crate::cli::{Cli, Command, SelfAction};
use crate::elevate::{self, Requirement};

/// Executes the parsed command line.
pub fn execute(cli: &Cli) -> Result<()> {
  let manager = AccountManager::new();
  if cli.verbose {
    eprintln!(
      "argvus-accounts: authorization provider '{}', actor '{}'",
      manager.provider_name(),
      AccountManager::current_username().unwrap_or_else(|_| "?".to_string())
    );
  }

  match &cli.command {
    Command::Manage => {
      use std::io::Read;
      let mut input = String::new();
      std::io::stdin()
        .take(65537)
        .read_to_string(&mut input)
        .map_err(|err| Error::system(err.to_string()))?;
      if input.len() > 65536 {
        return Err(Error::InvalidOperation("request too large".into()));
      }
      let result = argvus_accounts_core::admin::execute(&input)?;
      println!("{result}");
      Ok(())
    }
    Command::List { all } => cmd_list(&manager, *all),
    Command::Show { user } => cmd_show(&manager, user),
    Command::Name { user, name } => {
      // Validate before elevation so a typo never opens a polkit prompt.
      metadata::validate_display_name(name)?;
      elevate::ensure(&Requirement::Always)?;
      manager.set_display_name(user, name)?;
      println!("Display name updated successfully.");
      Ok(())
    }
    Command::Avatar {
      user,
      image,
      remove,
    } => {
      // Expand tilde before any elevation: pkexec resets the
      // environment and '~' would resolve to root's home.
      let expanded = image.as_deref().map(expand_tilde);
      elevate::ensure(&Requirement::OtherUsersOnly {
        target: user.clone(),
      })?;
      cmd_avatar(&manager, user, expanded.as_deref(), *remove)
    }
    Command::Groups { user, add, remove } => cmd_groups(&manager, user, add, remove),
    Command::Passwd { user } => cmd_passwd(&manager, user),
    Command::SelfAccount { action } => cmd_self(&manager, action.as_ref(), cli.verbose),
  }
}

fn cmd_list(manager: &AccountManager, all: bool) -> Result<()> {
  let users = manager.list_users(all)?;
  println!("{:<17}NAME", "USER");
  for user in users {
    let name = user.display_name().unwrap_or("(none)");
    println!("{:<17}{}", user.username, name);
  }
  Ok(())
}

fn cmd_show(manager: &AccountManager, user: &str) -> Result<()> {
  let view = manager.get_user(user)?;
  let info = &view.info;
  println!("Username: {}", info.username);
  println!("Name: {}", view.display_name.as_deref().unwrap_or("(none)"));
  println!("UID: {}", info.uid);
  println!("GID: {}", info.gid);
  println!("Home: {}", info.home.display());
  println!("Shell: {}", info.shell);
  println!("Groups: {}", join_or_none(&view.groups));
  let avatar_line = view
    .avatar
    .as_deref()
    .map(|path| path.display().to_string())
    .unwrap_or_else(|| "(none)".to_string());
  println!("Avatar: {avatar_line}");
  Ok(())
}

fn cmd_avatar(
  manager: &AccountManager,
  user: &str,
  image: Option<&Path>,
  remove: bool,
) -> Result<()> {
  if remove {
    manager.remove_avatar(user)?;
    println!("Avatar removed successfully.");
  } else {
    let image = image.expect("clap guarantees IMAGE when --remove is absent");
    let expanded = expand_tilde(image);
    manager.set_avatar(user, &expanded)?;
    println!("Avatar updated successfully.");
  }
  Ok(())
}

fn cmd_groups(
  manager: &AccountManager,
  user: &str,
  add: &[String],
  remove: &[String],
) -> Result<()> {
  if add.is_empty() && remove.is_empty() {
    let groups = manager.list_groups(user)?;
    println!("{}", join_or_none(&groups));
    return Ok(());
  }

  let remove_set: BTreeSet<&str> = remove.iter().map(String::as_str).collect();
  let overlap: Vec<&str> = add
    .iter()
    .map(String::as_str)
    .filter(|group| remove_set.contains(group))
    .collect();
  if !overlap.is_empty() {
    let names = overlap.join(", ");
    return Err(Error::InvalidOperation(format!(
      "cannot add and remove the same group(s) at once: {names}"
    )));
  }

  // Mutating membership always requires root (gpasswd).
  elevate::ensure(&Requirement::Always)?;

  for group in add {
    manager.add_group(user, group)?;
    println!("Added '{user}' to group '{group}'.");
  }
  for group in remove {
    manager.remove_group(user, group)?;
    println!("Removed '{user}' from group '{group}'.");
  }
  Ok(())
}

/// Upper bound for the password input. Three secrets of at most 511 bytes and
/// their line breaks fit far below this, and the limit keeps the pipe handed to
/// `pkexec` well inside its buffer.
const PASSWD_INPUT_LIMIT: u64 = 4096;

/// Reads the three password lines from standard input, bounded by
/// [`PASSWD_INPUT_LIMIT`].
///
/// Only three lines are read, so the command does not wait for the caller to
/// close standard input. A frontend can write the lines and keep the pipe open.
fn read_passwd_input() -> Result<String> {
  use std::io::{BufRead, Read};
  let mut reader = std::io::BufReader::new(std::io::stdin().lock().take(PASSWD_INPUT_LIMIT));
  let mut input = String::new();
  for _ in 0..3 {
    let read = reader
      .read_line(&mut input)
      .map_err(|err| Error::system(err.to_string()))?;
    if read == 0 {
      break;
    }
  }
  if input.len() as u64 >= PASSWD_INPUT_LIMIT {
    return Err(Error::InvalidOperation(
      "password input too large".to_string(),
    ));
  }
  Ok(input)
}

/// Splits the input into the current, new and confirmation passwords. Exactly
/// three lines are required, so a missing or extra line is a usage error.
fn parse_passwd_input(input: &str) -> Result<(&str, &str, &str)> {
  let mut lines = input.lines();
  match (lines.next(), lines.next(), lines.next(), lines.next()) {
    (Some(old), Some(new), Some(confirm), None) => Ok((old, new, confirm)),
    _ => Err(Error::InvalidOperation(
      "expected the current, new and confirmation passwords on standard input, one per line"
        .to_string(),
    )),
  }
}

fn cmd_passwd(manager: &AccountManager, user: &str) -> Result<()> {
  // The secrets come from standard input and travel to the elevated child
  // through a pipe (see `elevate::ensure_with_input`), never through argv.
  let input = read_passwd_input()?;
  let (old, new, confirm) = parse_passwd_input(&input)?;

  // Cheap local validation first: no prompts or elevation on typos.
  if new != confirm {
    return Err(Error::InvalidOperation(
      "new password and confirmation do not match".to_string(),
    ));
  }
  password::validate_new_password(new)?;

  // Fails early with UserNotFound for unknown targets.
  manager.get_user(user)?;
  let me = AccountManager::current_username()?;
  let is_self = me == *user;

  // Prove knowledge of the current password before asking for elevation,
  // so a typo never triggers an administrator prompt.
  if is_self && passwd::effective_uid() != 0 && !password::verify_own_password(user, old)? {
    return Err(Error::PermissionDenied(
      "current password does not match".to_string(),
    ));
  }

  elevate::ensure_with_input(&Requirement::Always, input.as_bytes())?;
  manager.change_password(user, if is_self { Some(old) } else { None }, new)?;
  println!("Password updated successfully for '{user}'.");
  Ok(())
}

fn cmd_self(manager: &AccountManager, action: Option<&SelfAction>, verbose: bool) -> Result<()> {
  let me = AccountManager::current_username()?;
  if verbose {
    eprintln!("argvus-accounts: self target '{me}'");
  }
  match action {
    None | Some(SelfAction::Show) => cmd_show(manager, &me),
    Some(SelfAction::Groups) => {
      let groups = manager.list_groups(&me)?;
      println!("{}", join_or_none(&groups));
      Ok(())
    }
    Some(SelfAction::Name { name }) => {
      // Same policy as `name USER`: validated first, then elevated —
      // no interactive chfn/PAM prompt.
      metadata::validate_display_name(name)?;
      elevate::ensure(&Requirement::Always)?;
      manager.set_display_name(&me, name)?;
      println!("Display name updated successfully.");
      Ok(())
    }
    Some(SelfAction::Avatar { image, remove }) => {
      cmd_avatar(manager, &me, image.as_deref(), *remove)
    }
  }
}

fn join_or_none(items: &[String]) -> String {
  if items.is_empty() {
    "(none)".to_string()
  } else {
    items.join(", ")
  }
}

fn expand_tilde(path: &Path) -> PathBuf {
  let text = path.to_string_lossy();
  if text == "~" {
    return home_dir().unwrap_or_else(|| path.to_path_buf());
  }
  if let Some(rest) = text.strip_prefix("~/")
    && let Some(home) = home_dir()
  {
    return home.join(rest);
  }
  path.to_path_buf()
}

fn home_dir() -> Option<PathBuf> {
  std::env::var_os("HOME")
    .filter(|value| !value.is_empty())
    .map(PathBuf::from)
    .or_else(|| passwd::current_user().ok().map(|user| user.home))
}
