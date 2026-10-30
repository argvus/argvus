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
}
#[derive(Debug, Clone, Copy)]
enum Phase {
  Detecting,
  Loading,
  Changing,
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
        self.inspect();
      }
      return false;
    }
    let count = match &self.state {
      Some(Integration::Ready { snapshots, .. }) => snapshots.len(),
      Some(Integration::ChooseConfig(configs)) => configs.len(),
      _ => 0,
    };
    match key.code {
      KeyCode::Up | KeyCode::Char('k') => {
        self.selection.index = self.selection.index.saturating_sub(1)
      }
      KeyCode::Down | KeyCode::Char('j') => {
        self.selection.index = (self.selection.index + 1).min(count.saturating_sub(1))
      }
      KeyCode::Enter => {
        if let Some(Integration::ChooseConfig(configs)) = &self.state {
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
