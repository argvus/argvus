//! Terminal UI rendering and interaction of the Session page.
use crate::{
  backend,
  journal::JournalEntry,
  model::{AutostartEntry, Component, ComponentStatus, DiagnosticsEntry, SessionPage, manifest},
};
use argvus_control_center_core::navigation::HomeCardId;
use argvus_control_center_core::{
  config::AppConfig,
  jobs::{JobHandle, JobManager, JobState},
};
use argvus_i18n::{Lang, tr};
use argvus_theme::Theme;
use argvus_tui::buttons::{Button, ButtonKind};
use argvus_tui::components::{
  ConfirmationDialog, ConfirmationOutcome, ConfirmationState, StatusKind, StatusMessage,
  draw_confirmation,
};
use argvus_tui::page::{list, readonly, shell, status};
use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::Line;

mod input;
mod jobs;
mod render;
mod rows;

#[derive(Debug, Clone)]
enum Pending {
  Restart(String),
  ToggleAutostart(String, bool),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SessionButton {
  Restart,
  Enable,
  Disable,
  NextLogFilter,
  Refresh,
}

#[derive(Debug, Clone)]
enum JobData {
  HomeSummary(Vec<Component>, Vec<AutostartEntry>),
  Components(Vec<Component>),
  Autostart(Vec<AutostartEntry>),
  Diagnostics(Vec<DiagnosticsEntry>),
  Logs(Vec<JournalEntry>),
  Action(String),
}

pub struct SessionApp {
  pub page: SessionPage,
  selected: usize,
  on_buttons: bool,
  button_selected: usize,
  button_from: Option<usize>,
  components: Vec<Component>,
  autostart: Vec<AutostartEntry>,
  diagnostics: Vec<DiagnosticsEntry>,
  logs: Vec<JournalEntry>,
  log_filter: Option<String>,
  job: Option<JobHandle<JobData>>,
  action: Option<JobHandle<JobData>>,
  pending: Option<Pending>,
  confirmation: ConfirmationState,
  manager: JobManager,
  pub lang: Lang,
  pub theme: Theme,
  pub status: Option<StatusMessage>,
}

impl SessionApp {
  /// Replaces the semantic theme used by this page.
  pub fn set_theme(&mut self, theme: &Theme) {
    self.theme = theme.clone();
  }

  pub fn new(lang: Lang, theme: Theme) -> Self {
    Self {
      page: SessionPage::Home,
      selected: 0,
      on_buttons: false,
      button_selected: 0,
      button_from: None,
      components: vec![],
      autostart: vec![],
      diagnostics: vec![],
      logs: vec![],
      log_filter: None,
      job: None,
      action: None,
      pending: None,
      confirmation: ConfirmationState::default(),
      manager: JobManager::default(),
      lang,
      theme,
      status: None,
    }
  }

  fn open(&mut self) {
    match self.page {
      SessionPage::Home => {
        self.page = [
          SessionPage::Components,
          SessionPage::Autostart,
          SessionPage::Diagnostics,
          SessionPage::Logs,
        ][self.selected.min(3)];
        self.selected = 0;
        self.refresh();
      }
      SessionPage::Components => {
        if let Some(id) = self.components.get(self.selected).map(|c| c.id.clone()) {
          self.pending = Some(Pending::Restart(id));
        }
      }
      SessionPage::Autostart => {
        if let Some(entry) = self.autostart.get(self.selected) {
          self.pending = Some(Pending::ToggleAutostart(entry.id.clone(), !entry.enabled));
        }
      }
      SessionPage::Logs => {
        if self.logs.get(self.selected).is_some() {
          self.page = SessionPage::LogDetail(self.selected);
        }
      }
      SessionPage::Diagnostics | SessionPage::LogDetail(_) => {}
    }
  }

  fn toggle_log_filter(&mut self) {
    let mut units: Vec<String> = self
      .components
      .iter()
      .filter_map(|component| component.unit.clone())
      .collect();
    units.extend(self.logs.iter().filter_map(|entry| entry.unit.clone()));
    units.sort();
    units.dedup();
    if units.is_empty() {
      return;
    }
    self.log_filter = match self.log_filter.as_deref() {
      None => units.first().cloned(),
      Some(current) => units
        .iter()
        .position(|unit| unit == current)
        .and_then(|index| units.get(index + 1).cloned()),
    };
    self.selected = 0;
    self.refresh();
  }

