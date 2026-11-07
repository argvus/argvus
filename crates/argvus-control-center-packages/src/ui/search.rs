//! The top Search bar owns the query, editing cursor and explicit submission.
use super::render::has_action_bar;
use super::*;

impl PackagesApp {
  pub(super) fn is_install_search(&self) -> bool {
    matches!(self.page, PackagesPage::Task(_, PackageTask::Install))
  }

  pub(super) fn on_aur_search(&self) -> bool {
    self.page == PackagesPage::Task(PackageSource::Aur, PackageTask::Install)
  }

  pub(super) fn focus_search(&mut self) {
    self.search_focused = true;
    self.query_cursor = self.query.len();
    self.actions.focus_list();
  }

  /// Search consumes editing before the list or its action bar sees the key.
  pub(super) fn handle_filter_key(&mut self, key: KeyCode) -> bool {
    if self.destructive() {
      return matches!(key, KeyCode::Char(_) | KeyCode::Backspace | KeyCode::Delete);
    }
    if key == KeyCode::Char('/') {
      self.focus_search();
      return true;
    }
    if self.search_focused {
      match key {
        KeyCode::Esc | KeyCode::Down | KeyCode::PageDown | KeyCode::Tab | KeyCode::BackTab => {
          self.search_focused = false;
          self.actions.focus_list();
        }
        KeyCode::Up | KeyCode::PageUp => {}
        KeyCode::Enter => {
          if self.is_install_search() {
            self.search_typed_query();
          } else {
            self.search_focused = false;
          }
        }
        _ => return self.edit_search_key(key),
      }
      return true;
    }
    if self.actions.cursor().is_none() {
      match key {
        KeyCode::Char(' ') if self.page == PackagesPage::Orphans => {
          self.toggle_mark();
          return true;
        }
        KeyCode::Up => {
          let rows = self.rows();
          self.menu.normalize(&rows);
          if self.menu.selected_index() == rows.iter().position(Row::is_selectable) {
            self.focus_search();
            return true;
          }
        }
        KeyCode::Tab | KeyCode::BackTab if !has_action_bar(self.page) => {
          self.focus_search();
          return true;
        }
        // Keep Enter useful on an empty Install list, even before '/' was used.
        KeyCode::Enter
          if self.is_install_search() && !self.rows().iter().any(Row::is_selectable) =>
        {
          self.focus_search();
          self.search_typed_query();
          return true;
        }
        _ => {}
      }
    }
    // Space on an Update action remains its activation shortcut.
    if key == KeyCode::Char(' ') && self.actions.cursor().is_some() {
      return false;
    }
    if matches!(key, KeyCode::Char(character) if !character.is_control())
      || key == KeyCode::Backspace
    {
      self.focus_search();
      return self.edit_search_key(key);
    }
    false
  }

  fn edit_search_key(&mut self, key: KeyCode) -> bool {
    let changed = match key {
      KeyCode::Char(character) if !character.is_control() => {
        self.insert_search_text(&character.to_string());
        return true;
      }
      KeyCode::Backspace if self.query_cursor > 0 => {
        let previous = self.query[..self.query_cursor]
          .char_indices()
          .next_back()
          .map(|(index, _)| index)
          .unwrap_or(0);
        self.query.replace_range(previous..self.query_cursor, "");
        self.query_cursor = previous;
        true
      }
      KeyCode::Delete if self.query_cursor < self.query.len() => {
        let length = self.query[self.query_cursor..]
          .chars()
          .next()
          .map(char::len_utf8)
          .unwrap_or(0);
        self
          .query
          .replace_range(self.query_cursor..self.query_cursor + length, "");
        true
      }
      KeyCode::Left => {
        self.query_cursor = self.query[..self.query_cursor]
          .char_indices()
          .next_back()
          .map(|(index, _)| index)
          .unwrap_or(0);
        false
      }
      KeyCode::Right => {
        self.query_cursor += self.query[self.query_cursor..]
          .chars()
          .next()
          .map(char::len_utf8)
          .unwrap_or(0);
        false
      }
      KeyCode::Home => {
        self.query_cursor = 0;
        false
      }
      KeyCode::End => {
        self.query_cursor = self.query.len();
        false
      }
      KeyCode::Backspace | KeyCode::Delete => false,
      _ => return false,
    };
    if changed {
      self.search_changed();
    }
    true
  }

  pub(super) fn insert_search_text(&mut self, text: &str) {
    let mut changed = false;
    for character in text.chars() {
      let character = if matches!(character, '\r' | '\n' | '\t') {
        ' '
      } else {
        character
      };
      if character.is_control() {
        continue;
      }
      if self.query.len() + character.len_utf8() > QUERY_MAX {
        break;
      }
      self.query.insert(self.query_cursor, character);
      self.query_cursor += character.len_utf8();
      changed = true;
    }
    if changed {
      self.search_changed();
    }
  }

  fn search_changed(&mut self) {
    if self.is_install_search() {
      // A draft is not a remote request. Cancel its old reader and remove
      // results that could otherwise install a target from a previous query.
      self.cancel_load();
      self.packages.clear();
      self.aur.clear();
      self.status = Some(StatusMessage {
        kind: StatusKind::Info,
        text: tr(
          self.lang,
          if self.on_aur_search() {
            "control_center.press_to_search_the_aur"
          } else {
            "control_center.press_to_search_packages"
          },
        )
        .into(),
      });
    }
    self.cursor_to_first_entry();
  }

  /// Only Enter in the Search bar submits an official/AUR repository query.
  pub(super) fn search_typed_query(&mut self) {
    let minimum = if self.on_aur_search() { 2 } else { 1 };
    if self.query.trim().chars().count() < minimum {
      self.error(tr(
        self.lang,
        if self.on_aur_search() {
          "control_center.aur_search_requires_at_least_2_characters"
        } else {
          "control_center.enter_a_non_empty_search_query"
        },
      ));
      return;
    }
    self.cancel_load();
    self.packages.clear();
    self.aur.clear();
    self.menu = MenuState::default();
    self.search_focused = false;
    self.actions.focus_list();
    // Explicit submission also refreshes an identical recently submitted query.
    self.reload_force();
  }

  pub(super) fn cursor_to_first_entry(&mut self) {
    self.menu = MenuState::default();
    let rows = self.rows();
    if let Some(first) = rows
      .iter()
      .filter(|row| row.is_selectable())
      .find_map(|row| row.id().copied().filter(|item| item.is_list_entry()))
    {
      self.menu.select(&rows, &first);
    }
  }
}
