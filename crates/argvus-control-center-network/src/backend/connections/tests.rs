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
