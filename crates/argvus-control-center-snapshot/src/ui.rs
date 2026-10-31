//! Keyboard-first page; render only consumes cached state, never runs a process.
use crate::{
  backend::{SnapshotBackend, check_output, validate_description},
  model::*,
};
use argvus_control_center_core::{
  jobs::{JobHandle, JobManager, JobState},
  navigation::HomeCardId,
  privileged::{PrivilegedRequest, SystemSettingsOperation},
  process::{ProcessRunner, SystemProcessRunner},
};
use argvus_i18n::{Lang, tr};
use argvus_theme::Theme;
use argvus_tui::{
  components::{StatusKind, StatusMessage, draw_status},
  confirm::{ConfirmDialog, ConfirmOutcome, ConfirmState, draw_confirm},
  page::{Selection, list, readonly, shell},
  text_editor::{EditorOutcome, TextEditor},
};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{Frame, layout::Rect, text::Line};
use std::time::Duration;

#[derive(Debug, Clone)]
enum PendingAction {
  Create(String),
  Delete(Snapshot),
}
#[derive(Debug)]
enum Reply {
  Root(String),
  Inspected(Integration),
  Changed(Option<u32>),
  Compared(Comparison),
  Diffed(FileDiff),
}
#[derive(Debug, Clone, Copy)]
enum Phase {
  Detecting,
  Loading,
  Changing,
  Comparing,
  Diffing,
}

pub struct SnapshotApp {
  pub lang: Lang,
  pub theme: Theme,
  state: Option<Integration>,
  error: Option<SnapshotError>,
  phase: Phase,
  selection: Selection,
  config: Option<String>,
  authorized: bool,
  job: Option<JobHandle<SnapshotResult<Reply>>>,
  jobs: JobManager,
  editor: Option<TextEditor>,
  confirmation: Option<(ConfirmState, PendingAction)>,
  details: bool,
  status: Option<StatusMessage>,
  origin: Option<Snapshot>,
  comparison: Option<Comparison>,
  change_selection: Selection,
  diff: Option<FileDiff>,
  diff_scroll: usize,
  diff_horizontal: usize,
}
impl SnapshotApp {
  pub fn new(lang: Lang, theme: Theme) -> Self {
    Self {
      lang,
      theme,
      state: None,
      error: None,
      phase: Phase::Detecting,
      selection: Selection::default(),
      config: None,
      authorized: false,
      job: None,
      jobs: JobManager::default(),
      editor: None,
      confirmation: None,
      details: false,
      status: None,
      origin: None,
      comparison: None,
      change_selection: Selection::default(),
      diff: None,
      diff_scroll: 0,
      diff_horizontal: 0,
    }
  }
  pub fn set_theme(&mut self, theme: &Theme) {
    self.theme = theme.clone();
  }
  pub fn captures_text(&self) -> bool {
    self.editor.is_some() || self.confirmation.is_some()
  }
  pub fn paste(&mut self, text: &str) {
    if let Some(editor) = &mut self.editor {
      editor.paste(text);
    }
  }
  pub fn busy(&self) -> bool {
    self.job.is_some()
  }
  pub fn breadcrumb(&self) -> String {
    format!(
      "{} > {}",
      HomeCardId::System.label(self.lang),
      label(self.lang, "title")
    )
  }

