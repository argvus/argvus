//! Rows and labels of the Session page lists.

use super::*;

impl SessionApp {
  pub(super) fn page_label(&self) -> String {
    match self.page {
      SessionPage::Home => tr(self.lang, "control_center.session").into(),
      SessionPage::Components => tr(self.lang, "control_center.components").into(),
      SessionPage::Autostart => tr(self.lang, "control_center.autostart").into(),
      SessionPage::Diagnostics => tr(self.lang, "control_center.diagnostics").into(),
      SessionPage::Logs => tr(self.lang, "control_center.logs").into(),
      SessionPage::LogDetail(_) => tr(self.lang, "control_center.logs_details").into(),
    }
  }

  pub(super) fn home_rows(&self) -> Vec<String> {
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

  pub(super) fn diagnostics_rows(&self) -> Vec<String> {
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

  pub(super) fn component_rows(&self) -> Vec<String> {
    self
      .components
      .iter()
      .map(|component| component_row(self.lang, component))
      .collect()
  }

  pub(super) fn autostart_rows(&self) -> Vec<String> {
    self
      .autostart
      .iter()
      .map(|entry| autostart_row(self.lang, entry))
      .collect()
  }

  pub(super) fn log_rows(&self) -> Vec<String> {
    self.logs.iter().map(log_line).collect()
  }

  pub(super) fn log_detail_lines(&self, index: usize) -> Vec<Line<'static>> {
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
}

pub(super) fn component_row(lang: Lang, component: &Component) -> String {
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

pub(super) fn autostart_row(lang: Lang, entry: &AutostartEntry) -> String {
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
