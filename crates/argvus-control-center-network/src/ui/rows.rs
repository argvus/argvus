//! Rows and labels of the Network & Wi-Fi page lists.

use super::*;

impl NetworkApp {
  pub(super) fn action_label(&self, label: &str) -> String {
    let key = match label {
      "Conectando" => "control_center.connecting",
      "Desconectando" => "control_center.disconnecting",
      "Esquecendo" => "control_center.forgetting",
      "Alterando Wi-Fi" => "control_center.changing_wifi",
      "Aplicando DNS" => "control_center.applying_dns",
      other => return other.to_owned(),
    };
    tr(self.lang, key).into()
  }

  /// Label of the current page, as shown in the breadcrumb.
  pub(super) fn page_label(&self) -> String {
    let label = |key| tr(self.lang, key).to_owned();
    match self.page {
      NetworkPage::Home => label("control_center.network"),
      NetworkPage::Status => label("control_center.status"),
      NetworkPage::Interfaces => label("control_center.interfaces"),
      NetworkPage::Wifi => label("control_center.wi_fi"),
      NetworkPage::Ethernet => label("control_center.ethernet"),
      NetworkPage::Vpn => label("control_center.vpn"),
      NetworkPage::Dns => label("control_center.dns"),
      NetworkPage::Proxy => label("control_center.proxy"),
      NetworkPage::Detail(_) => label("control_center.interface"),
      NetworkPage::WifiDetail(i) => self
        .snapshot
        .wifi
        .get(i)
        .map(|w| terminal_text(&w.ssid))
        .unwrap_or_else(|| label("control_center.wi_fi")),
      NetworkPage::VpnDetail(i) => self
        .snapshot
        .vpn
        .get(i)
        .map(|v| terminal_text(&v.name))
        .unwrap_or_else(|| label("control_center.vpn")),
      NetworkPage::Connections => label("control_center.net_connections"),
      NetworkPage::Activate => label("control_center.net_activate_connection"),
      NetworkPage::AddConnection => label("control_center.net_add_connection"),
      NetworkPage::Editor
      | NetworkPage::EditorList(_)
      | NetworkPage::EditorPorts
      | NetworkPage::EditorPeers
      | NetworkPage::EditorPeer(_) => self.editor_breadcrumb().join(" > "),
    }
  }

  pub(super) fn visible_wifi(&self) -> Vec<usize> {
    let mut indexes: Vec<usize> = self
      .snapshot
      .wifi
      .iter()
      .enumerate()
      .filter(|(_, w)| {
        self.search.as_ref().is_none_or(|q| {
          w.ssid
            .to_ascii_lowercase()
            .contains(&q.to_ascii_lowercase())
        })
      })
      .map(|(i, _)| i)
      .collect();
    indexes.sort_by_key(|i| {
      let w = &self.snapshot.wifi[*i];
      (
        !w.connected,
        std::cmp::Reverse(w.signal.unwrap_or(0)),
        w.ssid.clone(),
      )
    });
    indexes
  }

  pub(super) fn visible_interfaces(&self) -> Vec<usize> {
    self
      .snapshot
      .interfaces
      .iter()
      .enumerate()
      .filter(|(_, i)| {
        self.page != NetworkPage::Ethernet || i.kind.to_ascii_lowercase().contains("ethernet")
      })
      .map(|(i, _)| i)
      .collect()
  }

  pub(super) fn visible_vpn(&self) -> Vec<usize> {
    self
      .snapshot
      .vpn
      .iter()
      .enumerate()
      .filter(|(_, v)| {
        self.search.as_ref().is_none_or(|q| {
          v.name
            .to_ascii_lowercase()
            .contains(&q.to_ascii_lowercase())
        })
      })
      .map(|(i, _)| i)
      .collect()
  }

