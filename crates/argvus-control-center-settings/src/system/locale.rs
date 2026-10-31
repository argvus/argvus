//! Locale discovery and configuration of the Settings pages.
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::process::Stdio;

use crate::error::SettingsError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Distro {
  pub id: String,
  pub pretty_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocaleEntry {
  pub locale: String,
  pub encoding: String,
  pub enabled: bool,
  pub line_index: usize,
}

pub fn distro() -> Distro {
  parse_os_release(&fs::read_to_string("/etc/os-release").unwrap_or_default())
}

pub fn parse_os_release(contents: &str) -> Distro {
  let mut id = String::new();
  let mut pretty_name = String::new();
  for line in contents.lines() {
    let Some((key, value)) = line.split_once('=') else {
      continue;
    };
    let value = value.trim_matches('"').to_string();
    match key {
      "ID" => id = value,
      "PRETTY_NAME" => pretty_name = value,
      _ => {}
    }
  }
  Distro { id, pretty_name }
}

pub fn current_lang() -> String {
  argvus_i18n::user_locale()
}

pub fn generated_locales() -> Vec<String> {
  command_stdout("locale", &["-a"])
    .unwrap_or_default()
    .lines()
    .map(str::trim)
    .filter(|line| !line.is_empty() && *line != "C" && *line != "POSIX")
    .map(normalize_locale)
    .collect::<BTreeSet<_>>()
    .into_iter()
    .collect()
}

pub fn locale_gen_entries(path: &Path) -> Result<Vec<LocaleEntry>, SettingsError> {
  let contents = fs::read_to_string(path)
    .map_err(|error| SettingsError::System(format!("{}: {error}", path.display())))?;
  Ok(parse_locale_gen(&contents))
}

pub fn parse_locale_gen(contents: &str) -> Vec<LocaleEntry> {
  contents
    .lines()
    .enumerate()
    .filter_map(|(line_index, line)| parse_locale_gen_line(line, line_index))
    .collect()
}

fn parse_locale_gen_line(line: &str, line_index: usize) -> Option<LocaleEntry> {
  let trimmed = line.trim_start();
  let (enabled, body) = if let Some(rest) = trimmed.strip_prefix('#') {
    (false, rest.trim_start())
  } else {
    (true, trimmed)
  };
  if body.is_empty() || body.starts_with('#') {
    return None;
  }
  let mut parts = body.split_whitespace();
  let locale = parts.next()?;
  let encoding = parts.next()?;
  if parts.next().is_some() || !is_locale_name(locale) || !is_encoding_name(encoding) {
    return None;
  }
  Some(LocaleEntry {
    locale: locale.to_string(),
    encoding: encoding.to_string(),
    enabled,
    line_index,
  })
}

pub fn apply_locale_gen(contents: &str, selected: &BTreeSet<String>) -> String {
  let mut lines: Vec<String> = contents.lines().map(str::to_string).collect();
  for entry in parse_locale_gen(contents) {
    let should_enable = selected.contains(&format!("{} {}", entry.locale, entry.encoding));
    if should_enable == entry.enabled {
      continue;
    }
    let line = &mut lines[entry.line_index];
    if should_enable {
      if let Some(pos) = line.find('#') {
        let after_hash = &line[pos + 1..];
        if after_hash.trim_start().starts_with(&entry.locale) {
          let spaces = after_hash.len() - after_hash.trim_start().len();
          line.replace_range(pos..pos + 1 + spaces, "");
        }
      }
    } else if let Some(pos) = line.find(&entry.locale) {
      line.insert(pos, '#');
    }
  }
  let mut output = lines.join("\n");
  if contents.ends_with('\n') {
    output.push('\n');
  }
  output
}

#[derive(Debug, thiserror::Error)]
pub enum UserLocaleError {
  #[error("locale is invalid or not generated: {0}")]
  NotGenerated(String),
  #[error("cannot locate user locale configuration")]
  MissingConfigPath,
  #[error("could not save user locale: {0}")]
  Write(#[from] std::io::Error),
}

pub fn set_lang(locale: &str, generated: &[String]) -> Result<(), UserLocaleError> {
  let path = argvus_i18n::config_path().ok_or(UserLocaleError::MissingConfigPath)?;
  save_user_language(&path, locale, generated)
}

pub fn restart_control_panel() -> std::io::Result<()> {
  let output = argvus_control_center_core::process::command("systemctl")
    .args(["--user", "restart", "argvus-control-panel.service"])
    .stdin(Stdio::null())
    .output()?;
  if output.status.success() {
    return Ok(());
  }

  let message = String::from_utf8_lossy(&output.stderr).trim().to_owned();
  Err(std::io::Error::other(if message.is_empty() {
    format!("systemctl exited with {}", output.status)
  } else {
    message
  }))
}

fn save_user_language(
  path: &Path,
  locale: &str,
  generated: &[String],
) -> Result<(), UserLocaleError> {
  if !is_locale_name(locale) || !generated.iter().any(|candidate| candidate == locale) {
    return Err(UserLocaleError::NotGenerated(locale.to_owned()));
  }
  write_user_locale(path, locale)?;
  Ok(())
}

fn write_user_locale(path: &Path, locale: &str) -> std::io::Result<()> {
  use std::io::Write;
  use std::os::unix::fs::OpenOptionsExt;
  let previous = match fs::read_to_string(path) {
    Ok(value) => value,
    Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
    Err(error) => return Err(error),
  };
  let mut lines: Vec<&str> = previous
    .lines()
    .filter(|line| !line.trim_start().starts_with("LANG="))
    .collect();
  let setting = format!("LANG={locale}");
  lines.push(&setting);
  let parent = path
    .parent()
    .ok_or_else(|| std::io::Error::other("missing parent"))?;
  fs::create_dir_all(parent)?;
  let temporary = path.with_file_name(format!(".locale.conf.{}.tmp", std::process::id()));
  let result = (|| {
    let mut file = fs::OpenOptions::new()
      .write(true)
      .create_new(true)
      .mode(0o600)
      .open(&temporary)?;
    file.write_all((lines.join("\n") + "\n").as_bytes())?;
    file.sync_all()?;
    fs::rename(&temporary, path)
  })();
  if result.is_err() {
    let _ = fs::remove_file(temporary);
  }
  result
}

pub fn set_system_locales(
  entries: &[LocaleEntry],
  selected: &BTreeSet<String>,
) -> Result<(), SettingsError> {
  let valid: BTreeSet<String> = entries
    .iter()
    .map(|entry| format!("{} {}", entry.locale, entry.encoding))
    .collect();
  if selected.iter().any(|value| !valid.contains(value)) {
    return Err(SettingsError::System(
      "selected locale is not present in /etc/locale.gen".to_string(),
    ));
  }
  let mut args = vec!["locale".to_string(), "set-generated".to_string()];
  args.extend(selected.iter().cloned());
  let refs: Vec<&str> = args.iter().map(String::as_str).collect();
  super::privileged::run(&refs).map(|_| ())
}

pub fn encoding_from_locale(locale: &str) -> String {
  locale
    .rsplit_once('.')
    .map(|(_, encoding)| encoding.to_string())
    .unwrap_or_else(|| "UTF-8".to_string())
}

fn normalize_locale(value: &str) -> String {
  if let Some((base, encoding)) = value.rsplit_once('.') {
    format!(
      "{base}.{}",
      encoding.to_ascii_uppercase().replace("UTF8", "UTF-8")
    )
  } else {
    value.to_string()
  }
}

pub fn is_locale_name(value: &str) -> bool {
  !value.is_empty()
    && value
      .bytes()
      .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'.' | b'@' | b'-'))
}

