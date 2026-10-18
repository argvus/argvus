//! Appearance > Desktop > Icons: a search field, the icon-theme list and an
//! Actions bar whose only button applies the chosen theme. Enter and Space
//! choose a theme as a draft; nothing is written until `[ Apply ]` runs.
//! Leaving the page drops the draft (see `AppearanceApp::back_discards_draft`).

use super::{AppearanceApp, FOOTER_GAP, rows::Item};
use crate::backend;
use argvus_control_center_core::config::AppConfig;
use argvus_i18n::tr;
use argvus_tui::{
  action_buttons::{ActionButton, draw_aligned_transparent},
  components::{StatusKind, StatusMessage, draw_status},
  icons,
  menu::{MenuStyle, Row, draw_menu},
};
use crossterm::event::KeyCode;
use ratatui::{
  Frame,
  layout::{Alignment, Constraint, Layout, Rect},
  style::Style,
  widgets::{Block, Borders, Paragraph},
};

impl AppearanceApp {
  /// Rows of the Icons list: the installed themes, filtered by the search
  /// text. Info rows are hidden while a search is active.
  pub(super) fn icon_rows(&self) -> Vec<Row<Item>> {
    let rows = self.icon_theme_rows();
    let query = self.icon_search.trim().to_lowercase();
    if query.is_empty() {
      return rows;
    }
    rows
      .into_iter()
      .filter(|row| row.id().is_some() && row.label().to_lowercase().contains(&query))
      .collect()
  }

  /// Whether the draft names an installed theme other than the configured one.
  pub(super) fn icon_changed(&self) -> bool {
    self.icon_draft.as_deref().is_some_and(|draft| {
      draft != self.state.icon_theme
        && self
          .state
          .icon_themes
          .iter()
          .any(|entry| entry.directory_name == draft)
    })
  }

  /// Indexes of the enabled Actions buttons: only `[ Apply ]`, and only when
  /// the draft differs from the configured theme.
  fn icon_enabled_actions(&self) -> Vec<usize> {
    if self.icon_changed() {
      vec![0]
    } else {
      Vec::new()
    }
  }

  /// Space or Enter on a theme row: chooses it as the draft. Nothing is
  /// applied here.
  pub(super) fn select_icon_theme(&mut self, index: usize) {
    if let Some(entry) = self.state.icon_themes.get(index) {
      self.icon_draft = Some(entry.directory_name.clone());
    }
  }

  /// Runs `[ Apply ]`: writes the draft through the backend.
  fn apply_icon_draft(&mut self) {
    if !self.icon_changed() {
      return;
    }
    let Some(name) = self.icon_draft.clone() else {
      return;
    };
    self.apply(
      tr(self.lang, "control_center.icon_theme_applied").into(),
      move || backend::set_icon_theme(&name),
    );
  }

  /// Key handling of the Icons page. Returns `Some(leave)` when the key was
  /// handled here, and `None` to let the shared menu process it.
  pub(super) fn handle_icon_key(&mut self, key: KeyCode) -> Option<bool> {
    if self.icon_searching {
      match key {
        KeyCode::Char(c) => self.icon_search.push(c),
        KeyCode::Backspace => {
          self.icon_search.pop();
        }
        KeyCode::Esc if !self.icon_search.is_empty() => self.icon_search.clear(),
        KeyCode::Esc | KeyCode::Enter | KeyCode::Up | KeyCode::Down => {
          self.icon_searching = false;
        }
        _ => {}
      }
      // The matches change with the text, so the cursor starts at the top.
      self.menu = Default::default();
      return Some(false);
    }
    let on_actions = self.icon_actions.cursor().is_some();
    match key {
      KeyCode::Char('/') => {
        self.icon_searching = true;
        Some(false)
      }
      KeyCode::Tab | KeyCode::BackTab => {
        let enabled = self.icon_enabled_actions();
        self.icon_actions.toggle(&enabled);
        Some(false)
      }
      KeyCode::Left | KeyCode::Right => {
        let enabled = self.icon_enabled_actions();
        self.icon_actions.move_by(key == KeyCode::Left, &enabled);
        Some(false)
      }
      KeyCode::Enter if on_actions => {
        self.apply_icon_draft();
        Some(false)
      }
      KeyCode::Esc if on_actions => {
        self.icon_actions.focus_list();
        Some(false)
      }
      // Up and Down give the focus back to the list and move there.
      KeyCode::Up | KeyCode::Down if on_actions => {
        self.icon_actions.focus_list();
        None
      }
      _ => None,
    }
  }

