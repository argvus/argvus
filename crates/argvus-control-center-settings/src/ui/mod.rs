//! Implements module declarations for the `ui` subsystem in crate `argvus control center settings`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
pub mod footer;
pub mod header;
pub mod layout;
pub mod popup;
pub mod search;

use argvus_control_center_core::config::AppConfig;
use argvus_tui::action_buttons::{
  ActionButton, draw_aligned as draw_action_buttons, draw_aligned_transparent,
};
use argvus_tui::menu::{MenuStyle, Row, draw_menu};
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{
  Block, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Widget, Wrap,
};

use crate::app::{App, MIN_HEIGHT, MIN_WIDTH};
use crate::i18n::tr;
use crate::item::Item;
use crate::navigation::Page;

/// Renders `draw` while respecting the current domain state and semantic theme. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn draw(app: &mut App, frame: &mut Frame) {
  let area = frame.area();
  app.resize(area.width, area.height);
  Block::new()
    .borders(Borders::ALL)
    .border_style(Style::new().fg(app.theme.border_active))
    .style(
      Style::new()
        .bg(app.theme.background)
        .fg(app.theme.foreground),
    )
    .render(area, frame.buffer_mut());

  if app.too_small() {
    draw_too_small(app, frame, area);
    return;
  }

  if crate::administration::is_user_form(app.navigation.current().page) {
    draw_user_page(app, frame, area);
    return;
  }

  let areas = layout::areas(area);
  let rows = app.rows();
  let actions = app.action_rows();
  // Pages with a top search reserve a box above the list; the bottom line
  // then only shows status messages.
  let (search_area, body) = if app.search_on_top() {
    let split = Layout::default()
      .direction(Direction::Vertical)
      .constraints([Constraint::Length(3), Constraint::Min(0)])
      .split(areas.body);
    (Some(split[0]), split[1])
  } else {
    (None, areas.body)
  };
  frame.render_widget(Clear, areas.body);
  header::draw(frame, areas.header, app);
  if let Some(search_area) = search_area {
    search::draw_box(frame, search_area, app);
  }
  // A page with an action bar keeps it fixed under the list, so its buttons
  // are visible without scrolling; the list gets the space above it.
  let (list_area, action_area) = if actions.is_empty() {
    (body, None)
  } else {
    let split = Layout::default()
      .direction(Direction::Vertical)
      .constraints([Constraint::Min(3), Constraint::Length(3)])
      .split(body);
    (split[0], Some(split[1]))
  };
  let list_area = match action_area {
    Some(_) => {
      let list_block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Plain)
        .border_style(Style::new().fg(app.theme.border_active))
        .title(tr(app.lang, list_title_key(app.page())));
      let inner = list_block.inner(list_area);
      frame.render_widget(list_block, list_area);
      inner
    }
    None => list_area,
  };
  app.set_viewport(list_area.height as usize);
  // The key lists have a scrollbar: a key block is several rows tall, so the
  // end of the list is easy to miss. The bar takes one column of the list only
  // when the rows do not fit.
  let scrollable = matches!(app.page(), Page::SshKeys | Page::GpgKeyList)
    && rows.len() > list_area.height as usize;
  let (menu_area, scrollbar_area) = if scrollable {
    let split = Layout::default()
      .direction(Direction::Horizontal)
      .constraints([Constraint::Min(0), Constraint::Length(1)])
      .split(list_area);
    (split[0], Some(split[1]))
  } else {
    (list_area, None)
  };
  draw_menu(
    frame,
    menu_area,
    &app.theme,
    &rows,
    &mut app.navigation.current_mut().menu,
    MenuStyle {
      icons: AppConfig::icons_enabled(),
    },
  );
  if let Some(scrollbar_area) = scrollbar_area {
    // The widget reaches the bottom of its track only at position
    // `content_length - 1`, while the list's last scroll offset is
    // `rows - viewport`. Map the offset onto the widget's range, so the thumb
    // ends at the bottom exactly when the list shows its last row.
    let viewport = menu_area.height as usize;
    let position = scrollbar_position(app.navigation.current().menu.offset(), rows.len(), viewport);
    let mut scrollbar_state = ScrollbarState::new(rows.len())
      .viewport_content_length(viewport)
      .position(position);
    frame.render_stateful_widget(
      Scrollbar::new(ScrollbarOrientation::VerticalRight),
      scrollbar_area,
      &mut scrollbar_state,
    );
  }
  if let Some(action_area) = action_area {
    draw_action_bar(app, frame, action_area, &actions);
  }
  search::draw(frame, areas.message, app);
  footer::draw(frame, areas.footer, app);
  if app.hostname_editing {
    popup::draw_hostname_input(frame, area, app);
  }
  if let Some(editor) = &app.admin.editor {
    editor.draw(frame, area, app);
  }
  if let Some(message) = app.error_modal.as_deref() {
    popup::draw_error(frame, area, app, message);
  }
  popup::draw_confirmation(frame, area, app);
  draw_task_window(app, frame, area);
}

