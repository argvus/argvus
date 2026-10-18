//! Rules of the GPG keys: what a user ID, a key id, an algorithm, a size and an
//! expiry may be, the parsing of gpg's colon listing, the expiry date of a
//! timestamp, and the ARGVUS block of `gpg-agent.conf`. Nothing here runs gpg.

use std::path::{Path, PathBuf};

/// The version of the command set. The Control Center refuses GPG actions when
/// the installed value is lower than the one it needs, so it is raised whenever
/// a subcommand or an option is added or changed.
pub const GPG_API: u32 = 2;

/// The one pinentry ARGVUS uses. It is not a user choice.
pub const PINENTRY_BIN: &str = "pinentry-gtk";

/// The lines of the ARGVUS block in `gpg-agent.conf`. Everything outside the
/// block is the user's and is kept as it is.
pub const MARK_BEGIN: &str = "# BEGIN argvus-managed pinentry";
pub const MARK_END: &str = "# END argvus-managed pinentry";

/// A user ID is free text: only the length, control characters and a leading
/// dash (which gpg would read as an option) are refused.
pub fn valid_uid(uid: &str) -> bool {
  !uid.is_empty()
    && !uid.starts_with('-')
    && uid.len() <= 200
    && !uid.chars().any(|character| character.is_control())
}

/// A key id, fingerprint or mail address, with nothing that could be read as a
/// path or an option.
pub fn valid_id(id: &str) -> bool {
  !id.is_empty()
    && !id.starts_with('-')
    && id
      .bytes()
      .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'@' | b'.' | b'_' | b'+' | b'-'))
}

/// A full fingerprint: 40 upper-case hex digits. Secret-key commands take only
/// this form, so a short id can never select another key.
pub fn valid_fingerprint(fingerprint: &str) -> bool {
  fingerprint.len() == 40
    && fingerprint
      .bytes()
      .all(|byte| byte.is_ascii_digit() || (b'A'..=b'F').contains(&byte))
}

pub fn valid_algo(algo: &str) -> bool {
  matches!(algo, "rsa" | "dsa")
}

/// DSA keys are limited to 3072 bits by gpg, so 4096 is refused here with a
/// clear message instead of a generic gpg failure.
pub fn valid_key_bits(algo: &str, bits: &str) -> bool {
  matches!(
    (algo, bits),
    ("rsa", "1024" | "2048" | "3072" | "4096") | ("dsa", "1024" | "2048" | "3072")
  )
}

/// The size used when `--bits` is not given: RSA gets 4096 and DSA gets 2048,
/// the sizes gpg recommends.
pub fn default_bits(algo: &str) -> Option<&'static str> {
  match algo {
    "rsa" => Some("4096"),
    "dsa" => Some("2048"),
    _ => None,
  }
}

/// `0` for no expiry, a number followed by `d`, `w`, `m` or `y`, or an ISO date.
pub fn valid_expire(expire: &str) -> bool {
  if expire == "0" {
    return true;
  }
  if let Some(unit) = expire.chars().last()
    && matches!(unit, 'd' | 'w' | 'm' | 'y')
  {
    let number = &expire[..expire.len() - 1];
    return !number.is_empty() && number.bytes().all(|byte| byte.is_ascii_digit());
  }
  let parts: Vec<&str> = expire.split('-').collect();
  parts.len() == 3
    && parts[0].len() == 4
    && parts[1].len() == 2
    && parts[2].len() == 2
    && expire.bytes().filter(|byte| *byte == b'-').count() == 2
    && expire
      .bytes()
      .all(|byte| byte.is_ascii_digit() || byte == b'-')
}

/// The name gpg's numeric algorithm id stands for in the listing.
pub fn algo_name(id: &str) -> &'static str {
  match id {
    "1" => "rsa",
    "16" => "elgamal",
    "17" => "dsa",
    "18" => "ecc",
    "19" => "ecdsa",
    "22" => "ed25519",
    _ => "other",
  }
}

