//! Rendering of the single vertical menu list.
//!
//! Every row uses the same columns so pages line up with icons on and off:
//!
//! ```text
//!  > [x] 󰂵 Blur ............................ 40% ›
//!  │ │ │ │   │ │                                 │   └ Submenu marker
//!  │ │ │ │   │ └ label                           └ detail (right-aligned)
//!  │ │ │ │   └ item icon (column dropped when icons are off)
//!  │ │ │ └ state marker: `[x]`/`[ ]` (Toggle) or `●` (Choice)
//!  │ │ └ default mark `●` (column only when a row is marked)
//!  │ └ cursor
//! ```

use argvus_theme::Theme;
use ratatui::{
  Frame,
  layout::{Margin, Rect},
  style::{Color, Modifier, Style},
  text::{Line, Span},
  widgets::Paragraph,
};

use super::row::{Emphasis, Row, RowKind};
use super::state::MenuState;
use crate::icons::ICON_TEXT_GAP;
use crate::text::{display_width, ellipsize};

const CURSOR: &str = ">";
const TOGGLE_ON: &str = "[x]";
const TOGGLE_OFF: &str = "[ ]";
const CHOICE_CURRENT: &str = "●";
const DEFAULT_MARK: &str = "●";
const SUBMENU_MARKER: &str = "›";
const SEPARATOR_GLYPH: &str = "─";
const COLLAPSED_GLYPH: &str = "▸";
const EXPANDED_GLYPH: &str = "▾";

/// Presentation options that come from the application configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MenuStyle {
  /// Mirror of the Control Center "Icons" setting. When off, the icon
  /// column disappears from every row so labels stay aligned.
  pub icons: bool,
}

/// Column widths shared by all rows of one list.
#[derive(Debug, Clone, Copy)]
struct Columns {
  marker: usize,
  icon: bool,
  mark: bool,
}

impl Columns {
  fn for_rows<Id>(rows: &[Row<Id>], style: MenuStyle) -> Self {
    let marker = rows
      .iter()
      .map(|row| match row.kind() {
        RowKind::Toggle { .. } => display_width(TOGGLE_ON),
        RowKind::Choice { .. } => display_width(CHOICE_CURRENT),
        _ => 0,
      })
      .max()
      .unwrap_or(0);
    let icon = style.icons && rows.iter().any(|row| row.icon_glyph().is_some());
    let mark = rows.iter().any(Row::is_marked);
    Self { marker, icon, mark }
  }
}

/// Draws the list and updates the scroll position so the cursor stays
/// visible. Rendering performs no I/O and changes no selection.
pub fn draw_menu<Id>(
  frame: &mut Frame,
  area: Rect,
  theme: &Theme,
  rows: &[Row<Id>],
  state: &mut MenuState,
  style: MenuStyle,
) {
  let area = area.inner(Margin::new(1, 0));
  let viewport = usize::from(area.height);
  if viewport == 0 || area.width == 0 {
    return;
  }
  state.fit_viewport(rows, viewport);
  let columns = Columns::for_rows(rows, style);
  let width = usize::from(area.width);
  let lines = rows
    .iter()
    .enumerate()
    .skip(state.offset())
    .take(viewport)
    .map(|(index, row)| {
      let is_selected = state.selected_index() == Some(index);
      row_line(row, is_selected, columns, width, theme)
    })
    .collect::<Vec<_>>();
  frame.render_widget(
    Paragraph::new(lines).style(Style::new().bg(theme.background)),
    area,
  );
}

