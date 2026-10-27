//! Tests of the connection pages: Connections, Activate, Add and the editor.
//! None of them runs `nmcli`: the profiles and drafts are set directly.

use super::*;
use crate::{
  model::{ConnectionProfile, InterfaceInfo},
  profile::{ConnectionDraft, nmcli},
};
use argvus_tui::menu::RowKind;
use crossterm::event::KeyModifiers;

fn app() -> NetworkApp {
  let mut app = NetworkApp::new(
    Lang::for_locale("en-US"),
    argvus_theme::Theme::load(),
    Capabilities::default(),
  );
  app.owner = "boss".into();
  app.snapshot.available = true;
  app.snapshot.interfaces = vec![
    InterfaceInfo {
      name: "eno1".into(),
      kind: "ethernet".into(),
      state: "connected".into(),
      connection: Some("Wired".into()),
      ..Default::default()
    },
    InterfaceInfo {
      name: "eno2".into(),
      kind: "ethernet".into(),
      state: "disconnected".into(),
      ..Default::default()
    },
  ];
  app.snapshot.profiles = vec![
    ConnectionProfile {
      name: "Wired".into(),
      uuid: "uuid-wired".into(),
      kind: "802-3-ethernet".into(),
      device: Some("eno1".into()),
      active: true,
      autoconnect: true,
      ..Default::default()
    },
    ConnectionProfile {
      name: "br0".into(),
      uuid: "uuid-br0".into(),
      kind: "bridge".into(),
      ..Default::default()
    },
    ConnectionProfile {
      name: "Office VPN".into(),
      uuid: "uuid-wg".into(),
      kind: "wireguard".into(),
      ..Default::default()
    },
  ];
  app
}

/// Opens the editor on a new profile of `kind`, as the Add page does.
fn editing(kind: ConnectionKind) -> NetworkApp {
  let mut app = app();
  app.page = NetworkPage::AddConnection;
  app.activate(Item::AddKind(kind));
  assert_eq!(app.page, NetworkPage::Editor);
  app
}

fn row(app: &NetworkApp, item: Item) -> Row<Item> {
  app
    .rows()
    .into_iter()
    .find(|row| row.id() == Some(&item))
    .unwrap_or_else(|| panic!("no row {item:?}"))
}

fn select(app: &mut NetworkApp, item: Item) {
  let rows = app.rows();
  assert!(app.menu.select(&rows, &item), "{item:?} is not selectable");
}

fn type_text(app: &mut NetworkApp, text: &str) {
  for character in text.chars() {
    app.handle_key(KeyEvent::new(KeyCode::Char(character), KeyModifiers::NONE));
  }
}

fn draft(app: &NetworkApp) -> &ConnectionDraft {
  &app.editor.as_ref().expect("an open editor").draft
}

fn screen(app: &mut NetworkApp, width: u16, height: u16) -> String {
  let mut terminal =
    ratatui::Terminal::new(ratatui::backend::TestBackend::new(width, height)).unwrap();
  terminal.draw(|frame| app.draw(frame)).unwrap();
  terminal
    .backend()
    .buffer()
    .content
    .iter()
    .map(|cell| cell.symbol())
    .collect()
}

#[test]
fn connections_lists_add_first_then_profiles_by_kind() {
  let mut app = app();
  app.page = NetworkPage::Connections;
  let rows = app.rows();
  assert_eq!(rows[1].id(), Some(&Item::Open(NetworkPage::AddConnection)));
  let sections = rows
    .iter()
    .filter(|row| row.is_section())
    .map(|row| row.label().to_owned())
    .collect::<Vec<_>>();
  assert_eq!(sections, ["Actions", "Ethernet", "Bridge", "WireGuard"]);
  let wired = row(&app, Item::Profile(0));
  assert_eq!(wired.kind(), RowKind::Submenu);
  assert!(wired.detail_text().unwrap().contains("eno1"));
  select(&mut app, Item::Profile(0));
  assert!(app.footer_hints(&app.rows()).contains("a Disconnect"));
}