/// The `subkey` id of the encryption subkey for a primary key.
pub fn encryption_subkey_algo(algo: &str, bits: &str) -> Option<String> {
  match algo {
    "rsa" => Some(format!("rsa{bits}")),
    "dsa" => Some(format!("elg{bits}")),
    _ => None,
  }
}

/// One primary key of the listing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyRow {
  pub fingerprint: String,
  pub keyid: String,
  pub key_type: String,
  pub expires: String,
  pub secret: bool,
  pub validity: String,
  pub uid: String,
}

impl KeyRow {
  pub fn tsv(&self) -> String {
    format!(
      "{}\t{}\t{}\t{}\t{}\t{}\t{}",
      self.fingerprint,
      self.keyid,
      self.key_type,
      self.expires,
      if self.secret { "yes" } else { "no" },
      self.validity,
      self.uid
    )
  }
}

/// Parses gpg's `--with-colons --list-keys` output, joined with the fingerprints
/// of the secret keys. Only the primary key of each block is a row; its subkeys
/// are skipped, and the expiry is `never` or the date of the timestamp.
pub fn parse_key_listing(listing: &str, secret_fingerprints: &[String]) -> Vec<KeyRow> {
  let mut rows = Vec::new();
  let mut current: Option<KeyRow> = None;
  let mut in_primary = false;
  for line in listing.lines() {
    let fields: Vec<&str> = line.split(':').collect();
    match fields.first().copied() {
      Some("pub") => {
        rows.extend(current.take());
        let expires = match fields.get(6).and_then(|value| value.parse::<i64>().ok()) {
          Some(timestamp) => date_of(timestamp),
          None => "never".to_string(),
        };
        current = Some(KeyRow {
          fingerprint: String::new(),
          keyid: field(&fields, 4),
          key_type: algo_name(&field(&fields, 3)).to_string(),
          expires,
          secret: false,
          validity: field(&fields, 1),
          uid: String::new(),
        });
        in_primary = true;
      }
      Some("sub") => in_primary = false,
      Some("fpr") if in_primary => {
        if let Some(row) = current.as_mut()
          && row.fingerprint.is_empty()
        {
          row.fingerprint = field(&fields, 9);
        }
      }
      Some("uid") => {
        if let Some(row) = current.as_mut()
          && row.uid.is_empty()
        {
          row.uid = field(&fields, 9).replace("\\x3a", ":").replace('\t', " ");
        }
      }
      _ => {}
    }
  }
  rows.extend(current);
  for row in &mut rows {
    row.secret = secret_fingerprints.contains(&row.fingerprint);
  }
  rows.retain(|row| !row.fingerprint.is_empty());
  rows
}

/// The fingerprints of the secret keys, from the secret listing.
pub fn parse_fingerprints(listing: &str) -> Vec<String> {
  listing
    .lines()
    .filter(|line| line.starts_with("fpr:"))
    .map(|line| field(&line.split(':').collect::<Vec<_>>(), 9))
    .collect()
}

/// The subkey fingerprints of a key listing: a `fpr` line after a `sub` line.
pub fn parse_subkey_fingerprints(listing: &str) -> Vec<String> {
  let mut subkeys = Vec::new();
  let mut in_sub = false;
  for line in listing.lines() {
    let fields: Vec<&str> = line.split(':').collect();
    match fields.first().copied() {
      Some("pub") => in_sub = false,
      Some("sub") => in_sub = true,
      Some("fpr") if in_sub => {
        subkeys.push(field(&fields, 9));
        in_sub = false;
      }
      _ => {}
    }
  }
  subkeys
}

/// The fingerprint of the key that gpg created, from `--status-fd` output.
pub fn created_fingerprint(status: &str) -> Option<String> {
  status
    .lines()
    .find(|line| line.split_whitespace().nth(1) == Some("KEY_CREATED"))
    .and_then(|line| line.split_whitespace().nth(3))
    .map(str::to_string)
}

fn field(fields: &[&str], index: usize) -> String {
  fields.get(index).copied().unwrap_or("").to_string()
}

