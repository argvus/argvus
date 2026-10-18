//! Implements terminal UI rendering and interaction in crate `argvus control center session`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
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

#[derive(Debug, Clone)]
/// Defines `Pending`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
enum Pending {
  Restart(String),
  ToggleAutostart(String, bool),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Defines `SessionButton`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
enum SessionButton {
  Restart,
  Enable,
  Disable,
  NextLogFilter,
  Refresh,
}

#[derive(Debug, Clone)]
/// Defines `JobData`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
enum JobData {
  HomeSummary(Vec<Component>, Vec<AutostartEntry>),
  Components(Vec<Component>),
  Autostart(Vec<AutostartEntry>),
  Diagnostics(Vec<DiagnosticsEntry>),
  Logs(Vec<JournalEntry>),
  Action(String),
}

/// Represents `SessionApp`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
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
  /// Executes the `reload` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn reload(&mut self) {
    self.refresh();
  }

  /// Replaces the semantic theme used by this page.
  pub fn set_theme(&mut self, theme: &Theme) {
    self.theme = theme.clone();
  }

  /// Constructs `new` with this module's expected initial state. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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

  /// Executes the `refresh` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn refresh(&mut self) {
    if self.job.is_some() {
      return;
    }
    let page = self.page;
    let filter = self.log_filter.clone();
    let home = self.page == SessionPage::Home;
    self.job = Some(self.manager.spawn(move |_| match page {
      SessionPage::Components => Ok(JobData::Components(backend::list_components())),
      SessionPage::Autostart => Ok(JobData::Autostart(backend::autostart_entries())),
      SessionPage::Diagnostics => {
        let components = backend::list_components();
        Ok(JobData::Diagnostics(backend::diagnostics(&components)))
      }
      SessionPage::Logs | SessionPage::LogDetail(_) => {
        backend::logs(filter.as_deref(), 200).map(JobData::Logs)
      }
      _ if home => {
        let (components, autostart) =
          rayon::join(backend::list_components, backend::autostart_entries);
        Ok(JobData::HomeSummary(components, autostart))
      }
      _ => Ok(JobData::Components(backend::list_components())),
    }));
    self.status = Some(StatusMessage {
      kind: StatusKind::Info,
      text: tr(self.lang, "control_center.loading_session").into(),
    });
  }

  /// Executes the `poll` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn poll(&mut self) -> bool {
    let mut changed = false;
    if let Some(job) = &self.job
      && let JobState::Finished(result) = job.try_state()
    {
      self.job = None;
      match result {
        Ok(JobData::HomeSummary(components, autostart)) => {
          self.components = merge_component_order(components);
          self.autostart = autostart;
          self.selected = self.selected.min(self.selection_len().saturating_sub(1));
          self.status = None;
        }
        Ok(JobData::Components(components)) => {
          self.components = merge_component_order(components);
          self.selected = self.selected.min(self.selection_len().saturating_sub(1));
          self.status = None;
        }
        Ok(JobData::Autostart(entries)) => {
          self.autostart = entries;
          self.selected = self.selected.min(self.selection_len().saturating_sub(1));
          self.status = None;
        }
        Ok(JobData::Diagnostics(entries)) => {
          self.diagnostics = entries;
          self.selected = self.selected.min(self.selection_len().saturating_sub(1));
          self.status = None;
        }
        Ok(JobData::Logs(entries)) => {
          self.logs = entries;
          self.selected = self.selected.min(self.logs.len().saturating_sub(1));
          self.status = None;
        }
        Ok(JobData::Action(text)) => {
          self.status = Some(StatusMessage {
            kind: StatusKind::Success,
            text,
          });
          self.refresh();
        }
        Err(e) => {
          self.status = Some(StatusMessage {
            kind: StatusKind::Error,
            text: e,
          })
        }
      };
      changed = true;
    }
    if let Some(job) = &self.action
      && let JobState::Finished(result) = job.try_state()
    {
      self.action = None;
      match result {
        Ok(JobData::Action(text)) => {
          self.status = Some(StatusMessage {
            kind: StatusKind::Success,
            text,
          });
          self.refresh();
        }
        Ok(_) => {}
        Err(e) => {
          self.status = Some(StatusMessage {
            kind: StatusKind::Error,
            text: e,
          });
          self.refresh();
        }
      };
      changed = true;
    }
    changed
  }

  /// Processes `handle` in this module's event flow. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn handle(&mut self, key: KeyCode) -> bool {
    if self.pending.is_none() && self.action.is_none() && self.job.is_none() && self.on_buttons {
      match key {
        KeyCode::Tab | KeyCode::BackTab => {
          self.toggle_buttons(key == KeyCode::BackTab);
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
    if matches!(key, KeyCode::Esc | KeyCode::Left) && self.pending.is_none() {
      if self.page == SessionPage::Home {
        return true;
      }
      self.page = if matches!(self.page, SessionPage::LogDetail(_)) {
        SessionPage::Logs
      } else {
        SessionPage::Home
      };
      self.selected = 0;
      self.on_buttons = false;
      self.button_from = None;
      if self.page == SessionPage::Home {
        self.refresh();
      }
      return false;
    }
    if self.job.is_some() || self.action.is_some() {
      return false;
    }
    if self.pending.is_some() {
      match self.confirmation.handle(key) {
        ConfirmationOutcome::Confirmed => {
          self.confirmation = ConfirmationState::default();
          let pending = self.pending.take().unwrap();
          self.status = Some(StatusMessage {
            kind: StatusKind::Info,
            text: tr(self.lang, "control_center.applying").into(),
          });
          let lang = self.lang;
          self.action = Some(self.manager.spawn(move |_| match pending {
            Pending::Restart(id) => {
              backend::restart_component(&id)?;
              Ok(JobData::Action(format!(
                "{} '{id}'",
                tr(lang, "control_center.restarted")
              )))
            }
            Pending::ToggleAutostart(id, enabled) => {
              backend::set_autostart(&id, enabled)?;
              Ok(JobData::Action(format!(
                "{} '{id}'",
                if enabled {
                  tr(lang, "control_center.enabled")
                } else {
                  tr(lang, "control_center.disabled")
                }
              )))
            }
          }));
        }
        ConfirmationOutcome::Cancelled => {
          self.pending = None;
          self.confirmation = ConfirmationState::default();
        }
        ConfirmationOutcome::Pending => {}
      }
      return false;
    }
    match key {
      KeyCode::Tab | KeyCode::BackTab => self.toggle_buttons(key == KeyCode::BackTab),
      KeyCode::Char('r') => self.refresh(),
      KeyCode::Up | KeyCode::Char('k') => self.selected = self.selected.saturating_sub(1),
      KeyCode::Down | KeyCode::Char('j') => self.selected = self.selected.saturating_add(1),
      KeyCode::Home => self.selected = 0,
      KeyCode::End => self.selected = self.selection_len().saturating_sub(1),
      KeyCode::PageUp => self.selected = self.selected.saturating_sub(8),
      KeyCode::PageDown => {
        self.selected = self
          .selected
          .saturating_add(8)
          .min(self.selection_len().saturating_sub(1))
      }
      KeyCode::Enter | KeyCode::Right => self.open(),
      _ => {}
    }
    self.selected = self.selected.min(self.selection_len().saturating_sub(1));
    false
  }

  /// Executes the `selection_len` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn selection_len(&self) -> usize {
    match self.page {
      SessionPage::Home => 4,
      SessionPage::Components => self.components.len(),
      SessionPage::Autostart => self.autostart.len(),
      SessionPage::Diagnostics => self.diagnostics.len(),
      SessionPage::Logs => self.logs.len(),
      SessionPage::LogDetail(_) => 1,
    }
  }

  /// Executes the `open` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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

  /// Applies the `toggle_log_filter` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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

  /// Applies the `toggle_buttons` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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

  /// Executes the `buttons` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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

  /// Executes the `footer_hints` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn footer_hints(&self) -> &'static str {
    match self.page {
      SessionPage::Home | SessionPage::Diagnostics => tr(
        self.lang,
        "control_center.navigate_enter_open_r_refresh_esc_back_help",
      ),
      SessionPage::LogDetail(_) => tr(self.lang, "control_center.scroll_esc_back_help"),
      _ => tr(
        self.lang,
        "control_center.navigate_tab_actions_enter_activate_move_r_refresh_esc_back_help",
      ),
    }
  }

  /// Executes the `breadcrumb` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn breadcrumb(&self) -> String {
    let root = format!(
      "{} > {}",
      HomeCardId::Preferences.label(self.lang),
      tr(self.lang, "control_center.session")
    );
    if self.page == SessionPage::Home {
      root
    } else {
      format!("{root} > {}", self.page_label())
    }
  }

  /// Executes the `page_label` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn page_label(&self) -> String {
    match self.page {
      SessionPage::Home => tr(self.lang, "control_center.session").into(),
      SessionPage::Components => tr(self.lang, "control_center.components").into(),
      SessionPage::Autostart => tr(self.lang, "control_center.autostart").into(),
      SessionPage::Diagnostics => tr(self.lang, "control_center.diagnostics").into(),
      SessionPage::Logs => tr(self.lang, "control_center.logs").into(),
      SessionPage::LogDetail(_) => tr(self.lang, "control_center.logs_details").into(),
    }
  }

  /// Executes the `home_rows` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn home_rows(&self) -> Vec<String> {
    let running = self
      .components
      .iter()
      .filter(|component| component.status == ComponentStatus::Running)
      .count();
    let failed = self
      .components
      .iter()
      .filter(|component| component.status == ComponentStatus::Failed)
      .count();
    let autostart_enabled = self.autostart.iter().filter(|entry| entry.enabled).count();
    vec![
      format!(
        "{} {}  ·  {} {}",
        AppConfig::icon(argvus_tui::icons::APPS),
        tr(self.lang, "control_center.components"),
        running,
        tr(self.lang, "control_center.active_ae7190"),
      ),
      format!(
        "{} {}  ·  {} {}",
        AppConfig::icon(argvus_tui::icons::BOOT),
        tr(self.lang, "control_center.autostart"),
        autostart_enabled,
        tr(self.lang, "control_center.enabled_72aa06"),
      ),
      format!(
        "{} {}  ·  {} {}",
        AppConfig::icon(argvus_tui::icons::DIAGNOSTICS),
        tr(self.lang, "control_center.diagnostics"),
        failed,
        tr(self.lang, "control_center.failed_cc0486"),
      ),
      format!(
        "{} {}",
        AppConfig::icon(argvus_tui::icons::LOGS),
        tr(self.lang, "control_center.logs")
      ),
    ]
  }

  /// Executes the `diagnostics_rows` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn diagnostics_rows(&self) -> Vec<String> {
    if self.job.is_some() && self.diagnostics.is_empty() {
      return vec![tr(self.lang, "control_center.loading_diagnostics").into()];
    }
    self
      .diagnostics
      .iter()
      .map(|entry| {
        let symbol = if entry.warn {
          argvus_tui::icons::WARNING
        } else if entry.ok {
          "✓"
        } else {
          "✕"
        };
        format!(" {symbol} {}  ·  {}", entry.label, entry.detail)
      })
      .collect()
  }

  /// Executes the `component_rows` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn component_rows(&self) -> Vec<String> {
    self
      .components
      .iter()
      .map(|component| component_row(self.lang, component))
      .collect()
  }

  /// Executes the `autostart_rows` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn autostart_rows(&self) -> Vec<String> {
    self
      .autostart
      .iter()
      .map(|entry| autostart_row(self.lang, entry))
      .collect()
  }

  /// Executes the `log_rows` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn log_rows(&self) -> Vec<String> {
    self.logs.iter().map(log_line).collect()
  }

  /// Executes the `log_detail_lines` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn log_detail_lines(&self, index: usize) -> Vec<Line<'static>> {
    let Some(entry) = self.logs.get(index) else {
      return vec![Line::from(tr(self.lang, "control_center.log_not_found"))];
    };
    vec![
      Line::from(format!(
        "{}: {}",
        tr(self.lang, "control_center.timestamp"),
        entry.rendered_timestamp()
      )),
      Line::from(format!(
        "{}: {}",
        tr(self.lang, "control_center.unit"),
        entry.unit.as_deref().unwrap_or("—")
      )),
      Line::from(format!("PID: {}", entry.pid.as_deref().unwrap_or("—"))),
      Line::from(""),
      Line::from(entry.message.clone().unwrap_or_default()),
    ]
  }

  /// Renders `draw` while respecting the current domain state and semantic theme. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn draw(&mut self, frame: &mut Frame) {
    let area = frame.area();
    let body = shell(
      frame,
      area,
      &self.theme,
      self.lang,
      &self.breadcrumb(),
      self.footer_hints(),
    );
    let lines = match self.page {
      SessionPage::Home => self.home_rows(),
      SessionPage::Components => self.component_rows(),
      SessionPage::Autostart => self.autostart_rows(),
      SessionPage::Diagnostics => self.diagnostics_rows(),
      SessionPage::Logs => self.log_rows(),
      SessionPage::LogDetail(_) => Vec::new(),
    };
    if let SessionPage::LogDetail(index) = self.page {
      let entries = self.log_detail_lines(index);
      readonly(frame, body, &self.theme, &entries);
      return;
    }
    let buttons = self.buttons();
    let raw_buttons: Vec<Button> = buttons.into_iter().map(|(_, button)| button).collect();
    let (body, button_area) = if raw_buttons.is_empty() {
      (body, None)
    } else {
      let button_height = argvus_tui::buttons::height(&raw_buttons, body.width).min(body.height);
      let split =
        Layout::vertical([Constraint::Min(1), Constraint::Length(button_height)]).split(body);
      (split[0], Some(split[1]))
    };
    let list_selection = if self.job.is_some() {
      usize::MAX
    } else {
      self.selected.min(lines.len().saturating_sub(1))
    };
    list(frame, body, &self.theme, &lines, list_selection);
    if let Some(button_area) = button_area {
      let focus = if self.on_buttons {
        self.button_selected
      } else {
        usize::MAX
      };
      argvus_tui::buttons::draw(frame, button_area, &raw_buttons, focus, &self.theme);
    }
    if let Some(s) = &self.status {
      status(frame, area, &self.theme, s);
    }
    if let Some(pending) = &self.pending {
      self.draw_pending(frame, area, pending);
    }
  }

  /// Renders `draw_pending` while respecting the current domain state and semantic theme. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn draw_pending(&self, frame: &mut Frame, area: Rect, pending: &Pending) {
    let (title, message) = match pending {
      Pending::Restart(id) => (
        tr(self.lang, "control_center.confirm_restart"),
        format!(
          "{} {id}?",
          tr(self.lang, "control_center.restart_component")
        ),
      ),
      Pending::ToggleAutostart(id, enabled) => (
        tr(self.lang, "control_center.confirm_autostart"),
        format!(
          "{} {id}?",
          if *enabled {
            tr(self.lang, "control_center.enable_8adac7")
          } else {
            tr(self.lang, "control_center.disable_5de12e")
          }
        ),
      ),
    };
    draw_confirmation(
      frame,
      area,
      &self.theme,
      ConfirmationDialog {
        title,
        message: &message,
        confirm_label: tr(self.lang, "control_center.continue"),
        cancel_label: tr(self.lang, "control_center.cancel"),
        confirm_selected: self.confirmation.confirm_selected,
      },
    )
  }
}

