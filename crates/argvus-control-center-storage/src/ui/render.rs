//! Drawing of the Storage page: lists, overlays, footer hints and the breadcrumb.

use super::*;

impl StorageApp {
  pub fn breadcrumb(&self) -> String {
    let root = format!(
      "{} > {}",
      HomeCardId::Hardware.label(self.lang),
      tr(self.lang, "control_center.storage")
    );
    if self.page == StoragePage::Home {
      root
    } else {
      format!("{root} > {}", self.label())
    }
  }

  /// Reads this destination's visible labels from the current snapshot.
  pub fn search_content(&mut self, page: StoragePage) -> (String, Vec<String>) {
    let previous = std::mem::replace(&mut self.page, page);
    let path = self.breadcrumb();
    let words = self.rows();
    self.page = previous;
    (path, words)
  }

  pub fn draw(&mut self, frame: &mut Frame) {
    self.draw_in(frame, frame.area());
  }

  pub fn draw_in(&mut self, frame: &mut Frame, area: ratatui::layout::Rect) {
    let body = argvus_tui::page::shell_with_status(
      frame,
      area,
      &self.theme,
      self.lang,
      &self.breadcrumb(),
      if self.len() > 0 {
        tr(self.lang, "control_center.scroll_r_refresh_esc_back_help")
      } else {
        tr(self.lang, "control_center.r_refresh_esc_back_help")
      },
      argvus_tui::page::PageStatus {
        message: self.status.as_ref(),
        admin_required: false,
      },
    );
    let rows = self.rows();
    if matches!(
      self.page,
      StoragePage::Summary
        | StoragePage::DiskDetails(_)
        | StoragePage::PartitionDetails(_)
        | StoragePage::FilesystemDetails(_)
        | StoragePage::MountDetails(_)
        | StoragePage::SmartDetails(_)
    ) {
      readonly(
        frame,
        body,
        &self.theme,
        &rows.into_iter().map(Line::from).collect::<Vec<_>>(),
      );
    } else {
      list(
        frame,
        body,
        &self.theme,
        &rows,
        self.selection.index.min(rows.len().saturating_sub(1)),
      );
    }
  }
}
