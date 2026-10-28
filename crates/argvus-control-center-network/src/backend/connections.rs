//! Connection profiles through `nmcli`: listing, loading into the editor,
//! saving, activating, deleting and importing.
//!
//! Every change runs as the session user. NetworkManager authorizes it with
//! polkit: local members of `wheel` are allowed outright, and anyone else is
//! asked for a password by the session's polkit agent, so the write timeout
//! leaves room for that prompt.

use super::{NetworkBackend, NetworkError, split_nmcli};
use crate::{
  model::ConnectionProfile,
  profile::{ConnectionDraft, ConnectionKind, nmcli},
};
use argvus_control_center_core::process::{ProcessRequest, ProcessRunner};
use std::{collections::BTreeSet, path::Path, time::Duration};

/// Reads of NetworkManager's settings.
const READ_TIMEOUT: Duration = Duration::from_secs(5);
/// Writes, which may wait for a polkit password prompt.
const WRITE_TIMEOUT: Duration = Duration::from_secs(90);
/// Activations, which may wait for DHCP or an access point.
const ACTIVATION_TIMEOUT: Duration = Duration::from_secs(90);
/// Fields of `nmcli connection show` read by [`parse_profiles`].
const PROFILE_FIELDS: &str = "NAME,UUID,TYPE,DEVICE,ACTIVE,AUTOCONNECT,PORT,READONLY";
/// Where VPN plugins describe themselves.
const VPN_PLUGIN_DIR: &str = "/usr/lib/NetworkManager/VPN";

/// What a save did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveOutcome {
  pub uuid: String,
  /// The profile was active and was brought up again with the new settings.
  pub reactivated: bool,
}

/// An installed NetworkManager VPN plugin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VpnPlugin {
  /// Short name (`openvpn`), which is also the `nmcli connection import` type.
  pub name: String,
  /// `vpn.service-type` of its profiles.
  pub service: String,
}

/// Profiles from `nmcli -t -f NAME,UUID,TYPE,DEVICE,ACTIVE,AUTOCONNECT,PORT,READONLY connection show`.
pub fn parse_profiles(output: &str) -> Vec<ConnectionProfile> {
  output
    .lines()
    .filter_map(|line| {
      let fields = split_nmcli(line);
      if fields.len() < 8 || fields[1].is_empty() {
        return None;
      }
      let non_empty = |value: &String| (!value.is_empty() && value != "--").then(|| value.clone());
      Some(ConnectionProfile {
        name: fields[0].clone(),
        uuid: fields[1].clone(),
        kind: fields[2].clone(),
        device: non_empty(&fields[3]),
        active: fields[4] == "yes",
        autoconnect: fields[5] == "yes",
        port_type: non_empty(&fields[6]),
        read_only: fields[7] == "yes",
      })
    })
    .collect()
}

/// A plugin from its `.name` file (`[VPN Connection]` with `name=` and
/// `service=`), or `None` when either is missing.
pub fn parse_vpn_plugin(contents: &str) -> Option<VpnPlugin> {
  let mut in_section = false;
  let mut name = None;
  let mut service = None;
  for line in contents.lines().map(str::trim) {
    if line.starts_with('[') {
      in_section = line == "[VPN Connection]";
      continue;
    }
    if !in_section {
      continue;
    }
    match line.split_once('=') {
      Some(("name", value)) => name = Some(value.trim().to_owned()),
      Some(("service", value)) => service = Some(value.trim().to_owned()),
      _ => {}
    }
  }
  Some(VpnPlugin {
    name: name.filter(|value| !value.is_empty())?,
    service: service.filter(|value| !value.is_empty())?,
  })
}

/// The installed VPN plugins, by name.
pub fn installed_vpn_plugins() -> Vec<VpnPlugin> {
  vpn_plugins_in(Path::new(VPN_PLUGIN_DIR))
}

fn vpn_plugins_in(directory: &Path) -> Vec<VpnPlugin> {
  let Ok(entries) = std::fs::read_dir(directory) else {
    return Vec::new();
  };
  let mut plugins = entries
    .filter_map(Result::ok)
    .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "name"))
    .filter_map(|entry| std::fs::read_to_string(entry.path()).ok())
    .filter_map(|contents| parse_vpn_plugin(&contents))
    .collect::<Vec<_>>();
  plugins.sort_by(|left, right| left.name.cmp(&right.name));
  plugins.dedup_by(|left, right| left.service == right.service);
  plugins
}

/// The UUID in `nmcli connection add`'s report
/// (`Connection 'name' (uuid) successfully added.`).
fn added_uuid(output: &str) -> Option<String> {
  let start = output.rfind('(')? + 1;
  let end = start + output[start..].find(')')?;
  let uuid = output[start..end].trim();
  (uuid.len() == 36).then(|| uuid.to_owned())
}

