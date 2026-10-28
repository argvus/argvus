//! Reads and edits the unofficial repositories of `/etc/pacman.conf`.
//!
//! A repository is either a section written in `pacman.conf` itself
//! (`[name]` followed by `Server = …`) or a file of `/etc/pacman.d/` that
//! `pacman.conf` includes (`Include = /etc/pacman.d/name.conf`), which holds
//! its own `[name]` section. The Control Center lists repositories as the
//! user and edits them as root (`system-settings package repo-*`); both
//! sides use these functions so they agree on what a repository is.
//!
//! Every function is pure: callers read and write the files, with a backup
//! and an atomic replace on the root side.

use std::fmt;
use std::ops::Range;
use std::path::{Path, PathBuf};

/// The pacman configuration that lists the repositories.
pub const PACMAN_CONF: &str = "/etc/pacman.conf";
/// The directory of the repository files that `pacman.conf` includes.
pub const PACMAN_D: &str = "/etc/pacman.d";
/// Comment recording the fingerprint of the key imported with a repository,
/// so removing the repository can offer to delete that key too.
pub const KEY_MARKER: &str = "# argvus-key:";
/// Prefix of every line of an inline section the Control Center disabled.
/// A plain `#` would be confused with the commented examples of the default
/// `pacman.conf` (`#[custom]`, `#[core-testing]`).
pub const DISABLED_PREFIX: &str = "#argvus-off ";

/// Sections of the official repositories and the `[options]` section, which
/// are never listed or edited here.
const OFFICIAL_SECTIONS: &[&str] = &[
  "options",
  "core",
  "extra",
  "multilib",
  "core-testing",
  "extra-testing",
  "multilib-testing",
  "gnome-unstable",
  "kde-unstable",
  "testing",
  "community",
  "community-testing",
];

/// Longest repository name accepted; it also names the included file.
const MAX_NAME_LENGTH: usize = 64;

/// Trust values of `SigLevel`, each optionally prefixed by `Package` or
/// `Database` (see pacman.conf(5)).
const SIG_LEVEL_VALUES: &[&str] = &["Never", "Optional", "Required", "TrustedOnly", "TrustAll"];

/// Values of `Usage` (see pacman.conf(5)).
const USAGE_VALUES: &[&str] = &["Sync", "Search", "Install", "Upgrade", "All"];

/// Where a repository is written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepositoryOrigin {
  /// A section written in `pacman.conf` itself.
  Inline,
  /// A file of `/etc/pacman.d/` that `pacman.conf` includes.
  Include(PathBuf),
}

/// An unofficial repository configured in `pacman.conf`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Repository {
  pub name: String,
  /// `Server` addresses, and `Include` paths of mirror lists, in order.
  pub servers: Vec<String>,
  pub sig_level: Option<String>,
  pub origin: RepositoryOrigin,
  pub enabled: bool,
  /// Fingerprint of the signing key recorded when the Control Center added
  /// the repository ([`KEY_MARKER`]).
  pub key: Option<String>,
}

/// One section of a repository configuration being added.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RepositorySection {
  pub name: String,
  pub servers: Vec<String>,
  pub sig_level: Option<String>,
}

/// Why a repository configuration is refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfError {
  /// No `[name]` section.
  Empty,
  InvalidName(String),
  /// The section is an official repository or `[options]`.
  Official(String),
  Duplicate(String),
  /// A directive before the first section.
  OutsideSection(String),
  UnsupportedDirective(String),
  InvalidServer(String),
  InvalidSigLevel(String),
  InvalidUsage(String),
  /// `Include` outside `/etc/pacman.d/`.
  InvalidInclude(String),
  /// A section without `Server` or `Include`.
  NoServer(String),
}

impl fmt::Display for ConfError {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Empty => write!(formatter, "no [repository] section"),
      Self::InvalidName(name) => write!(formatter, "invalid repository name: {name}"),
      Self::Official(name) => write!(formatter, "[{name}] is an official section"),
      Self::Duplicate(name) => write!(formatter, "[{name}] appears twice"),
      Self::OutsideSection(line) => write!(formatter, "directive outside a section: {line}"),
      Self::UnsupportedDirective(key) => write!(formatter, "unsupported directive: {key}"),
      Self::InvalidServer(server) => write!(formatter, "invalid server: {server}"),
      Self::InvalidSigLevel(value) => write!(formatter, "invalid SigLevel: {value}"),
      Self::InvalidUsage(value) => write!(formatter, "invalid Usage: {value}"),
      Self::InvalidInclude(path) => write!(formatter, "Include must be in {PACMAN_D}: {path}"),
      Self::NoServer(name) => write!(formatter, "[{name}] has no Server"),
    }
  }
}

