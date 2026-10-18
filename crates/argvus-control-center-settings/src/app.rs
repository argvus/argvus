//! Implements application state and main-flow coordination in crate `argvus control center settings`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
mod actions;
mod keyboard_draft;
mod rows;

pub(crate) use actions::setting_values;

use std::collections::{BTreeMap, BTreeSet};

use argvus_tui::confirm::{ConfirmOutcome, ConfirmState};
use std::time::{Duration, Instant};

use argvus_control_center_apps::catalog::Category;

use crate::config::apps::AppsBackend;
use crate::config::fonts::{FontSettings, FontTarget, SettingKind};
use crate::i18n::{Lang, tr};
use crate::navigation::{Navigation, Page};
use crate::system::fonts::{self, FontEntry};
use crate::system::gpg_keys;
use crate::system::keybindings;
use crate::system::projects;
use crate::system::snippets;
use crate::system::ssh_keys;
use crate::system::window_rules;
use crate::system::{host, input, keyboard, locale, time};
use crate::theme::Theme;
use argvus_control_center_core::navigation::{HomeCardId, root_label};
use argvus_control_center_core::{
  jobs::{JobHandle, JobManager, JobState},
  privileged::{PrivilegedRequest, SystemSettingsOperation},
  process::{LiveProcess, ProcessRequest, ProcessRunner, SystemProcessRunner},
};

/// Defines the constant `MIN_WIDTH`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub const MIN_WIDTH: u16 = 60;
/// Defines the constant `MIN_HEIGHT`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub const MIN_HEIGHT: u16 = 15;

#[derive(Debug, Clone, Copy)]
/// Defines `StatusKind`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub enum StatusKind {
  Success,
  Error,
}

/// Represents `Status`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct Status {
  pub text: String,
  pub kind: StatusKind,
  created: Instant,
}

impl Status {
  pub(crate) fn new(text: String, kind: StatusKind) -> Self {
    Self {
      text,
      kind,
      created: Instant::now(),
    }
  }
}

/// A file choice running in the background, with the import it leads to.
type GpgPick = (
  JobHandle<Result<Option<std::path::PathBuf>, String>>,
  gpg_keys::GpgImportKind,
);

#[derive(Debug, Clone)]
/// Defines `PendingAction`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub enum PendingAction {
  /// An accounts or firewall request; `danger` marks deletions.
  Administration {
    message: String,
    danger: bool,
  },
  ResetApps,
  ResetApp(Category),
  ResetFonts,
  ResetFont(FontTarget),
  ResetFontSetting(SettingKind),
  ResetKeybindings,
  RemoveWindowRule(String),
  RemoveProject(String),
  RemoveSnippet(String),
  /// Deletes both files of a key pair, by the name it was listed with.
  DeleteSshKey(String),
  /// Generates a key without a passphrase, which the launcher only accepts
  /// with `--yes`. Confirmed before the form is sent.
  GenerateGpgNoPassphrase,
  /// Deletes a key by fingerprint.
  DeleteGpgKey(String),
  /// Exports the private key of a fingerprint; the path is typed next.
  ExportGpgPrivate(String),
  /// Imports a private key; the path is typed next.
  ImportGpgPrivate,
  /// Revokes a key by fingerprint with a reason.
  RevokeGpgKey(String, crate::system::gpg_keys::RevokeReason),
  /// Certifies the first fingerprint with the second one.
  SignGpgKey(String, String),
  /// Publishes a key by fingerprint to the keyserver.
  SendGpgKey(String),
  /// The `r` shortcut on one shortcut; `leave` also closes the edit page.
  RestoreKeybinding {
    id: String,
    leave: bool,
  },
  ApplySystemLocales,
  SetNtp(bool),
  /// Esc would leave a page with unsaved changes.
  DiscardDraft,
}

/// Represents `App`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct App {
  pub admin: crate::administration::Administration,
  pub lang: Lang,
  pub theme: Theme,
  pub navigation: Navigation,
  apps: AppsBackend,
  fonts: FontSettings,
  system_fonts: Vec<FontEntry>,
  pub search: String,
  pub searching: bool,
  pub pending_size: u16,
  pub status: Option<Status>,
  pub error_modal: Option<String>,
  pub confirm: Option<PendingAction>,
  /// Focus of the open confirmation (pending action or shortcut conflict).
  pub confirm_focus: ConfirmState,
  pub hostname_editing: bool,
  pub hostname_input: String,
  pub width: u16,
  pub height: u16,
  pub viewport: usize,
  distro: locale::Distro,
  timezones: Vec<String>,
  /// Zone picked on Time Zone, written only by its `Apply` row.
  timezone_draft: String,
  datetime: time::DateTimeInfo,
  generated_locales: Vec<String>,
  locale_gen_entries: Vec<locale::LocaleEntry>,
  selected_locales: BTreeSet<String>,
  keyboard_info: keyboard::KeyboardInfo,
  keyboard_layouts: Vec<keyboard::Layout>,
  keyboard_variants: Vec<keyboard::Variant>,
  /// Pending edits of Keyboard > Layout, written only by its `Apply` row.
  keyboard_draft: keyboard_draft::LayoutDraft,
  console_keymaps: Vec<String>,
  input: input::InputSettings,
  keybindings: Vec<keybindings::Binding>,
  pub(crate) window_rules: Vec<window_rules::WindowRule>,
  /// Unsaved edits to loaded window rules, keyed by the loaded name.
  pub(crate) window_rule_drafts: BTreeMap<String, window_rules::WindowRule>,
  pub(crate) new_window_rule_name: String,
  pub(crate) projects: projects::Projects,
  pub(crate) snippets: snippets::Snippets,
  pub(crate) new_snippet_name: String,
  pub(crate) new_snippet_content: String,
  /// Key pairs in `~/.ssh` and their agent state, as last listed.
  pub(crate) ssh_keys: ssh_keys::SshKeys,
  pub(crate) new_ssh_key_name: String,
  pub(crate) new_ssh_key_type: ssh_keys::KeyType,
  /// The keyring and the pinentry, as last listed.
  pub(crate) gpg_keys: gpg_keys::GpgKeys,
  pub(crate) gpg_form: gpg_keys::GpgForm,
  /// Key generation runs on a worker thread: with a passphrase, the pinentry
  /// window keeps the call open until the user answers it.
  gpg_job: Option<JobHandle<Result<(), String>>>,
  /// The key being edited, by fingerprint, so a refresh cannot point at another key.
  pub(crate) gpg_edit_fingerprint: Option<String>,
  /// The expiry typed on the edit page and its "never expires" switch.
  pub(crate) gpg_edit_expiry: String,
  pub(crate) gpg_edit_never: bool,
  /// A key change running in the background, with the message shown when it succeeds.
  gpg_action: Option<(JobHandle<Result<(), String>>, &'static str)>,
  /// The sections of the GPG keys page that are expanded; the others are collapsed.
  pub(crate) gpg_sections_open: std::collections::BTreeSet<crate::item::GpgSection>,
  /// The file manager choosing a key file to import, and which kind of import.
  gpg_pick: Option<GpgPick>,
  keybinding_capturing: bool,
  keybinding_editor_modifiers: [bool; 4],
  keybinding_editor_key: String,
  keybinding_editor_field: usize,
  keybinding_detected: bool,
  keybinding_super_required: bool,
  keybinding_conflict: Option<(String, String, Vec<String>)>,
  keybinding_edit_id: Option<String>,
  apps_loading: bool,
  apps_job: Option<JobHandle<AppsBackend>>,
  fonts_job: Option<JobHandle<Vec<FontEntry>>>,
  input_loading: bool,
  input_load_job: Option<JobHandle<input::InputSettings>>,
  input_device_job: Option<JobHandle<(input::Devices, Vec<crate::system::ratbag::Device>)>>,
  input_last_device_refresh: Instant,
  input_pending: Option<(usize, i8, Instant)>,
  input_toggle_pending: Option<usize>,
  input_job: Option<JobHandle<input::InputSettings>>,
  ratbag_device: usize,
  ratbag_pending_path: Option<String>,
  ratbag_job: Option<JobHandle<Vec<crate::system::ratbag::Device>>>,
  hostname: String,
  jobs: JobManager,
  dnd_enabled: bool,
  dnd_job: Option<JobHandle<(bool, bool)>>,
  dnd_last_refresh: Instant,
  /// Background listing of the SSH key pairs, run while the SSH keys page is open
  /// so the agent state follows a load done in a terminal.
  ssh_keys_job: Option<JobHandle<ssh_keys::SshKeys>>,
  ssh_keys_last_refresh: Instant,
  task_job: Option<JobHandle<Result<String, String>>>,
  pub task_live: Option<LiveProcess>,
  pub task_open: bool,
  pub task_scroll: u16,
  pub task_follow: bool,
}

