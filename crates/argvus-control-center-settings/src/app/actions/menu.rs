//! Settings list behavior: selection, activation, the action bar, search, going back and the footer.

use super::*;

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
pub(super) fn is_searchable(page: Page) -> bool {
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
  pub(super) fn selected_kind(&self, rows: &[Row<Item>]) -> Option<RowKind> {
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
  pub(super) fn toggle(&mut self, item: Item) {
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
  pub(super) fn adjust(&mut self, item: Item, delta: i32) {
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
  pub(super) fn confirm_item(&mut self, item: Item) {
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
  pub(super) fn enabled_action_indexes(&self) -> Vec<usize> {
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
  pub(super) fn leaving_drops_draft(&self) -> bool {
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
  pub(in super::super) fn discard_and_leave(&mut self) {
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
  /// of a list, the layout in use...), or on its first row.
  pub fn select_current(&mut self) {
    let rows = self.rows();
    let current = if self.page() == Page::KeyboardLayout {
      let active_layout = self.keyboard_info.x11_layout.clone();
      self
        .keyboard_layouts
        .iter()
        .position(|layout| layout.code == active_layout)
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
}
