//! Implements terminal UI rendering and interaction in crate `argvus control center audio`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use crate::{
  backend::{AudioBackend, AudioError, clamp_volume},
  model::{AudioDevice, AudioPage, AudioSnapshot},
};
use argvus_control_center_core::navigation::HomeCardId;
use argvus_control_center_core::{
  capabilities::Capabilities,
  config::AppConfig,
  jobs::{JobHandle, JobManager, JobState},
  process::SystemProcessRunner,
};
use argvus_i18n::{Lang, tr};
use argvus_theme::Theme;
use argvus_tui::{
  components::{
    ConfirmationDialog, ConfirmationOutcome, ConfirmationState, StatusKind, StatusMessage,
    draw_confirmation,
  },
  page::{Selection, list, readonly, shell, status},
};
use crossterm::event::KeyCode;
use ratatui::{
  Frame,
  text::Line,
  widgets::{Block, Clear, Paragraph},
};
use std::collections::BTreeMap;

/// Changes made on device pages that stay in the draft until `s` saves them.
#[derive(Debug, Clone, Default)]
struct AudioDraft {
  default_output: Option<u32>,
  default_input: Option<u32>,
  volumes: BTreeMap<u32, u8>,
  mutes: BTreeMap<u32, bool>,
}

/// One change that `s` writes to the audio server.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Change {
  Default(u32),
  Volume(u32, u8),
  Mute(u32, bool),
}

impl Change {
  /// Writes this change through the backend, the only place that touches the audio server.
  fn apply(self, backend: &AudioBackend<SystemProcessRunner>) -> Result<(), AudioError> {
    match self {
      Change::Default(id) => backend.set_default(id),
      Change::Volume(id, volume) => backend.set_volume(id, volume),
      Change::Mute(id, mute) => backend.set_mute(id, mute),
    }
  }
}

/// Represents `AudioApp`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct AudioApp {
  pub page: AudioPage,
  selected: Selection,
  snapshot: AudioSnapshot,
  job: Option<JobHandle<Result<AudioSnapshot, String>>>,
  action: Option<JobHandle<Result<String, String>>>,
  draft: AudioDraft,
  saving: bool,
  confirm_discard: bool,
  confirmation: ConfirmationState,
  jobs: JobManager,
  lang: Lang,
  theme: Theme,
  capabilities: Capabilities,
  pub status: Option<StatusMessage>,
  volume_input: Option<String>,
}

impl AudioApp {
  /// Replaces the semantic theme used by this page.
  pub fn set_theme(&mut self, theme: &Theme) {
    self.theme = theme.clone();
  }

