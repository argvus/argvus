//! Implements terminal UI rendering and interaction in crate `argvus control center diagnostics`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
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

/// Represents `DiagnosticsApp`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
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

  /// Constructs `new` with this module's expected initial state. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
  /// Executes the `reload` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn reload(&mut self) {
    self.refresh()
  }
  /// Executes the `refresh` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn refresh(&mut self) {
    if self.job.is_some() {
      return;
    }
    let cap = self.cap.clone();
    let lang = self.lang;
    self.job = Some(
      self
        .jobs
        .spawn(move |_| Ok::<_, String>(backend::evaluate(lang, &backend::collect(&cap), &cap))),
    );
    self.status = Some(StatusMessage {
      kind: StatusKind::Info,
      text: tr(self.lang, "control_center.running_checks").into(),
    });
  }
  /// Executes the `poll` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn poll(&mut self) -> bool {
    let Some(job) = &self.job else {
      return false;
    };
    if let JobState::Finished(result) = job.try_state() {
      self.job = None;
      match result {
        Ok(checks) => {
          self.checks = checks;
          self.selected.normalize(self.len());
          self.status = Some(completion_status(self.lang, &self.checks));
        }
        Err(error) => {
          self.status = Some(StatusMessage {
            kind: StatusKind::Error,
            text: error,
          });
        }
      }
      true
    } else {
      false
    }
  }
  /// Executes the `busy` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn busy(&self) -> bool {
    self.job.is_some()
  }
  /// Executes the `len` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn len(&self) -> usize {
    match self.page {
      DiagnosticPage::Home => self.dashboard_rows().len(),
      DiagnosticPage::Summary | DiagnosticPage::Detail(_) => 1,
      _ => self.category_checks().count(),
    }
  }
  /// Executes the `category` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
  /// Executes the `category_checks` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn category_checks(&self) -> impl Iterator<Item = (usize, &DiagnosticCheck)> {
    let category = self.category();
    self
      .checks
      .iter()
      .enumerate()
      .filter(move |(_, check)| check.category == category)
  }
  /// Executes the `detail_index` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn detail_index(&self) -> usize {
    match self.page {
      DiagnosticPage::Detail(index) => index,
      _ => 0,
    }
  }
  /// Executes the `back_page` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
  /// Processes `handle` in this module's event flow. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
  /// Executes the `open_selected` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
  /// Executes the `breadcrumb` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn breadcrumb(&self) -> String {
    let root = format!(
      "{} > {}",
      HomeCardId::Tools.label(self.lang),
      tr(self.lang, "control_center.diagnostics")
    );
    match self.page {
      DiagnosticPage::Home => root,
      DiagnosticPage::Summary => {
        format!("{root} > {}", tr(self.lang, "control_center.summary"))
      }
      DiagnosticPage::Detail(index) => format!(
        "{root} > {}",
        self
          .checks
          .get(index)
          .map(|check| category_label(self.lang, &check.category))
          .unwrap_or_else(|| tr(self.lang, "control_center.details"))
      ),
      _ => format!("{root} > {}", category_label(self.lang, self.category())),
    }
  }
  /// Executes the `footer_hints` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn footer_hints(&self) -> &'static str {
    match self.page {
      DiagnosticPage::Home => tr(
        self.lang,
        "control_center.navigate_enter_open_r_refresh_esc_back_help",
      ),
      DiagnosticPage::Summary | DiagnosticPage::Detail(_) => {
        tr(self.lang, "control_center.r_refresh_esc_back_help")
      }
      _ => tr(
        self.lang,
        "control_center.navigate_enter_details_r_refresh_esc_back_help",
      ),
    }
  }
  /// Executes the `dashboard_rows` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn dashboard_rows(&self) -> Vec<String> {
    let all: Vec<&DiagnosticCheck> = self.checks.iter().collect();
    let summary = format!(
      "{} {}  ·  {}",
      AppConfig::icon(argvus_tui::icons::MONITOR),
      tr(self.lang, "control_center.summary"),
      self.entry_status(&all)
    );
    vec![
      summary,
      self.dashboard_entry(
        "services",
        argvus_tui::icons::SETTINGS,
        tr(self.lang, "control_center.services"),
      ),
      self.dashboard_entry(
        "boot",
        argvus_tui::icons::MEMORY,
        tr(self.lang, "control_center.kernel_boot"),
      ),
      self.dashboard_entry(
        "graphics",
        argvus_tui::icons::GPU,
        tr(self.lang, "control_center.graphics"),
      ),
      self.dashboard_entry(
        "network",
        argvus_tui::icons::NETWORK,
        tr(self.lang, "control_center.network"),
      ),
      self.dashboard_entry(
        "audio",
        argvus_tui::icons::AUDIO,
        tr(self.lang, "control_center.audio"),
      ),
      self.dashboard_entry(
        "bluetooth",
        argvus_tui::icons::LINK,
        tr(self.lang, "control_center.bluetooth"),
      ),
      self.dashboard_entry(
        "storage",
        argvus_tui::icons::STORAGE,
        tr(self.lang, "control_center.storage"),
      ),
      self.dashboard_entry(
        "packages",
        argvus_tui::icons::PACKAGES,
        tr(self.lang, "control_center.packages"),
      ),
      self.dashboard_entry(
        "argvus",
        argvus_tui::icons::SUCCESS,
        tr(self.lang, "control_center.argvus"),
      ),
    ]
  }
  /// Executes the `dashboard_entry` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
  /// Executes the `entry_status` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
  /// Executes the `preview` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
  /// Executes the `category_rows` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn category_rows(&self) -> Vec<String> {
    self
      .category_checks()
      .map(|(_, check)| {
        format!(
          "{} {} — {}",
          severity_glyph(check.severity),
          check.title,
          check.summary
        )
      })
      .collect()
  }
  /// Executes the `summary_rows` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn summary_rows(&self) -> Vec<String> {
    let errors = self
      .checks
      .iter()
      .filter(|check| check.severity == Severity::Error)
      .count();
    let warnings = self
      .checks
      .iter()
      .filter(|check| check.severity == Severity::Warning)
      .count();
    let unknown = self
      .checks
      .iter()
      .filter(|check| check.severity == Severity::Unknown)
      .count();
    let mut rows = vec![
      format!(
        " {} {}",
        AppConfig::icon(argvus_tui::icons::MONITOR),
        tr(self.lang, "control_center.diagnostics_cc02b3")
      ),
      format!(
        "   {:<14} {}",
        tr(self.lang, "control_center.checks_75a9fc"),
        self.checks.len()
      ),
      format!(
        "   {:<14} {}",
        tr(self.lang, "control_center.errors"),
        errors
      ),
      format!(
        "   {:<14} {}",
        tr(self.lang, "control_center.warnings"),
        warnings
      ),
      format!(
        "   {:<14} {}",
        tr(self.lang, "control_center.unknown_5951be"),
        unknown
      ),
    ];
    let system: Vec<&DiagnosticCheck> = self
      .checks
      .iter()
      .filter(|check| check.category == "system")
      .collect();
    if !system.is_empty() {
      rows.push(String::new());
      rows.push(format!(
        " {} {}",
        AppConfig::icon(argvus_tui::icons::SETTINGS),
        tr(self.lang, "control_center.system_1af4b7")
      ));
      rows.extend(system.into_iter().map(|check| {
        format!(
          "   {} {} — {}",
          severity_glyph(check.severity),
          check.title,
          check.summary
        )
      }));
    }
    rows
  }
  /// Executes the `detail_rows` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn detail_rows(&self, index: usize) -> Vec<String> {
    let Some(check) = self.checks.get(index) else {
      return vec![tr(self.lang, "control_center.check_not_found").into()];
    };
    vec![
      format!(
        " {} {}",
        AppConfig::icon(category_icon(&check.category)),
        category_label(self.lang, &check.category).to_uppercase()
      ),
      format!(
        "   {:<14} {}",
        tr(self.lang, "control_center.category"),
        category_label(self.lang, &check.category)
      ),
      format!(
        "   {:<14} {}",
        tr(self.lang, "control_center.severity"),
        severity_label(self.lang, check.severity)
      ),
      format!(
        "   {:<14} {}",
        tr(self.lang, "control_center.title"),
        check.title
      ),
      format!(
        "   {:<14} {}",
        tr(self.lang, "control_center.summary_1543d0"),
        check.summary
      ),
      String::new(),
      format!(
        "   {:<14} {}",
        tr(self.lang, "control_center.details_1bae81"),
        check.details
      ),
      String::new(),
      format!(
        "   {:<14} {}",
        tr(self.lang, "control_center.remediation"),
        check
          .remediation_hint
          .as_deref()
          .unwrap_or_else(|| tr(self.lang, "control_center.no_automatic_action"))
      ),
    ]
  }
  /// Renders `draw` while respecting the current domain state and semantic theme. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn draw(&mut self, frame: &mut Frame) {
    let body = shell(
      frame,
      frame.area(),
      &self.theme,
      self.lang,
      &self.breadcrumb(),
      self.footer_hints(),
    );
    match self.page {
      DiagnosticPage::Home => {
        let rows = self.dashboard_rows();
        list(
          frame,
          body,
          &self.theme,
          &rows,
          self.selected.index.min(rows.len().saturating_sub(1)),
        );
      }
      DiagnosticPage::Summary => {
        let rows = self
          .summary_rows()
          .into_iter()
          .map(Line::from)
          .collect::<Vec<_>>();
        readonly(frame, body, &self.theme, &rows);
      }
      DiagnosticPage::Detail(_) => {
        let rows = self
          .detail_rows(self.detail_index())
          .into_iter()
          .map(Line::from)
          .collect::<Vec<_>>();
        readonly(frame, body, &self.theme, &rows);
      }
      _ => {
        let rows = self.category_rows();
        list(
          frame,
          body,
          &self.theme,
          &rows,
          self.selected.index.min(rows.len().saturating_sub(1)),
        );
      }
    }
    if let Some(message) = &self.status {
      status(frame, body, &self.theme, message);
    }
  }
}

