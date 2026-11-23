//! Drawing of the Session page: lists, overlays, footer hints and the breadcrumb.

use super::*;

impl SessionApp {
  pub(super) fn footer_hints(&self) -> &'static str {
    match self.page {
      SessionPage::Home | SessionPage::Diagnostics => tr(
        self.lang,
        "control_center.navigate_enter_open_r_refresh_esc_back_help",
      ),
      SessionPage::LogDetail(_) => tr(self.lang, "control_center.scroll_esc_back_help"),
      _ => tr(
        self.lang,
        "control_center.navigate_tab_actions_enter_activate_move_r_refresh_esc_back_help",
      ),
    }
  }

  pub(super) fn breadcrumb(&self) -> String {
    let root = format!(
      "{} > {}",
      HomeCardId::Preferences.label(self.lang),
      tr(self.lang, "control_center.session")
    );
    if self.page == SessionPage::Home {
      root
    } else {
      format!("{root} > {}", self.page_label())
    }
  }

  /// Reads this destination's visible labels from the current snapshot.
  pub fn search_content(&mut self, page: SessionPage) -> (String, Vec<String>) {
    let previous = std::mem::replace(&mut self.page, page);
    let path = self.breadcrumb();
    let words = match self.page {
      SessionPage::Home => self.home_rows(),
      SessionPage::Components => self.component_rows(),
      SessionPage::Autostart => self.autostart_rows(),
      SessionPage::Diagnostics => self.diagnostics_rows(),
      SessionPage::Logs => self.log_rows(),
      SessionPage::LogDetail(index) => self
        .log_detail_lines(index)
        .into_iter()
        .map(|line| line.to_string())
        .collect(),
    };
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
      self.footer_hints(),
      argvus_tui::page::PageStatus {
        message: self.status.as_ref(),
        admin_required: false,
      },
    );
    let lines = match self.page {
      SessionPage::Home => self.home_rows(),
      SessionPage::Components => self.component_rows(),
      SessionPage::Autostart => self.autostart_rows(),
      SessionPage::Diagnostics => self.diagnostics_rows(),
      SessionPage::Logs => self.log_rows(),
      SessionPage::LogDetail(_) => Vec::new(),
    };
    if let SessionPage::LogDetail(index) = self.page {
      let entries = self.log_detail_lines(index);
      readonly(frame, body, &self.theme, &entries);
      return;
    }
    let buttons = self.buttons();
    let raw_buttons: Vec<Button> = buttons.into_iter().map(|(_, button)| button).collect();
    let (body, button_area) = if raw_buttons.is_empty() {
      (body, None)
    } else {
      let button_height = (argvus_tui::buttons::height(&raw_buttons, body.width.saturating_sub(2))
        + 2)
        .min(body.height);
      let split =
        Layout::vertical([Constraint::Min(1), Constraint::Length(button_height)]).split(body);
      (split[0], Some(split[1]))
    };
    let list_selection = if self.job.is_some() {
      usize::MAX
    } else {
      self.selected.min(lines.len().saturating_sub(1))
    };
    list(frame, body, &self.theme, &lines, list_selection);
    if let Some(button_area) = button_area {
      let focus = if self.on_buttons {
        self.button_selected
      } else {
        usize::MAX
      };
      let block = ratatui::widgets::Block::bordered()
        .border_style(ratatui::style::Style::new().fg(self.theme.border_active))
        .title(
          Line::from(format!(
            "{} · {}",
            tr(self.lang, "control_center.actions"),
            tr(self.lang, "control_center.actions_hint")
          ))
          .style(ratatui::style::Style::new().fg(self.theme.foreground)),
        );
      let inner = block.inner(button_area);
      frame.render_widget(block, button_area);
      let buttons: Vec<_> = raw_buttons
        .iter()
        .map(|button| argvus_tui::action_buttons::ActionButton::secondary("", &button.label, ""))
        .collect();
      argvus_tui::action_buttons::draw_aligned_transparent(
        frame,
        inner,
        &buttons,
        focus,
        &self.theme,
        ratatui::layout::Alignment::Left,
      );
    }
    if let Some(pending) = &self.pending {
      self.draw_pending(frame, area, pending);
    }
  }

  pub(super) fn draw_pending(&self, frame: &mut Frame, area: Rect, pending: &Pending) {
    let (title, message) = match pending {
      Pending::Restart(id) => (
        tr(self.lang, "control_center.confirm_restart"),
        format!(
          "{} {id}?",
          tr(self.lang, "control_center.restart_component")
        ),
      ),
      Pending::ToggleAutostart(id, enabled) => (
        tr(self.lang, "control_center.confirm_autostart"),
        format!(
          "{} {id}?",
          if *enabled {
            tr(self.lang, "control_center.enable_8adac7")
          } else {
            tr(self.lang, "control_center.disable_5de12e")
          }
        ),
      ),
    };
    draw_confirmation(
      frame,
      area,
      &self.theme,
      ConfirmationDialog {
        title,
        message: &message,
        confirm_label: tr(self.lang, "control_center.continue"),
        cancel_label: tr(self.lang, "control_center.cancel"),
        confirm_selected: self.confirmation.confirm_selected,
      },
    )
  }
}
