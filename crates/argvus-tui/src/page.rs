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

use crate::chrome::{Header, draw_footer, draw_header};
use crate::components::StatusMessage;

/// The common shell used by every Control Center domain page.
///
/// The header is `<root> > <breadcrumb>`. The root comes from the catalog key
/// `control_center.breadcrumb.root`, resolved in `lang`; `breadcrumb` is the
/// already translated path below the root, such as `Hardware & Connectivity > Audio`.
pub fn shell(
  frame: &mut Frame,
  area: Rect,
  theme: &Theme,
  lang: Lang,
  breadcrumb: &str,
  hints: &str,
) -> Rect {
  Block::new()
    .borders(Borders::ALL)
    .border_style(Style::new().fg(theme.border_active))
    .style(Style::new().bg(theme.background).fg(theme.foreground))
    .render(area, frame.buffer_mut());
  let inner = area.inner(Margin::new(1, 1));
  let rows = Layout::vertical([
    Constraint::Length(1),
    Constraint::Min(1),
    Constraint::Length(2),
  ])
  .split(inner);
  let title = format!(
    "{} > {breadcrumb}",
    tr(lang, "control_center.breadcrumb.root")
  );
  draw_header(
    frame,
    rows[0],
    theme,
    Header {
      title: &title,
      version: None,
      version_label: "",
    },
  );
  draw_footer(frame, rows[2], theme, None, hints);
  rows[1]
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
  /// The shell header root is a catalog key, so it must exist and differ
  /// between en-US and pt-BR in the workspace `resources/locales` directory.
  fn shell_root_is_translated_in_en_us_and_pt_br() {
    let locales = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../resources/locales");
    let root = |locale: &str| {
      let path = locales.join(locale).join("control-center.json");
      let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
      let catalog: serde_json::Value = serde_json::from_str(&content).unwrap();
      catalog
        .get("control_center.breadcrumb.root")
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned)
        .unwrap_or_else(|| panic!("{locale} misses control_center.breadcrumb.root"))
    };
    assert_eq!(root("en-US"), "Control Center");
    assert_eq!(root("pt-BR"), "Central de Controle");
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
}
