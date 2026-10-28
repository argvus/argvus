//! Terminal UI rendering and interaction of the Diagnostics page.
use crate::{backend, model::*};
use argvus_control_center_core::navigation::HomeCardId;
use argvus_control_center_core::{
  capabilities::Capabilities,
  config::AppConfig,
  jobs::{JobHandle, JobManager, JobState},
};
use argvus_i18n::{Lang, tr};
use argvus_theme::Theme;
use argvus_tui::{
  components::{StatusKind, StatusMessage},
  page::{Selection, list, readonly, shell, status},
};
use crossterm::event::KeyCode;
use ratatui::{Frame, text::Line};

mod jobs;
mod render;
mod rows;

pub struct DiagnosticsApp {
  pub page: DiagnosticPage,
  checks: Vec<DiagnosticCheck>,
  selected: Selection,
  job: Option<JobHandle<Vec<DiagnosticCheck>>>,
  jobs: JobManager,
  cap: Capabilities,
  pub status: Option<StatusMessage>,
  pub lang: Lang,
  pub theme: Theme,
}
impl DiagnosticsApp {
  /// Replaces the semantic theme used by this page.
  pub fn set_theme(&mut self, theme: &Theme) {
    self.theme = theme.clone();
  }

  pub fn new(lang: Lang, theme: Theme, cap: Capabilities) -> Self {
    let mut a = Self {
      page: DiagnosticPage::Home,
      checks: Vec::new(),
      selected: Selection::default(),
      job: None,
      jobs: JobManager::default(),
      cap,
      status: None,
      lang,
      theme,
    };
    a.refresh();
    a
  }
  fn len(&self) -> usize {
    match self.page {
      DiagnosticPage::Home => self.dashboard_rows().len(),
      DiagnosticPage::Summary | DiagnosticPage::Detail(_) => 1,
      _ => self.category_checks().count(),
    }
  }
  fn category(&self) -> &str {
    match self.page {
      DiagnosticPage::Services => "services",
      DiagnosticPage::Boot => "boot",
      DiagnosticPage::Graphics => "graphics",
      DiagnosticPage::Network => "network",
      DiagnosticPage::Audio => "audio",
      DiagnosticPage::Bluetooth => "bluetooth",
      DiagnosticPage::Storage => "storage",
      DiagnosticPage::Packages => "packages",
      DiagnosticPage::Argvus => "argvus",
      _ => "system",
    }
  }
  fn category_checks(&self) -> impl Iterator<Item = (usize, &DiagnosticCheck)> {
    let category = self.category();
    self
      .checks
      .iter()
      .enumerate()
      .filter(move |(_, check)| check.category == category)
  }
  fn detail_index(&self) -> usize {
    match self.page {
      DiagnosticPage::Detail(index) => index,
      _ => 0,
    }
  }
  fn back_page(&self) -> DiagnosticPage {
    match self.page {
      DiagnosticPage::Detail(index) => self
        .checks
        .get(index)
        .and_then(|check| page_for_category(&check.category))
        .unwrap_or(DiagnosticPage::Home),
      DiagnosticPage::Summary => DiagnosticPage::Home,
      _ => DiagnosticPage::Home,
    }
  }
  pub fn handle(&mut self, key: KeyCode) -> bool {
    if matches!(key, KeyCode::Esc | KeyCode::Left) {
      if self.page == DiagnosticPage::Home {
        return true;
      }
      self.page = self.back_page();
      self.selected.index = 0;
      return false;
    }
    if self.busy() {
      return false;
    }
    if key == KeyCode::Char('r') {
      self.refresh();
      return false;
    }
    self.selected.handle(key, self.len(), 8);
    if matches!(key, KeyCode::Enter | KeyCode::Right) {
      self.open_selected();
    }
    false
  }
  fn open_selected(&mut self) {
    match self.page {
      DiagnosticPage::Home => {
        self.page = match self.selected.index {
          0 => DiagnosticPage::Summary,
          1 => DiagnosticPage::Services,
          2 => DiagnosticPage::Boot,
          3 => DiagnosticPage::Graphics,
          4 => DiagnosticPage::Network,
          5 => DiagnosticPage::Audio,
          6 => DiagnosticPage::Bluetooth,
          7 => DiagnosticPage::Storage,
          8 => DiagnosticPage::Packages,
          _ => DiagnosticPage::Argvus,
        };
        self.selected.index = 0;
      }
      DiagnosticPage::Services
      | DiagnosticPage::Boot
      | DiagnosticPage::Graphics
      | DiagnosticPage::Network
      | DiagnosticPage::Audio
      | DiagnosticPage::Bluetooth
      | DiagnosticPage::Storage
      | DiagnosticPage::Packages
      | DiagnosticPage::Argvus => {
        let index = self
          .category_checks()
          .nth(self.selected.index)
          .map(|(index, _)| index);
        if let Some(index) = index {
          self.page = DiagnosticPage::Detail(index);
          self.selected.index = 0;
        }
      }
      _ => {}
    }
  }
  fn dashboard_entry(&self, category: &str, icon: &str, label: &'static str) -> String {
    let checks: Vec<&DiagnosticCheck> = self
      .checks
      .iter()
      .filter(|check| check.category == category)
      .collect();
    format!(
      "{} {}  ·  {}",
      AppConfig::icon(icon),
      label,
      self.entry_status(&checks)
    )
  }
  fn entry_status(&self, checks: &[&DiagnosticCheck]) -> String {
    if checks.is_empty() {
      tr(self.lang, "control_center.no_data").into()
    } else {
      format!(
        "{} · {} {}",
        self.preview(checks),
        checks.len(),
        tr(self.lang, "control_center.checks")
      )
    }
  }
  fn preview(&self, checks: &[&DiagnosticCheck]) -> String {
    let errors = checks
      .iter()
      .filter(|check| check.severity == Severity::Error)
      .count();
    let warnings = checks
      .iter()
      .filter(|check| check.severity == Severity::Warning)
      .count();
    if checks.is_empty() {
      tr(self.lang, "control_center.no_data").into()
    } else if errors > 0 {
      format!("{} {}", errors, tr(self.lang, "control_center.error_s"))
    } else if warnings > 0 {
      format!("{} {}", warnings, tr(self.lang, "control_center.warning_s"))
    } else {
      tr(self.lang, "control_center.ok").into()
    }
  }
}

