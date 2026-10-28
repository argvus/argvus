//! Terminal UI rendering and interaction of the Packages page.
use crate::{
  backend::{PackageBackend, PackageError, parse_reflector_countries, reflector_args},
  model::{
    AurPackage, CachePackage, HistoryEntry, Mirror, Package, PackageDashboard, PackageDetails,
    PackageSource, PackageTask, PackagesPage, ReflectorOptions, TransactionPlan, Update,
  },
};
use argvus_control_center_core::format::human_bytes;
use argvus_control_center_core::navigation::HomeCardId;
use argvus_control_center_core::{
  capabilities::Capabilities,
  config::AppConfig,
  jobs::{JobHandle, JobManager, JobState},
  privileged::{PrivilegedRequest, SystemSettingsOperation},
  process::{LiveProcess, ProcessRequest, ProcessRunner, SystemProcessRunner},
  sanitize::terminal_text,
};
use argvus_i18n::{Lang, tr};
use argvus_theme::Theme;
use argvus_tui::{
  action_buttons::{ActionButton, ActionFocus, draw_aligned_transparent},
  chrome,
  components::{StatusKind, StatusMessage},
  confirm::{
    ChoiceDialog, ChoiceOutcome, ChoiceState, ConfirmDialog, ConfirmOutcome, ConfirmState,
    draw_choice, draw_confirm,
  },
  hints::{HintContext, choice_hints, confirm_hints, hints},
  icons,
  menu::{MenuEvent, MenuState, MenuStyle, Row, RowKind, draw_menu},
  page::{shell, status},
  text_editor::{EditorOutcome, TextEditor},
};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
  Frame,
  layout::{Alignment, Constraint, Layout, Rect},
  style::{Modifier, Style},
  text::{Line, Span},
  widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

mod repositories;

use argvus_control_center_core::pacman_conf::Repository;
use repositories::{DraftField, RepositoryDraft, RepositoryReview};

mod actions;
mod input;
mod jobs;
mod navigation;
mod render;
mod rows;

enum Loaded {
  Packages(Vec<Package>),
  Updates(Vec<Update>),
  Cache(Vec<CachePackage>),
  History(Vec<HistoryEntry>),
  Aur(Vec<AurPackage>),
  Mirrors(Vec<Mirror>),
  Details(Box<PackageDetails>),
  MirrorPreview(String),
  MirrorCountries(Vec<String>),
  Dashboard(PackageDashboard),
  Repositories(Vec<Repository>),
  /// The checked draft of the Add page; its confirmation opens next.
  RepositoryReview(Box<RepositoryReview>),
}
#[derive(Debug, Clone, PartialEq, Eq)]
enum Action {
  Install(Vec<String>),
  Remove(Vec<String>),
  /// Removes only the packages, skipping the dependency checks (`-Rdd`):
  /// the installed packages that need them stay and may stop working.
  RemoveOnly(Vec<String>),
  /// Removes the packages together with the installed packages that depend
  /// on them (`dependents`). When a plain removal would break them, the
  /// confirmation becomes a choice between this and [`Action::RemoveOnly`].
  RemoveCascade {
    packages: Vec<String>,
    dependents: Vec<String>,
  },
  Reinstall(String),
  UpgradePackage(String),
  Upgrade,
  RefreshDatabase,
  CleanCache(&'static str),
  Downgrade(String),
  AurInstall(String),
  /// Updates one AUR package with the helper, as the normal user.
  AurUpgradePackage(String),
  /// Updates every AUR package with the helper, as the normal user.
  AurUpgrade,
  ApplyMirrors(String),
  /// Adds the reviewed repository: imports and locally signs its key, writes
  /// its file in `/etc/pacman.d/` and includes it in `pacman.conf`.
  RepoAdd(Box<RepositoryReview>),
  /// The full upgrade offered right after a repository was added, so its
  /// database is synchronized without a partial upgrade.
  UpgradeAfterRepoAdd,
  RepoEnable(String),
  RepoDisable(String),
  /// Removes a repository; `key` is the fingerprint it recorded, deleted
  /// from the keyring too when `delete_key`.
  RepoRemove {
    name: String,
    key: Option<String>,
    delete_key: bool,
  },
}

impl Action {
  /// Removals, cache cleanups and downgrades, confirmed in the danger style
  /// like the Danger zone rows that start them; the other operations use
  /// the plain confirmation.
  fn is_destructive(&self) -> bool {
    matches!(
      self,
      Self::Remove(_)
        | Self::RemoveOnly(_)
        | Self::RemoveCascade { .. }
        | Self::CleanCache(_)
        | Self::Downgrade(_)
        | Self::RepoAdd(_)
        | Self::RepoRemove { .. }
    )
  }