impl App {
  /// Constructs `new` with this module's expected initial state. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn new(initial: Page) -> Self {
    let lang = canonical_language().unwrap_or_else(Lang::detect);
    Self::with_context(initial, lang, Theme::load())
  }

  /// Replaces the semantic theme used by this page.
  pub fn set_theme(&mut self, theme: &Theme) {
    self.theme = theme.clone();
  }

  /// Constructs `with_context` with this module's expected initial state. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn with_context(initial: Page, lang: Lang, theme: Theme) -> Self {
    let locale_gen_entries =
      locale::locale_gen_entries(std::path::Path::new("/etc/locale.gen")).unwrap_or_default();
    let selected_locales = locale_gen_entries
      .iter()
      .filter(|entry| entry.enabled)
      .map(|entry| format!("{} {}", entry.locale, entry.encoding))
      .collect();
    let datetime = time::datetime_info();
    let keyboard_info = keyboard::info();
    let hostname = host::current();
    let mut app = Self {
      admin: crate::administration::Administration::new(initial),
      lang,
      theme,
      navigation: Navigation::new(initial),
      apps: AppsBackend::empty(),
      fonts: FontSettings::load(),
      system_fonts: Vec::new(),
      search: String::new(),
      searching: false,
      pending_size: 13,
      status: None,
      error_modal: None,
      confirm: None,
      confirm_focus: ConfirmState::new(),
      hostname_editing: false,
      hostname_input: hostname.clone(),
      width: 80,
      height: 24,
      viewport: 1,
      distro: locale::distro(),
      timezones: time::list_timezones(),
      timezone_draft: datetime.time_zone.clone(),
      datetime,
      generated_locales: locale::generated_locales(),
      locale_gen_entries,
      selected_locales,
      keyboard_variants: keyboard::variants(&keyboard_info.x11_layout),
      keyboard_draft: keyboard_draft::LayoutDraft::loaded(&keyboard_info),
      keyboard_info,
      keyboard_layouts: keyboard::layouts(),
      console_keymaps: keyboard::console_keymaps(),
      input: input::InputSettings::default(),
      keybindings: keybindings::load(),
      window_rules: window_rules::load(),
      window_rule_drafts: BTreeMap::new(),
      new_window_rule_name: String::new(),
      projects: projects::load(),
      snippets: snippets::load(),
      new_snippet_name: String::new(),
      new_snippet_content: String::new(),
      ssh_keys: ssh_keys::load(),
      new_ssh_key_name: String::new(),
      new_ssh_key_type: ssh_keys::KeyType::default(),
      gpg_keys: gpg_keys::load(),
      gpg_form: gpg_keys::GpgForm::new(crate::system::date::DateOrder::for_locale(&lang.locale())),
      gpg_job: None,
      gpg_edit_fingerprint: None,
      gpg_edit_expiry: String::new(),
      gpg_edit_never: false,
      gpg_action: None,
      gpg_pick: None,
      // Both sections start expanded; Enter on a section title collapses it.
      gpg_sections_open: [
        crate::item::GpgSection::Generate,
        crate::item::GpgSection::Import,
      ]
      .into_iter()
      .collect(),
      keybinding_capturing: false,
      keybinding_editor_modifiers: [false; 4],
      keybinding_editor_key: String::new(),
      keybinding_editor_field: 0,
      keybinding_detected: false,
      keybinding_super_required: false,
      keybinding_conflict: None,
      keybinding_edit_id: None,
      apps_loading: true,
      apps_job: None,
      fonts_job: None,
      input_loading: true,
      input_load_job: None,
      input_device_job: None,
      input_last_device_refresh: Instant::now(),
      input_pending: None,
      input_toggle_pending: None,
      input_job: None,
      ratbag_device: 0,
      ratbag_pending_path: None,
      ratbag_job: None,
      hostname,
      jobs: JobManager::default(),
      dnd_enabled: false,
      dnd_job: None,
      dnd_last_refresh: Instant::now() - Duration::from_secs(2),
      ssh_keys_job: None,
      ssh_keys_last_refresh: Instant::now() - Duration::from_secs(3),
      task_job: None,
      task_live: None,
      task_open: false,
      task_scroll: 0,
      task_follow: false,
    };
    app.input_load_job = Some(app.jobs.spawn(|_| Ok(input::load())));
    app.apps_job = Some(app.jobs.spawn(|_| Ok(AppsBackend::load())));
    app.fonts_job = Some(
      app
        .jobs
        .spawn(|_| fonts::list().map_err(|error| error.to_string())),
    );
    if initial != Page::Main {
      app.select_current();
    }
    app
  }

  /// Executes the `page` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn page(&self) -> Page {
    self.navigation.current().page
  }

  /// Executes the `breadcrumb` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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

  /// Executes the `reset_page` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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

  /// Executes the `refresh_system` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn refresh_system(&mut self) {
    self.hostname = host::current();
    self.admin.load(false);
    self.success(tr(self.lang, "control_center.data_updated").to_string());
  }

  /// Lists the key pairs on a worker thread. `argvus-keys list` runs one
  /// ssh-keygen per key, so doing it in the draw loop would stall the screen.
  fn refresh_ssh_keys_in_background(&mut self) {
    if self.ssh_keys_job.is_some() {
      return;
    }
    self.ssh_keys_job = Some(self.jobs.spawn(|_| Ok(ssh_keys::load())));
    self.ssh_keys_last_refresh = Instant::now();
  }

  /// Executes the `refresh_dnd` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn refresh_dnd(&mut self) {
    if self.dnd_job.is_some() {
      return;
    }
    self.dnd_job = Some(self.jobs.spawn(|_| {
      Self::read_dnd_command(
        &ProcessRequest::new("argvus")
          .arg("notifications")
          .arg("dnd")
          .arg("status"),
      )
      .map(|enabled| (enabled, false))
    }));
    self.dnd_last_refresh = Instant::now();
  }

  /// Applies the `toggle_dnd` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn toggle_dnd(&mut self) {
    if self.dnd_job.is_some() {
      return;
    }
    self.dnd_job = Some(self.jobs.spawn(|_| {
      Self::read_dnd_command(
        &ProcessRequest::new("argvus")
          .arg("notifications")
          .arg("dnd")
          .arg("toggle"),
      )
      .map(|state| (state, true))
    }));
  }

  /// Retrieves data for `read_dnd_command` without mixing collection with TUI rendering. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn read_dnd_command(request: &ProcessRequest) -> Result<bool, String> {
    let output = SystemProcessRunner
      .run(&request.clone().timeout(Duration::from_secs(2)))
      .map_err(|error| error.to_string())?;
    if output.timed_out || output.status != Some(0) {
      let error = String::from_utf8_lossy(&output.stderr).trim().to_owned();
      return Err(if error.is_empty() {
        "notification backend failed".into()
      } else {
        error
      });
    }
    match String::from_utf8_lossy(&output.stdout).trim() {
      "dnd=true" => Ok(true),
      "dnd=false" => Ok(false),
      value => Err(format!("invalid notification state: {value}")),
    }
  }

  /// Executes the `input_success` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub(crate) fn input_success(&mut self) {
    self.success(tr(self.lang, "control_center.input_applied").to_string());
  }

  /// Executes the `input_failure` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub(crate) fn input_failure(&mut self, error: String) {
    eprintln!("argvus-control-center: input setting failed: {error}");
    self.fail(tr(self.lang, "control_center.input_apply_failed"));
  }

  /// Executes the `cycle_ratbag_device` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn cycle_ratbag_device(&mut self, direction: i8) {
    let Some(next) = crate::system::ratbag::next_device_index(
      self.ratbag_device,
      self.input.ratbag.len(),
      direction,
    ) else {
      return;
    };
    self.ratbag_device = next;
  }

  /// Executes the `queue_ratbag` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn queue_ratbag(&mut self, change: crate::system::ratbag::Change) {
    if self.ratbag_job.is_some() {
      return;
    }
    let Some(device) = self
      .input
      .ratbag
      .get(self.ratbag_device)
      .or_else(|| self.input.ratbag.first())
      .cloned()
    else {
      return;
    };
    self.status = Some(Status {
      text: tr(self.lang, "control_center.input_hardware_applying").to_string(),
      kind: StatusKind::Success,
      created: Instant::now(),
    });
    let device_path = device.path.clone();
    self.ratbag_job = Some(
      self
        .jobs
        .spawn(move |_| crate::system::ratbag::apply(&device, change)),
    );
    self.ratbag_pending_path = Some(device_path);
  }

  /// A key while the confirmation is open: Enter runs the focused row
  /// (Cancel first), `y` confirms, `n`/Esc cancel, arrows and Tab move.
  pub fn confirm_key(&mut self, key: crossterm::event::KeyCode) {
    match self.confirm_focus.handle(key) {
      ConfirmOutcome::Confirmed => self.confirm_accept(),
      ConfirmOutcome::Cancelled => self.cancel_modal(),
      ConfirmOutcome::Pending => {}
    }
  }

  /// Runs the confirmed pending action.
  pub fn confirm_accept(&mut self) {
    self.confirm_focus = ConfirmState::new();
    let Some(action) = self.confirm.take() else {
      return;
    };
    match action {
      PendingAction::ResetApps => match self.apps.reset_all() {
        Ok(()) => {
          self.success(tr(self.lang, "control_center.default_applications_restored").to_string())
        }
        Err(error) => self.fail(error),
      },
      PendingAction::ResetApp(category) => match self.apps.reset_default(category) {
        Ok(()) => self.success(format!(
          "{} → {}",
          category_label(self.lang, category),
          tr(self.lang, "control_center.argvus_default")
        )),
        Err(error) => self.fail(error),
      },
      PendingAction::ResetFonts => match self.fonts.reset_all() {
        Ok(()) => self.success(tr(self.lang, "control_center.font_settings_restored").to_string()),
        Err(error) => self.fail(error),
      },
      PendingAction::ResetFont(target) => match self.fonts.reset_font(target) {
        Ok(()) => self.success(format!(
          "{} → {}",
          font_target_label(self.lang, target),
          tr(self.lang, "control_center.argvus_default")
        )),
        Err(error) => self.fail(error),
      },
      PendingAction::ResetFontSetting(setting) => match self.fonts.reset_setting(setting) {
        Ok(()) => self.success(format!(
          "{} → {}",
          setting_label(self.lang, setting),
          tr(self.lang, "control_center.argvus_default")
        )),
        Err(error) => self.fail(error),
      },
      PendingAction::RemoveWindowRule(name) => match window_rules::remove(&name) {
        Ok(()) => {
          self.reload_window_rules();
          self.success(tr(self.lang, "control_center.window_rules_removed").to_string());
        }
        Err(error) => self.fail(error),
      },
      PendingAction::RemoveProject(path) => match projects::remove(&path) {
        Ok(()) => {
          self.projects = projects::load();
          self.success(tr(self.lang, "control_center.projects_removed").to_string());
        }
        Err(error) => self.fail(error),
      },
      PendingAction::RemoveSnippet(name) => match snippets::remove(&name) {
        Ok(()) => {
          self.snippets = snippets::load();
          self.success(tr(self.lang, "control_center.snippets_removed").to_string());
        }
        Err(error) => self.fail(error),
      },
      PendingAction::DeleteSshKey(name) => match ssh_keys::delete(&name) {
        Ok(()) => {
          self.ssh_keys = ssh_keys::load();
          self.success(tr(self.lang, "control_center.ssh_keys_deleted").to_string());
        }
        Err(error) => self.fail(error),
      },
      PendingAction::GenerateGpgNoPassphrase => self.start_gpg_generation(),
      PendingAction::DeleteGpgKey(fingerprint) => self
        .start_gpg_action("control_center.gpg_keys_deleted", move || {
          gpg_keys::delete(&fingerprint)
        }),
      PendingAction::ExportGpgPrivate(fingerprint) => self.open_gpg_export_private(fingerprint),
      PendingAction::ImportGpgPrivate => {
        self.start_gpg_import_pick(gpg_keys::GpgImportKind::Private)
      }
      PendingAction::RevokeGpgKey(fingerprint, reason) => self
        .start_gpg_action("control_center.gpg_keys_revoked", move || {
          gpg_keys::revoke(&fingerprint, reason)
        }),
      PendingAction::SignGpgKey(fingerprint, signer) => self
        .start_gpg_action("control_center.gpg_keys_signed", move || {
          gpg_keys::sign_key(&fingerprint, &signer)
        }),
      PendingAction::SendGpgKey(fingerprint) => self
        .start_gpg_action("control_center.gpg_keys_sent", move || {
          gpg_keys::keyserver_send(&fingerprint)
        }),
      PendingAction::ResetKeybindings => match keybindings::restore_all(&self.keybindings) {
        Ok(()) => {
          self.keybindings = keybindings::load();
          self.success(tr(self.lang, "control_center.keybindings_restored_all").to_string());
        }
        Err(error) => self.fail(error),
      },
      PendingAction::RestoreKeybinding { id, leave } => {
        self.restore_keybinding(&id);
        if leave {
          self.keybinding_capturing = false;
          self.navigation.back();
        }
      }
      PendingAction::ApplySystemLocales => self.apply_system_locales(),
      PendingAction::DiscardDraft => self.discard_and_leave(),
      PendingAction::Administration { .. } => self.admin.submit(),
      PendingAction::SetNtp(enabled) => match time::set_ntp(enabled) {
        Ok(()) => {
          self.refresh_time();
          self.success(tr(self.lang, "control_center.ntp_setting_applied").to_string());
        }
        Err(error) => self.fail(error),
      },
    }
  }

  /// Executes the `cancel_modal` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn cancel_modal(&mut self) {
    let delete_asked = self.admin.delete_confirm;
    self.admin.cancel_pending();
    self.confirm = None;
    self.confirm_focus = ConfirmState::new();
    // No on the delete confirmation returns to the user list.
    if delete_asked {
      self.leave_user_pages();
    }
  }

  /// Texts of the open confirmation: title, message, confirm label and
  /// whether it uses the danger style.
  pub fn confirm_dialog(&self) -> Option<(String, String, &'static str, bool)> {
    let label = |key: &str| tr(self.lang, key);
    if let Some((_, keys, conflicts)) = self.keybinding_conflict_state() {
      let mut message = format!(
        "{}\n{keys}\n\n{}",
        label("control_center.keybindings_conflict"),
        label("control_center.keybindings_used_by")
      );
      for id in conflicts {
        message.push_str(&format!("\n• {}", self.keybinding_label_for(&id)));
      }
      return Some((
        label("control_center.keybindings_conflict_title").into(),
        message,
        label("control_center.replace"),
        false,
      ));
    }
    let action = self.confirm.as_ref()?;
    let (title, message) = pending_action_text(self.lang, action);
    let (confirm, danger) = match action {
      // The user delete confirmation answers Yes / No.
      PendingAction::Administration { danger: true, .. } if self.admin.delete_confirm => {
        (label("control_center.yes"), true)
      }
      PendingAction::Administration { danger: true, .. } => (label("control_center.delete"), true),
      PendingAction::DiscardDraft => (label("control_center.discard"), false),
      PendingAction::ResetApps
      | PendingAction::ResetApp(_)
      | PendingAction::ResetFonts
      | PendingAction::ResetFont(_)
      | PendingAction::ResetFontSetting(_)
      | PendingAction::ResetKeybindings
      | PendingAction::RemoveWindowRule(_)
      | PendingAction::RemoveProject(_)
      | PendingAction::RemoveSnippet(_)
      | PendingAction::DeleteSshKey(_)
      | PendingAction::DeleteGpgKey(_)
      | PendingAction::ExportGpgPrivate(_)
      | PendingAction::RevokeGpgKey(..)
      | PendingAction::GenerateGpgNoPassphrase => (label("control_center.confirm"), true),
      _ => (label("control_center.confirm"), false),
    };
    Some((title, message, confirm, danger))
  }

  /// Executes the `hostname_input` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn hostname_input(&mut self, key: crossterm::event::KeyCode) {
    match key {
      crossterm::event::KeyCode::Esc => self.hostname_editing = false,
      crossterm::event::KeyCode::Enter => {
        let value = self.hostname_input.trim().to_string();
        match host::set(&value) {
          Ok(()) => {
            self.hostname_editing = false;
            self.hostname = host::current();
            self.success(format!(
              "{}: {value}",
              tr(self.lang, "control_center.hostname_changed")
            ));
          }
          Err(error) => self.fail(error),
        }
      }
      crossterm::event::KeyCode::Backspace => {
        self.hostname_input.pop();
      }
      crossterm::event::KeyCode::Char(character)
        if character.is_ascii_alphanumeric() || character == '-' =>
      {
        self.hostname_input.push(character);
      }
      _ => {}
    }
  }

  /// Executes the `adjust_size` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn adjust_size(&mut self, delta: i16) {
    if matches!(self.page(), Page::FontSelector(_)) && !self.searching {
      self.pending_size = (self.pending_size as i16 + delta).clamp(8, 32) as u16;
    }
  }

  /// Executes the `resize` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn resize(&mut self, width: u16, height: u16) {
    self.width = width;
    self.height = height;
  }

  /// Executes the `too_small` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn too_small(&self) -> bool {
    self.width < MIN_WIDTH || self.height < MIN_HEIGHT
  }

  /// Executes the `expire_status` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn expire_status(&mut self) -> bool {
    if let Some(result) = self.admin.poll() {
      match result {
        Ok(()) => {
          match self.admin.completed.take().as_deref() {
            Some("create") if self.page() == Page::CreateUser => {
              self.navigation.current_mut().page = Page::User;
              self.normalize_selection();
            }
            // A deleted user leaves every user subpage; the account is gone.
            Some("delete") => self.leave_user_pages(),
            Some("delete-group") => {
              self.navigation.back();
            }
            _ => {}
          }
          self.success(tr(self.lang, "control_center.data_updated").into());
        }
        Err(error) => self.fail(error),
      }
      return true;
    }
    if self
      .status
      .as_ref()
      .is_some_and(|status| status.created.elapsed() >= Duration::from_secs(4))
    {
      self.status = None;
      true
    } else {
      false
    }
  }

  /// Executes the `poll` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn poll(&mut self) -> bool {
    let mut changed = false;
    if self.dnd_job.is_none()
      && self.page() == Page::Notifications
      && self.dnd_last_refresh.elapsed() >= Duration::from_secs(2)
    {
      self.refresh_dnd();
    }
    if self.ssh_keys_job.is_none()
      && self.page() == Page::SshKeys
      && self.ssh_keys_last_refresh.elapsed() >= Duration::from_secs(3)
    {
      self.refresh_ssh_keys_in_background();
    }
    if let Some(job) = self.ssh_keys_job.take() {
      match job.try_state() {
        JobState::Running => self.ssh_keys_job = Some(job),
        JobState::Finished(Ok(keys)) => {
          self.ssh_keys = keys;
          changed = true;
        }
        // The listing itself never fails; the next tick simply tries again.
        JobState::Finished(Err(_)) => changed = true,
      }
    }
    if let Some((job, kind)) = self.gpg_pick.take() {
      match job.try_state() {
        JobState::Running => self.gpg_pick = Some((job, kind)),
        // A chosen file is imported at once; a GUI file manager reports no
        // choice, so the user types the path instead.
        JobState::Finished(Ok(Ok(Some(path)))) => {
          self.import_gpg_file(kind, &path.to_string_lossy());
          changed = true;
        }
        JobState::Finished(Ok(Ok(None))) => {
          self.open_gpg_import_path(kind);
          changed = true;
        }
        JobState::Finished(Ok(Err(error))) | JobState::Finished(Err(error)) => {
          self.fail(error);
          changed = true;
        }
      }
    }
    if let Some((job, success_key)) = self.gpg_action.take() {
      match job.try_state() {
        JobState::Running => self.gpg_action = Some((job, success_key)),
        JobState::Finished(Ok(Ok(()))) => {
          self.gpg_keys = gpg_keys::load();
          self.success(tr(self.lang, success_key).to_string());
          changed = true;
        }
        JobState::Finished(Ok(Err(error))) | JobState::Finished(Err(error)) => {
          self.fail(error);
          changed = true;
        }
      }
    }
    if let Some(job) = self.gpg_job.take() {
      match job.try_state() {
        JobState::Running => self.gpg_job = Some(job),
        JobState::Finished(Ok(Ok(()))) => {
          let order = self.gpg_date_order();
          self.gpg_form.clear(order);
          self.gpg_keys = gpg_keys::load();
          self.success(tr(self.lang, "control_center.gpg_keys_generated").to_string());
          changed = true;
        }
        JobState::Finished(Ok(Err(error))) | JobState::Finished(Err(error)) => {
          self.fail(error);
          changed = true;
        }
      }
    }
    if let Some(job) = self.dnd_job.take() {
      match job.try_state() {
        JobState::Running => self.dnd_job = Some(job),
        JobState::Finished(Ok((enabled, action))) => {
          self.dnd_enabled = enabled;
          if action {
            self.success(if enabled {
              tr(self.lang, "control_center.dnd_enabled").to_string()
            } else {
              tr(self.lang, "control_center.dnd_disabled").to_string()
            });
          }
          changed = true;
        }
        JobState::Finished(Err(error)) => {
          self.fail(format!(
            "{}: {error}",
            tr(self.lang, "control_center.notification_state_error")
          ));
          changed = true;
        }
      }
    }
    if let Some(job) = self.apps_job.take() {
      match job.try_state() {
        JobState::Running => self.apps_job = Some(job),
        JobState::Finished(Ok(apps)) => {
          self.apps = apps;
          self.apps_loading = false;
          self.select_current();
          changed = true;
        }
        JobState::Finished(Err(error)) => {
          self.apps_loading = false;
          self.error_modal = Some(error);
          changed = true;
        }
      }
    }
    if let Some(job) = self.fonts_job.take() {
      match job.try_state() {
        JobState::Running => self.fonts_job = Some(job),
        JobState::Finished(Ok(fonts)) => {
          self.system_fonts = fonts;
          self.select_current();
          changed = true;
        }
        JobState::Finished(Err(error)) => {
          self.error_modal = Some(error);
          changed = true;
        }
      }
    }
    if let Some(job) = self.input_load_job.take() {
      match job.try_state() {
        JobState::Running => self.input_load_job = Some(job),
        JobState::Finished(Ok(settings)) => {
          self.input = settings;
          self.input_loading = false;
          self.input_last_device_refresh = Instant::now();
          self.normalize_selection();
          changed = true;
        }
        JobState::Finished(Err(error)) => {
          self.input_loading = false;
          self.input_failure(error);
          changed = true;
        }
      }
    }
    if !self.input_loading
      && self.page() == Page::MouseTouchpad
      && self.input_device_job.is_none()
      && self.input_last_device_refresh.elapsed() >= Duration::from_secs(2)
    {
      self.input_last_device_refresh = Instant::now();
      self.input_device_job = Some(
        self
          .jobs
          .spawn(|_| Ok((input::detect_devices(), crate::system::ratbag::devices()))),
      );
      changed = true;
    }
    if let Some(job) = self.input_device_job.take() {
      match job.try_state() {
        JobState::Running => self.input_device_job = Some(job),
        JobState::Finished(Ok((devices, ratbag))) => {
          self.input.devices = devices;
          self.input.ratbag = ratbag;
          self.ratbag_device = self
            .ratbag_device
            .min(self.input.ratbag.len().saturating_sub(1));
          self.normalize_selection();
          changed = true;
        }
        JobState::Finished(Err(error)) => {
          eprintln!("argvus-control-center: input device refresh failed: {error}");
        }
      }
    }
    if self.input_job.is_none()
      && let Some(selected) = self.input_toggle_pending.take()
    {
      let mut settings = self.input.clone();
      self.input_job = Some(self.jobs.spawn(move |_| {
        settings.toggle(selected)?;
        Ok(settings)
      }));
      changed = true;
    }
    if self.input_job.is_none()
      && let Some((selected, direction, deadline)) = self.input_pending
      && deadline <= Instant::now()
    {
      self.input_pending = None;
      let mut settings = self.input.clone();
      self.input_job = Some(self.jobs.spawn(move |_| {
        settings.cycle_by(selected, direction)?;
        Ok(settings)
      }));
      changed = true;
    }
    if let Some(job) = self.input_job.take() {
      match job.try_state() {
        JobState::Running => self.input_job = Some(job),
        JobState::Finished(Ok(settings)) => {
          self.input = settings;
          self.input_success();
          changed = true;
        }
        JobState::Finished(Err(error)) => {
          self.input_failure(error);
          changed = true;
        }
      }
    }
    if let Some(job) = self.ratbag_job.take() {
      match job.try_state() {
        JobState::Running => self.ratbag_job = Some(job),
        JobState::Finished(Ok(devices)) => {
          let requested_path = self.ratbag_pending_path.take();
          self.input.ratbag = devices;
          if let Some(path) = requested_path
            && let Some(index) = self
              .input
              .ratbag
              .iter()
              .position(|device| device.path == path)
          {
            self.ratbag_device = index;
          }
          self.ratbag_device = self
            .ratbag_device
            .min(self.input.ratbag.len().saturating_sub(1));
          self.success(tr(self.lang, "control_center.input_applied").to_string());
          self.normalize_selection();
          changed = true;
        }
        JobState::Finished(Err(error)) => {
          self.ratbag_pending_path = None;
          eprintln!("argvus-control-center: ratbag setting failed: {error}");
          self.fail(tr(self.lang, "control_center.input_hardware_apply_failed"));
          changed = true;
        }
      }
    }
    if let Some(job) = self.task_job.take() {
      match job.try_state() {
        JobState::Running => {
          self.task_job = Some(job);
          if self.task_open {
            if self.task_follow
              && let Some(live) = &self.task_live
            {
              self.task_scroll = task_bottom_offset(&live.output());
            }
            changed = true;
          }
        }
        JobState::Finished(Ok(Ok(_))) => {
          self.locale_gen_entries =
            locale::locale_gen_entries(std::path::Path::new("/etc/locale.gen")).unwrap_or_default();
          self.generated_locales = locale::generated_locales();
          self.success(tr(self.lang, "control_center.locales_generated_successfully").to_string());
          changed = true;
        }
        JobState::Finished(Ok(Err(error))) | JobState::Finished(Err(error)) => self.fail(error),
      }
    }
    changed
  }

  /// Whether the System Locales page can run on this distribution. Its
  /// backend (`locale-gen`) is implemented only for Arch Linux, so callers
  /// hide the entry instead of offering a page that always fails.
  pub fn system_locales_available(&self) -> bool {
    self.distro.id == "arch"
  }

  /// Executes the `open_system_page` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn open_system_page(&mut self, page: Page) {
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

  /// Executes the `open_confirm` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn open_confirm(&mut self, action: PendingAction) {
    self.confirm = Some(action);
    self.confirm_focus = ConfirmState::new();
  }

  /// Applies the `apply_system_locales` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn apply_system_locales(&mut self) {
    let valid: BTreeSet<String> = self
      .locale_gen_entries
      .iter()
      .map(|entry| format!("{} {}", entry.locale, entry.encoding))
      .collect();
    if self
      .selected_locales
      .iter()
      .any(|value| !valid.contains(value))
    {
      self.fail("selected locale is not present in /etc/locale.gen");
      return;
    }
    let executable = match std::env::current_exe() {
      Ok(path) => path.to_string_lossy().into_owned(),
      Err(error) => {
        self.fail(error.to_string());
        return;
      }
    };
    let request = match PrivilegedRequest::new(
      "locale",
      "set-generated",
      self.selected_locales.iter().cloned().collect(),
    ) {
      Ok(request) => request,
      Err(error) => {
        self.fail(error.to_string());
        return;
      }
    };
    let live = LiveProcess::new();
    self.task_live = Some(live.clone());
    self.task_open = true;
    self.task_scroll = 0;
    self.task_follow = true;
    live.push_line("$ locale-gen");
    let applied = tr(self.lang, "control_center.applied").to_string();
    let operation = SystemSettingsOperation::new(SystemProcessRunner, executable);
    self.task_job = Some(self.jobs.spawn(move |_| {
      Ok(match operation.execute_live(&request, &live) {
        Ok(output) if output.status == Some(0) => {
          let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
          Ok(if stdout.is_empty() { applied } else { stdout })
        }
        Ok(output) => Err(String::from_utf8_lossy(&output.stderr).trim().to_string()),
        Err(error) => Err(error.to_string()),
      })
    }));
  }

  /// Executes the `refresh_time` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub(crate) fn refresh_time(&mut self) {
    self.datetime = time::datetime_info();
  }

  /// Executes the `refresh_keyboard` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn refresh_keyboard(&mut self) {
    // A draft with pending edits survives the refresh; an untouched draft
    // follows the system, so it never shows a stale layout list.
    let dirty = self.keyboard_layout_changed();
    self.keyboard_info = keyboard::info();
    self.keyboard_variants = keyboard::variants(&self.keyboard_info.x11_layout);
    if !dirty {
      self.keyboard_draft = keyboard_draft::LayoutDraft::loaded(&self.keyboard_info);
    }
  }

  /// Whether Time Zone has a pick that `Apply` has not written yet.
  pub(super) fn timezone_changed(&self) -> bool {
    self.timezone_draft != self.datetime.time_zone
  }

  /// Puts the Time Zone draft back on the zone the system uses.
  pub(super) fn load_timezone_draft(&mut self) {
    self.timezone_draft = self.datetime.time_zone.clone();
  }

  /// Whether Keyboard > Layout has edits that `Apply` has not written yet.
  pub(super) fn keyboard_layout_changed(&self) -> bool {
    self
      .keyboard_draft
      .changed_from(&keyboard_draft::LayoutDraft::loaded(&self.keyboard_info))
  }

  /// Executes the `edited_binding` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn edited_binding(&self) -> Option<&keybindings::Binding> {
    self
      .keybinding_edit_id
      .as_deref()
      .and_then(|id| self.keybindings.iter().find(|b| b.id == id))
  }

  /// Executes the `captured_shortcut` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn captured_shortcut(&self) -> String {
    let mut parts = Vec::new();
    for (enabled, modifier) in self
      .keybinding_editor_modifiers
      .into_iter()
      .zip(["CTRL", "ALT", "SHIFT", "SUPER"])
    {
      if enabled {
        parts.push(modifier);
      }
    }
    parts.push(self.keybinding_editor_key.as_str());
    keybindings::display_keys(&parts.join(" + "))
  }

  /// Executes the `begin_keybinding_capture` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn begin_keybinding_capture(&mut self) {
    if let Some(keys) = self
      .edited_binding()
      .or_else(|| self.selected_keybinding())
      .map(|binding| binding.keys.clone())
    {
      let parts: Vec<&str> = keys.split(" + ").collect();
      self.keybinding_super_required = parts.contains(&"SUPER");
      self.keybinding_editor_modifiers = [
        parts.contains(&"CTRL"),
        parts.contains(&"ALT"),
        parts.contains(&"SHIFT"),
        parts.contains(&"SUPER"),
      ];
      self.keybinding_editor_key = parts.last().copied().unwrap_or_default().to_string();
    }
    self.keybinding_editor_field = 0;
    self.keybinding_capturing = true;
    self.keybinding_detected = false;
    self.success(tr(self.lang, "control_center.keybindings_press_key").to_string());
  }
  /// Checks the condition represented by `is_keybinding_capturing` using only the state available to the module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn is_keybinding_capturing(&self) -> bool {
    self.keybinding_capturing
  }
  /// Checks the condition represented by `has_keybinding_conflict` using only the state available to the module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn has_keybinding_conflict(&self) -> bool {
    self.keybinding_conflict.is_some()
  }

  /// Whether a text field (or the shortcut capture) currently receives every
  /// typed character, so the Control Center must not treat `q`/`?` as the
  /// global quit/help keys. Mirrors the precedence of `event::handle_key`:
  /// the task log and modals sit above the text fields and are not typing.
  pub fn captures_text(&self) -> bool {
    if self.task_open || self.confirm.is_some() || self.error_modal.is_some() {
      return false;
    }
    let capturing_shortcut = self.keybinding_capturing
      && !self.has_keybinding_conflict()
      && matches!(
        self.page(),
        crate::navigation::Page::KeybindingEdit | crate::navigation::Page::KeybindingCapture
      );
    self.hostname_editing || self.admin.editor.is_some() || capturing_shortcut || self.searching
  }
  /// Executes the `keybinding_conflict_state` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn keybinding_conflict_state(&self) -> Option<(&str, &str, Vec<String>)> {
    self
      .keybinding_conflict
      .as_ref()
      .map(|(id, keys, conflicts)| (id.as_str(), keys.as_str(), conflicts.clone()))
  }
  /// Executes the `keybinding_label_for` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn keybinding_label_for(&self, id: &str) -> String {
    self
      .keybindings
      .iter()
      .find(|binding| binding.id == id)
      .map(|binding| keybinding_label(self.lang, binding))
      .unwrap_or_else(|| id.to_string())
  }
  /// Executes the `cancel_keybinding_conflict` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn cancel_keybinding_conflict(&mut self) {
    self.keybinding_conflict = None;
  }
  /// Executes the `replace_keybinding_conflict` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn replace_keybinding_conflict(&mut self) {
    let Some((id, keys, conflicts)) = self.keybinding_conflict.take() else {
      return;
    };
    for old_id in conflicts {
      let _ = keybindings::save_override(&old_id, None, Some(false), &self.keybindings);
    }
    if let Err(error) = keybindings::save_override(&id, Some(keys), Some(true), &self.keybindings) {
      self.fail(error);
      return;
    }
    self.keybindings = keybindings::load();
    self.success(tr(self.lang, "control_center.keybindings_replaced").to_string());
    if self.page() == Page::KeybindingCapture {
      self.navigation.back();
    }
    self.navigation.back();
  }
  /// Executes the `capture_keybinding` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn capture_keybinding(&mut self, key: crossterm::event::KeyEvent) {
    if key.code == crossterm::event::KeyCode::Esc {
      self.keybinding_capturing = false;
      self.keybinding_detected = false;
      return;
    }
    if self.keybinding_detected {
      if key.code == crossterm::event::KeyCode::Enter {
        self.apply_keybinding_editor();
      }
      return;
    }
    if key.code == crossterm::event::KeyCode::Enter && self.keybinding_editor_key.is_empty() {
      self.keybinding_editor_key = "Return".into();
    } else if key.code == crossterm::event::KeyCode::Tab && self.keybinding_editor_key.is_empty() {
      self.keybinding_editor_key = "Tab".into();
    } else if key.code == crossterm::event::KeyCode::Backspace
      && self.keybinding_editor_key.is_empty()
    {
      self.keybinding_editor_key = "BackSpace".into();
    } else {
      self.capture_keybinding_key(key);
    }
    if !self.keybinding_editor_key.is_empty() {
      self.keybinding_detected = true;
      self.keybinding_capturing = false;
    }
  }

  /// Executes the `capture_keybinding_key` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn capture_keybinding_key(&mut self, key: crossterm::event::KeyEvent) {
    if key.code == crossterm::event::KeyCode::Enter {
      self.apply_keybinding_editor();
      return;
    }
    self.keybinding_editor_modifiers = [
      key
        .modifiers
        .contains(crossterm::event::KeyModifiers::CONTROL),
      key.modifiers.contains(crossterm::event::KeyModifiers::ALT),
      key
        .modifiers
        .contains(crossterm::event::KeyModifiers::SHIFT),
      self.keybinding_super_required
        || key
          .modifiers
          .contains(crossterm::event::KeyModifiers::SUPER),
    ];
    let raw = match key.code {
      crossterm::event::KeyCode::Char(c) => match c {
        '?' => "/".into(),
        '_' => "-".into(),
        '+' => "=".into(),
        ':' => ";".into(),
        '"' => "'".into(),
        '{' => "[".into(),
        '}' => "]".into(),
        '|' => "\\".into(),
        '~' => "`".into(),
        '<' => ",".into(),
        '>' => ".".into(),
        other => other.to_string(),
      },
      crossterm::event::KeyCode::Enter => "Return".into(),
      crossterm::event::KeyCode::Tab => "Tab".into(),
      crossterm::event::KeyCode::Backspace => "BackSpace".into(),
      crossterm::event::KeyCode::Left => "left".into(),
      crossterm::event::KeyCode::Right => "right".into(),
      crossterm::event::KeyCode::Up => "up".into(),
      crossterm::event::KeyCode::Down => "down".into(),
      crossterm::event::KeyCode::Esc => "Escape".into(),
      crossterm::event::KeyCode::Delete => "Delete".into(),
      crossterm::event::KeyCode::Insert => "Insert".into(),
      crossterm::event::KeyCode::Home => "Home".into(),
      crossterm::event::KeyCode::End => "End".into(),
      crossterm::event::KeyCode::PageUp => "Page_Up".into(),
      crossterm::event::KeyCode::PageDown => "Page_Down".into(),
      crossterm::event::KeyCode::F(n) => format!("F{n}"),
      _ => return,
    };
    self.keybinding_editor_key = raw;
  }

  /// Applies the `apply_keybinding_editor` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn apply_keybinding_editor(&mut self) {
    let mut parts = Vec::new();
    for (enabled, modifier) in self
      .keybinding_editor_modifiers
      .into_iter()
      .zip(["CTRL", "ALT", "SHIFT", "SUPER"])
    {
      if enabled {
        parts.push(modifier);
      }
    }
    if self.keybinding_editor_key.is_empty() {
      return;
    }
    parts.push(self.keybinding_editor_key.as_str());
    let Ok(keys) = keybindings::normalize_keys(&parts.join(" + ")) else {
      return;
    };
    let binding = self.edited_binding().or_else(|| self.selected_keybinding());
    let Some(binding) = binding else {
      return;
    };
    let conflicts = keybindings::conflicts_in_context(
      &self.keybindings,
      &binding.id,
      &keys,
      &binding.context,
      &binding.input,
    );
    if !conflicts.is_empty() {
      self.keybinding_conflict = Some((binding.id.clone(), keys, conflicts));
      self.confirm_focus = ConfirmState::new();
      self.keybinding_capturing = false;
      return;
    }
    let id = binding.id.clone();
    if let Err(error) = keybindings::save_override(&id, Some(keys), Some(true), &self.keybindings) {
      self.fail(error);
      return;
    }
    self.keybindings = keybindings::load();
    self.keybinding_capturing = false;
    self.success(tr(self.lang, "control_center.keybindings_applied").to_string());
    if self.page() == Page::KeybindingCapture {
      self.navigation.back();
    }
    self.navigation.back();
  }

  /// Executes the `keybinding_editor_state` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn keybinding_editor_state(&self) -> ([bool; 4], &str, usize) {
    (
      self.keybinding_editor_modifiers,
      &self.keybinding_editor_key,
      self.keybinding_editor_field,
    )
  }

  /// Executes the `default_detail` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn default_detail(&self, value: String, default: bool) -> String {
    if default {
      format!(
        "{value} [{}]",
        tr(self.lang, "control_center.argvus_default_5cdad7")
      )
    } else {
      value
    }
  }

  /// Executes the `success` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub(crate) fn success(&mut self, text: String) {
    self.status = Some(Status {
      text,
      kind: StatusKind::Success,
      created: Instant::now(),
    });
  }

  /// Executes the `fail` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub(crate) fn fail(&mut self, error: impl ToString) {
    let error = error.to_string();
    self.status = Some(Status {
      text: error.clone(),
      kind: StatusKind::Error,
      created: Instant::now(),
    });
    self.error_modal = Some(error);
  }
}

