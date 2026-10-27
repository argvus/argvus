//! The pages that list connection profiles: Connections (open the editor),
//! Activate a connection (bring one up or down) and Add connection (pick a
//! kind, or import a VPN file).

use super::{
  Item, NetworkApp,
  editor::{EditTarget, EditorResult, ProfileSelector},
};
use crate::{
  backend::{VpnPlugin, installed_vpn_plugins},
  model::{ConnectionProfile, NetworkPage},
  profile::{ConnectionKind, KindGroup},
};
use argvus_i18n::tr;
use argvus_tui::{icons, menu::Row};

/// Kinds in the order the lists group them.
const LIST_ORDER: [ConnectionKind; 13] = [
  ConnectionKind::Ethernet,
  ConnectionKind::Wifi,
  ConnectionKind::Hotspot,
  ConnectionKind::Pppoe,
  ConnectionKind::Bridge,
  ConnectionKind::Bond,
  ConnectionKind::Team,
  ConnectionKind::Vlan,
  ConnectionKind::IpTunnel,
  ConnectionKind::Loopback,
  ConnectionKind::WireGuard,
  ConnectionKind::Vpn,
  ConnectionKind::Other,
];

/// The kind a listed profile is grouped under. The listing does not carry
/// the Wi-Fi mode, so hotspots are listed with Wi-Fi.
pub(super) fn listed_kind(profile: &ConnectionProfile) -> ConnectionKind {
  ConnectionKind::from_nmcli(&profile.kind, None)
}

/// A readable name for a VPN plugin.
pub(super) fn plugin_label(name: &str) -> String {
  match name {
    "openvpn" => "OpenVPN",
    "openconnect" => "OpenConnect",
    "vpnc" => "Cisco (vpnc)",
    "pptp" => "PPTP",
    "l2tp" => "L2TP",
    "sstp" => "SSTP",
    "strongswan" => "strongSwan",
    "libreswan" => "Libreswan",
    "fortisslvpn" => "Fortinet SSL",
    other => other,
  }
  .to_owned()
}

impl NetworkApp {
  /// The installed VPN plugins, read when the Add page opens.
  pub(super) fn refresh_vpn_plugins(&mut self) {
    self.vpn_plugins = installed_vpn_plugins();
  }

  /// The label of the plugin serving `service`.
  pub(super) fn vpn_plugin_label(&self, service: &str) -> Option<String> {
    self
      .vpn_plugins
      .iter()
      .find(|plugin| plugin.service == service)
      .map(|plugin| plugin_label(&plugin.name))
  }

  /// Indexes of the listed profiles, filtered by the search, grouped by
  /// kind and sorted by name.
  pub(super) fn visible_profiles(&self) -> Vec<usize> {
    let query = self.search.as_ref().map(|query| query.to_lowercase());
    let mut indexes = self
      .snapshot
      .profiles
      .iter()
      .enumerate()
      .filter(|(_, profile)| {
        query
          .as_ref()
          .is_none_or(|query| profile.name.to_lowercase().contains(query))
      })
      .map(|(index, _)| index)
      .collect::<Vec<_>>();
    indexes.sort_by_key(|index| {
      let profile = &self.snapshot.profiles[*index];
      let order = LIST_ORDER
        .iter()
        .position(|kind| *kind == listed_kind(profile))
        .unwrap_or(LIST_ORDER.len());
      (order, profile.name.to_lowercase())
    });
    indexes
  }

  /// The visible profiles under one section per kind. `row` builds the row
  /// of each profile.
  fn grouped_profile_rows(
    &self,
    row: impl Fn(usize, &ConnectionProfile) -> Row<Item>,
  ) -> Vec<Row<Item>> {
    let mut rows = Vec::new();
    let mut current = None;
    for index in self.visible_profiles() {
      let profile = &self.snapshot.profiles[index];
      let kind = listed_kind(profile);
      if current != Some(kind) {
        current = Some(kind);
        rows.push(Row::section(tr(self.lang, kind.label_key())));
      }
      rows.push(row(index, profile));
    }
    if rows.is_empty() {
      let key = if self.job.is_some() {
        "control_center.loading"
      } else if self.search.is_some() {
        "control_center.net_no_match"
      } else {
        "control_center.net_no_connections"
      };
      rows.push(Row::info(tr(self.lang, key), ""));
    }
    rows
  }

