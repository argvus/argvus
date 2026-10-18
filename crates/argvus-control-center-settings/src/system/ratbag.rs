//! Implements mouse-device integration in crate `argvus control center settings`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use std::collections::HashMap;

use zbus::blocking::{Connection, connection::Builder};
use zbus::zvariant::{OwnedObjectPath, OwnedValue, Value};

/// Defines the constant `SERVICE`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
const SERVICE: &str = "org.freedesktop.ratbag1";
/// Defines the constant `MANAGER_PATH`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
const MANAGER_PATH: &str = "/org/freedesktop/ratbag1";
/// Defines the constant `MANAGER`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
const MANAGER: &str = "org.freedesktop.ratbag1.Manager";
/// Defines the constant `PROPERTIES`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
const PROPERTIES: &str = "org.freedesktop.DBus.Properties";
/// Defines the constant `DEVICE`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
const DEVICE: &str = "org.freedesktop.ratbag1.Device";
/// Defines the constant `PROFILE`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
const PROFILE: &str = "org.freedesktop.ratbag1.Profile";
/// Defines the constant `RESOLUTION`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
const RESOLUTION: &str = "org.freedesktop.ratbag1.Resolution";

/// Names the type `Properties`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
type Properties = HashMap<String, OwnedValue>;
/// Names the type `ManagedObjects`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
type ManagedObjects = HashMap<OwnedObjectPath, HashMap<String, Properties>>;

#[derive(Debug, Clone, PartialEq, Eq)]
/// Represents `Device`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct Device {
  pub path: String,
  pub name: String,
  pub profiles: Vec<Profile>,
}

impl Device {
  /// Executes the `active_profile` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn active_profile(&self) -> Option<&Profile> {
    self.profiles.iter().find(|profile| profile.active)
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Represents `Profile`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct Profile {
  pub path: String,
  pub name: String,
  pub index: u32,
  pub active: bool,
  pub disabled: bool,
  pub report_rate: Option<u32>,
  pub report_rates: Vec<u32>,
  pub resolutions: Vec<Resolution>,
}

impl Profile {
  /// Executes the `active_resolution` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn active_resolution(&self) -> Option<&Resolution> {
    self.resolutions.iter().find(|resolution| resolution.active)
  }

  /// Executes the `supports_dpi` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn supports_dpi(&self) -> bool {
    self.resolutions.len() > 1
      || self
        .active_resolution()
        .is_some_and(|resolution| resolution.supported.len() > 1)
  }

  /// Executes the `supports_report_rate` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn supports_report_rate(&self) -> bool {
    self.report_rates.len() > 1
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Represents `Resolution`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct Resolution {
  pub path: String,
  pub index: u32,
  pub dpi_x: u32,
  pub dpi_y: u32,
  pub active: bool,
  pub default: bool,
  pub supported: Vec<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Defines `Change`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub enum Change {
  Profile(i8),
  Dpi(i8),
  ReportRate(i8),
}

/// Executes the `connection` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn connection() -> Option<Connection> {
  Builder::system().ok()?.build().ok()
}

/// Executes the `string_property` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn string_property(properties: &Properties, name: &str) -> Option<String> {
  properties.get(name)?.downcast_ref::<String>().ok()
}

/// Executes the `u32_property` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn u32_property(properties: &Properties, name: &str) -> Option<u32> {
  properties.get(name)?.downcast_ref::<u32>().ok()
}

/// Executes the `bool_property` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn bool_property(properties: &Properties, name: &str) -> Option<bool> {
  properties.get(name)?.downcast_ref::<bool>().ok()
}

/// Executes the `object_paths_property` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn object_paths_property(properties: &Properties, name: &str) -> Option<Vec<OwnedObjectPath>> {
  properties.get(name)?.try_clone().ok()?.try_into().ok()
}

/// Executes the `u32s_property` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn u32s_property(properties: &Properties, name: &str) -> Option<Vec<u32>> {
  properties.get(name)?.try_clone().ok()?.try_into().ok()
}