/// Executes the `keybinding_is_cheatsheet_entry` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn keybinding_is_cheatsheet_entry(binding: &keybindings::Binding) -> bool {
  !matches!(
    binding.id.as_str(),
    "resize.right"
      | "resize.left"
      | "resize.up"
      | "resize.down"
      | "resize.move_right"
      | "resize.move_left"
      | "resize.move_up"
      | "resize.move_down"
      | "resize.cancel_escape"
      | "resize.cancel_return"
  )
}

/// Executes the `keybinding_section` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn keybinding_section(lang: Lang, binding: &keybindings::Binding) -> String {
  let software = matches!(
    binding.id.as_str(),
    id if id.starts_with("app.")
      || matches!(
        id,
        "system.kitty_cheatsheet"
          | "system.hyprland_cheatsheet"
          | "system.clipboard"
          | "system.clipboard_clear"
          | "system.color_picker"
          | "system.emoji_picker"
          | "system.control_center"
      )
  );
  let media = matches!(
    binding.id.as_str(),
    "session.volume_up"
      | "session.volume_down"
      | "session.mute"
      | "session.brightness_up"
      | "session.brightness_down"
      | "session.play_pause"
      | "session.next_track"
      | "session.previous_track"
      | "session.stop_track"
  );
  let section_key = if media {
    "control_center.keybindings.section.media"
  } else if software {
    "control_center.keybindings.section.software"
  } else {
    "control_center.keybindings.section.desktop"
  };
  tr(lang, section_key).to_string()
}