  /// What a profile row says about its state.
  fn profile_detail(&self, profile: &ConnectionProfile) -> String {
    let mut parts = Vec::new();
    if profile.active {
      parts.push(self.profile_status(profile));
    }
    if let Some(port_type) = &profile.port_type {
      parts.push(format!(
        "{} ({port_type})",
        tr(self.lang, "control_center.net_port")
      ));
    }
    if profile.read_only {
      parts.push(tr(self.lang, "control_center.net_read_only").to_owned());
    }
    parts.join("  ·  ")
  }

  /// Connections: Add and Refresh, then every profile by kind.
  pub(super) fn connections_rows(&self) -> Vec<Row<Item>> {
    let mut rows = vec![
      Row::section(tr(self.lang, "control_center.section_actions")),
      Row::submenu(
        Item::Open(NetworkPage::AddConnection),
        tr(self.lang, "control_center.net_add_connection"),
      )
      .icon(icons::ADD),
      Row::action(Item::Refresh, tr(self.lang, "control_center.refresh")).icon(icons::REFRESH),
    ];
    rows.extend(self.grouped_profile_rows(|index, profile| {
      Row::submenu(
        Item::Profile(index),
        argvus_control_center_core::sanitize::terminal_text(&profile.name),
      )
      .detail(self.profile_detail(profile))
    }));
    rows
  }

  /// Activate a connection: every profile by kind; Enter brings it up or
  /// down.
  pub(super) fn activate_rows(&self) -> Vec<Row<Item>> {
    let mut rows = self.grouped_profile_rows(|index, profile| {
      let detail = if profile.active {
        self.profile_status(profile)
      } else {
        String::new()
      };
      Row::action(
        Item::Profile(index),
        argvus_control_center_core::sanitize::terminal_text(&profile.name),
      )
      .detail(detail)
    });
    rows.extend(self.refresh_section());
    rows
  }

  /// Add connection: the kinds by group, then the VPN plugins and imports.
  pub(super) fn add_rows(&self) -> Vec<Row<Item>> {
    let mut rows = Vec::new();
    for (group, title) in [
      (KindGroup::Hardware, "control_center.net_group_hardware"),
      (KindGroup::Virtual, "control_center.net_group_virtual"),
    ] {
      rows.push(Row::section(tr(self.lang, title)));
      for kind in ConnectionKind::ADDABLE
        .into_iter()
        .filter(|kind| kind.group() == group)
      {
        let row =
          Row::action(Item::AddKind(kind), tr(self.lang, kind.label_key())).icon(kind.icon());
        rows.push(match kind.missing_requirement(&self.capabilities) {
          Some(reason) => row.enabled(false).detail(tr(self.lang, reason)),
          None => row,
        });
      }
    }
    rows.push(Row::section(tr(self.lang, "control_center.vpn")));
    if self.vpn_plugins.is_empty() {
      rows.push(
        Row::action(Item::AddVpn(0), tr(self.lang, "control_center.vpn"))
          .icon(icons::VPN)
          .enabled(false)
          .detail(tr(self.lang, "control_center.net_requires_vpn_plugin")),
      );
    }
    rows.extend(self.vpn_plugins.iter().enumerate().map(|(index, plugin)| {
      Row::action(Item::AddVpn(index), plugin_label(&plugin.name)).icon(icons::VPN)
    }));
    rows.push(Row::section(tr(
      self.lang,
      "control_center.net_group_import",
    )));
    rows.push(
      Row::action(
        Item::ImportVpn(None),
        tr(self.lang, "control_center.net_import_wireguard"),
      )
      .icon(icons::IMPORT),
    );
    rows.extend(self.vpn_plugins.iter().enumerate().map(|(index, plugin)| {
      Row::action(
        Item::ImportVpn(Some(index)),
        self.lang.tr_args(
          "control_center.net_import_plugin",
          [("plugin", plugin_label(&plugin.name))],
        ),
      )
      .icon(icons::IMPORT)
    }));
    rows
  }

