//! Adapters for existing ARGVUS contracts. Provider failures are isolated.
use crate::search::{Action, Entry};
use argvus_i18n::{Lang, tr};
use std::{path::PathBuf, process::Command};

pub fn language() -> Lang {
  Lang::for_domain(&Lang::detect().locale(), "launcher")
}
pub fn argvus() -> PathBuf {
  std::env::current_exe()
    .ok()
    .filter(|path| path.is_file())
    .unwrap_or_else(|| PathBuf::from("argvus"))
}
pub fn output(args: &[&str]) -> Result<String, String> {
  let output = Command::new(argvus())
    .args(args)
    .output()
    .map_err(|e| e.to_string())?;
  if !output.status.success() {
    return Err(String::from_utf8_lossy(&output.stderr).trim().into());
  }
  String::from_utf8(output.stdout).map_err(|e| e.to_string())
}

pub fn settings() -> Result<Vec<Entry>, String> {
  let values: Vec<serde_json::Value> =
    serde_json::from_str(&output(&["control-center", "--search-catalog"])?)
      .map_err(|e| e.to_string())?;
  let lang = language();
  Ok(
    values
      .into_iter()
      .filter_map(|value| {
        let route = value["route"].as_str()?;
        let id = value["id"].as_str()?;
        let mut keywords: Vec<String> =
          serde_json::from_value(value["keywords"].clone()).unwrap_or_default();
        let intent = if route.starts_with("displays/") {
          Some("universal.intent_monitor")
        } else if route == "audio/input" {
          Some("universal.intent_microphone")
        } else if route == "diagnostics/network" {
          Some("universal.intent_network")
        } else {
          None
        };
        if let Some(key) = intent {
          keywords.extend(tr(lang, key).split('|').map(str::to_owned));
        }
        Some(Entry {
          id: format!("settings:{id}"),
          title: value["title"].as_str()?.into(),
          description: {
            let key = format!(
              "control_center.{}",
              value["category"].as_str().unwrap_or("settings")
            );
            let title = argvus_i18n::tr(Lang::detect(), &key);
            if title == key {
              String::new()
            } else {
              title.into()
            }
          },
          category: if id.starts_with("shortcut:") {
            "help"
          } else {
            "settings"
          }
          .into(),
          icon: "preferences-system".into(),
          keywords,
          is_active: false,
          usage_id: String::new(),
          action: if id.starts_with("shortcut:") {
            Action::Help
          } else {
            Action::Settings(id.into())
          },
        })
      })
      .collect(),
  )
}

pub fn projects() -> Result<Vec<Entry>, String> {
  Ok(
    output(&["projects", "list", "--resolved"])?
      .lines()
      .filter(|line| !line.trim().is_empty())
      .map(|path| Entry {
        id: format!("project:{path}"),
        title: std::path::Path::new(path)
          .file_name()
          .unwrap_or_default()
          .to_string_lossy()
          .into_owned(),
        description: path.into(),
        category: "projects".into(),
        icon: "folder-development".into(),
        keywords: vec![],
        is_active: false,
        usage_id: String::new(),
        action: Action::Project(path.into()),
      })
      .collect(),
  )
}
pub fn ssh() -> Result<Vec<Entry>, String> {
  Ok(
    output(&["ssh", "list"])?
      .lines()
      .filter(|host| !host.is_empty())
      .map(|host| Entry {
        id: format!("ssh:{host}"),
        title: host.into(),
        description: tr(language(), "universal.ssh").into(),
        category: "ssh".into(),
        icon: "network-server".into(),
        keywords: vec!["ssh".into()],
        is_active: false,
        usage_id: String::new(),
        action: Action::Ssh(host.into()),
      })
      .collect(),
  )
}
pub fn builtin() -> Vec<Entry> {
  let lang = language();
  vec![
    Entry {
      id: "action:audio-outputs".into(),
      title: tr(lang, "universal.audio_outputs").into(),
      description: tr(lang, "universal.choose_device").into(),
      category: "actions".into(),
      icon: "audio-speakers".into(),
      keywords: tr(lang, "universal.intent_output")
        .split('|')
        .map(str::to_owned)
        .collect(),
      is_active: false,
      usage_id: String::new(),
      action: Action::AudioOutputs,
    },
    Entry {
      id: "help:shortcuts".into(),
      title: tr(lang, "universal.shortcuts").into(),
      description: tr(lang, "universal.context_help").into(),
      category: "help".into(),
      icon: "help-browser".into(),
      keywords: tr(lang, "universal.intent_help")
        .split('|')
        .map(str::to_owned)
        .collect(),
      is_active: false,
      usage_id: String::new(),
      action: Action::Help,
    },
  ]
}

pub fn audio_outputs() -> Result<Vec<Entry>, String> {
  let backend = audio_backend();
  let snapshot = backend.snapshot().map_err(|error| error.to_string())?;
  Ok(
    snapshot
      .outputs
      .into_iter()
      .map(|device| Entry {
        id: format!("audio:{}", device.id),
        title: device.description,
        description: device.name,
        category: "actions".into(),
        icon: "audio-speakers".into(),
        keywords: Vec::new(),
        is_active: false,
        usage_id: String::new(),
        action: Action::AudioOutput(device.id.to_string()),
      })
      .collect(),
  )
}
fn audio_backend() -> argvus_control_center_audio::backend::AudioBackend<
  argvus_control_center_core::process::SystemProcessRunner,
> {
  argvus_control_center_audio::backend::AudioBackend::new(
    argvus_control_center_core::process::SystemProcessRunner,
    argvus_control_center_core::capabilities::Capabilities::detect(),
  )
}

pub fn activate(action: &Action) -> Result<(), String> {
  let mut command = Command::new(argvus());
  match action {
    Action::Settings(id) => {
      command.args(["control-center", "--search-id", id]);
    }
    Action::Project(path) => {
      command.args(["projects", "open", path]);
    }
    Action::Ssh(host) => {
      command.args(["ssh", "open", host]);
    }
    Action::File(path) => {
      command = Command::new("xdg-open");
      command.arg(path);
    }
    Action::Configure => {
      command.args(["control-center", "launcher"]);
    }
    Action::Window(address) => {
      command.args(["windows", "activate", address]);
    }
    Action::Help => {
      command.args(["control-center", "shortcuts", "list"]);
    }
    Action::AudioOutput(id) => {
      // Revalidate the live device ID before applying a stale selection.
      if !audio_outputs()?.iter().any(|entry| entry.action == *action) {
        return Err("audio device disconnected".into());
      }
      return audio_backend()
        .set_default(id.parse().map_err(|_| "invalid audio device")?)
        .map_err(|error| error.to_string());
    }
    _ => return Err("activation belongs to the frontend".into()),
  }
  let status = command
    .stdout(std::process::Stdio::null())
    .stderr(std::process::Stdio::null())
    .stdin(std::process::Stdio::null())
    .status()
    .map_err(|e| e.to_string())?;
  if status.success() {
    Ok(())
  } else {
    Err(format!("action exited with {status}"))
  }
}