/// Executes the `keybinding_section_order` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn keybinding_section_order(binding: &keybindings::Binding) -> usize {
  if matches!(
    binding.id.as_str(),
    "session.volume_up"
      | "session.volume_down"
      | "session.mute"
      | "session.brightness_up"
      | "session.brightness_down"
      | "session.play_pause"
      | "session.next_track"
      | "session.previous_track"
      | "session.stop_track"
  ) {
    return 2;
  }
  if binding.id.starts_with("app.")
    || matches!(
      binding.id.as_str(),
      "system.kitty_cheatsheet"
        | "system.hyprland_cheatsheet"
        | "system.clipboard"
        | "system.clipboard_clear"
        | "system.color_picker"
        | "system.emoji_picker"
        | "system.control_center"
    )
  {
    1
  } else {
    0
  }
}

/// Executes the `keybinding_label` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn keybinding_label(lang: Lang, binding: &keybindings::Binding) -> String {
  if let Some(description) = keybindings::cheatsheet_description(lang, binding) {
    return description;
  }
  let translated = tr(lang, &binding.description_key);
  if translated != binding.description_key {
    translated.to_string()
  } else {
    let label = binding
      .id
      .rsplit('.')
      .next()
      .unwrap_or(&binding.id)
      .split('_')
      .map(|part| {
        let mut chars = part.chars();
        match chars.next() {
          Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
          None => String::new(),
        }
      })
      .collect::<Vec<_>>()
      .join(" ");
    match label.as_str() {
      "Terminal" => "Open terminal".into(),
      "Browser" => "Open default browser".into(),
      "File Manager" => "Open file manager".into(),
      "Launcher" => "Open application launcher".into(),
      "Close" => "Close window".into(),
      "Maximize" => "Maximize window".into(),
      "Next" => "Next workspace".into(),
      "Previous" => "Previous workspace".into(),
      _ => label,
    }
  }
}

