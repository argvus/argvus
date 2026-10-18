//! Implements terminal UI rendering and interaction in crate `argvus control center services`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use crate::{
  backend,
  journal::JournalEntry,
  model::{ServicePage, Unit},
};
use argvus_control_center_core::navigation::HomeCardId;
use argvus_control_center_core::{
  config::AppConfig,
  jobs::{JobHandle, JobManager, JobState},
  privileged::{PrivilegedOperation, PrivilegedRequest, SystemSettingsOperation},
  process::{ProcessRequest, ProcessRunner, SystemProcessRunner},
  sanitize::terminal_text,
};
use argvus_i18n::{Lang, tr};
use argvus_theme::Theme;
use argvus_tui::buttons::{Button, ButtonKind};
use argvus_tui::components::{
  ConfirmationDialog, ConfirmationOutcome, ConfirmationState, StatusKind, StatusMessage,
  draw_confirmation,
};
use argvus_tui::hints::{HintContext, hints};
use argvus_tui::menu::{MenuEvent, MenuState, MenuStyle, Row, draw_menu};
use argvus_tui::page::{list, readonly, shell, status};
use crossterm::event::KeyCode;
use ratatui::{
  Frame,
  layout::{Constraint, Layout, Rect},
  text::Line,
};

#[derive(Debug, Clone)]
/// Defines `Pending`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
enum Pending {
  Action(String, String),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Defines `ActionButton`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
enum ActionButton {
  PrevBoot,
  NextUnit,
  Priority,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Identifies the rows of the unit lists (System, User, Failed).
enum Item {
  Filter,
  Unit(usize),
  DetailAction(&'static str),
}
/// Represents `ServicesApp`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct ServicesApp {
  pub page: ServicePage,
  detail_parent: ServicePage,
  detail_unit: Option<String>,
  selected: usize,
  on_buttons: bool,
  button_selected: usize,
  button_from: Option<usize>,
  units: Vec<Unit>,
  logs: Vec<JournalEntry>,
  search: String,
  searching: bool,
  job: Option<JobHandle<JobData>>,
  updates: Option<std::sync::mpsc::Receiver<Vec<Unit>>>,
  requested_page: ServicePage,
  action: Option<JobHandle<String>>,
  pending: Option<Pending>,
  confirmation: ConfirmationState,
  manager: JobManager,
  pub lang: Lang,
  pub theme: Theme,
  pub status: Option<StatusMessage>,
  previous_boot: bool,
  priority: Option<String>,
  logs_unit: Option<String>,
  filter: UnitFilter,
  log_search: String,
  menu: MenuState,
}
/// Defines `JobData`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
enum JobData {
  Units(Vec<Unit>),
  Logs(Vec<JournalEntry>),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Defines `UnitFilter`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
enum UnitFilter {
  All,
  Running,
  Stopped,
  Failed,
  Enabled,
  Disabled,
}
impl UnitFilter {
  /// Executes the `next` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn next(self) -> Self {
    match self {
      Self::All => Self::Running,
      Self::Running => Self::Stopped,
      Self::Stopped => Self::Failed,
      Self::Failed => Self::Enabled,
      Self::Enabled => Self::Disabled,
      Self::Disabled => Self::All,
    }
  }
  /// Executes the `label` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn label(self, lang: Lang) -> &'static str {
    match self {
      Self::All => tr(lang, "control_center.all"),
      Self::Running => tr(lang, "control_center.running"),
      Self::Stopped => tr(lang, "control_center.stopped"),
      Self::Failed => tr(lang, "control_center.failed"),
      Self::Enabled => tr(lang, "control_center.enabled_a41f69"),
      Self::Disabled => tr(lang, "control_center.disabled_2301f9"),
    }
  }
}
impl ServicesApp {
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
      page: ServicePage::Home,
      detail_parent: ServicePage::System,
      detail_unit: None,
      selected: 0,
      on_buttons: false,
      button_selected: 0,
      button_from: None,
      units: vec![],
      logs: vec![],
      search: String::new(),
      searching: false,
      job: None,
      updates: None,
      requested_page: ServicePage::Home,
      action: None,
      pending: None,
      confirmation: ConfirmationState::default(),
      manager: JobManager::default(),
      lang,
      theme,
      status: None,
      previous_boot: false,
      priority: None,
      logs_unit: None,
      filter: UnitFilter::All,
      log_search: String::new(),
      menu: MenuState::default(),
    }
  }
  /// Executes the `refresh` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn refresh(&mut self) {
    if self.job.is_some() || self.action.is_some() {
      return;
    }
    let page = self.page;
    let previous = self.previous_boot;
    let priority = self.priority.clone();
    let logs_unit = self.logs_unit.clone();
    let detail_user = self.detail_parent == ServicePage::User;
    let (sender, receiver) = std::sync::mpsc::channel();
    self.updates = Some(receiver);
    self.requested_page = page;
    self.job = Some(self.manager.spawn(move |_| {
      match page {
        ServicePage::Home => {
          let system_sender = sender.clone();
          let user_sender = sender.clone();
          let (system, user) = rayon::join(
            || {
              backend::list_progressive(false, |units| {
                let _ = system_sender.send(units);
              })
            },
            || {
              backend::list_progressive(true, |units| {
                let _ = user_sender.send(units);
              })
            },
          );
          let mut system = system?;
          system.append(&mut user?);
          Ok(JobData::Units(system))
        }
        ServicePage::User => backend::list_progressive(true, |units| {
          let _ = sender.send(units);
        })
        .map(JobData::Units),
        ServicePage::Failed => {
          let system_sender = sender.clone();
          let user_sender = sender;
          let (system, user) = rayon::join(
            || {
              backend::list_progressive(false, |units| {
                let _ = system_sender.send(units);
              })
            },
            || {
              backend::list_progressive(true, |units| {
                let _ = user_sender.send(units);
              })
            },
          );
          let mut system = system?;
          let mut user = user?;
          system.append(&mut user);
          Ok(JobData::Units(system))
        }
        ServicePage::Logs | ServicePage::LogDetail(_) => {
          backend::logs(logs_unit.as_deref(), previous, priority.as_deref(), 200).map(JobData::Logs)
        }
        ServicePage::Detail => backend::list(detail_user).map(JobData::Units),
        _ => backend::list_progressive(false, |_| {}).map(JobData::Units),
      }
    }));
    self.status = Some(StatusMessage {
      kind: StatusKind::Info,
      text: tr(self.lang, "control_center.loading_services").into(),
    });
  }
  /// Executes the `poll` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn poll(&mut self) -> bool {
    if self.job.is_some() && self.updates.is_some() && self.requested_page != self.page {
      if let Some(job) = self.job.take() {
        job.cancel();
      }
      self.updates = None;
      self.refresh();
    }
    let mut changed = false;
    let updates = self
      .updates
      .as_ref()
      .map(|receiver| receiver.try_iter().collect::<Vec<_>>())
      .unwrap_or_default();
    for units in updates {
      self.accept_units(units);
      changed = true;
    }
    if let Some(job) = &self.job
      && let JobState::Finished(result) = job.try_state()
    {
      self.job = None;
      self.updates = None;
      match result {
        Ok(JobData::Units(v)) => {
          self.units = v;
          self.selected = self.selected.min(self.filtered().len().saturating_sub(1));
          self.status = None;
        }
        Ok(JobData::Logs(v)) => {
          self.logs = v;
          self.selected = self.selected.min(self.logs.len().saturating_sub(1));
          self.status = None;
        }
        Err(e) => {
          self.status = Some(StatusMessage {
            kind: StatusKind::Error,
            text: e,
          })
        }
      };
      changed = true
    }
    if let Some(job) = &self.action
      && let JobState::Finished(result) = job.try_state()
    {
      self.action = None;
      match result {
        Ok(v) => {
          self.status = Some(StatusMessage {
            kind: StatusKind::Success,
            text: v,
          })
        }
        Err(e) => {
          self.status = Some(StatusMessage {
            kind: StatusKind::Error,
            text: e,
          })
        }
      };
      self.refresh();
      changed = true
    }
    changed
  }

  fn accept_units(&mut self, incoming: Vec<Unit>) {
    for unit in incoming {
      if let Some(existing) = self
        .units
        .iter_mut()
        .find(|existing| existing.name == unit.name && existing.scope == unit.scope)
      {
        *existing = unit;
      } else {
        self.units.push(unit);
      }
    }
    self.units.sort_by(|left, right| {
      left
        .scope
        .cmp(&right.scope)
        .then_with(|| left.name.cmp(&right.name))
    });
    self.selected = self.selected.min(self.filtered().len().saturating_sub(1));
  }
  /// Whether typed characters currently go to the unit or log search, so
  /// `q`/`?` must not act as the global quit/help keys. Mirrors the
  /// precedence of [`Self::handle`]: running jobs and the confirmation sit
  /// above the search and do not take text.
  pub fn captures_text(&self) -> bool {
    self.searching && self.job.is_none() && self.action.is_none() && self.pending.is_none()
  }

  /// Processes `handle` in this module's event flow. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn handle(&mut self, key: KeyCode) -> bool {
    if self.pending.is_none()
      && !self.searching
      && self.action.is_none()
      && self.job.is_none()
      && self.on_buttons
      && !self.buttons().is_empty()
    {
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
    if matches!(key, KeyCode::Esc | KeyCode::Left) && self.pending.is_none() && !self.searching {
      if self.page == ServicePage::Home {
        return true;
      }
      if self.is_list_page() {
        // The filter is per visit: leaving a list clears it.
        self.filter = UnitFilter::All;
      }
      self.page = if self.page == ServicePage::Detail {
        self.detail_parent
      } else if matches!(self.page, ServicePage::LogDetail(_)) {
        ServicePage::Logs
      } else {
        ServicePage::Home
      };
      self.selected = 0;
      self.menu = MenuState::default();
      self.on_buttons = false;
      self.button_from = None;
      if self.page == ServicePage::Home {
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
          let Pending::Action(a, u) = self.pending.take().unwrap();
          let user = self.page == ServicePage::User
            || self.detail_parent == ServicePage::User
            || self
              .units
              .iter()
              .any(|unit| unit.name == u && unit.scope == "user");
          self.status = Some(StatusMessage {
            kind: StatusKind::Info,
            text: tr(self.lang, "control_center.applying_action").into(),
          });
          self.action = Some(self.manager.spawn(move |_| run_action(user, &a, &u)));
        }
        ConfirmationOutcome::Cancelled => {
          self.pending = None;
          self.confirmation = ConfirmationState::default();
        }
        ConfirmationOutcome::Pending => {}
      }
      return false;
    }
    if self.searching && self.is_list_page() && matches!(key, KeyCode::Up | KeyCode::Down) {
      // Arrows leave the search field and move the cursor of the list.
      self.searching = false;
    }
    if self.searching {
      let logs = self.page == ServicePage::Logs;
      match key {
        KeyCode::Esc => self.searching = false,
        KeyCode::Backspace => {
          if logs {
            self.log_search.pop();
          } else {
            self.search.pop();
          }
          self.selected = 0
        }
        KeyCode::Enter => self.searching = false,
        KeyCode::Char(c) => {
          if logs {
            self.log_search.push(c);
          } else {
            self.search.push(c);
          }
        }
        KeyCode::Down => self.selected += 1,
        KeyCode::Up => self.selected = self.selected.saturating_sub(1),
        _ => {}
      }
      let count = if logs {
        self.filtered_logs().len()
      } else {
        self.filtered().len()
      };
      self.selected = self.selected.min(count.saturating_sub(1));
      return false;
    }
    if self.is_list_page() || self.page == ServicePage::Detail {
      match key {
        KeyCode::Char('r') => self.refresh(),
        KeyCode::Char('/') if self.is_list_page() => self.searching = true,
        _ => {
          let rows = self.menu_rows();
          match self.menu.handle(key, &rows, 8) {
            MenuEvent::Activate(Item::Filter) => self.cycle_filter(),
            MenuEvent::Activate(Item::Unit(index)) => {
              self.selected = index;
              self.open();
            }
            MenuEvent::Activate(Item::DetailAction(action)) => self.run_detail(action),
            _ => {}
          }
        }
      }
      return false;
    }
    match key {
      KeyCode::Tab | KeyCode::BackTab => self.toggle_buttons(key == KeyCode::BackTab),
      KeyCode::Char('r') => self.refresh(),
      KeyCode::Char('/')
        if matches!(
          self.page,
          ServicePage::System | ServicePage::User | ServicePage::Failed | ServicePage::Logs
        ) =>
      {
        self.searching = true
      }
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
    let count = match self.page {
      ServicePage::Home => 4,
      ServicePage::System | ServicePage::User | ServicePage::Failed => self.filtered().len(),
      ServicePage::Logs => self.filtered_logs().len(),
      ServicePage::Detail => self.detail_actions().len(),
      _ => 1,
    };
    self.selected = self.selected.min(count.saturating_sub(1));
    false
  }
  /// Executes the `selection_len` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn selection_len(&self) -> usize {
    match self.page {
      ServicePage::Home => 4,
      ServicePage::System | ServicePage::User | ServicePage::Failed => self.filtered().len(),
      ServicePage::Logs => self.logs.len(),
      ServicePage::Detail => self.detail_actions().len(),
      ServicePage::LogDetail(_) => 1,
    }
  }
  /// Executes the `open` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn open(&mut self) {
    match self.page {
      ServicePage::Home => {
        self.page = [
          ServicePage::System,
          ServicePage::User,
          ServicePage::Failed,
          ServicePage::Logs,
        ][self.selected.min(3)];
        self.selected = 0;
        self.menu = MenuState::default();
        self.refresh()
      }
      ServicePage::System | ServicePage::User | ServicePage::Failed => {
        if self.filtered().get(self.selected).is_some() {
          self.detail_parent = self.page;
          self.detail_unit = self
            .filtered()
            .get(self.selected)
            .map(|unit| unit.name.clone());
          self.page = ServicePage::Detail
        }
      }
      ServicePage::Detail => {
        if let Some((action, _)) = self.detail_actions().get(self.selected) {
          let action = *action;
          self.run_detail(action);
        }
      }
      ServicePage::Logs => {
        if self.filtered_logs().get(self.selected).is_some() {
          self.page = ServicePage::LogDetail(self.selected);
        }
      }
      ServicePage::LogDetail(_) => {}
    }
  }
  /// Runs one row of the service details: `logs` opens the journal, any
  /// other action asks for confirmation first.
  fn run_detail(&mut self, action: &'static str) {
    if action == "logs" {
      self.open_logs();
    } else {
      self.request(action);
    }
  }
  /// Executes the `request` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn request(&mut self, a: &str) {
    if matches!(
      self.page,
      ServicePage::System | ServicePage::User | ServicePage::Failed | ServicePage::Detail
    ) && let Some(u) = self.detail_or_selected_unit()
      && can_action(a, u)
    {
      self.pending = Some(Pending::Action(a.into(), u.name.clone()))
    }
  }
  /// Executes the `open_logs` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn open_logs(&mut self) {
    let unit = self
      .detail_unit
      .clone()
      .or_else(|| self.filtered().get(self.selected).map(|u| u.name.clone()));
    if let Some(unit) = unit {
      self.page = ServicePage::Logs;
      self.logs_unit = Some(unit.clone());
      self.job = Some(
        self
          .manager
          .spawn(move |_| backend::logs(Some(&unit), false, None, 200).map(JobData::Logs)),
      );
      self.selected = 0
    }
  }
  /// Executes the `filtered` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn filtered(&self) -> Vec<&Unit> {
    self
      .units
      .iter()
      .filter(|u| self.page != ServicePage::Failed || u.failed())
      .filter(|u| match self.filter {
        UnitFilter::All => true,
        UnitFilter::Running => u.active == "active",
        UnitFilter::Stopped => u.active == "inactive",
        UnitFilter::Failed => u.failed(),
        UnitFilter::Enabled => matches!(u.file_state.as_str(), "enabled" | "static" | "indirect"),
        UnitFilter::Disabled => matches!(u.file_state.as_str(), "disabled" | "masked"),
      })
      .filter(|u| {
        self.search.is_empty()
          || u
            .name
            .to_ascii_lowercase()
            .contains(&self.search.to_ascii_lowercase())
          || u
            .description
            .to_ascii_lowercase()
            .contains(&self.search.to_ascii_lowercase())
      })
      .collect()
  }
  /// Executes the `filtered_logs` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn filtered_logs(&self) -> Vec<&JournalEntry> {
    let query = self.log_search.to_ascii_lowercase();
    self
      .logs
      .iter()
      .filter(|entry| {
        query.is_empty()
          || entry
            .message
            .as_deref()
            .unwrap_or_default()
            .to_ascii_lowercase()
            .contains(&query)
          || entry
            .unit
            .as_deref()
            .unwrap_or_default()
            .to_ascii_lowercase()
            .contains(&query)
      })
      .collect()
  }
  /// Executes the `cycle_log_unit` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn cycle_log_unit(&mut self) {
    let mut units: Vec<String> = self
      .logs
      .iter()
      .filter_map(|entry| entry.unit.clone())
      .filter(|unit| !unit.is_empty())
      .collect();
    units.sort();
    units.dedup();
    if units.is_empty() {
      return;
    }
    self.logs_unit = match self.logs_unit.as_deref() {
      None => units.first().cloned(),
      Some(current) => units
        .iter()
        .position(|unit| unit == current)
        .and_then(|index| units.get(index + 1).cloned()),
    };
    self.selected = 0;
    self.refresh();
  }
  /// Executes the `breadcrumb` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn breadcrumb(&self) -> String {
    let root = format!(
      "{} > {}",
      HomeCardId::System.label(self.lang),
      tr(self.lang, "control_center.services")
    );
    if self.page == ServicePage::Home {
      root
    } else {
      format!("{root} > {}", self.label(self.page))
    }
  }
  /// Executes the `label` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn label(&self, p: ServicePage) -> &'static str {
    match p {
      ServicePage::System => tr(self.lang, "control_center.system"),
      ServicePage::User => tr(self.lang, "control_center.user"),
      ServicePage::Failed => tr(self.lang, "control_center.failed"),
      ServicePage::Logs => tr(self.lang, "control_center.logs"),
      ServicePage::Detail => tr(self.lang, "control_center.details"),
      ServicePage::LogDetail(_) => tr(self.lang, "control_center.logs_details"),
      ServicePage::Home => tr(self.lang, "control_center.services"),
    }
  }
  /// Executes the `cycle_filter` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn cycle_filter(&mut self) {
    self.filter = self.filter.next();
    self.selected = 0;
    self.menu = MenuState::default();
  }
  /// Executes the `cycle_priority` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn cycle_priority(&mut self) {
    self.priority = match self.priority.as_deref() {
      None | Some("7") => None,
      Some(value) => Some((value.parse::<u8>().unwrap_or(0) + 1).to_string()),
    };
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
    let Some((action, _)) = actions.get(self.button_selected) else {
      return;
    };
    match *action {
      ActionButton::PrevBoot => {
        self.previous_boot = !self.previous_boot;
        self.refresh();
      }
      ActionButton::NextUnit => self.cycle_log_unit(),
      ActionButton::Priority => self.cycle_priority(),
    }
  }
  /// Executes the `buttons` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn buttons(&self) -> Vec<(ActionButton, Button)> {
    let secondary = ButtonKind::Secondary;
    match self.page {
      ServicePage::Logs => vec![
        (
          ActionButton::PrevBoot,
          Button::new(
            format!(
              "{}: {}",
              tr(self.lang, "control_center.boot"),
              if self.previous_boot {
                tr(self.lang, "control_center.previous")
              } else {
                tr(self.lang, "control_center.current_c42247")
              }
            ),
            secondary,
          ),
        ),
        (
          ActionButton::NextUnit,
          Button::new(
            format!(
              "{}: {}",
              tr(self.lang, "control_center.service"),
              self.logs_unit.as_deref().unwrap_or("—")
            ),
            secondary,
          ),
        ),
        (
          ActionButton::Priority,
          Button::new(
            format!(
              "{}: {}",
              tr(self.lang, "control_center.priority"),
              self.priority.as_deref().unwrap_or("0-7")
            ),
            secondary,
          ),
        ),
      ],
      ServicePage::System
      | ServicePage::User
      | ServicePage::Failed
      | ServicePage::Home
      | ServicePage::Detail
      | ServicePage::LogDetail(_) => Vec::new(),
    }
  }
  /// Whether the page is one of the unit lists (System, User, Failed).
  fn is_list_page(&self) -> bool {
    matches!(
      self.page,
      ServicePage::System | ServicePage::User | ServicePage::Failed
    )
  }
  /// Builds the rows of a unit list: the filter, then the units with their
  /// Active and Enabled columns.
  fn list_rows(&self) -> Vec<Row<Item>> {
    let mut rows = vec![
      Row::value(
        Item::Filter,
        tr(self.lang, "control_center.filter"),
        self.filter.label(self.lang),
        None,
      ),
      Row::section(tr(self.lang, "control_center.name")).detail(format!(
        "  {:<COLUMN$}  {}",
        tr(self.lang, "control_center.active_095d39"),
        tr(self.lang, "control_center.enabled_78438d"),
        COLUMN = STATUS_COLUMN,
      )),
    ];
    if self.page == ServicePage::Failed && self.filtered().is_empty() {
      rows.push(Row::info(
        format!("[OK] {}", tr(self.lang, "control_center.no_failed_units")),
        "",
      ));
      return rows;
    }
    rows.extend(self.filtered().iter().enumerate().map(|(index, unit)| {
      let mut label = unit.name.clone();
      if self.page == ServicePage::Failed && unit.scope == "user" {
        label.push_str(&format!(
          "   [{}]",
          tr(self.lang, "control_center.user_e7acba")
        ));
      }
      Row::submenu(Item::Unit(index), label).detail(self.status_cells(unit))
    }));
    rows
  }
  /// The Active and Enabled cells of a unit. A filled dot means the state is
  /// on, an empty dot means it is off.
  fn status_cells(&self, unit: &Unit) -> String {
    let active = unit.active == "active";
    let enabled = matches!(unit.file_state.as_str(), "enabled" | "static" | "indirect");
    format!(
      "{} {:<COLUMN$}{} {}",
      state_dot(active),
      tr(self.lang, "control_center.active_095d39"),
      state_dot(enabled),
      tr(self.lang, "control_center.enabled_78438d"),
      COLUMN = STATUS_COLUMN,
    )
  }
  /// Executes the `footer_hints` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn footer_hints(&self) -> String {
    match self.page {
      ServicePage::LogDetail(_) => tr(self.lang, "control_center.scroll_esc_back_help").into(),
      ServicePage::Home => tr(
        self.lang,
        "control_center.navigate_enter_open_r_refresh_esc_back_help",
      )
      .into(),
      ServicePage::System | ServicePage::User | ServicePage::Failed => {
        let mut menu = self.menu;
        let rows = self.list_rows();
        menu.normalize(&rows);
        hints(
          self.lang,
          &HintContext {
            row: menu.selected_kind(&rows),
            can_go_back: true,
            search: true,
            refresh: true,
            ..HintContext::default()
          },
        )
      }
      ServicePage::Detail => {
        let mut menu = self.menu;
        let rows = self.detail_rows();
        menu.normalize(&rows);
        hints(
          self.lang,
          &HintContext {
            row: menu.selected_kind(&rows),
            can_go_back: true,
            refresh: true,
            ..HintContext::default()
          },
        )
      }
      _ => tr(
        self.lang,
        "control_center.navigate_tab_actions_enter_activate_move_search_r_refresh_esc_back_hel",
      )
      .into(),
    }
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
      &self.footer_hints(),
    );
    if self.is_list_page() {
      self.draw_list(frame, area, body);
      return;
    }
    if self.page == ServicePage::Detail {
      self.draw_detail(frame, area, body);
      return;
    }
    let mut lines: Vec<String> = if self.page == ServicePage::Home {
      self.home_rows()
    } else if self.page == ServicePage::Logs {
      self
        .filtered_logs()
        .iter()
        .map(|entry| log_line(entry))
        .map(|l| l.to_string())
        .collect()
    } else if let ServicePage::LogDetail(index) = self.page {
      self
        .log_detail_lines(index)
        .into_iter()
        .map(|l| l.to_string())
        .collect()
    } else {
      Vec::new()
    };
    if self.page == ServicePage::Logs {
      lines.insert(0, self.logs_header());
    }
    if self.searching {
      lines.insert(
        0,
        format!(
          "{}: {}_",
          tr(self.lang, "control_center.search"),
          self.search
        ),
      )
    }
    let visual_offset = usize::from(self.page == ServicePage::Logs) + usize::from(self.searching);
    let buttons = self.buttons();
    let raw_buttons: Vec<Button> = buttons.iter().map(|(_, button)| button.clone()).collect();
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
      self
        .selected
        .saturating_add(visual_offset)
        .min(lines.len().saturating_sub(1))
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
      status(frame, area, &self.theme, s)
    }
    if self.job.is_some() {
      status(
        frame,
        area,
        &self.theme,
        &StatusMessage {
          kind: StatusKind::Info,
          text: tr(self.lang, "control_center.loading_services").into(),
        },
      );
    }
    if let Some(Pending::Action(a, u)) = &self.pending {
      self.draw_pending(frame, area, a, u)
    }
  }
  /// Renders a unit list: the search field stays visible on top, then the
  /// menu with the filter and the units.
  fn draw_list(&mut self, frame: &mut Frame, area: Rect, body: Rect) {
    let rows = self.list_rows();
    self.menu.normalize(&rows);
    let sections = Layout::vertical([Constraint::Length(1), Constraint::Min(1)]).split(body);
    let search = format!(
      "{}: {}{}",
      tr(self.lang, "control_center.search"),
      self.search,
      if self.searching { "_" } else { "" },
    );
    readonly(frame, sections[0], &self.theme, &[Line::from(search)]);
    draw_menu(
      frame,
      sections[1],
      &self.theme,
      &rows,
      &mut self.menu,
      MenuStyle {
        icons: AppConfig::icons_enabled(),
      },
    );
    self.draw_overlays(frame, area);
  }
  /// Renders the details page: the service information and its actions.
  fn draw_detail(&mut self, frame: &mut Frame, area: Rect, body: Rect) {
    let rows = self.detail_rows();
    self.menu.normalize(&rows);
    draw_menu(
      frame,
      body,
      &self.theme,
      &rows,
      &mut self.menu,
      MenuStyle {
        icons: AppConfig::icons_enabled(),
      },
    );
    self.draw_overlays(frame, area);
  }
  /// Status line, loading indicator and the confirmation, shared by the
  /// menu pages.
  fn draw_overlays(&self, frame: &mut Frame, area: Rect) {
    if let Some(s) = &self.status {
      status(frame, area, &self.theme, s)
    }
    if self.job.is_some() {
      status(
        frame,
        area,
        &self.theme,
        &StatusMessage {
          kind: StatusKind::Info,
          text: tr(self.lang, "control_center.loading_services").into(),
        },
      );
    }
    if let Some(Pending::Action(a, u)) = &self.pending {
      self.draw_pending(frame, area, a, u)
    }
  }
  /// Renders `draw_pending` while respecting the current domain state and semantic theme. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn draw_pending(&self, frame: &mut Frame, area: ratatui::layout::Rect, action: &str, unit: &str) {
    draw_confirmation(
      frame,
      area,
      &self.theme,
      ConfirmationDialog {
        title: tr(self.lang, "control_center.confirm_action"),
        message: &format!("{} {}?", action_label(self.lang, action), unit),
        confirm_label: tr(self.lang, "control_center.continue"),
        cancel_label: tr(self.lang, "control_center.cancel"),
        confirm_selected: self.confirmation.confirm_selected,
      },
    )
  }
  /// Executes the `home_rows` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn home_rows(&self) -> Vec<String> {
    let (system_active, system_enabled) = self.counts("system");
    let (user_active, user_enabled) = self.counts("user");
    let failed = self.units.iter().filter(|unit| unit.failed()).count();
    let boot = if self.previous_boot {
      tr(self.lang, "control_center.previous")
    } else {
      tr(self.lang, "control_center.current_c42247")
    };
    vec![
      format!(
        "{} {}  ·  {} {} · {} {}",
        AppConfig::icon(argvus_tui::icons::SETTINGS),
        tr(self.lang, "control_center.system"),
        system_active,
        tr(self.lang, "control_center.active_ae7190"),
        system_enabled,
        tr(self.lang, "control_center.enabled_26a3ab"),
      ),
      format!(
        "{} {}  ·  {} {} · {} {}",
        AppConfig::icon(argvus_tui::icons::USER),
        tr(self.lang, "control_center.user"),
        user_active,
        tr(self.lang, "control_center.active_ae7190"),
        user_enabled,
        tr(self.lang, "control_center.enabled_26a3ab"),
      ),
      format!(
        "{} {}  ·  {} {}",
        AppConfig::icon(argvus_tui::icons::WARNING),
        tr(self.lang, "control_center.failed"),
        failed,
        tr(self.lang, "control_center.failed_cc0486"),
      ),
      format!(
        "{} {}  ·  {} {} · {} {}",
        AppConfig::icon(argvus_tui::icons::LOGS),
        tr(self.lang, "control_center.logs"),
        tr(self.lang, "control_center.boot"),
        boot,
        tr(self.lang, "control_center.priority"),
        self.priority.as_deref().unwrap_or("0-7"),
      ),
    ]
  }
  /// Executes the `counts` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn counts(&self, scope: &str) -> (usize, usize) {
    let active = self
      .units
      .iter()
      .filter(|unit| unit.scope == scope && unit.active == "active")
      .count();
    let enabled = self
      .units
      .iter()
      .filter(|unit| {
        unit.scope == scope && matches!(unit.file_state.as_str(), "enabled" | "static" | "indirect")
      })
      .count();
    (active, enabled)
  }
  /// Executes the `logs_header` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn logs_header(&self) -> String {
    let boot = if self.previous_boot {
      tr(self.lang, "control_center.previous")
    } else {
      tr(self.lang, "control_center.current_c42247")
    };
    format!(
      "{} {}  ·  {} {} · {} {} · {} {}",
      AppConfig::icon(argvus_tui::icons::LOGS),
      tr(self.lang, "control_center.logs"),
      tr(self.lang, "control_center.boot"),
      boot,
      tr(self.lang, "control_center.priority"),
      self.priority.as_deref().unwrap_or("0-7"),
      tr(self.lang, "control_center.service"),
      self.logs_unit.as_deref().unwrap_or("—"),
    )
  }
  /// Executes the `detail_lines` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  /// Rows of the service details: the service information, then its
  /// actions. Info rows are read-only; actions reuse `detail_actions`.
  fn detail_rows(&self) -> Vec<Row<Item>> {
    let mut rows = vec![Row::section(tr(self.lang, "control_center.service_bfe08e"))];
    let Some(u) = self.detail_or_selected_unit() else {
      rows.push(Row::info(
        tr(self.lang, "control_center.service_not_found"),
        "",
      ));
      return rows;
    };
    rows.extend([
      Row::info(tr(self.lang, "control_center.name"), u.name.clone()),
      Row::info(
        tr(self.lang, "control_center.description"),
        u.description.clone(),
      ),
      Row::info(tr(self.lang, "control_center.state"), state_value(u)),
      Row::info(
        tr(self.lang, "control_center.boot_f0640d"),
        file_state_label(self.lang, &u.file_state),
      ),
      Row::info(
        tr(self.lang, "control_center.main_pid"),
        u.main_pid
          .map(|v| v.to_string())
          .unwrap_or_else(|| "—".into()),
      ),
      Row::info(
        tr(self.lang, "control_center.unit_file"),
        u.fragment.clone().unwrap_or_else(|| "—".into()),
      ),
      Row::section(tr(self.lang, "control_center.section_actions")),
    ]);
    rows.extend(
      self
        .detail_actions()
        .into_iter()
        .map(|(action, label)| Row::action(Item::DetailAction(action), label)),
    );
    rows
  }
  /// The rows of the page that is currently a menu (lists and details).
  fn menu_rows(&self) -> Vec<Row<Item>> {
    if self.page == ServicePage::Detail {
      self.detail_rows()
    } else {
      self.list_rows()
    }
  }
  /// Executes the `detail_actions` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn detail_actions(&self) -> Vec<(&'static str, String)> {
    let Some(unit) = self.detail_or_selected_unit() else {
      return Vec::new();
    };
    let mut actions = [
      "start",
      "stop",
      "restart",
      "enable",
      "disable",
      "enable-now",
      "disable-now",
    ]
    .into_iter()
    .filter(|action| can_action(action, unit))
    .map(|action| (action, action_label(self.lang, action)))
    .collect::<Vec<_>>();
    actions.push(("logs", tr(self.lang, "control_center.view_logs").into()));
    actions
  }
  /// Executes the `detail_or_selected_unit` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn detail_or_selected_unit(&self) -> Option<&Unit> {
    self
      .detail_unit
      .as_deref()
      .and_then(|name| self.units.iter().find(|unit| unit.name == name))
      .or_else(|| self.filtered().get(self.selected).copied())
  }
  /// Executes the `log_detail_lines` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn log_detail_lines(&self, index: usize) -> Vec<Line<'static>> {
    let filtered = self.filtered_logs();
    let Some(entry) = filtered.get(index) else {
      return vec![Line::from(tr(self.lang, "control_center.log_not_found"))];
    };
    let timestamp = entry.timestamp.clone().unwrap_or_else(|| "—".into());
    let unit = entry.unit.clone().unwrap_or_else(|| "kernel".into());
    let priority = entry.priority_name().to_string();
    let pid = entry.pid.clone().unwrap_or_else(|| "—".into());
    let boot = entry.boot_id.clone().unwrap_or_else(|| "—".into());
    let executable = entry.executable.clone().unwrap_or_else(|| "—".into());
    let message = entry.message.clone().unwrap_or_default();
    vec![
      Line::from(format!(
        "{}: {}",
        tr(self.lang, "control_center.timestamp"),
        timestamp
      )),
      Line::from(format!(
        "{}: {}",
        tr(self.lang, "control_center.unit"),
        unit
      )),
      Line::from(format!(
        "{}: {}",
        tr(self.lang, "control_center.priority"),
        priority
      )),
      Line::from(format!("PID: {}", pid)),
      Line::from(format!(
        "{}: {}",
        tr(self.lang, "control_center.executable"),
        executable
      )),
      Line::from(format!(
        "{}: {}",
        tr(self.lang, "control_center.boot"),
        boot
      )),
      Line::from(""),
      Line::from(message),
    ]
  }
}
/// Width of the Active column in the unit lists; Enabled starts after it.
const STATUS_COLUMN: usize = 12;
/// A filled dot when the state is on, an empty dot when it is off.
fn state_dot(on: bool) -> &'static str {
  if on { "●" } else { "○" }
}
/// Checks the condition represented by `can_action` using only the state available to the module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn can_action(action: &str, unit: &Unit) -> bool {
  match action {
    "start" => !matches!(unit.active.as_str(), "active" | "activating"),
    "stop" => matches!(unit.active.as_str(), "active" | "activating"),
    "restart" => matches!(unit.active.as_str(), "active" | "activating"),
    "enable" | "enable-now" => {
      !matches!(unit.file_state.as_str(), "enabled" | "static" | "indirect")
    }
    "disable" | "disable-now" => matches!(unit.file_state.as_str(), "enabled" | "indirect"),
    _ => false,
  }
}
/// Executes the `action_label` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn action_label(lang: Lang, action: &str) -> String {
  match action {
    "start" => tr(lang, "control_center.start"),
    "stop" => tr(lang, "control_center.stop"),
    "restart" => tr(lang, "control_center.restart"),
    "enable" => tr(lang, "control_center.enable"),
    "disable" => tr(lang, "control_center.disable"),
    "enable-now" => tr(lang, "control_center.enable_and_start"),
    "disable-now" => tr(lang, "control_center.disable_and_stop"),
    other => other,
  }
  .into()
}
/// Executes the `state_value` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn state_value(u: &Unit) -> String {
  format!("{} / {}", u.active, u.sub)
}
/// Executes the `file_state_label` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn file_state_label(lang: Lang, state: &str) -> String {
  match state {
    "enabled" | "static" | "indirect" => tr(lang, "control_center.enabled_78438d").into(),
    "disabled" | "masked" => tr(lang, "control_center.disabled_483392").into(),
    _ => state.into(),
  }
}
/// Executes the `log_line` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn log_line(e: &JournalEntry) -> Line<'static> {
  Line::from(format!(
    "{:8} {:24} {:7} {}",
    e.timestamp.as_deref().unwrap_or(""),
    e.unit.as_deref().unwrap_or("kernel"),
    e.priority_name(),
    e.message.as_deref().unwrap_or("")
  ))
}
/// Executes the `run_action` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn run_action(user: bool, action: &str, unit: &str) -> Result<String, String> {
  let args = backend::action_args(action, unit)?;
  if !user {
    let executable = std::env::current_exe()
      .map_err(|error| error.to_string())?
      .to_string_lossy()
      .into_owned();
    let operation = SystemSettingsOperation::new(SystemProcessRunner, executable);
    let request = PrivilegedRequest::new("service", action, vec![unit.into()])?;
    let output = operation.execute(&request)?;
    return if output.status == Some(0) {
      Ok(format!("{} {}", action, unit))
    } else {
      Err(privileged_failure(&output))
    };
  }
  let mut r = ProcessRequest::new("systemctl");
  if user {
    r = r.arg("--user");
  }
  for a in &args {
    r = r.arg(a)
  }
  let out = SystemProcessRunner.run(&r).map_err(|e| e.to_string())?;
  if out.status.is_some_and(|s| s == 0) {
    Ok(format!("{} {}", action, unit))
  } else {
    Err(terminal_text(String::from_utf8_lossy(&out.stderr).trim()))
  }
}
/// Executes the `privileged_failure` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn privileged_failure(output: &argvus_control_center_core::process::ProcessOutput) -> String {
  let stderr = terminal_text(String::from_utf8_lossy(&output.stderr).trim());
  match output.status {
    Some(126) => "Authorization was cancelled.".into(),
    Some(127) => "Authorization was denied.".into(),
    _ if stderr.contains("pkexec was not found") => "pkexec was not found; install polkit.".into(),
    _ if stderr.is_empty() => "The privileged service operation failed.".into(),
    _ => stderr,
  }
}
#[cfg(test)]
mod tests {
  use super::*;
  use argvus_tui::menu::RowKind;
  #[test]
  /// Executes the `failed_filter_is_explicit` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn failed_filter_is_explicit() {
    let mut a = ServicesApp::new(Lang::for_locale("en-US"), Theme::load());
    a.page = ServicePage::Failed;
    a.units = vec![
      Unit {
        name: "a.service".into(),
        active: "failed".into(),
        ..Default::default()
      },
      Unit {
        name: "b.service".into(),
        active: "active".into(),
        ..Default::default()
      },
    ];
    assert_eq!(a.filtered().len(), 1);
  }

  #[test]
  /// Executes the `actions_are_structured_and_user_actions_are_not_elevated` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn actions_are_structured_and_user_actions_are_not_elevated() {
    assert_eq!(
      backend::action_args("enable-now", "NetworkManager.service").unwrap(),
      ["enable", "--now", "NetworkManager.service"]
    );
    assert!(backend::action_args("restart", "bad unit.service").is_err());
  }

  #[test]
  /// Executes the `service_list_filter_and_selection_stay_bounded` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn service_list_filter_and_selection_stay_bounded() {
    let mut app = ServicesApp::new(Lang::for_locale("en-US"), Theme::load());
    app.page = ServicePage::System;
    app.units = vec![Unit {
      name: "demo.service".into(),
      description: "Demo".into(),
      active: "active".into(),
      file_state: "enabled".into(),
      ..Default::default()
    }];
    app.handle(KeyCode::Down);
    assert_eq!(app.selected, 0);
    app.handle(KeyCode::Tab);
    assert!(!app.on_buttons);
    app.handle(KeyCode::Up);
    assert_eq!(app.menu.selected_id(&app.list_rows()), Some(Item::Filter));
    app.handle(KeyCode::Enter);
    assert_eq!(app.filter, UnitFilter::Running);
    assert_eq!(app.filtered().len(), 1);
    app.handle(KeyCode::Char('/'));
    app.handle(KeyCode::Char('z'));
    assert!(app.filtered().is_empty());
    assert_eq!(app.selected, 0);
  }

  #[test]
  /// Executes the `service_details_expose_actions_and_confirmation_defaults_to_cancel` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn service_details_expose_actions_and_confirmation_defaults_to_cancel() {
    let mut app = ServicesApp::new(Lang::for_locale("en-US"), Theme::load());
    app.page = ServicePage::Detail;
    app.detail_parent = ServicePage::System;
    app.detail_unit = Some("demo.service".into());
    app.units = vec![Unit {
      name: "demo.service".into(),
      active: "active".into(),
      file_state: "enabled".into(),
      ..Default::default()
    }];
    assert!(
      app
        .detail_actions()
        .iter()
        .any(|(action, _)| *action == "restart")
    );
    app.request("restart");
    app.handle(KeyCode::Enter);
    assert!(app.pending.is_none());
    assert!(app.action.is_none());
    app.request("restart");
    app.handle(KeyCode::Tab);
    assert!(app.confirmation.confirm_selected);
  }

  #[test]
  /// Executes the `polkit_exit_codes_are_not_collapsed_into_a_generic_error` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn polkit_exit_codes_are_not_collapsed_into_a_generic_error() {
    let output = argvus_control_center_core::process::ProcessOutput {
      stdout: Vec::new(),
      stderr: Vec::new(),
      status: Some(126),
      timed_out: false,
    };
    assert!(privileged_failure(&output).contains("cancelled"));
  }

  #[test]
  /// Executes the `services_expose_action_buttons_on_lists_but_not_home_or_detail` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn services_expose_action_buttons_on_lists_but_not_home_or_detail() {
    let home = ServicesApp::new(Lang::for_locale("en-US"), Theme::load());
    assert_eq!(home.buttons().len(), 0);
    let mut system = ServicesApp::new(Lang::for_locale("en-US"), Theme::load());
    system.page = ServicePage::System;
    assert_eq!(
      system.buttons().len(),
      0,
      "lists use menu rows, not buttons"
    );
    let mut logs = ServicesApp::new(Lang::for_locale("en-US"), Theme::load());
    logs.page = ServicePage::Logs;
    assert_eq!(logs.buttons().len(), 3);
  }

  #[test]
  /// Executes the `services_render_menu_rows_with_status_columns_on_lists` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn services_render_menu_rows_with_status_columns_on_lists() {
    let mut app = ServicesApp::new(Lang::for_locale("en-US"), Theme::load());
    app.page = ServicePage::System;
    let mut terminal = ratatui::Terminal::new(ratatui::backend::TestBackend::new(90, 25)).unwrap();
    terminal.draw(|frame| app.draw(frame)).unwrap();
    let text = terminal
      .backend()
      .buffer()
      .content
      .iter()
      .map(|cell| cell.symbol())
      .collect::<String>();
    assert!(text.contains("Active") && text.contains("Enabled"));
    assert!(!text.contains("Start") && !text.contains("Iniciar"));
  }

  #[test]
  /// Executes the `services_home_rows_act_as_a_status_dashboard` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn services_home_rows_act_as_a_status_dashboard() {
    let mut app = ServicesApp::new(Lang::for_locale("en-US"), Theme::load());
    app.units = vec![
      Unit {
        name: "a.service".into(),
        active: "active".into(),
        file_state: "enabled".into(),
        scope: "system".into(),
        ..Default::default()
      },
      Unit {
        name: "b.service".into(),
        active: "inactive".into(),
        file_state: "disabled".into(),
        scope: "system".into(),
        ..Default::default()
      },
      Unit {
        name: "c.service".into(),
        active: "failed".into(),
        file_state: "enabled".into(),
        scope: "user".into(),
        ..Default::default()
      },
    ];
    let rows = app.home_rows();
    assert_eq!(rows.len(), 4);
    assert!(rows[0].contains("System") && rows[0].contains("1 active"));
    assert!(rows[1].contains("User") && rows[1].contains("0 active"));
    assert!(rows[2].contains("Failed") && rows[2].contains("1 failed"));
    assert!(rows[3].contains("Logs"));
  }

  #[test]
  /// Executes the `services_list_rows_render_status_columns` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn services_list_rows_render_status_columns() {
    let mut app = ServicesApp::new(Lang::for_locale("en-US"), Theme::load());
    app.page = ServicePage::System;
    app.units = vec![
      Unit {
        name: "NetworkManager.service".into(),
        active: "active".into(),
        file_state: "enabled".into(),
        ..Default::default()
      },
      Unit {
        name: "cups.service".into(),
        active: "inactive".into(),
        file_state: "disabled".into(),
        ..Default::default()
      },
      Unit {
        name: "sshd.service".into(),
        active: "failed".into(),
        file_state: "enabled".into(),
        ..Default::default()
      },
    ];
    let rows = app.list_rows();
    // Filter, column header, then one row per unit.
    assert_eq!(rows.len(), 5);
    assert_eq!(rows[2].label(), "NetworkManager.service");
    assert_eq!(rows[2].detail_text(), Some("● Active      ● Enabled"));
    assert_eq!(rows[3].detail_text(), Some("○ Active      ○ Enabled"));
    assert_eq!(rows[4].detail_text(), Some("○ Active      ● Enabled"));
  }

  #[test]
  /// Executes the `services_detail_lines_use_section_header_and_aligned_rows` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn services_detail_rows_show_info_then_actions() {
    let mut app = ServicesApp::new(Lang::for_locale("en-US"), Theme::load());
    app.page = ServicePage::Detail;
    app.detail_parent = ServicePage::System;
    app.detail_unit = Some("demo.service".into());
    app.units = vec![Unit {
      name: "demo.service".into(),
      description: "Demo service".into(),
      active: "active".into(),
      sub: "running".into(),
      file_state: "enabled".into(),
      main_pid: Some(1234),
      fragment: Some("/usr/lib/systemd/system/demo.service".into()),
      ..Default::default()
    }];
    let rows = app.detail_rows();
    let text = rows
      .iter()
      .map(|row| format!("{} {}", row.label(), row.detail_text().unwrap_or_default()))
      .collect::<Vec<_>>()
      .join("\n");
    assert!(text.contains("demo.service"));
    assert!(text.contains("1234"));
    assert!(text.contains("Enabled"));
    assert!(text.contains("Unit file"));
    // Info rows are read-only; the actions follow them as menu rows.
    assert!(rows.iter().any(|row| row.is_section()));
    assert!(rows.iter().any(|row| row.kind() == RowKind::Action));
  }

  #[test]
  /// Leaving a unit list clears the filter, so each visit starts on All.
  fn leaving_a_list_clears_the_filter() {
    let mut app = ServicesApp::new(Lang::for_locale("en-US"), Theme::load());
    app.page = ServicePage::Failed;
    app.filter = UnitFilter::Enabled;
    app.handle(KeyCode::Esc);
    assert_eq!(app.page, ServicePage::Home);
    assert_eq!(app.filter, UnitFilter::All);
  }

  #[test]
  /// Retrieves data for `loading_status_is_cleared_once_units_arrive` without mixing collection with TUI rendering. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn loading_status_is_cleared_once_units_arrive() {
    let mut app = ServicesApp::new(Lang::for_locale("en-US"), Theme::load());
    app.page = ServicePage::System;
    app.status = Some(StatusMessage {
      kind: StatusKind::Info,
      text: tr(app.lang, "control_center.loading_services").into(),
    });
    app.job = Some(
      app
        .manager
        .spawn(|_| Ok(JobData::Units(vec![Unit::default()]))),
    );
    for _ in 0..2000 {
      if app.job.is_none() {
        break;
      }
      app.poll();
      std::thread::sleep(std::time::Duration::from_millis(1));
    }
    assert!(app.job.is_none());
    assert!(app.status.is_none());
    assert_eq!(app.units.len(), 1);
  }

  #[test]
  /// Executes the `arrows_are_blocked_while_services_are_loading` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn arrows_are_blocked_while_services_are_loading() {
    let mut app = ServicesApp::new(Lang::for_locale("en-US"), Theme::load());
    app.page = ServicePage::System;
    app.selected = 3;
    app.job = Some(app.manager.spawn(|_| {
      std::thread::sleep(std::time::Duration::from_millis(150));
      Ok(JobData::Units(vec![]))
    }));
    app.handle(KeyCode::Down);
    app.handle(KeyCode::Up);
    app.handle(KeyCode::Home);
    app.handle(KeyCode::Tab);
    assert_eq!(app.selected, 3);
    assert!(!app.on_buttons);
  }
}
