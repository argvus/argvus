use super::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::os::unix::fs::OpenOptionsExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct Origin {
  pub pid: i64,
  pub class: String,
  pub workspace: String,
  pub monitor: String,
  #[serde(default)]
  pub monitor_at: [i64; 2],
  pub at: [i64; 2],
  pub size: [i64; 2],
  pub fullscreen: i64,
  pub fullscreen_client: i64,
  pub pinned: bool,
}

pub(super) type Registry = BTreeMap<String, Origin>;

/// Group tabs share one desktop window. Hidden members must not become extra
/// buttons; minimized groups keep one representative so they remain recoverable.
pub(super) fn listed(window: &Value, origin: Option<&Origin>, output: &Value) -> bool {
  if window["mapped"] != true {
    return false;
  }
  if origin.is_some() {
    if let Some(first) = window["grouped"].as_array().and_then(|group| group.first())
      && first != &window["address"]
    {
      return false;
    }
  } else if window["hidden"] == true {
    return false;
  }
  let workspace = origin
    .map(|origin| origin.workspace.clone())
    .unwrap_or_else(|| workspace(window));
  if origin.is_none() && window["pinned"] == true {
    return true;
  }
  let special = output
    .pointer("/specialWorkspace/name")
    .and_then(Value::as_str)
    .unwrap_or("");
  let current = if special.is_empty() {
    output
      .pointer("/activeWorkspace/name")
      .and_then(Value::as_str)
      .unwrap_or("")
  } else {
    special
  };
  !current.is_empty() && workspace == current
}

pub(super) fn workspace(window: &Value) -> String {
  window
    .pointer("/workspace/name")
    .and_then(Value::as_str)
    .unwrap_or("1")
    .to_owned()
}

pub(super) fn workspace_selector(name: &str) -> String {
  if name.starts_with("special:") || name.parse::<i64>().is_ok() {
    name.to_owned()
  } else {
    format!("name:{name}")
  }
}

pub(super) fn same_window(origin: &Origin, window: &Value) -> bool {
  window["pid"].as_i64() == Some(origin.pid)
    && window["initialClass"].as_str() == Some(&origin.class)
}

pub(super) fn capture(window: &Value, monitors: &[Value]) -> Origin {
  let pair = |field: &str| {
    [
      window[field][0].as_i64().unwrap_or(0),
      window[field][1].as_i64().unwrap_or(0),
    ]
  };
  Origin {
    pid: window["pid"].as_i64().unwrap_or(-1),
    class: window["initialClass"].as_str().unwrap_or("").into(),
    workspace: workspace(window),
    monitor: monitors
      .iter()
      .find(|monitor| monitor["id"] == window["monitor"])
      .and_then(|monitor| monitor["name"].as_str())
      .unwrap_or("")
      .into(),
    monitor_at: monitors
      .iter()
      .find(|monitor| monitor["id"] == window["monitor"])
      .map(|monitor| {
        [
          monitor["x"].as_i64().unwrap_or(0),
          monitor["y"].as_i64().unwrap_or(0),
        ]
      })
      .unwrap_or([0, 0]),
    at: pair("at"),
    size: pair("size"),
    fullscreen: window["fullscreen"].as_i64().unwrap_or(0),
    fullscreen_client: window["fullscreenClient"].as_i64().unwrap_or(0),
    pinned: window["pinned"].as_bool().unwrap_or(false),
  }
}

pub(super) fn save(registry: &Registry) -> ConfigResult<()> {
  save_named(registry, "json")
}

pub(super) fn save_named(registry: &Registry, extension: &str) -> ConfigResult<()> {
  let path = runtime_path(extension)?;
  let temporary = path.with_extension(format!("{extension}.tmp"));
  let mut file = OpenOptions::new()
    .create(true)
    .truncate(true)
    .write(true)
    .mode(0o600)
    .open(&temporary)?;
  serde_json::to_writer(&mut file, registry)?;
  file.sync_all()?;
  fs::rename(temporary, path)?;
  Ok(())
}

pub(super) fn load() -> ConfigResult<Registry> {
  load_named("json")
}

pub(super) fn load_named(extension: &str) -> ConfigResult<Registry> {
  match fs::read(runtime_path(extension)?) {
    Ok(bytes) => Ok(serde_json::from_slice(&bytes)?),
    Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Registry::new()),
    Err(error) => Err(error.into()),
  }
}