  /// Rows of the current page.
  pub(super) fn rows(&self) -> Vec<Row<Item>> {
    match self.page {
      NetworkPage::Home => self.home_rows(),
      NetworkPage::Status => self.status_rows(),
      NetworkPage::Interfaces | NetworkPage::Ethernet => {
        let mut rows = self.interface_rows();
        rows.extend(self.refresh_section());
        rows
      }
      NetworkPage::Detail(i) => self.interface_detail_rows(i),
      NetworkPage::Wifi => {
        let mut rows = self.wifi_rows();
        rows.extend(self.refresh_section());
        rows
      }
      NetworkPage::WifiDetail(i) => self.wifi_detail_rows(i),
      NetworkPage::Vpn => {
        let mut rows = self.vpn_rows();
        rows.extend(self.refresh_section());
        rows
      }
      NetworkPage::VpnDetail(i) => self.vpn_detail_rows(i),
      NetworkPage::Dns => self.dns_rows(),
      NetworkPage::Proxy => self.proxy_rows(),
      NetworkPage::Connections => self.connections_rows(),
      NetworkPage::Activate => self.activate_rows(),
      NetworkPage::AddConnection => self.add_rows(),
      NetworkPage::Editor => self.editor_rows(),
      NetworkPage::EditorList(key) => self.editor_list_rows(key),
      NetworkPage::EditorPorts => self.editor_port_rows(),
      NetworkPage::EditorPeers => self.editor_peer_rows(),
      NetworkPage::EditorPeer(index) => self.editor_single_peer_rows(index),
    }
  }

  /// The `Edit connection` row of a detail page, opening its profile.
  pub(super) fn edit_connection_row(&self) -> Row<Item> {
    Row::submenu(
      Item::EditConnection,
      tr(self.lang, "control_center.net_edit_connection"),
    )
    .icon(icons::EDIT)
  }

  /// The Network home: one submenu per available page, with its state.
  pub(super) fn home_rows(&self) -> Vec<Row<Item>> {
    let loading = !self.snapshot.available && self.job.is_some();
    let active = self
      .snapshot
      .interfaces
      .iter()
      .find(|i| i.state == "connected");
    let active_iface = active
      .map(|i| i.name.clone())
      .unwrap_or_else(|| tr(self.lang, "control_center.none_247448").into());
    let ipv4 = active
      .and_then(|i| i.ipv4.first().cloned())
      .unwrap_or_else(|| "—".into());
    let wifi_status = if self.snapshot.wifi_enabled == Some(false) {
      tr(self.lang, "control_center.disabled_ac84bd").into()
    } else {
      self
        .snapshot
        .wifi
        .iter()
        .find(|w| w.connected)
        .map(|w| terminal_text(&w.ssid))
        .unwrap_or_else(|| tr(self.lang, "control_center.disconnected").into())
    };
    let active_vpn = self
      .snapshot
      .vpn
      .iter()
      .find(|v| v.active)
      .map(|v| terminal_text(&v.name))
      .unwrap_or_else(|| tr(self.lang, "control_center.inactive_6f56ef").into());
    let dns = if self.snapshot.dns.servers.is_empty() {
      tr(self.lang, "control_center.automatic").into()
    } else {
      self.snapshot.dns.servers.join(", ")
    };
    let proxy = if self.snapshot.proxy.http.is_some() || self.snapshot.proxy.https.is_some() {
      tr(self.lang, "control_center.active_095d39").to_owned()
    } else {
      tr(self.lang, "control_center.disabled").to_owned()
    };
    let ethernet = self
      .snapshot
      .interfaces
      .iter()
      .filter(|i| i.kind.to_ascii_lowercase().contains("ethernet"))
      .count();
    let mut active_connections = self
      .snapshot
      .interfaces
      .iter()
      .filter(|interface| interface.state == "connected" && interface.kind != "loopback")
      .filter_map(|interface| interface.connection.as_deref().map(terminal_text))
      .collect::<Vec<_>>();
    active_connections.dedup();
    let active_connections = if active_connections.is_empty() {
      "—".to_owned()
    } else {
      active_connections.join(", ")
    };

    self
      .home_pages()
      .into_iter()
      .filter_map(|page| {
        let (label, icon, detail): (&str, &'static str, String) = match page {
          NetworkPage::Status => (
            tr(self.lang, "control_center.status"),
            icons::NETWORK,
            format!("{} · {}", self.snapshot.connectivity, active_iface),
          ),
          NetworkPage::Interfaces => (
            tr(self.lang, "control_center.interfaces"),
            icons::LINK,
            format!("{} ({})", self.snapshot.interfaces.len(), ipv4),
          ),
          NetworkPage::Ethernet => (
            tr(self.lang, "control_center.ethernet"),
            icons::ETHERNET,
            ethernet.to_string(),
          ),
          NetworkPage::Wifi => (
            tr(self.lang, "control_center.wi_fi"),
            icons::WIFI,
            wifi_status.clone(),
          ),
          NetworkPage::Vpn => (
            tr(self.lang, "control_center.vpn"),
            icons::VPN,
            active_vpn.clone(),
          ),
          NetworkPage::Dns => (tr(self.lang, "control_center.dns"), icons::DNS, dns.clone()),
          NetworkPage::Proxy => (
            tr(self.lang, "control_center.proxy"),
            icons::PROXY,
            proxy.clone(),
          ),
          NetworkPage::Connections => (
            tr(self.lang, "control_center.net_connections"),
            icons::CONNECTIONS,
            active_connections.clone(),
          ),
          NetworkPage::Activate => (
            tr(self.lang, "control_center.net_activate_connection"),
            icons::ACTIVATE_CONNECTION,
            String::new(),
          ),
          _ => return None,
        };
        // Until the first inventory arrives the counts and addresses are not
        // known yet; showing them as `0` or `None` would read as real values.
        // The proxy comes from the environment and is always known.
        let detail = if loading && page != NetworkPage::Proxy {
          tr(self.lang, "control_center.loading").to_owned()
        } else {
          detail
        };
        Some(
          Row::submenu(Item::Open(page), label)
            .icon(icon)
            .detail(detail),
        )
      })
      .collect()
  }