  /// The ways to run this action when the confirmation offers more than one.
  fn choices(&self) -> Option<[Action; 2]> {
    match self {
      Self::RemoveCascade { packages, .. } => {
        Some([Self::RemoveOnly(packages.clone()), self.clone()])
      }
      Self::RepoRemove {
        name,
        key: Some(key),
        delete_key: false,
      } => Some([
        self.clone(),
        Self::RepoRemove {
          name: name.clone(),
          key: Some(key.clone()),
          delete_key: true,
        },
      ]),
      _ => None,
    }
  }
}

/// What the open text field edits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InputMode {
  /// The package search query (`/`).
  Search,
  /// Reflector maximum age, in hours.
  MirrorAge,
  /// Reflector mirror count.
  MirrorCount,
}

/// Bounds of the reflector maximum age, in hours (one year).
const MIRROR_AGE_RANGE: (u32, u32) = (1, 24 * 365);
/// Bounds of the reflector mirror count.
const MIRROR_COUNT_RANGE: (u32, u32) = (1, 100);
/// Longest search query accepted, as before the menu migration.
const QUERY_MAX: usize = 128;

/// Stable identity of a Packages menu row. List items keep their index in
/// the loaded source list (not their visible position), so filtering never
/// points an action at another package.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Item {
  /// A page opened from the Packages home, a source menu or the Remove list.
  Open(PackagesPage),
  /// Index into `packages` on the Install (Official) and Remove lists.
  Package(usize),
  /// Index into `aur`, on the Install (AUR) list.
  Aur(usize),
  /// Index into `packages` on the Orphans list.
  Orphan(usize),
  /// Index into `updates`, on the Update lists.
  Update(usize),
  /// Index into `cache`, on the Downgrade list.
  CacheFile(usize),
  /// Installs (or reinstalls) the open package.
  Install,
  /// Removes the open package.
  Remove,
  /// Upgrades every pending update of the open Update list.
  UpgradeAll,
  RefreshDatabase,
  /// Removes the orphans marked with Space.
  RemoveMarked,
  CleanKeepThree,
  CleanKeepOne,
  CleanUninstalled,
  /// Opens the read-only list of cached files.
  CacheFiles,
  /// Opens the reflector options.
  ConfigureMirrors,
  MirrorCountry,
  MirrorProtocol,
  MirrorAge,
  MirrorCount,
  MirrorSort,
  MirrorPreview,
  /// A field of the repository Add page.
  RepoField(DraftField),
  /// `Review and add` on the Add page.
  RepoReview,
  /// Index into `repositories`, on the List page.
  Repository(usize),
  /// The Enabled toggle of the open repository.
  RepoToggle,
  /// Remove, in the Danger zone of the open repository.
  RepoRemove,
}

impl Item {
  /// Rows that only open a page, show a confirmation or edit the reflector
  /// options: they work while a list loads. Package operations wait for
  /// every running job, as the action buttons did.
  fn waits_for_jobs(self) -> bool {
    matches!(
      self,
      Self::Package(_)
        | Self::Aur(_)
        | Self::Install
        | Self::Remove
        | Self::UpgradeAll
        | Self::RefreshDatabase
        | Self::RemoveMarked
        | Self::CleanKeepThree
        | Self::CleanKeepOne
        | Self::CleanUninstalled
        | Self::CacheFile(_)
    )
  }

  /// Rows of the reflector options, which the editor handled even while a
  /// transaction ran.
  fn is_mirror_option(self) -> bool {
    matches!(
      self,
      Self::MirrorCountry
        | Self::MirrorProtocol
        | Self::MirrorAge
        | Self::MirrorCount
        | Self::MirrorSort
        | Self::MirrorPreview
    )
  }

