//! Current page, entering and resetting pages, opening system pages and the breadcrumb of every settings page.

use super::labels::{category_label, font_target_label, setting_label};
use super::*;

impl App {
  pub fn page(&self) -> Page {
    self.navigation.current().page
  }

  pub fn breadcrumb(&self) -> String {
    let root = root_label(self.lang);
    // Every Home card owns a root segment, so a page's breadcrumb follows the
    // card it is listed under instead of the location it had before. The card
    // names come from `HomeCardId`, the same source the Home and search use.
    let category_root = |label: &str| format!("{root} > {label}");
    let desktop_input_root = category_root(HomeCardId::DesktopInput.label(self.lang));
    let appearance_root = category_root(HomeCardId::Appearance.label(self.lang));
    // Fonts is reached from Appearance > Desktop Style.
    let fonts_root = format!(
      "{appearance_root} > {}",
      tr(self.lang, "control_center.desktop_style")
    );
    let tools_root = category_root(HomeCardId::Tools.label(self.lang));
    let system_root = category_root(HomeCardId::System.label(self.lang));
    let language_root = format!(
      "{root} > {} > {}",
      HomeCardId::Preferences.label(self.lang),
      tr(self.lang, "control_center.language_region")
    );
    let users_groups_root = format!(
      "{system_root} > {}",
      tr(self.lang, "control_center.users_groups")
    );
    let preferences_root = category_root(HomeCardId::Preferences.label(self.lang));
    let settings_root = category_root(tr(self.lang, "control_center.settings"));
    match self.page() {
      Page::Main => root.to_string(),
      Page::DefaultApps => format!(
        "{preferences_root} > {}",
        tr(self.lang, "control_center.default_apps")
      ),
      Page::AppSelector(category) => format!(
        "{preferences_root} > {} > {}",
        tr(self.lang, "control_center.default_apps"),
        category_label(self.lang, category)
      ),
      Page::Fonts => format!("{fonts_root} > {}", tr(self.lang, "control_center.fonts")),
      Page::FontSelector(target) => format!(
        "{fonts_root} > {} > {}",
        tr(self.lang, "control_center.fonts"),
        font_target_label(self.lang, target)
      ),
      Page::SettingSelector(setting) => format!(
        "{fonts_root} > {} > {}",
        tr(self.lang, "control_center.fonts"),
        setting_label(self.lang, setting)
      ),
      // The former Locale & Region hub. The Home opens its children directly;
      // the hub stays reachable through the `locale` CLI route.
      Page::LocaleRegion => language_root,
      Page::MouseTouchpad => format!(
        "{desktop_input_root} > {}",
        tr(self.lang, "control_center.mouse_touchpad")
      ),
      Page::TimeZone => format!(
        "{language_root} > {}",
        tr(self.lang, "control_center.time_zone")
      ),
      Page::DateTime => format!(
        "{preferences_root} > {}",
        tr(self.lang, "control_center.date_time")
      ),
      // The first Language & Region entry (formerly Regional Locale).
      Page::RegionalLocale => format!(
        "{language_root} > {}",
        tr(self.lang, "control_center.language")
      ),
      Page::SystemLocales => format!(
        "{language_root} > {}",
        tr(self.lang, "control_center.system_locales")
      ),
      Page::Keyboard => format!(
        "{language_root} > {}",
        tr(self.lang, "control_center.keyboard")
      ),
      Page::Keybindings => format!(
        "{desktop_input_root} > {}",
        tr(self.lang, "control_center.keyboard_shortcuts")
      ),
      // Window rules is listed only under the Hyprland category.
      Page::WindowRules => format!(
        "{desktop_input_root} > {} > {}",
        tr(self.lang, "control_center.hyprland"),
        tr(self.lang, "control_center.window_rules")
      ),
      Page::Projects => format!(
        "{tools_root} > {}",
        tr(self.lang, "control_center.projects")
      ),
      Page::Snippets => format!(
        "{tools_root} > {}",
        tr(self.lang, "control_center.snippets")
      ),
      Page::SshKeys => format!(
        "{tools_root} > {}",
        tr(self.lang, "control_center.ssh_keys")
      ),
      Page::GpgKeys => format!(
        "{tools_root} > {}",
        tr(self.lang, "control_center.gpg_keys")
      ),
      Page::GpgKeyEdit => format!(
        "{tools_root} > {} > {}",
        tr(self.lang, "control_center.gpg_keys"),
        tr(self.lang, "control_center.gpg_keys_edit"),
      ),
      Page::GpgEditExpiration => format!(
        "{tools_root} > {} > {}",
        tr(self.lang, "control_center.gpg_keys"),
        tr(self.lang, "control_center.gpg_keys_expiration"),
      ),
      Page::GpgEditSign => format!(
        "{tools_root} > {} > {}",
        tr(self.lang, "control_center.gpg_keys"),
        tr(self.lang, "control_center.gpg_keys_edit_sign"),
      ),
      Page::GpgEditRevoke => format!(
        "{tools_root} > {} > {}",
        tr(self.lang, "control_center.gpg_keys"),
        tr(self.lang, "control_center.gpg_keys_edit_revoke"),
      ),
      Page::GpgFormExpiration => format!(
        "{tools_root} > {} > {}",
        tr(self.lang, "control_center.gpg_keys"),
        tr(self.lang, "control_center.gpg_keys_expiration"),
      ),
      Page::GpgFormPassphrase => format!(
        "{tools_root} > {} > {}",
        tr(self.lang, "control_center.gpg_keys"),
        tr(self.lang, "control_center.gpg_keys_passphrase_page"),
      ),
      Page::GpgKeyList => format!(
        "{tools_root} > {} > {}",
        tr(self.lang, "control_center.gpg_keys"),
        tr(self.lang, "control_center.gpg_keys_list"),
      ),
      Page::KeybindingEdit => format!(
        "{desktop_input_root} > {} > {}",
        tr(self.lang, "control_center.keyboard_shortcuts"),
        tr(self.lang, "control_center.edit_shortcut")
      ),
      Page::KeybindingCapture => format!(
        "{desktop_input_root} > {} > {}",
        tr(self.lang, "control_center.keyboard_shortcuts"),
        tr(self.lang, "control_center.change_shortcut")
      ),
      Page::KeyboardLayout => format!(
        "{language_root} > {}",
        tr(self.lang, "control_center.keyboard_layout")
      ),
      Page::KeyboardVariant => format!(
        "{language_root} > {} > {}",
        tr(self.lang, "control_center.keyboard"),
        tr(self.lang, "control_center.variant")
      ),
      Page::ConsoleKeymap => format!(
        "{language_root} > {} > {}",
        tr(self.lang, "control_center.keyboard"),
        tr(self.lang, "control_center.console_keymap")
      ),
      // Preferences keeps the interface language; Language & Region > Language
      // is the regional locale page.
      Page::Language => format!(
        "{settings_root} > {}",
        tr(self.lang, "control_center.language")
      ),
      Page::UsersGroups => users_groups_root,
      Page::Notifications => format!(
        "{system_root} > {}",
        tr(self.lang, "control_center.notifications")
      ),
      Page::Firewall => format!(
        "{system_root} > {}",
        tr(self.lang, "control_center.firewall")
      ),
      Page::Users => format!(
        "{users_groups_root} > {}",
        tr(self.lang, "control_center.users")
      ),
      Page::UserList => format!(
        "{users_groups_root} > {} > {}",
        tr(self.lang, "control_center.users"),
        tr(self.lang, "control_center.list")
      ),
      Page::SystemUsers => format!(
        "{users_groups_root} > {} > {}",
        tr(self.lang, "control_center.users"),
        tr(self.lang, "control_center.system_accounts")
      ),
      Page::User
      | Page::CreateUser
      | Page::UserGroups
      | Page::UserPassword
      | Page::UserShell
      | Page::UserPrimaryGroup
      | Page::UserAvatar
      | Page::UserUsername
      | Page::UserFullName
      | Page::UserDelete => format!(
        "{users_groups_root} > {}",
        tr(self.lang, "control_center.users")
      ),
      Page::Groups => format!(
        "{users_groups_root} > {}",
        tr(self.lang, "control_center.groups")
      ),
      Page::GroupList => format!(
        "{users_groups_root} > {} > {}",
        tr(self.lang, "control_center.groups"),
        tr(self.lang, "control_center.list")
      ),
      Page::SystemGroups => format!(
        "{users_groups_root} > {} > {}",
        tr(self.lang, "control_center.groups"),
        tr(self.lang, "control_center.system_accounts")
      ),
      Page::Group | Page::GroupMembers | Page::CreateGroup => format!(
        "{users_groups_root} > {}",
        tr(self.lang, "control_center.groups")
      ),
      Page::Hostname => format!(
        "{system_root} > {}",
        tr(self.lang, "control_center.hostname")
      ),
    }
  }

  pub fn reset_page(&self) -> bool {
    matches!(
      self.page(),
      Page::DefaultApps
        | Page::Fonts
        | Page::AppSelector(_)
        | Page::FontSelector(_)
        | Page::SettingSelector(_)
        | Page::Keybindings
    )
  }

  /// Prepares the state that a page needs as soon as it is opened.
  pub fn enter_page(&mut self, page: Page) {
    self.admin.ensure_loaded(page);
    match page {
      Page::UsersGroups if !self.admin.busy() && !self.admin.is_loaded() => {
        self.admin.load(false);
      }
      Page::Notifications => self.refresh_dnd(),
      _ => {}
    }
  }

  pub(super) fn open_system_page(&mut self, page: Page) {
    if page == Page::SystemLocales && !self.system_locales_available() {
      let name = if self.distro.pretty_name.is_empty() {
        self.distro.id.clone()
      } else {
        self.distro.pretty_name.clone()
      };
      self.fail(format!(
        "{}: {name}",
        tr(
          self.lang,
          "control_center.locales_backend_is_implemented_only_for_arch_linux"
        )
      ));
      return;
    }
    self.navigation.push(page);
    self.select_current();
  }
}
