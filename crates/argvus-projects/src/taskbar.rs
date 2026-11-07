//! Derived labels for the native Waybar workspace buttons; never project configuration.

use crate::messages;
use argvus_projects_core::projects;
use serde_json::{Map, Value, json};
use std::{env, fs, io, path::PathBuf};

pub(super) fn write_metadata(paths: &[String]) -> io::Result<()> {
  let runtime = env::var_os("XDG_RUNTIME_DIR")
    .filter(|path| !path.is_empty())
    .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "XDG_RUNTIME_DIR is not set"))?;
  let directory = PathBuf::from(runtime).join("argvus");
  fs::create_dir_all(&directory)?;
  let mut entries = Map::new();
  for (index, path) in paths.iter().enumerate() {
    let name = PathBuf::from(path)
      .file_name()
      .unwrap_or_default()
      .to_string_lossy()
      .into_owned();
    let label = messages::text("project.workspace_label", &[format!("name={name}")]);
    let shortcut = format!("SUPER + CTRL + {}", index + 1);
    let metadata = json!({ "label": label, "tooltip": format!("{label}\n{path}\n{shortcut}") });
    entries.insert(projects::workspace_name(path), metadata);
  }
  let temporary = directory.join(format!("project-workspaces.{}.json", std::process::id()));
  let result = (|| {
    fs::write(&temporary, Value::Object(entries).to_string())?;
    fs::rename(&temporary, directory.join("project-workspaces.json"))
  })();
  if result.is_err() {
    let _ = fs::remove_file(temporary);
  }
  result
}
