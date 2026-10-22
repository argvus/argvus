//! Volume and mute of the default output sink.
//!
//! The Waybar volume module, the Hyprland media keys and the Control Panel all
//! change the volume. Each change is applied to PipeWire through `wpctl`, and
//! the value PipeWire reports afterwards is recorded in the canonical store
//! (`/audio/output_volume` and `/audio/output_muted`). Without the second step
//! the store keeps an old value, which `argvus config project` writes back onto
//! the sink and which the Control Panel would show.

use crate::ConfigStore;
use serde_json::{Map, Value};
use std::process::{Command, Stdio};

/// The sink every audio command targets.
const DEFAULT_SINK: &str = "@DEFAULT_AUDIO_SINK@";
/// Highest volume `wpctl` may set when the caller gives no `--limit`.
const DEFAULT_LIMIT_PERCENT: u32 = 100;
/// The store keeps a percentage between 0 and 100, whatever the sink allows.
const STORE_MAX_PERCENT: u32 = 100;
const VOLUME_POINTER: &str = "/audio/output_volume";
const MUTE_POINTER: &str = "/audio/output_muted";
const USAGE: &str =
  "usage: argvus config audio volume <N|+N|-N> [--limit PERCENT] | mute <on|off|toggle>";

/// A requested change of the sink volume, in percentage points.
#[derive(Debug, PartialEq, Eq)]
enum VolumeChange {
  Set(u32),
  Raise(u32),
  Lower(u32),
}

/// A requested change of the sink mute state.
#[derive(Debug, PartialEq, Eq)]
enum MuteChange {
  On,
  Off,
  Toggle,
}

/// What PipeWire reports for the sink after a change.
#[derive(Debug, PartialEq, Eq)]
struct SinkState {
  percent: u32,
  is_muted: bool,
}

fn parse_percent(digits: &str) -> Option<u32> {
  if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
    return None;
  }
  digits.parse().ok()
}

/// Accepts `45`, `+5` and `-5`, each with an optional trailing `%`.
fn parse_volume_change(text: &str) -> Option<VolumeChange> {
  let text = text.strip_suffix('%').unwrap_or(text);
  if let Some(digits) = text.strip_prefix('+') {
    return parse_percent(digits).map(VolumeChange::Raise);
  }
  if let Some(digits) = text.strip_prefix('-') {
    return parse_percent(digits).map(VolumeChange::Lower);
  }
  parse_percent(text).map(VolumeChange::Set)
}

fn parse_mute_change(text: &str) -> Option<MuteChange> {
  match text {
    "on" => Some(MuteChange::On),
    "off" => Some(MuteChange::Off),
    "toggle" => Some(MuteChange::Toggle),
    _ => None,
  }
}

/// Reads `--limit PERCENT` from the arguments after the volume value.
fn parse_limit(arguments: &[String]) -> Result<u32, String> {
  let Some(position) = arguments.iter().position(|argument| argument == "--limit") else {
    return Ok(DEFAULT_LIMIT_PERCENT);
  };
  arguments
    .get(position + 1)
    .and_then(|value| parse_percent(value))
    .ok_or_else(|| USAGE.to_string())
}

/// Parses the `wpctl get-volume` line, for example `Volume: 0.45 [MUTED]`.
fn parse_sink_state(output: &str) -> Option<SinkState> {
  let mut words = output.split_whitespace();
  if words.next()? != "Volume:" {
    return None;
  }
  let fraction: f64 = words.next()?.parse().ok()?;
  if !fraction.is_finite() || fraction < 0.0 {
    return None;
  }
  Some(SinkState {
    percent: (fraction * 100.0).round() as u32,
    is_muted: words.any(|word| word == "[MUTED]"),
  })
}

/// The `wpctl set-volume` arguments. `-l` keeps a raise below the limit.
fn volume_arguments(change: &VolumeChange, limit_percent: u32) -> Vec<String> {
  let limit = format!("{:.2}", f64::from(limit_percent) / 100.0);
  let amount = match change {
    VolumeChange::Set(percent) => format!("{percent}%"),
    VolumeChange::Raise(percent) => format!("{percent}%+"),
    VolumeChange::Lower(percent) => format!("{percent}%-"),
  };
  vec![
    "set-volume".to_string(),
    "-l".to_string(),
    limit,
    DEFAULT_SINK.to_string(),
    amount,
  ]
}

fn mute_arguments(change: &MuteChange) -> Vec<String> {
  let value = match change {
    MuteChange::On => "1",
    MuteChange::Off => "0",
    MuteChange::Toggle => "toggle",
  };
  vec![
    "set-mute".to_string(),
    DEFAULT_SINK.to_string(),
    value.to_string(),
  ]
}