fn page_for_category(category: &str) -> Option<DiagnosticPage> {
  match category {
    "services" => Some(DiagnosticPage::Services),
    "boot" => Some(DiagnosticPage::Boot),
    "graphics" => Some(DiagnosticPage::Graphics),
    "network" => Some(DiagnosticPage::Network),
    "audio" => Some(DiagnosticPage::Audio),
    "bluetooth" => Some(DiagnosticPage::Bluetooth),
    "storage" => Some(DiagnosticPage::Storage),
    "packages" => Some(DiagnosticPage::Packages),
    "argvus" => Some(DiagnosticPage::Argvus),
    _ => None,
  }
}
fn severity_glyph(severity: Severity) -> &'static str {
  match severity {
    Severity::Error => "✖",
    Severity::Warning => "▲",
    Severity::Unknown => "?",
    Severity::Info => "•",
    Severity::Ok => "✔",
  }
}
fn category_icon(category: &str) -> &'static str {
  match category {
    "services" => argvus_tui::icons::SETTINGS,
    "boot" => argvus_tui::icons::MEMORY,
    "graphics" => argvus_tui::icons::GPU,
    "network" => argvus_tui::icons::NETWORK,
    "audio" => argvus_tui::icons::AUDIO,
    "bluetooth" => argvus_tui::icons::LINK,
    "storage" => argvus_tui::icons::STORAGE,
    "packages" => argvus_tui::icons::PACKAGES,
    "argvus" => argvus_tui::icons::SUCCESS,
    _ => argvus_tui::icons::MONITOR,
  }
}
fn completion_status(lang: Lang, checks: &[DiagnosticCheck]) -> StatusMessage {
  let errors = checks
    .iter()
    .filter(|check| check.severity == Severity::Error)
    .count();
  let warnings = checks
    .iter()
    .filter(|check| check.severity == Severity::Warning)
    .count();
  if errors > 0 {
    StatusMessage {
      kind: StatusKind::Error,
      text: format!("{} {}", errors, tr(lang, "control_center.error_s_found")),
    }
  } else if warnings > 0 {
    StatusMessage {
      kind: StatusKind::Warning,
      text: format!(
        "{} {}",
        warnings,
        tr(lang, "control_center.warning_s_found")
      ),
    }
  } else {
    StatusMessage {
      kind: StatusKind::Success,
      text: tr(lang, "control_center.diagnostics_completed_all_systems_ok").into(),
    }
  }
}

#[cfg(test)]
mod tests;