  /// Connectivity, the active interface and the connections, with actions.
  pub(super) fn status_rows(&self) -> Vec<Row<Item>> {
    let active = self
      .snapshot
      .interfaces
      .iter()
      .find(|i| i.state == "connected");
    // Before the inventory arrives the card is unknown, not missing; and the
    // radio of a host without a card reports `enabled`, which is no switch.
    let wifi_row = match self.snapshot.wifi_enabled {
      _ if !self.snapshot.available => Row::toggle(
        Item::WifiToggle,
        tr(self.lang, "control_center.wi_fi"),
        false,
      )
      .enabled(false)
      .detail(tr(self.lang, "control_center.loading")),
      Some(on) if self.has_wifi_hardware() => {
        Row::toggle(Item::WifiToggle, tr(self.lang, "control_center.wi_fi"), on)
      }
      _ => Row::toggle(
        Item::WifiToggle,
        tr(self.lang, "control_center.wi_fi"),
        false,
      )
      .enabled(false)
      .detail(tr(self.lang, "control_center.unavailable_no_wifi_card")),
    };
    let wifi = if self.snapshot.wifi_enabled == Some(false) {
      tr(self.lang, "control_center.disabled_ac84bd").into()
    } else {
      self
        .snapshot
        .wifi
        .iter()
        .find(|w| w.connected)
        .map(|w| terminal_text(&w.ssid))
        .unwrap_or_else(|| tr(self.lang, "control_center.inactive").into())
    };
    let vpn = self
      .snapshot
      .vpn
      .iter()
      .find(|v| v.active)
      .map(|v| terminal_text(&v.name))
      .unwrap_or_else(|| tr(self.lang, "control_center.inactive_6f56ef").into());
    vec![
      Row::section(tr(self.lang, "control_center.section_actions")),
      wifi_row,
      Row::action(Item::Refresh, tr(self.lang, "control_center.refresh")).icon(icons::REFRESH),
      Row::section(tr(self.lang, "control_center.section_summary")),
      Row::info(
        tr(self.lang, "control_center.connectivity"),
        self.snapshot.connectivity.clone(),
      ),
      Row::info(
        tr(self.lang, "control_center.interface_df13f7"),
        active.map(|i| i.name.clone()).unwrap_or_else(|| "—".into()),
      ),
      Row::info(
        tr(self.lang, "control_center.ipv4"),
        active
          .and_then(|i| i.ipv4.first().cloned())
          .unwrap_or_else(|| "—".into()),
      ),
      Row::info(
        tr(self.lang, "control_center.ipv6"),
        joined_or_dash(active.map(|i| i.ipv6.as_slice()).unwrap_or_default()),
      ),
      Row::info(
        tr(self.lang, "control_center.gateway"),
        active
          .and_then(|i| i.gateway.clone())
          .unwrap_or_else(|| "—".into()),
      ),
      Row::info(tr(self.lang, "control_center.wi_fi_ac2ca3"), wifi),
      Row::info(
        tr(self.lang, "control_center.dns_d80a0d"),
        joined_or_dash(&self.snapshot.dns.servers),
      ),
      Row::info(tr(self.lang, "control_center.vpn_fda889"), vpn),
    ]
  }

  /// One row per visible interface. Enter opens its details.
  pub(super) fn interface_rows(&self) -> Vec<Row<Item>> {
    self
      .visible_interfaces()
      .into_iter()
      .map(|i| {
        let v = &self.snapshot.interfaces[i];
        let connection = v
          .connection
          .clone()
          .unwrap_or_else(|| tr(self.lang, "control_center.no_connection").into());
        let detail = if v.state == "connected" {
          format!(
            "{connection}   ★ {}",
            tr(self.lang, "control_center.connected")
          )
        } else {
          connection
        };
        Row::submenu(Item::Interface(i), format!("{} ({})", v.name, v.kind)).detail(detail)
      })
      .collect()
  }