pub fn is_encoding_name(value: &str) -> bool {
  !value.is_empty()
    && value
      .bytes()
      .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_'))
}

fn command_stdout(command: &str, args: &[&str]) -> Option<String> {
  let output = argvus_control_center_core::process::command(command)
    .args(args)
    .stdin(Stdio::null())
    .output()
    .ok()?;
  output
    .status
    .success()
    .then(|| String::from_utf8_lossy(&output.stdout).to_string())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parses_os_release() {
    let distro = parse_os_release("ID=arch\nPRETTY_NAME=\"Arch Linux\"\n");
    assert_eq!(distro.id, "arch");
    assert_eq!(distro.pretty_name, "Arch Linux");
  }

  #[test]
  fn parses_locale_gen_and_preserves_comments() {
    let entries = parse_locale_gen("# comment\n#en_US.UTF-8 UTF-8\nde_DE.UTF-8 UTF-8\n\n");
    assert_eq!(entries.len(), 2);
    assert!(!entries[0].enabled);
    assert!(entries[1].enabled);
  }

  #[test]
  fn toggles_only_locale_lines() {
    let input = "# Header\n#en_US.UTF-8 UTF-8\nde_DE.UTF-8 UTF-8\n";
    let selected = BTreeSet::from(["en_US.UTF-8 UTF-8".to_string()]);
    let output = apply_locale_gen(input, &selected);
    assert!(output.contains("# Header"));
    assert!(output.contains("en_US.UTF-8 UTF-8"));
    assert!(output.contains("#de_DE.UTF-8 UTF-8"));
  }
}

