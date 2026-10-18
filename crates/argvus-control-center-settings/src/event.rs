//! Implements input event normalization and dispatch in crate `argvus control center settings`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use argvus_tui::confirm::ConfirmOutcome;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::administration::is_user_form;
use crate::app::App;
use crate::navigation::Page;

/// Processes `handle` in this module's event flow. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn handle(app: &mut App, event: Event) {
  match event {
    Event::Resize(width, height) => app.resize(width, height),
    Event::Key(key) if key.kind == KeyEventKind::Press => handle_key(app, key),
    _ => {}
  }
}

/// Processes `handle_key` in this module's event flow. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn handle_key(app: &mut App, key: KeyEvent) {
  if app.task_open {
    match key.code {
      KeyCode::Esc => {
        app.task_open = false;
        app.task_live = None;
        app.task_scroll = 0;
        app.task_follow = false;
      }
      KeyCode::Up | KeyCode::Char('k') => {
        app.task_follow = false;
        app.task_scroll = app.task_scroll.saturating_sub(1);
      }
      KeyCode::Down | KeyCode::Char('j') => {
        app.task_follow = false;
        app.task_scroll = app.task_scroll.saturating_add(1);
      }
      KeyCode::PageUp => {
        app.task_follow = false;
        app.task_scroll = app.task_scroll.saturating_sub(10);
      }
      KeyCode::PageDown => {
        app.task_follow = false;
        app.task_scroll = app.task_scroll.saturating_add(10);
      }
      KeyCode::Home => {
        app.task_follow = false;
        app.task_scroll = 0;
      }
      KeyCode::End => {
        if let Some(live) = &app.task_live {
          app.task_scroll = crate::app::task_bottom_offset(&live.output());
        }
      }
      _ => {}
    }
    return;
  }

  if app.confirm.is_some() {
    app.confirm_key(key.code);
    return;
  }

  if app.error_modal.is_some() {
    if matches!(key.code, KeyCode::Enter | KeyCode::Esc) {
      app.error_modal = None;
    }
    return;
  }

  if app.hostname_editing {
    app.hostname_input(key.code);
    return;
  }

  if app.admin.editor.is_some() {
    app.admin_input(key);
    return;
  }

  if matches!(
    app.page(),
    Page::Keybindings | Page::KeybindingEdit | Page::KeybindingCapture
  ) && app.has_keybinding_conflict()
  {
    // The conflict uses the single confirmation; `r` keeps replacing.
    if key.code == KeyCode::Char('r') {
      app.replace_keybinding_conflict();
      return;
    }
    match app.confirm_focus.handle(key.code) {
      ConfirmOutcome::Confirmed => app.replace_keybinding_conflict(),
      ConfirmOutcome::Cancelled => app.cancel_keybinding_conflict(),
      ConfirmOutcome::Pending => {}
    }
    return;
  }

  if matches!(app.page(), Page::KeybindingEdit | Page::KeybindingCapture)
    && (app.is_keybinding_capturing() || key.code == KeyCode::Char('e'))
  {
    if !app.is_keybinding_capturing() {
      app.begin_keybinding_capture();
    } else {
      app.capture_keybinding(key);
    }
    return;
  }

  if app.searching {
    match key.code {
      KeyCode::Esc => app.back(),
      KeyCode::Enter => {
        app.searching = false;
        app.open_or_apply();
      }
      KeyCode::Backspace => app.pop_search(),
      KeyCode::Down | KeyCode::Up => app.handle_menu_key(key.code),
      KeyCode::Char(character) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
        app.push_search(character)
      }
      _ => {}
    }
    return;
  }

  if app.page() == Page::MouseTouchpad {
    // `←/→` and `h/l` step any Mouse & Touchpad row, as before.
    let direction = match key.code {
      KeyCode::Left | KeyCode::Char('h') => Some(-1),
      KeyCode::Right | KeyCode::Char('l') => Some(1),
      _ => None,
    };
    if let Some(direction) = direction {
      if let Some(item) = app.selected_item() {
        app.input_cycle(item, direction);
      }
      return;
    }
  }

  handle_regular_key(app, key);
}

