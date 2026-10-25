//! Menu navigation and row activation of the Settings pages.
//!
//! Keys reach the page's [`MenuState`]; the resulting [`MenuEvent`] carries
//! the row's [`Item`], and the handlers below keep each option's original
//! effect and timing (immediate, confirmed, or draft).
use crossterm::event::KeyCode;

use argvus_control_center_apps::catalog::Category;
use argvus_tui::hints::{HintContext, confirm_hints, hints};
use argvus_tui::menu::{MenuEvent, MenuState, Row, RowKind};

use super::keyboard_draft;
use super::rows::LANGUAGES;
use super::{App, PendingAction, Status, StatusKind, keybinding_is_cheatsheet_entry};
use super::{
  keybinding_category_order, keybinding_label, keybinding_section, keybinding_section_order,
  search_matches,
};
use crate::administration::{EditTarget, Editor};
use crate::config::fonts::SettingKind;
use crate::i18n::{Lang, tr};
use crate::item::{Item, RatbagRow};
use crate::navigation::Page;
use crate::system::date::{CalendarDate, DateOrder};
use crate::system::gpg_keys;
use crate::system::snippets;
use crate::system::ssh_keys;
use crate::system::window_rules;
use crate::system::{keybindings, keyboard, locale, time};
use argvus_control_center_core::clipboard;

/// Values offered by a font rendering setting, in display order.
pub(crate) fn setting_values(setting: SettingKind) -> &'static [&'static str] {
  match setting {
    SettingKind::Antialiasing => &["enabled", "disabled"],
    SettingKind::Hinting => &["none", "slight", "medium", "full"],
    SettingKind::Subpixel => &["none", "rgb", "bgr", "vrgb", "vbgr"],
    SettingKind::Dpi => &[
      "automatic",
      "72",
      "84",
      "96",
      "108",
      "120",
      "144",
      "168",
      "192",
      "216",
      "240",
    ],
  }
}

/// Pages whose list can be narrowed with `/`.
fn is_searchable(page: Page) -> bool {
  matches!(
    page,
    Page::DefaultApps
      | Page::AppSelector(_)
      | Page::FontSelector(_)
      | Page::TimeZone
      | Page::RegionalLocale
      | Page::SystemLocales
      | Page::KeyboardLayout
      | Page::KeyboardVariant
      | Page::ConsoleKeymap
      | Page::Keybindings
      | Page::SystemUsers
      | Page::GroupList
      | Page::SystemGroups
      | Page::SshKeys
      | Page::GpgKeyList
  )
}

impl App {
  /// Keeps the cursor of the current page on a selectable row.
  pub fn normalize_selection(&mut self) {
    let rows = self.rows();
    self.navigation.current_mut().menu.normalize(&rows);
  }

  /// Identity of the selected row, if the page has one.
  pub fn selected_item(&self) -> Option<Item> {
    let rows = self.rows();
    let mut menu = self.navigation.current().menu;
    menu.normalize(&rows);
    menu.selected_id(&rows)
  }

  /// Kind of the selected row, used by the contextual footer.
  fn selected_kind(&self, rows: &[Row<Item>]) -> Option<RowKind> {
    let mut menu = self.navigation.current().menu;
    menu.normalize(rows);
    menu.selected_kind(rows)
  }

  /// Sends a key to the page's menu list and runs the resulting event.
  pub(crate) fn handle_menu_key(&mut self, key: KeyCode) {
    let mut rows = self.rows();
    if crate::administration::is_user_form(self.page()) {
      // The user page's list is the info block only; its buttons are driven
      // by Left/Right, so the cursor never walks into them.
      let actions_at = rows
        .iter()
        .position(|row| row.is_section())
        .unwrap_or(rows.len());
      rows.truncate(actions_at);
    }
    let page_size = self.viewport;
    let event = self
      .navigation
      .current_mut()
      .menu
      .handle(key, &rows, page_size);
    match event {
      MenuEvent::Activate(item) => self.activate(item),
      MenuEvent::Toggle(item) => self.toggle(item),
      MenuEvent::Adjust(item, delta) => self.adjust(item, delta),
      MenuEvent::Confirm(item) => self.confirm_item(item),
      MenuEvent::Back => self.back(),
      MenuEvent::Moved | MenuEvent::None => {}
    }
  }

  /// Runs the selected row, as Enter does.
  pub fn open_or_apply(&mut self) {
    self.handle_menu_key(KeyCode::Enter);
  }

