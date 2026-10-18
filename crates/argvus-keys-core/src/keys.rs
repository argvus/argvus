//! Domain rules of the SSH keys: names, comments, key types, and the parsing of
//! what `ssh-keygen` and `ssh-add` print. Nothing here touches the filesystem or
//! a process, so every rule can be tested directly.

/// The key types ARGVUS can generate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyType {
  Ed25519,
  Rsa,
}

impl KeyType {
  /// Parses the `--type` value. Only the two supported names are accepted.
  pub fn parse(value: &str) -> Option<Self> {
    match value {
      "ed25519" => Some(Self::Ed25519),
      "rsa" => Some(Self::Rsa),
      _ => None,
    }
  }

  /// The name given to `ssh-keygen -t`.
  pub fn as_str(self) -> &'static str {
    match self {
      Self::Ed25519 => "ed25519",
      Self::Rsa => "rsa",
    }
  }
}

/// A key name becomes a file name under `~/.ssh`, so only a conservative set is
/// accepted. A leading dot rules out `.` and `..`, and a `.pub` suffix would make
/// the public file of one key the name of another.
pub fn valid_name(name: &str) -> bool {
  !name.is_empty()
    && !name.starts_with('.')
    && !name.ends_with(".pub")
    && name
      .bytes()
      .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

/// A comment is written into a terminal command line when the passphrase is
/// asked there, so spaces and quotes are refused rather than escaped.
pub fn valid_comment(comment: &str) -> bool {
  !comment.is_empty()
    && comment
      .bytes()
      .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'@' | b'+' | b'-'))
}

/// One key of the list: the public key information and the state of the key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyRow {
  pub name: String,
  pub key_type: String,
  pub fingerprint: String,
  pub comment: String,
  pub passphrase: bool,
  pub loaded: Loaded,
}

/// Whether a key is in the running agent. `Unknown` when the agent cannot be
/// reached at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Loaded {
  Yes,
  No,
  Unknown,
}

impl Loaded {
  pub fn as_str(self) -> &'static str {
    match self {
      Self::Yes => "yes",
      Self::No => "no",
      Self::Unknown => "unknown",
    }
  }
}

impl KeyRow {
  /// The tab separated line of the `list` command.
  pub fn tsv(&self) -> String {
    format!(
      "{}\t{}\t{}\t{}\t{}\t{}",
      self.name,
      self.key_type,
      self.fingerprint,
      self.comment,
      if self.passphrase { "yes" } else { "no" },
      self.loaded.as_str()
    )
  }
}

/// The fields of one `ssh-keygen -lf` line, such as
/// `256 SHA256:abc user@host (ED25519)`: the type is the last field without its
/// parentheses, the fingerprint is the second field, and the comment is every
/// field between them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyInfo {
  pub key_type: String,
  pub fingerprint: String,
  pub comment: String,
}

pub fn parse_key_info(line: &str) -> Option<KeyInfo> {
  let fields: Vec<&str> = line.split_whitespace().collect();
  if fields.len() < 3 {
    return None;
  }
  let key_type = fields[fields.len() - 1]
    .trim_matches(|character| character == '(' || character == ')')
    .to_lowercase();
  let comment = if fields.len() > 3 {
    fields[2..fields.len() - 1].join(" ")
  } else {
    String::new()
  };
  Some(KeyInfo {
    key_type,
    fingerprint: fields[1].to_string(),
    comment,
  })
}

/// The fingerprints of `ssh-add -l` output, one per line; the second field of
/// each line is the fingerprint.
pub fn parse_agent_fingerprints(output: &str) -> Vec<String> {
  output
    .lines()
    .filter_map(|line| line.split_whitespace().nth(1))
    .map(str::to_string)
    .collect()
}

/// Quotes a value as a Lua string literal, for the `hl.dsp.exec_cmd` call that
/// opens a terminal.
pub fn lua_string(value: &str) -> String {
  format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

#[cfg(test)]
mod tests {
  use super::{
    KeyRow, KeyType, Loaded, lua_string, parse_agent_fingerprints, parse_key_info, valid_comment,
    valid_name,
  };

  #[test]
  fn names_refuse_paths_and_public_suffixes() {
    assert!(valid_name("github_ed25519"));
    assert!(!valid_name(""));
    assert!(!valid_name(".hidden"));
    assert!(!valid_name("work.pub"));
    assert!(!valid_name("a/b"));
    assert!(!valid_name("a b"));
  }

  #[test]
  fn comments_refuse_spaces_and_quotes() {
    assert!(valid_comment("user@host"));
    assert!(!valid_comment("two words"));
    assert!(!valid_comment("it's"));
    assert!(!valid_comment(""));
  }

  #[test]
  fn key_types_are_the_two_supported_names() {
    assert_eq!(KeyType::parse("ed25519"), Some(KeyType::Ed25519));
    assert_eq!(KeyType::parse("rsa").map(KeyType::as_str), Some("rsa"));
    assert_eq!(KeyType::parse("dsa"), None);
  }

  #[test]
  fn key_info_reads_the_type_fingerprint_and_comment() {
    let info = parse_key_info("256 SHA256:abc user at host (ED25519)").expect("line");
    assert_eq!(info.key_type, "ed25519");
    assert_eq!(info.fingerprint, "SHA256:abc");
    assert_eq!(info.comment, "user at host");
    let bare = parse_key_info("3072 SHA256:def (RSA)").expect("line");
    assert_eq!(bare.comment, "");
    assert!(parse_key_info("garbage").is_none());
  }

  #[test]
  fn agent_fingerprints_are_the_second_field() {
    let output = "256 SHA256:one user@a (ED25519)\n3072 SHA256:two (RSA)\n";
    assert_eq!(
      parse_agent_fingerprints(output),
      vec!["SHA256:one", "SHA256:two"]
    );
  }

  #[test]
  fn list_line_is_tab_separated_with_the_loaded_state() {
    let row = KeyRow {
      name: "work".into(),
      key_type: "ed25519".into(),
      fingerprint: "SHA256:abc".into(),
      comment: "me@box".into(),
      passphrase: true,
      loaded: Loaded::Unknown,
    };
    assert_eq!(row.tsv(), "work\ted25519\tSHA256:abc\tme@box\tyes\tunknown");
  }

  #[test]
  fn lua_strings_escape_quotes_and_backslashes() {
    assert_eq!(lua_string(r#"a"b\c"#), r#""a\"b\\c""#);
  }
}