/// Executes the `keybinding_category` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn keybinding_category(lang: Lang, category: &str) -> String {
  let key = format!("control_center.keybindings.category.{category}");
  let translated = tr(lang, &key);
  if translated == key {
    category.to_string()
  } else {
    translated.to_string()
  }
}

/// Executes the `keybinding_category_order` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn keybinding_category_order(category: &str) -> usize {
  [
    "applications",
    "windows",
    "navigation",
    "workspaces",
    "system",
    "session",
    "media",
    "screenshots",
    "appearance",
    "widgets",
  ]
  .iter()
  .position(|value| *value == category)
  .unwrap_or(usize::MAX)
}

/// Executes the `search_matches` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn search_matches(query: &str, values: &[&str]) -> bool {
  query.is_empty()
    || values
      .iter()
      .any(|value| value.to_lowercase().contains(&query.to_lowercase()))
}

/// Executes the `category_icon` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn category_icon(category: Category) -> &'static str {
  match category {
    Category::Terminal => argvus_tui::icons::TERMINAL,
    Category::FileManager => argvus_tui::icons::FOLDER,
    Category::TextEditor => argvus_tui::icons::TEXT_EDITOR,
    Category::TerminalEditor => argvus_tui::icons::KEYBOARD,
    Category::Browser => argvus_tui::icons::EARTH,
    Category::ImageViewer => argvus_tui::icons::IMAGE,
    Category::PdfViewer => argvus_tui::icons::PDF,
    Category::VideoPlayer => argvus_tui::icons::VIDEO,
    Category::AudioPlayer => argvus_tui::icons::MUSIC,
    Category::Archive => argvus_tui::icons::PACKAGES,
    Category::Launcher => argvus_tui::icons::LAUNCHER,
  }
}