  /// Enter/`→` on an Action, Submenu, Choice or Value row.
  pub(crate) fn activate(&mut self, item: Item) {
    if crate::administration::is_page(self.page()) {
      self.admin_activate(item);
      return;
    }
    match item {
      Item::DefaultApps => self.navigation.push(Page::DefaultApps),
      Item::Fonts => self.navigation.push(Page::Fonts),
      Item::LocaleRegion => self.navigation.push(Page::LocaleRegion),
      Item::System => self.navigation.push(Page::Hostname),
      Item::Category(category) => {
        self.navigation.push(Page::AppSelector(category));
        self.select_current();
      }
      Item::App(index) => {
        if let Page::AppSelector(category) = self.page() {
          self.apply_app(category, index);
        }
      }
      Item::FontTarget(target) => {
        self.pending_size = self.fonts.get(target).size;
        self.navigation.push(Page::FontSelector(target));
        self.select_current();
      }
      Item::FontSetting(setting) => {
        self.navigation.push(Page::SettingSelector(setting));
        self.select_current();
      }
      Item::FontSize => {
        self.admin.editor = Some(Editor::new(
          tr(self.lang, "control_center.font_size").into(),
          self.pending_size.to_string(),
          EditTarget::FontSize,
          false,
        ));
      }
      Item::Font(index) => {
        if let Page::FontSelector(target) = self.page() {
          self.apply_font(target, index);
        }
      }
      Item::SettingOption(index) => {
        if let Page::SettingSelector(setting) = self.page() {
          self.apply_font_setting(setting, index);
        }
      }
      Item::TimeZone => {
        // The draft starts on the system zone before the cursor is placed on it.
        self.load_timezone_draft();
        self.open_system_page(Page::TimeZone);
      }
      Item::DateTime => self.open_system_page(Page::DateTime),
      Item::RegionalLocale => self.open_system_page(Page::RegionalLocale),
      Item::SystemLocales => self.open_system_page(Page::SystemLocales),
      Item::Keyboard => self.open_system_page(Page::Keyboard),
      Item::WindowRuleName(index) => self.open_window_rule_name(index),
      Item::WindowRuleWorkspace(index) => self.adjust_window_rule(index, 1),
      Item::WindowRuleClasses(index) => self.open_window_rule_classes(index),
      Item::ApplyWindowRule(index) => self.apply_window_rule(index),
      Item::NewWindowRuleName => self.open_new_window_rule_name(),
      Item::AddWindowRule => self.add_window_rule(),
      Item::AddProjectPath => self.open_project_editor(false),
      Item::AddProjectRoot => self.open_project_editor(true),
      Item::NewSnippetName => self.open_new_snippet_name(),
      Item::NewSnippetContent => self.open_new_snippet_content(),
      Item::AddSnippet => self.add_snippet(),
      Item::OpenSnippetPicker => self.open_snippet_picker(),
      Item::SnippetContent(index) => self.open_snippet_content(index),
      Item::TypeSnippet(index) => self.type_snippet(index),
      Item::ImportGpgPrivate => self.confirm_gpg_import_private(),
      Item::ImportGpgKey => self.start_gpg_import_pick(gpg_keys::GpgImportKind::Public),
      Item::OpenGpgKeyEdit(index) => self.open_gpg_key_edit(index),
      Item::ExportGpgPrivate(index) => self.confirm_gpg_export_private(index),
      Item::CopyGpgPublic(index) => self.copy_gpg_public(index),
      Item::CopyGpgFingerprint(index) => self.copy_gpg_fingerprint(index),
      Item::OpenGpgEditExpiration => self.open_gpg_edit_expiration(),
      Item::GpgEditExpiry => self.open_gpg_field(
        "control_center.gpg_keys_expiry",
        self.gpg_edit_expiry.clone(),
        EditTarget::GpgEditExpiry,
      ),
      Item::ApplyGpgExpiry => self.apply_gpg_expiry(),
      Item::GpgEditPasswd => {
        if let Some(fingerprint) = self.gpg_edit_fingerprint.clone() {
          self.start_gpg_action("control_center.gpg_keys_passwd_changed", move || {
            gpg_keys::change_passphrase(&fingerprint)
          });
        }
      }
      Item::OpenGpgSign => self.navigation.push(Page::GpgEditSign),
      Item::GpgSignWith(index) => self.confirm_gpg_sign(index),
      Item::OpenGpgRevoke => self.navigation.push(Page::GpgEditRevoke),
      Item::GpgTrustWith(level) => {
        if let Some(fingerprint) = self.gpg_edit_fingerprint.clone() {
          self.start_gpg_action("control_center.gpg_keys_trust_changed", move || {
            gpg_keys::set_trust(&fingerprint, level)
          });
        }
      }
      Item::GpgKeyserverSend => {
        if let Some(fingerprint) = self.gpg_edit_fingerprint.clone() {
          self.open_confirm(PendingAction::SendGpgKey(fingerprint));
        }
      }
      Item::GpgKeyserverRecv => {
        if let Some(fingerprint) = self.gpg_edit_fingerprint.clone() {
          self.start_gpg_action("control_center.gpg_keys_received", move || {
            gpg_keys::keyserver_recv(&fingerprint)
          });
        }
      }
      Item::GpgFullName => self.open_gpg_field(
        "control_center.gpg_keys_full_name",
        self.gpg_form.full_name.clone(),
        EditTarget::GpgFullName,
      ),
      Item::GpgEmail => self.open_gpg_field(
        "control_center.gpg_keys_email",
        self.gpg_form.email.clone(),
        EditTarget::GpgEmail,
      ),
      Item::GpgComment => self.open_gpg_field(
        "control_center.gpg_keys_comment",
        self.gpg_form.comment.clone(),
        EditTarget::GpgComment,
      ),
      Item::GpgAlgorithm => self.gpg_form.toggle_algorithm(),
      Item::GpgKeySize => self.gpg_form.next_key_size(),
      Item::OpenGpgExpiration => self.navigation.push(Page::GpgFormExpiration),
      Item::GpgExpiry => self.open_gpg_field(
        "control_center.gpg_keys_expiry",
        self.gpg_form.expiry.clone(),
        EditTarget::GpgExpiry,
      ),
      Item::OpenGpgPassphrase => self.navigation.push(Page::GpgFormPassphrase),
      // Passphrase editors are masked by their target, so they use the same opener.
      Item::GpgPassphrase => self.open_gpg_field(
        "control_center.gpg_keys_passphrase",
        self.gpg_form.passphrase.clone(),
        EditTarget::GpgPassphrase,
      ),
      Item::GpgPassphraseConfirm => self.open_gpg_field(
        "control_center.gpg_keys_passphrase_confirm",
        self.gpg_form.passphrase_confirm.clone(),
        EditTarget::GpgPassphraseConfirm,
      ),
      Item::GenerateGpgKey => self.generate_gpg_key(),
      Item::CancelGpgForm => {
        let order = self.gpg_date_order();
        self.gpg_form.clear(order);
      }
      Item::OpenGpgKeys => self.navigation.push(Page::GpgKeyList),
      Item::ExportGpgKey(index) => self.open_gpg_export(index),
      Item::RefreshGpgKeys => {
        self.gpg_keys = gpg_keys::load();
        self.success(tr(self.lang, "control_center.gpg_keys_refreshed").to_string());
      }
      Item::NewSshKeyName => self.open_new_ssh_key_name(),
      Item::SshKeyType => self.toggle_ssh_key_type(),
      Item::GenerateSshKey => self.generate_ssh_key(),
      Item::CopySshPublicKey(index) => self.copy_ssh_public_key(index),
      Item::LoadSshKey(index) => self.load_ssh_key(index),
      Item::UnloadSshKey(index) => self.unload_ssh_key(index),
      Item::UnloadAllSshKeys => self.unload_all_ssh_keys(),
      Item::RefreshSshKeys => {
        self.ssh_keys = ssh_keys::load();
        self.success(tr(self.lang, "control_center.ssh_keys_refreshed").to_string());
      }
      Item::Zone(index) => self.select_timezone(index),
      Item::ApplyTimezone => self.apply_timezone(),
      Item::LocalDateTime if !self.datetime.ntp.unwrap_or(false) => {
        self.admin.editor = Some(Editor::new(
          tr(self.lang, "control_center.local_date_time").into(),
          self.datetime.local_time.clone(),
          EditTarget::DateTime,
          false,
        ));
      }
      Item::Locale(index) => self.apply_regional_locale(index),
      Item::ApplyLocales => self.open_confirm(PendingAction::ApplySystemLocales),
      Item::KeyboardLayout => self.open_system_page(Page::KeyboardLayout),
      Item::KeyboardVariant => self.open_system_page(Page::KeyboardVariant),
      Item::ConsoleKeymap => self.open_system_page(Page::ConsoleKeymap),
      Item::Layout(index) => self.apply_keyboard_layout(index),
      Item::ApplyKeyboardLayout => self.apply_keyboard_layout_draft(),
      Item::Variant(index) => self.apply_keyboard_variant(index),
      Item::Keymap(index) => self.apply_console_keymap(index),
      Item::Language(index) => self.apply_language(index),
      Item::Hostname => {
        self.hostname_editing = true;
        self.hostname_input = self.hostname.clone();
      }
      Item::Users => {
        self.admin.load(false);
        self.navigation.push(Page::Users);
      }
      Item::Groups => {
        self.admin.load(false);
        self.navigation.push(Page::Groups);
      }
      Item::Input(_) | Item::Ratbag(_) => self.apply_input(item),
      Item::Keybinding(index) => self.open_keybinding_edit(index),
      Item::ChangeShortcut => {
        self.keybinding_capturing = true;
        self.keybinding_detected = false;
        self.navigation.push(Page::KeybindingCapture);
      }
      Item::DisableShortcut => {
        self.keybinding_capturing = false;
        self.disable_edited_keybinding();
        self.navigation.back();
      }
      Item::RestoreShortcut => {
        self.keybinding_capturing = false;
        self.restore_edited_keybinding();
        self.navigation.back();
      }
      Item::ApplyCapture => {
        if self.keybinding_detected {
          self.apply_keybinding_editor();
        } else {
          self.keybinding_capturing = true;
        }
      }
      Item::CancelCapture => {
        self.keybinding_capturing = false;
        self.navigation.back();
      }
      _ => {}
    }
  }

