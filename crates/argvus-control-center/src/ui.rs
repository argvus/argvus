//! Implements terminal UI rendering and interaction in crate `argvus control center`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use argvus_control_center_core::config::AppConfig;
use argvus_control_center_core::navigation::HomeCardId;
use argvus_i18n::tr;
use argvus_tui::chrome::{Header, draw_footer, draw_header, draw_help, draw_too_small};
use argvus_tui::text::ellipsize;
use argvus_tui::{MIN_HEIGHT, MIN_WIDTH};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Margin, Rect};
use ratatui::style::{Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
  Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget,
  Widget,
};

use crate::app::{App, Route};
use crate::home::{HomeCategory, HomeDestination, HomeRow};

/// Renders `draw` while respecting the current domain state and semantic theme. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn draw(app: &mut App, frame: &mut Frame) {
  let area = frame.area();
  app.resize(area.width, area.height);
  if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
    draw_too_small(
      frame,
      area,
      &app.theme,
      tr(app.lang, "control_center.terminal_window_is_too_small"),
      tr(app.lang, "control_center.minimum_size"),
      tr(app.lang, "control_center.current_size"),
    );
    return;
  }

  match app.route {
    Route::Home => draw_home(app, frame),
    Route::AppSettings => app.settings_hub.draw(frame, app.lang, &app.theme),
    Route::Config => app.config.draw(frame),
    Route::Settings => argvus_control_center_settings::ui::draw(&mut app.settings, frame),
    #[cfg(feature = "about")]
    Route::About => argvus_control_center_about::ui::draw(&mut app.about, frame),
    #[cfg(feature = "hardware")]
    Route::Hardware => app.hardware.draw(frame),
    #[cfg(feature = "services")]
    Route::Services => app.services.draw(frame),
    #[cfg(feature = "dev-services")]
    Route::DevServices => app.dev_services.draw(frame),
    #[cfg(feature = "network")]
    Route::Network => app.network.draw(frame),
    #[cfg(feature = "audio")]
    Route::Audio => app.audio.draw(frame),
    #[cfg(feature = "bluetooth")]
    Route::Bluetooth => app.bluetooth.draw(frame),
    #[cfg(feature = "boot")]
    Route::Boot => app.boot.draw(frame),
    #[cfg(feature = "packages")]
    Route::Packages => app.packages.draw(frame),
    #[cfg(feature = "storage")]
    Route::Storage => app.storage.draw(frame),
    #[cfg(feature = "diagnostics")]
    Route::Diagnostics => app.diagnostics.draw(frame),
    #[cfg(feature = "power")]
    Route::Power => app.power.draw(frame),
    #[cfg(feature = "session")]
    Route::Session => app.session.draw(frame),
    #[cfg(feature = "displays")]
    Route::Displays => app.displays.draw(frame),
    #[cfg(feature = "appearance")]
    Route::Appearance => app.appearance.draw(frame),
    Route::Hyprland => crate::hyprland::draw(app, frame),
  }
  if app.help {
    draw_help(
      frame,
      area,
      &app.theme,
      tr(app.lang, "control_center.help"),
      &app.help_lines(),
    );
  }
}

