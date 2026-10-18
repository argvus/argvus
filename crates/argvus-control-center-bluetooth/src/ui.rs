//! Implements terminal UI rendering and interaction in crate `argvus control center bluetooth`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use crate::{
  agent::AgentHandle,
  backend::BluetoothBackend,
  model::{
    AgentEvent, AgentPrompt, AgentReply, BluetoothPage, BluetoothSnapshot, Device, PromptStep,
  },
};
use argvus_control_center_core::navigation::{HomeCardId, join};
use argvus_control_center_core::{
  capabilities::Capabilities,
  jobs::{JobHandle, JobManager, JobState},
  process::SystemProcessRunner,
};
use argvus_i18n::{Lang, tr};
use argvus_theme::Theme;
use argvus_tui::{
  buttons::{Button, ButtonKind},
  chrome::centered,
  components::{
    ConfirmationDialog, ConfirmationOutcome, ConfirmationState, StatusKind, StatusMessage,
    draw_confirmation, draw_status,
  },
  page::{Selection, list, shell},
};
use crossterm::event::KeyCode;
use ratatui::{
  Frame,
  layout::{Alignment, Constraint, Layout, Rect},
  style::{Modifier, Style},
  text::{Line, Span},
  widgets::{Block, Clear, Paragraph, Wrap},
};

/// Represents `BluetoothApp`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct BluetoothApp {
  pub page: BluetoothPage,
  selected: Selection,
  on_buttons: bool,
  button_selected: usize,
  button_from: Option<usize>,
  snapshot: BluetoothSnapshot,
  job: Option<JobHandle<Result<BluetoothSnapshot, String>>>,
  action: Option<JobHandle<Result<String, String>>>,
  jobs: JobManager,
  lang: Lang,
  theme: Theme,
  capabilities: Capabilities,
  status: Option<StatusMessage>,
  confirm_remove: bool,
  confirmation: ConfirmationState,
  scanning: bool,
  agent: Option<AgentHandle>,
  agent_ready: bool,
  prompt: Option<AgentPrompt>,
  prompt_confirm: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Defines `ActionButton`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
enum ActionButton {
  Power,
  Discoverable,
  Pair,
  Connect,
  Disconnect,
  Trust,
  Remove,
}

impl BluetoothApp {
  /// Replaces the semantic theme used by this page.
  pub fn set_theme(&mut self, theme: &Theme) {
    self.theme = theme.clone();
  }