  /// Enter/Space on a Toggle row.
  fn toggle(&mut self, item: Item) {
    if crate::administration::is_page(self.page()) {
      self.admin_activate(item);
      return;
    }
    match item {
      Item::Ntp => self.open_confirm(PendingAction::SetNtp(!self.datetime.ntp.unwrap_or(false))),
      Item::DoNotDisturb => self.toggle_dnd(),
      Item::LocaleGen(index) => self.toggle_system_locale(index),
      Item::Layout(index) => self.toggle_keyboard_layout(index),
      Item::Input(_) => self.apply_input(item),
      // A form field, not a setting: it changes only the draft form.
      Item::GpgNeverExpires => self.gpg_form.never_expires = !self.gpg_form.never_expires,
      Item::ToggleGpgSection(section) => {
        // `insert` reports whether the section was closed; a section that was
        // open is removed again.
        let was_closed = self.gpg_sections_open.insert(section);
        if !was_closed {
          self.gpg_sections_open.remove(&section);
        }
      }
      Item::GpgEditNeverExpires => self.gpg_edit_never = !self.gpg_edit_never,
      Item::GpgProtectPassphrase => {
        self.gpg_form.protect_with_passphrase = !self.gpg_form.protect_with_passphrase;
        // A key without protection must not keep a passphrase typed earlier.
        if !self.gpg_form.protect_with_passphrase {
          self.gpg_form.passphrase.clear();
          self.gpg_form.passphrase_confirm.clear();
        }
      }
      _ => {}
    }
  }

  /// `←/→` on a Value row with a step.
  fn adjust(&mut self, item: Item, delta: i32) {
    if crate::administration::is_page(self.page()) {
      self.admin_adjust(item, delta);
      return;
    }
    match item {
      Item::FontSize => self.adjust_size(delta.signum() as i16),
      Item::Input(_) | Item::Ratbag(_) => self.input_cycle(item, delta.signum() as i8),
      Item::WindowRuleWorkspace(index) => self.adjust_window_rule(index, delta),
      _ => {}
    }
  }

  /// Enter on a Destructive row: every one of them asks first.
  fn confirm_item(&mut self, item: Item) {
    if crate::administration::is_page(self.page()) {
      self.admin_activate(item);
      return;
    }
    match item {
      Item::ResetDefaults => self.reset_current(),
      Item::RestoreAllShortcuts => self.open_confirm(PendingAction::ResetKeybindings),
      Item::RemoveWindowRule(index) => {
        if let Some(rule) = self.window_rules.get(index) {
          let name = rule.name.clone();
          self.open_confirm(PendingAction::RemoveWindowRule(name));
        }
      }
      Item::RemoveProject(index) => {
        if let Some(entry) = self.projects.entries.get(index) {
          let path = entry.path.clone();
          self.open_confirm(PendingAction::RemoveProject(path));
        }
      }
      Item::RemoveSnippet(index) => {
        if let Some(entry) = self.snippets.entries.get(index) {
          let name = entry.name.clone();
          self.open_confirm(PendingAction::RemoveSnippet(name));
        }
      }
      // Destructive rows come here, so these ask before they run.
      Item::DeleteGpgKey(index) => self.confirm_gpg_delete(index),
      Item::GpgRevokeWith(reason) => {
        if let Some(fingerprint) = self.gpg_edit_fingerprint.clone() {
          self.open_confirm(PendingAction::RevokeGpgKey(fingerprint, reason));
        }
      }
      Item::DeleteSshKey(index) => {
        if let Some(entry) = self.ssh_keys.entries.get(index) {
          let name = entry.name.clone();
          self.open_confirm(PendingAction::DeleteSshKey(name));
        }
      }
      _ => {}
    }
  }

  /// The `r` shortcut. It restores defaults on the app and font pages and
  /// restores the selected shortcut on the shortcut pages, always through
  /// the confirmation (D6).
  pub fn reset_current(&mut self) {
    match self.page() {
      Page::DefaultApps => self.open_confirm(PendingAction::ResetApps),
      Page::AppSelector(category) => self.open_confirm(PendingAction::ResetApp(category)),
      Page::Fonts => self.open_confirm(PendingAction::ResetFonts),
      Page::FontSelector(target) => self.open_confirm(PendingAction::ResetFont(target)),
      Page::SettingSelector(setting) => {
        self.open_confirm(PendingAction::ResetFontSetting(setting));
      }
      Page::Keybindings => {
        if let Some(binding) = self.selected_keybinding() {
          let id = binding.id.clone();
          self.open_confirm(PendingAction::RestoreKeybinding { id, leave: false });
        }
      }
      Page::KeybindingEdit => {
        if let Some(id) = self.keybinding_edit_id.clone() {
          self.open_confirm(PendingAction::RestoreKeybinding { id, leave: true });
        }
      }
      _ => {}
    }
  }

  /// Whether the current page has an action bar under its list, with `Apply`.
  /// Such pages also keep their search box above the list.
  pub(crate) fn has_action_bar(&self) -> bool {
    matches!(
      self.page(),
      Page::DefaultApps
        | Page::KeyboardLayout
        | Page::TimeZone
        | Page::SystemLocales
        | Page::SshKeys
    )
  }

  /// Whether the page's search is a box above the list, as on the home
  /// screen, instead of the bottom line.
  pub(crate) fn search_on_top(&self) -> bool {
    // The GPG key list and the shortcuts list have no action bar, but their
    // search still sits on top, like the pages that have one.
    self.has_action_bar() || matches!(self.page(), Page::GpgKeyList | Page::Keybindings)
  }

  /// The action bar of the current page; empty for pages without one.
  pub(crate) fn action_rows(&self) -> Vec<Row<Item>> {
    match self.page() {
      Page::DefaultApps => self.default_apps_action_rows(),
      Page::KeyboardLayout => self.keyboard_action_rows(),
      Page::TimeZone => self.timezone_action_rows(),
      Page::SystemLocales => self.locales_action_rows(),
      Page::SshKeys => self.ssh_keys_action_rows(),
      _ => Vec::new(),
    }
  }

  /// Index of the focused action button, or `None` while the list has the focus.
  pub(crate) fn action_cursor(&self) -> Option<usize> {
    self.navigation.current().actions.cursor()
  }

  /// Indexes of the enabled buttons of the current page's action bar.
  fn enabled_action_indexes(&self) -> Vec<usize> {
    self
      .action_rows()
      .iter()
      .enumerate()
      .filter(|(_, row)| row.is_selectable())
      .map(|(index, _)| index)
      .collect()
  }

  /// `Tab`: moves the focus between the list and the action bar.
  pub(crate) fn toggle_action_focus(&mut self) {
    let enabled = self.enabled_action_indexes();
    self.navigation.current_mut().actions.toggle(&enabled);
  }

  /// `←/→` on the action bar: steps through its enabled buttons.
  pub(crate) fn move_action(&mut self, backwards: bool) {
    let enabled = self.enabled_action_indexes();
    self
      .navigation
      .current_mut()
      .actions
      .move_by(backwards, &enabled);
  }

  /// Gives the focus back to the list, as `↑/↓` do.
  pub(crate) fn focus_list(&mut self) {
    self.navigation.current_mut().actions.focus_list();
  }

  /// Enter/Space on the focused button: runs its row, as Enter on the row would.
  pub(crate) fn activate_action(&mut self) {
    let rows = self.action_rows();
    let Some(row) = self.action_cursor().and_then(|index| rows.get(index)) else {
      return;
    };
    if !row.is_selectable() {
      return;
    }
    if let Some(item) = row.id().copied() {
      // A destructive button asks first, as its row does in the list.
      if row.kind() == RowKind::Destructive {
        self.confirm_item(item);
      } else {
        self.activate(item);
      }
    }
  }

  /// Leaves the search, or goes back one page.
  pub fn back(&mut self) {
    if self.searching || !self.search.is_empty() {
      self.searching = false;
      self.search.clear();
      self.navigation.current_mut().menu = MenuState::default();
      self.select_current();
      return;
    }
    self.hostname_editing = false;
    if self.leaving_drops_draft() {
      self.open_confirm(PendingAction::DiscardDraft);
      return;
    }
    self.navigation.back();
  }