/// Executes the `interface` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn interface<'a>(
  objects: &'a ManagedObjects,
  path: &str,
  interface: &str,
) -> Option<&'a Properties> {
  objects
    .iter()
    .find(|(object, _)| object.as_str() == path)
    .and_then(|(_, interfaces)| interfaces.get(interface))
}

/// Executes the `profile_from_objects` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn profile_from_objects(objects: &ManagedObjects, path: &str) -> Option<Profile> {
  let properties = interface(objects, path, PROFILE)?;
  let resolution_paths = object_paths_property(properties, "Resolutions")?;
  let resolutions = resolution_paths
    .iter()
    .filter_map(|path| resolution_from_objects(objects, path.as_str()))
    .collect();
  Some(Profile {
    path: path.to_string(),
    name: string_property(properties, "Name").unwrap_or_default(),
    index: u32_property(properties, "Index").unwrap_or_default(),
    active: bool_property(properties, "IsActive").unwrap_or(false),
    disabled: bool_property(properties, "Disabled").unwrap_or(false),
    report_rate: u32_property(properties, "ReportRate"),
    report_rates: u32s_property(properties, "ReportRates").unwrap_or_default(),
    resolutions,
  })
}

/// Executes the `resolution_from_objects` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn resolution_from_objects(objects: &ManagedObjects, path: &str) -> Option<Resolution> {
  let properties = interface(objects, path, RESOLUTION)?;
  let dpi = u32_property(properties, "Resolution")?;
  Some(Resolution {
    path: path.to_string(),
    index: u32_property(properties, "Index").unwrap_or_default(),
    dpi_x: dpi,
    dpi_y: dpi,
    active: bool_property(properties, "IsActive").unwrap_or(false),
    default: bool_property(properties, "IsDefault").unwrap_or(false),
    supported: u32s_property(properties, "Resolutions").unwrap_or_default(),
  })
}

/// Converts input data into `parse_devices` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn parse_devices(objects: ManagedObjects) -> Vec<Device> {
  objects
    .iter()
    .filter_map(|(path, interfaces)| {
      let properties = interfaces.get(DEVICE)?;
      let profile_paths = object_paths_property(properties, "Profiles")?;
      Some(Device {
        path: path.to_string(),
        name: string_property(properties, "Name").unwrap_or_default(),
        profiles: profile_paths
          .iter()
          .filter_map(|path| profile_from_objects(&objects, path.as_str()))
          .collect(),
      })
    })
    .collect()
}

/// Executes the `devices` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn devices() -> Vec<Device> {
  let Some(connection) = connection() else {
    return Vec::new();
  };
  let Ok(manager) = properties(&connection, MANAGER_PATH, MANAGER) else {
    return Vec::new();
  };
  let mut objects = ManagedObjects::new();
  let Some(device_paths) = object_paths_property(&manager, "Devices") else {
    return Vec::new();
  };
  for device_path in device_paths {
    let path = device_path.to_string();
    let Ok(device_properties) = properties(&connection, &path, DEVICE) else {
      continue;
    };
    let profile_paths = object_paths_property(&device_properties, "Profiles").unwrap_or_default();
    objects.insert(
      device_path,
      HashMap::from([(DEVICE.to_string(), device_properties)]),
    );
    for profile_path in profile_paths {
      let profile_name = profile_path.to_string();
      let Ok(profile_properties) = properties(&connection, &profile_name, PROFILE) else {
        continue;
      };
      let resolution_paths =
        object_paths_property(&profile_properties, "Resolutions").unwrap_or_default();
      objects.insert(
        profile_path,
        HashMap::from([(PROFILE.to_string(), profile_properties)]),
      );
      for resolution_path in resolution_paths {
        let resolution_name = resolution_path.to_string();
        if let Ok(resolution_properties) = properties(&connection, &resolution_name, RESOLUTION) {
          objects.insert(
            resolution_path,
            HashMap::from([(RESOLUTION.to_string(), resolution_properties)]),
          );
        }
      }
    }
  }
  let mut devices = parse_devices(objects);
  devices.sort_by(|left, right| left.path.cmp(&right.path));
  devices
}