  /// Constructs `new` with this module's expected initial state. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn new(lang: Lang, theme: Theme, capabilities: Capabilities) -> Self {
    Self {
      page: BluetoothPage::Home,
      selected: Selection::default(),
      on_buttons: false,
      button_selected: 0,
      button_from: None,
      snapshot: Default::default(),
      job: None,
      action: None,
      jobs: JobManager::default(),
      lang,
      theme,
      capabilities,
      status: None,
      confirm_remove: false,
      confirmation: ConfirmationState::default(),
      scanning: false,
      agent: None,
      agent_ready: false,
      prompt: None,
      prompt_confirm: false,
    }
  }
  /// Executes the `reload` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn reload(&mut self) {
    if self.job.is_some() || self.action.is_some() {
      return;
    }
    let caps = self.capabilities.clone();
    self.job = Some(self.jobs.spawn(move |_| {
      Ok(
        BluetoothBackend::new(SystemProcessRunner, caps)
          .snapshot()
          .map_err(|e| e.to_string()),
      )
    }));
  }
  /// Executes the `poll` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn poll(&mut self) -> bool {
    let mut changed = false;
    let mut events = Vec::new();
    if let Some(agent) = &self.agent {
      self.agent_ready = agent.is_ready();
      while let Some(event) = agent.try_event() {
        events.push(event);
      }
    }
    for event in events {
      match event {
        AgentEvent::Cancelled | AgentEvent::Released => {
          if self.prompt.is_some() {
            if let Some(agent) = &self.agent {
              agent.reply(AgentReply::Reject);
            }
            self.prompt = None;
            self.prompt_confirm = false;
            changed = true;
          }
        }
        start => self.open_prompt(start),
      }
    }
    if let Some(j) = self.job.take() {
      match j.try_state() {
        JobState::Running => self.job = Some(j),
        JobState::Finished(r) => {
          match r {
            Ok(Ok(s)) => {
              self.snapshot = s;
              self.selected.normalize(self.row_count());
            }
            Ok(Err(e)) | Err(e) => {
              self.status = Some(StatusMessage {
                kind: StatusKind::Error,
                text: e,
              })
            }
          };
          self.scanning = false;
          changed = true;
        }
      }
    }
    if let Some(j) = self.action.take() {
      match j.try_state() {
        JobState::Running => self.action = Some(j),
        JobState::Finished(r) => {
          match r {
            Ok(Ok(msg)) => {
              self.status = Some(StatusMessage {
                kind: StatusKind::Success,
                text: msg,
              });
              self.reload();
            }
            Ok(Err(e)) | Err(e) => {
              self.status = Some(StatusMessage {
                kind: StatusKind::Error,
                text: e,
              })
            }
          };
          if self.prompt.is_some() {
            if let Some(agent) = &self.agent {
              agent.reply(AgentReply::Reject);
            }
            self.prompt = None;
            self.prompt_confirm = false;
          }
          changed = true;
        }
      }
    }
    changed
  }
  /// Executes the `row_count` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn row_count(&self) -> usize {
    match self.page {
      BluetoothPage::Home => 3,
      BluetoothPage::State => 2,
      BluetoothPage::Devices | BluetoothPage::Pair => self.snapshot.devices.len(),
    }
  }
  /// Executes the `selected_device` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn selected_device(&self) -> Option<&Device> {
    self.snapshot.devices.get(self.selected.index)
  }
  /// Executes the `start_action` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn start_action(
    &mut self,
    operation: impl FnOnce(
      BluetoothBackend<SystemProcessRunner>,
    ) -> Result<(), crate::backend::BluetoothError>
    + Send
    + 'static,
    message: String,
  ) {
    if self.action.is_some() {
      return;
    }
    self.status = Some(StatusMessage {
      kind: StatusKind::Info,
      text: tr(self.lang, "control_center.applying_bluetooth_change").into(),
    });
    let caps = self.capabilities.clone();
    self.action = Some(self.jobs.spawn(move |_| {
      Ok(
        operation(BluetoothBackend::new(SystemProcessRunner, caps))
          .map(|_| message)
          .map_err(|e| e.to_string()),
      )
    }));
  }
  /// Executes the `act_device` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn act_device(&mut self, action: &'static str) {
    if let Some(d) = self.selected_device() {
      let address = d.address.clone();
      let msg = tr(self.lang, "control_center.bluetooth_operation_completed").into();
      self.start_action(move |b| b.action(&address, action), msg);
    }
  }
  /// Executes the `act_pair` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn act_pair(&mut self) {
    if self.action.is_some() {
      return;
    }
    let Some(d) = self.selected_device() else {
      return;
    };
    let address = d.address.clone();
    let via_agent = self.agent_ready;
    let msg = tr(self.lang, "control_center.device_paired").into();
    self.start_action(move |b| b.pair(&address, via_agent), msg);
  }
  /// Executes the `open_agent` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn open_agent(&mut self) {
    if self.agent.is_some() {
      return;
    }
    self.agent_ready = false;
    self.agent = Some(AgentHandle::start());
  }
  /// Executes the `stop_agent` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn stop_agent(&mut self) {
    self.prompt = None;
    self.prompt_confirm = false;
    self.agent_ready = false;
    if let Some(mut agent) = self.agent.take() {
      agent.unregister();
    }
  }
  /// Executes the `open_prompt` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn open_prompt(&mut self, event: AgentEvent) {
    if self.prompt.is_some() {
      return;
    }
    if let Some(prompt) = AgentPrompt::from_event(event) {
      self.prompt = Some(prompt);
      self.prompt_confirm = false;
    }
  }
  /// Executes the `start_scan` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn start_scan(&mut self) {
    if self.action.is_some() || self.scanning {
      return;
    }
    self.scanning = true;
    self.status = Some(StatusMessage {
      kind: StatusKind::Info,
      text: tr(self.lang, "control_center.scanning_for_devices").into(),
    });
    let caps = self.capabilities.clone();
    self.action = Some(self.jobs.spawn(move |_| {
      let b = BluetoothBackend::new(SystemProcessRunner, caps);
      b.scan(true).map_err(|e| e.to_string())?;
      std::thread::sleep(std::time::Duration::from_secs(2));
      b.scan(false).map_err(|e| e.to_string())?;
      b.snapshot().map_err(|e| e.to_string())?;
      Ok(Ok("Scan concluído.".into()))
    }));
  }
  /// Processes `handle` in this module's event flow. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn handle(&mut self, key: KeyCode) -> bool {
    if let Some(mut prompt) = self.prompt.take() {
      let step = prompt.handle_key(key, &mut self.prompt_confirm);
      match step {
        PromptStep::Accepted(reply) => {
          if let Some(agent) = &self.agent {
            agent.reply(reply);
          }
          self.prompt_confirm = false;
        }
        PromptStep::Dismissed => {
          if let Some(agent) = &self.agent {
            agent.reply(AgentReply::Reject);
          }
          self.prompt_confirm = false;
        }
        PromptStep::Idle | PromptStep::Updated => {
          self.prompt = Some(prompt);
        }
      }
      return false;
    }
    if self.confirm_remove {
      match self.confirmation.handle(key) {
        ConfirmationOutcome::Confirmed => {
          self.confirm_remove = false;
          self.confirmation = ConfirmationState::default();
          self.act_device("remove");
        }
        ConfirmationOutcome::Cancelled => {
          self.confirm_remove = false;
          self.confirmation = ConfirmationState::default();
        }
        ConfirmationOutcome::Pending => {}
      }
      return false;
    }
    if self.on_buttons && !self.buttons().is_empty() {
      match key {
        KeyCode::Tab => {
          self.toggle_buttons(false);
          return false;
        }
        KeyCode::BackTab => {
          self.toggle_buttons(true);
          return false;
        }
        KeyCode::Left | KeyCode::Char('h') => {
          self.move_button(-1);
          return false;
        }
        KeyCode::Right | KeyCode::Char('l') => {
          self.move_button(1);
          return false;
        }
        KeyCode::Enter | KeyCode::Char(' ') => {
          self.activate_button();
          return false;
        }
        KeyCode::Up | KeyCode::Char('k') | KeyCode::Down | KeyCode::Char('j') => return false,
        _ => {}
      }
    }
    if matches!(key, KeyCode::Esc | KeyCode::Left) {
      if self.page == BluetoothPage::Home {
        return true;
      }
      self.page = BluetoothPage::Home;
      self.selected.index = 0;
      self.on_buttons = false;
      self.button_from = None;
      self.stop_agent();
      return false;
    }
    if key == KeyCode::Char('r') {
      if self.page == BluetoothPage::Pair {
        self.start_scan();
      } else {
        self.reload();
      }
      return false;
    }
    if self.action.is_some() {
      return false;
    }
    if self.page == BluetoothPage::Home && matches!(key, KeyCode::Enter | KeyCode::Right) {
      self.page = match self.selected.index {
        0 => BluetoothPage::State,
        1 => BluetoothPage::Devices,
        _ => BluetoothPage::Pair,
      };
      self.selected.index = 0;
      self.on_buttons = false;
      self.button_selected = 0;
      self.button_from = None;
      self.status = None;
      if self.page == BluetoothPage::Pair {
        self.open_agent();
      }
      self.reload();
      return false;
    }
    if self.selected.handle(key, self.row_count(), 8) {
      return false;
    }
    match (self.page, key) {
      (_, KeyCode::Tab | KeyCode::BackTab) => self.toggle_buttons(key == KeyCode::BackTab),
      (BluetoothPage::Pair, KeyCode::Enter) => self.act_pair(),
      (BluetoothPage::Devices, KeyCode::Enter) => {
        if self.selected_device().map(|d| d.connected).unwrap_or(false) {
          self.act_device("disconnect");
        } else if self.selected_device().map(|d| d.paired).unwrap_or(false) {
          self.act_device("connect");
        } else {
          self.act_device("pair");
        }
      }
      _ => {}
    }
    false
  }
  /// Applies the `toggle_buttons` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn toggle_buttons(&mut self, backwards: bool) {
    let count = self.buttons().len();
    if count == 0 {
      return;
    }
    if self.on_buttons {
      self.on_buttons = false;
      if let Some(index) = self.button_from.take() {
        self.selected.index = index;
      }
    } else {
      self.button_from = Some(self.selected.index);
      self.button_selected = if backwards { count - 1 } else { 0 };
      self.on_buttons = true;
    }
  }
  /// Executes the `move_button` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn move_button(&mut self, delta: isize) {
    let count = self.buttons().len();
    if count == 0 {
      return;
    }
    self.button_selected =
      (self.button_selected as isize + delta).rem_euclid(count as isize) as usize;
  }
  /// Executes the `activate_button` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn activate_button(&mut self) {
    let actions = self.buttons();
    let Some((action, _)) = actions.get(self.button_selected) else {
      return;
    };
    match *action {
      ActionButton::Power => {
        let on = !self
          .snapshot
          .adapter
          .as_ref()
          .map(|a| a.powered)
          .unwrap_or(false);
        self.start_action(
          move |b| b.power(on),
          tr(self.lang, "control_center.adapter_state_changed").into(),
        );
      }
      ActionButton::Discoverable => {
        let on = !self
          .snapshot
          .adapter
          .as_ref()
          .map(|a| a.discoverable)
          .unwrap_or(false);
        self.start_action(
          move |b| b.discoverable(on),
          tr(self.lang, "control_center.discoverability_changed").into(),
        );
      }
      ActionButton::Pair => self.act_pair(),
      ActionButton::Connect => self.act_device("connect"),
      ActionButton::Disconnect => self.act_device("disconnect"),
      ActionButton::Trust => {
        let action = if self.selected_device().map(|d| d.trusted).unwrap_or(false) {
          "untrust"
        } else {
          "trust"
        };
        self.act_device(action);
      }
      ActionButton::Remove => self.confirm_remove = true,
    }
  }
  /// Executes the `buttons` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn buttons(&self) -> Vec<(ActionButton, Button)> {
    match self.page {
      BluetoothPage::State => vec![
        (
          ActionButton::Power,
          Button::new(
            tr(self.lang, "control_center.power_f27008"),
            ButtonKind::Secondary,
          ),
        ),
        (
          ActionButton::Discoverable,
          Button::new(
            tr(self.lang, "control_center.discoverable"),
            ButtonKind::Secondary,
          ),
        ),
      ],
      BluetoothPage::Devices => vec![
        (
          ActionButton::Connect,
          Button::new(tr(self.lang, "control_center.connect"), ButtonKind::Primary),
        ),
        (
          ActionButton::Disconnect,
          Button::new(
            tr(self.lang, "control_center.disconnect"),
            ButtonKind::Secondary,
          ),
        ),
        (
          ActionButton::Trust,
          Button::new(tr(self.lang, "control_center.trust"), ButtonKind::Secondary),
        ),
        (
          ActionButton::Remove,
          Button::new(tr(self.lang, "control_center.remove"), ButtonKind::Danger),
        ),
      ],
      BluetoothPage::Pair => vec![
        (
          ActionButton::Pair,
          Button::new(tr(self.lang, "control_center.pair"), ButtonKind::Primary),
        ),
        (
          ActionButton::Connect,
          Button::new(
            tr(self.lang, "control_center.connect"),
            ButtonKind::Secondary,
          ),
        ),
        (
          ActionButton::Disconnect,
          Button::new(
            tr(self.lang, "control_center.disconnect"),
            ButtonKind::Secondary,
          ),
        ),
        (
          ActionButton::Trust,
          Button::new(tr(self.lang, "control_center.trust"), ButtonKind::Secondary),
        ),
        (
          ActionButton::Remove,
          Button::new(tr(self.lang, "control_center.remove"), ButtonKind::Danger),
        ),
      ],
      BluetoothPage::Home => Vec::new(),
    }
  }
  /// Executes the `footer_hints` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn footer_hints(&self) -> &'static str {
    match self.page {
      BluetoothPage::Home => tr(
        self.lang,
        "control_center.navigate_enter_open_esc_back_r_refresh_help",
      ),
      BluetoothPage::State | BluetoothPage::Devices | BluetoothPage::Pair => tr(
        self.lang,
        "control_center.navigate_tab_actions_move_enter_activate_r_refresh_esc_back_help",
      ),
    }
  }
  /// Renders `draw` while respecting the current domain state and semantic theme. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn draw(&self, f: &mut Frame) {
    // Bluetooth is listed under the Hardware card, so its path starts there.
    let page = match self.page {
      BluetoothPage::Home => tr(self.lang, "control_center.bluetooth"),
      BluetoothPage::State => tr(self.lang, "control_center.bluetooth_state"),
      BluetoothPage::Devices => tr(self.lang, "control_center.bluetooth_devices"),
      BluetoothPage::Pair => tr(self.lang, "control_center.bluetooth_pair"),
    };
    let breadcrumb = join(&[HomeCardId::Hardware.label(self.lang), page]);
    let area = shell(
      f,
      f.area(),
      &self.theme,
      self.lang,
      &breadcrumb,
      self.footer_hints(),
    );
    let buttons = self.buttons();
    let raw_buttons: Vec<Button> = buttons.iter().map(|(_, button)| button.clone()).collect();
    let (body, button_area) = if raw_buttons.is_empty() {
      (area, None)
    } else {
      let button_height = argvus_tui::buttons::height(&raw_buttons, area.width).min(area.height);
      let split =
        Layout::vertical([Constraint::Min(1), Constraint::Length(button_height)]).split(area);
      (split[0], Some(split[1]))
    };
    let rows: Vec<String> = match self.page {
      BluetoothPage::Home => vec![
        tr(self.lang, "control_center.state").into(),
        tr(self.lang, "control_center.devices").into(),
        tr(self.lang, "control_center.pair").into(),
      ],
      BluetoothPage::State => {
        let a = self.snapshot.adapter.as_ref();
        vec![
          format!(
            "{}  {}",
            tr(self.lang, "control_center.adapter"),
            a.map(|x| x.name.as_str()).unwrap_or("—")
          ),
          format!(
            "{}  {}   {}",
            tr(self.lang, "control_center.powered"),
            a.map(|x| {
              if x.powered {
                tr(self.lang, "control_center.yes")
              } else {
                tr(self.lang, "control_center.no")
              }
            })
            .unwrap_or("—"),
            if self.scanning { "[SCAN]" } else { "" }
          ),
        ]
      }
      BluetoothPage::Devices | BluetoothPage::Pair => self
        .snapshot
        .devices
        .iter()
        .map(|d| {
          let state = if d.connected {
            tr(self.lang, "control_center.connected")
          } else if d.paired {
            tr(self.lang, "control_center.paired")
          } else {
            tr(self.lang, "control_center.known")
          };
          let trusted = if d.trusted {
            tr(self.lang, "control_center.trusted")
          } else {
            ""
          };
          format!(
            "{}  {}  {}{}{}",
            if d.name.is_empty() {
              tr(self.lang, "control_center.unnamed")
            } else {
              &d.name
            },
            d.address,
            state,
            trusted,
            d.battery.map(|v| format!(" {}%", v)).unwrap_or_default()
          )
        })
        .collect(),
    };
    list(
      f,
      body,
      &self.theme,
      &rows,
      self.selected.index.min(rows.len().saturating_sub(1)),
    );
    if let Some(button_area) = button_area {
      let focus = if self.on_buttons {
        self.button_selected
      } else {
        usize::MAX
      };
      argvus_tui::buttons::draw(f, button_area, &raw_buttons, focus, &self.theme);
    }
    if let Some(s) = &self.status {
      draw_status(
        f,
        Rect::new(
          body.x,
          body.y + body.height.saturating_sub(1),
          body.width,
          1,
        ),
        &self.theme,
        s,
      );
    }
    if self.confirm_remove {
      draw_confirmation(
        f,
        f.area(),
        &self.theme,
        ConfirmationDialog {
          title: tr(self.lang, "control_center.remove_device"),
          message: tr(
            self.lang,
            "control_center.the_device_will_be_removed_from_known_devices",
          ),
          confirm_label: tr(self.lang, "control_center.remove"),
          cancel_label: tr(self.lang, "control_center.cancel"),
          confirm_selected: self.confirmation.confirm_selected,
        },
      );
    }
    if let Some(prompt) = &self.prompt {
      self.draw_agent_prompt(f, prompt);
    }
  }
  /// Renders `draw_agent_prompt` while respecting the current domain state and semantic theme. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn draw_agent_prompt(&self, f: &mut Frame, prompt: &AgentPrompt) {
    let device_name = prompt.device();
    let selected = |label: &str, active: bool| {
      if active {
        Span::styled(
          format!("[ {label} ]"),
          Style::new()
            .fg(self.theme.selected_foreground)
            .bg(self.theme.selected_background)
            .add_modifier(Modifier::BOLD),
        )
      } else {
        Span::styled(format!("[ {label} ]"), Style::new().fg(self.theme.accent))
      }
    };
    let input_line = |label: &str, value: &str| {
      Line::from(vec![
        Span::styled(format!("{label} "), Style::new().fg(self.theme.muted)),
        Span::styled(format!("{value}_"), Style::new().fg(self.theme.foreground)),
      ])
    };
    let mut lines = vec![Line::from("")];
    let (title, height) = match prompt {
      AgentPrompt::Pin { input, .. } => {
        lines.push(Line::from(tr(
          self.lang,
          "control_center.enter_the_pin_shown_by_the_device",
        )));
        lines.push(Line::from(""));
        lines.push(input_line("PIN:", input));
        lines.push(Line::from(""));
        (tr(self.lang, "control_center.device_pin").to_string(), 9)
      }
      AgentPrompt::Passkey { input, .. } => {
        lines.push(Line::from(tr(
          self.lang,
          "control_center.enter_the_verification_code_from_the_device",
        )));
        lines.push(Line::from(""));
        lines.push(input_line("Código:", input));
        lines.push(Line::from(""));
        (
          tr(self.lang, "control_center.verification_code").to_string(),
          9,
        )
      }
      AgentPrompt::DisplayPasskey { passkey, .. } => {
        lines.push(Line::from(self.lang.tr_args(
          "control_center.showing_the_number_check_it_on",
          [("name", device_name)],
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
          format!("{passkey:06}"),
          Style::new()
            .fg(self.theme.accent)
            .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
          selected(
            tr(self.lang, "control_center.continue"),
            self.prompt_confirm,
          ),
          Span::raw("  "),
          selected(tr(self.lang, "control_center.cancel"), !self.prompt_confirm),
        ]));
        (
          tr(self.lang, "control_center.verification_number").to_string(),
          9,
        )
      }
      AgentPrompt::Confirm { passkey, .. } => {
        lines.push(Line::from(self.lang.tr_args(
          "control_center.device_shows_the_number",
          [("name", device_name)],
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
          format!("{passkey:06}"),
          Style::new()
            .fg(self.theme.accent)
            .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
          selected(tr(self.lang, "control_center.yes"), self.prompt_confirm),
          Span::raw("  "),
          selected(tr(self.lang, "control_center.no"), !self.prompt_confirm),
        ]));
        (
          tr(self.lang, "control_center.confirm_pairing").to_string(),
          9,
        )
      }
      AgentPrompt::Authorize { .. } => {
        lines.push(Line::from(self.lang.tr_args(
          "control_center.allow_device_to_be_paired",
          [("name", device_name)],
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
          selected(tr(self.lang, "control_center.allow"), self.prompt_confirm),
          Span::raw("  "),
          selected(tr(self.lang, "control_center.cancel"), !self.prompt_confirm),
        ]));
        (
          tr(self.lang, "control_center.authorize_pairing").to_string(),
          8,
        )
      }
      AgentPrompt::Service { uuid, .. } => {
        lines.push(Line::from(self.lang.tr_args(
          "control_center.device_requests_access_to_service",
          [("name", device_name)],
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
          uuid,
          Style::new().fg(self.theme.foreground),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
          selected(
            tr(self.lang, "control_center.allow_d0cc72"),
            self.prompt_confirm,
          ),
          Span::raw("  "),
          selected(tr(self.lang, "control_center.deny"), !self.prompt_confirm),
        ]));
        (
          tr(self.lang, "control_center.service_access").to_string(),
          9,
        )
      }
    };
    lines.push(Line::from(tr(
      self.lang,
      "control_center.enter_ok_esc_cancel",
    )));
    let popup = centered(
      f.area(),
      f.area().width.saturating_sub(8).clamp(36, 72),
      height,
    );
    f.render_widget(Clear, popup);
    f.render_widget(
      Paragraph::new(lines)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true })
        .block(
          Block::bordered()
            .title(format!(" {title} "))
            .border_style(Style::new().fg(self.theme.border_active))
            .style(Style::new().bg(self.theme.background)),
        ),
      popup,
    );
  }
}