  /// Whether going back from the current page would drop unsaved changes.
  fn leaving_drops_draft(&self) -> bool {
    match self.page() {
      Page::SystemLocales => self.locales_changed(),
      Page::KeyboardLayout => self.keyboard_layout_changed(),
      Page::TimeZone => self.timezone_changed(),
      Page::WindowRules => self
        .window_rules
        .iter()
        .any(|rule| self.window_rule_changed(rule)),
      page => self.admin.leaving_drops_draft(page),
    }
  }

  /// Confirmed discard: the draft returns to the loaded state, then back.
  pub(super) fn discard_and_leave(&mut self) {
    match self.page() {
      Page::SystemLocales => {
        self.selected_locales = self
          .locale_gen_entries
          .iter()
          .filter(|entry| entry.enabled)
          .map(|entry| format!("{} {}", entry.locale, entry.encoding))
          .collect();
      }
      Page::KeyboardLayout => {
        self.keyboard_draft = keyboard_draft::LayoutDraft::loaded(&self.keyboard_info);
      }
      Page::TimeZone => self.load_timezone_draft(),
      Page::WindowRules => {
        self.window_rule_drafts.clear();
        self.new_window_rule_name.clear();
      }
      page => self.admin.discard_draft(page),
    }
    self.navigation.back();
  }

  pub fn begin_search(&mut self) {
    if is_searchable(self.page()) {
      self.searching = true;
      self.search.clear();
      self.navigation.current_mut().menu = MenuState::default();
    }
  }

  pub fn push_search(&mut self, character: char) {
    self.search.push(character);
    self.navigation.current_mut().menu = MenuState::default();
  }

  pub fn pop_search(&mut self) {
    self.search.pop();
    self.navigation.current_mut().menu = MenuState::default();
  }

  /// Records the visible list height, the PgUp/PgDn distance.
  pub fn set_viewport(&mut self, viewport: usize) {
    self.viewport = viewport.max(1);
  }

  /// Puts the cursor on the current choice of the page (the active theme
  /// of a list, the default keyboard layout...), or on its first row.
  pub fn select_current(&mut self) {
    let rows = self.rows();
    let current = if self.page() == Page::KeyboardLayout {
      let default_layout = self.keyboard_draft.default.clone();
      self
        .keyboard_layouts
        .iter()
        .position(|layout| layout.code == default_layout)
        .map(Item::Layout)
    } else {
      rows
        .iter()
        .find(|row| matches!(row.kind(), RowKind::Choice { current: true }))
        .and_then(|row| row.id().copied())
    };
    let menu = &mut self.navigation.current_mut().menu;
    if !current.is_some_and(|item| menu.select(&rows, &item)) {
      menu.normalize(&rows);
    }
  }

  /// Footer derived from the selected row and the page's shortcuts.
  pub fn footer(&self) -> String {
    if self.confirm.is_some() {
      return confirm_hints(self.lang);
    }
    if self.has_keybinding_conflict() {
      return format!(
        "{}   r {}",
        confirm_hints(self.lang),
        tr(self.lang, "control_center.replace")
      );
    }
    if self.action_cursor().is_some() {
      // The bar's title already names `←/→`, `Enter` and `Tab`; the footer
      // shows what the focused button does and how to go back.
      return hints(
        self.lang,
        &HintContext {
          row: Some(RowKind::Action),
          can_go_back: true,
          ..HintContext::default()
        },
      );
    }
    if self.searching {
      return tr(
        self.lang,
        "control_center.type_to_search_enter_apply_esc_cancel",
      )
      .into();
    }
    let label = |key: &str| tr(self.lang, key);
    let page = self.page();
    let rows = self.rows();
    let mut row = self.selected_kind(&rows);
    let mut extra = Vec::new();
    match page {
      Page::DefaultApps
      | Page::AppSelector(_)
      | Page::Fonts
      | Page::FontSelector(_)
      | Page::SettingSelector(_) => extra.push(("r", label("control_center.reset_defaults"))),
      Page::Keybindings => {
        extra.push(("Space", label("control_center.hint.toggle")));
        extra.push(("r", label("control_center.restore_default")));
      }
      Page::WindowRules if matches!(self.selected_item(), Some(Item::WindowRuleWorkspace(_))) => {
        // `←/→` steps the workspace both ways, so the footer names it.
        extra.push(("←/→", label("control_center.window_rules_workspace")));
      }
      Page::KeyboardLayout if matches!(self.selected_item(), Some(Item::Layout(_))) => {
        // Enter sets the default layout and Space enables or disables it,
        // so the generic Toggle hint of the row does not apply here.
        row = Some(RowKind::Info);
        extra.push(("Enter", label("control_center.set_default")));
        extra.push(("Space", label("control_center.hint.toggle")));
      }
      _ => {}
    }
    // `←/→` reach the action bar only while one of its buttons is enabled.
    if self.has_action_bar() && !self.enabled_action_indexes().is_empty() {
      extra.push(("←/→", label("control_center.actions")));
    }
    hints(
      self.lang,
      &HintContext {
        row,
        can_go_back: page != Page::Main,
        search: is_searchable(page),
        refresh: matches!(
          page,
          Page::Hostname
            | Page::Notifications
            | Page::UserList
            | Page::SystemUsers
            | Page::GroupList
            | Page::SystemGroups
        ),
        quit: page == Page::Main,
        extra: &extra,
      },
    )
  }

  fn apply_app(&mut self, category: Category, index: usize) {
    let Some((display, binary)) = self
      .apps
      .installed(category)
      .get(index)
      .map(|app| (app.display.clone(), app.binary.clone()))
    else {
      return;
    };
    match self.apps.set_default(category, &binary) {
      Ok(()) => self.success(format!(
        "{}: {display}",
        tr(self.lang, "control_center.default_application_changed")
      )),
      Err(error) => self.fail(error),
    }
  }

  fn apply_font(&mut self, target: crate::config::fonts::FontTarget, index: usize) {
    let Some(font) = self.system_fonts.get(index).cloned() else {
      return;
    };
    match self.fonts.apply_font(target, &font, self.pending_size) {
      Ok(()) => self.success(format!(
        "{}: {} {}",
        tr(self.lang, "control_center.font_applied"),
        font.display_name(),
        self.pending_size
      )),
      Err(error) => self.fail(error),
    }
  }

  fn apply_font_setting(&mut self, setting: SettingKind, index: usize) {
    let Some(value) = setting_values(setting).get(index) else {
      return;
    };
    match self.fonts.apply_setting(setting, value) {
      Ok(()) => self.success(tr(self.lang, "control_center.setting_applied").to_string()),
      Err(error) => self.fail(error),
    }
  }

  /// Enter on a zone: makes it the draft. `Apply` writes it.
  fn select_timezone(&mut self, index: usize) {
    if let Some(zone) = self.timezones.get(index) {
      self.timezone_draft = zone.clone();
    }
  }

  /// `Apply` of Time Zone: writes the draft with `timedatectl`, then reloads
  /// the system zone so the draft matches what was written.
  fn apply_timezone(&mut self) {
    let zone = self.timezone_draft.clone();
    match time::set_timezone(&zone, &self.timezones) {
      Ok(()) => {
        self.refresh_time();
        self.load_timezone_draft();
        self.success(format!(
          "{}: {zone}",
          tr(self.lang, "control_center.time_zone_changed")
        ));
      }
      Err(error) => self.fail(error),
    }
  }

  fn apply_regional_locale(&mut self, index: usize) {
    let Some(value) = self.generated_locales.get(index).cloned() else {
      return;
    };
    match locale::set_lang(&value, &self.generated_locales) {
      Ok(()) => self.success(format!(
        "{}: {value}",
        tr(self.lang, "control_center.regional_locale_changed")
      )),
      Err(error) => self.fail(error),
    }
  }