#[test]
fn connections_filter_by_name() {
  let mut app = app();
  app.page = NetworkPage::Connections;
  app.handle(KeyCode::Char('/'));
  for character in "vpn".chars() {
    app.handle(KeyCode::Char(character));
  }
  assert_eq!(app.visible_profiles(), [2]);
}

#[test]
fn activate_rows_bring_profiles_up_or_down() {
  let mut app = app();
  app.page = NetworkPage::Activate;
  let wired = row(&app, Item::Profile(0));
  assert_eq!(wired.kind(), RowKind::Action);
  assert!(wired.detail_text().unwrap().starts_with('★'));
}

#[test]
fn add_page_disables_kinds_whose_daemon_is_missing() {
  let mut app = app();
  app.page = NetworkPage::AddConnection;
  assert!(!row(&app, Item::AddKind(ConnectionKind::Team)).is_enabled());
  assert!(!row(&app, Item::AddKind(ConnectionKind::Pppoe)).is_enabled());
  assert!(row(&app, Item::AddKind(ConnectionKind::Bridge)).is_enabled());
  assert!(!row(&app, Item::AddVpn(0)).is_enabled(), "no VPN plugin");
  assert!(row(&app, Item::ImportVpn(None)).is_enabled());
  app.capabilities.has_teamd = true;
  assert!(row(&app, Item::AddKind(ConnectionKind::Team)).is_enabled());
}

#[test]
fn a_new_profile_opens_with_a_free_name_and_its_breadcrumb() {
  let app = editing(ConnectionKind::Ethernet);
  assert_eq!(draft(&app).value("connection.id"), "Ethernet 1");
  assert_eq!(
    app.breadcrumb(),
    "Hardware & Connectivity > Network & Wi-Fi > Connections > Add connection > Ethernet"
  );
  let titles = app
    .rows()
    .iter()
    .filter(|row| matches!(row.kind(), RowKind::CollapsibleSection { .. }))
    .map(|row| row.label().to_owned())
    .collect::<Vec<_>>();
  assert_eq!(titles, ["General", "Ethernet", "IPv4", "IPv6"]);
  assert!(row(&app, Item::Save).is_enabled());
  assert!(app.rows().iter().all(|row| row.id() != Some(&Item::Delete)));
}

#[test]
fn arrows_change_choices_and_the_gateway_follows_the_method() {
  let mut app = editing(ConnectionKind::Ethernet);
  assert!(!row(&app, Item::Field("ipv4.gateway")).is_enabled());
  select(&mut app, Item::Field("ipv4.method"));
  app.handle(KeyCode::Right);
  assert_eq!(draft(&app).value("ipv4.method"), "manual");
  assert!(row(&app, Item::Field("ipv4.gateway")).is_enabled());
  // Manual without an address cannot be saved; the Save row says why.
  let save = row(&app, Item::Save);
  assert!(!save.is_enabled());
  assert!(save.detail_text().unwrap().contains("Addresses"));
  app.handle(KeyCode::Left);
  assert_eq!(draft(&app).value("ipv4.method"), "auto");
}

