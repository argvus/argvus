//! Terminal UI rendering and interaction of the Network page.
mod input;
mod jobs;
mod render;
mod rows;

#[cfg(test)]
mod connection_tests;
mod connections;
mod editor;

use crate::{
  backend::{NetworkBackend, VpnPlugin},
  model::{NetworkPage, NetworkSnapshot},
  profile::{ConnectionKind, PeerField, schema::SectionId},
};
use argvus_control_center_core::navigation::HomeCardId;
use argvus_control_center_core::{
  capabilities::Capabilities,
  config::AppConfig,
  jobs::{JobHandle, JobManager, JobState},
  privileged::{PrivilegedOperation, PrivilegedRequest, SystemSettingsOperation},
  process::SystemProcessRunner,
  sanitize::terminal_text,
};
use argvus_i18n::{Lang, tr};
use argvus_theme::Theme;
use argvus_tui::{
  components::{StatusKind, StatusMessage},
  confirm::{ConfirmDialog, ConfirmOutcome, ConfirmState, draw_confirm},
  hints::{HintContext, confirm_hints, hints},
  icons,
  menu::{MenuEvent, MenuState, MenuStyle, Row, draw_menu},
  page::{shell, status},
  text_editor::{EditorOutcome, TextEditor},
};
use crossterm::event::{KeyCode, KeyEvent};
use editor::{EditTarget, EditorConfirm, EditorResult, EditorState};
use ratatui::{
  Frame,
  text::Line,
  widgets::{Block, Clear, Paragraph},
};

pub struct NetworkApp {
  pub page: NetworkPage,
  detail_parent: NetworkPage,
  menu: MenuState,
  list_height: u16,
  snapshot: NetworkSnapshot,
  job: Option<JobHandle<Result<NetworkSnapshot, String>>>,
  updates: Option<std::sync::mpsc::Receiver<NetworkSnapshot>>,
  requested_page: NetworkPage,
  refreshed: Option<std::time::Instant>,
  action: Option<JobHandle<Result<String, String>>>,
  jobs: JobManager,
  lang: Lang,
  theme: Theme,
  capabilities: Capabilities,
  password: Option<String>,
  pending_wifi: Option<usize>,
  dns_input: Option<String>,
  search: Option<String>,
  status: Option<StatusMessage>,
  scan: bool,
  confirm_forget: bool,
  confirmation: ConfirmState,
  /// The connection profile being edited, once loaded.
  editor: Option<EditorState>,
  /// The page the editor returns to.
  editor_return: NetworkPage,
  /// Loading, saving or deleting the edited profile.
  editor_job: Option<JobHandle<Result<EditorResult, String>>>,
  /// A confirmation asked by the editor.
  editor_confirm: Option<(EditorConfirm, ConfirmState)>,
  /// A text field being typed, and what it writes to.
  text_edit: Option<(EditTarget, TextEditor)>,
  /// The user a private connection profile is limited to.
  owner: String,
  /// VPN plugins installed, read when the Add page opens.
  vpn_plugins: Vec<VpnPlugin>,
}

/// Stable identity of a Network menu row. List items carry their index in
/// the snapshot, so filtering never points an action at another network.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Item {
  /// A page opened from the Network home.
  Open(NetworkPage),
  /// Index into `snapshot.interfaces`; Enter opens its details.
  Interface(usize),
  /// Index into `snapshot.wifi`; Enter connects.
  WifiNetwork(usize),
  /// Index into `snapshot.vpn`; Enter connects.
  VpnConnection(usize),
  WifiToggle,
  Connect,
  Disconnect,
  Forget,
  Refresh,
  DnsManual,
  DnsAutomatic,
  /// Opens the profile of the current detail page in the editor.
  EditConnection,
  /// Index into `snapshot.profiles`: Enter edits it (Connections) or brings
  /// it up or down (Activate).
  Profile(usize),
  AddKind(ConnectionKind),
  /// Index into `vpn_plugins`.
  AddVpn(usize),
  /// Import a file of a plugin (index into `vpn_plugins`), or WireGuard.
  ImportVpn(Option<usize>),
  /// A field of the edited profile, by key.
  Field(&'static str),
  /// Folds or unfolds an editor section.
  Section(SectionId),
  OpenPorts,
  /// Index into the port candidates.
  Port(usize),
  OpenPeers,
  /// Index into the edited profile's peers.
  Peer(usize),
  AddPeer,
  PeerField(PeerField),
  RemovePeer,
  /// Index into the entries of the list page's field.
  ListEntry(usize),
  AddListEntry,
  GenerateKey,
  Save,
  Cancel,
  /// Brings the edited profile up or down.
  ToggleActive,
  Delete,
}

impl NetworkApp {
  /// Replaces the semantic theme used by this page.
  pub fn set_theme(&mut self, theme: &Theme) {
    self.theme = theme.clone();
  }