/// Page shortcuts first, then the single menu list.
fn handle_regular_key(app: &mut App, key: KeyEvent) {
  let page = app.page();
  // The action bar of Keyboard > Layout and Time Zone: `←/→` step through its
  // buttons while it has the focus, and from the list they move the focus to
  // it, as `Tab` does. Enter/Space run the focused button. A list key gives the
  // list the focus back and is then handled as usual below.
  if app.has_action_bar() {
    let focused = app.action_cursor().is_some();
    match key.code {
      // On the list, `←/→` reach the bar instead of toggling the row under the
      // cursor; with Apply dimmed there is nothing to focus and they do nothing.
      KeyCode::Left | KeyCode::Char('h') | KeyCode::Right | KeyCode::Char('l') => {
        let backwards = matches!(key.code, KeyCode::Left | KeyCode::Char('h'));
        if focused {
          app.move_action(backwards);
        } else {
          app.toggle_action_focus();
        }
        return;
      }
      KeyCode::Enter | KeyCode::Char(' ') if focused => return app.activate_action(),
      KeyCode::Up
      | KeyCode::Down
      | KeyCode::Char('j')
      | KeyCode::Char('k')
      | KeyCode::PageUp
      | KeyCode::PageDown
      | KeyCode::Home
      | KeyCode::End
        if focused =>
      {
        app.focus_list()
      }
      _ => {}
    }
  }
  match key.code {
    KeyCode::Char('/') => return app.begin_search(),
    KeyCode::Char('r') => {
      match page {
        Page::Hostname => app.refresh_system(),
        Page::Notifications => app.refresh_dnd(),
        Page::UserList | Page::SystemUsers | Page::GroupList | Page::SystemGroups => {
          app.admin.load(false)
        }
        _ => app.reset_current(),
      }
      return;
    }
    KeyCode::Char('+') | KeyCode::Char('=') => return app.adjust_size(1),
    KeyCode::Char('-') => return app.adjust_size(-1),
    // Tab moves focus between the info list and the action buttons.
    KeyCode::Tab | KeyCode::BackTab if is_user_form(page) => return app.toggle_user_focus(),
    KeyCode::Tab | KeyCode::BackTab if app.has_action_bar() => {
      return app.toggle_action_focus();
    }
    // Tab only switches tabs or panes; Settings has none.
    KeyCode::Tab | KeyCode::BackTab => return,
    // Enter sets the default layout; Space (a Toggle) enables or disables it.
    KeyCode::Enter if page == Page::KeyboardLayout => {
      if let Some(item) = app.selected_item() {
        app.activate(item);
      }
      return;
    }
    // Space flips the shortcut under the cursor.
    KeyCode::Char(' ') if page == Page::Keybindings => return app.toggle_selected_keybinding(),
    _ => {}
  }
  // User page, cfdisk style: Up/Down move the info list, Left/Right move
  // across the action buttons, Enter runs the focused one.
  if is_user_form(page) {
    match key.code {
      KeyCode::Left | KeyCode::Char('h') => return app.move_user_button(true),
      KeyCode::Right | KeyCode::Char('l') => return app.move_user_button(false),
      KeyCode::Enter | KeyCode::Char(' ') if app.user_button_cursor().is_some() => {
        return app.activate_user_button();
      }
      KeyCode::Up
      | KeyCode::Down
      | KeyCode::PageUp
      | KeyCode::PageDown
      | KeyCode::Home
      | KeyCode::End
        if app.user_button_cursor().is_some() =>
      {
        app.focus_user_info()
      }
      _ => {}
    }
  }

  let code = match key.code {
    // Space keeps activating any row where it did before (account and
    // firewall pages, shortcut editor, Mouse & Touchpad).
    KeyCode::Char(' ')
      if crate::administration::is_page(page)
        || matches!(page, Page::KeybindingEdit | Page::MouseTouchpad) =>
    {
      KeyCode::Enter
    }
    KeyCode::Char('h') => KeyCode::Left,
    KeyCode::Char('l') => KeyCode::Right,
    code => code,
  };
  app.handle_menu_key(code);
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  /// Executes the `resize_is_forwarded` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn resize_is_forwarded() {
    let mut app = App::new(Page::Main);
    handle(&mut app, Event::Resize(100, 30));
    assert_eq!((app.width, app.height), (100, 30));
  }
}
