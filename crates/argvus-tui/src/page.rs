//! Shared page navigation primitives.
use argvus_i18n::{Lang, tr};
use argvus_theme::Theme;
use crossterm::event::KeyCode;
use ratatui::{
  Frame,
  layout::{Constraint, Layout, Margin, Rect},
  style::{Modifier, Style},
  text::{Line, Span},
  widgets::{Block, Borders, Paragraph, Widget},
};

use crate::chrome::{draw_control_center_header, draw_footer};
use crate::components::StatusMessage;

/// The common shell used by every Control Center domain page.
///
/// The branded title sits above the translated path from the Home card.
pub fn shell(
  frame: &mut Frame,
  area: Rect,
  theme: &Theme,
  lang: Lang,
  breadcrumb: &str,
  hints: &str,
) -> Rect {
  shell_with_status(
    frame,
    area,
    theme,
    lang,
    breadcrumb,
    hints,
    PageStatus::default(),
  )
}

/// Status content shared by the page footer and administrator warning.
#[derive(Default)]
pub struct PageStatus<'a> {
  pub message: Option<&'a StatusMessage>,
  pub admin_required: bool,
}

/// Draws the shell with an optional status below the navigation hints.
pub fn shell_with_status(
  frame: &mut Frame,
  area: Rect,
  theme: &Theme,
  lang: Lang,
  breadcrumb: &str,
  hints: &str,
  status: PageStatus<'_>,
) -> Rect {
  let PageStatus {
    message,
    admin_required,
  } = status;
  Block::new()
    .borders(Borders::ALL)
    .border_style(Style::new().fg(theme.border_active))
    .style(Style::new().bg(theme.background).fg(theme.foreground))
    .render(area, frame.buffer_mut());
  let inner = area.inner(Margin::new(1, 1));
  let status_height = message.map_or(0, |message| {
    crate::components::status_line_count(message, inner.width).min(usize::from(inner.height)) as u16
  });
  let rows = Layout::vertical([
    Constraint::Length(2),
    Constraint::Min(1),
    Constraint::Length(1),
    Constraint::Length(status_height),
  ])
  .split(inner);
  draw_control_center_header(frame, rows[0], theme, breadcrumb, None);
  draw_footer(frame, rows[2], theme, None, hints);
  if let Some(message) = message {
    crate::components::draw_status(frame, rows[3], theme, message);
    for y in rows[3].y..rows[3].bottom() {
      for x in rows[3].x..rows[3].right() {
        frame.buffer_mut()[(x, y)].set_bg(theme.background);
      }
    }
  }
  admin_warning_top(frame, rows[1], theme, lang, admin_required)
}

/// Reserves the bottom of a page's body for a persistent administration notice.
/// Call before drawing content or overlays; the returned body excludes the notice
/// and leaves the shell's status and keyboard footer intact.
pub fn admin_warning(
  frame: &mut Frame,
  body: Rect,
  theme: &Theme,
  lang: Lang,
  required: bool,
) -> Rect {
  admin_warning_at(frame, body, theme, lang, required, false)
}

/// Reserves the top of a page's body for the administration notice.
pub fn admin_warning_top(
  frame: &mut Frame,
  body: Rect,
  theme: &Theme,
  lang: Lang,
  required: bool,
) -> Rect {
  admin_warning_at(frame, body, theme, lang, required, true)
}

fn admin_warning_at(
  frame: &mut Frame,
  body: Rect,
  theme: &Theme,
  lang: Lang,
  required: bool,
  at_top: bool,
) -> Rect {
  if !required || body.is_empty() {
    return body;
  }
  let message = StatusMessage {
    kind: crate::components::StatusKind::Warning,
    text: tr(lang, "control_center.admin_privileges_required").to_owned(),
  };
  let height = crate::components::status_line_count(&message, body.width.saturating_sub(2))
    .saturating_add(2)
    .min(usize::from(body.height)) as u16;
  let content = Rect::new(
    body.x,
    if at_top {
      body.y.saturating_add(height)
    } else {
      body.y
    },
    body.width,
    body.height.saturating_sub(height),
  );
  let warning_y = if at_top { body.y } else { content.bottom() };
  let warning_area = Rect::new(body.x, warning_y, body.width, height);
  let block = Block::bordered().border_style(Style::new().fg(theme.border_active));
  let inner = block.inner(warning_area);
  frame.render_widget(block, warning_area);
  crate::components::draw_status(frame, inner, theme, &message);
  content
}