  /// Marks or unmarks one `/etc/locale.gen` entry; nothing is written until
  /// `Apply`.
  fn toggle_system_locale(&mut self, index: usize) {
    if let Some(entry) = self.locale_gen_entries.get(index) {
      let key = format!("{} {}", entry.locale, entry.encoding);
      if !self.selected_locales.remove(&key) {
        self.selected_locales.insert(key);
      }
    }
  }

  /// Enter on a layout: makes it the default in the draft. `Apply` writes it.
  fn apply_keyboard_layout(&mut self, index: usize) {
    let Some(code) = self
      .keyboard_layouts
      .get(index)
      .map(|layout| layout.code.clone())
    else {
      return;
    };
    self.keyboard_draft.set_default(&code);
  }

  /// Space on a layout: enables or disables it in the draft. `Apply` writes it.
  fn toggle_keyboard_layout(&mut self, index: usize) {
    let Some(code) = self
      .keyboard_layouts
      .get(index)
      .map(|layout| layout.code.clone())
    else {
      return;
    };
    if let Err(message) = self.keyboard_draft.toggle(&code) {
      self.fail(message);
    }
  }

  /// `Apply` of Keyboard > Layout: writes the draft in one backend call, then
  /// reloads the state so the draft matches what was written.
  fn apply_keyboard_layout_draft(&mut self) {
    let draft = self.keyboard_draft.clone();
    match keyboard::set_x11_layouts(&draft.default, &draft.selected, &self.keyboard_layouts) {
      Ok(()) => {
        self.refresh_keyboard();
        self.keyboard_draft = keyboard_draft::LayoutDraft::loaded(&self.keyboard_info);
        self.success(tr(self.lang, "control_center.keyboard_layouts_updated").to_string());
      }
      Err(error) => self.fail(error),
    }
  }

  fn apply_keyboard_variant(&mut self, index: usize) {
    let Some(variant) = self.keyboard_variants.get(index).cloned() else {
      return;
    };
    let layout = self.keyboard_info.x11_layout.clone();
    match keyboard::set_x11_variant(&layout, &variant.code, &self.keyboard_variants) {
      Ok(()) => {
        self.refresh_keyboard();
        self.success(format!(
          "{}: {}",
          tr(self.lang, "control_center.variant_changed"),
          variant.description
        ));
      }
      Err(error) => self.fail(error),
    }
  }

  fn apply_console_keymap(&mut self, index: usize) {
    let Some(keymap) = self.console_keymaps.get(index).cloned() else {
      return;
    };
    match keyboard::set_console_keymap(&keymap, &self.console_keymaps) {
      Ok(()) => {
        self.refresh_keyboard();
        self.success(format!(
          "{}: {keymap}",
          tr(self.lang, "control_center.console_keymap_changed")
        ));
      }
      Err(error) => self.fail(error),
    }
  }

  fn apply_language(&mut self, index: usize) {
    let Some((locale, _, _)) = LANGUAGES.get(index) else {
      return;
    };
    self.lang = Lang::for_locale(locale);
    let value = serde_json::to_string(&self.lang.locale()).unwrap_or_else(|_| "\"en-US\"".into());
    let persisted = argvus_control_center_core::process::command("argvus")
      .arg("config")
      .args(["set", "/session/language", &value])
      .status()
      .is_ok_and(|status| status.success());
    let projected =
      persisted && argvus_control_center_core::config::reload_argvus_config_service().is_ok();
    if projected {
      self.success(tr(self.lang, "control_center.interface_language_applied").to_string());
    } else {
      self.fail("argvus config could not persist the interface language");
    }
  }

  /// Enter/Space on a Mouse & Touchpad row: toggles flip, values step up.
  fn apply_input(&mut self, item: Item) {
    match item {
      Item::Ratbag(_) => self.input_cycle(item, 1),
      Item::Input(index @ (3 | 5 | 7..=11)) => {
        self.input_toggle_pending = Some(index);
        self.status = Some(Status::new(
          tr(self.lang, "control_center.input_applying").to_string(),
          StatusKind::Success,
        ));
      }
      Item::Input(_) => self.input_cycle(item, 1),
      _ => {}
    }
  }

  /// `←/→` (and `h/l`) on a Mouse & Touchpad row.
  pub(crate) fn input_cycle(&mut self, item: Item, direction: i8) {
    let selected = match item {
      Item::Ratbag(RatbagRow::Device) => return self.cycle_ratbag_device(direction),
      Item::Ratbag(RatbagRow::Profile) => {
        return self.queue_ratbag(crate::system::ratbag::Change::Profile(direction));
      }
      Item::Ratbag(RatbagRow::Dpi) => {
        return self.queue_ratbag(crate::system::ratbag::Change::Dpi(direction));
      }
      Item::Ratbag(RatbagRow::ReportRate) => {
        return self.queue_ratbag(crate::system::ratbag::Change::ReportRate(direction));
      }
      Item::Input(index) => index,
      _ => return,
    };
    let direction = direction.signum();
    let pending = self.input_pending.take();
    self.input_pending = Some(match pending {
      Some((pending_selected, pending_direction, _)) if pending_selected == selected => (
        selected,
        pending_direction.saturating_add(direction),
        std::time::Instant::now() + std::time::Duration::from_millis(100),
      ),
      _ => (
        selected,
        direction,
        std::time::Instant::now() + std::time::Duration::from_millis(100),
      ),
    });
    self.status = Some(Status::new(
      tr(self.lang, "control_center.input_applying").to_string(),
      StatusKind::Success,
    ));
  }

  /// Indexes of the shortcuts shown on the Keyboard Shortcuts page, in
  /// display order and narrowed by the search.
  pub(super) fn visible_keybindings(&self) -> Vec<usize> {
    let mut bindings: Vec<usize> = self
      .keybindings
      .iter()
      .enumerate()
      .filter(|(_, binding)| keybinding_is_cheatsheet_entry(binding))
      .filter(|(_, binding)| {
        search_matches(
          &self.search,
          &[
            &binding.id,
            &binding.category,
            &binding.keys,
            &keybindings::display_keys(&binding.keys),
            &binding.description_key,
            &keybinding_label(self.lang, binding),
            &keybinding_section(self.lang, binding),
          ],
        )
      })
      .map(|(index, _)| index)
      .collect();
    bindings.sort_by_key(|index| {
      let binding = &self.keybindings[*index];
      (
        keybinding_section_order(binding),
        keybinding_category_order(&binding.category),
        binding.id.clone(),
      )
    });
    bindings
  }

  /// The shortcut under the cursor on the Keyboard Shortcuts page.
  pub(super) fn selected_keybinding(&self) -> Option<&keybindings::Binding> {
    match self.selected_item() {
      Some(Item::Keybinding(index)) => self.keybindings.get(index),
      _ => None,
    }
  }

  fn open_keybinding_edit(&mut self, index: usize) {
    let Some(id) = self
      .keybindings
      .get(index)
      .map(|binding| binding.id.clone())
    else {
      return;
    };
    self.keybinding_edit_id = Some(id);
    self.navigation.push(Page::KeybindingEdit);
    self.begin_keybinding_capture();
    self.keybinding_capturing = false;
    self.status = None;
  }

  /// "Disable" of the edit page: turns the edited shortcut off.
  fn disable_edited_keybinding(&mut self) {
    let Some(binding) = self.edited_binding() else {
      return;
    };
    let (id, keys) = (binding.id.clone(), binding.keys.clone());
    if let Err(error) = keybindings::save_override(&id, Some(keys), Some(false), &self.keybindings)
    {
      self.fail(error);
      return;
    }
    self.keybindings = keybindings::load();
    self.success(tr(self.lang, "control_center.keybindings_applied").to_string());
  }