  /// Runs a row of the Connections, Activate or Add page. Returns `false`
  /// when the item is not theirs.
  pub(super) fn connections_activate(&mut self, item: Item) -> bool {
    match item {
      Item::Profile(index) => {
        let Some(profile) = self.snapshot.profiles.get(index).cloned() else {
          return true;
        };
        if self.page == NetworkPage::Activate {
          self.toggle_profile(&profile);
        } else {
          self.open_profile(ProfileSelector::Uuid(profile.uuid));
        }
      }
      Item::AddKind(kind) if kind.missing_requirement(&self.capabilities).is_none() => {
        self.open_new_profile(kind, None);
      }
      Item::AddVpn(index) => {
        if let Some(VpnPlugin { service, .. }) = self.vpn_plugins.get(index).cloned() {
          self.open_new_profile(ConnectionKind::Vpn, Some(service));
        }
      }
      Item::ImportVpn(plugin) => {
        self.open_text(
          EditTarget::ImportPath(plugin),
          "control_center.net_import_file",
          "",
          false,
        );
      }
      Item::AddKind(_) => {}
      _ => return false,
    }
    true
  }

  /// `a` on a Connections row: brings the profile up or down.
  pub(super) fn toggle_selected_profile(&mut self) {
    if let Some(Item::Profile(index)) = self.selected_item()
      && let Some(profile) = self.snapshot.profiles.get(index).cloned()
    {
      self.toggle_profile(&profile);
    }
  }

  /// Imports a VPN file, then opens the new profile in the editor.
  pub(super) fn import_vpn(&mut self, plugin: Option<usize>, path: String) {
    if path.is_empty() {
      return;
    }
    let kind = match plugin {
      Some(index) => match self.vpn_plugins.get(index) {
        Some(plugin) => plugin.name.clone(),
        None => return,
      },
      None => "wireguard".to_owned(),
    };
    let owner = self.owner.clone();
    let imported = tr(self.lang, "control_center.net_imported").to_owned();
    self.editor_return = NetworkPage::Connections;
    self.editor = None;
    self.go_to(NetworkPage::Editor);
    self.spawn_editor_job("control_center.net_importing", move |backend| {
      let uuid = backend.import_vpn(&kind, &path)?;
      let profiles = backend.list_profiles()?;
      let draft = backend.load_profile(&uuid, &profiles, &owner)?;
      Ok(EditorResult::Loaded {
        draft: Box::new(draft),
        profiles,
        message: Some(imported),
      })
    });
  }

  /// The profile behind the current detail page, for its "Edit connection"
  /// row: the interface's connection, the saved Wi-Fi network or the VPN.
  pub(super) fn detail_profile_selector(&self) -> Option<ProfileSelector> {
    match self.page {
      NetworkPage::Detail(index) => {
        let interface = self.snapshot.interfaces.get(index)?;
        Some(ProfileSelector::Name {
          name: interface.connection.clone()?,
          device: Some(interface.name.clone()),
        })
      }
      NetworkPage::WifiDetail(index) => {
        let network = self
          .snapshot
          .wifi
          .get(index)
          .filter(|network| network.known)?;
        Some(ProfileSelector::Name {
          name: network.ssid.clone(),
          device: None,
        })
      }
      NetworkPage::VpnDetail(index) => Some(ProfileSelector::Name {
        name: self.snapshot.vpn.get(index)?.name.clone(),
        device: None,
      }),
      _ => None,
    }
  }
}
