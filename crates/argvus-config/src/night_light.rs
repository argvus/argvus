//! Canonical night-light preferences, serialized runtime actions and local scheduling.
use crate::{ConfigError, ConfigResult, ConfigStore};
use chrono::{DateTime, Local, TimeZone, Timelike, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::net::Shutdown;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[cfg(test)]
mod tests;

const POINTER: &str = "/displays/night_light";
pub const MIN_TEMPERATURE: u32 = 1000;
pub const MAX_TEMPERATURE: u32 = 20000;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Settings {
  pub enabled: bool,
  pub temperature: u32,
  pub schedule_enabled: bool,
  pub start: String,
  pub end: String,
}

impl Default for Settings {
  fn default() -> Self {
    Self {
      enabled: false,
      temperature: 4500,
      schedule_enabled: false,
      start: "20:00".into(),
      end: "07:00".into(),
    }
  }
}

fn invalid(message: impl Into<String>) -> ConfigError {
  ConfigError::Invalid(message.into())
}

impl Settings {
  pub(crate) fn from_value(value: &Value) -> ConfigResult<Self> {
    let settings: Self = serde_json::from_value(value.clone())
      .map_err(|error| invalid(format!("displays.night_light: {error}")))?;
    settings.validate()?;
    Ok(settings)
  }

  fn validate(&self) -> ConfigResult<()> {
    if !(MIN_TEMPERATURE..=MAX_TEMPERATURE).contains(&self.temperature) {
      return Err(invalid("night-light temperature must be 1000–20000 K"));
    }
    let start = minutes(&self.start)?;
    let end = minutes(&self.end)?;
    if start == end {
      return Err(invalid("night-light start and end must differ"));
    }
    Ok(())
  }

  fn active_at(&self, minute: u32) -> bool {
    // Settings are validated before they reach the scheduler.
    let start = minutes(&self.start).unwrap_or(1200);
    let end = minutes(&self.end).unwrap_or(420);
    if start < end {
      minute >= start && minute < end
    } else {
      minute >= start || minute < end
    }
  }
}

fn minutes(text: &str) -> ConfigResult<u32> {
  let bytes = text.as_bytes();
  if bytes.len() != 5
    || bytes[2] != b':'
    || !bytes
      .iter()
      .enumerate()
      .all(|(index, byte)| index == 2 || byte.is_ascii_digit())
  {
    return Err(invalid("night-light time must use HH:MM (00:00–23:59)"));
  }
  let hour = u32::from(bytes[0] - b'0') * 10 + u32::from(bytes[1] - b'0');
  let minute = u32::from(bytes[3] - b'0') * 10 + u32::from(bytes[4] - b'0');
  if hour > 23 || minute > 59 {
    return Err(invalid("night-light time must use HH:MM (00:00–23:59)"));
  }
  Ok(hour * 60 + minute)
}

/// Scan real instants using the current local zone. This handles both missing
/// and repeated wall-clock times at DST changes, without guessing their offset.
fn next_transition<T: TimeZone>(settings: &Settings, now: &DateTime<T>) -> Option<i64> {
  if !settings.schedule_enabled {
    return None;
  }
  let active = settings.active_at(now.hour() * 60 + now.minute());
  let first = now.timestamp().div_euclid(60) * 60 + 60;
  (0..(49 * 60))
    .map(|index| first + index * 60)
    .find(|timestamp| {
      now
        .timezone()
        .timestamp_opt(*timestamp, 0)
        .single()
        .is_some_and(|local| settings.active_at(local.hour() * 60 + local.minute()) != active)
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Override {
  active: bool,
  expires_at: i64,
  settings: Settings,
  instance: String,
}

impl Override {
  fn valid(&self, settings: &Settings, next: Option<i64>, instance: &str) -> bool {
    &self.settings == settings && Some(self.expires_at) == next && self.instance == instance
  }
}

#[derive(Debug, Clone, Serialize)]
pub struct Status {
  pub schema_version: u32,
  pub available: bool,
  pub unavailable_reason: Option<String>,
  pub settings: Settings,
  pub active: Option<bool>,
  pub effective_temperature: Option<u32>,
  pub mode: &'static str,
  pub manual_override: bool,
  pub next_transition: Option<String>,
  pub minimum_temperature: u32,
  pub maximum_temperature: u32,
}

trait Backend {
  fn state(&self) -> ConfigResult<(bool, u32)>;
  fn apply(&self, active: bool, temperature: u32) -> ConfigResult<()>;
}

struct Ipc;

fn request(payload: &str) -> ConfigResult<String> {
  let runtime =
    env::var_os("XDG_RUNTIME_DIR").ok_or_else(|| invalid("XDG_RUNTIME_DIR is missing"))?;
  let signature =
    env::var("HYPRLAND_INSTANCE_SIGNATURE").map_err(|_| invalid("Hyprland is not running"))?;
  let control = crate::hyprland_ipc::socket_path(Path::new(&runtime), &signature)?;
  if !control.exists()
    || !env::split_paths(&env::var_os("PATH").unwrap_or_default())
      .any(|directory| directory.join("hyprsunset").is_file())
  {
    return Err(invalid("Hyprland and hyprsunset are required"));
  }
  let path = control.with_file_name(".hyprsunset.sock");
  let mut stream = UnixStream::connect(path)?;
  stream.set_read_timeout(Some(Duration::from_secs(2)))?;
  stream.set_write_timeout(Some(Duration::from_secs(2)))?;
  stream.write_all(payload.as_bytes())?;
  // Half-close the request so Hyprsunset closes after its reply. Reading to
  // EOF then handles fragmented replies without waiting for another request.
  stream.shutdown(Shutdown::Write)?;
  let mut reply = String::new();
  stream.take(4097).read_to_string(&mut reply)?;
  if reply.len() > 4096 {
    return Err(invalid("hyprsunset reply is too large"));
  }
  Ok(reply.trim().into())
}

impl Backend for Ipc {
  fn state(&self) -> ConfigResult<(bool, u32)> {
    let identity = match request("identity get")?.as_str() {
      "true" => true,
      "false" => false,
      reply => return Err(invalid(format!("hyprsunset identity: {reply}"))),
    };
    let reply = request("temperature")?;
    let temperature = reply
      .parse::<u32>()
      .ok()
      .filter(|value| (MIN_TEMPERATURE..=MAX_TEMPERATURE).contains(value))
      .ok_or_else(|| invalid(format!("hyprsunset temperature: {reply}")))?;
    Ok((!identity, temperature))
  }

  fn apply(&self, active: bool, temperature: u32) -> ConfigResult<()> {
    let payload = if active {
      format!("temperature {temperature}")
    } else {
      "identity".into()
    };
    let reply = request(&payload)?;
    if reply != "ok" {
      return Err(invalid(format!("hyprsunset: {reply}")));
    }
    Ok(())
  }
}

fn apply_if_needed(backend: &impl Backend, active: bool, temperature: u32) -> ConfigResult<()> {
  let (current_active, current_temperature) = backend.state()?;
  if current_active != active || (active && current_temperature != temperature) {
    backend.apply(active, temperature)?;
    let (actual_active, actual_temperature) = backend.state()?;
    if actual_active != active || (active && actual_temperature != temperature) {
      return Err(invalid("hyprsunset did not apply the requested state"));
    }
  }
  Ok(())
}

fn runtime_directory() -> ConfigResult<PathBuf> {
  let runtime =
    env::var_os("XDG_RUNTIME_DIR").ok_or_else(|| invalid("XDG_RUNTIME_DIR is missing"))?;
  let runtime = PathBuf::from(runtime);
  if !runtime.is_absolute() {
    return Err(invalid("XDG_RUNTIME_DIR must be absolute"));
  }
  Ok(runtime.join("argvus/night-light"))
}

fn read_override(path: &Path) -> ConfigResult<Option<Override>> {
  match fs::read(path) {
    Ok(bytes) => serde_json::from_slice(&bytes)
      .map(Some)
      .map_err(|error| invalid(format!("night-light override: {error}"))),
    Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
    Err(error) => Err(error.into()),
  }
}

fn remove_override(path: &Path) -> ConfigResult<()> {
  match fs::remove_file(path) {
    Ok(()) => Ok(()),
    Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
    Err(error) => Err(error.into()),
  }
}

fn write_override(path: &Path, overlay: &Override) -> ConfigResult<()> {
  let temporary = path.with_extension("tmp");
  let mut file = OpenOptions::new()
    .write(true)
    .create(true)
    .truncate(true)
    .mode(0o600)
    .open(&temporary)?;
  serde_json::to_writer(&mut file, overlay).map_err(|error| invalid(error.to_string()))?;
  file.sync_all()?;
  fs::rename(temporary, path)?;
  Ok(())
}

fn load_settings(store: &ConfigStore) -> ConfigResult<Settings> {
  let document = store.load()?;
  document
    .get(POINTER)
    .map(Settings::from_value)
    .unwrap_or_else(|| Ok(Settings::default()))
}

fn execute(store: &ConfigStore, arguments: &[String]) -> ConfigResult<Option<Status>> {
  let operation = arguments.first().map(String::as_str).unwrap_or("status");
  let parameters = &arguments[usize::from(!arguments.is_empty())..];
  let valid_arguments = matches!(
    (operation, parameters),
    (
      "status" | "enable" | "disable" | "toggle" | "apply" | "apply-schedule",
      []
    ) | ("temperature", [_])
      | ("schedule", [_, _, _])
  );
  if !valid_arguments {
    return Err(invalid(
      "night-light: status|enable|disable|toggle|temperature K|schedule on|off HH:MM HH:MM|apply|apply-schedule",
    ));
  }
  let directory = match runtime_directory() {
    Ok(directory) => directory,
    Err(error) if operation == "status" => {
      let settings = load_settings(store)?;
      let next = next_transition(&settings, &Local::now());
      return Ok(Some(make_status(settings, false, next, Err(error))));
    }
    Err(error) => return Err(error),
  };
  fs::DirBuilder::new()
    .recursive(true)
    .mode(0o700)
    .create(&directory)?;
  let lock = OpenOptions::new()
    .read(true)
    .write(true)
    .create(true)
    .truncate(false)
    .mode(0o600)
    .open(directory.join("lock"))?;
  crate::config_lock::lock_file(&lock)?;
  let _guard = crate::config_lock::LockGuard(&lock);
  execute_locked(store, operation, parameters, &directory, &Ipc)
}

fn execute_locked(
  store: &ConfigStore,
  operation: &str,
  parameters: &[String],
  directory: &Path,
  backend: &impl Backend,
) -> ConfigResult<Option<Status>> {
  let mut settings = load_settings(store)?;
  if operation == "apply-schedule" && !settings.schedule_enabled {
    return Ok(None);
  }
  let now = Local::now();
  let mut next = next_transition(&settings, &now);
  let instance = env::var("HYPRLAND_INSTANCE_SIGNATURE").unwrap_or_default();
  let override_path = directory.join("override.json");
  let mut overlay =
    read_override(&override_path)?.filter(|overlay| overlay.valid(&settings, next, &instance));
  if operation != "status" && overlay.is_none() {
    remove_override(&override_path)?;
  }
  let state = backend.state();
  if operation != "status" {
    let (current_active, _) = state.as_ref().map_err(|error| invalid(error.to_string()))?;
    let mut persist = false;
    match operation {
      "enable" | "disable" | "toggle" => {
        let active = match operation {
          "enable" => true,
          "disable" => false,
          _ => !current_active,
        };
        if settings.schedule_enabled {
          overlay = Some(Override {
            active,
            expires_at: next.ok_or_else(|| invalid("no next transition"))?,
            settings: settings.clone(),
            instance,
          });
          // Keep the previous override if IPC fails.
          apply_if_needed(backend, active, settings.temperature)?;
          if let Some(overlay) = &overlay {
            write_override(&override_path, overlay)?;
          }
        } else {
          settings.enabled = active;
          persist = true;
        }
      }
      "temperature" => {
        settings.temperature = parameters[0]
          .parse()
          .map_err(|_| invalid("temperature must be an integer"))?;
        persist = true;
      }
      "schedule" => {
        settings.schedule_enabled = match parameters[0].as_str() {
          "on" => true,
          "off" => false,
          _ => return Err(invalid("schedule must be on or off")),
        };
        settings.start.clone_from(&parameters[1]);
        settings.end.clone_from(&parameters[2]);
        persist = true;
      }
      _ => {}
    }
    if persist {
      settings.validate()?;
      store.update(
        POINTER,
        serde_json::to_value(&settings).map_err(|error| invalid(error.to_string()))?,
      )?;
      overlay = None;
      remove_override(&override_path)?;
      next = next_transition(&settings, &now);
    }
    let active = overlay
      .as_ref()
      .map(|overlay| overlay.active)
      .unwrap_or_else(|| {
        if settings.schedule_enabled {
          settings.active_at(now.hour() * 60 + now.minute())
        } else {
          settings.enabled
        }
      });
    apply_if_needed(backend, active, settings.temperature)?;
  }
  let effective = if operation == "status" {
    state
  } else {
    backend.state()
  };
  Ok(Some(make_status(
    settings,
    overlay.is_some(),
    next,
    effective,
  )))
}

fn make_status(
  settings: Settings,
  manual_override: bool,
  next: Option<i64>,
  effective: ConfigResult<(bool, u32)>,
) -> Status {
  let (available, active, temperature, reason) = match effective {
    Ok((active, temperature)) => (true, Some(active), active.then_some(temperature), None),
    Err(error) => (false, None, None, Some(error.to_string())),
  };
  Status {
    schema_version: 1,
    available,
    unavailable_reason: reason,
    mode: if settings.schedule_enabled {
      "scheduled"
    } else {
      "manual"
    },
    settings,
    active,
    effective_temperature: temperature,
    manual_override,
    next_transition: next
      .and_then(|timestamp| Utc.timestamp_opt(timestamp, 0).single())
      .map(|time| time.with_timezone(&Local).to_rfc3339()),
    minimum_temperature: MIN_TEMPERATURE,
    maximum_temperature: MAX_TEMPERATURE,
  }
}

pub fn run(store: &ConfigStore, arguments: &[String]) -> i32 {
  match execute(store, arguments) {
    Ok(Some(status)) => match serde_json::to_string(&status) {
      Ok(json) => {
        println!("{json}");
        0
      }
      Err(error) => {
        eprintln!("night-light: {error}");
        1
      }
    },
    Ok(None) => 0,
    Err(error) => {
      crate::i18n::print_in(
        "display",
        "night_light.error",
        &[format!("error={error}")],
        crate::i18n::Stream::Stderr,
      );
      1
    }
  }
}