  /// Details of one interface: its actions, then its properties.
  pub(super) fn interface_detail_rows(&self, i: usize) -> Vec<Row<Item>> {
    let Some(v) = self.snapshot.interfaces.get(i) else {
      return Vec::new();
    };
    let status = if v.state == "connected" {
      format!("★ {}", tr(self.lang, "control_center.connected"))
    } else {
      v.state.clone()
    };
    vec![
      Row::section(tr(self.lang, "control_center.section_actions")),
      Row::action(Item::Connect, tr(self.lang, "control_center.connect")).icon(icons::LINK_ON),
      Row::action(Item::Disconnect, tr(self.lang, "control_center.disconnect"))
        .icon(icons::LINK_OFF),
      self.edit_connection_row().enabled(v.connection.is_some()),
      Row::action(Item::Refresh, tr(self.lang, "control_center.refresh")).icon(icons::REFRESH),
      Row::section(tr(self.lang, "control_center.section_summary")),
      Row::info(tr(self.lang, "control_center.name"), v.name.clone()),
      Row::info(tr(self.lang, "control_center.type"), v.kind.clone()),
      Row::info(tr(self.lang, "control_center.status_bbe39c"), status),
      Row::info(
        tr(self.lang, "control_center.driver"),
        v.driver.clone().unwrap_or_else(|| "—".into()),
      ),
      Row::info(
        tr(self.lang, "control_center.operstate"),
        v.operstate.clone(),
      ),
      Row::info(
        tr(self.lang, "control_center.mac"),
        v.mac.clone().unwrap_or_else(|| "—".into()),
      ),
      Row::info(
        tr(self.lang, "control_center.ipv4"),
        joined_or_dash(&v.ipv4),
      ),
      Row::info(
        tr(self.lang, "control_center.ipv6"),
        joined_or_dash(&v.ipv6),
      ),
      Row::info(
        tr(self.lang, "control_center.mtu"),
        v.mtu.map(|m| m.to_string()).unwrap_or_else(|| "—".into()),
      ),
    ]
  }

  /// One row per visible Wi-Fi network. Enter connects; `i` opens details.
  pub(super) fn wifi_rows(&self) -> Vec<Row<Item>> {
    self
      .visible_wifi()
      .into_iter()
      .map(|i| {
        let w = &self.snapshot.wifi[i];
        let badge = if w.connected {
          format!("   ★ {}", tr(self.lang, "control_center.connected"))
        } else if w.known {
          format!("   ● {}", tr(self.lang, "control_center.saved"))
        } else {
          String::new()
        };
        let signal = w
          .signal
          .map(|s| format!("{s}%"))
          .unwrap_or_else(|| "—".into());
        Row::action(Item::WifiNetwork(i), terminal_text(&w.ssid))
          .detail(format!("{signal}  ·  {}{badge}", w.security))
      })
      .collect()
  }

  /// Details of one Wi-Fi network: its actions, its properties and the
  /// destructive Forget in the Danger zone, last.
  pub(super) fn wifi_detail_rows(&self, i: usize) -> Vec<Row<Item>> {
    let Some(w) = self.snapshot.wifi.get(i) else {
      return Vec::new();
    };
    let status = if w.connected {
      tr(self.lang, "control_center.connected_8b5cb3")
    } else if w.known {
      tr(self.lang, "control_center.saved")
    } else {
      "—"
    };
    vec![
      Row::section(tr(self.lang, "control_center.section_actions")),
      Row::action(Item::Connect, tr(self.lang, "control_center.connect")).icon(icons::LINK_ON),
      Row::action(Item::Disconnect, tr(self.lang, "control_center.disconnect"))
        .icon(icons::LINK_OFF),
      self.edit_connection_row().enabled(w.known),
      Row::action(Item::Refresh, tr(self.lang, "control_center.refresh")).icon(icons::REFRESH),
      Row::section(tr(self.lang, "control_center.section_summary")),
      Row::info(tr(self.lang, "control_center.name"), terminal_text(&w.ssid)),
      Row::info(
        tr(self.lang, "control_center.signal"),
        w.signal
          .map(|s| format!("{s}%"))
          .unwrap_or_else(|| "—".into()),
      ),
      Row::info(
        tr(self.lang, "control_center.security"),
        if w.security.is_empty() {
          "—".into()
        } else {
          w.security.clone()
        },
      ),
      Row::info(
        tr(self.lang, "control_center.frequency"),
        w.frequency.clone().unwrap_or_else(|| "—".into()),
      ),
      Row::info(tr(self.lang, "control_center.status_bbe39c"), status),
      Row::section(tr(self.lang, "control_center.danger_zone")),
      Row::destructive(Item::Forget, tr(self.lang, "control_center.forget")).icon(icons::DELETE),
    ]
  }

