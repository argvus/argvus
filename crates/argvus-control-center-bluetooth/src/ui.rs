//! Terminal UI rendering and interaction of the Bluetooth page.
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

mod input;
mod jobs;
mod render;

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
  fn row_count(&self) -> usize {
    match self.page {
      BluetoothPage::Home => 3,
      BluetoothPage::State => 2,
      BluetoothPage::Devices | BluetoothPage::Pair => self.snapshot.devices.len(),
    }
  }
  fn act_device(&mut self, action: &'static str) {
    if let Some(d) = self.selected_device() {
      let address = d.address.clone();
      let msg = tr(self.lang, "control_center.bluetooth_operation_completed").into();
      self.start_action(move |b| b.action(&address, action), msg);
    }
  }
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
  fn open_agent(&mut self) {
    if self.agent.is_some() {
      return;
    }
    self.agent_ready = false;
    self.agent = Some(AgentHandle::start());
  }
  fn stop_agent(&mut self) {
    self.prompt = None;
    self.prompt_confirm = false;
    self.agent_ready = false;
    if let Some(mut agent) = self.agent.take() {
      agent.unregister();
    }
  }
  fn open_prompt(&mut self, event: AgentEvent) {
    if self.prompt.is_some() {
      return;
    }
    if let Some(prompt) = AgentPrompt::from_event(event) {
      self.prompt = Some(prompt);
      self.prompt_confirm = false;
    }
  }
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
}

impl BluetoothApp {
  pub fn onboarding_ready(&self) -> bool {
    self.action.is_none() && !self.confirm_remove
  }
}
