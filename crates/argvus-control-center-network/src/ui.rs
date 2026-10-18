//! Implements terminal UI rendering and interaction in crate `argvus control center network`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use crate::{
  backend::NetworkBackend,
  model::{NetworkPage, NetworkSnapshot},
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
};
use crossterm::event::KeyCode;
use ratatui::{
  Frame,
  text::Line,
  widgets::{Block, Clear, Paragraph},
};

/// Represents `NetworkApp`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
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
    }
  }
  /// Executes the `reload` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn reload(&mut self) {
    if self.requested_page != self.page
      || self.refreshed.is_none_or(|at| at.elapsed().as_secs() >= 5)
    {
      self.start_refresh(false);
    }
  }
  /// Executes the `start_refresh` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn start_refresh(&mut self, scan: bool) {
    if self.job.is_some() || self.action.is_some() {
      return;
    }
    let cap = self.capabilities.clone();
    self.scan = scan;
    let page = self.page;
    self.requested_page = page;
    let previous = self.snapshot.clone();
    let (sender, receiver) = std::sync::mpsc::channel();
    self.updates = Some(receiver);
    self.job = Some(self.jobs.spawn(move |_| {
      Ok(
        NetworkBackend::new(SystemProcessRunner, cap)
          .load_page(page, scan, previous, |state| {
            let _ = sender.send(state);
          })
          .map_err(|e| e.to_string()),
      )
    }));
  }
  /// Executes the `poll` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn poll(&mut self) -> bool {
    if self.job.is_some() && self.requested_page != self.page {
      if let Some(job) = self.job.take() {
        job.cancel();
      }
      self.updates = None;
      self.scan = false;
      self.start_refresh(false);
    }
    let mut changed = false;
    let updates = self
      .updates
      .as_ref()
      .map(|receiver| receiver.try_iter().collect::<Vec<_>>())
      .unwrap_or_default();
    for snapshot in updates {
      self.accept_snapshot(snapshot);
      changed = true;
    }
    if let Some(job) = &self.job
      && let JobState::Finished(result) = job.try_state()
    {
      self.job = None;
      self.updates = None;
      match result {
        Ok(Ok(snapshot)) => {
          self.accept_snapshot(snapshot);
          self.refreshed = Some(std::time::Instant::now());
          if self.status.is_none() {
            self.status = Some(StatusMessage {
              kind: StatusKind::Success,
              text: if self.scan {
                tr(self.lang, "control_center.networks_updated")
              } else {
                tr(self.lang, "control_center.network_refreshed")
              }
              .into(),
            });
          }
        }
        Ok(Err(error)) | Err(error) => {
          self.status = Some(StatusMessage {
            kind: StatusKind::Error,
            text: error,
          })
        }
      }
      self.scan = false;
      if self.requested_page != self.page {
        self.start_refresh(false);
      }
      changed = true;
    }
    if let Some(job) = &self.action
      && let JobState::Finished(result) = job.try_state()
    {
      self.action = None;
      let (kind, text) = match result {
        Ok(Ok(text)) => (StatusKind::Success, text),
        Ok(Err(error)) | Err(error) => (StatusKind::Error, error),
      };
      self.status = Some(StatusMessage { kind, text });
      self.start_refresh(false);
      changed = true;
    }
    changed
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
  /// Executes the `action` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn action<F>(&mut self, label: &'static str, task: F)
  where
    F: FnOnce(NetworkBackend<SystemProcessRunner>) -> Result<(), crate::backend::NetworkError>
      + Send
      + 'static,
  {
    if self.action.is_some() || self.job.is_some() {
      return;
    }
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
  /// Executes the `action_label` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn action_label(&self, label: &str) -> String {
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
  /// Executes the `home_pages` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn home_pages(&self) -> Vec<NetworkPage> {
    let mut pages = vec![
      NetworkPage::Status,
      NetworkPage::Interfaces,
      NetworkPage::Ethernet,
    ];
    let has_wifi_hardware = self.snapshot.interfaces.iter().any(|i| {
      i.kind.to_ascii_lowercase().contains("wifi")
        || i.kind.to_ascii_lowercase().contains("wireless")
        || i.kind.eq_ignore_ascii_case("802-11-wireless")
    }) || !self.snapshot.wifi.is_empty();
    if has_wifi_hardware {
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
  /// Executes the `breadcrumb` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn breadcrumb(&self) -> String {
    let root = format!(
      "{} > {}",
      HomeCardId::Hardware.label(self.lang),
      tr(self.lang, "control_center.network_wifi")
    );
    if self.page == NetworkPage::Home {
      root
    } else {
      format!("{root} > {}", self.page_label())
    }
  }
  /// Label of the current page, as shown in the breadcrumb.
  fn page_label(&self) -> String {
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
    }
  }
  /// Executes the `connect_wifi` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
  /// Executes the `run_wifi` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn run_wifi(&mut self, index: usize, password: Option<String>) {
    let Some(network) = self.snapshot.wifi.get(index).cloned() else {
      return;
    };
    let cap = self.capabilities.clone();
    self.status = Some(StatusMessage {
      kind: StatusKind::Info,
      text: tr(self.lang, "control_center.connecting").into(),
    });
    self.action = Some(self.jobs.spawn(move |_| {
      Ok(
        NetworkBackend::new(SystemProcessRunner, cap)
          .connect_wifi(&network.ssid, password.as_deref())
          .map(|_| "Conectado".into())
          .map_err(|e| e.to_string()),
      )
    }));
  }
  /// Executes the `visible_wifi` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn visible_wifi(&self) -> Vec<usize> {
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
  /// Executes the `visible_interfaces` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn visible_interfaces(&self) -> Vec<usize> {
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
  /// Executes the `visible_vpn` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn visible_vpn(&self) -> Vec<usize> {
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
  /// The menu row under the cursor, if any.
  fn selected_item(&self) -> Option<Item> {
    let rows = self.rows();
    let mut menu = self.menu;
    menu.normalize(&rows);
    menu.selected_id(&rows)
  }
  /// Whether the page is a list that accepts `/` to filter.
  fn is_list_page(&self) -> bool {
    matches!(
      self.page,
      NetworkPage::Interfaces | NetworkPage::Ethernet | NetworkPage::Wifi | NetworkPage::Vpn
    )
  }
  /// Whether typed characters currently go to text (Wi-Fi password, manual
  /// DNS or the list filter), so `q`/`?` must not act as the global quit/help
  /// keys. Mirrors the precedence of [`Self::handle`]: the forget confirmation
  /// is not typing.
  pub fn captures_text(&self) -> bool {
    !self.confirm_forget
      && (self.password.is_some() || self.dns_input.is_some() || self.search.is_some())
  }

  pub fn handle(&mut self, key: KeyCode) -> bool {
    if self.confirm_forget {
      match self.confirmation.handle(key) {
        ConfirmOutcome::Confirmed => {
          self.confirm_forget = false;
          self.confirmation = ConfirmState::new();
          self.perform_forget();
        }
        ConfirmOutcome::Cancelled => {
          self.confirm_forget = false;
          self.confirmation = ConfirmState::new();
        }
        ConfirmOutcome::Pending => {}
      }
      return false;
    }
    if let Some(password) = &mut self.password {
      match key {
        KeyCode::Esc => {
          self.password = None;
          self.pending_wifi = None;
        }
        KeyCode::Backspace => {
          password.pop();
        }
        KeyCode::Char(c) => password.push(c),
        KeyCode::Enter => {
          let secret = std::mem::take(password);
          let index = self.pending_wifi.take();
          self.password = None;
          if let Some(index) = index {
            self.run_wifi(index, Some(secret));
          }
        }
        _ => {}
      }
      return false;
    }
    if let Some(input) = &mut self.dns_input {
      match key {
        KeyCode::Esc => self.dns_input = None,
        KeyCode::Backspace => {
          input.pop();
        }
        KeyCode::Char(c) if !c.is_control() => input.push(c),
        KeyCode::Enter => {
          let value = std::mem::take(input);
          self.dns_input = None;
          self.apply_dns(&value);
        }
        _ => {}
      }
      return false;
    }
    if let Some(search) = &mut self.search {
      match key {
        KeyCode::Esc => self.search = None,
        KeyCode::Backspace => {
          search.pop();
        }
        KeyCode::Char(c) => search.push(c),
        KeyCode::Enter => {}
        _ => {}
      }
      self.menu = MenuState::default();
      return false;
    }
    let rows = self.rows();
    let page_size = usize::from(self.list_height.max(1));
    match key {
      KeyCode::Char('r') => self.start_refresh(self.page == NetworkPage::Wifi),
      KeyCode::Char('/') if self.is_list_page() => self.search = Some(String::new()),
      KeyCode::Char('i')
        if matches!(
          self.selected_item(),
          Some(Item::WifiNetwork(_) | Item::VpnConnection(_))
        ) =>
      {
        self.open_details()
      }
      _ => match self.menu.handle(key, &rows, page_size) {
        MenuEvent::Back => return self.back(),
        MenuEvent::Activate(item) | MenuEvent::Toggle(item) | MenuEvent::Confirm(item) => {
          self.activate(item)
        }
        MenuEvent::Adjust(..) | MenuEvent::Moved | MenuEvent::None => {}
      },
    }
    false
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
  /// Leaves a page for its parent and puts the cursor back on the row that
  /// opened it. Returns `true` when the Network page itself should close.
  fn back(&mut self) -> bool {
    if self.page == NetworkPage::Home {
      return true;
    }
    let from = self.page;
    let parent = match from {
      NetworkPage::Detail(_) | NetworkPage::WifiDetail(_) | NetworkPage::VpnDetail(_) => {
        self.detail_parent
      }
      _ => NetworkPage::Home,
    };
    let opener = match from {
      NetworkPage::Detail(i) => Item::Interface(i),
      NetworkPage::WifiDetail(i) => Item::WifiNetwork(i),
      NetworkPage::VpnDetail(i) => Item::VpnConnection(i),
      other => Item::Open(other),
    };
    self.page = parent;
    self.menu = MenuState::default();
    let rows = self.rows();
    self.menu.select(&rows, &opener);
    false
  }
  /// Runs the action of a menu row.
  fn activate(&mut self, item: Item) {
    match item {
      Item::Open(page) => self.open_home_page(page),
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
  /// Footer hints for the current row. `i Details` appears only on a Wi-Fi
  /// network or a VPN row, where it opens the details page.
  fn footer_hints(&self, rows: &[Row<Item>]) -> String {
    if self.confirm_forget {
      return confirm_hints(self.lang);
    }
    let mut menu = self.menu;
    menu.normalize(rows);
    let details = [("i", tr(self.lang, "control_center.details"))];
    let on_network = matches!(
      menu.selected_id(rows),
      Some(Item::WifiNetwork(_) | Item::VpnConnection(_))
    );
    let extra: &[(&str, &str)] = if on_network { &details } else { &[] };
    hints(
      self.lang,
      &HintContext {
        row: menu.selected_kind(rows),
        can_go_back: true,
        search: self.is_list_page(),
        refresh: true,
        extra,
        ..HintContext::default()
      },
    )
  }
  /// Executes the `forget_selected` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn forget_selected(&mut self) {
    if let Some(i) = self.current_index()
      && self.snapshot.wifi.get(i).is_some()
    {
      self.confirmation = ConfirmState::new();
      self.confirm_forget = true;
    }
  }
  /// Executes the `perform_forget` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
  /// Applies the `toggle_wifi` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn toggle_wifi(&mut self) {
    let enabled = self.snapshot.wifi_enabled != Some(true);
    self.action("Alterando Wi-Fi", move |b| b.set_wifi_enabled(enabled));
  }
  /// Applies the `apply_dns` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
  /// Applies the `apply_dns_privileged` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn apply_dns_privileged(&mut self, connection: String, values: Vec<String>, automatic: bool) {
    if self.action.is_some() || self.job.is_some() {
      return;
    }
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
  /// Renders the page: the single menu list, then the text fields, the
  /// confirmation and the status.
  pub fn draw(&mut self, f: &mut Frame) {
    let area = f.area();
    let rows = self.rows();
    self.menu.normalize(&rows);
    let body = shell(
      f,
      area,
      &self.theme,
      self.lang,
      &self.breadcrumb(),
      &self.footer_hints(&rows),
    );
    self.list_height = body.height;
    draw_menu(
      f,
      body,
      &self.theme,
      &rows,
      &mut self.menu,
      MenuStyle {
        icons: AppConfig::icons_enabled(),
      },
    );
    if let Some(input) = &self.dns_input {
      let popup = argvus_tui::chrome::centered(area, 48, 7);
      f.render_widget(Clear, popup);
      f.render_widget(
        Paragraph::new(vec![
          Line::from(tr(self.lang, "control_center.dns_servers_comma_separated")),
          Line::from(format!("{input}_")),
          Line::from(tr(self.lang, "control_center.enter_apply_esc_cancel")),
        ])
        .block(Block::bordered().title(tr(self.lang, "control_center.dns_servers"))),
        popup,
      );
    } else if let Some(password) = &self.password {
      let popup = argvus_tui::chrome::centered(area, 48, 7);
      f.render_widget(Clear, popup);
      let shown = "•".repeat(argvus_tui::text::display_width(password));
      f.render_widget(
        Paragraph::new(vec![
          Line::from(tr(self.lang, "control_center.wi_fi_network_password")),
          Line::from(format!("{shown}_")),
          Line::from(tr(self.lang, "control_center.enter_connect_esc_cancel")),
        ])
        .block(Block::bordered().title(tr(self.lang, "control_center.wi_fi_password"))),
        popup,
      );
    }
    if self.confirm_forget {
      let name = self
        .current_index()
        .and_then(|i| self.snapshot.wifi.get(i))
        .map(|w| terminal_text(&w.ssid))
        .unwrap_or_default();
      draw_confirm(
        f,
        area,
        &self.theme,
        ConfirmDialog {
          title: tr(self.lang, "control_center.forget_network"),
          message: &format!(
            "{} {}?",
            tr(self.lang, "control_center.forget_network_58478a"),
            name
          ),
          confirm: tr(self.lang, "control_center.forget"),
          cancel: tr(self.lang, "control_center.cancel"),
          danger: true,
          deadline: None,
        },
        &self.confirmation,
      );
    }
    if let Some(status_message) = &self.status {
      status(f, body, &self.theme, status_message);
    }
    if self.job.is_some() {
      status(
        f,
        body,
        &self.theme,
        &StatusMessage {
          kind: StatusKind::Info,
          text: tr(
            self.lang,
            if self.scan {
              "control_center.scanning_for_networks"
            } else {
              "control_center.refreshing_network"
            },
          )
          .into(),
        },
      );
    }
  }
  /// Rows of the current page.
  fn rows(&self) -> Vec<Row<Item>> {
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
    }
  }
  /// The trailing Actions group of the list pages: `Refresh` (`r`).
  fn refresh_section(&self) -> Vec<Row<Item>> {
    vec![
      Row::section(tr(self.lang, "control_center.section_actions")),
      Row::action(Item::Refresh, tr(self.lang, "control_center.refresh")).icon(icons::REFRESH),
    ]
  }
  /// The Network home: one submenu per available page, with its state.
  fn home_rows(&self) -> Vec<Row<Item>> {
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
          _ => return None,
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
  fn status_rows(&self) -> Vec<Row<Item>> {
    let active = self
      .snapshot
      .interfaces
      .iter()
      .find(|i| i.state == "connected");
    let wifi_row = match self.snapshot.wifi_enabled {
      Some(on) => Row::toggle(Item::WifiToggle, tr(self.lang, "control_center.wi_fi"), on),
      None => Row::toggle(
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
  fn interface_rows(&self) -> Vec<Row<Item>> {
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
  fn interface_detail_rows(&self, i: usize) -> Vec<Row<Item>> {
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
  fn wifi_rows(&self) -> Vec<Row<Item>> {
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
  fn wifi_detail_rows(&self, i: usize) -> Vec<Row<Item>> {
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
  fn vpn_rows(&self) -> Vec<Row<Item>> {
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
  fn vpn_detail_rows(&self, i: usize) -> Vec<Row<Item>> {
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
      Row::action(Item::Refresh, tr(self.lang, "control_center.refresh")).icon(icons::REFRESH),
      Row::section(tr(self.lang, "control_center.section_summary")),
      Row::info(tr(self.lang, "control_center.name"), terminal_text(&v.name)),
      Row::info(tr(self.lang, "control_center.type"), v.kind.clone()),
      Row::info(tr(self.lang, "control_center.status_bbe39c"), status),
    ]
  }
  /// The DNS value is editable (`Enter` opens the server list); automatic
  /// DNS and the refresh are actions.
  fn dns_rows(&self) -> Vec<Row<Item>> {
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
  fn proxy_rows(&self) -> Vec<Row<Item>> {
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

/// Joins values with commas, or `—` when there are none.
fn joined_or_dash(values: &[String]) -> String {
  if values.is_empty() {
    "—".into()
  } else {
    values.join(", ")
  }
}

/// Applies the `apply_dns_privileged` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
mod tests {
  use super::*;
  use crate::model::{InterfaceInfo, VpnConnection, WifiNetwork};
  use argvus_control_center_core::capabilities::Capabilities;
  use argvus_i18n::Lang;
  use argvus_tui::menu::RowKind;

  /// Executes the `app` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
  }

  #[test]
  /// Executes the `wifi_is_only_listed_when_a_card_exists` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
  /// Executes the `vpn_is_only_listed_when_vpn_is_detected` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
    // Status, Interfaces, Ethernet, Wi-Fi (a card exists), DNS, Proxy. Firewall
    // lives under Security on the Control Center Home.
    assert_eq!(rows.len(), 6);
    assert!(rows.iter().all(|row| row.kind() == RowKind::Submenu));
    assert_eq!(rows[0].id(), Some(&Item::Open(NetworkPage::Status)));
    assert_eq!(rows[4].id(), Some(&Item::Open(NetworkPage::Dns)));
    assert!(rows[0].detail_text().unwrap().contains("full"));
    assert!(rows[0].detail_text().unwrap().contains("eno1"));
  }

  #[test]
  /// Interface details offer Connect, Disconnect and Refresh; the properties
  /// are information and never take the cursor.
  fn interface_detail_has_connect_disconnect_and_unselectable_info() {
    let mut app = app();
    with_network(&mut app);
    app.page = NetworkPage::Detail(0);
    let rows = app.rows();
    assert_eq!(rows[1].id(), Some(&Item::Connect));
    assert_eq!(rows[2].id(), Some(&Item::Disconnect));
    assert_eq!(rows[3].id(), Some(&Item::Refresh));
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
    assert_eq!(rows[3].id(), Some(&Item::Refresh));
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
}
