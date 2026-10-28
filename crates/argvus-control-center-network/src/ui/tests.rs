use super::*;
use crate::model::{InterfaceInfo, VpnConnection, WifiNetwork};
use argvus_control_center_core::capabilities::Capabilities;
use argvus_i18n::Lang;
use argvus_tui::menu::RowKind;

fn app() -> NetworkApp {
  NetworkApp::new(
    Lang::for_locale("en-US"),
    argvus_theme::Theme::load(),
    Capabilities::default(),
  )
}

/// A connected Ethernet interface and two Wi-Fi networks, one saved.
fn with_network(app: &mut NetworkApp) {
  app.snapshot.interfaces = vec![InterfaceInfo {
    name: "eno1".into(),
    kind: "ethernet".into(),
    state: "connected".into(),
    connection: Some("Wired connection 1".into()),
    ipv4: vec!["192.168.1.100".into()],
    ..Default::default()
  }];
  app.snapshot.wifi = vec![
    WifiNetwork {
      ssid: "cafe".into(),
      signal: Some(40),
      security: "WPA2".into(),
      ..Default::default()
    },
    WifiNetwork {
      ssid: "home".into(),
      signal: Some(80),
      security: "WPA2".into(),
      connected: true,
      known: true,
      ..Default::default()
    },
  ];
  app.snapshot.wifi_enabled = Some(true);
  app.snapshot.available = true;
}

#[test]
fn wifi_is_only_listed_when_a_card_exists() {
  let mut app = app();
  assert!(!app.home_pages().contains(&NetworkPage::Wifi));
  app.snapshot.interfaces.push(InterfaceInfo {
    name: "wlan0".into(),
    kind: "wifi".into(),
    ..Default::default()
  });
  assert!(app.home_pages().contains(&NetworkPage::Wifi));
}

#[test]
fn vpn_is_only_listed_when_vpn_is_detected() {
  let mut app = app();
  assert!(!app.home_pages().contains(&NetworkPage::Vpn));
  app.snapshot.vpn.push(VpnConnection {
    name: "company".into(),
    kind: "vpn".into(),
    active: false,
  });
  assert!(app.home_pages().contains(&NetworkPage::Vpn));
  app.snapshot.vpn.clear();
  app.snapshot.interfaces.push(InterfaceInfo {
    name: "wg0".into(),
    kind: "wireguard".into(),
    ..Default::default()
  });
  assert!(app.home_pages().contains(&NetworkPage::Vpn));
}

#[test]
/// The Home is one submenu per available page, and the status is its detail.
fn home_lists_each_page_as_a_submenu_with_its_state() {
  let mut app = app();
  with_network(&mut app);
  app.snapshot.connectivity = "full".into();
  let rows = app.rows();
  // Status, Connections, Activate, Interfaces, Ethernet, Wi-Fi (a card
  // exists), DNS, Proxy. Firewall lives under System Administration on the
  // Control Center Home.
  assert_eq!(rows.len(), 8);
  assert!(rows.iter().all(|row| row.kind() == RowKind::Submenu));
  assert_eq!(rows[0].id(), Some(&Item::Open(NetworkPage::Status)));
  assert_eq!(rows[1].id(), Some(&Item::Open(NetworkPage::Connections)));
  assert_eq!(rows[2].id(), Some(&Item::Open(NetworkPage::Activate)));
  assert_eq!(rows[6].id(), Some(&Item::Open(NetworkPage::Dns)));
  assert_eq!(rows[1].detail_text(), Some("Wired connection 1"));
  assert!(rows[0].detail_text().unwrap().contains("full"));
  assert!(rows[0].detail_text().unwrap().contains("eno1"));
}

#[test]
/// Interface details offer Connect, Disconnect, Edit connection and
/// Refresh; the properties are information and never take the cursor.
fn interface_detail_has_connect_disconnect_and_unselectable_info() {
  let mut app = app();
  with_network(&mut app);
  app.page = NetworkPage::Detail(0);
  let rows = app.rows();
  assert_eq!(rows[1].id(), Some(&Item::Connect));
  assert_eq!(rows[2].id(), Some(&Item::Disconnect));
  assert_eq!(rows[3].id(), Some(&Item::EditConnection));
  assert!(rows[3].is_enabled());
  assert_eq!(rows[4].id(), Some(&Item::Refresh));
  assert!(
    rows
      .iter()
      .any(|row| row.label() == "eno1" || row.detail_text() == Some("eno1"))
  );
  assert!(
    rows
      .iter()
      .filter(|row| row.kind() == RowKind::Info)
      .all(|row| !row.is_selectable())
  );
}

