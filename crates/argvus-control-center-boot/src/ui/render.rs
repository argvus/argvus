//! Drawing of the Boot page: lists, overlays, footer hints and the breadcrumb.

use super::actions::action_message;
use super::input::input_tail;
use super::*;

impl BootApp {
  /// The footer derived from the selected row.
  pub(super) fn footer_hints(&self, rows: &[Row<Item>]) -> String {
    if self.pending.is_some() {
      return confirm_hints(self.lang);
    }
    let mut menu = self.menu;
    menu.normalize(rows);
    hints(
      self.lang,
      &HintContext {
        row: menu.selected_kind(rows),
        can_go_back: true,
        refresh: true,
        ..HintContext::default()
      },
    )
  }

  pub fn draw(&mut self, frame: &mut Frame) {
    let area = frame.area();
    let rows = self.rows();
    self.menu.normalize(&rows);
    let body = shell(
      frame,
      area,
      &self.theme,
      self.lang,
      &self.breadcrumb(),
      &self.footer_hints(&rows),
    );
    self.list_height = body.height;
    draw_menu(
      frame,
      body,
      &self.theme,
      &rows,
      &mut self.menu,
      MenuStyle {
        icons: AppConfig::icons_enabled(),
      },
    );
    self.overlays(frame, area, body);
  }

  /// Draws the input field, the confirmation, the status and the live
  /// transaction output over the page.
  pub(super) fn overlays(
    &self,
    frame: &mut Frame,
    area: ratatui::layout::Rect,
    body: ratatui::layout::Rect,
  ) {
    if let Some(input) = &self.timeout_input {
      let width = if self.input_mode == Some(InputMode::GrubCmdline) {
        area
          .width
          .saturating_sub(4)
          .clamp(48, INPUT_POPUP_MAX_WIDTH)
      } else {
        48
      };
      let popup = argvus_tui::chrome::centered(area, width, 7);
      frame.render_widget(Clear, popup);
      frame.render_widget(
        Paragraph::new(vec![
          Line::from(if self.input_mode == Some(InputMode::GrubCmdline) {
            tr(self.lang, "control_center.new_kernel_command_line")
          } else {
            tr(self.lang, "control_center.new_timeout_in_seconds")
          }),
          Line::from(input_tail(
            input,
            usize::from(popup.width.saturating_sub(2)),
          )),
          Line::from(tr(self.lang, "control_center.enter_apply_esc_cancel")),
        ])
        .block(Block::bordered().title(
          if self.input_mode == Some(InputMode::GrubCmdline) {
            tr(self.lang, "control_center.kernel_command_line")
          } else {
            tr(self.lang, "control_center.timeout")
          },
        )),
        popup,
      );
    }
    if let Some(Pending::Action(action)) = &self.pending {
      let message = action_message(self.lang, action);
      draw_confirm(
        frame,
        area,
        &self.theme,
        ConfirmDialog {
          title: tr(self.lang, "control_center.confirm_boot_operation"),
          message: &message,
          confirm: tr(self.lang, "control_center.continue"),
          cancel: tr(self.lang, "control_center.cancel"),
          danger: action.is_destructive(),
          deadline: None,
        },
        &self.confirmation,
      );
    }
    if let Some(status_message) = &self.status {
      status(frame, body, &self.theme, status_message);
    }
    if self.transaction_open
      && let Some(output) = self.transaction_live.as_ref().map(|live| live.output())
    {
      let popup = chrome::centered(
        frame.area(),
        TRANSACTION_POPUP_WIDTH,
        TRANSACTION_POPUP_HEIGHT,
      );
      let total = transaction_wrapped_lines(&output);
      let shown_total = total.max(1);
      let current = (self.transaction_scroll as usize + 1).min(shown_total);
      frame.render_widget(Clear, popup);
      frame.render_widget(
        Paragraph::new(output.as_str())
          .block(
            Block::new()
              .borders(Borders::ALL)
              .title(Line::styled(
                format!(
                  " {} · {} {} {} {} ",
                  tr(self.lang, "control_center.process"),
                  tr(self.lang, "control_center.line"),
                  current,
                  tr(self.lang, "control_center.of"),
                  shown_total,
                ),
                Style::new().fg(self.theme.accent),
              ))
              .border_style(Style::new().fg(self.theme.border_active))
              .style(Style::new().bg(self.theme.background))
              .title_bottom(Line::from(tr(
                self.lang,
                "control_center.scroll_pgup_pgdn_home_end_esc_close",
              ))),
          )
          .scroll((self.transaction_scroll, 0))
          .wrap(Wrap { trim: false }),
        popup,
      );
    }
  }

  pub(super) fn breadcrumb(&self) -> String {
    let root = format!(
      "{} > {}",
      HomeCardId::System.label(self.lang),
      tr(self.lang, "control_center.boot")
    );
    if self.page == BootPage::Home {
      root
    } else {
      format!("{root} > {}", self.page_label())
    }
  }
}

pub(super) fn transaction_wrapped_lines(output: &str) -> usize {
  output
    .lines()
    .map(|line| (text::display_width(line).div_ceil(TRANSACTION_CONTENT_WIDTH)).max(1))
    .sum()
}

pub(super) fn transaction_bottom_offset(output: &str) -> u16 {
  transaction_wrapped_lines(output).saturating_sub(TRANSACTION_CONTENT_HEIGHT) as u16
}