/// Renders `draw_home` while respecting the current domain state and semantic theme. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn draw_home(app: &mut App, frame: &mut Frame) {
  let area = frame.area();
  Block::new()
    .borders(Borders::ALL)
    .border_style(Style::new().fg(app.theme.border_active))
    .bg(app.theme.background)
    .render(area, frame.buffer_mut());
  let inner = area.inner(Margin::new(1, 1));
  let rows = Layout::vertical([
    Constraint::Length(1),
    Constraint::Length(1),
    Constraint::Length(3),
    Constraint::Min(1),
    Constraint::Length(1),
    Constraint::Length(1),
  ])
  .split(inner);
  draw_header(
    frame,
    rows[0],
    &app.theme,
    Header {
      title: tr(app.lang, "control_center.argvus_control_center"),
      version: None,
      version_label: "",
    },
  );
  frame.render_widget(
    Paragraph::new(Line::from(Span::styled(
      tr(app.lang, "control_center.home_subtitle"),
      Style::new().fg(app.theme.muted),
    ))),
    rows[1].inner(Margin::new(1, 0)),
  );
  draw_search(app, frame, rows[2]);
  if app.search_active && !app.search_query.is_empty() {
    let mut lines: Vec<Line<'static>> = vec![Line::from(Span::styled(
      format!("  {}", tr(app.lang, "control_center.search_results")),
      Style::new()
        .fg(app.theme.accent)
        .add_modifier(Modifier::BOLD),
    ))];
    if app.search_result_ids.is_empty() {
      lines.push(Line::from(Span::styled(
        format!("  {}", tr(app.lang, "control_center.search_no_results")),
        Style::new().fg(app.theme.muted),
      )));
    } else {
      for (index, id) in app.search_result_ids.iter().enumerate() {
        if let Some(entry) = app
          .search_registry
          .entries()
          .iter()
          .find(|entry| &entry.id == id)
        {
          let is_selected = index == app.search_selected;
          let label = format!(
            "{} > {}",
            search_category(app.lang, &entry.category),
            entry.title
          );
          let style = if is_selected {
            Style::new()
              .fg(app.theme.selected_foreground)
              .bg(app.theme.selected_background)
              .add_modifier(Modifier::BOLD)
          } else {
            Style::new().fg(app.theme.foreground)
          };
          lines.push(
            Line::from(Span::styled(
              format!(
                " {} {}",
                if is_selected { ">" } else { " " },
                ellipsize(&label, rows[3].width.saturating_sub(6) as usize)
              ),
              style,
            ))
            .style(style),
          );
        }
      }
    }
    frame.render_widget(Paragraph::new(lines), rows[3].inner(Margin::new(2, 1)));
  } else {
    draw_home_grid(app, frame, rows[3]);
  }
  draw_standalone_home_entries(app, frame, rows[4]);
  draw_footer(
    frame,
    rows[5],
    &app.theme,
    None,
    if app.search_active {
      tr(app.lang, "control_center.search_navigation_help")
    } else {
      tr(app.lang, "control_center.home_navigation_search_help")
    },
  );
}

fn draw_home_grid(app: &mut App, frame: &mut Frame, area: Rect) {
  draw_home_cards(app, frame, area);
}

fn draw_home_cards(app: &mut App, frame: &mut Frame, area: Rect) {
  let content = area.inner(Margin::new(1, 0));
  let columns = App::home_grid_columns(content.width);
  let cards = home_cards(app);
  let mut rows: Vec<Vec<&HomeCard>> = Vec::new();
  let mut current = Vec::new();
  for card in &cards {
    current.push(card);
    if current.len() == columns {
      rows.push(current);
      current = Vec::new();
    }
  }
  if !current.is_empty() {
    rows.push(current);
  }

  let row_heights: Vec<u16> = rows
    .iter()
    .map(|row| row.iter().map(|card| card_height(card)).max().unwrap_or(1))
    .collect();
  let selected_row = rows
    .iter()
    .position(|row| {
      row.iter().any(|card| {
        card
          .items
          .iter()
          .any(|(index, _, _)| *index == app.home_selected)
      })
    })
    .unwrap_or(0);
  let visible_height = content.height.max(1);
  let mut scroll = app.home_scroll.min(rows.len().saturating_sub(1));
  while selected_row < scroll {
    scroll = scroll.saturating_sub(1);
  }
  while selected_row > scroll
    && row_heights[scroll..=selected_row]
      .iter()
      .map(|height| *height as usize)
      .sum::<usize>()
      > visible_height as usize
  {
    scroll += 1;
  }
  app.home_scroll = scroll.min(rows.len().saturating_sub(1));

  let mut y = content.y;
  for (row_index, row) in rows.iter().enumerate().skip(app.home_scroll) {
    let height = row_heights[row_index];
    if y >= content.bottom() || y + height > content.bottom() {
      break;
    }
    let row_area = Rect::new(content.x, y, content.width, height);
    let constraints = vec![Constraint::Ratio(1, row.len() as u32); row.len()];
    for (card_area, card) in Layout::horizontal(constraints)
      .split(row_area)
      .iter()
      .zip(row)
    {
      draw_home_card(app, frame, *card_area, card);
    }
    y += height;
  }

  let total_height: usize = row_heights.iter().map(|height| *height as usize).sum();
  if total_height > visible_height as usize {
    let track = Rect {
      x: area.right().saturating_sub(1),
      y: content.y,
      width: 1,
      height: content.height,
    };
    if track.x >= content.x && track.width > 0 {
      let mut state = ScrollbarState::new(rows.len()).position(app.home_scroll);
      Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(None)
        .end_symbol(None)
        .track_symbol(Some("│"))
        .thumb_style(Style::new().fg(app.theme.accent))
        .track_style(Style::new().fg(app.theme.border_active))
        .render(track, frame.buffer_mut(), &mut state);
    }
  }
}

