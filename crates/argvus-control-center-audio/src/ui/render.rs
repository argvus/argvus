//! Drawing of the Audio page: lists, overlays, footer hints and the breadcrumb.

use super::*;

impl AudioApp {
  /// Returns the footer hints for the current page: device pages list their keys, others stay read-only.
  pub(super) fn footer_hints(&self) -> &'static str {
    let devices = tr(
      self.lang,
      "control_center.navigate_volume_enter_set_default_r_refresh_s_save_esc_back_help",
    );
    let readonly = tr(self.lang, "control_center.r_refresh_esc_back_help");
    let home = tr(
      self.lang,
      "control_center.navigate_enter_open_esc_back_r_refresh_help",
    );
    if self.page == AudioPage::Home {
      home
    } else if self.has_device_actions() {
      devices
    } else {
      readonly
    }
  }

  /// Reads this destination's visible labels from the current snapshot.
  pub fn search_content(&mut self, page: AudioPage) -> (String, Vec<String>) {
    let previous = std::mem::replace(&mut self.page, page);
    let path = self.breadcrumb();
    let words = self.rows();
    self.page = previous;
    (path, words)
  }

  pub fn draw(&self, frame: &mut Frame) {
    self.draw_in(frame, frame.area());
  }

  pub fn draw_in(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
    let body = argvus_tui::page::shell_with_status(
      frame,
      area,
      &self.theme,
      self.lang,
      &self.breadcrumb(),
      self.footer_hints(),
      argvus_tui::page::PageStatus {
        message: self.status.as_ref(),
        admin_required: false,
      },
    );
    let rows = self.rows();
    if matches!(self.page, AudioPage::Summary | AudioPage::Devices) {
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
        self.selected.index.min(rows.len().saturating_sub(1)),
      );
    }
    if let Some(value) = &self.volume_input {
      let popup = argvus_tui::chrome::centered(area, 48, 7);
      frame.render_widget(Clear, popup);
      frame.render_widget(
        Paragraph::new(vec![
          Line::from(tr(self.lang, "control_center.enter_volume_0_100")),
          Line::from(format!("{value}_")),
          Line::from(tr(self.lang, "control_center.enter_apply_esc_cancel")),
        ])
        .block(Block::bordered().title(tr(self.lang, "control_center.volume_3e7bd7"))),
        popup,
      );
    }
    if self.confirm_discard {
      draw_confirmation(
        frame,
        area,
        &self.theme,
        ConfirmationDialog {
          title: tr(self.lang, "control_center.discard_changes_title"),
          message: tr(self.lang, "control_center.discard_changes_description"),
          confirm_label: tr(self.lang, "control_center.discard"),
          cancel_label: tr(self.lang, "control_center.cancel"),
          confirm_selected: self.confirmation.confirm_selected,
        },
      );
    }
  }

  pub(super) fn breadcrumb(&self) -> String {
    let root = format!(
      "{} > {}",
      HomeCardId::Hardware.label(self.lang),
      tr(self.lang, "control_center.audio")
    );
    if self.page == AudioPage::Home {
      root
    } else {
      format!("{root} > {}", self.page_label())
    }
  }
}