  /// Rows of a package list, where a new filter puts the cursor.
  fn is_list_entry(self) -> bool {
    matches!(
      self,
      Self::Package(_) | Self::Aur(_) | Self::Orphan(_) | Self::Update(_)
    )
  }
}

pub struct PackagesApp {
  pub page: PackagesPage,
  menu: MenuState,
  /// Focus of the action bar under the Updates list (the list has it by default).
  actions: ActionFocus,
  /// Body height during the last draw, used as the PgUp/PgDn distance.
  list_height: u16,
  packages: Vec<Package>,
  updates: Vec<Update>,
  cache: Vec<CachePackage>,
  history: Vec<HistoryEntry>,
  aur: Vec<AurPackage>,
  mirrors: Vec<Mirror>,
  dashboard: PackageDashboard,
  details: Option<PackageDetails>,
  /// Name of the package whose details page is open, known before its
  /// metadata arrives.
  details_name: Option<String>,
  /// The list that opened the details page, and the row to return to.
  details_parent: PackagesPage,
  details_origin: Option<Item>,
  query: String,
  job: Option<JobHandle<Result<Loaded, String>>>,
  last_loaded: Option<(String, std::time::Instant)>,
  plan: Option<JobHandle<Result<(Action, TransactionPlan), String>>>,
  action: Option<JobHandle<Result<String, String>>>,
  jobs: JobManager,
  lang: Lang,
  theme: Theme,
  capabilities: Capabilities,
  input: Option<String>,
  input_mode: Option<InputMode>,
  pending: Option<Action>,
  confirmation: ConfirmState,
  /// Focus of the choice that replaces the confirmation for actions with
  /// more than one way to run (see [`Action::choices`]).
  choice: ChoiceState,
  preview: Option<TransactionPlan>,
  pub status: Option<StatusMessage>,
  /// Names of the orphans marked with Space.
  marked: Vec<String>,
  /// Reflector options while the mirror editor page is open.
  mirror_options: Option<ReflectorOptions>,
  mirror_countries: Option<Vec<String>>,
  transaction_live: Option<LiveProcess>,
  transaction_open: bool,
  transaction_scroll: u16,
  transaction_follow: bool,
  /// The unofficial repositories of `pacman.conf`.
  repositories: Vec<Repository>,
  /// The repository being added on the Add page.
  repo_draft: RepositoryDraft,
  /// The text field open over the Add page, and the field it edits.
  editor: Option<(DraftField, TextEditor)>,
  /// The confirmation shown when leaving the Add page with a draft.
  discard: Option<ConfirmState>,
  /// The operation running now, to know what follows when it succeeds.
  running: Option<Action>,
  /// What to ask once the operation output is closed (the upgrade offered
  /// after adding a repository).
  follow_up: Option<Action>,
}

impl PackagesApp {
  /// Replaces the semantic theme used by this page.
  pub fn set_theme(&mut self, theme: &Theme) {
    self.theme = theme.clone();
  }

  pub fn new(lang: Lang, theme: Theme, capabilities: Capabilities) -> Self {
    Self {
      page: PackagesPage::Home,
      menu: MenuState::default(),
      actions: ActionFocus::default(),
      list_height: 0,
      packages: vec![],
      updates: vec![],
      cache: vec![],
      history: vec![],
      aur: vec![],
      mirrors: vec![],
      dashboard: PackageDashboard::default(),
      details: None,
      details_name: None,
      details_parent: PackagesPage::Task(PackageSource::Official, PackageTask::Install),
      details_origin: None,
      query: String::new(),
      job: None,
      last_loaded: None,
      plan: None,
      action: None,
      jobs: JobManager::default(),
      lang,
      theme,
      capabilities,
      input: None,
      input_mode: None,
      pending: None,
      confirmation: ConfirmState::new(),
      choice: ChoiceState::new(),
      preview: None,
      status: None,
      marked: vec![],
      mirror_options: None,
      mirror_countries: None,
      transaction_live: None,
      transaction_open: false,
      transaction_scroll: 0,
      transaction_follow: false,
      repositories: vec![],
      repo_draft: RepositoryDraft::default(),
      editor: None,
      discard: None,
      running: None,
      follow_up: None,
    }
  }
}

#[cfg(test)]
mod tests;
