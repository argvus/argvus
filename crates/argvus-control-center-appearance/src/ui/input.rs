//! Keys and pasted text on the appearance pages, and the page shortcuts.

use super::*;

impl AppearanceApp {
  pub fn paste(&mut self, text: &str) {
    if self.page != AppearancePage::AccentEdit {
      return;
    }
    let value = text.lines().next().unwrap_or_default().trim();
    if let Some(color) = normalize_hex_color(value) {
      self.prompt_buffer = color;
      self.prompt_error = None;
    } else {
      self.prompt_error = Some(tr(self.lang, "control_center.accent_invalid").into());
    }
  }

  /// Whether typed characters currently go to a prompt field (theme name,
  /// file path, gaps, borders) or to the accent HEX editor, so `q`/`?` must
  /// not act as the global quit/help keys. Same condition [`Self::handle`]
  /// uses to route keys to `prompt_key`.
  pub fn captures_text(&self) -> bool {
    self.prompt_back.is_some() || self.page == AppearancePage::AccentEdit || self.icon_searching
  }

  /// Processes one key press. Returns `true` when the user leaves the
  /// Appearance Home and the router should go back to the Control Center.
  pub fn handle(&mut self, key: KeyCode) -> bool {
    if self.page == AppearancePage::Icons
      && self.confirm.is_none()
      && let Some(leave) = self.handle_icon_key(key)
    {
      return leave;
    }
    if self.captures_text() {
      return self.prompt_key(key);
    }
    if let Some((confirmation, state)) = &mut self.confirm {
      let confirmation = *confirmation;
      match state.handle(key) {
        ConfirmOutcome::Pending => {}
        ConfirmOutcome::Cancelled => {
          self.confirm = None;
          self.cancel_confirmation(confirmation);
        }
        ConfirmOutcome::Confirmed => {
          self.confirm = None;
          return self.resolve_confirmation(confirmation);
        }
      }
      return false;
    }
    let rows = self.rows();
    let mut menu = self.menu;
    menu.normalize(&rows);
    let key = Self::page_key(menu.selected_kind(&rows), key);
    let event = menu.handle(key, &rows, usize::from(self.list_height));
    // While a load or an action runs, only moving and going back are
    // allowed (plus opening a section from the Home), as before.
    let busy = self.job.is_some() || self.action.is_some();
    if busy
      && !matches!(event, MenuEvent::Moved | MenuEvent::Back)
      && !(self.page == AppearancePage::Home && matches!(event, MenuEvent::Activate(_)))
    {
      return false;
    }
    self.menu = menu;
    match event {
      MenuEvent::Back => return self.back(),
      MenuEvent::Activate(item) | MenuEvent::Toggle(item) | MenuEvent::Confirm(item) => {
        self.activate(item);
      }
      MenuEvent::Adjust(item, delta) => self.adjust(item, delta),
      MenuEvent::Moved => {}
      MenuEvent::None => self.page_shortcut(key, &rows),
    }
    false
  }

  /// Appearance keeps two key aliases on top of the shared menu:
  ///
  /// - Space activates Action, Submenu, Choice and Destructive rows and opens
  ///   the editor of a Value row without a step, as it always did here (the
  ///   shared menu only uses Space for toggles);
  /// - `+`/`l` and `-`/`h` adjust a Value row with a step, like `→`/`←`.
  pub(super) fn page_key(selected: Option<RowKind>, key: KeyCode) -> KeyCode {
    match (selected, key) {
      (
        Some(
          RowKind::Action
          | RowKind::Submenu
          | RowKind::Choice { .. }
          | RowKind::Destructive
          | RowKind::Value { step: None },
        ),
        KeyCode::Char(' '),
      ) => KeyCode::Enter,
      (Some(RowKind::Value { step: Some(_) }), KeyCode::Char('+' | 'l')) => KeyCode::Right,
      (Some(RowKind::Value { step: Some(_) }), KeyCode::Char('-' | 'h')) => KeyCode::Left,
      _ => key,
    }
  }

  /// One-key shortcuts that are not part of the menu: `r` reloads, and the
  /// theme pages keep `e` (export), `i` (import) and `d` (delete).
  pub(super) fn page_shortcut(&mut self, key: KeyCode, rows: &[Row<Item>]) {
    let theme_page = matches!(
      self.page,
      AppearancePage::Themes | AppearancePage::CustomThemes
    );
    match key {
      KeyCode::Char('r') => self.refresh(),
      KeyCode::Char('e') if theme_page => self.open_prompt(PromptGoal::ExportProfile),
      KeyCode::Char('i') if theme_page => self.go(AppearancePage::ThemeImport),
      KeyCode::Char('d') if self.page == AppearancePage::CustomThemes => {
        if let Some(Item::CustomTheme(index)) = self.menu.selected_id(rows)
          && let Some(theme) = self.state.custom_themes.get(index).cloned()
        {
          self.delete_theme = Some(theme);
          self.confirm = Some((Confirmation::DeleteTheme, ConfirmState::new()));
        }
      }
      _ => {}
    }
  }
}