/// Executes the `merge_component_order` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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

/// Executes the `component_row` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn component_row(lang: Lang, component: &Component) -> String {
  let (symbol, label) = match component.status {
    ComponentStatus::Running => ("●", tr(lang, "control_center.active_095d39")),
    ComponentStatus::Stopped => ("○", tr(lang, "control_center.stopped_c2dfd3")),
    ComponentStatus::Failed => ("✕", tr(lang, "control_center.failed_b852d2")),
  };
  let role = if component.essential {
    tr(lang, "control_center.essential")
  } else {
    tr(lang, "control_center.optional")
  };
  let pid = component
    .pid
    .map(|pid| pid.to_string())
    .unwrap_or_else(|| "—".into());
  format!(
    "{}  {}  {symbol} {}  ·  PID {}  ·  {}",
    component.display_name, role, label, pid, component.description
  )
}

/// Executes the `autostart_row` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn autostart_row(lang: Lang, entry: &AutostartEntry) -> String {
  let symbol = if entry.enabled { "●" } else { "○" };
  let state = if entry.enabled {
    tr(lang, "control_center.enabled")
  } else {
    tr(lang, "control_center.disabled")
  };
  let origin = if entry.from_system {
    tr(lang, "control_center.system_8fb556")
  } else {
    tr(lang, "control_center.user_e7acba")
  };
  format!(
    "{}  {symbol} {} · {}: {} · {}",
    entry.name,
    state,
    tr(lang, "control_center.source"),
    origin,
    entry.command
  )
}

