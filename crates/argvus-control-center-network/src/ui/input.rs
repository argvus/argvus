//! Keys and text input on the Network & Wi-Fi page.

use super::*;

impl NetworkApp {
  /// The menu row under the cursor, if any.
  pub(super) fn selected_item(&self) -> Option<Item> {
    let rows = self.rows();
    let mut menu = self.menu;
    menu.normalize(&rows);
    menu.selected_id(&rows)
  }

  /// Whether typed characters currently go to text (Wi-Fi password, manual
  /// DNS or the list filter), so `q`/`?` must not act as the global quit/help
  /// keys. Mirrors the precedence of [`Self::handle`]: the forget confirmation
  /// is not typing.
  pub fn captures_text(&self) -> bool {
    if self.editor_confirm.is_some() {
      return false;
    }
    self.text_edit.is_some()
      || (!self.confirm_forget
        && (self.password.is_some() || self.dns_input.is_some() || self.search.is_some()))
  }

  /// Handles a key with its modifiers: the text fields of the connection
  /// pages take them (Ctrl+S saves); everything else goes to [`Self::handle`].
  /// Returns `true` when the Network page should close.
  pub fn handle_key(&mut self, key: KeyEvent) -> bool {
    if self.editor_confirm.is_none()
      && let Some((target, editor)) = &mut self.text_edit
    {
      match editor.handle(key) {
        EditorOutcome::Pending => {}
        EditorOutcome::Cancelled => self.text_edit = None,
        EditorOutcome::Saved(value) => {
          let target = *target;
          self.text_edit = None;
          self.apply_text(target, value);
        }
      }
      return false;
    }
    self.handle(key.code)
  }

  /// Pasted text goes to the open text field.
  pub fn paste(&mut self, text: &str) {
    if let Some((_, editor)) = &mut self.text_edit {
      editor.paste(text);
    }
  }

  pub fn handle(&mut self, key: KeyCode) -> bool {
    if let Some((confirm, state)) = &mut self.editor_confirm {
      let confirm = *confirm;
      match state.handle(key) {
        ConfirmOutcome::Confirmed => {
          self.editor_confirm = None;
          match confirm {
            EditorConfirm::Discard => self.close_editor(),
            EditorConfirm::SaveActive => self.perform_save(),
            EditorConfirm::Delete => self.perform_delete(),
          }
        }
        ConfirmOutcome::Cancelled => self.editor_confirm = None,
        ConfirmOutcome::Pending => {}
      }
      return false;
    }
    if self.text_edit.is_some() {
      return self.handle_key(KeyEvent::from(key));
    }
    if self.confirm_forget {
      match self.confirmation.handle(key) {
        ConfirmOutcome::Confirmed => {
          self.confirm_forget = false;
          self.confirmation = ConfirmState::new();
          self.perform_forget();
        }
        ConfirmOutcome::Cancelled => {
          self.confirm_forget = false;
          self.confirmation = ConfirmState::new();
        }
        ConfirmOutcome::Pending => {}
      }
      return false;
    }
    if let Some(password) = &mut self.password {
      match key {
        KeyCode::Esc => {
          self.password = None;
          self.pending_wifi = None;
        }
        KeyCode::Backspace => {
          password.pop();
        }
        KeyCode::Char(c) => password.push(c),
        KeyCode::Enter => {
          let secret = std::mem::take(password);
          let index = self.pending_wifi.take();
          self.password = None;
          if let Some(index) = index {
            self.run_wifi(index, Some(secret));
          }
        }
        _ => {}
      }
      return false;
    }
    if let Some(input) = &mut self.dns_input {
      match key {
        KeyCode::Esc => self.dns_input = None,
        KeyCode::Backspace => {
          input.pop();
        }
        KeyCode::Char(c) if !c.is_control() => input.push(c),
        KeyCode::Enter => {
          let value = std::mem::take(input);
          self.dns_input = None;
          self.apply_dns(&value);
        }
        _ => {}
      }
      return false;
    }
    if let Some(search) = &mut self.search {
      match key {
        KeyCode::Esc => self.search = None,
        KeyCode::Backspace => {
          search.pop();
        }
        KeyCode::Char(c) => search.push(c),
        KeyCode::Enter => {}
        _ => {}
      }
      self.menu = MenuState::default();
      return false;
    }
    let rows = self.rows();
    let page_size = usize::from(self.list_height.max(1));
    match key {
      KeyCode::Char('r') => self.start_refresh(self.page == NetworkPage::Wifi),
      KeyCode::Char('/') if self.is_list_page() => self.search = Some(String::new()),
      KeyCode::Char('a')
        if self.page == NetworkPage::Connections
          && matches!(self.selected_item(), Some(Item::Profile(_))) =>
      {
        self.toggle_selected_profile()
      }
      KeyCode::Char('i')
        if matches!(
          self.selected_item(),
          Some(Item::WifiNetwork(_) | Item::VpnConnection(_))
        ) =>
      {
        self.open_details()
      }
      _ => match self.menu.handle(key, &rows, page_size) {
        MenuEvent::Back => return self.back(),
        MenuEvent::Activate(item) | MenuEvent::Toggle(item) | MenuEvent::Confirm(item) => {
          self.activate(item)
        }
        MenuEvent::Adjust(item, step) if self.page.is_editor() => self.editor_adjust(item, step),
        MenuEvent::Adjust(..) | MenuEvent::Moved | MenuEvent::None => {}
      },
    }
    false
  }

  /// Leaves a page for its parent and puts the cursor back on the row that
  /// opened it. Returns `true` when the Network page itself should close.
  pub(super) fn back(&mut self) -> bool {
    if self.page == NetworkPage::Home {
      return true;
    }
    if self.page.is_editor() {
      self.editor_back();
      return false;
    }
    let from = self.page;
    let parent = match from {
      NetworkPage::Detail(_) | NetworkPage::WifiDetail(_) | NetworkPage::VpnDetail(_) => {
        self.detail_parent
      }
      NetworkPage::AddConnection => NetworkPage::Connections,
      _ => NetworkPage::Home,
    };
    let opener = match from {
      NetworkPage::Detail(i) => Item::Interface(i),
      NetworkPage::WifiDetail(i) => Item::WifiNetwork(i),
      NetworkPage::VpnDetail(i) => Item::VpnConnection(i),
      other => Item::Open(other),
    };
    self.page = parent;
    self.menu = MenuState::default();
    self.search = None;
    let rows = self.rows();
    self.menu.select(&rows, &opener);
    false
  }
}
