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
mod tests {
  use super::*;
  use crate::profile::ConnectionKind;
  use argvus_control_center_core::{
    capabilities::Capabilities,
    process::{ProcessError, ProcessOutput},
  };
  use std::sync::Mutex;

  /// Answers like NetworkManager would and records every request.
  #[derive(Default)]
  struct FakeNmcli {
    requests: Mutex<Vec<ProcessRequest>>,
    /// Standard error of `nmcli connection edit`.
    edit_stderr: &'static str,
  }

  impl ProcessRunner for FakeNmcli {
    fn run(&self, request: &ProcessRequest) -> Result<ProcessOutput, ProcessError> {
      self.requests.lock().unwrap().push(request.clone());
      let joined = request.args.join(" ");
      let (stdout, stderr) = if joined.starts_with("connection add") {
        (
          "Connection 'x' (11111111-2222-3333-4444-555555555555) successfully added.\n",
          "",
        )
      } else if joined.starts_with("connection edit") {
        ("", self.edit_stderr)
      } else if joined.contains("connection.controller,connection.interface-name") {
        if joined.ends_with("port-a") {
          ("br-uuid\neno1\n", "")
        } else {
          ("other-uuid\neno2\n", "")
        }
      } else if joined.starts_with("-t connection show uuid br-uuid") {
        (
          "connection.id:br1\nconnection.uuid:br-uuid\nconnection.type:bridge\nconnection.interface-name:br1\n",
          "",
        )
      } else {
        ("", "")
      };
      Ok(ProcessOutput {
        stdout: stdout.as_bytes().to_vec(),
        stderr: stderr.as_bytes().to_vec(),
        status: Some(0),
        timed_out: false,
      })
    }
  }

  fn backend(runner: FakeNmcli) -> NetworkBackend<FakeNmcli> {
    NetworkBackend::new(
      runner,
      Capabilities {
        has_nmcli: true,
        has_wireguard_tools: true,
        ..Default::default()
      },
    )
  }

  fn requests(backend: &NetworkBackend<FakeNmcli>) -> Vec<ProcessRequest> {
    backend.runner.requests.lock().unwrap().clone()
  }

  fn profile(uuid: &str, kind: &str, port_type: Option<&str>, active: bool) -> ConnectionProfile {
    ConnectionProfile {
      name: uuid.into(),
      uuid: uuid.into(),
      kind: kind.into(),
      active,
      port_type: port_type.map(str::to_owned),
      ..Default::default()
    }
  }

  #[test]
  fn profiles_parse_from_the_real_listing() {
    let profiles = parse_profiles(
      "br0:c2d8f4bc-00d0-468b-a6b4-11fdf0e20721:bridge:br0:yes:yes::no\nbr0-ethernet:c66ae93b-52c2-4ff9-a327-de09c8402975:802-3-ethernet:eno1:yes:yes:bridge:no\nCabeada1:40a5e8e4-ea6b-3b81-a165-a2ff0d08cb3f:802-3-ethernet::no:yes::no\n",
    );
    assert_eq!(profiles.len(), 3);
    assert!(profiles[0].active && profiles[0].port_type.is_none());
    assert_eq!(profiles[1].port_type.as_deref(), Some("bridge"));
    assert_eq!(profiles[2].device, None);
  }

  #[test]
  fn vpn_plugins_parse_from_their_name_files() {
    let plugin = parse_vpn_plugin(
      "[VPN Connection]\nname=openvpn\nservice=org.freedesktop.NetworkManager.openvpn\nprogram=/usr/lib/nm-openvpn-service\n\n[GNOME]\nauth-dialog=x\n",
    )
    .unwrap();
    assert_eq!(plugin.name, "openvpn");
    assert_eq!(plugin.service, "org.freedesktop.NetworkManager.openvpn");
    assert_eq!(parse_vpn_plugin("[GNOME]\nname=x\n"), None);
  }

  #[test]
  fn new_profiles_are_added_then_get_their_secrets_by_stdin() {
    let backend = backend(FakeNmcli::default());
    let mut draft = ConnectionDraft::new(ConnectionKind::Wifi, "Home", None, "boss");
    draft.set("802-11-wireless.ssid", "Home".into());
    draft.set("802-11-wireless-security.key-mgmt", "wpa-psk".into());
    draft.set_secret("802-11-wireless-security.psk", "hunter2hunter2".into());
    let outcome = backend.save_profile(&draft, &[]).unwrap();
    assert_eq!(outcome.uuid, "11111111-2222-3333-4444-555555555555");
    assert!(!outcome.reactivated);
    let requests = requests(&backend);
    assert_eq!(requests.len(), 2);
    assert!(requests.iter().all(|request| {
      !request
        .args
        .iter()
        .any(|argument| argument.contains("hunter2"))
    }));
    let edit = &requests[1];
    assert_eq!(
      edit.args,
      [
        "connection",
        "edit",
        "uuid",
        "11111111-2222-3333-4444-555555555555"
      ]
    );
    let stdin = String::from_utf8(edit.stdin.as_ref().unwrap().as_bytes().to_vec()).unwrap();
    assert!(stdin.contains("set 802-11-wireless-security.psk hunter2hunter2"));
  }

