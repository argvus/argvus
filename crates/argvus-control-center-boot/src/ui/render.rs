//! Drawing of the Boot page: lists, overlays, footer hints and the breadcrumb.

use super::actions::action_message;
use super::input::input_tail;
use super::*;
use ratatui::layout::{Alignment, Constraint, Layout};

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

  /// Reads this destination's visible labels from the current snapshot.
  pub fn search_content(&mut self, page: BootPage) -> (String, Vec<String>) {
    let previous = std::mem::replace(&mut self.page, page);
    let path = self.breadcrumb();
    let words = self
      .rows()
      .into_iter()
      .map(|row| row.label().to_string())
      .collect();
    self.page = previous;
    (path, words)
  }

  pub fn draw(&mut self, frame: &mut Frame) {
    self.draw_in(frame, frame.area());
  }

  pub fn draw_in(&mut self, frame: &mut Frame, area: ratatui::layout::Rect) {
    let rows = self.rows();
    self.menu.normalize(&rows);
    let body = argvus_tui::page::shell_with_status(
      frame,
      area,
      &self.theme,
      self.lang,
      &self.breadcrumb(),
      &self.footer_hints(&rows),
      argvus_tui::page::PageStatus {
        message: self.status.as_ref(),
        admin_required: false,
      },
    );
    let body = argvus_tui::page::admin_warning_top(
      frame,
      body,
      &self.theme,
      self.lang,
      !matches!(
        self.page,
        BootPage::Home | BootPage::Summary | BootPage::Kernel
      ),
    );
    let body = if self.page == BootPage::Initramfs {
      let [list, actions] =
        Layout::vertical([Constraint::Min(0), Constraint::Length(3)]).areas(body);
      let block = Block::bordered()
        .border_style(Style::new().fg(self.theme.border_active))
        .title(
          Line::from(format!(
            "{} · {}",
            tr(self.lang, "control_center.actions"),
            tr(self.lang, "control_center.actions_hint")
          ))
          .style(Style::new().fg(self.theme.foreground)),
        );
      let inner = block.inner(actions);
      frame.render_widget(block, actions);
      let label = tr(self.lang, "control_center.regenerate");
      let icon = if AppConfig::icons_enabled() {
        icons::SYNC
      } else {
        ""
      };
      let button = if self.busy() {
        ActionButton::secondary(icon, label, "")
      } else {
        ActionButton::danger(icon, label, "")
      };
      let selected = if self.busy() {
        usize::MAX
      } else {
        self.actions.cursor().unwrap_or(usize::MAX)
      };
      draw_aligned_transparent(
        frame,
        inner,
        &[button],
        selected,
        &self.theme,
        Alignment::Left,
      );
      list
    } else {
      body
    };
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
    _body: ratatui::layout::Rect,
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