/// Translation key of the title of the bordered list block on the pages that
/// have an action bar: the list is named after what it chooses.
/// Maps a list's scroll offset onto the scrollbar's position range.
///
/// `ScrollbarState` draws the thumb at its end only when its position is
/// `content_length - 1`, but a list's last scroll offset is
/// `rows - viewport`. Scaling the offset by that ratio puts the thumb at the
/// bottom exactly when the list shows its last row.
fn scrollbar_position(offset: usize, rows: usize, viewport: usize) -> usize {
  let max_offset = rows.saturating_sub(viewport);
  if max_offset == 0 {
    return 0;
  }
  offset.min(max_offset) * rows.saturating_sub(1) / max_offset
}

fn list_title_key(page: Page) -> &'static str {
  match page {
    Page::TimeZone => "control_center.time_zone",
    Page::SshKeys => "control_center.ssh_keys",
    Page::GpgKeys => "control_center.gpg_keys",
    Page::GpgFormExpiration => "control_center.gpg_keys_expiration",
    Page::GpgFormPassphrase => "control_center.gpg_keys_passphrase_page",
    Page::GpgKeyList => "control_center.gpg_keys_list",
    Page::GpgKeyEdit => "control_center.gpg_keys_edit",
    Page::GpgEditExpiration => "control_center.gpg_keys_expiration",
    Page::GpgEditSign => "control_center.gpg_keys_edit_sign",
    Page::GpgEditRevoke => "control_center.gpg_keys_edit_revoke",
    _ => "control_center.layout",
  }
}

/// The page's action bar: a bordered block titled like the user page's
/// `Actions` block, holding one button per action row. Disabled buttons are
/// drawn muted, and only the focused button is highlighted.
fn draw_action_bar(app: &App, frame: &mut Frame, area: Rect, actions: &[Row<Item>]) {
  let block = Block::default()
    .borders(Borders::ALL)
    .border_type(ratatui::widgets::BorderType::Plain)
    .border_style(Style::new().fg(app.theme.border_active))
    .title(format!(
      "{} · {}",
      tr(app.lang, "control_center.actions"),
      tr(app.lang, "control_center.actions_hint")
    ));
  let inner = block.inner(area);
  frame.render_widget(block, area);

  let buttons: Vec<ActionButton> = actions
    .iter()
    .filter(|row| row.id().is_some())
    .map(|row| {
      let icon = if AppConfig::icons_enabled() {
        row.icon_glyph().unwrap_or("")
      } else {
        ""
      };
      if row.is_selectable() {
        ActionButton::primary(icon, row.label(), "")
      } else {
        ActionButton::secondary(icon, row.label(), "")
      }
    })
    .collect();
  let selected = app.action_cursor().unwrap_or(usize::MAX);
  draw_aligned_transparent(
    frame,
    inner,
    &buttons,
    selected,
    &app.theme,
    Alignment::Left,
  );
}

