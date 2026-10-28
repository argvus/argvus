//! Application state and main-flow coordination of the Settings pages.
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

mod confirmations;
mod jobs;
mod keybinding_capture;
mod labels;
mod navigation;
mod system_settings;

pub use confirmations::pending_action_text;
use labels::canonical_language;
pub use labels::{
  category_icon, category_label, font_target_icon, font_target_label, search_matches, setting_icon,
  setting_label,
};

pub const MIN_WIDTH: u16 = 60;
pub const MIN_HEIGHT: u16 = 15;

#[derive(Debug, Clone, Copy)]
pub enum StatusKind {
  Success,
  Error,
}

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
  /// Last poll of the layout in use, so the keyboard pages follow the taskbar.
  keyboard_last_refresh: Instant,
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
  pub fn new(initial: Page) -> Self {
    let lang = canonical_language().unwrap_or_else(Lang::detect);
    Self::with_context(initial, lang, Theme::load())
  }

  /// Replaces the semantic theme used by this page.
  pub fn set_theme(&mut self, theme: &Theme) {
    self.theme = theme.clone();
  }

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
      keyboard_variants: keyboard::variants(&keyboard_info.default_layout),
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
      keyboard_last_refresh: Instant::now() - Duration::from_secs(1),
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

  pub fn adjust_size(&mut self, delta: i16) {
    if matches!(self.page(), Page::FontSelector(_)) && !self.searching {
      self.pending_size = (self.pending_size as i16 + delta).clamp(8, 32) as u16;
    }
  }

  pub fn resize(&mut self, width: u16, height: u16) {
    self.width = width;
    self.height = height;
  }

  pub fn too_small(&self) -> bool {
    self.width < MIN_WIDTH || self.height < MIN_HEIGHT
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
}

pub(crate) const TASK_POPUP_WIDTH: u16 = 100;
pub(crate) const TASK_POPUP_HEIGHT: u16 = 20;
pub(crate) const TASK_CONTENT_WIDTH: usize = TASK_POPUP_WIDTH as usize - 2;
pub(crate) const TASK_CONTENT_HEIGHT: usize = TASK_POPUP_HEIGHT as usize - 2;

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

pub(crate) fn task_bottom_offset(output: &str) -> u16 {
  task_wrapped_lines(output).saturating_sub(TASK_CONTENT_HEIGHT) as u16
}

#[cfg(test)]
mod tests;
