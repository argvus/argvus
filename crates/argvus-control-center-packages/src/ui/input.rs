//! Keyboard and text input: list keys, the action bar, the transaction window, filters, search and number fields.

use super::render::{filters_while_typing, has_action_bar, transaction_bottom_offset};
use super::*;

impl PackagesApp {
  /// Whether typed characters currently go to text (the input prompt, or the
  /// filter of a package list), so `q`/`?` must not act as the global
  /// quit/help keys. Mirrors the precedence of [`Self::handle`]:
  /// confirmations, the transaction log and running operations sit above
  /// the list filter and do not take text.
  pub fn captures_text(&self) -> bool {
    if self.input.is_some() || self.editor.is_some() {
      return true;
    }
    self.pending.is_none()
      && !self.transaction_open
      && !self.destructive()
      && filters_while_typing(self.page)
  }

  pub fn handle(&mut self, key: KeyCode) -> bool {
    if self.editor.is_some() {
      return self.handle_key(KeyEvent::from(key));
    }
    if self.input.is_some() {
      return self.handle_input(key);
    }
    if let Some(state) = self.discard.as_mut() {
      match state.handle(key) {
        ConfirmOutcome::Confirmed => {
          self.discard = None;
          self.repo_draft = RepositoryDraft::default();
          return self.back();
        }
        ConfirmOutcome::Cancelled => self.discard = None,
        ConfirmOutcome::Pending => {}
      }
      return false;
    }
    if let Some(choices) = self.pending.as_ref().and_then(Action::choices) {
      match self.choice.handle(key, choices.len()) {
        ChoiceOutcome::Chosen(index) => {
          // The choice is the confirmation: the chosen way starts now.
          let [first, second] = choices;
          self.pending = Some(if index == 0 { first } else { second });
          self.choice = ChoiceState::new();
          self.start_pending();
        }
        ChoiceOutcome::Cancelled => {
          self.pending = None;
          self.preview = None;
          self.choice = ChoiceState::new();
        }
        ChoiceOutcome::Pending => {}
      }
      return false;
    }
    if self.pending.is_some() {
      match self.confirmation.handle(key) {
        ConfirmOutcome::Confirmed => {
          self.confirmation = ConfirmState::new();
          self.start_pending();
        }
        ConfirmOutcome::Cancelled => {
          self.pending = None;
          self.preview = None;
          self.confirmation = ConfirmState::new();
        }
        ConfirmOutcome::Pending => {}
      }
      return false;
    }
    if self.transaction_open {
      self.handle_transaction(key);
      return false;
    }
    if has_action_bar(self.page) && self.handle_action_bar(key) {
      return false;
    }
    if key == KeyCode::Right && self.open_selected_details() {
      return false;
    }
    // Tab only switches tabs or panes; the other pages have none.
    if matches!(key, KeyCode::Tab | KeyCode::BackTab) {
      return false;
    }
    if filters_while_typing(self.page) {
      if self.handle_filter_key(key) {
        return false;
      }
    } else if key == KeyCode::Char('r') && self.page != PackagesPage::MirrorEditor {
      if !self.destructive() {
        self.reload_force();
      }
      return false;
    } else if key == KeyCode::Char(' ') && self.page == PackagesPage::MirrorEditor {
      // Space moved every reflector option forward, as before.
      let rows = self.rows();
      if let Some(item) = self.menu.selected_id(&rows) {
        self.adjust(item, 1);
      }
      return false;
    }
    let rows = self.rows();
    let page_size = usize::from(self.list_height.max(1));
    match self.menu.handle(key, &rows, page_size) {
      MenuEvent::Back => return self.back(),
      MenuEvent::Activate(item) | MenuEvent::Toggle(item) | MenuEvent::Confirm(item) => {
        self.activate(item)
      }
      MenuEvent::Adjust(item, step) => self.adjust(item, step),
      MenuEvent::Moved | MenuEvent::None => {}
    }
    false
  }

  /// A key with its modifiers, from the event loop: the text editor of the
  /// repository Add page needs `Ctrl+S`; every other key goes to
  /// [`Self::handle`].
  pub fn handle_key(&mut self, key: KeyEvent) -> bool {
    let Some((field, editor)) = self.editor.as_mut() else {
      return self.handle(key.code);
    };
    match editor.handle(key) {
      EditorOutcome::Saved(text) => {
        let field = *field;
        self.editor = None;
        self.save_repository_field(field, text);
      }
      EditorOutcome::Cancelled => self.editor = None,
      EditorOutcome::Pending => {}
    }
    false
  }

