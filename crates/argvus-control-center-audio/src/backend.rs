//! Implements isolated integration with system tools and APIs in crate `argvus control center audio`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use crate::model::*;
use argvus_control_center_core::{
  capabilities::Capabilities,
  process::{ProcessError, ProcessRequest, ProcessRunner},
  sanitize::terminal_text,
};
use std::time::Duration;
use thiserror::Error;
#[derive(Debug, Error)]
/// Defines `AudioError`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub enum AudioError {
  #[error("audio backend unavailable")]
  BackendUnavailable,
  #[error("process failed: {0}")]
  Process(#[from] ProcessError),
  #[error("audio command failed: {0}")]
  Command(String),
  #[error("invalid audio value")]
  InvalidValue,
}
/// Represents `AudioBackend`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct AudioBackend<R> {
  pub runner: R,
  pub capabilities: Capabilities,
}
impl<R: ProcessRunner> AudioBackend<R> {
  /// Constructs `new` with this module's expected initial state. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn new(runner: R, capabilities: Capabilities) -> Self {
    Self {
      runner,
      capabilities,
    }
  }
  /// Executes the `run` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn run(&self, args: &[&str]) -> Result<String, AudioError> {
    let req = args
      .iter()
      .fold(ProcessRequest::new("wpctl"), |r, a| r.arg(*a))
      .timeout(Duration::from_secs(5));
    let o = self.runner.run(&req)?;
    if o.timed_out {
      return Err(AudioError::Command("timeout".into()));
    }
    if o.status != Some(0) {
      return Err(AudioError::Command(terminal_text(
        &String::from_utf8_lossy(&o.stderr),
      )));
    }
    Ok(terminal_text(&String::from_utf8_lossy(&o.stdout)))
  }

  fn persist_default_output(&self, pointer: &str, value: &str) -> Result<(), AudioError> {
    let request = ProcessRequest::new("argvus")
      .arg("config")
      .arg("set")
      .arg(pointer)
      .arg(value)
      .timeout(Duration::from_secs(60));
    let output = match self.runner.run(&request) {
      Ok(output) => output,
      Err(ProcessError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
      Err(error) => return Err(AudioError::Process(error)),
    };
    if output.status != Some(0) {
      return Err(AudioError::Command(terminal_text(
        &String::from_utf8_lossy(&output.stderr),
      )));
    }
    argvus_control_center_core::config::reload_argvus_config_service_with(&self.runner)
      .map_err(AudioError::Command)
  }
  /// Retrieves data for `snapshot` without mixing collection with TUI rendering. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn snapshot(&self) -> Result<AudioSnapshot, AudioError> {
    if !self.capabilities.has_wpctl {
      return Err(AudioError::BackendUnavailable);
    }
    let text = self.run(&["status"])?;
    let devices = parse_status(&text);
    let default_output = self.default_id("@DEFAULT_AUDIO_SINK@");
    let default_input = self.default_id("@DEFAULT_AUDIO_SOURCE@");
    Ok(AudioSnapshot {
      available: true,
      backend: "PipeWire/WirePlumber".into(),
      outputs: devices
        .iter()
        .filter(|d| d.direction == "output")
        .cloned()
        .collect(),
      inputs: devices
        .iter()
        .filter(|d| d.direction == "input")
        .cloned()
        .collect(),
      default_output,
      default_input,
    })
  }
  /// Executes the `default_id` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn default_id(&self, target: &str) -> Option<u32> {
    self.run(&["inspect", target]).ok().and_then(|v| {
      v.lines().find_map(|l| {
        l.trim()
          .strip_prefix("id ")
          .and_then(|v| v.split(',').next())
          .map(str::trim)
          .and_then(|v| v.parse().ok())
      })
    })
  }
  /// Applies the `set_default` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn set_default(&self, id: u32) -> Result<(), AudioError> {
    validate_node_id(id)?;
    self.run(&["set-default", &id.to_string()]).map(|_| ())
  }
  /// Applies the `set_volume` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn set_volume(&self, id: u32, percent: u8) -> Result<(), AudioError> {
    validate_node_id(id)?;
    if percent > 100 {
      return Err(AudioError::InvalidValue);
    }
    self.run(&["set-volume", &id.to_string(), &format!("{}%", percent)])?;
    self.persist_default_output("/audio/output_volume", &percent.to_string())
  }
  /// Applies the `set_mute` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn set_mute(&self, id: u32, mute: bool) -> Result<(), AudioError> {
    validate_node_id(id)?;
    self.run(&["set-mute", &id.to_string(), if mute { "1" } else { "0" }])?;
    self.persist_default_output("/audio/output_muted", if mute { "true" } else { "false" })
  }
}
/// Executes the `validate_node_id` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn validate_node_id(id: u32) -> Result<(), AudioError> {
  (id > 0).then_some(()).ok_or(AudioError::InvalidValue)
}
/// Converts input data into `parse_status` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn parse_status(input: &str) -> Vec<AudioDevice> {
  let mut section = None;
  input
    .lines()
    .filter_map(|line| {
      let trimmed = line.trim();
      if trimmed.ends_with("Sinks:") {
        section = Some("output");
        return None;
      }
      if trimmed.ends_with("Sources:") {
        section = Some("input");
        return None;
      }
      if trimmed.ends_with(':') || (!line.starts_with(' ') && !line.starts_with('\t')) {
        section = None;
        return None;
      }
      let direction = section?;
      let t = trimmed
        .trim_start_matches(['│', '├', '└', '─', ' '])
        .trim_start();
      let is_default = t.starts_with('*');
      let t = t.trim_start_matches('*').trim_start();
      let (id, rest) = t.split_once('.')?;
      let id: u32 = id.trim().parse().ok()?;
      if id == 0 {
        return None;
      }
      let raw = rest.trim();
      let vol_bracket = raw.find("[vol: ").map(|start| {
        let tail = &raw[start + 6..];
        tail
          .split_once(']')
          .map(|(inside, _)| inside)
          .unwrap_or(tail)
      });
      let volume = vol_bracket
        .and_then(|inside| inside.split_whitespace().next())
        .and_then(|v| v.parse::<f32>().ok())
        .map(|value| clamp_volume((value * 100.0).round() as i16));
      let muted = raw.contains("[MUTED]")
        || vol_bracket.is_some_and(|inside| inside.split_whitespace().any(|t| t == "MUTED"));
      let name = raw.split(" [vol:").next().unwrap_or(raw).trim().to_string();
      if name.is_empty() {
        return None;
      }
      Some(AudioDevice {
        id,
        name: name.clone(),
        description: name,
        direction: direction.into(),
        volume,
        muted,
        is_default,
      })
    })
    .collect()
}
/// Executes the `clamp_volume` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn clamp_volume(value: i16) -> u8 {
  value.clamp(0, 100) as u8
}
#[cfg(test)]
mod tests {
  use super::*;
  use argvus_control_center_core::process::{ProcessOutput, ProcessRequest};
  use std::sync::Mutex;