/// Whether `name` is an official repository or `[options]`.
pub fn is_official(name: &str) -> bool {
  OFFICIAL_SECTIONS.contains(&name)
}

/// Whether `name` can name a repository and its file in `/etc/pacman.d/`.
pub fn is_valid_name(name: &str) -> bool {
  !name.is_empty()
    && name.len() <= MAX_NAME_LENGTH
    && !name.starts_with(['-', '.'])
    && name
      .chars()
      .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.'))
}

/// Whether `value` is an OpenPGP v4 fingerprint (40 hexadecimal digits).
pub fn is_fingerprint(value: &str) -> bool {
  value.len() == 40 && value.chars().all(|character| character.is_ascii_hexdigit())
}

/// The file of `/etc/pacman.d/` that holds the repository `name`.
pub fn repository_file(name: &str) -> PathBuf {
  Path::new(PACMAN_D).join(format!("{name}.conf"))
}

/// A configuration line, read without its surrounding spaces.
enum Line<'a> {
  Section(&'a str),
  Directive(&'a str, &'a str),
  /// Blank lines, comments and bare options such as `Color`.
  Other,
}

fn classify(line: &str) -> Line<'_> {
  let line = line.trim();
  if line.starts_with('#') {
    return Line::Other;
  }
  if let Some(name) = line
    .strip_prefix('[')
    .and_then(|rest| rest.strip_suffix(']'))
  {
    return Line::Section(name.trim());
  }
  match line.split_once('=') {
    Some((key, value)) => Line::Directive(key.trim(), value.trim()),
    None => Line::Other,
  }
}

/// The path of an `Include` that brings a whole repository file: a `.conf`
/// in `/etc/pacman.d/`. Other includes, such as mirror lists, belong to the
/// section above them.
fn repository_include(line: &str) -> Option<&str> {
  match classify(line) {
    Line::Directive("Include", path) if is_repository_file_path(path) => Some(path),
    _ => None,
  }
}

/// The path of a commented repository include (`# Include = /etc/pacman.d/x.conf`).
fn commented_repository_include(line: &str) -> Option<&str> {
  let uncommented = line.trim().strip_prefix('#')?.trim_start_matches('#');
  repository_include(uncommented)
}

fn is_repository_file_path(path: &str) -> bool {
  path.starts_with(PACMAN_D) && path.ends_with(".conf") && !path.contains("..")
}

/// The line without the prefix of a disabled inline section, and whether it
/// had it.
fn strip_disabled(line: &str) -> (&str, bool) {
  match line.strip_prefix(DISABLED_PREFIX) {
    Some(rest) => (rest, true),
    None => (line, false),
  }
}

/// The repository sections of an included file, read leniently (the file is
/// already in use), with the key recorded in it.
fn file_repositories(path: &Path, text: &str, enabled: bool) -> Vec<Repository> {
  let key = text.lines().find_map(|line| {
    line
      .trim()
      .strip_prefix(KEY_MARKER)
      .map(|fingerprint| fingerprint.trim().to_owned())
      .filter(|fingerprint| is_fingerprint(fingerprint))
  });
  let mut repositories: Vec<Repository> = Vec::new();
  for line in text.lines() {
    match classify(line) {
      Line::Section(name) if !is_official(name) => repositories.push(Repository {
        name: name.to_owned(),
        servers: Vec::new(),
        sig_level: None,
        origin: RepositoryOrigin::Include(path.to_path_buf()),
        enabled,
        key: key.clone(),
      }),
      Line::Directive(directive, value) => {
        if let Some(repository) = repositories.last_mut() {
          record_directive(repository, directive, value);
        }
      }
      _ => {}
    }
  }
  repositories
}

fn record_directive(repository: &mut Repository, directive: &str, value: &str) {
  match directive {
    "Server" | "Include" => repository.servers.push(value.to_owned()),
    "SigLevel" => repository.sig_level = Some(value.to_owned()),
    _ => {}
  }
}