#[test]
fn list_fields_add_edit_and_remove_entries_on_their_page() {
  let mut app = editing(ConnectionKind::Ethernet);
  app.activate(Item::Field("ipv4.dns"));
  assert_eq!(app.page, NetworkPage::EditorList("ipv4.dns"));
  assert!(app.breadcrumb().ends_with("Ethernet > DNS servers"));
  app.activate(Item::AddListEntry);
  assert!(app.captures_text());
  type_text(&mut app, "1.1.1.1");
  app.handle(KeyCode::Enter);
  app.activate(Item::AddListEntry);
  type_text(&mut app, "not-an-ip");
  app.handle(KeyCode::Enter);
  assert_eq!(draft(&app).value("ipv4.dns"), "1.1.1.1");
  assert_eq!(app.status.as_ref().unwrap().kind, StatusKind::Error);
  app.activate(Item::AddListEntry);
  type_text(&mut app, "9.9.9.9");
  app.handle(KeyCode::Enter);
  assert_eq!(draft(&app).value("ipv4.dns"), "1.1.1.1, 9.9.9.9");
  // Clearing an entry removes it.
  app.activate(Item::ListEntry(0));
  for _ in 0..7 {
    app.handle(KeyCode::Backspace);
  }
  app.handle(KeyCode::Enter);
  assert_eq!(draft(&app).value("ipv4.dns"), "9.9.9.9");
  // Esc returns to the editor, on the list's row.
  app.handle(KeyCode::Esc);
  assert_eq!(app.page, NetworkPage::Editor);
  let rows = app.rows();
  assert_eq!(app.menu.selected_id(&rows), Some(Item::Field("ipv4.dns")));
}

#[test]
fn secrets_are_typed_masked_and_never_shown() {
  let mut app = editing(ConnectionKind::Wifi);
  assert!(
    app
      .rows()
      .iter()
      .all(|row| row.id() != Some(&Item::Field("802-11-wireless-security.psk")))
  );
  select(&mut app, Item::Field("802-11-wireless-security.key-mgmt"));
  app.handle(KeyCode::Right);
  app.activate(Item::Field("802-11-wireless-security.psk"));
  let (_, editor) = app.text_edit.as_ref().unwrap();
  assert!(editor.is_masked());
  type_text(&mut app, "correct horse");
  app.handle(KeyCode::Enter);
  assert_eq!(
    draft(&app).secret("802-11-wireless-security.psk"),
    Some("correct horse")
  );
  let password = row(&app, Item::Field("802-11-wireless-security.psk"));
  assert!(!password.detail_text().unwrap().contains("correct"));
  assert!(!screen(&mut app, 100, 60).contains("correct horse"));
}

#[test]
fn escape_with_changes_asks_before_discarding() {
  let mut app = editing(ConnectionKind::Bridge);
  app.handle(KeyCode::Esc);
  let (confirm, state) = app.editor_confirm.as_ref().unwrap();
  assert_eq!(*confirm, EditorConfirm::Discard);
  assert!(!state.is_confirm_focused(), "focus starts on Cancel");
  assert!(!app.captures_text());
  app.handle(KeyCode::Enter);
  assert!(app.editor.is_some(), "Enter on Cancel keeps the draft");
  app.handle(KeyCode::Esc);
  app.handle(KeyCode::Char('y'));
  assert!(app.editor.is_none());
  assert_eq!(app.page, NetworkPage::AddConnection);
}

#[test]
fn controllers_pick_their_ports_on_a_page() {
  let mut app = editing(ConnectionKind::Bridge);
  app.activate(Item::OpenPorts);
  assert_eq!(app.page, NetworkPage::EditorPorts);
  assert_eq!(app.port_candidates(), ["eno1", "eno2"]);
  app.activate(Item::Port(1));
  assert!(draft(&app).ports.contains("eno2"));
  assert_eq!(
    row(&app, Item::Port(1)).kind(),
    RowKind::Toggle { on: true }
  );
  app.handle(KeyCode::Esc);
  assert_eq!(app.page, NetworkPage::Editor);
  assert_eq!(row(&app, Item::OpenPorts).detail_text(), Some("eno2"));
}