impl<R: ProcessRunner> NetworkBackend<R> {
  fn nmcli(&self, args: &[String], timeout: Duration) -> Result<String, NetworkError> {
    if !self.capabilities.has_nmcli {
      return Err(NetworkError::BackendUnavailable);
    }
    let request = args
      .iter()
      .fold(ProcessRequest::new("nmcli"), |request, argument| {
        request.arg(argument.as_str())
      })
      .timeout(timeout);
    self.execute(&request)
  }

  fn nmcli_str(&self, args: &[&str], timeout: Duration) -> Result<String, NetworkError> {
    let owned = args
      .iter()
      .map(|argument| (*argument).to_owned())
      .collect::<Vec<_>>();
    self.nmcli(&owned, timeout)
  }

  /// Every connection profile NetworkManager has.
  pub fn list_profiles(&self) -> Result<Vec<ConnectionProfile>, NetworkError> {
    self
      .nmcli_str(
        &["-t", "-f", PROFILE_FIELDS, "connection", "show"],
        READ_TIMEOUT,
      )
      .map(|output| parse_profiles(&output))
  }

  /// The profile `uuid` as the editor shows it: its settings, the devices
  /// that are its ports and its WireGuard peers. `owner` is the user a
  /// private profile is limited to.
  pub fn load_profile(
    &self,
    uuid: &str,
    profiles: &[ConnectionProfile],
    owner: &str,
  ) -> Result<ConnectionDraft, NetworkError> {
    let output = self.nmcli_str(&["-t", "connection", "show", "uuid", uuid], READ_TIMEOUT)?;
    let mut draft = nmcli::draft_from_settings(&nmcli::parse_settings(&output), owner);
    let ports = match draft.kind.port_type() {
      Some(port_type) => self
        .ports_of(&draft, port_type, profiles)
        .into_iter()
        .map(|(device, _)| device)
        .collect(),
      None => BTreeSet::new(),
    };
    let peers = if draft.kind == ConnectionKind::WireGuard {
      // Read with secrets: rewriting the peers must keep their preshared keys.
      self
        .nmcli_str(
          &[
            "-s",
            "-g",
            "wireguard.peers",
            "connection",
            "show",
            "uuid",
            uuid,
          ],
          READ_TIMEOUT,
        )
        .map(|value| nmcli::parse_peers(value.trim()))?
    } else {
      Vec::new()
    };
    draft.set_loaded_members(ports, peers);
    Ok(draft)
  }

  /// The port profiles of a controller, as (device, port UUID). A port names
  /// its controller by UUID or by interface name.
  fn ports_of(
    &self,
    draft: &ConnectionDraft,
    port_type: &str,
    profiles: &[ConnectionProfile],
  ) -> Vec<(String, String)> {
    let Some(uuid) = draft.uuid.as_deref() else {
      return Vec::new();
    };
    let interface = draft.value("connection.interface-name");
    profiles
      .iter()
      .filter(|profile| profile.port_type.as_deref() == Some(port_type))
      .filter_map(|profile| {
        let output = self
          .nmcli_str(
            &[
              "-t",
              "-g",
              "connection.controller,connection.interface-name",
              "connection",
              "show",
              "uuid",
              &profile.uuid,
            ],
            READ_TIMEOUT,
          )
          .ok()?;
        let mut lines = output.lines().map(|line| nmcli::unescape(line.trim()));
        let controller = lines.next()?;
        let device = lines.next().filter(|device| !device.is_empty())?;
        let ours = controller == uuid || (!interface.is_empty() && controller == interface);
        ours.then(|| (device, profile.uuid.clone()))
      })
      .collect()
  }

  /// Creates or updates the profile, then writes its secrets, its ports and
  /// its peers. A profile that was active is brought up again, so the new
  /// settings apply now.
  pub fn save_profile(
    &self,
    draft: &ConnectionDraft,
    profiles: &[ConnectionProfile],
  ) -> Result<SaveOutcome, NetworkError> {
    let uuid = match &draft.uuid {
      Some(uuid) => {
        if let Some(args) = nmcli::modify_arguments(draft) {
          self.nmcli(&args, WRITE_TIMEOUT)?;
        }
        for setting in nmcli::removed_settings(draft) {
          self.nmcli_str(
            &["connection", "modify", "uuid", uuid, "remove", setting],
            WRITE_TIMEOUT,
          )?;
        }
        uuid.clone()
      }
      None => {
        let output = self.nmcli(&nmcli::add_arguments(draft), WRITE_TIMEOUT)?;
        added_uuid(&output).ok_or_else(|| NetworkError::Command(output.trim().to_owned()))?
      }
    };
    if let Some(script) = nmcli::secret_script(draft) {
      self.edit_with_script(&uuid, script)?;
    }
    if let Some(port_type) = draft.kind.port_type()
      && draft.ports_changed()
    {
      let mut saved = draft.clone();
      saved.uuid = Some(uuid.clone());
      let existing = self.ports_of(&saved, port_type, profiles);
      let (added, removed) = draft.port_changes();
      for device in added {
        self.nmcli(
          &nmcli::port_arguments(port_type, &uuid, &device),
          WRITE_TIMEOUT,
        )?;
      }
      for (_, port_uuid) in existing
        .iter()
        .filter(|(device, _)| removed.contains(device))
      {
        self.nmcli_str(&["connection", "delete", "uuid", port_uuid], WRITE_TIMEOUT)?;
      }
    }
    let was_active = profiles
      .iter()
      .any(|profile| profile.uuid == uuid && profile.active);
    if was_active {
      self.activate(&uuid)?;
    }
    Ok(SaveOutcome {
      uuid,
      reactivated: was_active,
    })
  }