  /// Space on the Keyboard Shortcuts list: enables or disables the
  /// shortcut under the cursor.
  pub(crate) fn toggle_selected_keybinding(&mut self) {
    let Some(binding) = self.selected_keybinding() else {
      return;
    };
    let (id, keys, enabled) = (binding.id.clone(), binding.keys.clone(), binding.enabled);
    if let Err(error) =
      keybindings::save_override(&id, Some(keys), Some(!enabled), &self.keybindings)
    {
      self.fail(error);
      return;
    }
    self.keybindings = keybindings::load();
    self.success(tr(self.lang, "control_center.keybindings_applied").to_string());
  }

  /// "Restore default" of the edit page: immediate, as before.
  fn restore_edited_keybinding(&mut self) {
    if let Some(id) = self.keybinding_edit_id.clone() {
      self.restore_keybinding(&id);
    }
  }

  /// Restores one shortcut to its ARGVUS default.
  pub(super) fn restore_keybinding(&mut self, id: &str) {
    if let Err(error) = keybindings::restore(id, &self.keybindings) {
      self.fail(error);
      return;
    }
    self.keybindings = keybindings::load();
    self.success(tr(self.lang, "control_center.keybindings_restored").to_string());
  }

  /// Reloads the saved rules and keeps the drafts whose rule still exists.
  pub(crate) fn reload_window_rules(&mut self) {
    self.window_rules = window_rules::load();
    let rules = &self.window_rules;
    self
      .window_rule_drafts
      .retain(|name, _| rules.iter().any(|rule| &rule.name == name));
  }

  /// Whether a rule differs from its saved state. Only drafts can differ.
  pub(crate) fn window_rule_changed(&self, rule: &window_rules::WindowRule) -> bool {
    self
      .window_rule_drafts
      .get(&rule.name)
      .is_some_and(|draft| draft != rule)
  }

  /// Changes the draft of a saved rule; nothing reaches the config until Apply.
  fn edit_window_rule(&mut self, index: usize, edit: impl FnOnce(&mut window_rules::WindowRule)) {
    let Some(loaded) = self.window_rules.get(index).cloned() else {
      return;
    };
    let draft = self
      .window_rule_drafts
      .entry(loaded.name.clone())
      .or_insert_with(|| loaded.clone());
    edit(draft);
  }

  fn adjust_window_rule(&mut self, index: usize, delta: i32) {
    self.edit_window_rule(index, |rule| {
      rule.workspace = window_rules::cycle_workspace(rule.workspace, delta);
    });
  }

  /// Opens the name editor of a saved rule, showing its draft name.
  fn open_window_rule_name(&mut self, index: usize) {
    let Some(loaded) = self.window_rules.get(index) else {
      return;
    };
    let draft = self.window_rule_drafts.get(&loaded.name).unwrap_or(loaded);
    self.admin.editor = Some(Editor::new(
      tr(self.lang, "control_center.window_rules_name").into(),
      draft.name.clone(),
      EditTarget::WindowRuleName(loaded.name.clone()),
      false,
    ));
  }

  fn open_window_rule_classes(&mut self, index: usize) {
    let Some(loaded) = self.window_rules.get(index) else {
      return;
    };
    let draft = self.window_rule_drafts.get(&loaded.name).unwrap_or(loaded);
    self.admin.editor = Some(Editor::new(
      tr(self.lang, "control_center.window_rules_classes").into(),
      draft.classes.join(", "),
      EditTarget::WindowRuleClasses(loaded.name.clone()),
      false,
    ));
  }

  fn open_new_window_rule_name(&mut self) {
    self.admin.editor = Some(Editor::new(
      tr(self.lang, "control_center.window_rules_name").into(),
      self.new_window_rule_name.clone(),
      EditTarget::NewWindowRuleName,
      false,
    ));
  }

  /// Saves the draft of one rule. A new name renames the rule.
  fn apply_window_rule(&mut self, index: usize) {
    let Some(loaded) = self.window_rules.get(index).cloned() else {
      return;
    };
    let Some(draft) = self.window_rule_drafts.get(&loaded.name).cloned() else {
      return;
    };
    if let Err(message) = self.check_window_rule_name(&draft.name, Some(loaded.name.as_str())) {
      self.fail(message);
      return;
    }
    match window_rules::save(Some(loaded.name.as_str()), &draft) {
      Ok(()) => {
        self.window_rule_drafts.remove(&loaded.name);
        self.reload_window_rules();
        self.success(tr(self.lang, "control_center.window_rules_applied").to_string());
      }
      Err(error) => self.fail(error),
    }
  }

  /// Adds a rule named by the typed name, or by the next free `rule-N` when empty.
  fn add_window_rule(&mut self) {
    let typed = self.new_window_rule_name.trim().to_owned();
    let name = if typed.is_empty() {
      window_rules::next_rule_name(&self.window_rules)
    } else {
      typed
    };
    if let Err(message) = self.check_window_rule_name(&name, None) {
      self.fail(message);
      return;
    }
    let rule = window_rules::WindowRule {
      name,
      workspace: 1,
      classes: Vec::new(),
    };
    match window_rules::save(None, &rule) {
      Ok(()) => {
        self.new_window_rule_name.clear();
        self.reload_window_rules();
        self.success(tr(self.lang, "control_center.window_rules_saved").to_string());
      }
      Err(error) => self.fail(error),
    }
  }

  /// Validates a rule name. `current` is the saved name of the rule being
  /// changed, which may keep its own name.
  pub(crate) fn check_window_rule_name(
    &self,
    name: &str,
    current: Option<&str>,
  ) -> Result<(), String> {
    if !window_rules::is_valid_name(name) {
      return Err(tr(self.lang, "control_center.window_rules_invalid_name").into());
    }
    let taken = self
      .window_rules
      .iter()
      .any(|rule| rule.name == name && Some(rule.name.as_str()) != current);
    if taken {
      return Err(tr(self.lang, "control_center.window_rules_name_in_use").into());
    }
    Ok(())
  }

  /// Opens the single-line editor for a new project path, or a new root.
  fn open_project_editor(&mut self, root: bool) {
    let title = if root {
      tr(self.lang, "control_center.projects_add_root")
    } else {
      tr(self.lang, "control_center.projects_add_path")
    };
    self.admin.editor = Some(Editor::new(
      title.into(),
      String::new(),
      EditTarget::ProjectPath(root),
      false,
    ));
  }

  fn open_new_snippet_name(&mut self) {
    self.admin.editor = Some(Editor::new(
      tr(self.lang, "control_center.snippets_name").into(),
      self.new_snippet_name.clone(),
      EditTarget::NewSnippetName,
      false,
    ));
  }

  fn open_new_snippet_content(&mut self) {
    self.admin.editor = Some(Editor::new(
      tr(self.lang, "control_center.snippets_content").into(),
      self.new_snippet_content.clone(),
      EditTarget::NewSnippetContent,
      false,
    ));
  }

  /// Opens the editor with a saved snippet's content; saving replaces it.
  fn open_snippet_content(&mut self, index: usize) {
    let Some(entry) = self.snippets.entries.get(index) else {
      return;
    };
    self.admin.editor = Some(Editor::new(
      tr(self.lang, "control_center.snippets_content").into(),
      entry.content.clone(),
      EditTarget::SnippetContent(entry.name.clone()),
      false,
    ));
  }

  fn add_snippet(&mut self) {
    let name = self.new_snippet_name.trim().to_owned();
    if name.is_empty() {
      self.fail(tr(self.lang, "control_center.snippets_name_required").to_string());
      return;
    }
    match snippets::add(&name, &self.new_snippet_content) {
      Ok(()) => {
        self.new_snippet_name.clear();
        self.new_snippet_content.clear();
        self.snippets = snippets::load();
        self.success(tr(self.lang, "control_center.snippets_saved").to_string());
      }
      Err(error) => self.fail(error),
    }
  }