/// The unofficial repositories of `pacman_conf`, in file order, enabled and
/// disabled. `read_file` reads an included repository file; an active
/// include whose file cannot be read is still listed (by its file name) so
/// it can be removed.
pub fn list_repositories(
  pacman_conf: &str,
  read_file: impl Fn(&Path) -> Option<String>,
) -> Vec<Repository> {
  let mut repositories: Vec<Repository> = Vec::new();
  // The inline repository whose directives the following lines fill.
  let mut current: Option<usize> = None;
  for raw_line in pacman_conf.lines() {
    if let Some(path) = commented_repository_include(raw_line) {
      // A disabled include is listed only when its file still exists:
      // otherwise there is nothing to enable.
      if let Some(text) = read_file(Path::new(path)) {
        repositories.extend(file_repositories(Path::new(path), &text, false));
      }
      current = None;
      continue;
    }
    let (line, disabled) = strip_disabled(raw_line);
    if let Some(path) = repository_include(line) {
      let path = Path::new(path);
      match read_file(path) {
        Some(text) => repositories.extend(file_repositories(path, &text, !disabled)),
        None => repositories.push(Repository {
          name: path
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
            .unwrap_or_default(),
          servers: Vec::new(),
          sig_level: None,
          origin: RepositoryOrigin::Include(path.to_path_buf()),
          enabled: !disabled,
          key: None,
        }),
      }
      current = None;
      continue;
    }
    match classify(line) {
      Line::Section(name) => {
        current = (!is_official(name)).then(|| {
          repositories.push(Repository {
            name: name.to_owned(),
            servers: Vec::new(),
            sig_level: None,
            origin: RepositoryOrigin::Inline,
            enabled: !disabled,
            key: None,
          });
          repositories.len() - 1
        });
      }
      Line::Directive(directive, value) => {
        // Only lines in the same state as their header belong to it.
        if let Some(repository) = current.and_then(|index| repositories.get_mut(index))
          && repository.enabled != disabled
        {
          record_directive(repository, directive, value);
        }
      }
      Line::Other => {}
    }
  }
  repositories
}

/// Checks a repository configuration being added and returns its sections.
///
/// # Errors
///
/// Returns the first problem found: no section, an official or invalid
/// name, a directive pacman would not accept in a repository, a server that
/// is not `https://`, `http://` or `file://`, or an `Include` outside
/// `/etc/pacman.d/`.
pub fn parse_repository_conf(text: &str) -> Result<Vec<RepositorySection>, ConfError> {
  let mut sections: Vec<RepositorySection> = Vec::new();
  for line in text.lines() {
    match classify(line) {
      Line::Section(name) => {
        if !is_valid_name(name) {
          return Err(ConfError::InvalidName(name.to_owned()));
        }
        if is_official(name) {
          return Err(ConfError::Official(name.to_owned()));
        }
        if sections.iter().any(|section| section.name == name) {
          return Err(ConfError::Duplicate(name.to_owned()));
        }
        sections.push(RepositorySection {
          name: name.to_owned(),
          ..RepositorySection::default()
        });
      }
      Line::Directive(directive, value) => {
        let Some(section) = sections.last_mut() else {
          return Err(ConfError::OutsideSection(line.trim().to_owned()));
        };
        parse_directive(section, directive, value)?;
      }
      Line::Other => {
        let trimmed = line.trim();
        if !trimmed.is_empty() && !trimmed.starts_with('#') {
          return Err(ConfError::UnsupportedDirective(trimmed.to_owned()));
        }
      }
    }
  }
  if sections.is_empty() {
    return Err(ConfError::Empty);
  }
  if let Some(section) = sections.iter().find(|section| section.servers.is_empty()) {
    return Err(ConfError::NoServer(section.name.clone()));
  }
  Ok(sections)
}

fn parse_directive(
  section: &mut RepositorySection,
  directive: &str,
  value: &str,
) -> Result<(), ConfError> {
  match directive {
    "Server" | "CacheServer" => {
      let has_scheme = ["https://", "http://", "file://"]
        .iter()
        .any(|scheme| value.starts_with(scheme));
      if !has_scheme || value.chars().any(char::is_whitespace) {
        return Err(ConfError::InvalidServer(value.to_owned()));
      }
      if directive == "Server" {
        section.servers.push(value.to_owned());
      }
    }
    "Include" => {
      if !value.starts_with(PACMAN_D) || value.contains("..") {
        return Err(ConfError::InvalidInclude(value.to_owned()));
      }
      section.servers.push(value.to_owned());
    }
    "SigLevel" => {
      let valid = !value.is_empty()
        && value.split_whitespace().all(|level| {
          let trust = level
            .strip_prefix("Package")
            .or_else(|| level.strip_prefix("Database"))
            .unwrap_or(level);
          SIG_LEVEL_VALUES.contains(&trust)
        });
      if !valid {
        return Err(ConfError::InvalidSigLevel(value.to_owned()));
      }
      section.sig_level = Some(value.to_owned());
    }
    "Usage" => {
      if value.is_empty()
        || !value
          .split_whitespace()
          .all(|usage| USAGE_VALUES.contains(&usage))
      {
        return Err(ConfError::InvalidUsage(value.to_owned()));
      }
    }
    other => return Err(ConfError::UnsupportedDirective(other.to_owned())),
  }
  Ok(())
}

