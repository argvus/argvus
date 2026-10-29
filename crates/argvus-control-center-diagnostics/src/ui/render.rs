//! Drawing of the Diagnostics page: lists, overlays, footer hints and the breadcrumb.

use super::rows::category_label;
use super::*;

impl DiagnosticsApp {
  pub(super) fn breadcrumb(&self) -> String {
    let root = format!(
      "{} > {}",
      HomeCardId::Tools.label(self.lang),
      tr(self.lang, "control_center.system_doctor")
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

  pub(super) fn footer_hints(&self) -> &'static str {
    match self.page {
      DiagnosticPage::Home => tr(
        self.lang,
        "control_center.navigate_enter_open_r_refresh_esc_back_help",
      ),
      DiagnosticPage::Summary => tr(self.lang, "control_center.r_refresh_esc_back_help"),
      DiagnosticPage::Detail(_) => tr(self.lang, "control_center.doctor_detail_help"),
      _ => tr(
        self.lang,
        "control_center.navigate_enter_details_r_refresh_esc_back_help",
      ),
    }
  }

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
