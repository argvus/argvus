use super::*;
use crate::{ConfigStore, session_paths::SessionPaths};
use std::fs;
use std::process::{Child, Command};

pub(super) enum PositionMode {
  Save,
  Apply,
}

pub(super) fn position(position: &str, mode: PositionMode) -> ConfigResult<()> {
  if !matches!(position, "top" | "bottom") {
    return Err(invalid("position must be top or bottom"));
  }
  let store = ConfigStore::from_environment()?;
  let document = store.update("/layout/taskbar/position", json!(position))?;
  let paths =
    SessionPaths::from_environment().ok_or_else(|| invalid("configuration paths unavailable"))?;
  crate::project::project_layout_only(&store, &document)?;
  if matches!(mode, PositionMode::Save) {
    return Ok(());
  }
  let path = crate::user_copy::ensure_user_copy(&paths, "taskbar/config/argvus-taskbar.jsonc")?;
  let before = fs::read_to_string(&path)?;
  let pattern = regex::Regex::new(r#""position"\s*:\s*"(?:top|bottom)""#)
    .map_err(|error| invalid(error.to_string()))?;
  let after = pattern
    .replace(&before, format!("\"position\": \"{position}\""))
    .into_owned();
  if before != after {
    crate::text_edit::replace_file(&path, &after)?;
    if std::env::var("ARGVUS_NO_RUNTIME").as_deref() != Ok("1")
      && std::env::var("ARGVUS_SKIP_WAYBAR_RESTART").as_deref() != Ok("1")
    {
      let status = Command::new("argvus")
        .args(["sessionctl", "restart", "taskbar"])
        .status()?;
      if !status.success() {
        return Err(invalid("taskbar position saved but restart failed"));
      }
    }
  }
  Ok(())
}

fn stop(child: &mut Option<Child>) {
  if let Some(mut child) = child.take() {
    let _ = child.kill();
    let _ = child.wait();
  }
}

fn configuration(paths: &SessionPaths, snapshot: &Value) -> ConfigResult<(String, String)> {
  let source = paths.read_config("taskbar/config/window-bar.json");
  let mut config: Value = serde_json::from_str(&fs::read_to_string(source)?)?;
  config["position"] = json!(if snapshot["position"] == "bottom" {
    "top"
  } else {
    "bottom"
  });
  // Read the same effective files as the primary bar, including native overrides.
  // Theme layout adapters write Spaced margins/radii there, rather than into
  // every canonical field. Mirror vertical margins when swapping the edge.
  let primary_path = paths.read_config("taskbar/config/argvus-taskbar.jsonc");
  let primary_text = fs::read_to_string(primary_path)?;
  let primary: Value =
    serde_json::from_str(&crate::legacy_formats::strip_json_comments(&primary_text))?;
  inherit_geometry(&mut config, &primary);
  let primary_css = paths.read_config("taskbar/config/argvus-taskbar.css");
  let primary_style = fs::read_to_string(&primary_css)?;
  let mut hasher = DefaultHasher::new();
  primary_style.hash(&mut hasher);
  let mut css = format!(
    "@import url({});\n/* primary style: {:016x} */\n",
    serde_json::to_string(&primary_css.to_string_lossy())?,
    hasher.finish()
  );
  css.push_str(&fs::read_to_string(
    paths.read_config("taskbar/config/window-bar.css"),
  )?);
  if let Some(group) = primary_group_style(&primary_style) {
    css.push_str(&format!("\n#argvus-windows button {{{group}}}\n"));
  }
  Ok((serde_json::to_string_pretty(&config)?, css))
}

fn primary_group_style(css: &str) -> Option<&str> {
  // This is the same shared group block edited by waybar::apply_layout and
  // set_taskbar_border_width. Keep its spacing, rounding and border overrides.
  let start = css.find("#right-0, #right-1, #right-2,")?;
  let (_, body) = css[start..].split_once('{')?;
  Some(body.split_once('}')?.0)
}

fn inherit_geometry(config: &mut Value, primary: &Value) {
  for key in ["height", "width", "margin", "spacing"] {
    if let Some(value) = primary.get(key) {
      config[key] = value.clone();
    }
  }
  for (edge, source) in [
    ("top", "bottom"),
    ("bottom", "top"),
    ("left", "left"),
    ("right", "right"),
  ] {
    if let Some(value) = primary.get(format!("margin-{source}")) {
      config[format!("margin-{edge}")] = value.clone();
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn window_buttons_share_primary_group_border_and_rounding() {
    let css = "#right-0, #right-1, #right-2, #focus { border: 3px solid @th-border-rights; border-radius: 5px; padding: 0 10px; }";
    let group = primary_group_style(css).unwrap();
    assert!(group.contains("border: 3px"));
    assert!(group.contains("border-radius: 5px"));
    assert!(group.contains("padding: 0 10px"));
    assert!(primary_group_style("window#waybar { border: none; }").is_none());
  }

  #[test]
  fn secondary_bar_mirrors_effective_spaced_geometry() {
    let mut secondary = json!({"name":"argvus-window-bar", "position":"bottom"});
    inherit_geometry(
      &mut secondary,
      &json!({"height":32,"margin-top":16,"margin-bottom":3,"margin-left":18,"margin-right":20}),
    );
    assert_eq!(secondary["height"], 32);
    assert_eq!(secondary["margin-top"], 3);
    assert_eq!(secondary["margin-bottom"], 16);
    assert_eq!(secondary["margin-left"], 18);
    assert_eq!(secondary["margin-right"], 20);
    assert_eq!(secondary["name"], "argvus-window-bar");
    assert_eq!(secondary["position"], "bottom");
  }
}

/// A separate, supervised Waybar process. Mode and position are derived; this
/// runner owns no independent preferences and never kills the primary bar.
pub(super) fn run() -> ConfigResult<()> {
  let paths =
    SessionPaths::from_environment().ok_or_else(|| invalid("configuration paths unavailable"))?;
  let mut child: Option<Child> = None;
  let mut previous = None;
  loop {
    if let Some(process) = child.as_mut()
      && process.try_wait()?.is_some()
    {
      child = None;
      previous = None;
    }
    match request("list", None) {
      Ok(snapshot) if snapshot["floating"] == true || snapshot["recovering"] == true => {
        let current = configuration(&paths, &snapshot)?;
        if previous.as_ref() != Some(&current) || child.is_none() {
          stop(&mut child);
          let config_path = runtime_path("bar.json")?;
          let css_path = runtime_path("bar.css")?;
          crate::text_edit::replace_file(&config_path, &current.0)?;
          crate::text_edit::replace_file(&css_path, &current.1)?;
          child = Some(
            Command::new("waybar")
              .args(["--config"])
              .arg(config_path)
              .arg("--style")
              .arg(css_path)
              .spawn()?,
          );
          previous = Some(current);
        }
      }
      Ok(_) => {
        stop(&mut child);
        previous = None;
      }
      Err(error) => {
        eprintln!("argvus-window-bar: {error}");
        stop(&mut child);
        previous = None;
      }
    }
    std::thread::sleep(Duration::from_secs(1));
  }
}