  fn open_snippet_picker(&mut self) {
    match snippets::open_picker() {
      Ok(()) => self.success(tr(self.lang, "control_center.snippets_picker_opening").to_string()),
      Err(error) => self.fail(error),
    }
  }

  fn type_snippet(&mut self, index: usize) {
    match snippets::type_position(index + 1) {
      Ok(()) => self.success(tr(self.lang, "control_center.snippets_typing").to_string()),
      Err(error) => self.fail(error),
    }
  }

  fn open_new_ssh_key_name(&mut self) {
    self.admin.editor = Some(Editor::new(
      tr(self.lang, "control_center.ssh_keys_name").into(),
      self.new_ssh_key_name.clone(),
      EditTarget::NewSshKeyName,
      false,
    ));
  }

  /// Changes only the form's algorithm; nothing is written until Generate runs.
  fn toggle_ssh_key_type(&mut self) {
    self.new_ssh_key_type = self.new_ssh_key_type.toggled();
  }

  fn generate_ssh_key(&mut self) {
    let name = self.new_ssh_key_name.trim().to_owned();
    if name.is_empty() {
      self.fail(tr(self.lang, "control_center.ssh_keys_name_required").to_string());
      return;
    }
    match ssh_keys::generate(&name, self.new_ssh_key_type) {
      Ok(()) => {
        self.new_ssh_key_name.clear();
        self.ssh_keys = ssh_keys::load();
        self.success(tr(self.lang, "control_center.ssh_keys_generated").to_string());
      }
      Err(error) => self.fail(error),
    }
  }

  /// Opens a terminal for ssh-add, which asks for the passphrase there.
  /// The key is not in the agent yet when this returns.
  fn copy_ssh_public_key(&mut self, index: usize) {
    let Some(name) = self
      .ssh_keys
      .entries
      .get(index)
      .map(|entry| entry.name.clone())
    else {
      return;
    };
    match ssh_keys::copy_public_key(&name) {
      Ok(()) => {
        self.success(tr(self.lang, "control_center.ssh_keys_public_key_copied").to_string())
      }
      Err(ssh_keys::CopyError::Unreadable) => {
        self.fail(tr(self.lang, "control_center.ssh_keys_public_key_unreadable").to_string());
      }
      Err(ssh_keys::CopyError::Clipboard) => {
        self.fail(tr(self.lang, "control_center.ssh_keys_clipboard_unavailable").to_string());
      }
    }
  }

  fn load_ssh_key(&mut self, index: usize) {
    let Some(name) = self
      .ssh_keys
      .entries
      .get(index)
      .map(|entry| entry.name.clone())
    else {
      return;
    };
    match ssh_keys::load_key(&name) {
      Ok(()) => self.success(tr(self.lang, "control_center.ssh_keys_loading").to_string()),
      Err(error) => self.fail(error),
    }
  }

  fn unload_ssh_key(&mut self, index: usize) {
    let Some(name) = self
      .ssh_keys
      .entries
      .get(index)
      .map(|entry| entry.name.clone())
    else {
      return;
    };
    match ssh_keys::unload(&name) {
      Ok(()) => {
        self.ssh_keys = ssh_keys::load();
        self.success(tr(self.lang, "control_center.ssh_keys_unloaded").to_string());
      }
      Err(error) => self.fail(error),
    }
  }

  /// The date layout follows the language, the same rule the form uses to read
  /// and write dates.
  pub(super) fn gpg_date_order(&self) -> DateOrder {
    DateOrder::for_locale(&self.lang.locale())
  }

  /// Opens the single-line editor for one text field of the generate form.
  fn open_gpg_field(&mut self, title_key: &str, value: String, target: EditTarget) {
    self.admin.editor = Some(Editor::new(
      tr(self.lang, title_key).into(),
      value,
      target,
      false,
    ));
  }

  fn open_gpg_import(&mut self) {
    self.admin.editor = Some(Editor::new(
      tr(self.lang, "control_center.gpg_keys_import_title").into(),
      String::new(),
      EditTarget::GpgImportPath,
      false,
    ));
  }

  /// Proposes `~/<fingerprint>.asc`, which the user can change before the export
  /// runs. The launcher never overwrites an existing file.
  fn open_gpg_export(&mut self, index: usize) {
    let Some(entry) = self.gpg_keys.entries.get(index) else {
      return;
    };
    let home = std::env::var("HOME").unwrap_or_default();
    self.admin.editor = Some(Editor::new(
      tr(self.lang, "control_center.gpg_keys_export_title").into(),
      format!("{home}/{}.asc", entry.fingerprint),
      EditTarget::GpgExportPath(entry.fingerprint.clone()),
      false,
    ));
  }

  /// The key on the edit page, looked up by fingerprint every time, so a
  /// refresh that removes the key leaves the page with nothing to act on.
  pub(super) fn gpg_edit_entry(&self) -> Option<&gpg_keys::GpgKeyEntry> {
    let fingerprint = self.gpg_edit_fingerprint.as_deref()?;
    self
      .gpg_keys
      .entries
      .iter()
      .find(|entry| entry.fingerprint == fingerprint)
  }

  /// Opens the edit page of one key. The page follows the fingerprint, not the index.
  fn open_gpg_key_edit(&mut self, index: usize) {
    let Some(fingerprint) = self
      .gpg_keys
      .entries
      .get(index)
      .map(|entry| entry.fingerprint.clone())
    else {
      return;
    };
    self.gpg_edit_fingerprint = Some(fingerprint);
    self.navigation.push(Page::GpgKeyEdit);
  }

  /// Starts the expiry page with the key's current expiry. A key that never
  /// expires starts with the switch on and a date two years away.
  fn open_gpg_edit_expiration(&mut self) {
    let order = self.gpg_date_order();
    let expires = self
      .gpg_edit_entry()
      .map(|entry| entry.expires.clone())
      .unwrap_or_default();
    if expires.is_empty() || expires == "never" {
      self.gpg_edit_never = true;
      self.gpg_edit_expiry = CalendarDate::today().add_years(2).display(order);
    } else {
      self.gpg_edit_never = false;
      // The launcher prints expiry as ISO, so it is read year first.
      self.gpg_edit_expiry = CalendarDate::parse(&expires, DateOrder::YearFirst)
        .map(|date| date.display(order))
        .unwrap_or_default();
    }
    self.navigation.push(Page::GpgEditExpiration);
  }

  /// Checks the typed expiry, then changes the key in the background.
  fn apply_gpg_expiry(&mut self) {
    let Some(fingerprint) = self.gpg_edit_fingerprint.clone() else {
      return;
    };
    let order = self.gpg_date_order();
    match gpg_keys::expiry_argument(self.gpg_edit_never, &self.gpg_edit_expiry, order) {
      Ok(expire) => self.start_gpg_action("control_center.gpg_keys_expiry_changed", move || {
        gpg_keys::set_expiry(&fingerprint, &expire)
      }),
      Err(error) => {
        let message = self.gpg_form_error_text(error);
        self.fail(message);
      }
    }
  }