  fn toggle_buttons(&mut self, backwards: bool) {
    let count = self.buttons().len();
    if count == 0 {
      return;
    }
    if self.on_buttons {
      self.on_buttons = false;
      if let Some(index) = self.button_from.take() {
        self.selected = index;
      }
    } else {
      self.button_from = Some(self.selected);
      self.button_selected = if backwards { count - 1 } else { 0 };
      self.on_buttons = true;
    }
  }

  fn activate_button(&mut self) {
    if self.job.is_some() || self.action.is_some() {
      return;
    }
    let actions = self.buttons();
    let Some((button, _)) = actions.get(self.button_selected) else {
      return;
    };
    match button {
      SessionButton::Restart => {
        if let Some(id) = self.components.get(self.selected).map(|c| c.id.clone()) {
          self.pending = Some(Pending::Restart(id));
        }
      }
      SessionButton::Enable => {
        if let Some(id) = self.autostart.get(self.selected).map(|e| e.id.clone()) {
          self.pending = Some(Pending::ToggleAutostart(id, true));
        }
      }
      SessionButton::Disable => {
        if let Some(id) = self.autostart.get(self.selected).map(|e| e.id.clone()) {
          self.pending = Some(Pending::ToggleAutostart(id, false));
        }
      }
      SessionButton::NextLogFilter => self.toggle_log_filter(),
      SessionButton::Refresh => self.refresh(),
    }
  }

  fn buttons(&self) -> Vec<(SessionButton, Button)> {
    let primary = ButtonKind::Primary;
    let secondary = ButtonKind::Secondary;
    match self.page {
      SessionPage::Components => vec![
        (
          SessionButton::Restart,
          Button::new(tr(self.lang, "control_center.restart"), primary),
        ),
        (
          SessionButton::Refresh,
          Button::new(tr(self.lang, "control_center.refresh"), secondary),
        ),
      ],
      SessionPage::Autostart => vec![
        (
          SessionButton::Enable,
          Button::new(tr(self.lang, "control_center.enable_8adac7"), primary),
        ),
        (
          SessionButton::Disable,
          Button::new(tr(self.lang, "control_center.disable_5de12e"), secondary),
        ),
        (
          SessionButton::Refresh,
          Button::new(tr(self.lang, "control_center.refresh"), secondary),
        ),
      ],
      SessionPage::Logs => vec![
        (
          SessionButton::NextLogFilter,
          Button::new(
            format!(
              "{}: {}",
              tr(self.lang, "control_center.service"),
              self.log_filter.as_deref().unwrap_or("—")
            ),
            secondary,
          ),
        ),
        (
          SessionButton::Refresh,
          Button::new(tr(self.lang, "control_center.refresh"), secondary),
        ),
      ],
      SessionPage::Home | SessionPage::Diagnostics | SessionPage::LogDetail(_) => Vec::new(),
    }
  }
}

fn merge_component_order(components: Vec<Component>) -> Vec<Component> {
  let order: Vec<String> = manifest()
    .into_iter()
    .map(|component| component.id)
    .collect();
  let mut sorted = components;
  sorted.sort_by_key(|component| {
    order
      .iter()
      .position(|id| id == &component.id)
      .unwrap_or(usize::MAX)
  });
  sorted
}

fn log_line(entry: &JournalEntry) -> String {
  format!(
    "{:8} {:28} {:7} {}",
    entry.rendered_timestamp(),
    entry.unit.as_deref().unwrap_or("user"),
    entry.pid.as_deref().unwrap_or("—"),
    entry.message.as_deref().unwrap_or("")
  )
}

impl SessionApp {
  pub fn onboarding_ready(&self) -> bool {
    self.action.is_none() && self.pending.is_none()
  }
}

#[cfg(test)]
mod tests {
  use super::rows::{autostart_row, component_row};
  use super::*;

  fn component(id: &str, status: ComponentStatus) -> Component {
    Component {
      id: id.into(),
      display_name: id.into(),
      description: String::new(),
      process: vec![],
      unit: None,
      essential: false,
      status,
      pid: Some(42),
    }
  }

