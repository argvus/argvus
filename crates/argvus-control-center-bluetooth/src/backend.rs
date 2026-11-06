//! Isolated integration with system tools and APIs of the Bluetooth page.
use crate::model::*;
use argvus_control_center_core::{
  capabilities::Capabilities,
  process::{ProcessError, ProcessRequest, ProcessRunner},
  sanitize::terminal_text,
};
use std::collections::HashMap;
use std::time::Duration;
use thiserror::Error;
use zbus::zvariant::{OwnedObjectPath, OwnedValue};
#[derive(Debug, Error)]
pub enum BluetoothError {
  #[error("bluetooth backend unavailable")]
  BackendUnavailable,
  #[error("process failed: {0}")]
  Process(#[from] ProcessError),
  #[error("bluetooth command failed: {0}")]
  Command(String),
  #[error("invalid bluetooth address")]
  InvalidAddress,
}
pub struct BluetoothBackend<R> {
  pub runner: R,
  pub capabilities: Capabilities,
}
impl<R: ProcessRunner> BluetoothBackend<R> {
  pub fn new(runner: R, capabilities: Capabilities) -> Self {
    Self {
      runner,
      capabilities,
    }
  }
  fn run(&self, args: &[&str]) -> Result<String, BluetoothError> {
    self.run_with_timeout(args, Duration::from_secs(20))
  }
  fn run_with_timeout(&self, args: &[&str], timeout: Duration) -> Result<String, BluetoothError> {
    if !self.capabilities.has_bluetoothctl {
      return Err(BluetoothError::BackendUnavailable);
    }
    let req = args
      .iter()
      .fold(ProcessRequest::new("bluetoothctl"), |r, a| r.arg(*a))
      .timeout(timeout);
    let o = self.runner.run(&req)?;
    if o.timed_out {
      return Err(BluetoothError::Command("timeout".into()));
    }
    if o.status != Some(0) {
      return Err(BluetoothError::Command(terminal_text(
        &String::from_utf8_lossy(&o.stderr),
      )));
    }
    Ok(terminal_text(&String::from_utf8_lossy(&o.stdout)))
  }
  pub fn snapshot(&self) -> Result<BluetoothSnapshot, BluetoothError> {
    let show = match self.run(&["show"]) {
      Ok(value) => value,
      Err(BluetoothError::Command(message))
        if message
          .to_ascii_lowercase()
          .contains("no default controller") =>
      {
        String::new()
      }
      Err(error) => return Err(error),
    };
    let adapter = parse_adapter(&show);
    let mut devices = parse_devices(&self.run(&["devices"]).unwrap_or_default());
    // Keep bluetoothctl load bounded while avoiding one full timeout per device.
    for batch in devices.chunks_mut(4) {
      std::thread::scope(|scope| {
        for device in batch {
          scope.spawn(move || {
            if let Ok(info) =
              self.run_with_timeout(&["info", &device.address], Duration::from_secs(5))
            {
              merge_device_info(device, &info);
            }
          });
        }
      });
    }
    Ok(BluetoothSnapshot {
      available: true,
      adapter,
      devices,
      discovering: show.lines().any(|line| line.trim() == "Discovering: yes"),
    })
  }
  pub fn action(&self, address: &str, action: &str) -> Result<(), BluetoothError> {
    validate_address(address)?;
    if !matches!(
      action,
      "pair" | "trust" | "untrust" | "connect" | "disconnect" | "remove"
    ) {
      return Err(BluetoothError::Command("unsupported action".into()));
    }
    self.run(&[action, address]).map(|_| ())
  }
  pub fn pair(&self, address: &str, via_dbus: bool) -> Result<(), BluetoothError> {
    validate_address(address)?;
    if via_dbus {
      return pair_device_via_dbus(address);
    }
    self.run(&["pair", address]).map(|_| ())
  }
  pub fn power(&self, on: bool) -> Result<(), BluetoothError> {
    self
      .run(&["power", if on { "on" } else { "off" }])
      .map(|_| ())
  }
  pub fn scan(&self, on: bool) -> Result<(), BluetoothError> {
    self
      .run(&["scan", if on { "on" } else { "off" }])
      .map(|_| ())
  }
  pub fn discoverable(&self, on: bool) -> Result<(), BluetoothError> {
    self
      .run(&["discoverable", if on { "on" } else { "off" }])
      .map(|_| ())
  }
}
const BLUEZ: &str = "org.bluez";
const OBJECT_MANAGER: &str = "org.freedesktop.DBus.ObjectManager";
const DEVICE_INTERFACE: &str = "org.bluez.Device1";
const PAIR_TIMEOUT: Duration = Duration::from_secs(300);

pub fn pair_device_via_dbus(address: &str) -> Result<(), BluetoothError> {
  validate_address(address)?;
  let connection = match zbus::blocking::connection::Builder::system() {
    Ok(builder) => builder.method_timeout(PAIR_TIMEOUT).build(),
    Err(error) => return Err(pair_error(error)),
  }
  .map_err(pair_error)?;
  let path = find_device(&connection, address)?;
  connection
    .call_method(Some(BLUEZ), path, Some(DEVICE_INTERFACE), "Pair", &())
    .map_err(pair_error)?;
  Ok(())
}

fn find_device(
  connection: &zbus::blocking::Connection,
  address: &str,
) -> Result<OwnedObjectPath, BluetoothError> {
  let reply = connection
    .call_method(
      Some(BLUEZ),
      "/",
      Some(OBJECT_MANAGER),
      "GetManagedObjects",
      &(),
    )
    .map_err(pair_error)?;
  let objects: HashMap<OwnedObjectPath, HashMap<String, HashMap<String, OwnedValue>>> = reply
    .body()
    .deserialize()
    .map_err(|error| BluetoothError::Command(format!("D-Bus: {error}")))?;
  let expected = address.to_ascii_uppercase();
  objects
    .into_iter()
    .find(|(_, interfaces)| {
      interfaces
        .get(DEVICE_INTERFACE)
        .and_then(|properties| properties.get("Address"))
        .and_then(|value| value.downcast_ref::<String>().ok())
        .is_some_and(|value| value == expected)
    })
    .map(|(path, _)| path)
    .ok_or_else(|| BluetoothError::Command("device not found on the bus".into()))
}

fn pair_error(error: zbus::Error) -> BluetoothError {
  match error {
    zbus::Error::MethodError(name, message, _) => BluetoothError::Command(normalize_pair_error(
      name.as_str(),
      message.as_deref().unwrap_or_default(),
    )),
    zbus::Error::InputOutput(input) if input.kind() == std::io::ErrorKind::TimedOut => {
      BluetoothError::Command("pairing timed out".into())
    }
    other => BluetoothError::Command(format!("D-Bus: {other}")),
  }
}

fn normalize_pair_error(name: &str, message: &str) -> String {
  let label = match name.rsplit('.').next().unwrap_or("") {
    "AlreadyPaired" | "AlreadyExists" => "device is already paired",
    "Canceled" | "Rejected" => "pairing canceled or denied by the user",
    "AuthenticationFailed" | "AuthenticationRejected" => {
      "authentication failed; check the PIN or passkey"
    }
    "AuthenticationTimeout" | "Timeout" => "pairing timed out",
    "Failed" => "pairing failed",
    "NotAuthorized" => "pairing is not authorized",
    "InProgress" => "pairing is already in progress",
    "NotSupported" => "pairing is not supported by the device",
    "DoesNotExist" | "NotAvailable" => "no pairing agent is available",
    _ => "",
  };
  if !label.is_empty() {
    return label.to_string();
  }
  let rendered = terminal_text(message);
  if rendered.is_empty() {
    format!("pairing failed ({name})")
  } else {
    rendered
  }
}

pub fn parse_adapter(input: &str) -> Option<Adapter> {
  let mut a = Adapter::default();
  for line in input.lines() {
    let l = line.trim();
    if let Some(v) = l.strip_prefix("Controller ") {
      a.address = v.split_whitespace().next()?.into();
      a.name = v
        .split_once(' ')
        .map(|(_, n)| n.to_string())
        .unwrap_or_default()
    } else if let Some(v) = l.strip_prefix("Powered: ") {
      a.powered = v == "yes"
    } else if let Some(v) = l.strip_prefix("Discoverable: ") {
      a.discoverable = v == "yes"
    } else if let Some(v) = l.strip_prefix("Pairable: ") {
      a.pairable = v == "yes"
    }
  }
  (!a.address.is_empty()).then_some(a)
}
pub fn parse_devices(input: &str) -> Vec<Device> {
  input
    .lines()
    .filter_map(|line| {
      let mut fields = line.splitn(3, ' ');
      fields.next()?;
      let address = fields.next()?.to_string();
      let name = terminal_text(fields.next().unwrap_or(""));
      if address.len() != 17 {
        return None;
      }
      Some(Device {
        address,
        name,
        ..Default::default()
      })
    })
    .collect()
}

pub fn merge_device_info(device: &mut Device, input: &str) {
  for line in input.lines() {
    let line = line.trim();
    if let Some(value) = line.strip_prefix("Name: ") {
      device.name = terminal_text(value);
    } else if let Some(value) = line.strip_prefix("Alias: ") {
      if device.name.is_empty() {
        device.name = terminal_text(value);
      }
    } else if let Some(value) = line.strip_prefix("Paired: ") {
      device.paired = value == "yes";
    } else if let Some(value) = line.strip_prefix("Trusted: ") {
      device.trusted = value == "yes";
    } else if let Some(value) = line.strip_prefix("Connected: ") {
      device.connected = value == "yes";
    } else if let Some(value) = line.strip_prefix("Battery Percentage: ") {
      device.battery = parse_battery_percentage(value);
    }
  }
}
fn parse_battery_percentage(value: &str) -> Option<u8> {
  let value = value.trim();
  let percentage = if let Some(hexadecimal) = value.strip_prefix("0x") {
    let (hexadecimal, decimal) = hexadecimal
      .split_once(' ')
      .map_or((hexadecimal, None), |(hex, decimal)| {
        (hex, Some(decimal.trim()))
      });
    let parsed = u8::from_str_radix(hexadecimal, 16).ok()?;
    if let Some(decimal) = decimal {
      let decimal = decimal
        .strip_prefix('(')?
        .strip_suffix(')')?
        .parse::<u8>()
        .ok()?;
      if decimal != parsed {
        return None;
      }
    }
    parsed
  } else {
    value
      .strip_prefix('[')
      .and_then(|value| value.strip_suffix(']'))
      .unwrap_or(value)
      .parse::<u8>()
      .ok()?
  };
  (percentage <= 100).then_some(percentage)
}

pub fn validate_address(v: &str) -> Result<(), BluetoothError> {
  if v.len() == 17
    && v.chars().enumerate().all(|(i, c)| {
      if i % 3 == 2 {
        c == ':'
      } else {
        c.is_ascii_hexdigit()
      }
    })
  {
    Ok(())
  } else {
    Err(BluetoothError::InvalidAddress)
  }
}
#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn adapter() {
    let a = parse_adapter(
      "Controller AA:BB:CC:DD:EE:FF Laptop\n\tPowered: yes\n\tDiscoverable: no\n\tPairable: yes",
    )
    .unwrap();
    assert!(a.powered);
    assert!(!a.discoverable)
  }
  #[test]
  fn address_is_strict() {
    assert!(validate_address("AA:BB:CC:DD:EE:FF").is_ok());
    assert!(validate_address("bad").is_err())
  }

  #[test]
  fn device_info_is_normalized_without_exposing_extra_fields() {
    let mut device = Device {
      address: "AA:BB:CC:DD:EE:FF".into(),
      ..Default::default()
    };
    merge_device_info(
      &mut device,
      "Name: Headset\nPaired: yes\nTrusted: no\nConnected: yes\nBattery Percentage: [77]",
    );
    assert_eq!(device.name, "Headset");
    assert!(device.paired && device.connected && !device.trusted);
    assert_eq!(device.battery, Some(77));
  }

  #[test]
  fn pairing_errors_are_mapped_to_friendly_messages() {
    assert_eq!(
      normalize_pair_error("org.bluez.Error.AlreadyPaired", "already done"),
      "device is already paired"
    );
    assert_eq!(
      normalize_pair_error("org.bluez.Error.Canceled", "canceled"),
      "pairing canceled or denied by the user"
    );
    assert_eq!(
      normalize_pair_error("org.bluez.Error.AuthenticationFailed", "bad pin"),
      "authentication failed; check the PIN or passkey"
    );
    assert_eq!(
      normalize_pair_error("org.bluez.Error.SomeOther", "raw message"),
      "raw message"
    );
  }

  #[test]
  fn pair_requires_strict_address() {
    assert!(pair_device_via_dbus("bad").is_err());
  }
  #[test]
  fn parses_real_bluetoothctl_battery_percentage() {
    let mut device = Device::default();
    merge_device_info(&mut device, "Battery Percentage: 0x55 (85)");
    assert_eq!(device.battery, Some(85));
  }

  #[test]
  fn device_info_queries_use_bounded_parallelism() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    #[derive(Default)]
    struct Runner {
      active: AtomicUsize,
      maximum: AtomicUsize,
    }
    impl ProcessRunner for Runner {
      fn run(
        &self,
        request: &ProcessRequest,
      ) -> Result<argvus_control_center_core::process::ProcessOutput, ProcessError> {
        let text = match request.args[0].as_str() {
          "devices" => (0..8)
            .map(|index| format!("Device AA:BB:CC:DD:EE:{index:02X} Device {index}\n"))
            .collect(),
          "info" => {
            assert_eq!(request.timeout, Some(Duration::from_secs(5)));
            let active = self.active.fetch_add(1, Ordering::SeqCst) + 1;
            self.maximum.fetch_max(active, Ordering::SeqCst);
            std::thread::sleep(Duration::from_millis(30));
            self.active.fetch_sub(1, Ordering::SeqCst);
            "Battery Percentage: 0x55 (85)".to_owned()
          }
          _ => String::new(),
        };
        Ok(argvus_control_center_core::process::ProcessOutput {
          stdout: text.into_bytes(),
          stderr: vec![],
          status: Some(0),
          timed_out: false,
          cancelled: false,
        })
      }
    }
    let backend = BluetoothBackend::new(
      Runner::default(),
      Capabilities {
        has_bluetoothctl: true,
        ..Default::default()
      },
    );
    let snapshot = backend.snapshot().unwrap();
    let maximum = backend.runner.maximum.load(Ordering::SeqCst);
    assert!(maximum > 1 && maximum <= 4, "maximum={maximum}");
    assert_eq!(snapshot.devices.len(), 8);
    assert!(
      snapshot
        .devices
        .iter()
        .all(|device| device.battery == Some(85))
    );
  }
  #[test]
  fn battery_percentages_accept_supported_forms_and_reject_invalid_values() {
    for value in ["0x00 (0)", "0x64 (100)", "0x55", "85", "[85]"] {
      assert!(parse_battery_percentage(value).is_some(), "{value}");
    }
    for value in ["0x55 (90)", "0xff (255)", "101", "[256]", "unknown", "0xgg"] {
      assert!(parse_battery_percentage(value).is_none(), "{value}");
    }
  }
}
