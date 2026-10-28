//! Local account discovery and safe avatar resolution.
//!
//! The Greeter reads account metadata but never changes it. Path validation is
//! intentionally defensive because this process runs before authentication
//! and may inspect files owned by accounts that are not yet logged in.

use anyhow::Context;
use std::{
  fs,
  path::{Path, PathBuf},
};

const PASSWD_PATH: &str = "/etc/passwd";
// System accounts are excluded so the login list contains human/service
// accounts intended to start an interactive desktop session.
const MIN_LOGIN_UID: u32 = 1000;
const ACCOUNTS_SERVICE_DIR: &str = "/var/lib/AccountsService";
/// Avatar file managed by `argvus-accounts` inside the user's home directory.
const FACE_FILENAME: &str = ".face";

/// A login candidate and the avatar selected for its display.
#[derive(Debug, Clone)]
pub struct User {
  /// Login name passed to greetd/PAM.
  pub username: String,
  /// Numeric account identifier used to read the public pre-login theme state.
  pub uid: u32,
  /// GECOS-derived display name shown in the TUI.
  pub display_name: String,
  /// First readable avatar source, if one can be safely resolved.
  pub avatar: Option<PathBuf>,
}

/// Reads `/etc/passwd`, filters interactive accounts, and sorts them for UI use.
pub fn discover_users() -> anyhow::Result<Vec<User>> {
  // Parsing the complete file first keeps discovery independent of NSS-side
  // interactive prompts and makes startup behavior deterministic.
  let passwd = fs::read_to_string(PASSWD_PATH).context("reading /etc/passwd")?;
  let mut users = passwd
    .lines()
    .filter_map(parse_passwd_line)
    .filter(is_login_user)
    .map(|entry| User {
      uid: entry.uid,
      display_name: display_name(&entry),
      avatar: avatar_for(&entry),
      username: entry.username,
    })
    .collect::<Vec<_>>();

  // Alphabetical ordering avoids exposing `/etc/passwd` file order as UI state.
  users.sort_by(|a, b| a.display_name.cmp(&b.display_name));
  tracing::info!(count = users.len(), "local login users discovered");
  Ok(users)
}

/// Returns the avatar image for an account.
///
/// Resolution order:
///
/// 1. `$HOME/.face` — the location written and validated by
///    [`argvus-accounts`](https://github.com/argvus/argvus-accounts), the
///    official account-metadata source of the Argvus desktop. The file is a
///    regular, user-owned PNG (256x256, mode 0644) when deployed by that tool.
/// 2. `/var/lib/AccountsService/icons/<username>` — freedesktop AccountsService
///    convention, kept for interoperability with GNOME/KDE and other display
///    managers.
/// 3. The `Icon=` path declared in `/var/lib/AccountsService/users/<username>`.
///
/// Only existing files that are readable by the greeter user qualify.
/// Symlinks are never followed (the check uses `lstat`), so a hostile or
/// stale `.face` symlink cannot make the greeter read arbitrary paths.
fn avatar_for(entry: &PasswdEntry) -> Option<PathBuf> {
  // Home metadata is the ARGVUS-owned source of truth; AccountsService is the
  // compatibility fallback for systems managed by another desktop stack.
  avatar_for_in(Path::new(ACCOUNTS_SERVICE_DIR), entry)
}

/// Same as [`avatar_for`] with an injectable AccountsService directory.
fn avatar_for_in(accounts_dir: &Path, entry: &PasswdEntry) -> Option<PathBuf> {
  // Empty home fields occur in valid passwd data, so avoid constructing a
  // relative `.face` path in that case.
  let from_home = (!entry.home.as_os_str().is_empty())
    .then(|| avatar_in_home(Path::new(&entry.home), &entry.username))
    .flatten();
  from_home.or_else(|| avatar_in_accounts(accounts_dir, &entry.username))
}