fn run_wpctl(arguments: &[String]) -> Result<String, String> {
  let output = Command::new("wpctl")
    .args(arguments)
    .stdin(Stdio::null())
    .stderr(Stdio::inherit())
    .output()
    .map_err(|error| format!("cannot run wpctl: {error}"))?;
  if !output.status.success() {
    return Err(format!("wpctl {} failed: {}", arguments[0], output.status));
  }
  Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn read_sink_state() -> Result<SinkState, String> {
  let output = run_wpctl(&["get-volume".to_string(), DEFAULT_SINK.to_string()])?;
  parse_sink_state(&output).ok_or_else(|| format!("unexpected wpctl output: {}", output.trim()))
}

/// Records the sink state in the store. A boost above 100% is stored as 100%,
/// the highest value the store accepts.
fn persist_sink_state(
  store: &ConfigStore,
  state: &SinkState,
) -> Result<(), Box<dyn std::error::Error>> {
  let mut values = Map::new();
  values.insert(
    VOLUME_POINTER.to_string(),
    Value::from(state.percent.min(STORE_MAX_PERCENT)),
  );
  values.insert(MUTE_POINTER.to_string(), Value::Bool(state.is_muted));
  store.patch(&values)?;
  Ok(())
}

/// Entry point of `argvus config audio …`.
///
/// # Errors
///
/// Returns an error for unknown arguments, when `wpctl` fails or reports an
/// unexpected line, or when the store cannot be written. The sink keeps the
/// requested change in the last case.
pub fn run(store: &ConfigStore, arguments: &[String]) -> Result<(), Box<dyn std::error::Error>> {
  match arguments.first().map(String::as_str) {
    Some("volume") => {
      let change = arguments
        .get(1)
        .and_then(|value| parse_volume_change(value))
        .ok_or(USAGE)?;
      let limit_percent = parse_limit(&arguments[2..])?;
      run_wpctl(&volume_arguments(&change, limit_percent))?;
    }
    Some("mute") => {
      let change = arguments
        .get(1)
        .and_then(|value| parse_mute_change(value))
        .ok_or(USAGE)?;
      run_wpctl(&mute_arguments(&change))?;
    }
    _ => return Err(USAGE.into()),
  }
  let state = read_sink_state()?;
  persist_sink_state(store, &state)?;
  println!("volume={} muted={}", state.percent, state.is_muted);
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parses_absolute_and_relative_volume_changes() {
    assert_eq!(parse_volume_change("45"), Some(VolumeChange::Set(45)));
    assert_eq!(parse_volume_change("45%"), Some(VolumeChange::Set(45)));
    assert_eq!(parse_volume_change("+5"), Some(VolumeChange::Raise(5)));
    assert_eq!(parse_volume_change("-5%"), Some(VolumeChange::Lower(5)));
  }

  #[test]
  fn rejects_malformed_volume_changes() {
    for text in ["", "+", "-", "abc", "5.5", "++3", "--3", "+-3", "%"] {
      assert_eq!(parse_volume_change(text), None, "input: {text:?}");
    }
  }

  #[test]
  fn parses_mute_changes() {
    assert_eq!(parse_mute_change("on"), Some(MuteChange::On));
    assert_eq!(parse_mute_change("off"), Some(MuteChange::Off));
    assert_eq!(parse_mute_change("toggle"), Some(MuteChange::Toggle));
    assert_eq!(parse_mute_change("1"), None);
  }

  #[test]
  fn limit_defaults_to_the_store_maximum_and_reads_the_flag() {
    assert_eq!(parse_limit(&[]), Ok(DEFAULT_LIMIT_PERCENT));
    let arguments = ["--limit".to_string(), "150".to_string()];
    assert_eq!(parse_limit(&arguments), Ok(150));
    let missing_value = ["--limit".to_string()];
    assert!(parse_limit(&missing_value).is_err());
  }

  #[test]
  fn reads_volume_and_mute_from_the_wpctl_line() {
    assert_eq!(
      parse_sink_state("Volume: 0.45\n"),
      Some(SinkState {
        percent: 45,
        is_muted: false
      })
    );
    assert_eq!(
      parse_sink_state("Volume: 1.00 [MUTED]\n"),
      Some(SinkState {
        percent: 100,
        is_muted: true
      })
    );
    assert_eq!(
      parse_sink_state("Volume: 0.456").map(|state| state.percent),
      Some(46)
    );
  }

  #[test]
  fn rejects_unexpected_wpctl_output() {
    for output in ["", "Volume:", "Volume: loud", "Volume: -0.2", "Mute: 0.5"] {
      assert_eq!(parse_sink_state(output), None, "output: {output:?}");
    }
  }

  #[test]
  fn builds_wpctl_arguments_with_the_volume_limit() {
    assert_eq!(
      volume_arguments(&VolumeChange::Raise(2), 100),
      ["set-volume", "-l", "1.00", "@DEFAULT_AUDIO_SINK@", "2%+"]
    );
    assert_eq!(
      volume_arguments(&VolumeChange::Lower(5), 150),
      ["set-volume", "-l", "1.50", "@DEFAULT_AUDIO_SINK@", "5%-"]
    );
    assert_eq!(
      volume_arguments(&VolumeChange::Set(30), 100)[4],
      "30%".to_string()
    );
    assert_eq!(
      mute_arguments(&MuteChange::Toggle),
      ["set-mute", "@DEFAULT_AUDIO_SINK@", "toggle"]
    );
  }
}
