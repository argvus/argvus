//! Implements page navigation and selection rules in crate `argvus control center settings`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use argvus_control_center_apps::catalog::Category;
use argvus_tui::action_buttons::ActionFocus;
use argvus_tui::menu::MenuState;

use crate::config::fonts::{FontTarget, SettingKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Defines `Page`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub enum Page {
  Main,
  DefaultApps,
  AppSelector(Category),
  Fonts,
  FontSelector(FontTarget),
  SettingSelector(SettingKind),
  LocaleRegion,
  TimeZone,
  DateTime,
  RegionalLocale,
  SystemLocales,
  Keyboard,
  Keybindings,
  KeybindingEdit,
  KeybindingCapture,
  WindowRules,
  Projects,
  Snippets,
  SshKeys,
  GpgKeys,
  /// Expiry date and "never expires" of the GPG generate form.
  GpgFormExpiration,
  /// Passphrase and its confirmation of the GPG generate form.
  GpgFormPassphrase,
  /// The keyring, listed one block per key.
  GpgKeyList,
  /// One key: its details and the actions that change it.
  GpgKeyEdit,
  /// Expiry date of the key being edited.
  GpgEditExpiration,
  /// The key that certifies the key being edited.
  GpgEditSign,
  /// The reason of the revocation of the key being edited.
  GpgEditRevoke,
  MouseTouchpad,
  KeyboardLayout,
  KeyboardVariant,
  ConsoleKeymap,
  Language,
  Hostname,
  /// Users & Groups hub: Users and Groups submenus.
  UsersGroups,
  /// Notifications: Do Not Disturb.
  Notifications,
  Firewall,
  Users,
  UserList,
  SystemUsers,
  User,
  CreateUser,
  UserGroups,
  UserPassword,
  UserShell,
  UserPrimaryGroup,
  UserAvatar,
  UserUsername,
  UserFullName,
  UserDelete,
  Groups,
  GroupList,
  SystemGroups,
  Group,
  GroupMembers,
  CreateGroup,
}

#[derive(Debug, Clone, Copy)]
/// Represents `Location`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct Location {
  pub page: Page,
  /// Cursor and scroll of the page's menu list, restored when going back.
  pub menu: MenuState,
  /// Whether the page's action bar has the focus, and on which button.
  pub actions: ActionFocus,
}

impl Location {
  /// Executes the const function documented in this module. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
  pub fn new(page: Page) -> Self {
    Self {
      page,
      menu: MenuState::default(),
      actions: ActionFocus::default(),
    }
  }
}

#[derive(Debug)]
/// Represents `Navigation`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct Navigation {
  current: Location,
  stack: Vec<Location>,
}

impl Navigation {
  /// Constructs `new` with this module's expected initial state. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn new(page: Page) -> Self {
    Self {
      current: Location::new(page),
      stack: if page == Page::Main {
        Vec::new()
      } else {
        vec![Location::new(Page::Main)]
      },
    }
  }

  /// Executes the const function documented in this module. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
  pub const fn current(&self) -> Location {
    self.current
  }

  /// Executes the `current_mut` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn current_mut(&mut self) -> &mut Location {
    &mut self.current
  }

  /// Executes the `push` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn push(&mut self, page: Page) {
    self.stack.push(self.current);
    self.current = Location::new(page);
  }

  /// Executes the `back` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn back(&mut self) -> bool {
    if let Some(previous) = self.stack.pop() {
      self.current = previous;
      true
    } else {
      false
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  /// Executes the `stack_restores_page_selection_and_scroll` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn stack_restores_page_selection_and_scroll() {
    use argvus_tui::menu::Row;
    let rows: Vec<Row<u8>> = (0..4).map(|id| Row::action(id, "row")).collect();
    let mut navigation = Navigation::new(Page::Main);
    assert!(navigation.current_mut().menu.select(&rows, &2));
    navigation.push(Page::Fonts);
    assert_eq!(navigation.current().page, Page::Fonts);
    assert_eq!(navigation.current().menu.selected_index(), None);
    assert!(navigation.back());
    assert_eq!(navigation.current().page, Page::Main);
    assert_eq!(navigation.current().menu.selected_id(&rows), Some(2));
    assert!(!navigation.back());
  }
}