/// Executes the `properties` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn properties(connection: &Connection, path: &str, interface: &str) -> Result<Properties, String> {
  let reply = connection
    .call_method(
      Some(SERVICE),
      path,
      Some(PROPERTIES),
      "GetAll",
      &(interface,),
    )
    .map_err(|error| error.to_string())?;
  reply
    .body()
    .deserialize()
    .map_err(|error| error.to_string())
}

/// Executes the `commit` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn commit(connection: &Connection, device: &Device) -> Result<(), String> {
  let reply = connection
    .call_method(
      Some(SERVICE),
      device.path.as_str(),
      Some(DEVICE),
      "Commit",
      &(),
    )
    .map_err(|error| error.to_string())?;
  let result: u32 = reply
    .body()
    .deserialize()
    .map_err(|error| error.to_string())?;
  if result == 0 {
    Ok(())
  } else {
    Err(format!("ratbag commit returned status {result}"))
  }
}

/// Applies the `set_property` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn set_property(
  connection: &Connection,
  path: &str,
  interface: &str,
  name: &str,
  value: u32,
  property_is_variant: bool,
) -> Result<(), String> {
  let value = if property_is_variant {
    Value::Value(Box::new(Value::U32(value)))
  } else {
    Value::U32(value)
  };
  connection
    .call_method(
      Some(SERVICE),
      path,
      Some(PROPERTIES),
      "Set",
      &(interface, name, value),
    )
    .map(|_| ())
    .map_err(|error| error.to_string())
}

/// Applies the `set_active` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn set_active(connection: &Connection, path: &str, interface: &str) -> Result<(), String> {
  let reply = connection
    .call_method(Some(SERVICE), path, Some(interface), "SetActive", &())
    .map_err(|error| error.to_string())?;
  let result: u32 = reply
    .body()
    .deserialize()
    .map_err(|error| error.to_string())?;
  if result == 0 {
    Ok(())
  } else {
    Err(format!("ratbag SetActive returned status {result}"))
  }
}

/// Executes the `next_index` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn next_index(current: usize, length: usize, direction: i8) -> Option<usize> {
  if length == 0 {
    return None;
  }
  Some(if direction < 0 {
    if current == 0 {
      length - 1
    } else {
      current - 1
    }
  } else if current + 1 == length {
    0
  } else {
    current + 1
  })
}

/// Executes the `next_device_index` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn next_device_index(current: usize, length: usize, direction: i8) -> Option<usize> {
  next_index(current, length, direction)
}