  /// Builds the Network page on the home list, with no snapshot loaded.
  pub fn new(lang: Lang, theme: Theme, capabilities: Capabilities) -> Self {
    Self {
      page: NetworkPage::Home,
      detail_parent: NetworkPage::Interfaces,
      menu: MenuState::default(),
      list_height: 0,
      snapshot: Default::default(),
      job: None,
      updates: None,
      requested_page: NetworkPage::Home,
      refreshed: None,
      action: None,
      jobs: JobManager::default(),
      lang,
      theme,
      capabilities,
      password: None,
      pending_wifi: None,
      dns_input: None,
      search: None,
      status: None,
      scan: false,
      confirm_forget: false,
      confirmation: ConfirmState::new(),
      editor: None,
      editor_return: NetworkPage::Connections,
      editor_job: None,
      editor_confirm: None,
      text_edit: None,
      owner: std::env::var("USER")
        .or_else(|_| std::env::var("LOGNAME"))
        .unwrap_or_default(),
      vpn_plugins: Vec::new(),
    }
  }

  /// Replaces the snapshot. A detail page follows the same network even when
  /// a refresh reorders the lists, and falls back to its list when it is gone.
  fn accept_snapshot(&mut self, snapshot: NetworkSnapshot) {
    let interface = match self.page {
      NetworkPage::Detail(i) => self.snapshot.interfaces.get(i).map(|v| v.name.clone()),
      _ => None,
    };
    let wifi = match self.page {
      NetworkPage::WifiDetail(i) => self.snapshot.wifi.get(i).map(|w| w.ssid.clone()),
      _ => None,
    };
    let vpn = match self.page {
      NetworkPage::VpnDetail(i) => self.snapshot.vpn.get(i).map(|v| v.name.clone()),
      _ => None,
    };
    self.snapshot = snapshot;
    if let (NetworkPage::Detail(_), Some(name)) = (self.page, interface) {
      self.page = match self.snapshot.interfaces.iter().position(|v| v.name == name) {
        Some(index) => NetworkPage::Detail(index),
        None => NetworkPage::Interfaces,
      };
    }
    if let (NetworkPage::WifiDetail(_), Some(ssid)) = (self.page, wifi) {
      self.page = match self.snapshot.wifi.iter().position(|w| w.ssid == ssid) {
        Some(index) => NetworkPage::WifiDetail(index),
        None => NetworkPage::Wifi,
      };
    }
    if let (NetworkPage::VpnDetail(_), Some(name)) = (self.page, vpn) {
      self.page = match self.snapshot.vpn.iter().position(|v| v.name == name) {
        Some(index) => NetworkPage::VpnDetail(index),
        None => NetworkPage::Vpn,
      };
    }
  }
  fn action<F>(&mut self, label: &'static str, task: F)
  where
    F: FnOnce(NetworkBackend<SystemProcessRunner>) -> Result<(), crate::backend::NetworkError>
      + Send
      + 'static,
  {
    if self.action.is_some() {
      return;
    }
    self.cancel_refresh_for_action();
    let cap = self.capabilities.clone();
    let success = self.action_label(label);
    self.status = Some(StatusMessage {
      kind: StatusKind::Info,
      text: format!("{}...", self.action_label(label)),
    });
    self.action = Some(self.jobs.spawn(move |_| {
      Ok(
        task(NetworkBackend::new(SystemProcessRunner, cap))
          .map(|_| success)
          .map_err(|e| e.to_string()),
      )
    }));
  }
  /// Whether the host has a Wi-Fi card, from the device inventory (or a
  /// Wi-Fi list kept from an earlier load).
  fn has_wifi_hardware(&self) -> bool {
    self
      .snapshot
      .interfaces
      .iter()
      .any(|interface| crate::backend::is_wifi_kind(&interface.kind))
      || !self.snapshot.wifi.is_empty()
  }

  /// Drops a refresh in progress so that an action can start now instead of
  /// being ignored until the refresh ends. The action refreshes the page
  /// itself when it finishes, so nothing the refresh would show is lost.
  fn cancel_refresh_for_action(&mut self) {
    if let Some(job) = self.job.take() {
      job.cancel();
    }
    self.updates = None;
    self.scan = false;
  }
  fn home_pages(&self) -> Vec<NetworkPage> {
    let mut pages = vec![
      NetworkPage::Status,
      NetworkPage::Connections,
      NetworkPage::Activate,
      NetworkPage::Interfaces,
      NetworkPage::Ethernet,
    ];
    if self.has_wifi_hardware() {
      pages.push(NetworkPage::Wifi);
    }
    let has_vpn = !self.snapshot.vpn.is_empty()
      || self
        .snapshot
        .interfaces
        .iter()
        .any(|i| i.kind.to_ascii_lowercase().contains("vpn"))
      || self
        .snapshot
        .interfaces
        .iter()
        .any(|i| i.kind.to_ascii_lowercase().contains("wireguard"));
    if has_vpn {
      pages.push(NetworkPage::Vpn);
    }
    pages.extend([NetworkPage::Dns, NetworkPage::Proxy]);
    pages
  }
  fn connect_wifi(&mut self, index: usize) {
    let Some(network) = self.snapshot.wifi.get(index).cloned() else {
      return;
    };
    if !network.security.is_empty()
      && network.security != "--"
      && !network.security.eq_ignore_ascii_case("none")
    {
      self.pending_wifi = Some(index);
      self.password = Some(String::new());
    } else {
      self.run_wifi(index, None);
    }
  }
  /// Index into the snapshot of the interface, Wi-Fi network or VPN shown by
  /// a detail page.
  fn current_index(&self) -> Option<usize> {
    match self.page {
      NetworkPage::Detail(index)
      | NetworkPage::WifiDetail(index)
      | NetworkPage::VpnDetail(index) => Some(index),
      _ => None,
    }
  }
  /// Whether the page is a list that accepts `/` to filter.
  fn is_list_page(&self) -> bool {
    matches!(
      self.page,
      NetworkPage::Interfaces
        | NetworkPage::Ethernet
        | NetworkPage::Wifi
        | NetworkPage::Vpn
        | NetworkPage::Connections
        | NetworkPage::Activate
    )
  }

  /// Opens the details of the Wi-Fi network or VPN under the cursor (`i`).
  fn open_details(&mut self) {
    match self.selected_item() {
      Some(Item::WifiNetwork(i)) => self.open_page_detail(NetworkPage::WifiDetail(i)),
      Some(Item::VpnConnection(i)) => self.open_page_detail(NetworkPage::VpnDetail(i)),
      _ => {}
    }
  }
  /// Moves into a detail page, remembering the page it was opened from.
  fn open_page_detail(&mut self, page: NetworkPage) {
    self.detail_parent = self.page;
    self.page = page;
    self.menu = MenuState::default();
    self.reload();
  }
  /// Opens a page from the Network home.
  fn open_home_page(&mut self, page: NetworkPage) {
    self.page = page;
    self.menu = MenuState::default();
    self.start_refresh(false);
  }
  /// Runs the action of a menu row.
  fn activate(&mut self, item: Item) {
    if self.page.is_editor() && self.editor_activate(item) {
      return;
    }
    if self.connections_activate(item) {
      return;
    }
    match item {
      Item::Open(NetworkPage::AddConnection) => {
        self.refresh_vpn_plugins();
        self.open_home_page(NetworkPage::AddConnection);
      }
      Item::Open(page) => self.open_home_page(page),
      Item::EditConnection => {
        if let Some(selector) = self.detail_profile_selector() {
          self.open_profile(selector);
        }
      }
      Item::Interface(i) => self.open_page_detail(NetworkPage::Detail(i)),
      Item::WifiNetwork(i) => self.connect_wifi(i),
      Item::VpnConnection(i) => self.connect_vpn(i),
      Item::WifiToggle => self.toggle_wifi(),
      Item::Connect => self.connect_current(),
      Item::Disconnect => self.disconnect_current(),
      Item::Forget => self.forget_selected(),
      Item::Refresh => self.start_refresh(self.page == NetworkPage::Wifi),
      Item::DnsManual => self.dns_input = Some(String::new()),
      Item::DnsAutomatic => self.apply_dns(""),
      // Rows of the connection pages, handled above (or not shown).
      Item::Profile(_)
      | Item::AddKind(_)
      | Item::AddVpn(_)
      | Item::ImportVpn(_)
      | Item::Field(_)
      | Item::Section(_)
      | Item::OpenPorts
      | Item::Port(_)
      | Item::OpenPeers
      | Item::Peer(_)
      | Item::AddPeer
      | Item::PeerField(_)
      | Item::RemovePeer
      | Item::ListEntry(_)
      | Item::AddListEntry
      | Item::GenerateKey
      | Item::Save
      | Item::Cancel
      | Item::ToggleActive
      | Item::Delete => {}
    }
  }
  /// Connects the network shown by the current detail page.
  fn connect_current(&mut self) {
    match self.page {
      NetworkPage::Detail(i) => self.connect_interface(i),
      NetworkPage::WifiDetail(i) => self.connect_wifi(i),
      NetworkPage::VpnDetail(i) => self.connect_vpn(i),
      _ => {}
    }
  }
  /// Disconnects the network shown by the current detail page.
  fn disconnect_current(&mut self) {
    match self.page {
      NetworkPage::Detail(i) => self.disconnect_interface(i),
      NetworkPage::WifiDetail(i) => {
        if let Some(name) = self.snapshot.wifi.get(i).map(|w| w.ssid.clone()) {
          self.action("Desconectando", move |b| b.connection_action(&name, "down"));
        }
      }
      NetworkPage::VpnDetail(i) => {
        if let Some(name) = self.snapshot.vpn.get(i).map(|v| v.name.clone()) {
          self.action("Desconectando", move |b| b.connection_action(&name, "down"));
        }
      }
      _ => {}
    }
  }
  /// Brings an interface up: its connection when it has one, else the device.
  fn connect_interface(&mut self, i: usize) {
    if let Some(connection) = self
      .snapshot
      .interfaces
      .get(i)
      .and_then(|v| v.connection.clone())
    {
      self.action("Conectando", move |b| {
        b.connection_action(&connection, "up")
      });
    } else if let Some(name) = self.snapshot.interfaces.get(i).map(|v| v.name.clone()) {
      self.action("Conectando", move |b| b.device_action(&name, "connect"));
    }
  }
  /// Takes an interface device down.
  fn disconnect_interface(&mut self, i: usize) {
    if let Some(name) = self.snapshot.interfaces.get(i).map(|v| v.name.clone()) {
      self.action("Desconectando", move |b| {
        b.device_action(&name, "disconnect")
      });
    }
  }
  /// Brings a VPN connection up.
  fn connect_vpn(&mut self, i: usize) {
    if let Some(name) = self.snapshot.vpn.get(i).map(|v| v.name.clone()) {
      self.action("Conectando", move |b| b.connection_action(&name, "up"));
    }
  }
  fn forget_selected(&mut self) {
    if let Some(i) = self.current_index()
      && self.snapshot.wifi.get(i).is_some()
    {
      self.confirmation = ConfirmState::new();
      self.confirm_forget = true;
    }
  }
  fn perform_forget(&mut self) {
    if let Some(i) = self.current_index()
      && let Some(name) = self
        .snapshot
        .wifi
        .get(i)
        .map(|network| network.ssid.clone())
    {
      self.action("Esquecendo", move |backend| {
        backend.connection_action(&name, "delete")
      });
    }
  }
  fn toggle_wifi(&mut self) {
    let enabled = self.snapshot.wifi_enabled != Some(true);
    self.action("Alterando Wi-Fi", move |b| b.set_wifi_enabled(enabled));
  }
  fn apply_dns(&mut self, raw: &str) {
    let values: Vec<String> = raw
      .split(|c: char| c == ',' || c.is_whitespace())
      .filter(|v| !v.is_empty())
      .map(str::to_owned)
      .collect();
    if values
      .iter()
      .all(|v| crate::backend::valid_ipv4(v) || crate::backend::valid_ipv6(v))
    {
      if let Some(connection) = self
        .snapshot
        .interfaces
        .iter()
        .find(|v| v.state == "connected")
        .and_then(|v| v.connection.clone())
      {
        let automatic = values.is_empty();
        self.apply_dns_privileged(connection, values, automatic);
      }
    } else {
      self.status = Some(StatusMessage {
        kind: StatusKind::Error,
        text: tr(self.lang, "control_center.invalid_dns_server").into(),
      });
    }
  }
  fn apply_dns_privileged(&mut self, connection: String, values: Vec<String>, automatic: bool) {
    if self.action.is_some() {
      return;
    }
    self.cancel_refresh_for_action();
    let executable = match std::env::current_exe() {
      Ok(path) => path.to_string_lossy().into_owned(),
      Err(error) => {
        self.status = Some(StatusMessage {
          kind: StatusKind::Error,
          text: error.to_string(),
        });
        return;
      }
    };
    let lang = self.lang;
    self.status = Some(StatusMessage {
      kind: StatusKind::Info,
      text: self.action_label("Aplicando DNS"),
    });
    self.action = Some(self.jobs.spawn(move |_| {
      Ok(apply_dns_privileged(
        executable, lang, connection, automatic, values,
      ))
    }));
  }
}

/// Joins values with commas, or `—` when there are none.
fn joined_or_dash(values: &[String]) -> String {
  if values.is_empty() {
    "—".into()
  } else {
    values.join(", ")
  }
}

fn apply_dns_privileged(
  executable: String,
  lang: Lang,
  connection: String,
  automatic: bool,
  values: Vec<String>,
) -> Result<String, String> {
  let operation = SystemSettingsOperation::new(SystemProcessRunner, executable);
  let request = PrivilegedRequest::new(
    "network",
    "dns",
    vec![
      connection,
      if automatic {
        "automatic".into()
      } else {
        "manual".into()
      },
      values.join(","),
    ],
  )?;
  let output = operation.execute(&request)?;
  if output.status == Some(0) {
    Ok(tr(lang, "control_center.dns_applied").into())
  } else {
    Err(terminal_text(&String::from_utf8_lossy(&output.stderr)))
  }
}

#[cfg(test)]
mod tests;
