//! Rows and labels of the Diagnostics page lists.

use super::*;

impl DiagnosticsApp {
  pub(super) fn dashboard_rows(&self) -> Vec<String> {
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

  pub(super) fn category_rows(&self) -> Vec<String> {
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

  pub(super) fn summary_rows(&self) -> Vec<String> {
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

  pub(super) fn detail_rows(&self, index: usize) -> Vec<String> {
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
}

pub(super) fn severity_label(lang: Lang, severity: Severity) -> &'static str {
  match severity {
    Severity::Error => tr(lang, "control_center.error"),
    Severity::Warning => tr(lang, "control_center.warning"),
    Severity::Unknown => tr(lang, "control_center.unknown_f9ac0c"),
    Severity::Info => tr(lang, "control_center.info"),
    Severity::Ok => tr(lang, "control_center.ok"),
  }
}

pub(super) fn category_label(lang: Lang, category: &str) -> &'static str {
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
