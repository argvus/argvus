//! Terminal UI rendering and interaction of the Audio page.
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

mod input;
mod jobs;
mod render;
mod rows;

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
  fn success(&mut self, text: impl Into<String>) {
    self.status = Some(StatusMessage {
      kind: StatusKind::Success,
      text: text.into(),
    });
  }
  fn error(&mut self, text: impl Into<String>) {
    self.status = Some(StatusMessage {
      kind: StatusKind::Error,
      text: text.into(),
    });
  }
  fn normalize(&mut self) {
    self.selected.normalize(self.row_count());
  }
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
}

#[cfg(test)]
mod tests;