  #[derive(Default)]
  /// Represents `RecordingRunner`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
  struct RecordingRunner(Mutex<Vec<ProcessRequest>>);
  impl ProcessRunner for RecordingRunner {
    /// Executes the `run` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
    fn run(&self, request: &ProcessRequest) -> Result<ProcessOutput, ProcessError> {
      self.0.lock().unwrap().push(request.clone());
      Ok(ProcessOutput {
        stdout: Vec::new(),
        stderr: Vec::new(),
        status: Some(0),
        timed_out: false,
      })
    }
  }
  #[test]
  /// Converts input data into `parses_devices` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn parses_devices() {
    let x = parse_status(
      "Audio\n ├─ Sinks:\n │  *   42. Built-in Audio [vol: 0.72]\n ├─ Sources:\n │      43. USB Headset [vol: 0.40] [MUTED]\nSettings\n └─ Default Configured Devices:\n         0. Audio/Sink alsa_output.pci",
    );
    assert_eq!(x.len(), 2);
    assert_eq!(x[0].id, 42);
    assert_eq!(x[0].volume, Some(72));
    assert_eq!(x[0].direction, "output");
    assert!(x[0].is_default);
    assert!(x[1].muted);
  }
  #[test]
  /// Executes the `ignores_headers_clients_streams_and_configured_default_ids` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn ignores_headers_clients_streams_and_configured_default_ids() {
    let x = parse_status(
      "PipeWire 'pipewire-0'\n └─ Clients:\n        32. WirePlumber\nAudio\n ├─ Devices:\n │      48. Audio Controller [alsa]\n ├─ Sinks:\n │  *   46. USB Audio Device [vol: 1.00]\n └─ Streams:\n        84. Browser\nSettings\n └─ Default Configured Devices:\n         0. Audio/Sink alsa_output.usb\n         1. Audio/Source alsa_input.usb",
    );
    assert_eq!(x.iter().map(|device| device.id).collect::<Vec<_>>(), [46]);
    assert!(x.iter().all(|device| device.id > 0));
  }
  #[test]
  /// Executes the `actions_reject_zero_node_id` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn actions_reject_zero_node_id() {
    let backend = AudioBackend::new(
      argvus_control_center_core::process::SystemProcessRunner,
      Capabilities::default(),
    );
    assert!(matches!(
      backend.set_default(0),
      Err(AudioError::InvalidValue)
    ));
    assert!(matches!(
      backend.set_volume(0, 50),
      Err(AudioError::InvalidValue)
    ));
    assert!(matches!(
      backend.set_mute(0, true),
      Err(AudioError::InvalidValue)
    ));
  }
  #[test]
  /// Executes the `audio_actions_use_separate_valid_wpctl_arguments` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn audio_actions_use_separate_valid_wpctl_arguments() {
    let backend = AudioBackend::new(RecordingRunner::default(), Capabilities::default());
    backend.set_default(46).unwrap();
    backend.set_volume(46, 75).unwrap();
    backend.set_mute(57, true).unwrap();
    let requests = backend.runner.0.lock().unwrap();
    let wpctl_requests: Vec<_> = requests
      .iter()
      .filter(|request| request.program == "wpctl")
      .collect();
    assert_eq!(wpctl_requests[0].args, ["set-default", "46"]);
    assert_eq!(wpctl_requests[1].args, ["set-volume", "46", "75%"]);
    assert_eq!(wpctl_requests[2].args, ["set-mute", "57", "1"]);
  }
  #[test]
  /// Executes the `volume_is_bounded` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn volume_is_bounded() {
    assert_eq!(clamp_volume(-2), 0);
    assert_eq!(clamp_volume(140), 100)
  }
  #[test]
  /// Converts input data into `parses_real_wpctl_status_shape` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn parses_real_wpctl_status_shape() {
    let real = concat!(
      "PipeWire 'pipewire-0' [1.6.8, boss@archlinux, cookie:505526840]\n",
      " └─ Clients:\n",
      "        32. xdg-desktop-portal                  [1.6.8, boss@archlinux, pid:3375]\n",
      "Audio\n",
      " ├─ Devices:\n",
      " │      48. GM107 High Definition Audio Controller [GeForce 940MX] [alsa]\n",
      " │      49. USB Audio Device                    [alsa]\n",
      " ├─ Sinks:\n",
      " │  *   46. USB Audio Device Estéreo analógico [vol: 0.05 MUTED]\n",
      " │      58. GM107 High Definition Audio Controller [GeForce 940MX] Estéreo digital (HDMI) [vol: 0.05 MUTED]\n",
      " ├─ Sources:\n",
      " │      57. USB Audio Device Mono               [vol: 1.00 MUTED]\n",
      " │  *   59. Áudio interno Estéreo analógico  [vol: 0.05 MUTED]\n",
      " ├─ Filters:\n",
      " │  \n",
      " └─ Streams:\n",
      "        84. speech-dispatcher-dummy\n",
      "             85. output_FL       > USB Audio Device:playback_FL\t[init]\n",
      "Video\n",
      " ├─ Devices:\n",
      "Settings\n",
      " └─ Default Configured Devices:\n",
      "         0. Audio/Sink    alsa_output.usb-C-Media_Electronics_Inc._USB_Audio_Device-00.analog-stereo\n",
      "         1. Audio/Source  alsa_input.pci-0000_00_1f.3.analog-stereo\n",
    );
    let parsed = parse_status(real);
    let ids: Vec<u32> = parsed.iter().map(|device| device.id).collect();
    assert_eq!(ids, [46, 58, 57, 59]);
    assert!(parsed.iter().all(|device| device.id > 0));
    assert_eq!(parsed[0].direction, "output");
    assert!(parsed[0].is_default);
    assert_eq!(parsed[0].volume, Some(5));
    assert!(parsed[0].muted);
    assert_eq!(parsed[1].direction, "output");
    assert!(!parsed[1].is_default);
    assert_eq!(parsed[2].direction, "input");
    assert!(parsed[3].is_default);
    assert!(
      parsed
        .iter()
        .all(|device| !device.name.starts_with("Audio/Sink"))
    );
  }
  #[test]
  /// Executes the `default_id_extracts_digit_before_comma_in_inspect_first_line` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn default_id_extracts_digit_before_comma_in_inspect_first_line() {
    /// Represents `InspectFixture`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
    struct InspectFixture;
    impl ProcessRunner for InspectFixture {
      /// Executes the `run` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
      fn run(&self, _: &ProcessRequest) -> Result<ProcessOutput, ProcessError> {
        Ok(ProcessOutput {
          stdout: b"id 62, type PipeWire:Interface:Node\n\tnode.name = \"alsa_output.usb\"\n"
            .to_vec(),
          stderr: Vec::new(),
          status: Some(0),
          timed_out: false,
        })
      }
    }
    let backend = AudioBackend::new(InspectFixture, Capabilities::default());
    assert_eq!(backend.default_id("@DEFAULT_AUDIO_SINK@"), Some(62));
  }
}