  pub fn reload(&mut self) {
    if self.busy() {
      return;
    }
    self.editor = None;
    self.confirmation = None;
    self.error = None;
    self.comparison = None;
    self.diff = None;
    self.phase = Phase::Detecting;
    self.job = Some(self.jobs.spawn(|_| {
      Ok(
        SnapshotBackend::new(SystemProcessRunner)
          .root_filesystem()
          .map(Reply::Root),
      )
    }));
  }
  fn inspect(&mut self) {
    self.phase = Phase::Loading;
    let config = self.config.clone();
    let authorized = self.authorized;
    self.job = Some(self.jobs.spawn(move |_| {
      Ok(if authorized {
        elevated("inspect", &config.into_iter().collect::<Vec<_>>())
          .and_then(|output| {
            serde_json::from_slice::<SnapshotResult<Integration>>(&output).map_err(wire_error)?
          })
          .map(Reply::Inspected)
      } else {
        SnapshotBackend::new(SystemProcessRunner)
          .inspect(config.as_deref())
          .map(Reply::Inspected)
      })
    }));
  }
  pub fn poll(&mut self) -> bool {
    let Some(job) = &self.job else {
      return false;
    };
    let JobState::Finished(result) = job.try_state() else {
      return false;
    };
    self.job = None;
    match result.unwrap_or_else(|detail| Err(SnapshotError::new(ErrorKind::Snapper, detail))) {
      Ok(Reply::Root(filesystem)) if filesystem == "btrfs" => self.inspect(),
      Ok(Reply::Root(filesystem)) => {
        self.state = Some(Integration::NonBtrfs(filesystem));
      }
      Ok(Reply::Inspected(state)) => {
        let selected = self.selected().map(|snapshot| snapshot.number);
        if let Integration::Ready { config, snapshots } = &state {
          if self.config.as_deref().is_some_and(|old| old != config) {
            self.origin = None;
          }
          self.config = Some(config.clone());
          self.selection.index = selected
            .and_then(|number| {
              snapshots
                .iter()
                .position(|snapshot| snapshot.number == number)
            })
            .unwrap_or(0);
        }
        self.state = Some(state);
      }
      Ok(Reply::Changed(number)) => {
        self.status = Some(StatusMessage {
          kind: StatusKind::Success,
          text: number.map_or_else(
            || label(self.lang, "deleted").into(),
            |number| format!("{} #{number}", label(self.lang, "created")),
          ),
        });
        self.reload();
      }
      Ok(Reply::Compared(comparison)) => {
        self.change_selection.index = 0;
        self.comparison = Some(comparison);
        self.diff = None;
      }
      Ok(Reply::Diffed(diff)) => {
        self.diff_scroll = 0;
        self.diff_horizontal = 0;
        self.diff = Some(diff);
      }
      Err(error) => {
        // A mutation may have completed despite a lost reply. Read back, never retry it.
        if matches!(self.phase, Phase::Changing) {
          self.status = Some(StatusMessage {
            kind: StatusKind::Error,
            text: error_text(self.lang, &error),
          });
          self.reload();
        } else {
          // A configuration can be renamed or replaced outside this page.
          // Let the next explicit refresh discover the available choices again.
          if error.kind == ErrorKind::Invalid {
            self.config = None;
          }
          self.error = Some(error);
        }
      }
    }
    true
  }
  fn compare(&mut self) {
    let (Some(config), Some(origin), Some(destination)) = (
      self.config.clone(),
      self.origin.as_ref().map(|snapshot| snapshot.number),
      self.selected().map(|snapshot| snapshot.number),
    ) else {
      return;
    };
    if origin == destination {
      self.status = Some(StatusMessage {
        kind: StatusKind::Error,
        text: label(self.lang, "same_snapshot").into(),
      });
      return;
    }
    self.phase = Phase::Comparing;
    self.error = None;
    let authorized = self.authorized;
    self.job = Some(self.jobs.spawn(move |_| {
      Ok(if authorized {
        elevated(
          "status",
          &[config, origin.to_string(), destination.to_string()],
        )
        .and_then(|output| {
          serde_json::from_slice::<SnapshotResult<Comparison>>(&output).map_err(wire_error)?
        })
        .map(Reply::Compared)
      } else {
        SnapshotBackend::new(SystemProcessRunner)
          .compare(&config, origin, destination)
          .map(Reply::Compared)
      })
    }));
  }
  fn open_diff(&mut self) {
    let (Some(config), Some(comparison)) = (self.config.clone(), self.comparison.as_ref()) else {
      return;
    };
    let (Some(change), origin, destination) = (
      comparison.changes.get(self.change_selection.index),
      comparison.origin,
      comparison.destination,
    ) else {
      return;
    };
    let path = change.path.clone();
    self.phase = Phase::Diffing;
    self.error = None;
    let authorized = self.authorized;
    self.job = Some(self.jobs.spawn(move |_| {
      Ok(if authorized {
        elevated(
          "diff",
          &[config, origin.to_string(), destination.to_string(), path],
        )
        .and_then(|output| {
          serde_json::from_slice::<SnapshotResult<FileDiff>>(&output).map_err(wire_error)?
        })
        .map(Reply::Diffed)
      } else {
        SnapshotBackend::new(SystemProcessRunner)
          .diff(&config, origin, destination, &path)
          .map(Reply::Diffed)
      })
    }));
  }
  fn selected(&self) -> Option<&Snapshot> {
    match &self.state {
      Some(Integration::Ready { snapshots, .. }) => snapshots.get(self.selection.index),
      _ => None,
    }
  }
  fn change(&mut self, action: PendingAction) {
    let Some(config) = self.config.clone() else {
      return;
    };
    self.phase = Phase::Changing;
    self.error = None;
    self.status = None;
    self.job = Some(self.jobs.spawn(move |_| {
      let (action, value) = match action {
        PendingAction::Create(description) => ("create", description),
        PendingAction::Delete(snapshot) => ("delete", snapshot.number.to_string()),
      };
      Ok(
        elevated(action, &[config, value])
          .and_then(|output| {
            serde_json::from_slice::<SnapshotResult<Option<u32>>>(&output).map_err(wire_error)?
          })
          .map(Reply::Changed),
      )
    }));
  }