/// Executes the `page_for_category` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
/// Executes the `severity_glyph` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn severity_glyph(severity: Severity) -> &'static str {
  match severity {
    Severity::Error => "✖",
    Severity::Warning => "▲",
    Severity::Unknown => "?",
    Severity::Info => "•",
    Severity::Ok => "✔",
  }
}
/// Executes the `severity_label` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn severity_label(lang: Lang, severity: Severity) -> &'static str {
  match severity {
    Severity::Error => tr(lang, "control_center.error"),
    Severity::Warning => tr(lang, "control_center.warning"),
    Severity::Unknown => tr(lang, "control_center.unknown_f9ac0c"),
    Severity::Info => tr(lang, "control_center.info"),
    Severity::Ok => tr(lang, "control_center.ok"),
  }
}
/// Executes the `category_label` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn category_label(lang: Lang, category: &str) -> &'static str {
  match category {
    "services" => tr(lang, "control_center.services"),
    "boot" => tr(lang, "control_center.kernel_boot"),
    "graphics" => tr(lang, "control_center.graphics"),
    "network" => tr(lang, "control_center.network"),
    "audio" => tr(lang, "control_center.audio"),
    "bluetooth" => tr(lang, "control_center.bluetooth"),
    "storage" => tr(lang, "control_center.storage"),
    "packages" => tr(lang, "control_center.packages"),
    "argvus" => tr(lang, "control_center.argvus"),
    _ => tr(lang, "control_center.system"),
  }
}
/// Executes the `category_icon` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
/// Executes the `completion_status` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
mod tests {
  use super::*;
  use ratatui::{Terminal, backend::TestBackend};