  /// Runs one key change on a worker thread. Only one runs at a time, because
  /// each may open the pinentry window. The page reloads the keyring when it ends.
  pub(super) fn start_gpg_action<F>(&mut self, success_key: &'static str, task: F)
  where
    F: FnOnce() -> Result<(), String> + Send + 'static,
  {
    if self.gpg_action.is_some() {
      self.fail(tr(self.lang, "control_center.gpg_keys_action_busy").to_string());
      return;
    }
    self.gpg_action = Some((self.jobs.spawn(move |_| Ok(task())), success_key));
    self.success(tr(self.lang, "control_center.gpg_keys_working").to_string());
  }

  fn confirm_gpg_delete(&mut self, index: usize) {
    if let Some(entry) = self.gpg_keys.entries.get(index) {
      let fingerprint = entry.fingerprint.clone();
      self.open_confirm(PendingAction::DeleteGpgKey(fingerprint));
    }
  }

  /// Only a key with a private part has something to export.
  fn confirm_gpg_export_private(&mut self, index: usize) {
    let Some(entry) = self.gpg_keys.entries.get(index) else {
      return;
    };
    if !entry.secret {
      self.fail(tr(self.lang, "control_center.gpg_keys_no_private").to_string());
      return;
    }
    let fingerprint = entry.fingerprint.clone();
    self.open_confirm(PendingAction::ExportGpgPrivate(fingerprint));
  }

  fn confirm_gpg_import_private(&mut self) {
    self.open_confirm(PendingAction::ImportGpgPrivate);
  }

  /// Certifies the key on the edit page with the key at `index`.
  fn confirm_gpg_sign(&mut self, index: usize) {
    let Some(fingerprint) = self.gpg_edit_fingerprint.clone() else {
      return;
    };
    let Some(signer) = self
      .gpg_keys
      .entries
      .get(index)
      .map(|entry| entry.fingerprint.clone())
    else {
      return;
    };
    self.open_confirm(PendingAction::SignGpgKey(fingerprint, signer));
  }

  /// Opens the path editor for a private export. The proposed file is
  /// `<fingerprint>-private.asc`, so it never collides with the public export;
  /// it can be changed, and the launcher never overwrites a file.
  pub(super) fn open_gpg_export_private(&mut self, fingerprint: String) {
    let home = std::env::var("HOME").unwrap_or_default();
    self.admin.editor = Some(Editor::new(
      tr(self.lang, "control_center.gpg_keys_export_private_title").into(),
      format!("{home}/{fingerprint}-private.asc"),
      EditTarget::GpgExportPrivatePath(fingerprint),
      false,
    ));
  }

  /// Opens the file manager to choose a key file, on a worker thread because it
  /// stays open until the user chooses or closes it.
  pub(super) fn start_gpg_import_pick(&mut self, kind: gpg_keys::GpgImportKind) {
    if self.gpg_pick.is_some() || self.gpg_action.is_some() {
      self.fail(tr(self.lang, "control_center.gpg_keys_action_busy").to_string());
      return;
    }
    self.gpg_pick = Some((
      self.jobs.spawn(move |_| {
        Ok(argvus_control_center_core::picker::pick_file(
          "argvus-gpg-picker",
          "gpg-selection",
        ))
      }),
      kind,
    ));
    self.success(tr(self.lang, "control_center.gpg_keys_working").to_string());
  }

  /// Imports the file at `path` in the background, the same way as the other key changes.
  pub(super) fn import_gpg_file(&mut self, kind: gpg_keys::GpgImportKind, path: &str) {
    let path = path.to_owned();
    match kind {
      gpg_keys::GpgImportKind::Public => self
        .start_gpg_action("control_center.gpg_keys_imported", move || {
          gpg_keys::import(&path)
        }),
      gpg_keys::GpgImportKind::Private => self
        .start_gpg_action("control_center.gpg_keys_private_imported", move || {
          gpg_keys::import_private(&path)
        }),
    }
  }

  /// The typed-path editor, used when the file manager cannot report a choice.
  pub(super) fn open_gpg_import_path(&mut self, kind: gpg_keys::GpgImportKind) {
    match kind {
      gpg_keys::GpgImportKind::Public => self.open_gpg_import(),
      gpg_keys::GpgImportKind::Private => self.open_gpg_import_private(),
    }
  }

  pub(super) fn open_gpg_import_private(&mut self) {
    self.admin.editor = Some(Editor::new(
      tr(self.lang, "control_center.gpg_keys_import_private_title").into(),
      String::new(),
      EditTarget::GpgImportPrivatePath,
      false,
    ));
  }

  /// Copies the armored public key through the launcher's standard output.
  fn copy_gpg_public(&mut self, index: usize) {
    let Some(fingerprint) = self
      .gpg_keys
      .entries
      .get(index)
      .map(|entry| entry.fingerprint.clone())
    else {
      return;
    };
    match gpg_keys::public_key(&fingerprint).and_then(|key| {
      clipboard::copy(&key)
        .map_err(|_| tr(self.lang, "control_center.gpg_keys_copy_failed").to_string())
    }) {
      Ok(()) => self.success(tr(self.lang, "control_center.gpg_keys_public_copied").to_string()),
      Err(error) => self.fail(error),
    }
  }

  fn copy_gpg_fingerprint(&mut self, index: usize) {
    let Some(fingerprint) = self
      .gpg_keys
      .entries
      .get(index)
      .map(|entry| entry.fingerprint.clone())
    else {
      return;
    };
    match clipboard::copy(&fingerprint) {
      Ok(()) => {
        self.success(tr(self.lang, "control_center.gpg_keys_fingerprint_copied").to_string())
      }
      Err(_) => self.fail(tr(self.lang, "control_center.gpg_keys_copy_failed").to_string()),
    }
  }

  /// Checks the form first, so a mistake is reported before gpg runs. Without a
  /// passphrase the page asks first.
  fn generate_gpg_key(&mut self) {
    let order = self.gpg_date_order();
    if let Err(error) = self.gpg_form.validate(order) {
      let message = self.gpg_form_error_text(error);
      self.fail(message);
      return;
    }
    if self.gpg_form.protect_with_passphrase {
      self.start_gpg_generation();
    } else {
      self.open_confirm(PendingAction::GenerateGpgNoPassphrase);
    }
  }

  /// Runs the launcher on a worker thread. The passphrase is cleared from the
  /// page as soon as the job has its copy, so it lives only in that job. When the
  /// job fails the other fields stay and the passphrase must be typed again.
  pub(super) fn start_gpg_generation(&mut self) {
    let order = self.gpg_date_order();
    let form = self.gpg_form.clone();
    self.gpg_form.passphrase.clear();
    self.gpg_form.passphrase_confirm.clear();
    self.gpg_job = Some(
      self
        .jobs
        .spawn(move |_| Ok(gpg_keys::generate(&form, order))),
    );
    self.success(tr(self.lang, "control_center.gpg_keys_generating").to_string());
  }

  /// The message for one problem with the generate form, in the current language.
  fn gpg_form_error_text(&self, error: gpg_keys::FormError) -> String {
    let key = match error {
      gpg_keys::FormError::NameRequired => "control_center.gpg_keys_name_required",
      gpg_keys::FormError::EmailInvalid => "control_center.gpg_keys_email_invalid",
      gpg_keys::FormError::DateInvalid => {
        let layout = self.gpg_date_order().layout();
        return self
          .lang
          .tr_args("control_center.gpg_keys_date_invalid", [("layout", layout)])
          .to_string();
      }
      gpg_keys::FormError::DateNotFuture => "control_center.gpg_keys_date_not_future",
      gpg_keys::FormError::PassphraseEmpty => "control_center.gpg_keys_passphrase_empty",
      gpg_keys::FormError::PassphraseInvalid => "control_center.gpg_keys_passphrase_invalid",
      gpg_keys::FormError::PassphraseMismatch => "control_center.gpg_keys_passphrase_mismatch",
    };
    tr(self.lang, key).to_string()
  }

  fn unload_all_ssh_keys(&mut self) {
    match ssh_keys::unload_all() {
      Ok(()) => {
        self.ssh_keys = ssh_keys::load();
        self.success(tr(self.lang, "control_center.ssh_keys_unloaded_all").to_string());
      }
      Err(error) => self.fail(error),
    }
  }
}