/// Keep exact geometry on the original output. On a removed/moved output,
/// translate to the replacement's coordinate space and keep the window visible.
pub(super) fn restored_geometry(origin: &Origin, output: &Value) -> ([i64; 2], [i64; 2]) {
  let current = [
    output["x"].as_i64().unwrap_or(0),
    output["y"].as_i64().unwrap_or(0),
  ];
  if output["name"] == origin.monitor && current == origin.monitor_at {
    return (origin.at, origin.size);
  }
  let scale = output["scale"]
    .as_f64()
    .filter(|scale| *scale > 0.0)
    .unwrap_or(1.0);
  let width = (output["width"].as_f64().unwrap_or(1920.0) / scale) as i64;
  let height = (output["height"].as_f64().unwrap_or(1080.0) / scale) as i64;
  let reserve = |index: usize| output["reserved"][index].as_i64().unwrap_or(0);
  let size = [
    origin.size[0].clamp(1, (width - reserve(0) - reserve(2)).max(1)),
    origin.size[1].clamp(1, (height - reserve(1) - reserve(3) - 28).max(1)),
  ];
  let at = [
    current[0]
      + (origin.at[0] - origin.monitor_at[0])
        .clamp(reserve(0), (width - reserve(2) - size[0]).max(reserve(0))),
    current[1]
      + (origin.at[1] - origin.monitor_at[1]).clamp(
        reserve(1) + 28,
        (height - reserve(3) - size[1]).max(reserve(1) + 28),
      ),
  ];
  (at, size)
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn registry_rejects_reused_addresses_and_retains_geometry() {
    let window = json!({"pid":42,"initialClass":"editor","workspace":{"name":"3"},"at":[10,20],"size":[800,600],"fullscreen":1,"pinned":true});
    let origin = capture(&window, &[]);
    assert!(same_window(&origin, &window));
    assert!(!same_window(
      &origin,
      &json!({"pid":43,"initialClass":"editor"})
    ));
    let decoded: Origin = serde_json::from_value(serde_json::to_value(origin).unwrap()).unwrap();
    assert_eq!(decoded.workspace, "3");
    assert_eq!(decoded.size, [800, 600]);
    assert!(decoded.pinned);
  }
}

#[cfg(test)]
mod monitor_tests {
  use super::*;
  #[test]
  fn named_workspaces_use_a_hyprland_name_selector() {
    assert_eq!(workspace_selector("dev"), "name:dev");
    assert_eq!(workspace_selector("4"), "4");
    assert_eq!(workspace_selector("special:notes"), "special:notes");
  }
  #[test]
  fn removed_monitor_restores_inside_the_remaining_output() {
    let window = json!({"pid":1,"initialClass":"app","workspace":{"name":"2"},"monitor":2,"at":[2100,200],"size":[1200,900]});
    let origin = capture(&window, &[json!({"id":2,"name":"removed","x":1920,"y":0})]);
    let (at, size) = restored_geometry(
      &origin,
      &json!({"name":"remaining","x":0,"y":0,"width":960,"height":600,"scale":1,"reserved":[0,30,0,30]}),
    );
    assert!(at[0] >= 0 && at[1] >= 58);
    assert!(at[0] + size[0] <= 960);
    assert!(at[1] + size[1] <= 570);
  }
}

#[cfg(test)]
mod taskbar_tests {
  use super::*;

  #[test]
  fn lists_only_current_workspace_and_pinned_windows() {
    let output = json!({"activeWorkspace":{"name":"dev"},"specialWorkspace":{"name":""}});
    let mut window = json!({"mapped":true,"workspace":{"name":"dev"}});
    assert!(listed(&window, None, &output));
    window["workspace"]["name"] = json!("other");
    assert!(!listed(&window, None, &output));
    window["pinned"] = json!(true);
    assert!(listed(&window, None, &output));
    window["hidden"] = json!(true);
    assert!(!listed(&window, None, &output));
  }

  #[test]
  fn special_workspace_replaces_normal_workspace() {
    let output =
      json!({"activeWorkspace":{"name":"dev"},"specialWorkspace":{"name":"special:notes"}});
    let mut window = json!({"mapped":true,"workspace":{"name":"dev"}});
    assert!(!listed(&window, None, &output));
    window["workspace"]["name"] = json!("special:notes");
    assert!(listed(&window, None, &output));
  }

  #[test]
  fn minimized_group_has_one_entry_on_its_original_workspace() {
    let output = json!({"activeWorkspace":{"name":"dev"}});
    let mut window = json!({"mapped":true,"hidden":true,"address":"0x1","grouped":["0x1","0x2"],"workspace":{"name":"dev"}});
    let origin = capture(&window, &[]);
    assert!(!listed(&window, None, &output));
    window["workspace"]["name"] = json!("special:argvus-minimized");
    assert!(listed(&window, Some(&origin), &output));
    window["address"] = json!("0x2");
    assert!(!listed(&window, Some(&origin), &output));
    window["address"] = json!("0x1");
    assert!(!listed(
      &window,
      Some(&origin),
      &json!({"activeWorkspace":{"name":"other"}})
    ));
  }
}
