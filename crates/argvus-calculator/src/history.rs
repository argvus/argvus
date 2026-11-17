//! Bounded, atomically replaced history in the user's XDG data directory.
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::{
  fs, io,
  path::{Path, PathBuf},
};

#[derive(Clone, Serialize, Deserialize)]
pub struct Record {
  #[serde(default)]
  pub context: Option<Context>,
  pub expression: String,
  pub result: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Context {
  pub preferences: crate::preferences::Preferences,
  #[serde(default)]
  pub fields: [String; 3],
}

pub fn path() -> io::Result<PathBuf> {
  let data = std::env::var_os("XDG_DATA_HOME")
    .filter(|value| !value.is_empty())
    .map(PathBuf::from)
    .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share")))
    .ok_or_else(|| io::Error::other("user data directory unavailable"))?;
  Ok(data.join("argvus/calculator/history.json"))
}
pub fn load(path: &Path) -> io::Result<Vec<Record>> {
  let data = match fs::read(path) {
    Ok(data) => data,
    Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
    Err(error) => return Err(error),
  };
  let mut records: Vec<Record> = serde_json::from_slice(&data)?;
  records.retain(|record| record.expression.len() <= 4096 && record.result.len() <= 64);
  if records.len() > 100 {
    records.drain(..records.len() - 100);
  }
  Ok(records)
}
pub fn save(path: &Path, records: &[Record]) -> io::Result<()> {
  save_json(path, &records)
}

pub(crate) fn save_json(path: &Path, value: &impl Serialize) -> io::Result<()> {
  let directory = path
    .parent()
    .ok_or_else(|| io::Error::other("invalid history path"))?;
  fs::create_dir_all(directory)?;
  let temporary = path.with_extension(format!("{}.tmp", std::process::id()));
  let outcome = (|| {
    let mut file = fs::OpenOptions::new()
      .write(true)
      .create_new(true)
      .mode(0o600)
      .open(&temporary)?;
    file.write_all(&serde_json::to_vec(value)?)?;
    file.sync_all()?;
    fs::rename(&temporary, path)?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
  })();
  if outcome.is_err() {
    let _ = fs::remove_file(temporary);
  }
  outcome
}
#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn persists_and_clears_history() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("calculator/history.json");
    assert!(load(&path).unwrap().is_empty());
    save(
      &path,
      &[Record {
        context: None,
        expression: "2+2".into(),
        result: "4".into(),
      }],
    )
    .unwrap();
    assert_eq!(load(&path).unwrap()[0].result, "4");
    assert_eq!(
      fs::metadata(&path).unwrap().permissions().mode() & 0o777,
      0o600
    );
    save(&path, &[]).unwrap();
    assert!(load(&path).unwrap().is_empty());
  }
  #[test]
  fn legacy_and_contextual_records_remain_readable() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("history.json");
    fs::write(&path, r#"[{"expression":"2+2","result":"4"}]"#).unwrap();
    let mut records = load(&path).unwrap();
    assert!(records[0].context.is_none());
    let preferences = crate::preferences::Preferences {
      profile: crate::preferences::Profile::Programming,
      ..Default::default()
    };
    records.push(Record {
      expression: "0xFF".into(),
      result: "255".into(),
      context: Some(Context {
        preferences,
        fields: std::array::from_fn(|_| String::new()),
      }),
    });
    save(&path, &records).unwrap();
    assert_eq!(
      load(&path).unwrap()[1]
        .context
        .as_ref()
        .unwrap()
        .preferences
        .profile,
      crate::preferences::Profile::Programming
    );
  }
  #[test]
  fn malformed_history_is_reported_without_overwriting_it() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("history.json");
    fs::write(&path, "invalid").unwrap();
    assert!(load(&path).is_err());
    assert_eq!(fs::read_to_string(path).unwrap(), "invalid");
  }
}