pub fn list(frame: &mut Frame, area: Rect, theme: &Theme, rows: &[String], selected: usize) {
  let no_selection = selected == usize::MAX;
  let len = rows.len();
  let height = (area.height as usize).max(1);
  let selected = if no_selection || len == 0 {
    0
  } else {
    selected.min(len - 1)
  };
  let scroll = if no_selection || len <= height {
    0
  } else {
    selected.saturating_sub(height - 1).min(len - height)
  };
  // Only the viewport is built: a long list costs the same per frame as a short one.
  let lines = rows
    .iter()
    .enumerate()
    .skip(scroll)
    .take(height)
    .map(|(index, row)| {
      let chosen = !no_selection && index == selected;
      let style = if chosen {
        Style::new()
          .fg(theme.selected_foreground)
          .bg(theme.selected_background)
          .add_modifier(Modifier::BOLD)
      } else {
        Style::new().fg(theme.foreground).bg(theme.background)
      };
      Line::from(Span::styled(
        format!(" {} {}", if chosen { ">" } else { " " }, row),
        style,
      ))
      .style(style)
    })
    .collect::<Vec<_>>();
  frame.render_widget(
    Paragraph::new(lines).style(Style::new().bg(theme.background)),
    area.inner(Margin::new(1, 0)),
  );
}

