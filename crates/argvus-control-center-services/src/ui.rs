//! Terminal UI rendering and interaction of the Services page.
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
use argvus_tui::components::{
  ConfirmationDialog, ConfirmationOutcome, ConfirmationState, StatusKind, StatusMessage,
  draw_confirmation,
};
use argvus_tui::filter_search::{filter_line, search_box, titled_block};
use argvus_tui::hints::{HintContext, hints};
use argvus_tui::menu::{MenuEvent, MenuState, MenuStyle, Row, draw_menu};
use argvus_tui::page::{list, shell, status};
use crossterm::event::KeyCode;
use ratatui::{
  Frame,
  layout::{Constraint, Layout, Rect},
  text::Line,
};

mod input;
mod jobs;
mod render;
mod rows;

use rows::action_label;

#[derive(Debug, Clone)]
enum Pending {
  Action(String, String),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Identifies the rows of the unit lists (System, User, Failed) and the
/// action rows of the details page.
enum Item {
  Unit(usize),
  DetailAction(&'static str),
}
pub struct ServicesApp {
  pub page: ServicePage,
  detail_parent: ServicePage,
  detail_unit: Option<String>,
  selected: usize,
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
enum JobData {
  Units(Vec<Unit>),
  Logs(Vec<JournalEntry>),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UnitFilter {
  All,
  Running,
  Stopped,
  Failed,
  Enabled,
  Disabled,
}
impl UnitFilter {
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
  /// Replaces the semantic theme used by this page.
  pub fn set_theme(&mut self, theme: &Theme) {
    self.theme = theme.clone();
  }

  pub fn new(lang: Lang, theme: Theme) -> Self {
    Self {
      page: ServicePage::Home,
      detail_parent: ServicePage::System,
      detail_unit: None,
      selected: 0,
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
  fn cycle_filter(&mut self) {
    self.filter = self.filter.next();
    self.selected = 0;
    self.menu = MenuState::default();
  }
  fn cycle_priority(&mut self) {
    // `None` is "0-7" (every level). Starting from it must reach "0"; the
    // previous match sent `None` back to `None`, so the filter never moved.
    self.priority = match self.priority.as_deref() {
      None => Some("0".into()),
      Some("7") => None,
      Some(value) => Some((value.parse::<u8>().unwrap_or(0) + 1).to_string()),
    };
    self.refresh();
  }
  /// Shows the previous boot's journal or the current one, then reloads.
  fn toggle_previous_boot(&mut self) {
    self.previous_boot = !self.previous_boot;
    self.refresh();
  }
  /// Whether the page is one of the unit lists (System, User, Failed).
  fn is_list_page(&self) -> bool {
    matches!(
      self.page,
      ServicePage::System | ServicePage::User | ServicePage::Failed
    )
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
  /// The value of the Logs filter line: the boot, service and priority in
  /// use. The keys `b`, `u` and `p` change them.
  fn logs_filter_summary(&self) -> String {
    let boot = if self.previous_boot {
      tr(self.lang, "control_center.previous")
    } else {
      tr(self.lang, "control_center.current_c42247")
    };
    format!(
      "{}: {} · {}: {} · {}: {}",
      tr(self.lang, "control_center.boot"),
      boot,
      tr(self.lang, "control_center.service"),
      self.logs_unit.as_deref().unwrap_or("—"),
      tr(self.lang, "control_center.priority"),
      self.priority.as_deref().unwrap_or("0-7"),
    )
  }
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
  fn detail_or_selected_unit(&self) -> Option<&Unit> {
    self
      .detail_unit
      .as_deref()
      .and_then(|name| self.units.iter().find(|unit| unit.name == name))
      .or_else(|| self.filtered().get(self.selected).copied())
  }
}
/// Width of the Active column in the unit lists; Enabled starts after it.
const STATUS_COLUMN: usize = 12;
/// A filled dot when the state is on, an empty dot when it is off.
fn state_dot(on: bool) -> &'static str {
  if on { "●" } else { "○" }
}
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
fn state_value(u: &Unit) -> String {
  format!("{} / {}", u.active, u.sub)
}
fn log_line(e: &JournalEntry) -> Line<'static> {
  Line::from(format!(
    "{:8} {:24} {:7} {}",
    e.timestamp.as_deref().unwrap_or(""),
    e.unit.as_deref().unwrap_or("kernel"),
    e.priority_name(),
    e.message.as_deref().unwrap_or("")
  ))
}
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
mod tests;