/// A Home card: its heading and the destinations it lists, in order.
#[derive(Debug, Clone)]
struct HomeCard {
  category: HomeCategory,
  items: Vec<(usize, &'static str, HomeDestination)>,
}

/// Builds the Home cards, one per heading of the catalog.
fn home_cards(app: &App) -> Vec<HomeCard> {
  let mut cards = Vec::new();
  let mut item_index = 0;
  let mut current: Option<HomeCard> = None;
  for row in app.home_rows() {
    match row {
      HomeRow::Header(category) => {
        if let Some(card) = current.take() {
          cards.push(card);
        }
        current = Some(HomeCard {
          category,
          items: Vec::new(),
        });
      }
      HomeRow::Item { label, destination } => {
        if let Some(card) = current.as_mut() {
          card.items.push((item_index, label, destination));
        }
        item_index += 1;
      }
      // Standalone entries are drawn below the cards, see `draw_standalone_home_entries`.
      HomeRow::Standalone { .. } => {
        if let Some(card) = current.take() {
          cards.push(card);
        }
        item_index += 1;
      }
    }
  }
  if let Some(card) = current {
    cards.push(card);
  }
  cards
}

/// Draws the entries listed outside the cards (Settings), one row below the grid.
fn draw_standalone_home_entries(app: &App, frame: &mut Frame, area: Rect) {
  let Some(label) = app.home_rows().iter().find_map(|row| match row {
    HomeRow::Standalone { label, .. } => Some(*label),
    _ => None,
  }) else {
    return;
  };
  // A standalone entry is always the last destination of the Home.
  let index = app.home_item_count().saturating_sub(1);
  let selected = app.home_selected == index;
  let style = if selected {
    Style::new()
      .fg(app.theme.selected_foreground)
      .bg(app.theme.selected_background)
      .add_modifier(Modifier::BOLD)
  } else {
    Style::new()
      .fg(app.theme.foreground)
      .bg(app.theme.background)
  };
  let text = format!(
    "{}{}{}",
    if selected { "> " } else { "  " },
    home_icon_for_item(HomeDestination::Settings),
    label
  );
  frame.render_widget(
    Paragraph::new(Line::from(Span::styled(
      ellipsize(&text, area.width.saturating_sub(2) as usize),
      style,
    ))),
    area,
  );
}

/// Rows a card needs: its border, heading, description, divider and one row per item.
fn card_height(card: &HomeCard) -> u16 {
  5 + card.items.len().max(1) as u16
}

/// Draws one Home card: border, icon and title, description, divider, items.
fn draw_home_card(app: &App, frame: &mut Frame, area: Rect, card: &HomeCard) {
  let selected = card
    .items
    .iter()
    .any(|(index, _, _)| *index == app.home_selected);
  let border = if selected {
    app.theme.border_active
  } else {
    app.theme.border
  };
  let width = area.width.saturating_sub(2) as usize;
  let title = format!(
    "{}{}",
    argvus_tui::icons::icon_label(AppConfig::icon(card.category.glyph), ""),
    card.category.title
  );
  let header_lines = vec![
    Line::from(Span::styled(
      ellipsize(&title, width),
      Style::new()
        .fg(app.theme.accent)
        .add_modifier(Modifier::BOLD),
    )),
    Line::from(Span::styled(
      ellipsize(card.category.description, width),
      Style::new().fg(app.theme.muted),
    )),
    Line::from(Span::styled(
      "─".repeat(width),
      Style::new().fg(app.theme.border),
    )),
  ];
  frame.render_widget(
    Block::bordered()
      .border_style(Style::new().fg(border))
      .bg(app.theme.background),
    area,
  );
  let header_area = Rect::new(
    area.x + 1,
    area.y + 1,
    area.width.saturating_sub(2),
    3.min(area.height.saturating_sub(2)),
  );
  frame.render_widget(Paragraph::new(header_lines), header_area);
  let item_area = Rect::new(
    area.x + 1,
    area.y + 4,
    area.width.saturating_sub(2),
    area.height.saturating_sub(5),
  );
  for (index, item) in card.items.iter().enumerate() {
    let Ok(row) = u16::try_from(index) else {
      break;
    };
    if row >= item_area.height {
      break;
    }
    let item_rect = Rect::new(item_area.x, item_area.y + row, item_area.width, 1);
    let is_selected = item.0 == app.home_selected;
    let style = if is_selected {
      Style::new()
        .fg(app.theme.selected_foreground)
        .bg(app.theme.selected_background)
        .add_modifier(Modifier::BOLD)
    } else {
      Style::new().fg(app.theme.foreground)
    };
    let text = format!(
      "{}{}{}",
      if is_selected { "> " } else { "  " },
      home_icon_for_item(item.2),
      item.1
    );
    frame.render_widget(
      Paragraph::new(Line::from(Span::styled(
        ellipsize(&text, item_rect.width.saturating_sub(1) as usize),
        style,
      ))),
      item_rect,
    );
  }
}

/// Renders `draw_search` while respecting the current domain state and semantic theme. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn draw_search(app: &App, frame: &mut Frame, area: ratatui::layout::Rect) {
  let value = &app.search_query;
  let style = if app.search_active {
    Style::new()
      .fg(app.theme.foreground)
      .bg(app.theme.background)
  } else {
    Style::new().fg(app.theme.muted).bg(app.theme.background)
  };
  let width = area.width.saturating_sub(4) as usize;
  let prefix = format!("{}: ", tr(app.lang, "control_center.search_2c43ee"));
  let prefix_width = argvus_tui::text::display_width(&prefix);
  let cursor_width = usize::from(app.search_active && app.search_cursor_visible);
  let value_width = width.saturating_sub(prefix_width + cursor_width);
  let value = ellipsize(value, value_width);
  let cursor = if app.search_active && app.search_cursor_visible {
    Span::styled(
      "▌",
      Style::new().fg(app.theme.accent).bg(app.theme.background),
    )
  } else {
    Span::raw("")
  };
  let value_span = Span::styled(value, style);
  frame.render_widget(
    Paragraph::new(Line::from(vec![
      Span::styled(
        prefix,
        Style::new()
          .fg(app.theme.accent)
          .add_modifier(Modifier::BOLD),
      ),
      value_span,
      cursor,
    ]))
    .style(style)
    .block(
      Block::bordered().border_style(Style::new().fg(if app.search_active {
        app.theme.border_active
      } else {
        app.theme.border
      })),
    ),
    area.inner(Margin::new(1, 0)),
  );
}