  #[test]
  fn home_rows_act_as_dashboard() {
    let mut app = SessionApp::new(Lang::for_locale("en-US"), Theme::load());
    app.components = vec![
      component("a", ComponentStatus::Running),
      component("b", ComponentStatus::Running),
      component("c", ComponentStatus::Failed),
    ];
    app.autostart = vec![AutostartEntry {
      id: "x".into(),
      name: "X".into(),
      command: "x".into(),
      from_system: true,
      enabled: true,
      path: "/tmp/x.desktop".into(),
    }];
    let rows = app.home_rows();
    assert_eq!(rows.len(), 4);
    assert!(rows[0].contains("2 active"));
    assert!(rows[1].contains("1 enabled"));
    assert!(rows[2].contains("1 failed"));
  }

  #[test]
  fn component_rows_show_state_badges() {
    let rows = [
      component("a", ComponentStatus::Running),
      component("b", ComponentStatus::Failed),
    ]
    .iter()
    .map(|component| component_row(Lang::for_locale("en-US"), component))
    .collect::<Vec<_>>();
    assert!(rows[0].contains("●") && rows[0].contains("Active"));
    assert!(rows[1].contains("✕") && rows[1].contains("Failed"));
  }

  #[test]
  fn autostart_rows_show_origin() {
    let entry = AutostartEntry {
      id: "sys".into(),
      name: "Sys".into(),
      command: "run --x".into(),
      from_system: true,
      enabled: false,
      path: "/etc/xdg/autostart/sys.desktop".into(),
    };
    let row = autostart_row(Lang::for_locale("en-US"), &entry);
    assert!(row.contains("Disabled"));
    assert!(row.contains("system"));
  }

  #[test]
  fn order_follows_manifest() {
    let components = merge_component_order(vec![
      component("hyprpaper", ComponentStatus::Running),
      component("argvus-session", ComponentStatus::Running),
    ]);
    assert_eq!(components[0].id, "argvus-session");
  }

  #[test]
  fn session_app_navigates_and_opens_pages() {
    let mut app = SessionApp::new(Lang::for_locale("en-US"), Theme::load());
    app.handle(KeyCode::Down);
    app.handle(KeyCode::Enter);
    assert_eq!(app.page, SessionPage::Autostart);
    app.handle(KeyCode::Esc);
    assert_eq!(app.page, SessionPage::Home);
  }

  #[test]
  fn session_app_renders_components_page() {
    let mut app = SessionApp::new(Lang::for_locale("en-US"), Theme::load());
    app.page = SessionPage::Components;
    app.components = vec![
      component("waybar", ComponentStatus::Running),
      component("hypridle", ComponentStatus::Stopped),
    ];
    let mut terminal = ratatui::Terminal::new(ratatui::backend::TestBackend::new(110, 25)).unwrap();
    terminal.draw(|frame| app.draw(frame)).unwrap();
    let text = terminal
      .backend()
      .buffer()
      .content
      .iter()
      .map(|cell| cell.symbol())
      .collect::<String>();
    assert!(text.contains("waybar") && (text.contains("Restart") || text.contains("Reiniciar")));
  }

  #[test]
  fn session_app_renders_home() {
    let mut app = SessionApp::new(Lang::for_locale("en-US"), Theme::load());
    let mut terminal = ratatui::Terminal::new(ratatui::backend::TestBackend::new(110, 25)).unwrap();
    terminal.draw(|frame| app.draw(frame)).unwrap();
  }

  #[test]
  fn diagnostics_rows_render_symbols() {
    let mut app = SessionApp::new(Lang::for_locale("en-US"), Theme::load());
    app.diagnostics = vec![
      DiagnosticsEntry::good("A", "ok"),
      DiagnosticsEntry::warning("B", "warn"),
      DiagnosticsEntry::bad("C", "bad"),
    ];
    let rows = app.diagnostics_rows();
    assert!(rows[0].starts_with(" ✓ "));
    assert!(rows[1].starts_with(&format!(" {} ", argvus_tui::icons::WARNING)));
    assert!(rows[2].starts_with(" ✕ "));
  }
}