/// Renderiza a tela de usuário igual à demo.
fn draw_user_page(app: &mut App, frame: &mut Frame, area: Rect) {
  let inner = area.inner(ratatui::layout::Margin {
    horizontal: 1,
    vertical: 1,
  });

  let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
      Constraint::Length(3),
      Constraint::Min(8),
      Constraint::Length(4),
      Constraint::Length(1),
      Constraint::Length(1),
    ])
    .split(inner);

  // Title
  let title_text = app.breadcrumb();
  let title = Paragraph::new(title_text)
    .style(
      Style::default()
        .fg(app.theme.accent)
        .add_modifier(Modifier::BOLD),
    )
    .block(
      Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Plain)
        .border_style(Style::new().fg(app.theme.border_active)),
    );
  frame.render_widget(title, chunks[0]);

  // The rows before the "Actions" section are the info block; the rows
  // after it are the buttons below. Both come from the same single list.
  let all_rows = app.rows();
  let actions_at = all_rows
    .iter()
    .position(|row| row.is_section())
    .unwrap_or(all_rows.len());
  let (info_rows, action_rows) = all_rows.split_at(actions_at);

  app.set_viewport(chunks[1].height as usize);
  frame.render_widget(Clear, chunks[1]);

  let menu_block = Block::default()
    .borders(Borders::ALL)
    .border_type(ratatui::widgets::BorderType::Plain)
    .border_style(Style::new().fg(app.theme.border_active))
    .title(tr(app.lang, "control_center.user_info"));
  frame.render_widget(menu_block, chunks[1]);

  let menu_inner = Rect {
    x: chunks[1].x + 1,
    y: chunks[1].y + 1,
    width: chunks[1].width.saturating_sub(2),
    height: chunks[1].height.saturating_sub(2),
  };

  draw_menu(
    frame,
    menu_inner,
    &app.theme,
    info_rows,
    &mut app.navigation.current_mut().menu,
    MenuStyle {
      icons: AppConfig::icons_enabled(),
    },
  );

  // Action buttons are the action rows of the single list, so they run the
  // same activation as Enter on the row. The selected button follows Tab.
  let mut buttons = Vec::new();
  for row in action_rows.iter().filter(|row| row.id().is_some()) {
    let Some(&item) = row.id() else { continue };
    let label = row.label();
    // No shortcut is shown: the footer explains how the buttons run.
    let button = match row.kind() {
      argvus_tui::menu::RowKind::Destructive => ActionButton::danger("", label, ""),
      _ if item == Item::SaveUser => ActionButton::primary("", label, ""),
      _ => ActionButton::secondary("", label, ""),
    };
    buttons.push(button);
  }
  // Only the focused button is highlighted; none while the info list has focus.
  let selected_button = app.user_button_cursor().unwrap_or(usize::MAX);

  let buttons_block = Block::default()
    .borders(Borders::ALL)
    .border_type(ratatui::widgets::BorderType::Plain)
    .border_style(Style::new().fg(app.theme.border_active))
    .title(format!(
      "{} · {}",
      tr(app.lang, "control_center.actions"),
      tr(app.lang, "control_center.actions_hint")
    ));
  frame.render_widget(buttons_block, chunks[2]);

  // The buttons fill the block's inside; the bar wraps them to its width.
  let inner_buttons = Rect {
    x: chunks[2].x + 1,
    y: chunks[2].y + 1,
    width: chunks[2].width.saturating_sub(2),
    height: chunks[2].height.saturating_sub(2),
  };

  draw_action_buttons(
    frame,
    inner_buttons,
    &buttons,
    selected_button,
    &app.theme,
    Alignment::Center,
  );

  // Status
  // Footer: a warning while any edit waits for Save, otherwise the ready state.
  let status = if app.admin.any_user_draft_changed() {
    Paragraph::new(tr(app.lang, "control_center.unsaved_changes")).style(
      Style::default()
        .fg(app.theme.warning)
        .add_modifier(Modifier::BOLD),
    )
  } else {
    Paragraph::new(tr(app.lang, "control_center.ready")).style(Style::default().fg(app.theme.muted))
  };
  frame.render_widget(status, chunks[3]);

  // Navigation footer, always the last line of the screen.
  let nav = Paragraph::new(tr(app.lang, "control_center.user_page_keys"))
    .alignment(Alignment::Right)
    .style(Style::default().fg(app.theme.muted));
  frame.render_widget(nav, chunks[4]);

  // Popups/Dialogs
  if app.hostname_editing {
    popup::draw_hostname_input(frame, area, app);
  }
  if let Some(editor) = &app.admin.editor {
    editor.draw(frame, area, app);
  }
  if let Some(message) = app.error_modal.as_deref() {
    popup::draw_error(frame, area, app, message);
  }
  popup::draw_confirmation(frame, area, app);
  draw_task_window(app, frame, area);
}