  pub fn handle_key(&mut self, key: KeyEvent) -> bool {
    if self.busy() {
      if key.code == KeyCode::Esc && matches!(self.phase, Phase::Comparing | Phase::Diffing) {
        if let Some(job) = &self.job {
          job.cancel();
        }
        self.job = None;
      }
      return false;
    }
    if let Some(editor) = &mut self.editor {
      match editor.handle(key) {
        EditorOutcome::Saved(description) => {
          let description = if description.trim().is_empty() {
            label(self.lang, "default_description").into()
          } else {
            description
          };
          match validate_description(&description) {
            Ok(()) => {
              self.editor = None;
              self.confirmation = Some((ConfirmState::new(), PendingAction::Create(description)));
            }
            Err(_) => {
              self.status = Some(StatusMessage {
                kind: StatusKind::Error,
                text: label(self.lang, "description_invalid").into(),
              })
            }
          }
        }
        EditorOutcome::Cancelled => self.editor = None,
        EditorOutcome::Pending => {}
      }
      return false;
    }
    if let Some((confirmation, action)) = &mut self.confirmation {
      match confirmation.handle(key.code) {
        ConfirmOutcome::Confirmed => {
          let action = action.clone();
          self.confirmation = None;
          self.change(action);
        }
        ConfirmOutcome::Cancelled => self.confirmation = None,
        ConfirmOutcome::Pending => {}
      }
      return false;
    }
    if key.code == KeyCode::Esc {
      if self.error.take().is_some() {
        return false;
      }
      if self.diff.take().is_some() {
        return false;
      }
      if self.comparison.take().is_some() {
        return false;
      }
      if self.details {
        self.details = false;
        return false;
      }
      return true;
    }
    if key.code == KeyCode::Char('r') {
      self.status = None;
      self.reload();
      return false;
    }
    if self.error.is_some() {
      if key.code == KeyCode::Char('a')
        && self
          .error
          .as_ref()
          .is_some_and(|error| error.kind == ErrorKind::Permission)
      {
        self.authorized = true;
        self.error = None;
        if matches!(self.phase, Phase::Comparing) {
          self.compare();
        } else if matches!(self.phase, Phase::Diffing) {
          self.open_diff();
        } else {
          self.inspect();
        }
      }
      return false;
    }
    if let Some(diff) = &self.diff {
      if matches!(diff, FileDiff::Text(_)) {
        match key.code {
          KeyCode::Down | KeyCode::Char('j') => {
            self.diff_scroll = self.diff_scroll.saturating_add(1)
          }
          KeyCode::Up | KeyCode::Char('k') => self.diff_scroll = self.diff_scroll.saturating_sub(1),
          KeyCode::PageDown => self.diff_scroll = self.diff_scroll.saturating_add(10),
          KeyCode::PageUp => self.diff_scroll = self.diff_scroll.saturating_sub(10),
          KeyCode::Right | KeyCode::Char('l') => {
            self.diff_horizontal = self.diff_horizontal.saturating_add(4)
          }
          KeyCode::Left | KeyCode::Char('h') => {
            self.diff_horizontal = self.diff_horizontal.saturating_sub(4)
          }
          _ => {}
        }
      }
      return false;
    }
    if let Some(comparison) = &self.comparison {
      match key.code {
        KeyCode::Down | KeyCode::Char('j') => {
          self.change_selection.index =
            (self.change_selection.index + 1).min(comparison.changes.len().saturating_sub(1))
        }
        KeyCode::Up | KeyCode::Char('k') => {
          self.change_selection.index = self.change_selection.index.saturating_sub(1)
        }
        KeyCode::Enter => self.open_diff(),
        _ => {}
      }
      return false;
    }
    let count = match &self.state {
      Some(Integration::Ready { snapshots, .. }) => snapshots.len(),
      Some(Integration::ChooseConfig(configs)) => configs.len(),
      _ => 0,
    };
    match key.code {
      KeyCode::Char('b') if self.selected().is_some() => {
        self.origin = self.selected().cloned();
      }
      KeyCode::Char('v') if self.origin.is_some() => self.compare(),
      KeyCode::Up | KeyCode::Char('k') => {
        self.selection.index = self.selection.index.saturating_sub(1)
      }
      KeyCode::Down | KeyCode::Char('j') => {
        self.selection.index = (self.selection.index + 1).min(count.saturating_sub(1))
      }
      KeyCode::Enter => {
        if let Some(Integration::ChooseConfig(configs)) = &self.state {
          self.origin = None;
          self.config = configs.get(self.selection.index).cloned();
          self.reload();
        } else {
          self.details = self.selected().is_some();
        }
      }
      KeyCode::Char('c') if matches!(self.state, Some(Integration::Ready { .. })) => {
        self.editor = Some(TextEditor::single_line(label(self.lang, "description"), ""));
      }
      KeyCode::Delete | KeyCode::Char('d') => {
        if let Some(snapshot) = self
          .selected()
          .filter(|snapshot| snapshot.deletable())
          .cloned()
        {
          self.confirmation = Some((ConfirmState::new(), PendingAction::Delete(snapshot)));
        }
      }
      _ => {}
    }
    false
  }
}