  /// Pasted text (bracketed paste) goes to the open text editor.
  pub fn paste(&mut self, text: &str) {
    if let Some((_, editor)) = self.editor.as_mut() {
      editor.paste(text);
    }
  }

  /// The action bar under the Update lists: `Tab` moves the focus to its
  /// buttons and back, `←/→` move between them, `Enter`/`Space` run the
  /// focused one, and `↑/↓` give the focus back to the list. On the list,
  /// `→` opens the package details and `←` goes back (D58). `true` when the
  /// key was consumed.
  pub(super) fn handle_action_bar(&mut self, key: KeyCode) -> bool {
    let focused = self.actions.cursor().is_some();
    match key {
      KeyCode::Left | KeyCode::Right if focused => {
        let enabled = self.enabled_action_indexes();
        self.actions.move_by(key == KeyCode::Left, &enabled);
        true
      }
      KeyCode::Tab | KeyCode::BackTab => {
        let enabled = self.enabled_action_indexes();
        self.actions.toggle(&enabled);
        true
      }
      KeyCode::Enter | KeyCode::Char(' ') if focused => {
        self.activate_focused_action();
        true
      }
      KeyCode::Up
      | KeyCode::Down
      | KeyCode::PageUp
      | KeyCode::PageDown
      | KeyCode::Home
      | KeyCode::End
      | KeyCode::Backspace
      | KeyCode::Char(_)
        if focused =>
      {
        // The list and its filter handle the key itself, below: typing on
        // the bar goes back to the filter instead of leaving the focus there.
        self.actions.focus_list();
        false
      }
      _ => false,
    }
  }

  /// Indexes of the enabled buttons of the Updates action bar.
  pub(super) fn enabled_action_indexes(&self) -> Vec<usize> {
    self
      .update_action_rows()
      .iter()
      .enumerate()
      .filter(|(_, row)| row.is_selectable())
      .map(|(index, _)| index)
      .collect()
  }

  /// Enter/Space on the focused button: runs its row, as Enter on the row would.
  pub(super) fn activate_focused_action(&mut self) {
    let rows = self.update_action_rows();
    let Some(row) = self.actions.cursor().and_then(|index| rows.get(index)) else {
      return;
    };
    if !row.is_selectable() {
      return;
    }
    if let Some(item) = row.id().copied() {
      self.activate(item);
    }
  }

  /// Scrolls the live transaction output; Esc closes it.
  pub(super) fn handle_transaction(&mut self, key: KeyCode) {
    match key {
      KeyCode::Esc => {
        self.transaction_open = false;
        self.transaction_live = None;
        self.transaction_scroll = 0;
        self.transaction_follow = false;
        if self.action.is_none() {
          self.ask_follow_up();
        }
      }
      KeyCode::Up | KeyCode::Char('k') => {
        self.transaction_follow = false;
        self.transaction_scroll = self.transaction_scroll.saturating_sub(1);
      }
      KeyCode::Down | KeyCode::Char('j') => {
        self.transaction_follow = false;
        self.transaction_scroll = self.transaction_scroll.saturating_add(1);
      }
      KeyCode::PageUp => {
        self.transaction_follow = false;
        self.transaction_scroll = self.transaction_scroll.saturating_sub(10);
      }
      KeyCode::PageDown => {
        self.transaction_follow = false;
        self.transaction_scroll = self.transaction_scroll.saturating_add(10);
      }
      KeyCode::Home => {
        self.transaction_follow = false;
        self.transaction_scroll = 0;
      }
      KeyCode::End => {
        if let Some(live) = &self.transaction_live {
          self.transaction_scroll = transaction_bottom_offset(&live.output());
        }
      }
      _ => {}
    }
  }

  /// Keys of the lists that filter while typing (D8). Returns whether the
  /// key was used; arrows, PgUp/PgDn, Home/End, Enter and Esc go on to the
  /// menu.
  pub(super) fn handle_filter_key(&mut self, key: KeyCode) -> bool {
    match key {
      KeyCode::Char(' ') if self.page == PackagesPage::Orphans => {
        if !self.destructive() {
          self.toggle_mark();
        }
      }
      KeyCode::Char('/') => {
        if !self.destructive() {
          self.input = Some(self.query.clone());
          self.input_mode = Some(InputMode::Search);
        }
      }
      KeyCode::Char(c) if !c.is_control() => {
        if !self.destructive() && self.query.len() < QUERY_MAX {
          self.query.push(c);
          self.cursor_to_first_entry();
        }
      }
      KeyCode::Backspace => {
        if !self.destructive() {
          self.query.pop();
          self.cursor_to_first_entry();
        }
      }
      // With nothing to install, Enter on the Install lists searches for
      // the typed text, as before.
      KeyCode::Enter
        if matches!(self.page, PackagesPage::Task(_, PackageTask::Install))
          && !self.rows().iter().any(Row::is_selectable) =>
      {
        if !self.destructive() {
          self.search_typed_query();
        }
      }
      _ => return false,
    }
    true
  }