/// Executes the `category_label` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn category_label(lang: Lang, category: Category) -> &'static str {
  let key = match category {
    Category::Terminal => "control_center.terminal",
    Category::FileManager => "control_center.file_manager",
    Category::TextEditor => "control_center.text_editor",
    Category::TerminalEditor => "control_center.terminal_editor",
    Category::Browser => "control_center.browser",
    Category::ImageViewer => "control_center.image_viewer",
    Category::PdfViewer => "control_center.pdf_viewer",
    Category::VideoPlayer => "control_center.video_player",
    Category::AudioPlayer => "control_center.audio_player",
    Category::Archive => "control_center.archive",
    Category::Launcher => "control_center.launcher",
  };
  crate::i18n::tr(lang, key)
}

/// Executes the `font_target_icon` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn font_target_icon(target: FontTarget) -> &'static str {
  match target {
    FontTarget::Taskbar => argvus_tui::icons::TASKBAR,
    FontTarget::Sysinfo => argvus_tui::icons::DIAGNOSTICS,
    FontTarget::ControlPanel => argvus_tui::icons::CONTROL_PANEL,
    FontTarget::System => argvus_tui::icons::SETTINGS,
    FontTarget::Apps => argvus_tui::icons::APPS,
    FontTarget::Terminal => argvus_tui::icons::TERMINAL,
    FontTarget::Browser => argvus_tui::icons::EARTH,
  }
}