#[test]
/// Enter on a Wi-Fi row connects (it is an action), and `i` opens the
/// network's details, where Connect, Disconnect and Forget live.
fn wifi_row_is_an_action_and_i_opens_its_details() {
  let mut app = app();
  with_network(&mut app);
  app.page = NetworkPage::Wifi;
  let rows = app.rows();
  assert_eq!(rows[0].kind(), RowKind::Action);
  let first = app.visible_wifi()[0];
  assert_eq!(rows[0].id(), Some(&Item::WifiNetwork(first)));
  assert!(app.footer_hints(&rows).contains("i Details"));

  app.handle(KeyCode::Char('i'));
  assert_eq!(app.page, NetworkPage::WifiDetail(first));
  assert_eq!(app.detail_parent, NetworkPage::Wifi);
}

#[test]
/// `i Details` is not offered on Interfaces, where Enter already opens it.
fn details_hint_is_only_offered_on_wifi_and_vpn_rows() {
  let mut app = app();
  with_network(&mut app);
  app.page = NetworkPage::Interfaces;
  let rows = app.rows();
  assert!(!app.footer_hints(&rows).contains("i Details"));
}

#[test]
/// Wi-Fi details: actions first, properties next, and Forget last in the
/// Danger zone.
fn wifi_detail_has_actions_and_forget_last() {
  let mut app = app();
  with_network(&mut app);
  app.page = NetworkPage::WifiDetail(1);
  let rows = app.rows();
  assert_eq!(rows[1].id(), Some(&Item::Connect));
  assert_eq!(rows[2].id(), Some(&Item::Disconnect));
  assert_eq!(rows[3].id(), Some(&Item::EditConnection));
  assert_eq!(rows[4].id(), Some(&Item::Refresh));
  assert_eq!(rows.last().unwrap().id(), Some(&Item::Forget));
  assert_eq!(rows.last().unwrap().kind(), RowKind::Destructive);
}

#[test]
/// Esc returns to the Wi-Fi list with the cursor on the network it opened.
fn escape_from_wifi_details_returns_to_the_network_under_the_cursor() {
  let mut app = app();
  with_network(&mut app);
  app.page = NetworkPage::WifiDetail(0);
  app.detail_parent = NetworkPage::Wifi;
  assert!(!app.handle(KeyCode::Esc));
  assert_eq!(app.page, NetworkPage::Wifi);
  assert_eq!(
    app.menu.selected_id(&app.rows()),
    Some(Item::WifiNetwork(0))
  );
}

#[test]
/// The Wi-Fi switch is a toggle that reflects the radio. Without a Wi-Fi
/// card it stays disabled and says why.
fn wifi_switch_reflects_the_radio_and_is_disabled_without_a_card() {
  let mut app = app();
  with_network(&mut app);
  app.page = NetworkPage::Status;
  let on = app
    .rows()
    .into_iter()
    .find(|row| row.id() == Some(&Item::WifiToggle))
    .unwrap();
  assert_eq!(on.kind(), RowKind::Toggle { on: true });
  assert!(on.is_selectable());

  app.snapshot.wifi_enabled = None;
  let off = app
    .rows()
    .into_iter()
    .find(|row| row.id() == Some(&Item::WifiToggle))
    .unwrap();
  assert!(!off.is_enabled());
  assert!(!off.is_selectable());
  assert!(off.detail_text().is_some());
}

/// The radio of a host without a Wi-Fi card still reports `enabled`; the
/// switch must not offer it. Before the inventory arrives the card is
/// unknown, so the switch says it is loading instead of "no Wi-Fi card".
fn wifi_toggle(app: &NetworkApp) -> Row<Item> {
  app
    .rows()
    .into_iter()
    .find(|row| row.id() == Some(&Item::WifiToggle))
    .unwrap()
}

#[test]
fn wifi_switch_waits_for_the_inventory_and_needs_a_card() {
  let mut app = app();
  app.page = NetworkPage::Status;
  let loading = wifi_toggle(&app);
  assert!(!loading.is_selectable());
  assert_eq!(
    loading.detail_text(),
    Some(tr(app.lang, "control_center.loading"))
  );

  with_network(&mut app);
  app.snapshot.wifi.clear();
  let without_card = wifi_toggle(&app);
  assert!(!without_card.is_selectable());
  assert_eq!(
    without_card.detail_text(),
    Some(tr(app.lang, "control_center.unavailable_no_wifi_card"))
  );
}