#[cfg(test)]
mod user_locale_tests {
  use super::*;
  #[test]
  fn changing_one_profile_preserves_other_profiles_and_regional_settings() {
    let directory = tempfile::tempdir().unwrap();
    let first = directory.path().join("first/locale.conf");
    let second = directory.path().join("second/locale.conf");
    write_user_locale(&first, "en_US.UTF-8").unwrap();
    write_user_locale(&second, "en_US.UTF-8").unwrap();
    fs::write(
      &first,
      "# user preference\nLANG=en_US.UTF-8\nLC_TIME=en_GB.UTF-8\n",
    )
    .unwrap();
    write_user_locale(&first, "pt_BR.UTF-8").unwrap();
    let value = fs::read_to_string(first).unwrap();
    assert!(value.contains("LANG=pt_BR.UTF-8"));
    assert!(value.contains("LC_TIME=en_GB.UTF-8"));
    assert!(value.contains("# user preference"));
    assert_eq!(fs::read_to_string(second).unwrap(), "LANG=en_US.UTF-8\n");
  }

  #[test]
  fn language_selection_writes_exact_lang() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("config/locale.conf");
    let generated = vec!["en_US.UTF-8".into(), "pt_BR.UTF-8".into()];
    for locale in ["en_US.UTF-8", "pt_BR.UTF-8"] {
      save_user_language(&path, locale, &generated).unwrap();
      assert_eq!(
        fs::read_to_string(&path).unwrap(),
        format!("LANG={locale}\n")
      );
    }
  }

  #[test]
  fn invalid_or_unavailable_locale_does_not_write() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("locale.conf");
    for locale in ["xx_XX.UTF-8", "en_US.UTF-8\nLC_ALL=C"] {
      assert!(matches!(
        save_user_language(&path, locale, &["en_US.UTF-8".into()]),
        Err(UserLocaleError::NotGenerated(_))
      ));
      assert!(!path.exists());
    }
  }

  #[test]
  fn write_failure_preserves_other_files() {
    let directory = tempfile::tempdir().unwrap();
    let parent = directory.path().join("not-a-directory");
    fs::write(&parent, "keep me").unwrap();
    assert!(matches!(
      save_user_language(
        &parent.join("locale.conf"),
        "en_US.UTF-8",
        &["en_US.UTF-8".into()]
      ),
      Err(UserLocaleError::Write(_))
    ));
    assert_eq!(fs::read_to_string(parent).unwrap(), "keep me");
  }
}
