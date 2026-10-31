//! Page navigation and selection rules of the Settings pages.
use argvus_control_center_apps::catalog::Category;
use argvus_tui::action_buttons::ActionFocus;
use argvus_tui::menu::MenuState;

use crate::config::fonts::{FontTarget, SettingKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

impl Page {
  /// Pages whose configuration includes a system-wide, elevated operation.
  pub fn requires_admin(self) -> bool {
    matches!(
      self,
      Self::TimeZone
        | Self::DateTime
        | Self::SystemLocales
        | Self::ConsoleKeymap
        | Self::Hostname
        | Self::Firewall
        | Self::User
        | Self::CreateUser
        | Self::UserGroups
        | Self::UserPassword
        | Self::UserShell
        | Self::UserPrimaryGroup
        | Self::UserAvatar
        | Self::UserUsername
        | Self::UserFullName
        | Self::UserDelete
        | Self::Group
        | Self::GroupMembers
        | Self::CreateGroup
    )
  }
}

#[derive(Debug, Clone, Copy)]
pub struct Location {
  pub page: Page,
  /// Cursor and scroll of the page's menu list, restored when going back.
  pub menu: MenuState,
  /// Whether the page's action bar has the focus, and on which button.
  pub actions: ActionFocus,
}

impl Location {
  pub fn new(page: Page) -> Self {
    Self {
      page,
      menu: MenuState::default(),
      actions: ActionFocus::default(),
    }
  }
}

#[derive(Debug)]
pub struct Navigation {
  current: Location,
  stack: Vec<Location>,
}

impl Navigation {
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

  pub const fn current(&self) -> Location {
    self.current
  }

  pub fn current_mut(&mut self) -> &mut Location {
    &mut self.current
  }

  pub fn push(&mut self, page: Page) {
    self.stack.push(self.current);
    self.current = Location::new(page);
  }

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