/// Executes the `home_icon_for_item` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn home_icon_for_item(destination: HomeDestination) -> String {
  let icon = AppConfig::icon(destination.glyph());
  argvus_tui::icons::icon_label(icon, "")
}

/// Executes the `search_category` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn search_category(lang: argvus_i18n::Lang, category: &str) -> String {
  // A card category shares its label with the Home and the breadcrumbs.
  if let Some(card) = HomeCardId::from_search_category(category) {
    return card.label(lang).to_string();
  }
  // Hyprland is a page inside the Desktop & Input card, so its results keep
  // that root, as the breadcrumb of the Hyprland pages does.
  if category == "hyprland" {
    return format!(
      "{} > {}",
      HomeCardId::DesktopInput.label(lang),
      tr(lang, "control_center.hyprland")
    );
  }
  let key = format!("control_center.{category}");
  let translated = tr(lang, &key);
  if translated == key {
    category.to_string()
  } else {
    translated.to_string()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::app::InitialRoute;
  use ratatui::{Terminal, backend::TestBackend};

  #[test]
  /// A search card label and a breadcrumb card label come from the same
  /// `HomeCardId`, so they read the same in en-US and pt-BR.
  fn search_and_breadcrumbs_share_card_names() {
    for lang in [
      argvus_i18n::Lang::for_locale("en-US"),
      argvus_i18n::Lang::for_locale("pt-BR"),
    ] {
      for card in HomeCardId::ALL {
        let category = card.title_key().strip_prefix("control_center.").unwrap();
        assert_eq!(
          search_category(lang, category),
          card.label(lang),
          "{card:?}"
        );
      }
    }
  }

  #[test]
  /// Executes the `home_renders_existing_and_phase_two_destinations` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn home_renders_existing_and_phase_two_destinations() {
    let mut app = App::new(InitialRoute::Home);
    let mut terminal = Terminal::new(TestBackend::new(90, 32)).unwrap();
    terminal.draw(|frame| draw(&mut app, frame)).unwrap();
    let text = terminal
      .backend()
      .buffer()
      .content
      .iter()
      .map(|cell| cell.symbol())
      .collect::<String>();
    assert!(text.contains("ARGVUS"));
    assert!(text.contains("Control Center") || text.contains("Central de Controle"));
    assert!(text.contains("/ Search") || text.contains("/ Pesquisar") || text.contains("Search"));
  }

  #[test]
  /// Executes the `domain_pages_use_the_shared_chrome` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn domain_pages_use_the_shared_chrome() {
    let cases: [(Route, [&str; 2]); _] = [
      #[cfg(feature = "network")]
      (Route::Network, ["Network", "Rede"]),
      #[cfg(feature = "boot")]
      (Route::Boot, ["Boot", "Boot"]),
      #[cfg(feature = "packages")]
      (Route::Packages, ["Packages", "Pacotes"]),
    ];
    for (route, marker) in cases {
      let mut app = App::new(InitialRoute::Home);
      app.route = route;
      let mut terminal = Terminal::new(TestBackend::new(90, 25)).unwrap();
      terminal.draw(|frame| draw(&mut app, frame)).unwrap();
      let text = terminal
        .backend()
        .buffer()
        .content
        .iter()
        .map(|cell| cell.symbol())
        .collect::<String>();
      assert!(text.contains("ARGVUS"));
      assert!(text.contains(marker[0]) || text.contains(marker[1]));
      assert!(text.contains("Enter") || text.contains("Abrir"));
      assert!(!text.contains("┌ Network") && !text.contains("┌ Boot"));
    }
  }

  #[test]
  /// Executes the `home_renders_the_live_global_query_and_result` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn home_renders_the_live_global_query_and_result() {
    let mut app = App::new(InitialRoute::Home);
    app.begin_global_search();
    app.update_global_search("wifi".into());
    let mut terminal = Terminal::new(TestBackend::new(90, 32)).unwrap();
    terminal.draw(|frame| draw(&mut app, frame)).unwrap();
    let text = terminal
      .backend()
      .buffer()
      .content
      .iter()
      .map(|cell| cell.symbol())
      .collect::<String>();
    assert!(text.contains("wifi"));
    assert!(text.contains("Wi-Fi"));
    assert!(text.contains("▌"));
  }

  #[test]
  /// Executes the `config_screen_renders_with_chrome_and_checkbox` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn config_screen_renders_with_chrome_and_checkbox() {
    let mut app = App::new(InitialRoute::Home);
    app.route = Route::Config;
    let mut terminal = Terminal::new(TestBackend::new(90, 24)).unwrap();
    terminal.draw(|frame| draw(&mut app, frame)).unwrap();
    let text = terminal
      .backend()
      .buffer()
      .content
      .iter()
      .map(|cell| cell.symbol())
      .collect::<String>();
    assert!(text.contains("ARGVUS"));
    assert!(text.contains("Estilo") || text.contains("Style"));
    assert!(text.contains("[x]") || text.contains("[ ]"));
    assert!(text.contains("Ícones") || text.contains("Icons"));
  }

  #[cfg(feature = "apps")]
  #[test]
  fn the_key_pages_on_the_home_have_an_icon() {
    // A Home row without a glyph looks broken next to the others; the SSH and
    // GPG key pages are the ones that were missing theirs.
    assert!(!HomeDestination::SshKeys.glyph().is_empty(), "SSH keys");
    assert!(!HomeDestination::GpgKeys.glyph().is_empty(), "GPG keys");
  }
}
