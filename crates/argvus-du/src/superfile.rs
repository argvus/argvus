//! Prepare picker-specific Superfile shortcuts without changing user files.
use std::{ffi::OsString, fs, io::Write, path::PathBuf, process::Command};

pub struct PickerKeys {
  pub args: Vec<String>,
  pub config_home: PathBuf,
  _file: tempfile::NamedTempFile,
}

pub fn prepare(arguments: &[OsString]) -> Result<PickerKeys, String> {
  let output = Command::new("argvus")
    .args(["--spf", "path-list"])
    .output()
    .map_err(|e| e.to_string())?;
  if !output.status.success() {
    return Err("Cannot locate Superfile configuration".into());
  }
  let text = strip_ansi(&output.stdout);
  let default_hotkeys = path_from_list(&text, "[Hotkeys file path]")?;
  let directory = path_from_list(&text, "[Configuration directory path]")?;
  let mut args = Vec::new();
  let mut override_path = None;
  let mut iter = arguments.iter();
  while let Some(arg) = iter.next() {
    let arg = arg.to_str().ok_or("Invalid file manager argument")?;
    if arg == "--hotkey-file" || arg == "--hf" {
      override_path = Some(PathBuf::from(
        iter.next().ok_or("Missing hotkey file argument")?,
      ));
    } else if let Some(path) = arg
      .strip_prefix("--hotkey-file=")
      .or_else(|| arg.strip_prefix("--hf="))
    {
      override_path = Some(PathBuf::from(path));
    } else {
      args.push(arg.to_owned());
    }
  }
  let path = override_path.unwrap_or(default_hotkeys);
  let input = fs::read_to_string(path).map_err(|e| e.to_string())?;
  let mut table = toml::from_str::<toml::Table>(&input).map_err(|e| e.to_string())?;
  for (key, value) in &mut table {
    if key == "cd_quit" || key.contains("typing") {
      continue;
    }
    if let Some(keys) = value.as_array_mut() {
      keys.retain(|key| !matches!(key.as_str(), Some("space" | " ")));
    }
  }
  table.insert(
    "cd_quit".into(),
    toml::Value::Array(vec![toml::Value::String("space".into())]),
  );
  let cache = std::env::var_os("XDG_CACHE_HOME")
    .map(PathBuf::from)
    .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache")))
    .unwrap_or_else(|| PathBuf::from("/tmp/argvus"))
    .join("argvus");
  fs::create_dir_all(&cache).map_err(|e| e.to_string())?;
  let mut file = tempfile::Builder::new()
    .prefix("du-superfile-hotkeys-")
    .tempfile_in(cache)
    .map_err(|e| e.to_string())?;
  write!(
    file,
    "{}",
    toml::to_string(&table).map_err(|e| e.to_string())?
  )
  .map_err(|e| e.to_string())?;
  args.extend([
    "--hotkey-file".into(),
    file.path().to_string_lossy().into_owned(),
  ]);
  let config_home = directory
    .parent()
    .ok_or("Invalid Superfile configuration directory")?
    .to_path_buf();
  Ok(PickerKeys {
    args,
    config_home,
    _file: file,
  })
}
fn path_from_list(text: &str, label: &str) -> Result<PathBuf, String> {
  text
    .lines()
    .find_map(|line| {
      line
        .split_once(label)
        .map(|(_, path)| PathBuf::from(path.trim_start().trim_end_matches('\r')))
    })
    .filter(|path| !path.as_os_str().is_empty())
    .ok_or_else(|| format!("Missing {label} in Superfile paths"))
}
fn strip_ansi(bytes: &[u8]) -> String {
  let mut out = Vec::new();
  let mut iter = bytes.iter().copied();
  while let Some(byte) = iter.next() {
    if byte == 27 {
      if iter.next() == Some(b'[') {
        for c in iter.by_ref() {
          if (0x40..=0x7e).contains(&c) {
            break;
          }
        }
      }
    } else {
      out.push(byte);
    }
  }
  String::from_utf8_lossy(&out).into_owned()
}