fn label(lang: Lang, key: &str) -> &'static str {
  tr(lang, &format!("control_center.snapshot.{key}"))
}
fn error_text(lang: Lang, error: &SnapshotError) -> String {
  let key = match error.kind {
    ErrorKind::Detection => "detection_error",
    ErrorKind::Permission => "permission",
    ErrorKind::Authentication => "authentication",
    ErrorKind::Timeout => "timeout",
    ErrorKind::Incompatible => "incompatible",
    ErrorKind::Invalid => "invalid",
    ErrorKind::Snapper => "failed",
  };
  format!("{}\n{}", label(lang, key), error.detail)
}
fn wire_error(error: serde_json::Error) -> SnapshotError {
  SnapshotError::new(ErrorKind::Invalid, error.to_string())
}
fn elevated(action: &str, values: &[String]) -> SnapshotResult<Vec<u8>> {
  let executable = std::env::current_exe()
    .map_err(|error| SnapshotError::new(ErrorKind::Snapper, error.to_string()))?;
  let operation = SystemSettingsOperation::new(SystemProcessRunner, executable.to_string_lossy());
  let privileged = PrivilegedRequest::new("snapshot", action, values.to_vec())
    .map_err(|detail| SnapshotError::new(ErrorKind::Invalid, detail))?;
  let request = operation
    .process_for(&privileged)
    .map_err(|detail| SnapshotError::new(ErrorKind::Invalid, detail))?
    .timeout(Duration::from_secs(if action == "inspect" {
      90
    } else {
      180
    }))
    .output_limit(1024 * 1024);
  SystemProcessRunner
    .run(&request)
    .map_err(|error| SnapshotError::new(ErrorKind::Snapper, error.to_string()))
    .and_then(check_output)
}

mod render;

#[cfg(test)]
mod tests;