  /// Footer shortcuts of the Icons page, drawn on the right.
  fn icon_hints(&self) -> String {
    let label = |key: &str| tr(self.lang, key);
    [
      ("↑/↓", label("control_center.hint.navigate")),
      ("←/→", label("control_center.actions")),
      ("Space", label("control_center.hint.select")),
      ("/", label("control_center.hint.search")),
      ("r", label("control_center.refresh")),
      ("Esc", label("control_center.hint.back")),
      ("?", label("control_center.hint.help")),
    ]
    .map(|(keys, action)| format!("{keys} {action}"))
    .join(FOOTER_GAP)
  }

  /// Draws the Icons body: the search box, the theme list and the Actions bar.
  pub(super) fn draw_icons(&mut self, frame: &mut Frame, area: Rect, rows: &[Row<Item>]) {
    let parts = Layout::vertical([
      Constraint::Length(3),
      Constraint::Min(1),
      Constraint::Length(3),
    ])
    .split(area);
    let (search_area, list_area, actions_area) = (parts[0], parts[1], parts[2]);

    let search_block = Block::default()
      .borders(Borders::ALL)
      .border_style(Style::new().fg(self.theme.border_active));
    let cursor = if self.icon_searching { "_" } else { "" };
    let search_text = format!(
      "{} {}{cursor}",
      tr(self.lang, "control_center.search_field"),
      self.icon_search
    );
    let search_inner = search_block.inner(search_area);
    frame.render_widget(search_block, search_area);
    frame.render_widget(
      Paragraph::new(search_text).style(Style::new().fg(self.theme.foreground)),
      search_inner,
    );

    let list_block = Block::default()
      .borders(Borders::ALL)
      .border_style(Style::new().fg(self.theme.border_active))
      .title(tr(self.lang, "control_center.icons"));
    let list_inner = list_block.inner(list_area);
    frame.render_widget(list_block, list_area);
    self.list_height = list_inner.height;
    draw_menu(
      frame,
      list_inner,
      &self.theme,
      rows,
      &mut self.menu,
      MenuStyle {
        icons: AppConfig::icons_enabled(),
      },
    );

    let actions_block = Block::default()
      .borders(Borders::ALL)
      .border_style(Style::new().fg(self.theme.border_active))
      .title(format!(
        "{} · {}",
        tr(self.lang, "control_center.actions"),
        tr(self.lang, "control_center.actions_hint")
      ));
    let actions_inner = actions_block.inner(actions_area);
    frame.render_widget(actions_block, actions_area);
    let apply_icon = if AppConfig::icons_enabled() {
      icons::APPLY
    } else {
      ""
    };
    let apply_label = tr(self.lang, "control_center.apply");
    let apply = if self.icon_enabled_actions().is_empty() {
      ActionButton::secondary(apply_icon, apply_label, "")
    } else {
      ActionButton::primary(apply_icon, apply_label, "")
    };
    let selected = self.icon_actions.cursor().unwrap_or(usize::MAX);
    draw_aligned_transparent(
      frame,
      actions_inner,
      &[apply],
      selected,
      &self.theme,
      Alignment::Left,
    );
  }

  /// Draws the Icons footer under the body: the status on the left, the
  /// shortcuts on the right. `body` is the area `shell` returned.
  pub(super) fn draw_icons_footer(&self, frame: &mut Frame, body: Rect) {
    let footer = Rect::new(body.x, body.y + body.height, body.width, 2);
    let status_row = Rect::new(
      footer.x.saturating_add(1),
      footer.y,
      footer.width.saturating_sub(2),
      1,
    );
    let hints_row = Rect::new(status_row.x, footer.y + 1, status_row.width, 1);

    if self.status_loading {
      draw_status(
        frame,
        status_row,
        &self.theme,
        &StatusMessage {
          kind: StatusKind::Info,
          text: tr(self.lang, "control_center.loading_appearance").into(),
        },
      );
    } else if let Some(message) = &self.status {
      draw_status(frame, status_row, &self.theme, message);
    }
    frame.render_widget(
      Paragraph::new(self.icon_hints())
        .alignment(Alignment::Right)
        .style(Style::new().fg(self.theme.muted)),
      hints_row,
    );
  }
}