/// Renders `draw_task_window` while respecting the current domain state and semantic theme. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn draw_task_window(app: &App, frame: &mut Frame, area: Rect) {
  if !app.task_open {
    return;
  }
  let Some(output) = app.task_live.as_ref().map(|live| live.output()) else {
    return;
  };
  let popup = layout::centered(
    area,
    crate::app::TASK_POPUP_WIDTH,
    crate::app::TASK_POPUP_HEIGHT,
  );
  let total = crate::app::task_wrapped_lines(&output);
  let shown_total = total.max(1);
  let current = (app.task_scroll as usize + 1).min(shown_total);
  frame.render_widget(Clear, popup);
  frame.render_widget(
    Paragraph::new(output.as_str())
      .block(
        Block::new()
          .borders(Borders::ALL)
          .title(Line::styled(
            format!(
              " {} · {} {} {} {} ",
              tr(app.lang, "control_center.process"),
              tr(app.lang, "control_center.line"),
              current,
              tr(app.lang, "control_center.of"),
              shown_total,
            ),
            Style::new().fg(app.theme.accent),
          ))
          .border_style(Style::new().fg(app.theme.border_active))
          .style(
            Style::new()
              .bg(app.theme.background)
              .fg(app.theme.foreground),
          )
          .title_bottom(Line::from(tr(
            app.lang,
            "control_center.scroll_pgup_pgdn_home_end_esc_close",
          ))),
      )
      .scroll((app.task_scroll, 0))
      .wrap(Wrap { trim: false }),
    popup,
  );
}

