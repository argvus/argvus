//! Popup and editor rendering of the Settings pages.
use argvus_tui::confirm::{ConfirmDialog, draw_confirm};
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Clear, Paragraph};

use crate::app::App;
use crate::i18n::tr;

pub fn draw_error(frame: &mut Frame, area: Rect, app: &App, message: &str) {
  let width = area.width.saturating_sub(8).clamp(30, 72).min(area.width);
  let message = argvus_control_center_core::sanitize::terminal_text(message);
  let paragraph = Paragraph::new(message)
    .alignment(Alignment::Center)
    .wrap(ratatui::widgets::Wrap { trim: true });
  let rows = paragraph.line_count(width.saturating_sub(2));
  // Borders and spacing add five rows. Keep the acknowledgement separate so
  // wrapping or a small terminal can never clip the key used to dismiss the error.
  let height = u16::try_from(rows.saturating_add(5))
    .unwrap_or(u16::MAX)
    .min(area.height);
  let popup = super::layout::centered(area, width, height);
  let block = Block::bordered()
    .title(tr(app.lang, "control_center.error_32d0ef"))
    .border_style(Style::new().fg(app.theme.error))
    .style(
      Style::new()
        .bg(app.theme.background)
        .fg(app.theme.foreground),
    );
  let inner = block.inner(popup);
  frame.render_widget(Clear, popup);
  frame.render_widget(block, popup);
  let areas = Layout::vertical([
    Constraint::Length(1),
    Constraint::Min(1),
    Constraint::Length(1),
    Constraint::Length(1),
  ])
  .split(inner);
  frame.render_widget(paragraph, areas[1]);
  frame.render_widget(
    Paragraph::new(tr(app.lang, "control_center.enter_ok"))
      .alignment(Alignment::Center)
      .style(
        Style::new()
          .fg(app.theme.accent)
          .add_modifier(Modifier::BOLD),
      ),
    areas[3],
  );
}

/// Draws the open confirmation (pending action or shortcut conflict) with
/// the single confirmation component.
pub fn draw_confirmation(frame: &mut Frame, area: Rect, app: &App) {
  let Some((title, message, confirm, danger)) = app.confirm_dialog() else {
    return;
  };
  let dialog = ConfirmDialog {
    title: &title,
    message: &message,
    confirm,
    cancel: if app.admin.delete_confirm {
      tr(app.lang, "control_center.no")
    } else {
      tr(app.lang, "control_center.cancel")
    },
    danger,
    deadline: None,
  };
  draw_confirm(frame, area, &app.theme, dialog, &app.confirm_focus);
}

pub fn draw_hostname_input(frame: &mut Frame, area: Rect, app: &App) {
  let width = area.width.saturating_sub(8).clamp(40, 52);
  let height = 7.min(area.height);
  let popup = super::layout::centered(area, width, height);
  frame.render_widget(Clear, popup);
  let title = tr(app.lang, "control_center.hostname");
  let content = vec![
    Line::from(tr(app.lang, "control_center.new_hostname")),
    Line::from(format!("{}_", app.hostname_input)),
    Line::from(tr(app.lang, "control_center.enter_apply_esc_cancel")),
  ];
  frame.render_widget(
    Paragraph::new(content).block(
      Block::bordered()
        .title(format!(" {title} "))
        .border_style(Style::new().fg(app.theme.border_active))
        .style(Style::new().bg(app.theme.background)),
    ),
    popup,
  );
}