/// The UTC date `YYYY-MM-DD` of a Unix timestamp.
pub fn date_of(timestamp: i64) -> String {
  // Civil date from days since 1970-01-01 (proleptic Gregorian calendar).
  let days = timestamp.div_euclid(86_400) + 719_468;
  let era = days.div_euclid(146_097);
  let day_of_era = days.rem_euclid(146_097);
  let year_of_era =
    (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
  let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
  let month_index = (5 * day_of_year + 2) / 153;
  let day = day_of_year - (153 * month_index + 2) / 5 + 1;
  let month = if month_index < 10 {
    month_index + 3
  } else {
    month_index - 9
  };
  let year = year_of_era + era * 400 + i64::from(month <= 2);
  format!("{year:04}-{month:02}-{day:02}")
}

/// The pinentry program of the ARGVUS block, if the block has one.
pub fn managed_pinentry(conf: &str) -> Option<String> {
  let mut inside = false;
  for line in conf.lines() {
    if line == MARK_BEGIN {
      inside = true;
    } else if line == MARK_END {
      inside = false;
    } else if inside && let Some(program) = pinentry_value(line, false) {
      return Some(program);
    }
  }
  None
}

/// A `pinentry-program` line the user wrote outside the ARGVUS block.
pub fn user_pinentry(conf: &str) -> Option<String> {
  let mut inside = false;
  for line in conf.lines() {
    if line == MARK_BEGIN {
      inside = true;
    } else if line == MARK_END {
      inside = false;
    } else if !inside && let Some(program) = pinentry_value(line, true) {
      return Some(program);
    }
  }
  None
}

/// The value of a `pinentry-program` line. A user line may be indented; the
/// managed line is written without indentation.
fn pinentry_value(line: &str, allow_indent: bool) -> Option<String> {
  let trimmed = if allow_indent {
    line.trim_start()
  } else {
    line
  };
  let rest = trimmed.strip_prefix("pinentry-program")?;
  if !rest.starts_with([' ', '\t']) {
    return None;
  }
  Some(rest.split_whitespace().next().unwrap_or("").to_string())
}

/// `conf` without its ARGVUS block, then the block with `program`.
pub fn with_managed_pinentry(conf: &str, program: &str) -> String {
  let mut kept = String::new();
  let mut inside = false;
  for line in conf.lines() {
    if line == MARK_BEGIN {
      inside = true;
    } else if line == MARK_END {
      inside = false;
    } else if !inside {
      kept.push_str(line);
      kept.push('\n');
    }
  }
  kept.push_str(&format!(
    "{MARK_BEGIN}\npinentry-program {program}\n{MARK_END}\n"
  ));
  kept
}

/// Where the agent configuration lives.
pub fn agent_conf(gnupg_home: &Path) -> PathBuf {
  gnupg_home.join("gpg-agent.conf")
}

/// The message key and its name for a revocation reason.
pub fn revocation_code(reason: &str) -> Option<&'static str> {
  match reason {
    "none" => Some("0"),
    "compromised" => Some("1"),
    "superseded" => Some("2"),
    "unused" => Some("3"),
    _ => None,
  }
}

/// The gpg trust value of a trust level.
pub fn trust_value(level: &str) -> Option<&'static str> {
  match level {
    "none" => Some("2"),
    "marginal" => Some("3"),
    "full" => Some("4"),
    "ultimate" => Some("5"),
    _ => None,
  }
}

#[cfg(test)]
mod tests {
  use super::{
    KeyRow, MARK_BEGIN, MARK_END, created_fingerprint, date_of, managed_pinentry,
    parse_key_listing, parse_subkey_fingerprints, revocation_code, trust_value, user_pinentry,
    valid_expire, valid_fingerprint, valid_key_bits, valid_uid, with_managed_pinentry,
  };

  #[test]
  fn expiry_accepts_days_weeks_months_years_dates_and_zero() {
    for ok in ["0", "2y", "10d", "3w", "6m", "2030-01-01"] {
      assert!(valid_expire(ok), "{ok}");
    }
    for bad in ["", "y", "2y5", "2030-1-1", "soon", "-1y"] {
      assert!(!valid_expire(bad), "{bad}");
    }
  }