/// Renders `draw_too_small` while respecting the current domain state and semantic theme. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn draw_too_small(app: &App, frame: &mut Frame, area: Rect) {
  let width = area.width.min(52);
  let height = area.height.min(7);
  let popup = layout::centered(area, width, height);
  frame.render_widget(Clear, popup);
  let text = format!(
    "{}\n\n{}: {MIN_WIDTH}x{MIN_HEIGHT}\n{}: {}x{}",
    tr(app.lang, "control_center.terminal_window_is_too_small"),
    tr(app.lang, "control_center.minimum_size"),
    tr(app.lang, "control_center.current_size"),
    area.width,
    area.height
  );
  frame.render_widget(
    Paragraph::new(text).alignment(Alignment::Center).block(
      Block::bordered()
        .border_style(Style::new().fg(app.theme.error))
        .style(
          Style::new()
            .bg(app.theme.background)
            .fg(app.theme.foreground),
        ),
    ),
    popup,
  );
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::navigation::Page;
  use crossterm::event::{Event, KeyCode, KeyEvent};
  use ratatui::{Terminal, backend::TestBackend};
  use serde_json::json;

  #[test]
  fn scrollbar_thumb_reaches_the_bottom_on_the_last_row() {
    // 30 rows in a 10-row viewport: offsets run 0..=20, the thumb spans 0..=29.
    assert_eq!(scrollbar_position(0, 30, 10), 0);
    assert_eq!(scrollbar_position(20, 30, 10), 29);
    assert!(scrollbar_position(10, 30, 10) < 29);
  }

  #[test]
  fn scrollbar_stays_at_the_top_when_rows_fit() {
    assert_eq!(scrollbar_position(0, 5, 10), 0);
  }

  #[test]
  fn ssh_keys_search_opens_on_slash_and_filters_rows() {
    let mut app = App::with_context(
      Page::SshKeys,
      crate::i18n::Lang::for_locale("en-US"),
      crate::theme::Theme::load(),
    );
    app.ssh_keys = crate::system::ssh_keys::SshKeys {
      available: true,
      entries: vec![crate::system::ssh_keys::SshKeyEntry {
        name: "alpha".into(),
        key_type: "ed25519".into(),
        fingerprint: "SHA256:alpha".into(),
        comment: "alpha@host".into(),
        has_passphrase: false,
        agent: crate::system::ssh_keys::AgentState::NotLoaded,
      }],
    };
    app.begin_search();
    assert!(app.searching, "/ must open the search on the SSH keys page");
    app.push_search('z');
    assert!(
      app
        .rows()
        .iter()
        .all(|row| row.id() != Some(&Item::LoadSshKey(0))),
      "a query matching no key hides its block"
    );
  }

  /// Executes the `user_app` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn user_app() -> App {
    let mut app = App::with_context(
      Page::Main,
      crate::i18n::Lang::for_locale("en-US"),
      crate::theme::Theme::load(),
    );
    app.error_modal = None;
    app.navigation.push(Page::User);
    app.admin.accounts = json!({"actor_uid":1000, "shells":["/bin/bash", "/bin/zsh"], "groups":["users", "wheel"],
    "group_details":[
      {"name":"users","gid":1000,"members":[]},
      {"name":"wheel","gid":998,"members":["alice"]}
    ],
    "users":[
      {"user":"root", "uid":0, "gid":0, "name":"root", "shell":"/bin/bash", "primary_group":"root", "groups":[]},
      {"user":"alice", "uid":1000, "gid":1000, "name":"Alice", "shell":"/bin/bash", "primary_group":"users", "groups":["wheel"]}
    ]});
    app.admin.user = app.admin.accounts["users"][1].clone();
    app
  }

  /// Background of the first cell of the line that shows `label`.
  fn line_bg(terminal: &Terminal<TestBackend>, label: &str) -> Option<ratatui::style::Color> {
    let width = terminal.backend().buffer().area.width as usize;
    terminal
      .backend()
      .buffer()
      .content
      .chunks(width)
      .find(|line| {
        line
          .iter()
          .map(|cell| cell.symbol())
          .collect::<String>()
          .contains(label)
      })
      .map(|line| line[3].bg)
  }

  #[test]
  fn actions_are_rows_and_only_the_cursor_row_is_highlighted() {
    let mut terminal = Terminal::new(TestBackend::new(80, 40)).unwrap();
    let mut app = user_app();
    let selected_bg = app.theme.selected_background;
    terminal.draw(|frame| draw(&mut app, frame)).unwrap();
    // The list starts on its first option; the buttons are not highlighted.
    assert_eq!(line_bg(&terminal, "Avatar image"), Some(selected_bg));
    assert_ne!(line_bg(&terminal, "Save"), Some(selected_bg));

    // End moves the list to its last option, the info block's end.
    crate::event::handle(&mut app, Event::Key(KeyEvent::from(KeyCode::End)));
    terminal.draw(|frame| draw(&mut app, frame)).unwrap();
    assert_eq!(line_bg(&terminal, "Password"), Some(selected_bg));
    assert_ne!(line_bg(&terminal, "Avatar image"), Some(selected_bg));
  }

  #[test]
  /// Executes the `main_page_renders_brand_menu_and_footer` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn main_page_renders_brand_menu_and_footer() {
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    let mut app = App::new(Page::Main);
    app.error_modal = None;
    terminal.draw(|frame| draw(&mut app, frame)).unwrap();
    let rendered = terminal
      .backend()
      .buffer()
      .content
      .iter()
      .map(|cell| cell.symbol())
      .collect::<String>();
    assert!(rendered.contains("ARGVUS"));
    assert!(rendered.contains("Default Apps") || rendered.contains("Aplicativos padrão"));
    let quit = format!(
      "q {}",
      crate::i18n::tr(app.lang, "control_center.hint.quit")
    );
    assert!(app.footer().ends_with(&quit), "{}", app.footer());
  }

  #[test]
  /// Executes the `mouse_touchpad_page_renders_integrated_input_controls` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn mouse_touchpad_page_renders_integrated_input_controls() {
    let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();
    let mut app = App::new(Page::MouseTouchpad);
    app.error_modal = None;
    terminal.draw(|frame| draw(&mut app, frame)).unwrap();
    let rendered = terminal
      .backend()
      .buffer()
      .content
      .iter()
      .map(|cell| cell.symbol())
      .collect::<String>();
    let rows = app.rows();
    for row in rows
      .iter()
      .filter(|row| row.detail_text().is_some())
      .take(3)
    {
      assert!(
        rendered.contains(row.label()),
        "missing rendered row: {}",
        row.label()
      );
    }
  }

  #[test]
  fn reset_pages_render_a_restore_defaults_row_without_brackets() {
    let mut terminal = Terminal::new(TestBackend::new(80, 40)).unwrap();
    for page in [Page::DefaultApps, Page::Fonts] {
      let mut app = App::new(page);
      app.error_modal = None;
      terminal.draw(|frame| draw(&mut app, frame)).unwrap();
      let rendered = terminal
        .backend()
        .buffer()
        .content
        .iter()
        .map(|cell| cell.symbol())
        .collect::<String>();
      assert!(rendered.contains(crate::i18n::tr(app.lang, "control_center.reset_defaults")));
      assert!(!rendered.contains("[ "), "no button labels on {page:?}");
    }
  }

  #[test]
  /// Executes the `hostname_editing_renders_popup_with_typed_value` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn hostname_editing_renders_popup_with_typed_value() {
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    let mut app = App::new(Page::Hostname);
    app.error_modal = None;
    app.hostname_editing = true;
    app.hostname_input = "mybox".into();
    terminal.draw(|frame| draw(&mut app, frame)).unwrap();
    let rendered = terminal
      .backend()
      .buffer()
      .content
      .iter()
      .map(|cell| cell.symbol())
      .collect::<String>();
    assert!(rendered.contains("mybox_"), "typed value must render");
    assert!(
      rendered.contains("Enter apply") || rendered.contains("Enter aplicar"),
      "popup hint must render"
    );
    assert!(
      rendered.contains("Current hostname") || rendered.contains("Hostname atual"),
      "list row must still show the current hostname"
    );
  }

  #[test]
  fn tab_no_longer_jumps_to_actions_and_end_reaches_the_reset_row() {
    let mut app = App::new(Page::DefaultApps);
    app.error_modal = None;
    let before = app.selected_item();
    crate::event::handle(&mut app, Event::Key(KeyEvent::from(KeyCode::Tab)));
    assert_eq!(app.selected_item(), before);
    crate::event::handle(&mut app, Event::Key(KeyEvent::from(KeyCode::End)));
    crate::event::handle(&mut app, Event::Key(KeyEvent::from(KeyCode::Enter)));
    assert!(matches!(
      app.confirm,
      Some(crate::app::PendingAction::ResetApps)
    ));
    crate::event::handle(&mut app, Event::Key(KeyEvent::from(KeyCode::Esc)));
    assert!(app.confirm.is_none());
  }

  #[test]
  fn font_selector_page_has_reset_row_and_search_still_works() {
    let mut app = App::new(Page::Fonts);
    app.error_modal = None;
    app
      .navigation
      .push(Page::FontSelector(crate::config::fonts::FontTarget::Apps));
    assert_eq!(
      app.rows().last().and_then(|row| row.id().copied()),
      Some(crate::item::Item::ResetDefaults)
    );
    crate::event::handle(&mut app, Event::Key(KeyEvent::from(KeyCode::Char('/'))));
    assert!(app.searching);
    crate::event::handle(&mut app, Event::Key(KeyEvent::from(KeyCode::Esc)));
    assert!(!app.searching);
    crate::event::handle(&mut app, Event::Key(KeyEvent::from(KeyCode::Char('r'))));
    assert!(app.confirm.is_some());
  }

  #[test]
  /// Executes the `minimum_size_message_renders` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn minimum_size_message_renders() {
    let mut terminal = Terminal::new(TestBackend::new(42, 10)).unwrap();
    let mut app = App::new(Page::Main);
    app.error_modal = None;
    terminal.draw(|frame| draw(&mut app, frame)).unwrap();
    let rendered = terminal
      .backend()
      .buffer()
      .content
      .iter()
      .map(|cell| cell.symbol())
      .collect::<String>();
    assert!(rendered.contains("60x15"));
    assert!(rendered.contains("42x10"));
  }

  #[test]
  /// Executes the `task_window_renders_while_open` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn task_window_renders_while_open() {
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    let mut app = App::new(Page::Main);
    app.error_modal = None;
    let live = argvus_control_center_core::process::LiveProcess::new();
    live.push_line("locale-gen");
    app.task_live = Some(live);
    app.task_open = true;
    terminal.draw(|frame| draw(&mut app, frame)).unwrap();
    let rendered = terminal
      .backend()
      .buffer()
      .content
      .iter()
      .map(|cell| cell.symbol())
      .collect::<String>();
    assert!(rendered.contains("locale-gen"), "live line must render");
    assert!(
      rendered.contains("Process") || rendered.contains("Processo"),
      "process window title must render"
    );
  }
}