/// Executes the `setting_icon` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn setting_icon(setting: SettingKind) -> &'static str {
  match setting {
    SettingKind::Antialiasing => argvus_tui::icons::EFFECT,
    SettingKind::Hinting => argvus_tui::icons::RULER,
    SettingKind::Subpixel => argvus_tui::icons::PALETTE,
    SettingKind::Dpi => argvus_tui::icons::MONITOR,
  }
}

/// Executes the `font_target_label` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn font_target_label(lang: Lang, target: FontTarget) -> &'static str {
  match target {
    FontTarget::Taskbar => tr(lang, "control_center.taskbar_font"),
    FontTarget::Sysinfo => tr(lang, "control_center.widget_telemetry_font"),
    FontTarget::ControlPanel => tr(lang, "control_center.control_panel_font"),
    FontTarget::System => tr(lang, "control_center.system_font"),
    FontTarget::Apps => tr(lang, "control_center.applications_font"),
    FontTarget::Terminal => tr(lang, "control_center.terminal_font"),
    FontTarget::Browser => tr(lang, "control_center.browser_font"),
  }
}

/// Executes the `setting_label` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn setting_label(lang: Lang, setting: SettingKind) -> &'static str {
  match setting {
    SettingKind::Antialiasing => tr(lang, "control_center.antialiasing"),
    SettingKind::Hinting => "Hinting",
    SettingKind::Subpixel => tr(lang, "control_center.subpixel_order"),
    SettingKind::Dpi => "DPI",
  }
}