#[test]
fn home_says_loading_until_the_inventory_arrives() {
  let mut app = app();
  app.job = Some(app.jobs.spawn(|_| {
    std::thread::sleep(std::time::Duration::from_secs(2));
    Ok(Ok(NetworkSnapshot::default()))
  }));
  let loading = tr(app.lang, "control_center.loading");
  let rows = app.rows();
  let detail = |page: NetworkPage| {
    rows
      .iter()
      .find(|row| row.id() == Some(&Item::Open(page)))
      .and_then(|row| row.detail_text().map(str::to_owned))
  };
  assert_eq!(detail(NetworkPage::Status).as_deref(), Some(loading));
  assert_eq!(detail(NetworkPage::Interfaces).as_deref(), Some(loading));
  assert_ne!(detail(NetworkPage::Proxy).as_deref(), Some(loading));

  // Without a load running (no NetworkManager), the values are shown as is.
  app.job = None;
  let rows = app.rows();
  assert!(rows.iter().all(|row| row.detail_text() != Some(loading)));
}

#[test]
fn an_action_replaces_a_refresh_instead_of_being_ignored() {
  let mut app = app();
  with_network(&mut app);
  app.job = Some(app.jobs.spawn(|_| {
    std::thread::sleep(std::time::Duration::from_secs(2));
    Ok(Ok(NetworkSnapshot::default()))
  }));
  let (_sender, receiver) = std::sync::mpsc::channel();
  app.updates = Some(receiver);

  app.toggle_wifi();

  assert!(app.job.is_none(), "the refresh is dropped");
  assert!(app.updates.is_none());
  assert!(app.action.is_some(), "the action started");
}

#[test]
/// DNS keeps the manual server list as an editable value and automatic DNS
/// as an action.
fn dns_page_has_a_manual_value_and_an_automatic_action() {
  let mut app = app();
  app.page = NetworkPage::Dns;
  let rows = app.rows();
  let manual = rows
    .iter()
    .find(|row| row.id() == Some(&Item::DnsManual))
    .unwrap();
  assert_eq!(manual.kind(), RowKind::Value { step: None });
  let automatic = rows
    .iter()
    .find(|row| row.id() == Some(&Item::DnsAutomatic))
    .unwrap();
  assert_eq!(automatic.kind(), RowKind::Action);
}

#[test]
/// Forget waits for a confirmation that starts on Cancel, and Esc keeps the
/// network.
fn forget_asks_first_and_starts_on_cancel() {
  let mut app = app();
  with_network(&mut app);
  app.page = NetworkPage::WifiDetail(1);
  app.activate(Item::Forget);
  assert!(app.confirm_forget);
  assert!(!app.confirmation.is_confirm_focused());
  assert_eq!(app.footer_hints(&app.rows()), confirm_hints(app.lang));
  assert!(!app.handle(KeyCode::Esc));
  assert!(!app.confirm_forget);
  assert_eq!(app.snapshot.wifi.len(), 2);
}

#[test]
/// Tab no longer moves focus anywhere on the list pages.
fn tab_does_not_change_the_page() {
  let mut app = app();
  with_network(&mut app);
  app.page = NetworkPage::Interfaces;
  assert!(!app.handle(KeyCode::Tab));
  assert_eq!(app.page, NetworkPage::Interfaces);
  assert!(!app.handle(KeyCode::BackTab));
  assert_eq!(app.page, NetworkPage::Interfaces);
}

#[test]
/// The interfaces page draws a single menu list, without button brackets.
fn interfaces_page_draws_one_menu_list() {
  let mut app = app();
  with_network(&mut app);
  app.page = NetworkPage::Interfaces;
  let mut terminal = ratatui::Terminal::new(ratatui::backend::TestBackend::new(90, 25)).unwrap();
  terminal.draw(|frame| app.draw(frame)).unwrap();
  let text = terminal
    .backend()
    .buffer()
    .content
    .iter()
    .map(|cell| cell.symbol())
    .collect::<String>();
  assert!(text.contains("eno1"));
  assert!(!text.contains("[ "));
}