  /// One row per VPN connection. Enter connects; `i` opens details.
  pub(super) fn vpn_rows(&self) -> Vec<Row<Item>> {
    self
      .visible_vpn()
      .into_iter()
      .map(|i| {
        let v = &self.snapshot.vpn[i];
        let badge = if v.active {
          format!("   ★ {}", tr(self.lang, "control_center.active_c7cc67"))
        } else {
          String::new()
        };
        Row::action(Item::VpnConnection(i), terminal_text(&v.name))
          .detail(format!("{}{badge}", v.kind))
      })
      .collect()
  }

  /// Details of one VPN connection: its actions, then its properties.
  pub(super) fn vpn_detail_rows(&self, i: usize) -> Vec<Row<Item>> {
    let Some(v) = self.snapshot.vpn.get(i) else {
      return Vec::new();
    };
    let status = if v.active {
      tr(self.lang, "control_center.active_c7cc67")
    } else {
      tr(self.lang, "control_center.inactive_6f56ef")
    };
    vec![
      Row::section(tr(self.lang, "control_center.section_actions")),
      Row::action(Item::Connect, tr(self.lang, "control_center.connect")).icon(icons::LINK_ON),
      Row::action(Item::Disconnect, tr(self.lang, "control_center.disconnect"))
        .icon(icons::LINK_OFF),
      self.edit_connection_row(),
      Row::action(Item::Refresh, tr(self.lang, "control_center.refresh")).icon(icons::REFRESH),
      Row::section(tr(self.lang, "control_center.section_summary")),
      Row::info(tr(self.lang, "control_center.name"), terminal_text(&v.name)),
      Row::info(tr(self.lang, "control_center.type"), v.kind.clone()),
      Row::info(tr(self.lang, "control_center.status_bbe39c"), status),
    ]
  }

  /// The DNS value is editable (`Enter` opens the server list); automatic
  /// DNS and the refresh are actions.
  pub(super) fn dns_rows(&self) -> Vec<Row<Item>> {
    let current = if self.snapshot.dns.servers.is_empty() {
      tr(self.lang, "control_center.automatic").into()
    } else {
      self.snapshot.dns.servers.join(", ")
    };
    vec![
      Row::section(tr(self.lang, "control_center.section_actions")),
      Row::value(
        Item::DnsManual,
        tr(self.lang, "control_center.manual_dns"),
        current,
        None,
      ),
      Row::action(
        Item::DnsAutomatic,
        tr(self.lang, "control_center.automatic_dns"),
      )
      .icon(icons::AUTORENEW),
      Row::action(Item::Refresh, tr(self.lang, "control_center.refresh")).icon(icons::REFRESH),
      Row::section(tr(self.lang, "control_center.section_summary")),
      Row::info(
        tr(self.lang, "control_center.source_2c26d9"),
        terminal_text(&self.snapshot.dns.source),
      ),
      Row::info(
        tr(self.lang, "control_center.servers"),
        joined_or_dash(&self.snapshot.dns.servers),
      ),
      Row::info(
        tr(self.lang, "control_center.search_domains"),
        joined_or_dash(&self.snapshot.dns.search_domains),
      ),
    ]
  }

  /// The proxy variables from the environment, read-only, with a refresh.
  pub(super) fn proxy_rows(&self) -> Vec<Row<Item>> {
    let proxy = &self.snapshot.proxy;
    let value = |v: &Option<String>| v.clone().unwrap_or_else(|| "—".into());
    vec![
      Row::section(tr(self.lang, "control_center.section_actions")),
      Row::action(Item::Refresh, tr(self.lang, "control_center.refresh")).icon(icons::REFRESH),
      Row::section(tr(self.lang, "control_center.section_summary")),
      Row::info("HTTP_PROXY:", value(&proxy.http)),
      Row::info("HTTPS_PROXY:", value(&proxy.https)),
      Row::info("ALL_PROXY:", value(&proxy.all)),
      Row::info("NO_PROXY:", value(&proxy.no_proxy)),
    ]
  }
}