fn row_line<Id>(
  row: &Row<Id>,
  is_selected: bool,
  columns: Columns,
  width: usize,
  theme: &Theme,
) -> Line<'static> {
  if let RowKind::CollapsibleSection { collapsed } = row.kind() {
    return collapsible_line(row, collapsed, is_selected, width, theme);
  }
  if row.is_section() {
    return section_line(row, width, theme);
  }
  if row.kind() == RowKind::Separator {
    return Line::from(Span::styled(
      SEPARATOR_GLYPH.repeat(width),
      Style::new().fg(theme.border).bg(theme.background),
    ));
  }

  let base = row_style(row, is_selected, theme);
  let cursor = if is_selected { CURSOR } else { " " };
  // The mark column is dropped entirely when no row of the list is marked.
  let mark = if columns.mark {
    mark_cell(row.is_marked())
  } else {
    String::new()
  };
  let marker = marker_cell(row.kind(), columns.marker);
  let icon = if columns.icon {
    // Rows without an icon keep the column blank to stay aligned.
    format!("{}{ICON_TEXT_GAP}", row.icon_glyph().unwrap_or(" "))
  } else {
    String::new()
  };
  let prefix = format!(" {cursor} {mark}{marker}{icon}");

  let suffix = if row.kind() == RowKind::Submenu {
    format!(" {SUBMENU_MARKER}")
  } else {
    String::new()
  };
  let detail = row.detail_text().unwrap_or_default();
  let available = width.saturating_sub(display_width(&prefix) + display_width(&suffix));
  let (label, detail) = fit_label_and_detail(row.label(), detail, available);
  let padding = available.saturating_sub(display_width(&label) + display_width(&detail));

  // Info labels are dimmed while their values keep the regular foreground,
  // so read-only pages stay legible without looking selectable.
  let detail_style = if row.kind() == RowKind::Info && !is_selected {
    Style::new().fg(theme.foreground).bg(theme.background)
  } else {
    base
  };
  Line::from(vec![
    Span::styled(prefix, base),
    Span::styled(label, base),
    Span::styled(" ".repeat(padding), base),
    Span::styled(detail, detail_style),
    Span::styled(suffix, base),
  ])
  .style(base)
}

/// Section title: accent and bold, starting in the cursor column so the
/// section's rows read as indented under it. The optional detail is dimmed
/// and right-aligned like an Info value.
fn section_line<Id>(row: &Row<Id>, width: usize, theme: &Theme) -> Line<'static> {
  let title_style = Style::new()
    .fg(theme.accent)
    .bg(theme.background)
    .add_modifier(Modifier::BOLD);
  let detail_style = Style::new().fg(theme.muted).bg(theme.background);
  let prefix = " ";
  let available = width.saturating_sub(display_width(prefix));
  let detail = row.detail_text().unwrap_or_default();
  let (label, detail) = fit_label_and_detail(row.label(), detail, available);
  let padding = available.saturating_sub(display_width(&label) + display_width(&detail));
  Line::from(vec![
    Span::styled(prefix, title_style),
    Span::styled(label, title_style),
    Span::styled(" ".repeat(padding), detail_style),
    Span::styled(detail, detail_style),
  ])
}

/// A collapsible section title: the cursor, a chevron that shows the state, and
/// the title in the accent color. It keeps the highlight of a selected row.
fn collapsible_line<Id>(
  row: &Row<Id>,
  collapsed: bool,
  is_selected: bool,
  width: usize,
  theme: &Theme,
) -> Line<'static> {
  let chevron = if collapsed {
    COLLAPSED_GLYPH
  } else {
    EXPANDED_GLYPH
  };
  let cursor = if is_selected { CURSOR } else { " " };
  // A selected row keeps the selection colors: the accent would disappear on the
  // selection background when the two colors are the same.
  let style = if is_selected {
    row_style(row, true, theme)
  } else {
    Style::new()
      .fg(theme.accent)
      .bg(theme.background)
      .add_modifier(Modifier::BOLD)
  };
  let text = ellipsize(&format!("{cursor} {chevron} {}", row.label()), width);
  let padding = width.saturating_sub(display_width(&text));
  Line::from(vec![
    Span::styled(text, style),
    Span::styled(" ".repeat(padding), style),
  ])
}

/// The default mark: `● ` on a marked row, blank otherwise.
fn mark_cell(marked: bool) -> String {
  if marked {
    format!("{DEFAULT_MARK} ")
  } else {
    "  ".to_owned()
  }
}

fn marker_cell(kind: RowKind, width: usize) -> String {
  if width == 0 {
    return String::new();
  }
  let marker = match kind {
    RowKind::Toggle { on: true } => TOGGLE_ON,
    RowKind::Toggle { on: false } => TOGGLE_OFF,
    RowKind::Choice { current: true } => CHOICE_CURRENT,
    _ => "",
  };
  let fill = width.saturating_sub(display_width(marker));
  format!("{marker}{} ", " ".repeat(fill))
}

fn row_style<Id>(row: &Row<Id>, is_selected: bool, theme: &Theme) -> Style {
  if is_selected {
    return Style::new()
      .fg(theme.selected_foreground)
      .bg(theme.selected_background)
      .add_modifier(Modifier::BOLD);
  }
  let foreground: Color = if row.kind() == RowKind::Info || !row.is_enabled() {
    theme.muted
  } else {
    match row.emphasis_kind() {
      Emphasis::Normal => theme.foreground,
      Emphasis::Primary => theme.accent,
      Emphasis::Danger => theme.error,
    }
  };
  Style::new().fg(foreground).bg(theme.background)
}