/// Informational rows are rendered without entering the keyboard focus model.
pub fn readonly(frame: &mut Frame, area: Rect, theme: &Theme, rows: &[Line<'static>]) {
  let lines = rows
    .iter()
    .map(|line| {
      line
        .clone()
        .style(Style::new().fg(theme.foreground).bg(theme.background))
    })
    .collect::<Vec<_>>();
  frame.render_widget(
    Paragraph::new(lines).style(Style::new().bg(theme.background)),
    area.inner(Margin::new(1, 0)),
  );
}

pub fn status(frame: &mut Frame, outer: Rect, theme: &Theme, message: &StatusMessage) {
  let inner = outer.inner(Margin::new(1, 1));
  let lines = crate::components::status_line_count(message, inner.width).max(1);
  let height = (lines as u16).min(inner.height);
  let y = inner.y + inner.height.saturating_sub(height);
  crate::components::draw_status(
    frame,
    Rect::new(inner.x, y, inner.width, height),
    theme,
    message,
  );
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Selection {
  pub index: usize,
}
impl Selection {
  pub fn normalize(&mut self, len: usize) {
    self.index = self.index.min(len.saturating_sub(1));
  }
  pub fn handle(&mut self, key: KeyCode, len: usize, page: usize) -> bool {
    let old = self.index;
    match key {
      KeyCode::Up | KeyCode::Char('k') => self.index = self.index.saturating_sub(1),
      KeyCode::Down | KeyCode::Char('j') => self.index = self.index.saturating_add(1),
      KeyCode::Home => self.index = 0,
      KeyCode::End => self.index = len.saturating_sub(1),
      KeyCode::PageUp => self.index = self.index.saturating_sub(page.max(1)),
      KeyCode::PageDown => {
        self.index = self
          .index
          .saturating_add(page.max(1))
          .min(len.saturating_sub(1))
      }
      _ => return false,
    }
    self.normalize(len);
    old != self.index
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use ratatui::{Terminal, backend::TestBackend};

  #[test]
  fn shell_draws_title_and_path_on_separate_rows() {
    let mut terminal = Terminal::new(TestBackend::new(80, 12)).unwrap();
    terminal
      .draw(|frame| {
        let area = frame.area();
        shell(
          frame,
          area,
          &Theme::load(),
          Lang::for_locale("en-US"),
          "Desktop & Input > Hyprland",
          "",
        );
      })
      .unwrap();
    let buffer = terminal.backend().buffer();
    let row = |y| (0..80).map(|x| buffer[(x, y)].symbol()).collect::<String>();
    assert!(row(1).contains("ARGVUS  Control Center"));
    assert_eq!(
      row(2).trim_matches([' ', '│']),
      "Desktop & Input > Hyprland"
    );
  }

  #[test]
  fn selection_is_bounded_and_supports_navigation() {
    let mut s = Selection { index: 99 };
    s.normalize(3);
    assert_eq!(s.index, 2);
    s.handle(KeyCode::Home, 3, 2);
    assert_eq!(s.index, 0);
    s.handle(KeyCode::PageDown, 3, 2);
    assert_eq!(s.index, 2);
    s.handle(KeyCode::Down, 0, 2);
    assert_eq!(s.index, 0);
  }

  fn rendered_rows(selected: usize, height: u16, count: usize) -> String {
    let mut terminal = Terminal::new(TestBackend::new(40, height)).unwrap();
    let rows: Vec<String> = (0..count).map(|i| format!("row{i}")).collect();
    terminal
      .draw(|frame| {
        let theme = Theme::load();
        let area = ratatui::layout::Rect::new(0, 0, 40, height);
        list(frame, area, &theme, &rows, selected);
      })
      .unwrap();
    terminal
      .backend()
      .buffer()
      .content
      .iter()
      .map(|cell| cell.symbol())
      .collect()
  }

  #[test]
  fn list_scrolls_to_keep_selection_visible() {
    let visible = rendered_rows(0, 3, 10);
    assert!(visible.contains("row0"));
    assert!(visible.contains("row1"));
    assert!(!visible.contains("row4"));

    let deep = rendered_rows(8, 3, 10);
    assert!(deep.contains("row6"));
    assert!(deep.contains("row8"));
    assert!(!deep.contains("row0"));

    let bottom = rendered_rows(9, 3, 10);
    assert!(bottom.contains("row8"));
    assert!(!bottom.contains("row0"));
  }

  #[test]
  fn list_without_selection_stays_top_aligned() {
    let text = rendered_rows(usize::MAX, 3, 10);
    assert!(text.contains("row0"));
    assert!(text.contains("row1"));
    assert!(!text.contains("row5"));
  }
  #[test]
  fn admin_warning_reserves_wrapped_lines_above_footer() {
    for locale in argvus_i18n::installed_locales() {
      let locale = locale.locale;
      for (width, height) in [(60, 18), (100, 24)] {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        let theme = Theme::load();
        let lang = Lang::for_locale(&locale);
        terminal
          .draw(|frame| {
            let body = shell(frame, frame.area(), &theme, lang, "Test", "Escape Back");
            let content = admin_warning(frame, body, &theme, lang, true);
            assert!(content.bottom() < body.bottom());
            assert_eq!(content.y, body.y);
            frame.render_widget(Paragraph::new("content"), content);
          })
          .unwrap();
        let lines: Vec<String> = terminal
          .backend()
          .buffer()
          .content
          .chunks(width as usize)
          .map(|row| row.iter().map(|cell| cell.symbol()).collect())
          .collect();
        let warning = lines.iter().position(|row| row.contains("[WARN]")).unwrap();
        let footer = lines
          .iter()
          .position(|row| row.contains("Escape Back"))
          .unwrap();
        assert!(warning < footer);
        let rendered = lines[warning..footer].join("");
        assert!(!rendered.contains('…'));
        assert!(!rendered.contains("control_center."));
        assert_eq!(
          terminal.backend().buffer()[(2, warning as u16)].fg,
          theme.warning
        );
      }
    }
  }

  #[test]
  fn personal_pages_keep_their_full_body() {
    let mut terminal = Terminal::new(TestBackend::new(60, 18)).unwrap();
    terminal
      .draw(|frame| {
        let body = frame.area();
        assert_eq!(
          admin_warning(
            frame,
            body,
            &Theme::load(),
            Lang::for_locale("en-US"),
            false
          ),
          body
        );
        assert_eq!(
          admin_warning(
            frame,
            Rect::default(),
            &Theme::load(),
            Lang::for_locale("en-US"),
            true
          ),
          Rect::default()
        );
      })
      .unwrap();
    assert!(
      !terminal
        .backend()
        .buffer()
        .content
        .iter()
        .map(|cell| cell.symbol())
        .collect::<String>()
        .contains("[WARN]")
    );
  }
}