  /// Executes the `app` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn app() -> DiagnosticsApp {
    app_in(Lang::for_locale("en-US"))
  }

  /// The diagnostics page in `lang`, with fixed checks.
  fn app_in(lang: Lang) -> DiagnosticsApp {
    let mut app = DiagnosticsApp::new(lang, Theme::load(), Capabilities::default());
    app.job = None;
    app.checks = vec![
      DiagnosticCheck {
        id: "system.failed_units".into(),
        category: "services".into(),
        severity: Severity::Warning,
        title: "Failed system units".into(),
        summary: "2 system unit(s) failed".into(),
        details: "The system manager reports failed units.".into(),
        remediation_hint: Some("Open Services > Failed".into()),
        ..Default::default()
      },
      DiagnosticCheck {
        id: "graphics.opengl".into(),
        category: "graphics".into(),
        severity: Severity::Ok,
        title: "OpenGL renderer".into(),
        summary: "Mesa hardware renderer".into(),
        details: "OpenGL reports a hardware renderer.".into(),
        ..Default::default()
      },
      DiagnosticCheck {
        id: "graphics.environment".into(),
        category: "system".into(),
        severity: Severity::Info,
        title: "Graphics environment".into(),
        summary: "Hardware graphics checks are provided by Hardware".into(),
        ..Default::default()
      },
    ];
    app
  }

  #[test]
  /// Executes the `diagnostics_home_dashboard_lists_categories_with_previews` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn diagnostics_home_dashboard_lists_categories_with_previews() {
    let app = app();
    let rows = app.dashboard_rows();
    assert_eq!(rows.len(), 10);
    assert!(
      rows[0].contains("Resumo") || rows[0].contains("Summary"),
      "first row must summarize"
    );
    assert!(rows[1].contains("Serviços") || rows[1].contains("Services"));
    assert!(rows[1].contains("warning"));
    assert!(rows[3].contains("Gráficos") || rows[3].contains("Graphics"));
    assert!(rows[3].contains("OK"));
    assert!(
      rows[9].contains("ARGVUS"),
      "last row must cover the ARGVUS category"
    );
  }

  #[test]
  /// Executes the `diagnostics_category_rows_render_severity_glyphs` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn diagnostics_category_rows_render_severity_glyphs() {
    let mut app = app();
    app.page = DiagnosticPage::Services;
    let rows = app.category_rows();
    assert_eq!(rows.len(), 1);
    assert!(rows[0].starts_with("▲"));
    assert!(rows[0].contains("Failed system units"));
    assert!(rows[0].contains("2 system unit(s) failed"));
  }

  #[test]
  /// Executes the `diagnostics_detail_rows_render_aligned_sections` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn diagnostics_detail_rows_render_aligned_sections() {
    let app = app();
    let rows = app.detail_rows(0);
    assert!(
      rows[0].contains("SERVICES"),
      "first row is a section header"
    );
    assert!(rows.iter().any(|row| row.contains("Severity:")));
    assert!(
      rows
        .iter()
        .any(|row| row.contains("Open Services > Failed"))
    );
  }

  #[test]
  /// Executes the `diagnostics_summary_rows_include_counters_and_system_checks` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn diagnostics_summary_rows_include_counters_and_system_checks() {
    let app = app();
    let rows = app.summary_rows();
    assert!(rows[0].contains("DIAGNOSTICS"));
    assert!(rows.iter().any(|row| row.contains("Checks:")));
    assert!(rows.iter().any(|row| row.contains("SYSTEM")));
    assert!(rows.iter().any(|row| row.contains("Graphics environment")));
  }

  #[test]
  /// Executes the `diagnostics_home_opens_summary_and_escapes_back` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn diagnostics_home_opens_summary_and_escapes_back() {
    let mut app = app();
    app.handle(KeyCode::Enter);
    assert_eq!(app.page, DiagnosticPage::Summary);
    app.handle(KeyCode::Esc);
    assert_eq!(app.page, DiagnosticPage::Home);
    assert!(
      app.handle(KeyCode::Esc),
      "second back returns to global home"
    );
  }

  #[test]
  /// Executes the `diagnostics_category_details_back_out_one_level` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn diagnostics_category_details_back_out_one_level() {
    let mut app = app();
    app.page = DiagnosticPage::Services;
    app.handle(KeyCode::Enter);
    assert_eq!(app.page, DiagnosticPage::Detail(0));
    app.handle(KeyCode::Esc);
    assert_eq!(app.page, DiagnosticPage::Services);
    app.handle(KeyCode::Esc);
    assert_eq!(app.page, DiagnosticPage::Home);
  }

  #[test]
  /// Executes the `diagnostics_uses_shared_chrome_and_contextual_footer` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn diagnostics_uses_shared_chrome_and_contextual_footer() {
    let mut app = app();
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
    assert!(text.contains("Diagnostics"));
    assert!(text.contains("Enter") && text.contains("Back"));

    app.page = DiagnosticPage::Services;
    let mut terminal = Terminal::new(TestBackend::new(90, 25)).unwrap();
    terminal.draw(|frame| app.draw(frame)).unwrap();
    let text = terminal
      .backend()
      .buffer()
      .content
      .iter()
      .map(|cell| cell.symbol())
      .collect::<String>();
    assert!(text.contains("Services"));
    assert!(text.contains("Details"));
  }

  #[test]
  /// The header is drawn from the translated root and card: in pt-BR the rendered
  /// screen shows the full path in Portuguese and no English navigation segment.
  fn diagnostics_header_is_translated_in_pt_br() {
    let app = app_in(Lang::for_locale("pt-BR"));
    let mut terminal = Terminal::new(TestBackend::new(120, 30)).unwrap();
    let mut app = app;
    terminal.draw(|frame| app.draw(frame)).unwrap();
    let rendered = terminal
      .backend()
      .buffer()
      .content
      .iter()
      .map(|cell| cell.symbol())
      .collect::<String>();
    assert!(
      rendered.contains("Central de Controle > Ferramentas e diagnóstico > Diagnóstico"),
      "{rendered}"
    );
    assert!(!rendered.contains("Tools & Diagnostics"), "{rendered}");
  }

  #[test]
  /// Executes the `diagnostics_detail_footer_and_breadcrumb_follow_the_parent_category` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn diagnostics_detail_footer_and_breadcrumb_follow_the_parent_category() {
    let app = app();
    assert_eq!(app.breadcrumb(), "Tools & Diagnostics > Diagnostics");
    let mut detail = app;
    detail.page = DiagnosticPage::Detail(2);
    assert!(detail.breadcrumb().ends_with("System"));
    assert!(detail.footer_hints().contains("Back"));
  }

  #[test]
  /// Executes the `completion_status_reflects_worst_severity` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn completion_status_reflects_worst_severity() {
    assert_eq!(
      completion_status(Lang::for_locale("en-US"), &[]).kind,
      StatusKind::Success
    );
    let mut checks = app();
    checks.checks = vec![DiagnosticCheck {
      id: "storage.usage./".into(),
      category: "storage".into(),
      severity: Severity::Error,
      ..Default::default()
    }];
    assert_eq!(
      completion_status(Lang::for_locale("en-US"), &checks.checks).kind,
      StatusKind::Error
    );
    let mut warnings = app();
    warnings.checks = vec![DiagnosticCheck {
      id: "packages.lock".into(),
      category: "packages".into(),
      severity: Severity::Warning,
      ..Default::default()
    }];
    assert_eq!(
      completion_status(Lang::for_locale("en-US"), &warnings.checks).kind,
      StatusKind::Warning
    );
  }
}