/// Fits label and detail into `available` cells with at least one space
/// between them. The label keeps priority up to half of the width; the
/// detail is shortened first, then the label.
fn fit_label_and_detail(label: &str, detail: &str, available: usize) -> (String, String) {
  let label_width = display_width(label);
  let detail_width = display_width(detail);
  if detail_width == 0 {
    return (ellipsize(label, available), String::new());
  }
  if label_width + 1 + detail_width <= available {
    return (label.to_owned(), detail.to_owned());
  }
  let label_budget = label_width.min(available / 2);
  let detail_budget = available.saturating_sub(label_budget + 1);
  let detail = ellipsize(detail, detail_budget);
  let label_budget = available.saturating_sub(display_width(&detail) + 1);
  (ellipsize(label, label_budget), detail)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::icons;
  use ratatui::{Terminal, backend::TestBackend};

  fn render(
    rows: &[Row<u8>],
    state: &mut MenuState,
    width: u16,
    height: u16,
    icons_on: bool,
  ) -> Vec<String> {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    let theme = Theme::load();
    terminal
      .draw(|frame| {
        draw_menu(
          frame,
          Rect::new(0, 0, width, height),
          &theme,
          rows,
          state,
          MenuStyle { icons: icons_on },
        );
      })
      .unwrap();
    let buffer = terminal.backend().buffer().clone();
    buffer
      .content
      .chunks(usize::from(width))
      .map(|line| line.iter().map(|cell| cell.symbol()).collect::<String>())
      .collect()
  }

  fn sample_rows() -> Vec<Row<u8>> {
    vec![
      Row::info("Kernel", "6.18"),
      Row::toggle(1, "Blur", true).icon(icons::BLUR),
      Row::choice(2, "Top", true).icon(icons::ARROW_UP),
      Row::submenu(3, "Themes")
        .icon(icons::PALETTE)
        .detail("Nord"),
      Row::action(4, "Refresh"),
    ]
  }

  #[test]
  fn a_selected_collapsible_section_is_drawn_with_the_selection_colors() {
    let rows: Vec<Row<u8>> = vec![
      Row::collapsible_section(0, "Generate key", true),
      Row::action(1, "Go"),
    ];
    let theme = Theme::load();
    let mut terminal = Terminal::new(TestBackend::new(40, 4)).unwrap();
    let mut state = MenuState::default();
    terminal
      .draw(|frame| {
        draw_menu(
          frame,
          Rect::new(0, 0, 40, 4),
          &theme,
          &rows,
          &mut state,
          MenuStyle { icons: false },
        );
      })
      .unwrap();
    let buffer = terminal.backend().buffer().clone();
    let title = (0..40)
      .map(|x| buffer.cell((x, 0)).expect("cell inside the buffer"))
      .find(|cell| cell.symbol() == "G")
      .expect("the section title is drawn on the first row");
    assert_eq!(title.fg, theme.selected_foreground);
    assert_eq!(title.bg, theme.selected_background);
  }

  #[test]
  fn info_rows_never_show_the_cursor() {
    let rows = sample_rows();
    let mut state = MenuState::default();
    let screen = render(&rows, &mut state, 50, 6, true);
    assert!(!screen[0].contains(CURSOR), "{:?}", screen[0]);
    assert!(
      screen[1].contains(&format!("{CURSOR} {TOGGLE_ON}")),
      "{:?}",
      screen[1]
    );
  }

  #[test]
  fn markers_icons_and_submenu_suffix_are_drawn_in_their_columns() {
    let rows = sample_rows();
    let mut state = MenuState::default();
    let screen = render(&rows, &mut state, 50, 6, true);
    assert!(screen[1].contains(&format!("{TOGGLE_ON} {} Blur", icons::BLUR)));
    assert!(screen[2].contains(&format!("{CHOICE_CURRENT}   {} Top", icons::ARROW_UP)));
    assert!(
      screen[3]
        .trim_end()
        .ends_with(&format!("Nord {SUBMENU_MARKER}"))
    );
    assert!(
      screen[0].trim_end().ends_with("6.18"),
      "Info value is right-aligned"
    );
  }

  #[test]
  fn labels_stay_aligned_with_icons_on_and_off() {
    let rows = sample_rows();
    for icons_on in [true, false] {
      let mut state = MenuState::default();
      let screen = render(&rows, &mut state, 50, 6, icons_on);
      let label_column =
        |line: &str, label: &str| line.find(label).map(|byte| display_width(&line[..byte]));
      let blur = label_column(&screen[1], "Blur");
      assert_eq!(blur, label_column(&screen[2], "Top"), "icons_on={icons_on}");
      assert_eq!(
        blur,
        label_column(&screen[4], "Refresh"),
        "icons_on={icons_on}"
      );
      assert_eq!(
        screen.iter().any(|line| line.contains(icons::BLUR)),
        icons_on
      );
    }
  }

  #[test]
  fn long_rows_are_ellipsized_within_the_width() {
    let rows = vec![
      Row::submenu(1u8, "A very long submenu label that cannot fit")
        .detail("an equally long detail value"),
    ];
    let mut state = MenuState::default();
    let screen = render(&rows, &mut state, 40, 1, false);
    assert!(screen[0].contains('…'), "{:?}", screen[0]);
    assert!(
      screen[0].trim_end().ends_with(SUBMENU_MARKER),
      "{:?}",
      screen[0]
    );
  }

  #[test]
  fn scrolling_keeps_the_selected_row_visible() {
    let rows: Vec<Row<u8>> = (0..10u8)
      .map(|id| Row::action(id, format!("row{id}")))
      .collect();
    let mut state = MenuState::default();
    state.normalize(&rows);
    for _ in 0..8 {
      state.handle(crossterm::event::KeyCode::Down, &rows, 3);
    }
    let screen = render(&rows, &mut state, 40, 3, false);
    assert!(
      screen.iter().any(|line| line.contains("> row8")),
      "{screen:?}"
    );
    assert!(!screen.iter().any(|line| line.contains("row0")));
  }

  #[test]
  fn section_titles_are_drawn_without_decoration_and_skip_the_cursor() {
    let rows = vec![
      Row::section("Account").detail("ready"),
      Row::action(1u8, "Save"),
    ];
    let mut state = MenuState::default();
    let screen = render(&rows, &mut state, 40, 2, false);
    assert!(screen[0].starts_with("  Account"), "{:?}", screen[0]);
    assert!(!screen[0].contains("--") && !screen[0].contains(SEPARATOR_GLYPH));
    assert!(screen[0].trim_end().ends_with("ready"), "{:?}", screen[0]);
    assert!(
      screen[1].contains(&format!("{CURSOR} Save")),
      "{:?}",
      screen[1]
    );
  }

  #[test]
  fn default_mark_sits_before_the_state_marker_and_keeps_labels_aligned() {
    let rows = vec![
      Row::toggle(1u8, "br", true).marked(true),
      Row::toggle(2u8, "us", true),
      Row::toggle(3u8, "de", false),
    ];
    for icons_on in [true, false] {
      let mut state = MenuState::default();
      let screen = render(&rows, &mut state, 50, 3, icons_on);
      assert!(screen[0].contains(&format!("{DEFAULT_MARK} {TOGGLE_ON} br")));
      assert!(
        screen[1].contains(&format!("  {TOGGLE_ON} us")),
        "{:?}",
        screen[1]
      );
      let label_column =
        |line: &str, label: &str| line.find(label).map(|byte| display_width(&line[..byte]));
      let br = label_column(&screen[0], "br");
      assert_eq!(br, label_column(&screen[1], "us"), "icons_on={icons_on}");
      assert_eq!(br, label_column(&screen[2], "de"), "icons_on={icons_on}");
    }
  }

  #[test]
  fn mark_column_is_absent_when_no_row_is_marked() {
    let rows = vec![Row::toggle(1u8, "us", true)];
    let mut state = MenuState::default();
    let screen = render(&rows, &mut state, 40, 1, false);
    assert!(screen[0].starts_with(" "), "{:?}", screen[0]);
    assert!(
      screen[0].contains(&format!(" {TOGGLE_ON} us")),
      "{:?}",
      screen[0]
    );
    assert!(!screen[0].contains(DEFAULT_MARK));
  }

  #[test]
  fn fits_80_columns_without_overflow() {
    let rows = sample_rows();
    let mut state = MenuState::default();
    let screen = render(&rows, &mut state, 80, 6, true);
    let submenu = &screen[3];
    assert!(
      submenu.contains(&format!("Nord {SUBMENU_MARKER}")),
      "{submenu:?}"
    );
    assert!(!submenu.contains('…'), "short rows are not truncated");
    assert!(
      submenu.ends_with(&format!("{SUBMENU_MARKER} ")),
      "right margin stays free: {submenu:?}"
    );
  }
}