  /// Runs `nmcli connection edit` with `script` on its standard input, so the
  /// secrets never appear in an argument list. The editor reports a refused
  /// value on its standard error and still exits with success, so any
  /// `Error:` line is a failure.
  fn edit_with_script(&self, uuid: &str, script: String) -> Result<(), NetworkError> {
    if !self.capabilities.has_nmcli {
      return Err(NetworkError::BackendUnavailable);
    }
    let request = ProcessRequest::new("nmcli")
      .arg("connection")
      .arg("edit")
      .arg("uuid")
      .arg(uuid)
      .stdin(script)
      .timeout(WRITE_TIMEOUT);
    let output = self.runner.run(&request)?;
    if output.timed_out {
      return Err(NetworkError::Timeout);
    }
    let stderr =
      argvus_control_center_core::sanitize::terminal_text(&String::from_utf8_lossy(&output.stderr));
    let refused = stderr
      .lines()
      .filter(|line| line.trim_start().starts_with("Error"))
      .collect::<Vec<_>>()
      .join("\n");
    if output.status != Some(0) || !refused.is_empty() {
      let message = if refused.is_empty() { stderr } else { refused };
      return Err(super::command_error(message));
    }
    Ok(())
  }

  /// Deletes the profile, and the port profiles of a controller first, so no
  /// port is left pointing at nothing.
  pub fn delete_profile(
    &self,
    draft: &ConnectionDraft,
    profiles: &[ConnectionProfile],
  ) -> Result<(), NetworkError> {
    let uuid = draft.uuid.as_deref().ok_or(NetworkError::InvalidInput)?;
    if let Some(port_type) = draft.kind.port_type() {
      for (_, port_uuid) in self.ports_of(draft, port_type, profiles) {
        self.nmcli_str(&["connection", "delete", "uuid", &port_uuid], WRITE_TIMEOUT)?;
      }
    }
    self
      .nmcli_str(&["connection", "delete", "uuid", uuid], WRITE_TIMEOUT)
      .map(|_| ())
  }

  /// Brings the profile up.
  pub fn activate(&self, uuid: &str) -> Result<(), NetworkError> {
    self
      .nmcli_str(&["connection", "up", "uuid", uuid], ACTIVATION_TIMEOUT)
      .map(|_| ())
  }

  /// Takes the profile down.
  pub fn deactivate(&self, uuid: &str) -> Result<(), NetworkError> {
    self
      .nmcli_str(&["connection", "down", "uuid", uuid], WRITE_TIMEOUT)
      .map(|_| ())
  }

  /// Creates a profile from a VPN configuration file (`.ovpn`, a WireGuard
  /// `.conf`...). `kind` is the plugin name, or `wireguard`.
  pub fn import_vpn(&self, kind: &str, path: &str) -> Result<String, NetworkError> {
    if kind.is_empty() || path.trim().is_empty() || path.chars().any(char::is_control) {
      return Err(NetworkError::InvalidInput);
    }
    let output = self.nmcli_str(
      &["connection", "import", "type", kind, "file", path.trim()],
      WRITE_TIMEOUT,
    )?;
    added_uuid(&output).ok_or_else(|| NetworkError::Command(output.trim().to_owned()))
  }

  /// A new WireGuard private key (`wg genkey`).
  pub fn wireguard_private_key(&self) -> Result<String, NetworkError> {
    if !self.capabilities.has_wireguard_tools {
      return Err(NetworkError::BackendUnavailable);
    }
    self
      .run("wg", &["genkey"], READ_TIMEOUT)
      .map(|key| key.trim().to_owned())
  }

  /// The public key of a WireGuard private key (`wg pubkey`, the private key
  /// on its standard input).
  pub fn wireguard_public_key(&self, private_key: &str) -> Result<String, NetworkError> {
    if !self.capabilities.has_wireguard_tools {
      return Err(NetworkError::BackendUnavailable);
    }
    let request = ProcessRequest::new("wg")
      .arg("pubkey")
      .stdin(format!("{}\n", private_key.trim()))
      .timeout(READ_TIMEOUT);
    self.execute(&request).map(|key| key.trim().to_owned())
  }
}

#[cfg(test)]
mod tests;