  /// Runs the remote search for the typed filter on Search and AUR.
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
    } else {
      self.reload();
    }
  }

  /// The AUR Install list, whose remote search needs two characters.
  pub(super) fn on_aur_search(&self) -> bool {
    self.page == PackagesPage::Task(PackageSource::Aur, PackageTask::Install)
  }

  /// After the filter changed, puts the cursor on the first package, as the
  /// lists did before (or on the first selectable row without matches).
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

  /// Space on an orphan marks or unmarks it for `Remove marked`.
  pub(super) fn toggle_mark(&mut self) {
    let rows = self.rows();
    let Some(Item::Orphan(index)) = self.menu.selected_id(&rows) else {
      return;
    };
    let Some(name) = self.packages.get(index).map(|package| package.name.clone()) else {
      return;
    };
    if let Some(position) = self.marked.iter().position(|marked| *marked == name) {
      self.marked.remove(position);
    } else {
      self.marked.push(name);
    }
  }

  pub(super) fn handle_input(&mut self, key: KeyCode) -> bool {
    let mode = self.input_mode.unwrap_or(InputMode::Search);
    let Some(input) = self.input.as_mut() else {
      return false;
    };
    match key {
      KeyCode::Char(c) if accepts_input_char(mode, input, c) => input.push(c),
      KeyCode::Backspace => {
        input.pop();
      }
      KeyCode::Esc => {
        self.input = None;
        self.input_mode = None;
        if mode == InputMode::Search {
          self.query.clear();
          self.packages.clear();
          self.aur.clear();
          self.menu = MenuState::default();
        }
      }
      KeyCode::Enter => {
        let candidate = self.input.take().unwrap_or_default();
        self.input_mode = None;
        match mode {
          InputMode::Search => self.apply_search_input(candidate),
          InputMode::MirrorAge | InputMode::MirrorCount => {
            self.apply_number_input(mode, &candidate)
          }
        }
      }
      _ => {}
    }
    false
  }

  /// Confirms the `/` search field.
  pub(super) fn apply_search_input(&mut self, candidate: String) {
    let minimum = if self.on_aur_search() { 2 } else { 1 };
    if candidate.trim().chars().count() < minimum {
      self.error(if self.on_aur_search() {
        tr(
          self.lang,
          "control_center.aur_search_requires_at_least_2_characters",
        )
      } else {
        tr(self.lang, "control_center.enter_a_non_empty_search_query")
      });
      return;
    }
    self.query = candidate;
    self.menu = MenuState::default();
    self.reload()
  }

  /// Confirms the numeric field of a reflector option, within its bounds.
  pub(super) fn apply_number_input(&mut self, mode: InputMode, candidate: &str) {
    let (min, max) = if mode == InputMode::MirrorAge {
      MIRROR_AGE_RANGE
    } else {
      MIRROR_COUNT_RANGE
    };
    match candidate.parse::<u32>() {
      Ok(value) if (min..=max).contains(&value) => {
        if let Some(options) = self.mirror_options.as_mut() {
          if mode == InputMode::MirrorAge {
            options.age_hours = value;
          } else {
            options.count = value;
          }
        }
      }
      _ => self.error(
        tr(self.lang, "control_center.value_must_be_between")
          .replace("{min}", &min.to_string())
          .replace("{max}", &max.to_string()),
      ),
    }
  }
}

/// Whether the open field takes `character`: the search takes any text up
/// to its limit; the reflector numbers take digits only.
pub(super) fn accepts_input_char(mode: InputMode, input: &str, character: char) -> bool {
  match mode {
    InputMode::Search => !character.is_control() && input.len() < QUERY_MAX,
    InputMode::MirrorAge => character.is_ascii_digit() && input.len() < 4,
    InputMode::MirrorCount => character.is_ascii_digit() && input.len() < 3,
  }
}

/// `value` moved by `step`, kept within `(min, max)`.
pub(super) fn step_within(value: u32, step: i32, (min, max): (u32, u32)) -> u32 {
  value.saturating_add_signed(step).clamp(min, max)
}