#[test]
fn wireguard_peers_are_added_edited_and_removed() {
  let mut app = editing(ConnectionKind::WireGuard);
  assert!(!row(&app, Item::GenerateKey).is_enabled(), "no wg tool");
  app.activate(Item::OpenPeers);
  app.activate(Item::AddPeer);
  assert_eq!(app.page, NetworkPage::EditorPeer(0));
  app.activate(Item::PeerField(PeerField::Endpoint));
  type_text(&mut app, "vpn.example.org:51820");
  app.handle(KeyCode::Enter);
  assert_eq!(draft(&app).peers[0].endpoint, "vpn.example.org:51820");
  // A peer without its public key keeps the profile from being saved.
  app.handle(KeyCode::Esc);
  app.handle(KeyCode::Esc);
  assert!(!row(&app, Item::Save).is_enabled());
  app.activate(Item::OpenPeers);
  app.activate(Item::Peer(0));
  app.activate(Item::RemovePeer);
  assert_eq!(app.page, NetworkPage::EditorPeers);
  assert!(draft(&app).peers.is_empty());
}

#[test]
fn saving_an_active_profile_asks_first_and_deleting_starts_on_cancel() {
  let mut app = app();
  let settings = nmcli::parse_settings(
    "connection.id:Wired\nconnection.uuid:uuid-wired\nconnection.type:802-3-ethernet\n",
  );
  app.editor = Some(EditorState {
    draft: nmcli::draft_from_settings(&settings, "boss"),
    collapsed: Default::default(),
    public_key: None,
  });
  app.page = NetworkPage::Editor;
  assert!(!row(&app, Item::Save).is_enabled(), "nothing changed");
  assert_eq!(
    row(&app, Item::ToggleActive).label(),
    tr(app.lang, "control_center.disconnect")
  );
  assert_eq!(app.rows().last().unwrap().id(), Some(&Item::Delete));
  app.activate(Item::Field("connection.autoconnect"));
  app.activate(Item::Save);
  assert_eq!(
    app.editor_confirm.as_ref().map(|(confirm, _)| *confirm),
    Some(EditorConfirm::SaveActive)
  );
  app.handle(KeyCode::Esc);
  assert!(app.editor_confirm.is_none());
  app.activate(Item::Delete);
  let (confirm, state) = app.editor_confirm.as_ref().unwrap();
  assert_eq!(*confirm, EditorConfirm::Delete);
  assert!(!state.is_confirm_focused());
  assert_eq!(app.footer_hints(&app.rows()), confirm_hints(app.lang));
}

#[test]
fn sections_fold_and_unfold() {
  let mut app = editing(ConnectionKind::Ethernet);
  let before = app.rows().len();
  app.activate(Item::Section(SectionId::Ipv6));
  assert!(app.rows().len() < before);
  assert!(
    app
      .rows()
      .iter()
      .all(|row| row.id() != Some(&Item::Field("ipv6.method")))
  );
  app.activate(Item::Section(SectionId::Ipv6));
  assert_eq!(app.rows().len(), before);
}

#[test]
fn every_kind_draws_at_80_by_24_without_buttons() {
  for kind in ConnectionKind::ADDABLE {
    let mut app = app();
    app.capabilities.has_teamd = true;
    app.capabilities.has_pppd = true;
    app.page = NetworkPage::AddConnection;
    app.open_new_profile(kind, None);
    let text = screen(&mut app, 80, 24);
    assert!(text.contains("General"), "{kind:?}");
    // Toggles draw `[ ]`; buttons would read `[ Save ]`.
    for button in ["[ Save", "[ Cancel", "[ OK"] {
      assert!(!text.contains(button), "{kind:?}: {button}");
    }
  }
  let mut app = app();
  app.page = NetworkPage::Connections;
  assert!(screen(&mut app, 80, 24).contains("Add connection"));
}

#[test]
fn detail_pages_open_their_profile_in_the_editor() {
  let mut app = app();
  app.page = NetworkPage::Detail(0);
  assert!(matches!(
    app.detail_profile_selector(),
    Some(editor::ProfileSelector::Name { ref name, ref device })
      if name == "Wired" && device.as_deref() == Some("eno1")
  ));
  app.page = NetworkPage::Detail(1);
  assert!(app.detail_profile_selector().is_none());
  assert!(!row(&app, Item::EditConnection).is_enabled());
}