  #[test]
  fn key_sizes_follow_gpg_limits() {
    assert!(valid_key_bits("rsa", "4096"));
    assert!(valid_key_bits("dsa", "3072"));
    assert!(!valid_key_bits("dsa", "4096"));
    assert!(!valid_key_bits("rsa", "512"));
  }

  #[test]
  fn user_ids_and_fingerprints_are_checked() {
    assert!(valid_uid("Ana <ana@example.com>"));
    assert!(!valid_uid("-x"));
    assert!(!valid_uid("tab\there"));
    assert!(valid_fingerprint(
      "0123456789ABCDEF0123456789ABCDEF01234567"
    ));
    assert!(!valid_fingerprint("0123abc"));
  }

  #[test]
  fn date_of_a_timestamp_is_utc() {
    assert_eq!(date_of(0), "1970-01-01");
    assert_eq!(date_of(1_700_000_000), "2023-11-14");
    assert_eq!(date_of(951_782_400), "2000-02-29");
  }

  #[test]
  fn listing_rows_come_from_the_primary_keys() {
    let listing = "pub:u:4096:1:AAAA1111BBBB2222:1600000000:1700000000::::::::::\n\
fpr:::::::::0123456789ABCDEF0123456789ABCDEF01234567:\n\
uid:u::::1700000000::H::Ana <ana@example.com>:\n\
sub:u:4096:1:CCCC3333DDDD4444:1700000000::::::::::\n\
fpr:::::::::FEDCBA9876543210FEDCBA9876543210FEDCBA98:\n";
    let rows = parse_key_listing(
      listing,
      &["0123456789ABCDEF0123456789ABCDEF01234567".into()],
    );
    assert_eq!(rows.len(), 1);
    let row: &KeyRow = &rows[0];
    assert_eq!(row.key_type, "rsa");
    assert_eq!(row.keyid, "AAAA1111BBBB2222");
    assert_eq!(row.expires, "2023-11-14");
    assert!(row.secret);
    assert_eq!(row.uid, "Ana <ana@example.com>");
    assert_eq!(row.tsv().split('\t').count(), 7);
  }

  #[test]
  fn subkeys_are_listed_apart() {
    let listing =
      "pub:u:1:1:K:1::::::::::\nfpr:::::::::AAAA:\nsub:u:1:1:S:1::::::::::\nfpr:::::::::BBBB:\n";
    assert_eq!(parse_subkey_fingerprints(listing), vec!["BBBB"]);
  }

  #[test]
  fn the_created_fingerprint_is_read_from_the_status_line() {
    let status = "[GNUPG:] KEY_CREATED P 0123ABCD\n";
    assert_eq!(created_fingerprint(status).as_deref(), Some("0123ABCD"));
  }

  #[test]
  fn the_managed_pinentry_block_is_replaced_and_user_lines_kept() {
    let conf =
      format!("allow-loopback-pinentry\n{MARK_BEGIN}\npinentry-program /usr/bin/old\n{MARK_END}\n");
    assert_eq!(managed_pinentry(&conf).as_deref(), Some("/usr/bin/old"));
    let updated = with_managed_pinentry(&conf, "/usr/bin/pinentry-gtk");
    assert_eq!(
      updated,
      format!(
        "allow-loopback-pinentry\n{MARK_BEGIN}\npinentry-program /usr/bin/pinentry-gtk\n{MARK_END}\n"
      )
    );
  }

  #[test]
  fn a_user_pinentry_outside_the_block_is_reported() {
    assert_eq!(
      user_pinentry("  pinentry-program /usr/bin/x\n").as_deref(),
      Some("/usr/bin/x")
    );
    let managed = format!("{MARK_BEGIN}\npinentry-program /usr/bin/x\n{MARK_END}\n");
    assert_eq!(user_pinentry(&managed), None);
  }

  #[test]
  fn revocation_reasons_and_trust_levels_map_to_gpg_codes() {
    assert_eq!(revocation_code("superseded"), Some("2"));
    assert_eq!(revocation_code(""), None);
    assert_eq!(trust_value("ultimate"), Some("5"));
    assert_eq!(trust_value("bogus"), None);
  }
}
