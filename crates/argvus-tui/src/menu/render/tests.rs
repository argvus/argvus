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