  /// Constructs `new` with this module's expected initial state. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn new(lang: Lang, theme: Theme, capabilities: Capabilities) -> Self {
    Self {
      page: AudioPage::Home,
      selected: Selection::default(),
      snapshot: Default::default(),
      job: None,
      action: None,
      draft: AudioDraft::default(),
      saving: false,
      confirm_discard: false,
      confirmation: ConfirmationState::default(),
      jobs: JobManager::default(),
      lang,
      theme,
      capabilities,
      status: None,
      volume_input: None,
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
        AudioBackend::new(SystemProcessRunner, caps)
          .snapshot()
          .map_err(|e| e.to_string()),
      )
    }));
    self.status = Some(StatusMessage {
      kind: StatusKind::Info,
      text: tr(self.lang, "control_center.loading_audio").into(),
    });
  }
  /// Executes the `poll` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn poll(&mut self) -> bool {
    let mut changed = false;
    if let Some(j) = &self.job
      && let JobState::Finished(r) = j.try_state()
    {
      self.job = None;
      match r {
        Ok(Ok(s)) => {
          self.snapshot = s;
          self.normalize();
          self.success(tr(self.lang, "control_center.audio_refreshed"));
        }
        Ok(Err(e)) | Err(e) => self.error(e),
      };
      changed = true;
    }
    if let Some(j) = &self.action
      && let JobState::Finished(r) = j.try_state()
    {
      self.action = None;
      match r {
        Ok(Ok(msg)) => {
          if self.saving {
            self.draft = AudioDraft::default();
          }
          self.success(msg);
          self.reload();
        }
        Ok(Err(e)) | Err(e) => {
          self.error(e);
          self.reload();
        }
      };
      self.saving = false;
      changed = true;
    }
    changed
  }
  /// Executes the `success` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn success(&mut self, text: impl Into<String>) {
    self.status = Some(StatusMessage {
      kind: StatusKind::Success,
      text: text.into(),
    });
  }
  /// Executes the `error` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn error(&mut self, text: impl Into<String>) {
    self.status = Some(StatusMessage {
      kind: StatusKind::Error,
      text: text.into(),
    });
  }
  /// Executes the `normalize` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn normalize(&mut self) {
    self.selected.normalize(self.row_count());
  }
  /// Executes the `row_count` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn row_count(&self) -> usize {
    match self.page {
      AudioPage::Home => 4,
      AudioPage::Summary => 1,
      AudioPage::Output => self.snapshot.outputs.len(),
      AudioPage::Input => self.snapshot.inputs.len(),
      AudioPage::Devices => self.snapshot.outputs.len() + self.snapshot.inputs.len(),
    }
  }
  /// Executes the `devices` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn devices(&self) -> Vec<&AudioDevice> {
    match self.page {
      AudioPage::Output => self.snapshot.outputs.iter().collect(),
      AudioPage::Input => self.snapshot.inputs.iter().collect(),
      AudioPage::Devices => self
        .snapshot
        .outputs
        .iter()
        .chain(self.snapshot.inputs.iter())
        .collect(),
      AudioPage::Home | AudioPage::Summary => Vec::new(),
    }
  }
  /// Executes the `selected_device` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn selected_device(&self) -> Option<&AudioDevice> {
    self
      .devices()
      .get(self.selected.index)
      .copied()
      .filter(|device| device.id > 0)
  }
  /// Looks up a device from the last snapshot by its id, on either side.
  fn snapshot_device(&self, id: u32) -> Option<&AudioDevice> {
    self
      .snapshot
      .outputs
      .iter()
      .chain(self.snapshot.inputs.iter())
      .find(|device| device.id == id)
  }
  /// Volume shown and edited for a device: the draft value when one exists, otherwise the system value.
  fn volume_of(&self, device: &AudioDevice) -> Option<u8> {
    self
      .draft
      .volumes
      .get(&device.id)
      .copied()
      .or(device.volume)
  }
  /// Mute state shown for a device: the draft value when one exists, otherwise the system value.
  fn muted_of(&self, device: &AudioDevice) -> bool {
    self
      .draft
      .mutes
      .get(&device.id)
      .copied()
      .unwrap_or(device.muted)
  }
  /// Whether a device is the default for its direction, counting the draft choice.
  fn is_default_of(&self, device: &AudioDevice) -> bool {
    let (draft, current) = if device.direction == "output" {
      (self.draft.default_output, self.snapshot.default_output)
    } else {
      (self.draft.default_input, self.snapshot.default_input)
    };
    draft.or(current) == Some(device.id)
  }
  /// Changes in the draft that differ from the system state and therefore need `s`.
  fn pending_changes(&self) -> Vec<Change> {
    let mut changes = Vec::new();
    let defaults = [
      (self.draft.default_output, self.snapshot.default_output),
      (self.draft.default_input, self.snapshot.default_input),
    ];
    for (draft, current) in defaults {
      if let Some(id) = draft.filter(|id| Some(*id) != current) {
        changes.push(Change::Default(id));
      }
    }
    for (&id, &volume) in &self.draft.volumes {
      if self
        .snapshot_device(id)
        .is_some_and(|device| device.volume != Some(volume))
      {
        changes.push(Change::Volume(id, volume));
      }
    }
    for (&id, &mute) in &self.draft.mutes {
      if self
        .snapshot_device(id)
        .is_some_and(|device| device.muted != mute)
      {
        changes.push(Change::Mute(id, mute));
      }
    }
    changes
  }
  /// Whether the draft holds changes that `s` would apply.
  fn has_changes(&self) -> bool {
    !self.pending_changes().is_empty()
  }
  /// Executes the `start_action` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn start_action(
    &mut self,
    action: impl FnOnce(AudioBackend<SystemProcessRunner>) -> Result<(), AudioError> + Send + 'static,
    success: String,
  ) {
    if self.action.is_some() {
      return;
    }
    self.status = Some(StatusMessage {
      kind: StatusKind::Info,
      text: tr(self.lang, "control_center.applying_audio_change").into(),
    });
    let caps = self.capabilities.clone();
    self.action = Some(self.jobs.spawn(move |_| {
      Ok(
        action(AudioBackend::new(SystemProcessRunner, caps))
          .map(|_| success)
          .map_err(|e| e.to_string()),
      )
    }));
  }
  /// Applies every pending draft change to the audio server. Nothing else writes to the system.
  fn save(&mut self) {
    if self.action.is_some() {
      return;
    }
    let changes = self.pending_changes();
    if changes.is_empty() {
      self.status = Some(StatusMessage {
        kind: StatusKind::Info,
        text: tr(self.lang, "control_center.nothing_to_save").into(),
      });
      return;
    }
    self.saving = true;
    self.start_action(
      move |backend| {
        changes
          .into_iter()
          .try_for_each(|change| change.apply(&backend))
      },
      tr(self.lang, "control_center.saved").into(),
    );
  }
  /// Stages the selected device as the default for its direction; `s` applies it.
  fn set_draft_default(&mut self) {
    if let Some(d) = self.selected_device() {
      let (id, output) = (d.id, d.direction == "output");
      if output {
        self.draft.default_output = Some(id);
      } else {
        self.draft.default_input = Some(id);
      }
    } else {
      self.warn_disappeared();
    }
  }
  /// Stages a volume step of the selected device; `s` applies it.
  fn adjust_volume(&mut self, delta: i16) {
    if let Some(d) = self.selected_device() {
      let id = d.id;
      let current = self.volume_of(d).unwrap_or(0) as i16;
      self.draft.volumes.insert(id, clamp_volume(current + delta));
    } else {
      self.warn_disappeared();
    }
  }
  /// Stages the mute toggle of the selected device; `s` applies it.
  fn toggle_mute(&mut self) {
    if let Some(d) = self.selected_device() {
      let id = d.id;
      let mute = !self.muted_of(d);
      self.draft.mutes.insert(id, mute);
    } else {
      self.warn_disappeared();
    }
  }
  /// Executes the `warn_disappeared` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn warn_disappeared(&mut self) {
    self.error(tr(
      self.lang,
      "control_center.device_is_no_longer_available_press_r_to_refresh",
    ));
  }
  /// Leaves the device page and drops any draft that was not saved.
  fn leave_page(&mut self) {
    self.page = AudioPage::Home;
    self.selected.index = 0;
    self.draft = AudioDraft::default();
  }
  /// Whether typed characters currently go to the volume value field, so
  /// `q`/`?` must not act as the global quit/help keys. The confirmation
  /// sits above the field in [`Self::handle`] and does not take text.
  pub fn captures_text(&self) -> bool {
    !self.confirm_discard && self.volume_input.is_some()
  }

  /// Processes `handle` in this module's event flow. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn handle(&mut self, key: KeyCode) -> bool {
    if self.confirm_discard {
      match self.confirmation.handle(key) {
        ConfirmationOutcome::Confirmed => {
          self.confirm_discard = false;
          self.confirmation = ConfirmationState::default();
          self.leave_page();
        }
        ConfirmationOutcome::Cancelled => {
          self.confirm_discard = false;
          self.confirmation = ConfirmationState::default();
        }
        ConfirmationOutcome::Pending => {}
      }
      return false;
    }
    if self.volume_input.is_some() {
      return self.handle_volume_input(key);
    }
    if self.has_device_actions() {
      match key {
        KeyCode::Left | KeyCode::Char('-') => {
          self.adjust_volume(-5);
          return false;
        }
        KeyCode::Right | KeyCode::Char('+') | KeyCode::Char('=') => {
          self.adjust_volume(5);
          return false;
        }
        KeyCode::Char(' ') => {
          self.toggle_mute();
          return false;
        }
        KeyCode::Char('v') => {
          self.volume_input = Some(String::new());
          return false;
        }
        KeyCode::Char('s') => {
          self.save();
          return false;
        }
        _ => {}
      }
    }
    if matches!(key, KeyCode::Esc | KeyCode::Left) {
      if self.page == AudioPage::Home {
        return true;
      }
      if self.has_changes() {
        self.confirm_discard = true;
        self.confirmation = ConfirmationState::default();
        return false;
      }
      self.leave_page();
      return false;
    }
    match key {
      KeyCode::Char('r') => self.reload(),
      KeyCode::Up
      | KeyCode::Char('k')
      | KeyCode::Down
      | KeyCode::Char('j')
      | KeyCode::Home
      | KeyCode::End
      | KeyCode::PageUp
      | KeyCode::PageDown => {
        self.selected.handle(key, self.row_count(), 8);
      }
      KeyCode::Enter | KeyCode::Right => self.open_selected(),
      _ => {}
    }
    false
  }
  /// Executes the `open_selected` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn open_selected(&mut self) {
    if self.page == AudioPage::Home {
      self.page = match self.selected.index {
        0 => AudioPage::Summary,
        1 => AudioPage::Output,
        2 => AudioPage::Input,
        _ => AudioPage::Devices,
      };
      self.selected.index = 0;
      self.reload();
    } else if matches!(self.page, AudioPage::Output | AudioPage::Input) {
      self.set_draft_default();
    }
  }
  /// Whether the current page lists devices that accept default, volume and mute actions.
  fn has_device_actions(&self) -> bool {
    matches!(self.page, AudioPage::Output | AudioPage::Input) && !self.devices().is_empty()
  }
  /// Returns the footer hints for the current page: device pages list their keys, others stay read-only.
  fn footer_hints(&self) -> &'static str {
    let devices = tr(
      self.lang,
      "control_center.navigate_volume_enter_set_default_r_refresh_s_save_esc_back_help",
    );
    let readonly = tr(self.lang, "control_center.r_refresh_esc_back_help");
    let home = tr(
      self.lang,
      "control_center.navigate_enter_open_esc_back_r_refresh_help",
    );
    if self.page == AudioPage::Home {
      home
    } else if self.has_device_actions() {
      devices
    } else {
      readonly
    }
  }
  /// Processes `handle_volume_input` in this module's event flow. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn handle_volume_input(&mut self, key: KeyCode) -> bool {
    let input = self.volume_input.as_mut().unwrap();
    match key {
      KeyCode::Esc => self.volume_input = None,
      KeyCode::Enter => {
        let value = input
          .parse::<i16>()
          .ok()
          .filter(|value| (0..=100).contains(value))
          .map(clamp_volume);
        self.volume_input = None;
        let id = self.selected_device().map(|d| d.id);
        match (value, id) {
          (Some(v), Some(id)) => {
            self.draft.volumes.insert(id, v);
          }
          _ => self.error(tr(self.lang, "control_center.invalid_volume_0_100")),
        }
      }
      KeyCode::Backspace => {
        input.pop();
      }
      KeyCode::Char(c) if c.is_ascii_digit() && input.len() < 3 => input.push(c),
      _ => {}
    }
    false
  }
  /// Renders `draw` while respecting the current domain state and semantic theme. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn draw(&self, frame: &mut Frame) {
    let area = frame.area();
    let body = shell(
      frame,
      area,
      &self.theme,
      self.lang,
      &self.breadcrumb(),
      self.footer_hints(),
    );
    let rows = self.rows();
    if matches!(self.page, AudioPage::Summary | AudioPage::Devices) {
      readonly(
        frame,
        body,
        &self.theme,
        &rows.into_iter().map(Line::from).collect::<Vec<_>>(),
      );
    } else {
      list(
        frame,
        body,
        &self.theme,
        &rows,
        self.selected.index.min(rows.len().saturating_sub(1)),
      );
    }
    if let Some(value) = &self.volume_input {
      let popup = argvus_tui::chrome::centered(area, 48, 7);
      frame.render_widget(Clear, popup);
      frame.render_widget(
        Paragraph::new(vec![
          Line::from(tr(self.lang, "control_center.enter_volume_0_100")),
          Line::from(format!("{value}_")),
          Line::from(tr(self.lang, "control_center.enter_apply_esc_cancel")),
        ])
        .block(Block::bordered().title(tr(self.lang, "control_center.volume_3e7bd7"))),
        popup,
      );
    }
    if self.confirm_discard {
      draw_confirmation(
        frame,
        area,
        &self.theme,
        ConfirmationDialog {
          title: tr(self.lang, "control_center.discard_changes_title"),
          message: tr(self.lang, "control_center.discard_changes_description"),
          confirm_label: tr(self.lang, "control_center.discard"),
          cancel_label: tr(self.lang, "control_center.cancel"),
          confirm_selected: self.confirmation.confirm_selected,
        },
      );
    }
    if let Some(status_message) = &self.status {
      status(frame, body, &self.theme, status_message);
    }
  }
  /// Executes the `rows` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn rows(&self) -> Vec<String> {
    match self.page {
      AudioPage::Home => self.home_rows(),
      AudioPage::Summary => self.summary_rows(),
      AudioPage::Output | AudioPage::Input => self.device_rows(),
      AudioPage::Devices => self.all_devices_rows(),
    }
  }
  /// Executes the `home_rows` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn home_rows(&self) -> Vec<String> {
    let summary_label = tr(self.lang, "control_center.summary");
    let output_label = tr(self.lang, "control_center.output");
    let input_label = tr(self.lang, "control_center.input");
    let devices_label = tr(self.lang, "control_center.devices");

    let status_str = if self.snapshot.available {
      tr(self.lang, "control_center.active")
    } else {
      tr(self.lang, "control_center.unavailable")
    };

    let default_out_str = self
      .snapshot
      .default_output
      .and_then(|id| self.snapshot.outputs.iter().find(|d| d.id == id))
      .map(|d| d.description.as_str())
      .unwrap_or("—");

    let default_in_str = self
      .snapshot
      .default_input
      .and_then(|id| self.snapshot.inputs.iter().find(|d| d.id == id))
      .map(|d| d.description.as_str())
      .unwrap_or("—");

    let total_devs = self.snapshot.outputs.len() + self.snapshot.inputs.len();

    vec![
      format!(
        "{} {}  ·  {} · {}",
        AppConfig::icon(argvus_tui::icons::MONITOR),
        summary_label,
        if self.snapshot.backend.is_empty() {
          "PipeWire/WirePlumber".into()
        } else {
          self.snapshot.backend.clone()
        },
        status_str
      ),
      format!(
        "{} {}  ·  {}  ·  {}",
        AppConfig::icon(argvus_tui::icons::AUDIO),
        output_label,
        format!(
          "{} {}",
          self.snapshot.outputs.len(),
          tr(self.lang, "control_center.outputs")
        ),
        default_out_str
      ),
      format!(
        "{} {}  ·  {}  ·  {}",
        AppConfig::icon(argvus_tui::icons::MICROPHONE),
        input_label,
        format!(
          "{} {}",
          self.snapshot.inputs.len(),
          tr(self.lang, "control_center.inputs")
        ),
        default_in_str
      ),
      format!(
        "{} {}  ·  {}",
        AppConfig::icon(argvus_tui::icons::SPEAKER),
        devices_label,
        format!(
          "{} {}",
          total_devs,
          tr(self.lang, "control_center.devices_a41e65")
        )
      ),
    ]
  }
  /// Executes the `summary_rows` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn summary_rows(&self) -> Vec<String> {
    if !self.snapshot.available {
      return vec![
        format!(
          " {} {}",
          AppConfig::icon(argvus_tui::icons::MONITOR),
          tr(self.lang, "control_center.audio_system")
        ),
        format!("   Status: {}", tr(self.lang, "control_center.unavailable")),
      ];
    }

    let default_out = self
      .snapshot
      .default_output
      .and_then(|id| self.snapshot.outputs.iter().find(|d| d.id == id))
      .map(|d| {
        format!(
          "{} ({}%)",
          d.description,
          d.volume
            .map(|v| v.to_string())
            .unwrap_or_else(|| "—".into())
        )
      })
      .unwrap_or_else(|| "—".into());

    let default_in = self
      .snapshot
      .default_input
      .and_then(|id| self.snapshot.inputs.iter().find(|d| d.id == id))
      .map(|d| {
        format!(
          "{} ({}%)",
          d.description,
          d.volume
            .map(|v| v.to_string())
            .unwrap_or_else(|| "—".into())
        )
      })
      .unwrap_or_else(|| "—".into());

    vec![
      format!(
        " {} {}",
        AppConfig::icon(argvus_tui::icons::MONITOR),
        tr(self.lang, "control_center.audio_system")
      ),
      format!(
        "   Servidor:    {}",
        if self.snapshot.backend.is_empty() {
          "PipeWire / WirePlumber".into()
        } else {
          self.snapshot.backend.clone()
        }
      ),
      format!("   Status:      {}", tr(self.lang, "control_center.active")),
      "".into(),
      format!(
        " {} {}",
        AppConfig::icon(argvus_tui::icons::AUDIO),
        tr(self.lang, "control_center.default_devices")
      ),
      format!("   Saída:       {}", default_out),
      format!("   Entrada:     {}", default_in),
      "".into(),
      format!(
        " {} {}",
        AppConfig::icon(argvus_tui::icons::SPEAKER),
        tr(self.lang, "control_center.available_devices")
      ),
      format!("   Saídas:      {}", self.snapshot.outputs.len()),
      format!("   Entradas:    {}", self.snapshot.inputs.len()),
    ]
  }
  /// Executes the `device_rows` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn device_rows(&self) -> Vec<String> {
    if !self.snapshot.available {
      return vec![
        tr(
          self.lang,
          "control_center.pipewire_wireplumber_is_unavailable",
        )
        .into(),
      ];
    }
    let devices = self.devices();
    if devices.is_empty() {
      return vec![tr(self.lang, "control_center.no_devices_found").into()];
    }
    devices
      .iter()
      .map(|d| {
        let mut badges = Vec::new();
        if self.is_default_of(d) {
          badges.push(format!("● {}", tr(self.lang, "control_center.default")));
        }
        if self.muted_of(d) {
          badges.push(format!("✖ {}", tr(self.lang, "control_center.muted")));
        }
        let vol = self
          .volume_of(d)
          .map(|v| format!("{}%", v))
          .unwrap_or_else(|| "—".into());
        let badge_str = if badges.is_empty() {
          String::new()
        } else {
          format!("   {}", badges.join(" · "))
        };
        format!("{} · {}{}", d.description, vol, badge_str)
      })
      .collect()
  }
  /// Executes the `all_devices_rows` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn all_devices_rows(&self) -> Vec<String> {
    if !self.snapshot.available {
      return vec![
        tr(
          self.lang,
          "control_center.pipewire_wireplumber_is_unavailable",
        )
        .into(),
      ];
    }
    let mut rows = Vec::new();

    rows.push(format!(
      " {} {}",
      AppConfig::icon(argvus_tui::icons::AUDIO),
      tr(self.lang, "control_center.output_devices")
    ));
    if self.snapshot.outputs.is_empty() {
      rows.push(format!(
        "   {}",
        tr(self.lang, "control_center.no_output_found")
      ));
    } else {
      for d in &self.snapshot.outputs {
        let is_def = Some(d.id) == self.snapshot.default_output;
        let mut flags = Vec::new();
        if is_def {
          flags.push(tr(self.lang, "control_center.default"));
        }
        if d.muted {
          flags.push(tr(self.lang, "control_center.muted"));
        }
        let flag_str = if flags.is_empty() {
          String::new()
        } else {
          format!(" [{}]", flags.join(", "))
        };
        let vol = d
          .volume
          .map(|v| format!("{}%", v))
          .unwrap_or_else(|| "—".into());
        rows.push(format!("   {} ({}){}", d.description, vol, flag_str));
      }
    }

    rows.push("".into());
    rows.push(format!(
      " {} {}",
      AppConfig::icon(argvus_tui::icons::MICROPHONE),
      tr(self.lang, "control_center.input_devices")
    ));
    if self.snapshot.inputs.is_empty() {
      rows.push(format!(
        "   {}",
        tr(self.lang, "control_center.no_input_found")
      ));
    } else {
      for d in &self.snapshot.inputs {
        let is_def = Some(d.id) == self.snapshot.default_input;
        let mut flags = Vec::new();
        if is_def {
          flags.push(tr(self.lang, "control_center.default"));
        }
        if d.muted {
          flags.push(tr(self.lang, "control_center.muted"));
        }
        let flag_str = if flags.is_empty() {
          String::new()
        } else {
          format!(" [{}]", flags.join(", "))
        };
        let vol = d
          .volume
          .map(|v| format!("{}%", v))
          .unwrap_or_else(|| "—".into());
        rows.push(format!("   {} ({}){}", d.description, vol, flag_str));
      }
    }

    rows
  }
  /// Executes the `breadcrumb` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn breadcrumb(&self) -> String {
    let root = format!(
      "{} > {}",
      HomeCardId::Hardware.label(self.lang),
      tr(self.lang, "control_center.audio")
    );
    if self.page == AudioPage::Home {
      root
    } else {
      format!("{root} > {}", self.page_label())
    }
  }
  /// Executes the `page_label` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn page_label(&self) -> &'static str {
    match self.page {
      AudioPage::Home => tr(self.lang, "control_center.audio"),
      AudioPage::Summary => tr(self.lang, "control_center.summary"),
      AudioPage::Output => tr(self.lang, "control_center.output"),
      AudioPage::Input => tr(self.lang, "control_center.input"),
      AudioPage::Devices => tr(self.lang, "control_center.devices"),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use ratatui::{Terminal, backend::TestBackend};

  /// Builds an app on an output page with one device, without touching the audio server.
  fn app_with_output(volume: Option<u8>) -> AudioApp {
    let mut app = AudioApp::new(
      Lang::for_locale("en-US"),
      Theme::load(),
      Capabilities::default(),
    );
    app.snapshot.available = true;
    app.snapshot.outputs = vec![AudioDevice {
      id: 1,
      description: "Speakers".into(),
      direction: "output".into(),
      volume,
      ..Default::default()
    }];
    app.page = AudioPage::Output;
    app
  }

  #[test]
  /// Executes the `audio_home_rows_act_as_a_status_dashboard` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn audio_home_rows_act_as_a_status_dashboard() {
    let mut app = AudioApp::new(
      Lang::for_locale("en-US"),
      Theme::load(),
      Capabilities::default(),
    );
    app.snapshot.available = true;
    app.snapshot.backend = "PipeWire".into();
    app.snapshot.outputs = vec![AudioDevice {
      id: 42,
      description: "Speakers".into(),
      direction: "output".into(),
      volume: Some(80),
      ..Default::default()
    }];
    app.snapshot.inputs = vec![AudioDevice {
      id: 43,
      description: "Mic".into(),
      direction: "input".into(),
      volume: Some(50),
      ..Default::default()
    }];
    app.snapshot.default_output = Some(42);
    app.snapshot.default_input = Some(43);

    let rows = app.rows();
    assert_eq!(rows.len(), 4);
    assert!(rows[0].contains("Summary") || rows[0].contains("Resumo"));
    assert!(rows[0].contains("PipeWire"));
    assert!(rows[1].contains("Output") || rows[1].contains("Saída"));
    assert!(rows[1].contains("Speakers"));
    assert!(rows[2].contains("Input") || rows[2].contains("Entrada"));
    assert!(rows[2].contains("Mic"));
    assert!(rows[3].contains("2"));
  }

  #[test]
  /// Executes the `audio_device_rows_render_clean_badges` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn audio_device_rows_render_clean_badges() {
    let mut app = AudioApp::new(
      Lang::for_locale("en-US"),
      Theme::load(),
      Capabilities::default(),
    );
    app.snapshot.available = true;
    app.snapshot.outputs = vec![
      AudioDevice {
        id: 1,
        description: "Built-in Audio".into(),
        direction: "output".into(),
        volume: Some(70),
        muted: false,
        ..Default::default()
      },
      AudioDevice {
        id: 2,
        description: "USB Headset".into(),
        direction: "output".into(),
        volume: Some(50),
        muted: true,
        ..Default::default()
      },
    ];
    app.snapshot.default_output = Some(1);
    app.page = AudioPage::Output;

    let rows = app.rows();
    assert_eq!(rows.len(), 2);
    assert!(rows[0].contains("Built-in Audio · 70%"));
    assert!(rows[0].contains("Default") || rows[0].contains("Padrão"));
    assert!(rows[1].contains("USB Headset · 50%"));
    assert!(rows[1].contains("Muted") || rows[1].contains("Mudo"));
  }

  #[test]
  /// Executes the `audio_home_and_details_are_keyboard_navigable` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn audio_home_and_details_are_keyboard_navigable() {
    let mut app = AudioApp::new(
      Lang::for_locale("en-US"),
      Theme::load(),
      Capabilities::default(),
    );
    app.snapshot.outputs = vec![AudioDevice {
      id: 1,
      description: "Speakers".into(),
      direction: "output".into(),
      ..Default::default()
    }];
    app.handle(KeyCode::Down); // select Output
    app.handle(KeyCode::Enter);
    assert_eq!(app.page, AudioPage::Output);
    app.job = None;
    app.handle(KeyCode::Esc);
    assert_eq!(app.page, AudioPage::Home);
  }

  #[test]
  /// Enter stages the default device; nothing reaches the system until `s`, and leaving asks first.
  fn audio_default_stays_in_draft_until_save() {
    let mut app = AudioApp::new(
      Lang::for_locale("en-US"),
      Theme::load(),
      Capabilities::default(),
    );
    app.snapshot.outputs = vec![AudioDevice {
      id: 10,
      description: "Headphones".into(),
      direction: "output".into(),
      ..Default::default()
    }];
    app.page = AudioPage::Output;
    app.handle(KeyCode::Enter);
    assert_eq!(app.draft.default_output, Some(10));
    assert!(app.action.is_none());
    assert!(app.has_changes());
    app.handle(KeyCode::Esc); // asks before dropping the draft
    assert!(app.confirm_discard);
    app.handle(KeyCode::Esc); // cancel keeps the page and the draft
    assert!(!app.confirm_discard);
    assert_eq!(app.page, AudioPage::Output);
    assert_eq!(app.draft.default_output, Some(10));
  }

  #[test]
  /// Volume and mute keys change only the draft and show it in the rows.
  fn audio_volume_and_mute_stay_in_draft_until_save() {
    let mut app = app_with_output(Some(70));
    app.handle(KeyCode::Right);
    assert_eq!(app.draft.volumes.get(&1), Some(&75));
    app.handle(KeyCode::Char(' '));
    assert_eq!(app.draft.mutes.get(&1), Some(&true));
    assert!(app.action.is_none());
    let rows = app.rows();
    assert!(rows[0].contains("75%"));
    assert!(rows[0].contains("Muted") || rows[0].contains("Mudo"));
  }

  #[test]
  /// Saving with no pending change reports it instead of touching the audio server.
  fn audio_save_without_changes_reports_nothing_to_save() {
    let mut app = app_with_output(Some(70));
    app.handle(KeyCode::Char('s'));
    assert!(app.action.is_none());
    assert_eq!(
      app.status.as_ref().map(|s| s.text.as_str()),
      Some(tr(app.lang, "control_center.nothing_to_save"))
    );
  }

  #[test]
  /// Device pages expose their actions and the save key in the footer and render no button bar.
  fn audio_device_footer_lists_volume_default_and_save_keys() {
    let app = app_with_output(Some(70));
    assert_eq!(
      app.footer_hints(),
      tr(
        app.lang,
        "control_center.navigate_volume_enter_set_default_r_refresh_s_save_esc_back_help"
      )
    );
  }

  #[test]
  /// Executes the `audio_uses_shared_chrome_and_contextual_footer` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn audio_uses_shared_chrome_and_contextual_footer() {
    let app = AudioApp::new(
      Lang::for_locale("en-US"),
      Theme::load(),
      Capabilities::default(),
    );
    let mut terminal = Terminal::new(TestBackend::new(90, 25)).unwrap();
    terminal.draw(|frame| app.draw(frame)).unwrap();
    let text = terminal
      .backend()
      .buffer()
      .content
      .iter()
      .map(|cell| cell.symbol())
      .collect::<String>();
    assert!(text.contains("ARGVUS"));
    assert!(text.contains("Audio") || text.contains("Áudio"));
    assert!(text.contains("Enter") && text.contains("Back"));
  }
}