/// The configuration of a repository built from the form fields, in the
/// layout of the Arch Wiki. `sig_level` `None` keeps the global default.
pub fn form_conf(name: &str, server: &str, sig_level: Option<&str>) -> String {
  let mut conf = format!("[{name}]\n");
  if let Some(sig_level) = sig_level {
    conf.push_str(&format!("SigLevel = {sig_level}\n"));
  }
  conf.push_str(&format!("Server = {server}\n"));
  conf
}

/// `conf` with the fingerprint of its signing key recorded on top.
pub fn with_key_marker(conf: &str, fingerprint: &str) -> String {
  format!("{KEY_MARKER} {fingerprint}\n{conf}")
}

/// Rebuilds a file from its lines, keeping its final newline.
fn join_lines(lines: &[String], original: &str) -> String {
  let mut text = lines.join("\n");
  if original.ends_with('\n') || original.is_empty() {
    text.push('\n');
  }
  text
}

fn owned_lines(text: &str) -> Vec<String> {
  text.lines().map(str::to_owned).collect()
}

/// `pacman_conf` including the repository file `path`: a commented include
/// of the same file is enabled again, otherwise the line is appended.
pub fn add_include(pacman_conf: &str, path: &Path) -> String {
  let include = format!("Include = {}", path.display());
  let mut lines = owned_lines(pacman_conf);
  if let Some(line) = lines
    .iter_mut()
    .find(|line| commented_repository_include(line).is_some_and(|found| Path::new(found) == path))
  {
    *line = include;
  } else {
    if lines.last().is_some_and(|line| !line.trim().is_empty()) {
      lines.push(String::new());
    }
    lines.push(include);
  }
  join_lines(&lines, pacman_conf)
}

/// Whether `line` includes `path`, active or commented.
fn includes_path(line: &str, path: &Path) -> bool {
  repository_include(line)
    .or_else(|| commented_repository_include(line))
    .is_some_and(|found| Path::new(found) == path)
}

/// `pacman_conf` with the include of `path` enabled or commented out.
///
/// # Errors
///
/// Returns an error when `pacman_conf` does not include `path`.
pub fn set_include_enabled(
  pacman_conf: &str,
  path: &Path,
  enabled: bool,
) -> Result<String, String> {
  let mut lines = owned_lines(pacman_conf);
  let mut found = false;
  for line in lines.iter_mut().filter(|line| includes_path(line, path)) {
    found = true;
    let include = format!("Include = {}", path.display());
    *line = if enabled {
      include
    } else {
      format!("# {include}")
    };
  }
  found
    .then(|| join_lines(&lines, pacman_conf))
    .ok_or_else(|| format!("{} is not included in {PACMAN_CONF}", path.display()))
}

/// `pacman_conf` without the include of `path`, active or commented.
///
/// # Errors
///
/// Returns an error when `pacman_conf` does not include `path`.
pub fn remove_include(pacman_conf: &str, path: &Path) -> Result<String, String> {
  let lines = owned_lines(pacman_conf);
  let kept: Vec<String> = lines
    .iter()
    .filter(|line| !includes_path(line, path))
    .cloned()
    .collect();
  if kept.len() == lines.len() {
    return Err(format!(
      "{} is not included in {PACMAN_CONF}",
      path.display()
    ));
  }
  Ok(join_lines(&kept, pacman_conf))
}

/// The lines of the inline section `name`: its header and the directives
/// under it, up to the next section, repository include or comment.
fn inline_block(lines: &[String], name: &str) -> Option<Range<usize>> {
  let start = lines.iter().position(|line| {
    let (line, _) = strip_disabled(line);
    matches!(classify(line), Line::Section(found) if found == name)
  })?;
  let mut end = start + 1;
  for (index, raw_line) in lines.iter().enumerate().skip(start + 1) {
    let (line, _) = strip_disabled(raw_line);
    if line.trim().is_empty() {
      continue;
    }
    if repository_include(line).is_some() {
      break;
    }
    match classify(line) {
      Line::Directive(..) => end = index + 1,
      Line::Section(_) | Line::Other => break,
    }
  }
  Some(start..end)
}

