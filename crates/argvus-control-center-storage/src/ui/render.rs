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

  pub fn draw(&mut self, frame: &mut Frame) {
    let body = shell(
      frame,
      frame.area(),
      &self.theme,
      self.lang,
      &self.breadcrumb(),
      if self.len() > 0 {
        tr(self.lang, "control_center.scroll_r_refresh_esc_back_help")
      } else {
        tr(self.lang, "control_center.r_refresh_esc_back_help")
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
    if let Some(status) = &self.status {
      draw_status(
        frame,
        Rect::new(body.x, body.bottom().saturating_sub(1), body.width, 1),
        &self.theme,
        status,
      );
    }
  }
}