  #[test]
  fn a_refused_secret_is_reported_even_though_nmcli_succeeds() {
    let backend = backend(FakeNmcli {
      edit_stderr: "Error: failed to set 'psk' property: invalid\n",
      ..Default::default()
    });
    let mut draft = ConnectionDraft::new(ConnectionKind::Pppoe, "DSL", None, "boss");
    draft.set("pppoe.parent", "eno1".into());
    draft.set("pppoe.username", "me".into());
    draft.set_secret("pppoe.password", "secret".into());
    let error = backend.save_profile(&draft, &[]).unwrap_err();
    assert!(error.to_string().contains("failed to set"));
  }

  #[test]
  fn saving_an_active_profile_reapplies_it() {
    let backend = backend(FakeNmcli::default());
    let mut draft = backend.load_profile("br-uuid", &[], "boss").unwrap();
    draft.toggle("bridge.stp");
    let profiles = [profile("br-uuid", "bridge", None, true)];
    let outcome = backend.save_profile(&draft, &profiles).unwrap();
    assert!(outcome.reactivated);
    let last = requests(&backend).last().unwrap().args.join(" ");
    assert_eq!(last, "connection up uuid br-uuid");
  }

  #[test]
  fn controllers_load_their_ports_and_apply_changes_on_save() {
    let backend = backend(FakeNmcli::default());
    let profiles = [
      profile("br-uuid", "bridge", None, false),
      profile("port-a", "802-3-ethernet", Some("bridge"), false),
      profile("port-b", "802-3-ethernet", Some("bridge"), false),
    ];
    let mut draft = backend.load_profile("br-uuid", &profiles, "boss").unwrap();
    assert_eq!(draft.ports, BTreeSet::from(["eno1".to_owned()]));
    draft.ports = BTreeSet::from(["eno3".to_owned()]);
    backend.save_profile(&draft, &profiles).unwrap();
    let commands = requests(&backend)
      .iter()
      .map(|request| request.args.join(" "))
      .collect::<Vec<_>>();
    assert!(commands.iter().any(|command| command
      == "connection add type ethernet con-name bridge-port-eno3 ifname eno3 port-type bridge controller br-uuid"));
    assert!(
      commands
        .iter()
        .any(|command| command == "connection delete uuid port-a")
    );
    assert!(
      !commands
        .iter()
        .any(|command| command == "connection delete uuid port-b")
    );
  }

  #[test]
  fn deleting_a_controller_deletes_its_ports_first() {
    let backend = backend(FakeNmcli::default());
    let profiles = [
      profile("br-uuid", "bridge", None, false),
      profile("port-a", "802-3-ethernet", Some("bridge"), false),
    ];
    let draft = backend.load_profile("br-uuid", &profiles, "boss").unwrap();
    backend.delete_profile(&draft, &profiles).unwrap();
    let deletes = requests(&backend)
      .iter()
      .map(|request| request.args.join(" "))
      .filter(|command| command.starts_with("connection delete"))
      .collect::<Vec<_>>();
    assert_eq!(
      deletes,
      [
        "connection delete uuid port-a",
        "connection delete uuid br-uuid"
      ]
    );
  }

  #[test]
  fn wireguard_public_keys_are_derived_through_stdin() {
    let backend = backend(FakeNmcli::default());
    let _ = backend.wireguard_public_key("yAnz5TF+lXXJte14tji3zlMNq+hd2rYUIgJBgB3fBmk=");
    let request = requests(&backend).pop().unwrap();
    assert_eq!(request.program, "wg");
    assert_eq!(request.args, ["pubkey"]);
    assert!(request.stdin.is_some());
  }

  #[test]
  fn polkit_refusals_have_their_own_error() {
    assert!(matches!(
      super::super::command_error("Error: Not authorized to modify connections".into()),
      NetworkError::NotAuthorized
    ));
    assert_eq!(
      added_uuid("Connection 'x' (11111111-2222-3333-4444-555555555555) successfully added."),
      Some("11111111-2222-3333-4444-555555555555".into())
    );
  }
}
