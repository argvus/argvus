//! Data-only themes in the user's XDG data directory.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::error::Error;
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Theme {
  version: u8,
  id: String,
  name: String,
  top: String,
  bottom: String,
  border: String,
  indicator: String,
  opacity: f64,
  border_width: u8,
  #[serde(skip_serializing_if = "Option::is_none")]
  description: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  radius: Option<u8>,
  #[serde(skip_serializing_if = "Option::is_none")]
  padding: Option<u8>,
  #[serde(skip_serializing_if = "Option::is_none")]
  spacing: Option<u8>,
  #[serde(skip_serializing_if = "Option::is_none")]
  shadow: Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  reflection: Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  focus: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  separator: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  popup: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  foreground: Option<String>,
}

fn valid_id(id: &str) -> bool {
  id.starts_with("user-")
    && (6..=64).contains(&id.len())
    && id
      .bytes()
      .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
}

fn read_theme(path: &Path) -> Result<Theme, Box<dyn Error>> {
  let file = OpenOptions::new()
    .read(true)
    .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
    .open(path)?;
  if !file.metadata()?.is_file() {
    return Err("theme must be a regular file".into());
  }
  let mut bytes = Vec::new();
  file.take(16385).read_to_end(&mut bytes)?;
  if bytes.len() > 16384 {
    return Err("theme exceeds 16 KiB".into());
  }
  let theme: Theme = serde_json::from_slice(&bytes)?;
  let colors = [&theme.top, &theme.bottom, &theme.border, &theme.indicator];
  if theme.version != 1
    || !valid_id(&theme.id)
    || theme.name.trim().is_empty()
    || theme.name.chars().count() > 80
    || theme.name.chars().any(char::is_control)
    || theme
      .description
      .as_ref()
      .is_some_and(|s| s.chars().count() > 240 || s.chars().any(char::is_control))
    || theme.radius.is_some_and(|n| n > 32)
    || theme.padding.is_some_and(|n| !(6..=24).contains(&n))
    || theme.spacing.is_some_and(|n| n > 24)
    || theme.border_width > 4
    || !(0.0..=1.0).contains(&theme.opacity)
    || !colors
      .into_iter()
      .chain(
        [
          theme.focus.as_ref(),
          theme.separator.as_ref(),
          theme.popup.as_ref(),
          theme.foreground.as_ref(),
        ]
        .into_iter()
        .flatten(),
      )
      .all(|color| {
        color
          .strip_prefix('#')
          .is_some_and(|hex| hex.len() == 6 && hex.bytes().all(|c| c.is_ascii_hexdigit()))
      })
  {
    return Err("invalid Dock theme".into());
  }
  Ok(theme)
}

fn directory() -> Result<PathBuf, Box<dyn Error>> {
  let data = std::env::var_os("XDG_DATA_HOME")
    .map(PathBuf::from)
    .filter(|p| p.is_absolute())
    .or_else(|| std::env::var_os("HOME").map(|p| PathBuf::from(p).join(".local/share")))
    .ok_or("no XDG data directory")?;
  Ok(data.join("argvus/dock/themes"))
}

pub(super) fn run(arguments: &[String]) -> Result<(), Box<dyn Error>> {
  run_at(arguments, &directory()?)
}

fn run_at(arguments: &[String], dir: &Path) -> Result<(), Box<dyn Error>> {
  if dir.symlink_metadata().is_ok_and(|meta| !meta.is_dir()) {
    return Err("theme directory must be a directory".into());
  }
  match arguments {
    [command] if command == "list" => {
      let mut themes = BTreeMap::new();
      if dir.try_exists()? {
        for entry in fs::read_dir(dir)? {
          let path = entry?.path();
          if path.extension().is_some_and(|ext| ext == "json")
            && let Ok(theme) = read_theme(&path)
            && path.file_stem().and_then(|stem| stem.to_str()) == Some(&theme.id)
          {
            themes.insert(theme.id.clone(), theme);
          }
        }
      }
      println!("{}", serde_json::to_string(&themes)?);
    }
    [command, source] if command == "import" => {
      let theme = read_theme(Path::new(source))?;
      fs::create_dir_all(dir)?;
      let path = dir.join(format!("{}.json", theme.id));
      let bytes = serde_json::to_vec_pretty(&theme)?;
      // Publish the complete regular file atomically without replacing an existing ID.
      let temporary = dir.join(format!(".import-{}", std::process::id()));
      let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&temporary)?;
      let result = file
        .write_all(&bytes)
        .and_then(|()| file.sync_all())
        .and_then(|()| fs::hard_link(&temporary, path));
      fs::remove_file(temporary)?;
      result?;
      println!("{}", theme.id);
    }
    [command, id] if command == "remove" && valid_id(id) => {
      fs::remove_file(dir.join(format!("{id}.json")))?;
    }
    _ => return Err("usage: argvus config dock-theme list | import FILE | remove ID".into()),
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::os::unix::fs::symlink;

  #[test]
  fn custom_theme_import_is_bounded_and_never_overwrites_an_existing_id() {
    let root = PathBuf::from("/tmp/argvus").join(format!("dock-theme-test-{}", std::process::id()));
    fs::create_dir_all(root.parent().unwrap()).unwrap();
    fs::create_dir(&root).unwrap();
    let themes = root.join("themes");
    let source = root.join("example.json");
    let valid = serde_json::json!({"id":"user-test","version":1,"name":"Test","top":"#abcdef","bottom":"#000000","indicator":"#FFFFFF","border":"#123456","opacity":0.8,"borderWidth":2});
    fs::write(&source, valid.to_string()).unwrap();
    let import = ["import".to_string(), source.to_string_lossy().into_owned()];
    run_at(&import, &themes).unwrap();
    assert!(run_at(&import, &themes).is_err());
    let installed = themes.join("user-test.json");
    assert_eq!(read_theme(&installed).unwrap().id, "user-test");
    for (key, bad) in [
      ("version", serde_json::json!(2)),
      ("name", serde_json::json!("")),
      ("opacity", serde_json::json!(2)),
      ("id", serde_json::json!("other")),
      ("border", serde_json::json!("red")),
    ] {
      let mut invalid = valid.clone();
      invalid[key] = bad;
      fs::write(&source, invalid.to_string()).unwrap();
      assert!(read_theme(&source).is_err());
    }
    let link = root.join("link.json");
    symlink(&installed, &link).unwrap();
    assert!(read_theme(&link).is_err());
    fs::write(&source, [b' '; 16385]).unwrap();
    assert!(read_theme(&source).is_err());
    assert!(run_at(&["remove".into(), "../outside".into()], &themes).is_err());
    run_at(&["remove".into(), "user-test".into()], &themes).unwrap();
    assert!(!installed.exists());
    fs::remove_dir_all(root).unwrap();
  }
}