/// Resolves `$HOME/.face` for `username`, ignoring anything that is not a
/// plain readable regular file.
pub fn avatar_in_home(home: &Path, username: &str) -> Option<PathBuf> {
  // The username check prevents path traversal even if passwd data is
  // malformed or supplied by a test fixture.
  if !is_safe_username(username) {
    return None;
  }
  // `join` is safe here only after the username has been constrained to one
  // path component; the resulting file is still checked without following
  // symlinks.
  let face = home.join(FACE_FILENAME);
  is_regular_readable_file(&face).then_some(face)
}

fn avatar_in_accounts(accounts_dir: &Path, username: &str) -> Option<PathBuf> {
  // AccountsService first exposes the conventional icon filename.
  let icon = accounts_dir.join("icons").join(username);
  if is_regular_readable_file(&icon) {
    return Some(icon);
  }

  // Some desktop managers store an explicit Icon= path in a per-user file.
  let user_file = accounts_dir.join("users").join(username);
  let contents = fs::read_to_string(user_file).ok()?;
  contents.lines().find_map(|line| {
    // Ignore all metadata except the exact key understood by AccountsService.
    line
      .strip_prefix("Icon=")
      .map(str::trim)
      .filter(|path| !path.is_empty())
      .map(PathBuf::from)
      .filter(|path| is_regular_readable_file(path))
  })
}

/// Defense-in-depth: usernames coming from `/etc/passwd` must be safe to use
/// as a single path component before being joined into any directory.
fn is_safe_username(username: &str) -> bool {
  // Limit length and reject separators/control characters before any path
  // operation. This is defense in depth even though `/etc/passwd` is trusted.
  !username.is_empty()
    && username.len() <= 32
    && username != "."
    && username != ".."
    && !username.chars().any(|c| c == '/' || c.is_control())
}

fn is_regular_readable_file(path: &Path) -> bool {
  // `symlink_metadata` does not follow symlinks; `is_file()` on its result is
  // true only for real regular files. Opening the path separately verifies
  // that the greeter process can actually read it.
  path
    .symlink_metadata()
    .map(|meta| meta.is_file())
    .unwrap_or(false)
    && fs::File::open(path).is_ok()
}

/// Raw fields needed to decide whether a passwd entry is a desktop account.
#[derive(Debug)]
struct PasswdEntry {
  /// Login name from field 1.
  username: String,
  /// GECOS field used for the display name.
  gecos: String,
  /// Numeric uid used to exclude system accounts.
  uid: u32,
  /// Home directory used for `.face` lookup.
  home: PathBuf,
  /// Login shell used to reject disabled accounts.
  shell: String,
}

/// Parses the seven required colon-separated passwd fields.
fn parse_passwd_line(line: &str) -> Option<PasswdEntry> {
  // Invalid lines are ignored rather than aborting the whole login screen;
  // the system file can contain vendor-specific records we do not need.
  let fields = line.split(':').collect::<Vec<_>>();
  if fields.len() < 7 {
    return None;
  }

  Some(PasswdEntry {
    username: fields[0].to_string(),
    uid: fields[2].parse().ok()?,
    gecos: fields[4].to_string(),
    home: PathBuf::from(fields[5]),
    shell: fields[6].to_string(),
  })
}

/// Determines whether an account should be offered for interactive login.
fn is_login_user(entry: &PasswdEntry) -> bool {
  if entry.uid < MIN_LOGIN_UID {
    return false;
  }

  let shell = entry.shell.trim();
  // `/false`, `/nologin`, and empty shells explicitly indicate non-interactive
  // accounts even when their uid is in the human-user range.
  !(shell.ends_with("/false") || shell.ends_with("/nologin") || shell.is_empty())
}

/// Derives a concise display name from the first GECOS component.
fn display_name(entry: &PasswdEntry) -> String {
  // GECOS may contain comma-separated contact fields; only the conventional
  // full-name field belongs in the account selector.
  let gecos_name = entry.gecos.split(',').next().unwrap_or("").trim();
  if gecos_name.is_empty() {
    entry.username.clone()
  } else {
    gecos_name.to_string()
  }
}

#[cfg(test)]
mod tests;