/// `pacman_conf` with the inline section `name` enabled, or disabled by
/// prefixing each of its lines with [`DISABLED_PREFIX`].
///
/// # Errors
///
/// Returns an error when there is no inline section `name`.
pub fn set_inline_enabled(pacman_conf: &str, name: &str, enabled: bool) -> Result<String, String> {
  if is_official(name) {
    return Err(format!("[{name}] is an official section"));
  }
  let mut lines = owned_lines(pacman_conf);
  let block =
    inline_block(&lines, name).ok_or_else(|| format!("[{name}] is not in {PACMAN_CONF}"))?;
  for line in &mut lines[block] {
    let (content, _) = strip_disabled(line);
    let content = content.to_owned();
    *line = if enabled || content.trim().is_empty() {
      content
    } else {
      format!("{DISABLED_PREFIX}{content}")
    };
  }
  Ok(join_lines(&lines, pacman_conf))
}

/// `pacman_conf` without the inline section `name`, enabled or disabled.
///
/// # Errors
///
/// Returns an error when there is no inline section `name`.
pub fn remove_inline(pacman_conf: &str, name: &str) -> Result<String, String> {
  if is_official(name) {
    return Err(format!("[{name}] is an official section"));
  }
  let mut lines = owned_lines(pacman_conf);
  let mut block =
    inline_block(&lines, name).ok_or_else(|| format!("[{name}] is not in {PACMAN_CONF}"))?;
  // Drop the blank line after the section too, so no double gap is left.
  let blank_before = block.start == 0 || lines[block.start - 1].trim().is_empty();
  if blank_before
    && lines
      .get(block.end)
      .is_some_and(|line| line.trim().is_empty())
  {
    block.end += 1;
  }
  lines.drain(block);
  Ok(join_lines(&lines, pacman_conf))
}

/// Largest signing key accepted, in bytes; real public keys are a few KiB.
pub const MAX_KEY_BYTES: usize = 64 * 1024;

/// A public key read from `gpg --show-keys --with-colons`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyInfo {
  pub fingerprint: String,
  /// The first user ID, such as `ARGVUS <packages@example.org>`.
  pub user_id: Option<String>,
}

/// The primary keys of a `gpg --with-colons` listing, in order. Subkey
/// fingerprints (after `sub:`) are skipped.
pub fn parse_key_listing(listing: &str) -> Vec<KeyInfo> {
  let mut keys: Vec<KeyInfo> = Vec::new();
  // The next `fpr:` record belongs to a primary key.
  let mut primary_pending = false;
  for record in listing.lines() {
    let fields: Vec<&str> = record.split(':').collect();
    match fields.first().copied() {
      Some("pub") => primary_pending = true,
      Some("sub") => primary_pending = false,
      Some("fpr") if primary_pending => {
        primary_pending = false;
        if let Some(fingerprint) = fields.get(9).filter(|value| is_fingerprint(value)) {
          keys.push(KeyInfo {
            fingerprint: fingerprint.to_ascii_uppercase(),
            user_id: None,
          });
        }
      }
      Some("uid") => {
        if let Some(key) = keys.last_mut()
          && key.user_id.is_none()
        {
          key.user_id = fields
            .get(9)
            .map(|user_id| user_id.replace("\\x3a", ":"))
            .filter(|user_id| !user_id.is_empty());
        }
      }
      _ => {}
    }
  }
  keys
}

/// `key` as hexadecimal text: a downloaded key is often binary, and a
/// privileged argument must be printable.
pub fn encode_key(key: &[u8]) -> String {
  key.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// The key bytes of [`encode_key`] text; `None` when the text is not
/// hexadecimal, is empty or exceeds [`MAX_KEY_BYTES`].
pub fn decode_key(text: &str) -> Option<Vec<u8>> {
  if text.is_empty() || !text.len().is_multiple_of(2) || text.len() / 2 > MAX_KEY_BYTES {
    return None;
  }
  (0..text.len())
    .step_by(2)
    .map(|index| u8::from_str_radix(text.get(index..index + 2)?, 16).ok())
    .collect()
}

#[cfg(test)]
mod tests;