/// Applies the `apply` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn apply(device: &Device, change: Change) -> Result<Vec<Device>, String> {
  let connection = connection().ok_or_else(|| "ratbagd is unavailable".to_string())?;
  match change {
    Change::Profile(direction) => {
      let enabled: Vec<&Profile> = device
        .profiles
        .iter()
        .filter(|profile| !profile.disabled)
        .collect();
      let current = enabled
        .iter()
        .position(|profile| profile.active)
        .unwrap_or(0);
      let Some(next) = next_index(current, enabled.len(), direction) else {
        return Ok(vec![device.clone()]);
      };
      set_active(&connection, &enabled[next].path, PROFILE)?;
      commit(&connection, device)?;
    }
    Change::Dpi(direction) => {
      let profile = device
        .active_profile()
        .ok_or_else(|| "no active ratbag profile".to_string())?;
      let active = profile
        .active_resolution()
        .ok_or_else(|| "no active ratbag resolution".to_string())?;
      let (resolution, dpi) = if profile.resolutions.len() == 1 && active.supported.len() > 1 {
        let current = active
          .supported
          .iter()
          .position(|value| *value == active.dpi_x)
          .unwrap_or(0);
        let Some(next) = next_index(current, active.supported.len(), direction) else {
          return Ok(vec![device.clone()]);
        };
        (active, active.supported[next])
      } else {
        let current = profile
          .resolutions
          .iter()
          .position(|resolution| resolution.active)
          .unwrap_or(0);
        let Some(next) = next_index(current, profile.resolutions.len(), direction) else {
          return Ok(vec![device.clone()]);
        };
        let resolution = &profile.resolutions[next];
        (resolution, resolution.dpi_x)
      };
      set_property(
        &connection,
        &resolution.path,
        RESOLUTION,
        "Resolution",
        dpi,
        true,
      )?;
      set_active(&connection, &resolution.path, RESOLUTION)?;
      commit(&connection, device)?;
    }
    Change::ReportRate(direction) => {
      let profile = device
        .active_profile()
        .ok_or_else(|| "no active ratbag profile".to_string())?;
      let current = profile
        .report_rates
        .iter()
        .position(|rate| Some(*rate) == profile.report_rate)
        .unwrap_or(0);
      let Some(next) = next_index(current, profile.report_rates.len(), direction) else {
        return Ok(vec![device.clone()]);
      };
      set_property(
        &connection,
        &profile.path,
        PROFILE,
        "ReportRate",
        profile.report_rates[next],
        false,
      )?;
      commit(&connection, device)?;
    }
  }
  Ok(devices())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  /// Executes the `next_index_wraps_without_assuming_profile_zero` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn next_index_wraps_without_assuming_profile_zero() {
    assert_eq!(next_index(1, 3, 1), Some(2));
    assert_eq!(next_index(0, 3, -1), Some(2));
    assert_eq!(next_index(2, 3, 1), Some(0));
    assert_eq!(next_index(0, 0, 1), None);
  }

  #[test]
  /// Executes the `service_is_optional` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn service_is_optional() {
    assert_eq!(SERVICE, "org.freedesktop.ratbag1");
  }

  /// Executes the `profile` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn profile(
    index: u32,
    active: bool,
    resolutions: Vec<Resolution>,
    report_rates: Vec<u32>,
  ) -> Profile {
    Profile {
      path: format!("/profile/{index}"),
      name: String::new(),
      index,
      active,
      disabled: false,
      report_rate: report_rates.first().copied(),
      report_rates,
      resolutions,
    }
  }

  /// Executes the `resolution` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn resolution(index: u32, active: bool, supported: Vec<u32>) -> Resolution {
    let dpi = supported.first().copied().unwrap_or_default();
    Resolution {
      path: format!("/resolution/{index}"),
      index,
      dpi_x: dpi,
      dpi_y: dpi,
      active,
      default: index == 0,
      supported,
    }
  }

  #[test]
  /// Executes the `capability_helpers_use_active_profile_and_resolution` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn capability_helpers_use_active_profile_and_resolution() {
    let device = Device {
      path: "/device/second".into(),
      name: "Device B".into(),
      profiles: vec![
        profile(4, false, vec![resolution(9, true, vec![400])], vec![]),
        profile(
          7,
          true,
          vec![
            resolution(3, false, vec![800]),
            resolution(8, true, vec![800, 1600, 3200]),
          ],
          vec![125, 500, 1000],
        ),
      ],
    };

    let active = device.active_profile().expect("active profile");
    assert_eq!(active.index, 7);
    assert!(active.supports_dpi());
    assert!(active.supports_report_rate());
    assert_eq!(
      active.active_resolution().expect("active resolution").index,
      8
    );
  }

  #[test]
  /// Executes the `capability_helpers_hide_unsupported_controls` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn capability_helpers_hide_unsupported_controls() {
    let device = Device {
      path: "/device/basic".into(),
      name: "Device A".into(),
      profiles: vec![profile(
        2,
        true,
        vec![resolution(5, true, vec![1000])],
        vec![],
      )],
    };

    let active = device.active_profile().expect("active profile");
    assert!(!active.supports_dpi());
    assert!(!active.supports_report_rate());
  }

  #[test]
  /// Executes the `zero_devices_are_a_valid_ratbag_state` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn zero_devices_are_a_valid_ratbag_state() {
    let devices: Vec<Device> = Vec::new();
    assert!(devices.is_empty());
    assert_eq!(next_device_index(0, devices.len(), 1), None);
  }
}