/// Executes the `pending_action_text` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn pending_action_text(lang: Lang, action: &PendingAction) -> (String, String) {
  match action {
    PendingAction::Administration { message, .. } => (
      tr(lang, "control_center.confirm_administrative_operation").into(),
      message.clone(),
    ),
    PendingAction::ResetApps => (
      tr(lang, "control_center.reset_default_applications").to_string(),
      tr(
        lang,
        "control_center.this_will_restore_the_application_choices_defined_by_argvus",
      )
      .to_string(),
    ),
    PendingAction::ResetApp(category) => (
      format!("{}?", tr(lang, "control_center.reset_default_application")),
      category_label(lang, *category).to_string(),
    ),
    PendingAction::ResetFonts => (
      tr(lang, "control_center.reset_font_settings").to_string(),
      tr(
        lang,
        "control_center.this_will_restore_the_font_settings_defined_by_argvus",
      )
      .to_string(),
    ),
    PendingAction::ResetFont(target) => (
      format!("{}?", tr(lang, "control_center.reset_font")),
      font_target_label(lang, *target).to_string(),
    ),
    PendingAction::ResetFontSetting(setting) => (
      format!("{}?", tr(lang, "control_center.reset_setting")),
      setting_label(lang, *setting).to_string(),
    ),
    PendingAction::ResetKeybindings => (
      tr(lang, "control_center.restore_all_shortcuts").to_string(),
      tr(lang, "control_center.restore_all_shortcuts_description").to_string(),
    ),
    PendingAction::RemoveWindowRule(name) => (
      tr(lang, "control_center.window_rules_remove").to_string(),
      name.clone(),
    ),
    PendingAction::RemoveProject(path) => (
      tr(lang, "control_center.projects_remove_title").to_string(),
      path.clone(),
    ),
    PendingAction::RemoveSnippet(name) => (
      tr(lang, "control_center.snippets_remove_title").to_string(),
      name.clone(),
    ),
    PendingAction::DeleteSshKey(name) => (
      tr(lang, "control_center.ssh_keys_delete_title").to_string(),
      format!(
        "{name}. {}",
        tr(lang, "control_center.ssh_keys_delete_description")
      ),
    ),
    PendingAction::GenerateGpgNoPassphrase => (
      tr(lang, "control_center.gpg_keys_no_passphrase_title").to_string(),
      tr(lang, "control_center.gpg_keys_no_passphrase_description").to_string(),
    ),
    PendingAction::DeleteGpgKey(fingerprint) => (
      tr(lang, "control_center.gpg_keys_delete_title").to_string(),
      format!(
        "{fingerprint}. {}",
        tr(lang, "control_center.gpg_keys_delete_description")
      ),
    ),
    PendingAction::ExportGpgPrivate(fingerprint) => (
      tr(lang, "control_center.gpg_keys_export_private_confirm_title").to_string(),
      format!(
        "{fingerprint}. {}",
        tr(lang, "control_center.gpg_keys_export_private_description")
      ),
    ),
    PendingAction::ImportGpgPrivate => (
      tr(lang, "control_center.gpg_keys_import_private_confirm_title").to_string(),
      tr(lang, "control_center.gpg_keys_import_private_description").to_string(),
    ),
    PendingAction::RevokeGpgKey(fingerprint, _) => (
      tr(lang, "control_center.gpg_keys_revoke_title").to_string(),
      format!(
        "{fingerprint}. {}",
        tr(lang, "control_center.gpg_keys_revoke_description")
      ),
    ),
    PendingAction::SignGpgKey(fingerprint, signer) => (
      tr(lang, "control_center.gpg_keys_sign_title").to_string(),
      format!(
        "{fingerprint} → {signer}. {}",
        tr(lang, "control_center.gpg_keys_sign_description")
      ),
    ),
    PendingAction::SendGpgKey(fingerprint) => (
      tr(lang, "control_center.gpg_keys_send_title").to_string(),
      format!(
        "{fingerprint}. {}",
        tr(lang, "control_center.gpg_keys_send_description")
      ),
    ),
    PendingAction::RestoreKeybinding { .. } => (
      tr(lang, "control_center.restore_shortcut_title").to_string(),
      tr(lang, "control_center.restore_shortcut_description").to_string(),
    ),
    PendingAction::ApplySystemLocales => (
      tr(lang, "control_center.apply_locale_changes").to_string(),
      tr(
        lang,
        "control_center.this_will_update_etc_locale_gen_and_run_locale_gen",
      )
      .to_string(),
    ),
    PendingAction::DiscardDraft => (
      tr(lang, "control_center.discard_changes_title").to_string(),
      tr(lang, "control_center.discard_changes_description").to_string(),
    ),
    PendingAction::SetNtp(enabled) => (
      tr(lang, "control_center.change_automatic_date_time").to_string(),
      enabled_label(lang, *enabled).to_string(),
    ),
  }
}

/// Executes the `setting_value_label` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn setting_value_label(lang: Lang, value: &str) -> String {
  match value {
    "enabled" => tr(lang, "control_center.enabled_16b283").to_string(),
    "disabled" => tr(lang, "control_center.disabled_8ccfd8").to_string(),
    "none" => tr(lang, "control_center.none").to_string(),
    "slight" => tr(lang, "control_center.slight").to_string(),
    "medium" => tr(lang, "control_center.medium").to_string(),
    "full" => tr(lang, "control_center.full").to_string(),
    "automatic" => tr(lang, "control_center.automatic_b0d36e").to_string(),
    _ => value.to_string(),
  }
}

/// Executes the `enabled_label` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn enabled_label(lang: Lang, enabled: bool) -> &'static str {
  if enabled {
    tr(lang, "control_center.enabled")
  } else {
    tr(lang, "control_center.disabled")
  }
}

/// Executes the `non_empty` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn non_empty(value: &str) -> String {
  if value.is_empty() {
    "-".to_string()
  } else {
    value.to_string()
  }
}

/// Defines the constant `TASK_POPUP_WIDTH`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub(crate) const TASK_POPUP_WIDTH: u16 = 100;
/// Defines the constant `TASK_POPUP_HEIGHT`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub(crate) const TASK_POPUP_HEIGHT: u16 = 20;
/// Defines the constant `TASK_CONTENT_WIDTH`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub(crate) const TASK_CONTENT_WIDTH: usize = TASK_POPUP_WIDTH as usize - 2;
/// Defines the constant `TASK_CONTENT_HEIGHT`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub(crate) const TASK_CONTENT_HEIGHT: usize = TASK_POPUP_HEIGHT as usize - 2;

/// Executes the `task_wrapped_lines` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub(crate) fn task_wrapped_lines(output: &str) -> usize {
  output
    .lines()
    .map(|line| {
      let width = argvus_tui::text::display_width(line);
      if width == 0 {
        1
      } else {
        width.div_ceil(TASK_CONTENT_WIDTH)
      }
    })
    .sum()
}

/// Executes the `task_bottom_offset` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub(crate) fn task_bottom_offset(output: &str) -> u16 {
  task_wrapped_lines(output).saturating_sub(TASK_CONTENT_HEIGHT) as u16
}

/// Executes the `language_from_selected` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn canonical_language() -> Option<Lang> {
  let output = argvus_control_center_core::process::command("argvus")
    .arg("config")
    .args(["get", "/session/language", "--effective", "--raw"])
    .output()
    .ok()?;
  if !output.status.success() {
    return None;
  }
  match String::from_utf8_lossy(&output.stdout).trim() {
    "pt-BR" | "pt_BR" => Some(Lang::for_locale("pt-BR")),
    "en-US" | "en_US" => Some(Lang::for_locale("en-US")),
    _ => None,
  }
}

#[cfg(test)]
mod tests;