/// Executes the `log_line` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn log_line(entry: &JournalEntry) -> String {
  format!(
    "{:8} {:28} {:7} {}",
    entry.rendered_timestamp(),
    entry.unit.as_deref().unwrap_or("user"),
    entry.pid.as_deref().unwrap_or("—"),
    entry.message.as_deref().unwrap_or("")
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  /// Executes the `component` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
  /// Executes the `home_rows_act_as_dashboard` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
  /// Executes the `component_rows_show_state_badges` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
  /// Executes the `autostart_rows_show_origin` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
  /// Executes the `order_follows_manifest` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn order_follows_manifest() {
    let components = merge_component_order(vec![
      component("hyprpaper", ComponentStatus::Running),
      component("argvus-session", ComponentStatus::Running),
    ]);
    assert_eq!(components[0].id, "argvus-session");
  }

  #[test]
  /// Executes the `session_app_navigates_and_opens_pages` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn session_app_navigates_and_opens_pages() {
    let mut app = SessionApp::new(Lang::for_locale("en-US"), Theme::load());
    app.handle(KeyCode::Down);
    app.handle(KeyCode::Enter);
    assert_eq!(app.page, SessionPage::Autostart);
    app.handle(KeyCode::Esc);
    assert_eq!(app.page, SessionPage::Home);
  }

  #[test]
  /// Executes the `session_app_renders_components_page` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
  /// Executes the `session_app_renders_home` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn session_app_renders_home() {
    let mut app = SessionApp::new(Lang::for_locale("en-US"), Theme::load());
    let mut terminal = ratatui::Terminal::new(ratatui::backend::TestBackend::new(110, 25)).unwrap();
    terminal.draw(|frame| app.draw(frame)).unwrap();
  }

  #[test]
  /// Executes the `diagnostics_rows_render_symbols` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
