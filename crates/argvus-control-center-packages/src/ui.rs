//! Implements terminal UI rendering and interaction in crate `argvus control center packages`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use crate::{
  backend::{PackageBackend, parse_reflector_countries, reflector_args},
  model::{
    AurPackage, CachePackage, HistoryEntry, Mirror, Package, PackageDashboard, PackageDetails,
    PackagesPage, ReflectorOptions, TransactionPlan, Update,
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
  chrome,
  components::{StatusKind, StatusMessage},
  confirm::{ConfirmDialog, ConfirmOutcome, ConfirmState, draw_confirm},
  hints::{HintContext, confirm_hints, hints},
  icons,
  menu::{MenuEvent, MenuState, MenuStyle, Row, RowKind, draw_menu},
  page::{shell, status},
};
use crossterm::event::KeyCode;
use ratatui::{
  Frame,
  style::Style,
  text::Line,
  widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

/// Defines `Loaded`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
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
}
#[derive(Debug, Clone, PartialEq, Eq)]
/// Defines `Action`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
enum Action {
  Install(Vec<String>),
  Remove(Vec<String>),
  Reinstall(String),
  UpgradePackage(String),
  Upgrade,
  RefreshDatabase,
  CleanCache(&'static str),
  Downgrade(String),
  AurInstall(String),
  ApplyMirrors(String),
}

impl Action {
  /// Removals, cache cleanups and downgrades, confirmed in the danger style
  /// like the Danger zone rows that start them; the other operations use
  /// the plain confirmation.
  fn is_destructive(&self) -> bool {
    matches!(
      self,
      Self::Remove(_) | Self::CleanCache(_) | Self::Downgrade(_)
    )
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
  /// A page opened from the Packages home.
  Open(PackagesPage),
  /// Index into `packages` on the Search and Installed lists.
  Package(usize),
  /// Index into `aur`.
  Aur(usize),
  /// Index into `packages` on the Orphans list.
  Orphan(usize),
  /// Index into `updates`.
  Update(usize),
  /// Index into `cache`, on the Downgrade list.
  CacheFile(usize),
  /// Installs (or reinstalls) the open package.
  Install,
  /// Removes the open package.
  Remove,
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
}

impl Item {
  /// Rows that only open a page, show a confirmation or edit the reflector
  /// options: they work while a list loads. Package operations wait for
  /// every running job, as the action buttons did.
  fn waits_for_jobs(self) -> bool {
    matches!(
      self,
      Self::Install
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

/// Represents `PackagesApp`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct PackagesApp {
  pub page: PackagesPage,
  menu: MenuState,
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
  details_parent: PackagesPage,
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
}

/// Defines the constant `TRANSACTION_POPUP_WIDTH`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
const TRANSACTION_POPUP_WIDTH: u16 = 100;
/// Defines the constant `TRANSACTION_POPUP_HEIGHT`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
const TRANSACTION_POPUP_HEIGHT: u16 = 20;
/// Defines the constant `TRANSACTION_CONTENT_WIDTH`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
const TRANSACTION_CONTENT_WIDTH: usize = TRANSACTION_POPUP_WIDTH as usize - 2;
/// Defines the constant `TRANSACTION_CONTENT_HEIGHT`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
const TRANSACTION_CONTENT_HEIGHT: usize = TRANSACTION_POPUP_HEIGHT as usize - 2;

/// Executes the `transaction_wrapped_lines` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn transaction_wrapped_lines(output: &str) -> usize {
  output
    .lines()
    .map(|line| (argvus_tui::text::display_width(line).div_ceil(TRANSACTION_CONTENT_WIDTH)).max(1))
    .sum()
}

/// Executes the `transaction_bottom_offset` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn transaction_bottom_offset(output: &str) -> u16 {
  transaction_wrapped_lines(output).saturating_sub(TRANSACTION_CONTENT_HEIGHT) as u16
}

/// Whether typed characters filter the list on `page` (D8): letters,
/// including `j`, `k`, `r`, `q` and `?`, become text of the filter.
fn filters_while_typing(page: PackagesPage) -> bool {
  matches!(
    page,
    PackagesPage::Search
      | PackagesPage::Installed
      | PackagesPage::Orphans
      | PackagesPage::Updates
      | PackagesPage::Aur
  )
}

impl PackagesApp {
  /// Replaces the semantic theme used by this page.
  pub fn set_theme(&mut self, theme: &Theme) {
    self.theme = theme.clone();
  }

  /// Constructs `new` with this module's expected initial state. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn new(lang: Lang, theme: Theme, capabilities: Capabilities) -> Self {
    Self {
      page: PackagesPage::Home,
      menu: MenuState::default(),
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
      details_parent: PackagesPage::Search,
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
      preview: None,
      status: None,
      marked: vec![],
      mirror_options: None,
      mirror_countries: None,
      transaction_live: None,
      transaction_open: false,
      transaction_scroll: 0,
      transaction_follow: false,
    }
  }
  /// Executes the `busy` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn busy(&self) -> bool {
    self.job.is_some() || self.plan.is_some() || self.action.is_some()
  }
  /// Executes the `destructive` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn destructive(&self) -> bool {
    self.plan.is_some() || self.action.is_some()
  }
  /// Executes the `reload` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn reload(&mut self) {
    self.start_reload(false);
  }

  fn reload_force(&mut self) {
    self.start_reload(true);
  }

  fn reload_key(&self) -> String {
    format!(
      "{:?}|{}|{}",
      self.page,
      self.query,
      self.details_target().unwrap_or_default()
    )
  }

  fn start_reload(&mut self, force: bool) {
    if self.destructive() {
      return;
    }
    if self.job.is_some() {
      return;
    }
    if matches!(self.page, PackagesPage::Search | PackagesPage::Aur) && self.query.trim().is_empty()
    {
      self.status = Some(StatusMessage {
        kind: StatusKind::Info,
        text: match self.page {
          PackagesPage::Aur => tr(self.lang, "control_center.press_to_search_the_aur"),
          _ => tr(self.lang, "control_center.press_to_search_packages"),
        }
        .into(),
      });
      return;
    }
    let key = self.reload_key();
    if !force
      && self
        .last_loaded
        .as_ref()
        .is_some_and(|(loaded_key, at)| loaded_key == &key && at.elapsed().as_secs() < 10)
    {
      return;
    }
    let caps = self.capabilities.clone();
    let page = self.page;
    let query = self.query.clone();
    let name = self.details_target();
    self.job = Some(self.jobs.spawn(move |_| {
      let b = PackageBackend::new(SystemProcessRunner, caps);
      let data = match page {
        PackagesPage::Installed => Loaded::Packages(b.installed().map_err(|e| e.to_string())?),
        PackagesPage::Search => Loaded::Packages(b.search(&query).map_err(|e| e.to_string())?),
        PackagesPage::Updates => Loaded::Updates(b.updates().map_err(|e| e.to_string())?),
        PackagesPage::Orphans => Loaded::Packages(b.orphans().map_err(|e| e.to_string())?),
        PackagesPage::Cache | PackagesPage::CacheFiles | PackagesPage::Downgrade => {
          Loaded::Cache(b.cache())
        }
        PackagesPage::History => Loaded::History(b.history().map_err(|e| e.to_string())?),
        PackagesPage::Mirrors | PackagesPage::MirrorEditor => Loaded::Mirrors(b.mirrors()),
        PackagesPage::Aur => {
          let h = b.aur_helper().ok_or("AUR helper is unavailable")?;
          Loaded::Aur(b.aur_search(h, &query).map_err(|e| e.to_string())?)
        }
        PackagesPage::Details(_) | PackagesPage::HistoryDetails(_) => Loaded::Details(Box::new(
          b.details(&name.ok_or("package not found")?)
            .map_err(|e| e.to_string())?,
        )),
        PackagesPage::Home => Loaded::Dashboard(b.dashboard()),
      };
      Ok(Ok(data))
    }));
    self.status = Some(StatusMessage {
      kind: StatusKind::Info,
      text: tr(self.lang, "control_center.loading_packages").into(),
    });
  }
  /// Executes the `poll` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn poll(&mut self) -> bool {
    if let Some(plan) = self.plan.take() {
      match plan.try_state() {
        JobState::Running => {
          self.plan = Some(plan);
          return false;
        }
        JobState::Finished(Ok(Ok((action, preview)))) => {
          self.request(action);
          self.preview = Some(preview);
          self.status = None;
          return true;
        }
        JobState::Finished(Ok(Err(error))) | JobState::Finished(Err(error)) => {
          self.error(error);
          return true;
        }
      }
    }
    if let Some(job) = self.job.take() {
      match job.try_state() {
        JobState::Running => {
          self.job = Some(job);
          return false;
        }
        JobState::Finished(Ok(Ok(d))) => {
          if !self.apply(d) {
            self.success(tr(self.lang, "control_center.packages_refreshed"));
          }
          self.last_loaded = Some((self.reload_key(), std::time::Instant::now()));
          return true;
        }
        JobState::Finished(Ok(Err(e))) | JobState::Finished(Err(e)) => {
          self.error(e);
          return true;
        }
      }
    }
    self.poll_action()
  }
  /// Executes the `poll_action` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn poll_action(&mut self) -> bool {
    let Some(job) = self.action.take() else {
      return false;
    };
    match job.try_state() {
      JobState::Running => {
        self.action = Some(job);
        if self.transaction_open {
          if self.transaction_follow
            && let Some(live) = &self.transaction_live
          {
            self.transaction_scroll = transaction_bottom_offset(&live.output());
          }
          true
        } else {
          false
        }
      }
      JobState::Finished(Ok(Ok(_output))) => {
        self.action = None;
        self.success(tr(
          self.lang,
          "control_center.operation_completed_refreshing_packages",
        ));
        self.reload_force();
        true
      }
      JobState::Finished(Ok(Err(e))) | JobState::Finished(Err(e)) => {
        self.action = None;
        self.error(e);
        true
      }
    }
  }
  /// Applies the `apply` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn apply(&mut self, d: Loaded) -> bool {
    let mut reported_error = false;
    match d {
      Loaded::Packages(v) => {
        self.packages = v;
        if self.page == PackagesPage::Orphans {
          // Marks follow package names, so a reload keeps the ones that are
          // still orphans and drops the removed ones.
          let packages = &self.packages;
          self
            .marked
            .retain(|name| packages.iter().any(|package| &package.name == name));
        }
      }
      Loaded::Updates(v) => self.updates = v,
      Loaded::Cache(v) => self.cache = v,
      Loaded::History(v) => self.history = v,
      Loaded::Aur(v) => self.aur = v,
      Loaded::Mirrors(v) => self.mirrors = v,
      Loaded::Dashboard(mut v) => {
        if let Some(error) = v.error.take() {
          self.error(error);
          reported_error = true;
        }
        self.dashboard = v;
      }
      Loaded::Details(v) => self.details = Some(*v),
      Loaded::MirrorPreview(content) => {
        self.mirror_options = None;
        if self.page == PackagesPage::MirrorEditor {
          self.go(PackagesPage::Mirrors);
          let rows = self.rows();
          self.menu.select(&rows, &Item::ConfigureMirrors);
        }
        self.request(Action::ApplyMirrors(content));
      }
      Loaded::MirrorCountries(countries) => {
        self.mirror_countries = Some(countries.clone());
        if !countries.iter().any(|country| country == "Brazil")
          && let Some(options) = &mut self.mirror_options
          && options
            .countries
            .first()
            .is_some_and(|country| country == "Brazil")
          && let Some(first) = countries.first()
        {
          options.countries = vec![first.clone()];
        }
      }
    }
    self.normalize();
    reported_error
  }
  /// Executes the `error` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn error(&mut self, e: impl Into<String>) {
    self.status = Some(StatusMessage {
      kind: StatusKind::Error,
      text: terminal_text(&e.into()),
    });
  }
  /// Executes the `success` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn success(&mut self, e: impl Into<String>) {
    self.status = Some(StatusMessage {
      kind: StatusKind::Success,
      text: e.into(),
    });
  }
  /// Keeps the cursor on a selectable row after the rows changed.
  fn normalize(&mut self) {
    let rows = self.rows();
    self.menu.normalize(&rows);
  }
  /// Name of the package whose details page is open.
  fn details_target(&self) -> Option<String> {
    if !matches!(self.page, PackagesPage::Details(_)) {
      return None;
    }
    self
      .details
      .as_ref()
      .map(|details| details.package.name.clone())
      .or_else(|| self.details_name.clone())
  }
  /// Executes the `home_pages` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn home_pages(&self) -> Vec<PackagesPage> {
    let mut pages = vec![
      PackagesPage::Search,
      PackagesPage::Installed,
      PackagesPage::Orphans,
      PackagesPage::Updates,
      PackagesPage::Cache,
      PackagesPage::History,
      PackagesPage::Downgrade,
      PackagesPage::Mirrors,
    ];
    if self.capabilities.has_paru || self.capabilities.has_yay {
      pages.insert(1, PackagesPage::Aur);
    }
    pages
  }
  /// Executes the `matches_query` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn matches_query(&self, name: &str, description: &str) -> bool {
    let query = self.query.trim().to_ascii_lowercase();
    query.is_empty()
      || name.to_ascii_lowercase().contains(&query)
      || description.to_ascii_lowercase().contains(&query)
  }
  /// Packages matching the filter, with their index in `packages`.
  fn visible_packages(&self) -> Vec<(usize, &Package)> {
    self
      .packages
      .iter()
      .enumerate()
      .filter(|(_, package)| self.matches_query(&package.name, &package.description))
      .collect()
  }
  /// Updates matching the filter, with their index in `updates`.
  fn visible_updates(&self) -> Vec<(usize, &Update)> {
    self
      .updates
      .iter()
      .enumerate()
      .filter(|(_, update)| self.matches_query(&update.name, ""))
      .collect()
  }
  /// AUR results matching the filter, with their index in `aur`.
  fn visible_aur(&self) -> Vec<(usize, &AurPackage)> {
    self
      .aur
      .iter()
      .enumerate()
      .filter(|(_, package)| self.matches_query(&package.name, &package.description))
      .collect()
  }
  /// Whether typed characters currently go to text (the input prompt, or the
  /// filter of a package list), so `q`/`?` must not act as the global
  /// quit/help keys. Mirrors the precedence of [`Self::handle`]:
  /// confirmations, the transaction log and running operations sit above
  /// the list filter and do not take text.
  pub fn captures_text(&self) -> bool {
    if self.input.is_some() {
      return true;
    }
    self.pending.is_none()
      && !self.transaction_open
      && !self.destructive()
      && filters_while_typing(self.page)
  }

  /// Processes `handle` in this module's event flow. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn handle(&mut self, key: KeyCode) -> bool {
    if self.input.is_some() {
      return self.handle_input(key);
    }
    if self.pending.is_some() {
      match self.confirmation.handle(key) {
        ConfirmOutcome::Confirmed => {
          self.confirmation = ConfirmState::new();
          self.start_pending();
        }
        ConfirmOutcome::Cancelled => {
          self.pending = None;
          self.preview = None;
          self.confirmation = ConfirmState::new();
        }
        ConfirmOutcome::Pending => {}
      }
      return false;
    }
    if self.transaction_open {
      self.handle_transaction(key);
      return false;
    }
    // Tab only switches tabs or panes; Packages has none.
    if matches!(key, KeyCode::Tab | KeyCode::BackTab) {
      return false;
    }
    if filters_while_typing(self.page) {
      if self.handle_filter_key(key) {
        return false;
      }
    } else if key == KeyCode::Char('r') && self.page != PackagesPage::MirrorEditor {
      if !self.destructive() {
        self.reload_force();
      }
      return false;
    } else if key == KeyCode::Char(' ') && self.page == PackagesPage::MirrorEditor {
      // Space moved every reflector option forward, as before.
      let rows = self.rows();
      if let Some(item) = self.menu.selected_id(&rows) {
        self.adjust(item, 1);
      }
      return false;
    }
    let rows = self.rows();
    let page_size = usize::from(self.list_height.max(1));
    match self.menu.handle(key, &rows, page_size) {
      MenuEvent::Back => return self.back(),
      MenuEvent::Activate(item) | MenuEvent::Toggle(item) | MenuEvent::Confirm(item) => {
        self.activate(item)
      }
      MenuEvent::Adjust(item, step) => self.adjust(item, step),
      MenuEvent::Moved | MenuEvent::None => {}
    }
    false
  }
  /// Scrolls the live transaction output; Esc closes it.
  fn handle_transaction(&mut self, key: KeyCode) {
    match key {
      KeyCode::Esc => {
        self.transaction_open = false;
        self.transaction_live = None;
        self.transaction_scroll = 0;
        self.transaction_follow = false;
      }
      KeyCode::Up | KeyCode::Char('k') => {
        self.transaction_follow = false;
        self.transaction_scroll = self.transaction_scroll.saturating_sub(1);
      }
      KeyCode::Down | KeyCode::Char('j') => {
        self.transaction_follow = false;
        self.transaction_scroll = self.transaction_scroll.saturating_add(1);
      }
      KeyCode::PageUp => {
        self.transaction_follow = false;
        self.transaction_scroll = self.transaction_scroll.saturating_sub(10);
      }
      KeyCode::PageDown => {
        self.transaction_follow = false;
        self.transaction_scroll = self.transaction_scroll.saturating_add(10);
      }
      KeyCode::Home => {
        self.transaction_follow = false;
        self.transaction_scroll = 0;
      }
      KeyCode::End => {
        if let Some(live) = &self.transaction_live {
          self.transaction_scroll = transaction_bottom_offset(&live.output());
        }
      }
      _ => {}
    }
  }
  /// Keys of the lists that filter while typing (D8). Returns whether the
  /// key was used; arrows, PgUp/PgDn, Home/End, Enter and Esc go on to the
  /// menu.
  fn handle_filter_key(&mut self, key: KeyCode) -> bool {
    match key {
      KeyCode::Char(' ') if self.page == PackagesPage::Orphans => {
        if !self.destructive() {
          self.toggle_mark();
        }
      }
      KeyCode::Char('/') => {
        if !self.destructive() {
          self.input = Some(self.query.clone());
          self.input_mode = Some(InputMode::Search);
        }
      }
      KeyCode::Char(c) if !c.is_control() => {
        if !self.destructive() && self.query.len() < QUERY_MAX {
          self.query.push(c);
          self.cursor_to_first_entry();
        }
      }
      KeyCode::Backspace => {
        if !self.destructive() {
          self.query.pop();
          self.cursor_to_first_entry();
        }
      }
      // With nothing to open, Enter on the Search and AUR lists searches
      // for the typed text, as before.
      KeyCode::Enter
        if matches!(self.page, PackagesPage::Search | PackagesPage::Aur)
          && !self.rows().iter().any(Row::is_selectable) =>
      {
        if !self.destructive() {
          self.search_typed_query();
        }
      }
      _ => return false,
    }
    true
  }
  /// Runs the remote search for the typed filter on Search and AUR.
  fn search_typed_query(&mut self) {
    let minimum = if self.page == PackagesPage::Aur { 2 } else { 1 };
    if self.query.trim().chars().count() < minimum {
      self.error(tr(
        self.lang,
        if self.page == PackagesPage::Aur {
          "control_center.aur_search_requires_at_least_2_characters"
        } else {
          "control_center.enter_a_non_empty_search_query"
        },
      ));
    } else {
      self.reload();
    }
  }
  /// After the filter changed, puts the cursor on the first package, as the
  /// lists did before (or on the first selectable row without matches).
  fn cursor_to_first_entry(&mut self) {
    self.menu = MenuState::default();
    let rows = self.rows();
    if let Some(first) = rows
      .iter()
      .filter(|row| row.is_selectable())
      .find_map(|row| row.id().copied().filter(|item| item.is_list_entry()))
    {
      self.menu.select(&rows, &first);
    }
  }
  /// Space on an orphan marks or unmarks it for `Remove marked`.
  fn toggle_mark(&mut self) {
    let rows = self.rows();
    let Some(Item::Orphan(index)) = self.menu.selected_id(&rows) else {
      return;
    };
    let Some(name) = self.packages.get(index).map(|package| package.name.clone()) else {
      return;
    };
    if let Some(position) = self.marked.iter().position(|marked| *marked == name) {
      self.marked.remove(position);
    } else {
      self.marked.push(name);
    }
  }

  /// Opens `page` with the cursor on its first selectable row.
  fn go(&mut self, page: PackagesPage) {
    self.page = page;
    self.menu = MenuState::default();
  }

  /// Esc/`←`: one level up, with the cursor back on the row that opened the
  /// page; `true` leaves the Packages home.
  fn back(&mut self) -> bool {
    let (parent, origin) = match self.page {
      PackagesPage::Home => return true,
      PackagesPage::Details(index) => {
        let origin = match self.details_parent {
          PackagesPage::Aur => Some(Item::Aur(index)),
          PackagesPage::Orphans => Some(Item::Orphan(index)),
          PackagesPage::Search | PackagesPage::Installed => Some(Item::Package(index)),
          _ => None,
        };
        (self.details_parent, origin)
      }
      PackagesPage::HistoryDetails(_) => (PackagesPage::History, None),
      PackagesPage::CacheFiles => (PackagesPage::Cache, Some(Item::CacheFiles)),
      PackagesPage::MirrorEditor => {
        self.mirror_options = None;
        (PackagesPage::Mirrors, Some(Item::ConfigureMirrors))
      }
      page => (PackagesPage::Home, Some(Item::Open(page))),
    };
    self.go(parent);
    if let Some(origin) = origin {
      let rows = self.rows();
      self.menu.select(&rows, &origin);
    }
    false
  }

  /// Runs the row `item`.
  fn activate(&mut self, item: Item) {
    if item.waits_for_jobs() && self.busy() {
      return;
    }
    if !item.is_mirror_option() && self.destructive() {
      return;
    }
    match item {
      Item::Open(page) => {
        self.go(page);
        if matches!(page, PackagesPage::Search | PackagesPage::Installed) {
          self.query.clear();
          self.packages.clear();
        }
        self.reload();
      }
      Item::Package(index) | Item::Orphan(index) => {
        if let Some(name) = self.packages.get(index).map(|package| package.name.clone()) {
          self.details_parent = self.page;
          self.details = None;
          self.details_name = Some(name);
          self.go(PackagesPage::Details(index));
          self.reload();
        }
      }
      Item::Aur(index) => {
        if let Some(package) = self.aur.get(index) {
          self.details = Some(PackageDetails {
            package: Package {
              name: package.name.clone(),
              version: package.version.clone(),
              repository: Some("AUR".into()),
              description: package.description.clone(),
              installed: package.installed,
              ..Default::default()
            },
            ..Default::default()
          });
          self.details_name = Some(package.name.clone());
          self.details_parent = PackagesPage::Aur;
          self.go(PackagesPage::Details(index));
        }
      }
      Item::Update(index) => {
        if let Some(update) = self.updates.get(index) {
          let name = update.name.clone();
          self.request(Action::UpgradePackage(name));
        }
      }
      Item::CacheFile(index) => {
        if let Some(file) = self.cache.get(index) {
          let path = file.path.clone();
          self.request(Action::Downgrade(path));
        }
      }
      Item::Install => self.install_open_package(),
      Item::Remove => self.remove_open_package(),
      Item::UpgradeAll => self.begin_plan(Action::Upgrade),
      Item::RefreshDatabase => self.request(Action::RefreshDatabase),
      Item::RemoveMarked => {
        if !self.marked.is_empty() {
          self.begin_plan(Action::Remove(self.marked.clone()));
        }
      }
      Item::CleanKeepThree => self.request(Action::CleanCache("keep-three")),
      Item::CleanKeepOne => self.request(Action::CleanCache("keep-one")),
      Item::CleanUninstalled => self.request(Action::CleanCache("uninstalled")),
      Item::CacheFiles => self.go(PackagesPage::CacheFiles),
      Item::ConfigureMirrors => self.open_mirror_editor(),
      Item::MirrorAge => self.open_number_input(InputMode::MirrorAge),
      Item::MirrorCount => self.open_number_input(InputMode::MirrorCount),
      Item::MirrorCountry | Item::MirrorProtocol | Item::MirrorSort => self.adjust(item, 1),
      Item::MirrorPreview => {
        if let Some(options) = self.mirror_options.clone() {
          self.start_mirror_preview(options);
        }
      }
    }
  }
  /// `←/→` (and Space) on a reflector option: cycles the choices or moves
  /// the number one step, within the same bounds as before.
  fn adjust(&mut self, item: Item, step: i32) {
    let countries = self.mirror_countries.clone().unwrap_or_default();
    let Some(options) = self.mirror_options.as_mut() else {
      return;
    };
    match item {
      Item::MirrorCountry => cycle_country(options, &countries, step),
      Item::MirrorProtocol => cycle_protocol(options, step),
      Item::MirrorSort => cycle_sort(options, step),
      Item::MirrorAge => options.age_hours = step_within(options.age_hours, step, MIRROR_AGE_RANGE),
      Item::MirrorCount => options.count = step_within(options.count, step, MIRROR_COUNT_RANGE),
      _ => {}
    }
  }
  /// Asks for confirmation before running `action`.
  fn request(&mut self, action: Action) {
    self.confirmation = ConfirmState::new();
    self.preview = None;
    self.pending = Some(action);
  }
  /// Executes the `open_mirror_editor` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn open_mirror_editor(&mut self) {
    if self.capabilities.has_reflector {
      let default_country = self
        .mirror_countries
        .as_deref()
        .filter(|countries| countries.iter().any(|country| country == "Brazil"))
        .map(|_| "Brazil".into())
        .or_else(|| {
          self
            .mirror_countries
            .as_deref()
            .and_then(|c| c.first().cloned())
        })
        .unwrap_or_else(|| "Brazil".into());
      self.mirror_options = Some(ReflectorOptions {
        countries: vec![default_country],
        ..Default::default()
      });
      self.go(PackagesPage::MirrorEditor);
      if self.mirror_countries.is_none() && !self.busy() {
        self.spawn_country_list();
      }
    } else {
      self.error(tr(self.lang, "control_center.reflector_is_unavailable"));
    }
  }
  /// Executes the `spawn_country_list` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn spawn_country_list(&mut self) {
    let capabilities = self.capabilities.clone();
    self.job = Some(
      self
        .jobs
        .spawn(move |_| Ok(fetch_reflector_countries(capabilities).map(Loaded::MirrorCountries))),
    );
  }
  /// Executes the `start_mirror_preview` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn start_mirror_preview(&mut self, options: ReflectorOptions) {
    if self.busy() {
      return;
    }
    let caps = self.capabilities.clone();
    self.status = Some(StatusMessage {
      kind: StatusKind::Info,
      text: tr(self.lang, "control_center.generating_mirror_preview").into(),
    });
    self.job = Some(
      self
        .jobs
        .spawn(move |_| Ok(generate_mirror_preview(options, caps).map(Loaded::MirrorPreview))),
    );
  }
  /// Opens the numeric field of a reflector option.
  fn open_number_input(&mut self, mode: InputMode) {
    self.input = Some(String::new());
    self.input_mode = Some(mode);
  }
  /// Processes `handle_input` in this module's event flow. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn handle_input(&mut self, key: KeyCode) -> bool {
    let mode = self.input_mode.unwrap_or(InputMode::Search);
    let Some(input) = self.input.as_mut() else {
      return false;
    };
    match key {
      KeyCode::Char(c) if accepts_input_char(mode, input, c) => input.push(c),
      KeyCode::Backspace => {
        input.pop();
      }
      KeyCode::Esc => {
        self.input = None;
        self.input_mode = None;
        if mode == InputMode::Search {
          self.query.clear();
          self.packages.clear();
          self.aur.clear();
          self.menu = MenuState::default();
        }
      }
      KeyCode::Enter => {
        let candidate = self.input.take().unwrap_or_default();
        self.input_mode = None;
        match mode {
          InputMode::Search => self.apply_search_input(candidate),
          InputMode::MirrorAge | InputMode::MirrorCount => {
            self.apply_number_input(mode, &candidate)
          }
        }
      }
      _ => {}
    }
    false
  }
  /// Confirms the `/` search field.
  fn apply_search_input(&mut self, candidate: String) {
    let minimum = if self.page == PackagesPage::Aur { 2 } else { 1 };
    if candidate.trim().chars().count() < minimum {
      self.error(if self.page == PackagesPage::Aur {
        tr(
          self.lang,
          "control_center.aur_search_requires_at_least_2_characters",
        )
      } else {
        tr(self.lang, "control_center.enter_a_non_empty_search_query")
      });
      return;
    }
    self.query = candidate;
    self.menu = MenuState::default();
    self.reload()
  }
  /// Confirms the numeric field of a reflector option, within its bounds.
  fn apply_number_input(&mut self, mode: InputMode, candidate: &str) {
    let (min, max) = if mode == InputMode::MirrorAge {
      MIRROR_AGE_RANGE
    } else {
      MIRROR_COUNT_RANGE
    };
    match candidate.parse::<u32>() {
      Ok(value) if (min..=max).contains(&value) => {
        if let Some(options) = self.mirror_options.as_mut() {
          if mode == InputMode::MirrorAge {
            options.age_hours = value;
          } else {
            options.count = value;
          }
        }
      }
      _ => self.error(
        tr(self.lang, "control_center.value_must_be_between")
          .replace("{min}", &min.to_string())
          .replace("{max}", &max.to_string()),
      ),
    }
  }
  /// Executes the `start_pending` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn start_pending(&mut self) {
    let Some(a) = self.pending.take() else { return };
    let caps = self.capabilities.clone();
    self.preview = None;
    self.status = Some(StatusMessage {
      kind: StatusKind::Info,
      text: tr(self.lang, "control_center.running_operation").into(),
    });
    let live = LiveProcess::new();
    self.transaction_live = Some(live.clone());
    self.transaction_open = true;
    self.transaction_scroll = 0;
    self.transaction_follow = true;
    self.action = Some(self.jobs.spawn(move |_| Ok(run_action(a, caps, live))));
  }
  /// Executes the `begin_plan` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn begin_plan(&mut self, action: Action) {
    if self.busy() {
      return;
    }
    let caps = self.capabilities.clone();
    self.status = Some(StatusMessage {
      kind: StatusKind::Info,
      text: tr(self.lang, "control_center.planning_transaction").into(),
    });
    self.plan = Some(self.jobs.spawn(move |_| {
      let backend = PackageBackend::new(SystemProcessRunner, caps);
      let preview = match &action {
        Action::Install(packages) | Action::Remove(packages) => {
          if matches!(action, Action::Install(_)) {
            backend.plan_install(packages)
          } else {
            backend.plan_remove(packages)
          }
        }
        Action::Reinstall(package) => backend.plan_install(std::slice::from_ref(package)),
        Action::Upgrade | Action::UpgradePackage(_) => backend.plan_upgrade(),
        _ => Ok(TransactionPlan::default()),
      }
      .map_err(|error| error.to_string())?;
      Ok(Ok((action, preview)))
    }));
  }
  /// Whether the open package is installed. Before its metadata arrives,
  /// the flag of the list entry that opened it is used.
  fn open_package_installed(&self) -> bool {
    if let Some(details) = &self.details {
      return details.package.installed;
    }
    match (self.page, self.details_parent) {
      (PackagesPage::Details(index), PackagesPage::Aur) => {
        self.aur.get(index).is_some_and(|package| package.installed)
      }
      (PackagesPage::Details(index), _) => self
        .packages
        .get(index)
        .is_some_and(|package| package.installed),
      _ => false,
    }
  }
  /// `Install`/`Reinstall` on the package details page.
  fn install_open_package(&mut self) {
    let Some(name) = self.details_target() else {
      return;
    };
    if self.open_package_installed() {
      self.begin_plan(Action::Reinstall(name));
    } else if self.details_parent == PackagesPage::Aur {
      // The AUR helper has no transaction plan: the confirmation shows only
      // the PKGBUILD warning, without the empty 0/0/0 counters.
      self.request(Action::AurInstall(name));
    } else {
      self.begin_plan(Action::Install(vec![name]));
    }
  }
  /// `Remove` on the package details page; only installed packages.
  fn remove_open_package(&mut self) {
    if let Some(name) = self.details_target()
      && self.open_package_installed()
    {
      self.begin_plan(Action::Remove(vec![name]));
    }
  }

  /// A translated field label without the trailing colon some catalog
  /// entries carry, since Info rows draw label and value in columns.
  fn field(&self, key: &str) -> &'static str {
    tr(self.lang, key).trim_end_matches(':')
  }
  /// Rows of the current page.
  fn rows(&self) -> Vec<Row<Item>> {
    match self.page {
      PackagesPage::Home => self.home_rows(),
      PackagesPage::Search | PackagesPage::Installed => self.package_rows(),
      PackagesPage::Aur => self.aur_rows(),
      PackagesPage::Orphans => self.orphan_rows(),
      PackagesPage::Updates => self.update_rows(),
      PackagesPage::Details(_) => self.details_rows(),
      PackagesPage::Cache => self.cache_rows(),
      PackagesPage::CacheFiles => self.cache_file_rows(),
      PackagesPage::Downgrade => self.downgrade_rows(),
      PackagesPage::History | PackagesPage::HistoryDetails(_) => self.history_rows(),
      PackagesPage::Mirrors => self.mirror_rows(),
      PackagesPage::MirrorEditor => self.mirror_editor_rows(),
    }
  }
  /// The filter typed on the list, shown as the first (read-only) row.
  fn search_row(&self) -> Row<Item> {
    Row::info(
      self.field("control_center.search_2c43ee"),
      format!("{}_", self.query),
    )
  }
  /// Executes the `home_rows` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn home_rows(&self) -> Vec<Row<Item>> {
    let dashboard = &self.dashboard;
    let lang = self.lang;
    let packages = tr(lang, "control_center.packages_c945db");
    self
      .home_pages()
      .into_iter()
      .map(|page| {
        let (icon, label, detail) = match page {
          PackagesPage::Search => (
            icons::SEARCH,
            "control_center.install_official",
            format!("{} {packages}", dashboard.available_count),
          ),
          PackagesPage::Aur => (
            icons::ADD,
            "control_center.install_aur",
            dashboard.aur_helper.clone().unwrap_or_else(|| "—".into()),
          ),
          PackagesPage::Installed => (
            icons::INSTALLED,
            "control_center.installed_official",
            format!("{} {packages}", dashboard.installed_count),
          ),
          PackagesPage::Orphans => (
            icons::CLEAN,
            "control_center.installed_orphans",
            format!(
              "{} {}",
              dashboard.orphan_count,
              tr(lang, "control_center.orphans")
            ),
          ),
          PackagesPage::Updates => (
            icons::UPDATE,
            "control_center.updates_official",
            format!(
              "{} {}",
              dashboard.update_count,
              tr(lang, "control_center.pending")
            ),
          ),
          PackagesPage::Cache => (
            icons::DATABASE,
            "control_center.cache",
            format!(
              "{} {} · {}",
              dashboard.cache_count,
              tr(lang, "control_center.files_7093b3"),
              human_bytes(dashboard.cache_bytes)
            ),
          ),
          PackagesPage::History => (
            icons::HISTORY,
            "control_center.history",
            format!(
              "{} {}",
              dashboard.history_count,
              tr(lang, "control_center.entries")
            ),
          ),
          PackagesPage::Downgrade => (
            icons::DOWNGRADE,
            "control_center.downgrade",
            format!(
              "{} {}",
              dashboard.cache_count,
              tr(lang, "control_center.versions")
            ),
          ),
          _ => (
            icons::NETWORK,
            "control_center.mirrors",
            format!(
              "{}/{} {}",
              dashboard.mirrors_enabled,
              dashboard.mirrors_total,
              tr(lang, "control_center.active_ae7190")
            ),
          ),
        };
        Row::submenu(Item::Open(page), tr(lang, label))
          .icon(icon)
          .detail(detail)
      })
      .collect()
  }
  /// Search and Installed: each package opens its details page.
  fn package_rows(&self) -> Vec<Row<Item>> {
    let installed = tr(self.lang, "control_center.installed");
    let mut rows = vec![self.search_row()];
    rows.extend(self.visible_packages().into_iter().map(|(index, package)| {
      let mut parts = vec![package.version.clone()];
      if self.page == PackagesPage::Search {
        if let Some(repository) = &package.repository {
          parts.push(repository.clone());
        } else if package.foreign {
          parts.push("AUR".into());
        }
        if package.installed {
          parts.push(installed.into());
        }
      } else if package.foreign {
        parts.push("AUR".into());
      }
      Row::submenu(Item::Package(index), package.name.clone()).detail(joined_parts(&parts))
    }));
    rows
  }
  /// AUR search results; each opens its details page.
  fn aur_rows(&self) -> Vec<Row<Item>> {
    let installed = tr(self.lang, "control_center.installed");
    let mut rows = vec![self.search_row()];
    rows.extend(self.visible_aur().into_iter().map(|(index, package)| {
      let mut parts = vec![package.version.clone(), "AUR".into()];
      if package.installed {
        parts.push(installed.into());
      }
      Row::submenu(Item::Aur(index), package.name.clone()).detail(joined_parts(&parts))
    }));
    rows
  }
  /// Orphans: Space marks (`[x]`), Enter opens the details; the marked set
  /// is removed from the Danger zone.
  fn orphan_rows(&self) -> Vec<Row<Item>> {
    let mut rows = vec![self.search_row()];
    rows.extend(self.visible_packages().into_iter().map(|(index, package)| {
      let marked = self.marked.contains(&package.name);
      let row = Row::toggle(Item::Orphan(index), package.name.clone(), marked);
      if package.version.is_empty() {
        row
      } else {
        row.detail(package.version.clone())
      }
    }));
    if !self.packages.is_empty() {
      rows.push(Row::section(tr(self.lang, "control_center.danger_zone")));
      rows.push(
        Row::destructive(
          Item::RemoveMarked,
          format!(
            "{} ({})",
            tr(self.lang, "control_center.remove_marked"),
            self.marked.len()
          ),
        )
        .icon(icons::DELETE)
        .enabled(!self.marked.is_empty()),
      );
    }
    rows
  }
  /// Updates: the page actions, then one row per package (Enter updates
  /// it after the confirmation, as before).
  fn update_rows(&self) -> Vec<Row<Item>> {
    let visible = self.visible_updates();
    let mut rows = vec![
      self.search_row(),
      Row::section(tr(self.lang, "control_center.section_actions")),
      Row::action(
        Item::UpgradeAll,
        tr(self.lang, "control_center.upgrade_all"),
      )
      .icon(icons::UPDATE)
      .enabled(!visible.is_empty()),
      Row::action(
        Item::RefreshDatabase,
        tr(self.lang, "control_center.refresh_database"),
      )
      .icon(icons::SYNC),
    ];
    if !visible.is_empty() {
      rows.push(Row::section(tr(
        self.lang,
        "control_center.section_packages",
      )));
      rows.extend(visible.into_iter().map(|(index, update)| {
        Row::action(Item::Update(index), update.name.clone())
          .detail(format!("{} → {}", update.current, update.available))
      }));
    }
    rows
  }
  /// Details of one package: its actions first, the metadata in sections,
  /// and `Remove` in the Danger zone.
  fn details_rows(&self) -> Vec<Row<Item>> {
    let lang = self.lang;
    let installed = self.open_package_installed();
    let mut rows = vec![
      Row::section(tr(lang, "control_center.section_actions")),
      if installed {
        Row::action(Item::Install, tr(lang, "control_center.reinstall")).icon(icons::RESTART)
      } else {
        Row::action(Item::Install, tr(lang, "control_center.install")).icon(icons::ADD)
      },
    ];
    if let Some(d) = &self.details {
      let dash = |value: Option<&str>| value.unwrap_or("—").to_owned();
      let status = if d.package.installed {
        tr(lang, "control_center.installed")
      } else {
        tr(lang, "control_center.not_installed")
      };
      rows.extend([
        Row::section(tr(lang, "control_center.section_package")),
        Row::info(self.field("control_center.name"), d.package.name.clone()),
        Row::info(
          self.field("control_center.version_20bc85"),
          d.package.version.clone(),
        ),
        Row::info(
          self.field("control_center.repository"),
          dash(d.package.repository.as_deref()),
        ),
        Row::info(self.field("control_center.status"), status),
        Row::info(
          self.field("control_center.description"),
          d.package.description.clone(),
        ),
        Row::info(
          self.field("control_center.installed_size"),
          d.installed_size
            .map(human_bytes)
            .unwrap_or_else(|| "—".into()),
        ),
        Row::info(
          self.field("control_center.download"),
          d.download_size
            .map(human_bytes)
            .unwrap_or_else(|| "—".into()),
        ),
        Row::section(tr(lang, "control_center.section_source")),
        Row::info(
          self.field("control_center.architecture"),
          dash(d.architecture.as_deref()),
        ),
        Row::info(self.field("control_center.url"), dash(d.url.as_deref())),
        Row::info(
          self.field("control_center.licenses"),
          join_or_dash(&d.licenses),
        ),
        Row::info(self.field("control_center.groups"), join_or_dash(&d.groups)),
        Row::info(
          self.field("control_center.install_date"),
          dash(d.install_date.as_deref()),
        ),
        Row::section(tr(lang, "control_center.section_dependencies")),
        Row::info(
          self.field("control_center.depends_on"),
          join_or_dash(&d.dependencies),
        ),
        Row::info(
          self.field("control_center.optional"),
          join_or_dash(&d.optional_dependencies),
        ),
        Row::info(
          self.field("control_center.required_by"),
          join_or_dash(&d.required_by),
        ),
        Row::info(
          self.field("control_center.provides"),
          join_or_dash(&d.provides),
        ),
        Row::info(
          self.field("control_center.conflicts"),
          join_or_dash(&d.conflicts),
        ),
        Row::info(
          self.field("control_center.replaces"),
          join_or_dash(&d.replaces),
        ),
      ]);
    }
    rows.push(Row::section(tr(lang, "control_center.danger_zone")));
    rows.push(
      Row::destructive(Item::Remove, tr(lang, "control_center.remove"))
        .icon(icons::DELETE)
        .enabled(installed),
    );
    rows
  }
  /// Cache: a summary, the read-only file list in its own page, and the
  /// three cleanups in the Danger zone.
  fn cache_rows(&self) -> Vec<Row<Item>> {
    let lang = self.lang;
    let bytes = self.cache.iter().map(|file| file.bytes).sum();
    vec![
      Row::section(tr(lang, "control_center.section_summary")),
      Row::info(
        tr(lang, "control_center.cache_file_count"),
        self.cache.len().to_string(),
      ),
      Row::info(self.field("control_center.total_size"), human_bytes(bytes)),
      Row::submenu(Item::CacheFiles, tr(lang, "control_center.cache_files"))
        .icon(icons::FOLDER)
        .detail(format!(
          "{} {}",
          self.cache.len(),
          tr(lang, "control_center.files_7093b3")
        )),
      Row::section(tr(lang, "control_center.danger_zone")),
      Row::destructive(
        Item::CleanKeepThree,
        tr(lang, "control_center.clean_cache_keep_three"),
      )
      .icon(icons::CLEAN),
      Row::destructive(
        Item::CleanKeepOne,
        tr(lang, "control_center.clean_cache_keep_one"),
      )
      .icon(icons::CLEAN),
      Row::destructive(
        Item::CleanUninstalled,
        tr(lang, "control_center.clean_cache_uninstalled"),
      )
      .icon(icons::CLEAN),
    ]
  }
  /// Read-only list of the cached package files; the page only scrolls.
  fn cache_file_rows(&self) -> Vec<Row<Item>> {
    self
      .cache
      .iter()
      .map(|file| Row::info(cached_display_name(file), human_bytes(file.bytes)))
      .collect()
  }
  /// Downgrade: every cached file installs that version after the
  /// confirmation.
  fn downgrade_rows(&self) -> Vec<Row<Item>> {
    self
      .cache
      .iter()
      .enumerate()
      .map(|(index, file)| {
        Row::destructive(Item::CacheFile(index), cached_display_name(file))
          .detail(human_bytes(file.bytes))
      })
      .collect()
  }
  /// The pacman log, read-only; the page only scrolls.
  fn history_rows(&self) -> Vec<Row<Item>> {
    self
      .history
      .iter()
      .map(|entry| {
        let versions = match (&entry.old_version, &entry.new_version) {
          (Some(old), Some(new)) => format!("{old} → {new}"),
          _ => String::new(),
        };
        Row::info(
          format!("{}  {}  {}", entry.timestamp, entry.action, entry.package),
          versions,
        )
      })
      .collect()
  }
  /// Mirrors: `Configure mirrors` opens the reflector options; the current
  /// mirror list follows, read-only.
  fn mirror_rows(&self) -> Vec<Row<Item>> {
    let lang = self.lang;
    let configure = Row::submenu(
      Item::ConfigureMirrors,
      tr(lang, "control_center.configure_mirrors"),
    )
    .icon(icons::EDIT);
    let mut rows = vec![
      Row::section(tr(lang, "control_center.section_actions")),
      if self.capabilities.has_reflector {
        configure
      } else {
        configure
          .detail(tr(lang, "control_center.unavailable_no_reflector"))
          .enabled(false)
      },
    ];
    if !self.mirrors.is_empty() {
      rows.push(Row::section(tr(lang, "control_center.section_servers")));
      rows.extend(self.mirrors.iter().map(|mirror| {
        Row::info(
          mirror.server.clone(),
          tr(
            lang,
            if mirror.enabled {
              "control_center.mirror_enabled"
            } else {
              "control_center.mirror_disabled"
            },
          ),
        )
      }));
    }
    rows
  }
  /// Reflector options and `Generate preview`, which confirms before the
  /// mirror list is replaced. Not a draft: nothing is saved until then.
  fn mirror_editor_rows(&self) -> Vec<Row<Item>> {
    let lang = self.lang;
    let Some(options) = &self.mirror_options else {
      return Vec::new();
    };
    let country = options
      .countries
      .first()
      .cloned()
      .unwrap_or_else(|| tr(lang, "control_center.all").into());
    vec![
      Row::value(
        Item::MirrorCountry,
        tr(lang, "control_center.country"),
        country,
        Some(1),
      )
      .icon(icons::EARTH),
      Row::value(
        Item::MirrorProtocol,
        tr(lang, "control_center.protocol"),
        options.protocols.join(","),
        Some(1),
      )
      .icon(icons::LINK),
      Row::value(
        Item::MirrorAge,
        tr(lang, "control_center.maximum_age"),
        format!("{} h", options.age_hours),
        Some(1),
      )
      .icon(icons::CLOCK),
      Row::value(
        Item::MirrorCount,
        tr(lang, "control_center.count"),
        options.count.to_string(),
        Some(1),
      )
      .icon(icons::COUNTER),
      Row::value(
        Item::MirrorSort,
        tr(lang, "control_center.sort"),
        options.sort.clone(),
        Some(1),
      )
      .icon(icons::SORT),
      Row::separator(),
      Row::action(
        Item::MirrorPreview,
        tr(lang, "control_center.generate_preview"),
      )
      .icon(icons::VISIBLE)
      .emphasis(argvus_tui::menu::Emphasis::Primary),
    ]
  }
  /// The footer derived from the selected row.
  fn footer_hints(&self, rows: &[Row<Item>]) -> String {
    if self.pending.is_some() {
      return confirm_hints(self.lang);
    }
    let mut menu = self.menu;
    menu.normalize(rows);
    let typing = filters_while_typing(self.page);
    // Enter opens an orphan's details; Space marks it.
    let row = match menu.selected_kind(rows) {
      Some(RowKind::Toggle { .. }) if self.page == PackagesPage::Orphans => Some(RowKind::Submenu),
      kind => kind,
    };
    let mark = [("Space", tr(self.lang, "control_center.mark"))];
    hints(
      self.lang,
      &HintContext {
        row,
        can_go_back: true,
        search: typing,
        refresh: !typing && self.page != PackagesPage::MirrorEditor,
        extra: if self.page == PackagesPage::Orphans {
          &mark
        } else {
          &[]
        },
        ..HintContext::default()
      },
    )
  }
  /// Renders `draw` while respecting the current domain state and semantic theme. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn draw(&mut self, f: &mut Frame) {
    let area = f.area();
    let rows = self.rows();
    self.menu.normalize(&rows);
    let body = shell(
      f,
      area,
      &self.theme,
      self.lang,
      &self.breadcrumb(),
      &self.footer_hints(&rows),
    );
    self.list_height = body.height;
    draw_menu(
      f,
      body,
      &self.theme,
      &rows,
      &mut self.menu,
      MenuStyle {
        icons: AppConfig::icons_enabled(),
      },
    );
    self.overlays(f);
  }
  /// Draws the input field, the confirmation, the status and the live
  /// transaction output over the page.
  fn overlays(&self, f: &mut Frame) {
    if let Some(input) = &self.input {
      let mode = self.input_mode.unwrap_or(InputMode::Search);
      let (title, prompt) = match mode {
        InputMode::Search => (
          tr(self.lang, "control_center.search_2c43ee"),
          tr(self.lang, "control_center.search_packages"),
        ),
        InputMode::MirrorAge => (
          tr(self.lang, "control_center.maximum_age"),
          tr(self.lang, "control_center.input_5ca63b"),
        ),
        InputMode::MirrorCount => (
          tr(self.lang, "control_center.count"),
          tr(self.lang, "control_center.input_5ca63b"),
        ),
      };
      let popup = chrome::centered(f.area(), 50, 7);
      f.render_widget(Clear, popup);
      f.render_widget(
        Paragraph::new(vec![
          Line::from(prompt),
          Line::from(format!("{input}_")),
          Line::from(tr(self.lang, "control_center.enter_apply_esc_cancel")),
        ])
        .block(Block::bordered().title(title)),
        popup,
      );
    }
    if let Some(a) = &self.pending {
      let message = format!(
        "{}{}",
        action_message(self.lang, a),
        self
          .preview
          .as_ref()
          .map(|plan| preview_message(self.lang, plan))
          .unwrap_or_default()
      );
      draw_confirm(
        f,
        f.area(),
        &self.theme,
        ConfirmDialog {
          title: tr(self.lang, "control_center.confirm_transaction"),
          message: &message,
          confirm: tr(self.lang, "control_center.apply"),
          cancel: tr(self.lang, "control_center.cancel"),
          danger: a.is_destructive(),
          deadline: None,
        },
        &self.confirmation,
      );
    }
    if let Some(s) = &self.status {
      status(f, f.area(), &self.theme, s)
    }
    if self.transaction_open
      && let Some(output) = self.transaction_live.as_ref().map(|live| live.output())
    {
      let popup = chrome::centered(f.area(), TRANSACTION_POPUP_WIDTH, TRANSACTION_POPUP_HEIGHT);
      let total = transaction_wrapped_lines(&output);
      let shown_total = total.max(1);
      let current = (self.transaction_scroll as usize + 1).min(shown_total);
      f.render_widget(Clear, popup);
      f.render_widget(
        Paragraph::new(output.as_str())
          .block(
            Block::new()
              .borders(Borders::ALL)
              .title(Line::styled(
                format!(
                  " {} · {} {} {} {} ",
                  tr(self.lang, "control_center.transaction_output"),
                  tr(self.lang, "control_center.line"),
                  current,
                  tr(self.lang, "control_center.of"),
                  shown_total,
                ),
                Style::new().fg(self.theme.accent),
              ))
              .border_style(Style::new().fg(self.theme.border_active))
              .style(Style::new().bg(self.theme.background))
              .title_bottom(Line::from(tr(
                self.lang,
                "control_center.scroll_pgup_pgdn_home_end_esc_close",
              ))),
          )
          .scroll((self.transaction_scroll, 0))
          .wrap(Wrap { trim: false }),
        popup,
      );
    }
  }
  /// Executes the `breadcrumb` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn breadcrumb(&self) -> String {
    let root = format!(
      "{} > {}",
      HomeCardId::System.label(self.lang),
      tr(self.lang, "control_center.packages")
    );
    if self.page == PackagesPage::Home {
      root
    } else {
      format!("{root} > {}", self.page_label())
    }
  }
  /// Executes the `page_label` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn page_label(&self) -> String {
    let label = |key| tr(self.lang, key).to_owned();
    match self.page {
      PackagesPage::Search => label("control_center.search_2c43ee"),
      PackagesPage::Details(_) => format!(
        "{} > {}",
        match self.details_parent {
          PackagesPage::Installed => label("control_center.installed_e91b6d"),
          PackagesPage::Orphans => label("control_center.orphans_29aae8"),
          PackagesPage::Aur => label("control_center.aur"),
          _ => label("control_center.search_2c43ee"),
        },
        self.details_target().unwrap_or_default()
      ),
      PackagesPage::Installed => label("control_center.installed_e91b6d"),
      PackagesPage::Updates => label("control_center.updates"),
      PackagesPage::Orphans => label("control_center.orphans_29aae8"),
      PackagesPage::Cache => label("control_center.cache"),
      PackagesPage::CacheFiles => format!(
        "{} > {}",
        label("control_center.cache"),
        label("control_center.cache_files")
      ),
      PackagesPage::Aur => label("control_center.aur"),
      PackagesPage::History | PackagesPage::HistoryDetails(_) => label("control_center.history"),
      PackagesPage::Downgrade => label("control_center.downgrade"),
      PackagesPage::Mirrors => label("control_center.mirrors"),
      PackagesPage::MirrorEditor => format!(
        "{} > {}",
        label("control_center.mirrors"),
        label("control_center.configure_mirrors")
      ),
      PackagesPage::Home => label("control_center.packages"),
    }
  }
}
/// Whether the open field takes `character`: the search takes any text up
/// to its limit; the reflector numbers take digits only.
fn accepts_input_char(mode: InputMode, input: &str, character: char) -> bool {
  match mode {
    InputMode::Search => !character.is_control() && input.len() < QUERY_MAX,
    InputMode::MirrorAge => character.is_ascii_digit() && input.len() < 4,
    InputMode::MirrorCount => character.is_ascii_digit() && input.len() < 3,
  }
}
/// `value` moved by `step`, kept within `(min, max)`.
fn step_within(value: u32, step: i32, (min, max): (u32, u32)) -> u32 {
  value.saturating_add_signed(step).clamp(min, max)
}
/// Non-empty parts of a list row's detail, joined with ` · `.
fn joined_parts(parts: &[String]) -> String {
  parts
    .iter()
    .filter(|part| !part.is_empty())
    .cloned()
    .collect::<Vec<_>>()
    .join(" · ")
}
/// Executes the `join_or_dash` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn join_or_dash(values: &[String]) -> String {
  if values.is_empty() {
    "—".into()
  } else {
    values.join(" ")
  }
}
/// Executes the `cached_display_name` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn cached_display_name(package: &CachePackage) -> String {
  if !package.version.is_empty() {
    return format!("{} {}", package.name, package.version);
  }
  /// Defines the constant `SUFFIXES`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
  const SUFFIXES: &[&str] = &[
    ".pkg.tar.zst",
    ".pkg.tar.xz",
    ".pkg.tar.gz",
    ".pkg.tar.lrz",
    ".pkg.tar.lz4",
    ".pkg.tar.lzo",
    ".pkg.tar",
  ];
  let mut name = package.name.as_str();
  for suffix in SUFFIXES {
    if let Some(stripped) = name.strip_suffix(suffix) {
      name = stripped;
      break;
    }
  }
  name.to_owned()
}
/// Executes the `action_message` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn action_message(lang: Lang, a: &Action) -> String {
  match a {
    Action::Install(v) => format!("{}: {}", tr(lang, "control_center.install"), v.join(", ")),
    Action::Remove(v) => format!("{}: {}", tr(lang, "control_center.remove"), v.join(", ")),
    Action::Reinstall(v) => format!("{}: {v}", tr(lang, "control_center.reinstall")),
    Action::Upgrade => tr(lang, "control_center.perform_a_full_system_upgrade").into(),
    Action::UpgradePackage(v) => format!("{}: {v}", tr(lang, "control_center.update_package")),
    Action::RefreshDatabase => tr(lang, "control_center.refresh_package_database").into(),
    Action::CleanCache(v) => format!("{} ({v})?", tr(lang, "control_center.clean_cache")),
    Action::Downgrade(v) => format!("{}: {v}", tr(lang, "control_center.downgrade_a3e71b")),
    Action::AurInstall(v) => format!(
      "{} AUR: {v}? {}",
      tr(lang, "control_center.install"),
      tr(
        lang,
        "control_center.pkgbuilds_execute_build_instructions_as_the_normal_user"
      )
    ),
    Action::ApplyMirrors(content) => format!(
      "{}\n{}",
      tr(lang, "control_center.apply_these_mirrors"),
      mirror_preview_summary(content)
    ),
  }
}
/// The transaction plan counters shown under the confirmation message, with
/// the download size in readable units.
fn preview_message(lang: Lang, plan: &TransactionPlan) -> String {
  format!(
    "\n\n{}: {}\n{}: {}\n{}: {}",
    tr(lang, "control_center.install"),
    plan.install.len(),
    tr(lang, "control_center.remove"),
    plan.remove.len(),
    tr(lang, "control_center.download").trim_end_matches(':'),
    human_bytes(plan.download_bytes)
  )
}
/// What a package operation runs, built before anything starts so the
/// exact command can be checked without running it.
#[derive(Debug, Clone)]
enum ActionCommand {
  /// `<executable> system-settings package <action> <arguments>`, which
  /// elevates itself with `pkexec`.
  Privileged {
    /// Line echoed at the top of the live output.
    echo: String,
    request: PrivilegedRequest,
  },
  /// The AUR helper, run as the normal user (PKGBUILDs must not build as
  /// root).
  User {
    echo: String,
    process: ProcessRequest,
  },
}

/// Builds the command of `a` without running it.
fn action_command(a: Action, caps: &Capabilities) -> Result<ActionCommand, String> {
  let privileged = |echo: String, name: &str, args: Vec<String>| {
    Ok(ActionCommand::Privileged {
      echo,
      request: PrivilegedRequest::new("package", name, args)?,
    })
  };
  match a {
    Action::Install(v) => privileged(
      format!("$ pacman -S --needed {}", v.join(" ")),
      "install",
      v,
    ),
    Action::Remove(v) => privileged(format!("$ pacman -Rns {}", v.join(" ")), "remove", v),
    Action::Reinstall(v) => privileged(format!("$ pacman -S {v}"), "reinstall", vec![v]),
    Action::Upgrade => privileged("$ pacman -Syu".into(), "upgrade", vec![]),
    Action::UpgradePackage(v) => privileged(format!("$ pacman -S {v}"), "upgrade-package", vec![v]),
    Action::RefreshDatabase => privileged("$ pacman -Syy".into(), "refresh-db", vec![]),
    Action::CleanCache(v) => privileged("$ paccache -r".into(), "clean-cache", vec![v.into()]),
    Action::Downgrade(v) => privileged(format!("$ pacman -U {v}"), "downgrade", vec![v]),
    Action::ApplyMirrors(content) => {
      privileged("$ update mirrorlist".into(), "mirror-apply", vec![content])
    }
    Action::AurInstall(name) => {
      let helper = if caps.has_paru {
        "paru"
      } else if caps.has_yay {
        "yay"
      } else {
        return Err("AUR helper is unavailable".into());
      };
      Ok(ActionCommand::User {
        echo: format!("$ {helper} -S {name}"),
        process: ProcessRequest::new(helper).arg("-S").arg(name),
      })
    }
  }
}

/// Runs the command of `a`, streaming its output into `live`.
fn run_action(a: Action, caps: Capabilities, live: LiveProcess) -> Result<String, String> {
  match action_command(a, &caps)? {
    ActionCommand::Privileged { echo, request } => {
      live.push_line(&echo);
      let executable = std::env::current_exe()
        .map_err(|error| error.to_string())?
        .to_string_lossy()
        .into_owned();
      let o = SystemSettingsOperation::new(SystemProcessRunner, executable)
        .execute_live(&request, &live)?;
      if o.status == Some(0) {
        let stdout = terminal_text(&String::from_utf8_lossy(&o.stdout))
          .trim()
          .to_owned();
        Ok(if stdout.is_empty() {
          "Operation completed.".into()
        } else {
          stdout
        })
      } else {
        Err(terminal_text(&String::from_utf8_lossy(&o.stderr)))
      }
    }
    ActionCommand::User { echo, process } => {
      live.push_line(&echo);
      let o = SystemProcessRunner
        .run_live(&process, &live)
        .map_err(|e| e.to_string())?;
      if o.status == Some(0) {
        Ok(terminal_text(&String::from_utf8_lossy(&o.stdout)))
      } else {
        Err(terminal_text(&String::from_utf8_lossy(&o.stderr)))
      }
    }
  }
}

/// Executes the `generate_mirror_preview` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn generate_mirror_preview(
  options: ReflectorOptions,
  caps: Capabilities,
) -> Result<String, String> {
  if !caps.has_reflector {
    return Err("reflector is unavailable".into());
  }
  let args = reflector_args(&options).map_err(|error| error.to_string())?;
  let request = args
    .iter()
    .fold(ProcessRequest::new("reflector"), |request, argument| {
      request.arg(argument)
    });
  let output = SystemProcessRunner
    .run(&request)
    .map_err(|error| error.to_string())?;
  if output.status != Some(0) {
    return Err(terminal_text(&String::from_utf8_lossy(&output.stderr)));
  }
  let content = terminal_text(&String::from_utf8_lossy(&output.stdout));
  if !content
    .lines()
    .any(|line| line.trim_start().starts_with("Server = "))
  {
    return Err("reflector generated no valid mirrors".into());
  }
  Ok(content)
}

/// Retrieves data for `fetch_reflector_countries` without mixing collection with TUI rendering. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn fetch_reflector_countries(caps: Capabilities) -> Result<Vec<String>, String> {
  if !caps.has_reflector {
    return Err("reflector is unavailable".into());
  }
  let request = ProcessRequest::new("reflector").arg("--list-countries");
  let output = SystemProcessRunner
    .run(&request)
    .map_err(|error| error.to_string())?;
  if output.status != Some(0) {
    return Err(terminal_text(&String::from_utf8_lossy(&output.stderr)));
  }
  Ok(parse_reflector_countries(&String::from_utf8_lossy(
    &output.stdout,
  )))
}

/// Executes the `mirror_preview_summary` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn mirror_preview_summary(content: &str) -> String {
  let mirrors = content
    .lines()
    .filter(|line| line.trim_start().starts_with("Server = "))
    .collect::<Vec<_>>();
  let mut summary = format!("{} mirror(s)", mirrors.len());
  for mirror in mirrors.into_iter().take(3) {
    summary.push('\n');
    summary.push_str(mirror);
  }
  summary
}

/// Moves `index` by `step` within `len` choices, wrapping around.
fn cycled(index: usize, step: i32, len: usize) -> usize {
  (index as i64 + i64::from(step)).rem_euclid(len.max(1) as i64) as usize
}

/// Moves the reflector country by `step`: forward with `→`/Space, back
/// with `←`. Until reflector lists its countries, a short fallback list is
/// used; `None` means every country.
fn cycle_country(options: &mut ReflectorOptions, countries: &[String], step: i32) {
  /// Defines the constant `FALLBACK`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
  const FALLBACK: &[Option<&str>] = &[
    None,
    Some("Brazil"),
    Some("United States"),
    Some("Germany"),
    Some("France"),
  ];
  let current = options.countries.first().map(String::as_str);
  let mut names: Vec<Option<&str>> = vec![None];
  if countries.is_empty() {
    names = FALLBACK.to_vec();
  } else {
    let mut seen = std::collections::HashSet::new();
    for country in countries {
      if seen.insert(country.as_str()) {
        names.push(Some(country.as_str()));
      }
    }
  }
  let index = names
    .iter()
    .position(|country| *country == current)
    .unwrap_or(0);
  options.countries = names[cycled(index, step, names.len())]
    .map(|country| vec![country.into()])
    .unwrap_or_default();
}

/// Moves the reflector protocol by `step`.
fn cycle_protocol(options: &mut ReflectorOptions, step: i32) {
  /// Defines the constant `PROTOCOLS`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
  const PROTOCOLS: &[&str] = &["https", "http", "rsync"];
  let current = options
    .protocols
    .first()
    .map(String::as_str)
    .unwrap_or("https");
  let index = PROTOCOLS
    .iter()
    .position(|protocol| *protocol == current)
    .unwrap_or(0);
  options.protocols = vec![PROTOCOLS[cycled(index, step, PROTOCOLS.len())].into()];
}

/// Moves the reflector sort order by `step`.
fn cycle_sort(options: &mut ReflectorOptions, step: i32) {
  /// Defines the constant `SORTS`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
  const SORTS: &[&str] = &["rate", "age", "score", "delay", "country"];
  let index = SORTS
    .iter()
    .position(|sort| *sort == options.sort)
    .unwrap_or(0);
  options.sort = SORTS[cycled(index, step, SORTS.len())].into();
}

#[cfg(test)]
mod tests {
  use super::*;
  use ratatui::{Terminal, backend::TestBackend};

  fn app() -> PackagesApp {
    PackagesApp::new(
      Lang::for_locale("en-US"),
      Theme::load(),
      Capabilities::default(),
    )
  }

  fn screen(app: &mut PackagesApp, width: u16, height: u16) -> String {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal.draw(|frame| app.draw(frame)).unwrap();
    let buffer = terminal.backend().buffer();
    (0..height)
      .map(|y| {
        (0..width)
          .map(|x| buffer[(x, y)].symbol())
          .collect::<String>()
      })
      .collect::<Vec<_>>()
      .join("\n")
  }

  fn row(rows: &[Row<Item>], item: Item) -> &Row<Item> {
    rows
      .iter()
      .find(|row| row.id() == Some(&item))
      .unwrap_or_else(|| panic!("no row {item:?}"))
  }

  fn package(name: &str, installed: bool) -> Package {
    Package {
      name: name.into(),
      version: "1.0".into(),
      installed,
      ..Default::default()
    }
  }

  /// Moves the cursor to `item` and presses Enter.
  fn press(app: &mut PackagesApp, item: Item) {
    let rows = app.rows();
    assert!(app.menu.select(&rows, &item), "{item:?} is not selectable");
    app.handle(KeyCode::Enter);
  }

  fn orphans_app() -> PackagesApp {
    let mut app = app();
    app.page = PackagesPage::Orphans;
    app.packages = vec![
      package("libfoo", true),
      package("python-bar", true),
      package("libbaz", true),
    ];
    app
  }

  #[test]
  fn package_home_rows_are_submenus_with_their_counters() {
    let mut app = app();
    app.dashboard = PackageDashboard {
      installed_count: 1200,
      update_count: 7,
      orphan_count: 3,
      cache_count: 12,
      cache_bytes: 240_057_409_536,
      history_count: 40,
      mirrors_total: 6,
      mirrors_enabled: 4,
      available_count: 18_500,
      aur_helper: None,
      ..Default::default()
    };
    let rows = app.home_rows();
    assert_eq!(rows.len(), 8);
    assert!(rows.iter().all(|row| row.kind() == RowKind::Submenu));
    assert_eq!(
      rows.iter().map(Row::icon_glyph).collect::<Vec<_>>(),
      [
        icons::SEARCH,
        icons::INSTALLED,
        icons::CLEAN,
        icons::UPDATE,
        icons::DATABASE,
        icons::HISTORY,
        icons::DOWNGRADE,
        icons::NETWORK
      ]
      .map(Some)
    );
    assert_eq!(rows[0].detail_text(), Some("18500 packages"));
    assert_eq!(rows[1].label(), "Installed / Official");
    assert_eq!(rows[3].detail_text(), Some("7 pending"));
    assert_eq!(rows[4].detail_text(), Some("12 files · 223.6 GiB"));
    assert_eq!(rows[7].detail_text(), Some("4/6 active"));
  }

  #[test]
  fn aur_appears_on_the_home_only_with_a_helper() {
    let app = PackagesApp::new(
      Lang::for_locale("en-US"),
      Theme::load(),
      Capabilities {
        has_paru: true,
        ..Default::default()
      },
    );
    let rows = app.home_rows();
    assert_eq!(rows[1].id(), Some(&Item::Open(PackagesPage::Aur)));
    assert_eq!(rows[1].icon_glyph(), Some(icons::ADD));
  }

  #[test]
  fn package_home_is_navigable_and_back_returns_to_the_origin() {
    let mut app = app();
    app.handle(KeyCode::Down);
    app.handle(KeyCode::Enter);
    assert_eq!(app.page, PackagesPage::Installed);
    assert!(!app.handle(KeyCode::Esc));
    assert_eq!(app.page, PackagesPage::Home);
    assert_eq!(
      app.menu.selected_id(&app.rows()),
      Some(Item::Open(PackagesPage::Installed))
    );
    assert!(app.handle(KeyCode::Esc));

    let text = screen(&mut app, 90, 25);
    assert!(text.contains("ARGVUS"));
    assert!(text.contains("Packages"));
    assert!(text.contains("Enter"));
  }

  #[test]
  fn packages_pages_have_no_button_bar_and_tab_does_nothing() {
    let mut app = app();
    app.page = PackagesPage::Updates;
    app.updates.push(Update {
      name: "linux".into(),
      current: "1".into(),
      available: "2".into(),
      ..Default::default()
    });
    let before = app.menu;
    app.handle(KeyCode::Tab);
    app.handle(KeyCode::BackTab);
    assert_eq!(app.menu, before);
    let text = screen(&mut app, 90, 25);
    assert!(!text.contains("[ "), "{text}");
    assert!(text.contains("Upgrade all"));
    assert!(text.contains("linux"));
  }

  #[test]
  fn typing_filters_lists_and_keeps_letters_as_text() {
    let mut app = app();
    app.page = PackagesPage::Installed;
    app.packages = vec![package("firefox", true), package("jq", true)];
    for key in ['j', 'q'] {
      app.handle(KeyCode::Char(key));
    }
    assert_eq!(app.query, "jq");
    assert!(app.captures_text());
    let rows = app.rows();
    assert_eq!(rows.iter().filter(|row| row.is_selectable()).count(), 1);
    assert_eq!(app.menu.selected_id(&rows), Some(Item::Package(1)));
    app.handle(KeyCode::Backspace);
    app.handle(KeyCode::Char('r'));
    assert_eq!(app.query, "jr");
    assert!(app.job.is_none(), "r is text on the filtered lists");
  }

  #[test]
  fn empty_search_does_not_create_package_action() {
    let mut app = app();
    app.page = PackagesPage::Search;
    app.handle(KeyCode::Tab);
    app.handle(KeyCode::Enter);
    assert!(app.pending.is_none());
    app.handle(KeyCode::Right);
    app.handle(KeyCode::Enter);
    assert!(app.pending.is_none());
    assert!(app.plan.is_none());
  }

  #[test]
  fn enter_opens_the_details_and_back_returns_to_the_package() {
    let mut app = app();
    app.page = PackagesPage::Search;
    app.packages = vec![package("firefox", false), package("zsh", false)];
    app.handle(KeyCode::Down);
    app.handle(KeyCode::Enter);
    assert_eq!(app.page, PackagesPage::Details(1));
    assert_eq!(app.details_target().as_deref(), Some("zsh"));
    app.handle(KeyCode::Esc);
    assert_eq!(app.page, PackagesPage::Search);
    assert_eq!(app.menu.selected_id(&app.rows()), Some(Item::Package(1)));
  }

  #[test]
  fn search_pages_do_not_spawn_until_a_valid_query_is_confirmed() {
    let mut app = app();
    app.page = PackagesPage::Search;
    app.reload();
    assert!(app.job.is_none());
    app.page = PackagesPage::Aur;
    app.reload();
    assert!(app.job.is_none());
    app.handle(KeyCode::Char('/'));
    app.handle(KeyCode::Char('a'));
    app.handle(KeyCode::Enter);
    assert!(app.job.is_none());
    assert!(app.status.as_ref().unwrap().text.contains("at least 2"));
    app.handle(KeyCode::Char('a'));
    app.handle(KeyCode::Enter);
    assert!(app.job.is_none());
    assert!(app.status.as_ref().unwrap().text.contains("at least 2"));
  }

  #[test]
  fn cancelling_package_search_clears_the_query() {
    let mut app = app();
    app.page = PackagesPage::Search;
    app.query = "firefox".into();
    app.input = Some(app.query.clone());
    app.input_mode = Some(InputMode::Search);
    app.handle(KeyCode::Esc);
    assert!(app.query.is_empty());
    assert!(app.input.is_none());
  }

  #[test]
  fn opening_package_details_keeps_the_selected_name_until_metadata_arrives() {
    let mut app = app();
    app.page = PackagesPage::Search;
    app.packages.push(package("firefox", false));
    press(&mut app, Item::Package(0));
    assert!(app.details.is_none());
    assert_eq!(app.details_target().as_deref(), Some("firefox"));
    assert_eq!(app.reload_key(), "Details(0)||firefox");
  }

  #[test]
  fn package_details_render_actions_sections_and_danger_zone() {
    let mut app = app();
    app.page = PackagesPage::Details(0);
    app.details = Some(PackageDetails {
      package: Package {
        name: "firefox".into(),
        version: "155.0.1-1".into(),
        repository: Some("extra".into()),
        installed: true,
        ..Default::default()
      },
      architecture: Some("x86_64".into()),
      url: Some("https://www.mozilla.org".into()),
      installed_size: Some(87_654_321),
      dependencies: vec!["gtk4".into(), "libx11".into()],
      ..Default::default()
    });
    let rows = app.rows();
    let titles = rows
      .iter()
      .filter(|row| row.is_section())
      .map(Row::label)
      .collect::<Vec<_>>();
    assert_eq!(
      titles,
      [
        "control_center.section_actions",
        "control_center.section_package",
        "control_center.section_source",
        "control_center.section_dependencies",
        "control_center.danger_zone",
      ]
      .map(|key| tr(app.lang, key))
    );
    assert_eq!(row(&rows, Item::Install).label(), "Reinstall");
    assert_eq!(row(&rows, Item::Install).icon_glyph(), Some(icons::RESTART));
    let remove = rows.last().unwrap();
    assert_eq!(remove.kind(), RowKind::Destructive);
    assert!(remove.is_selectable());
    let info = |label: &str| {
      rows
        .iter()
        .find(|row| row.kind() == RowKind::Info && row.label() == label)
        .and_then(Row::detail_text)
        .map(str::to_owned)
    };
    assert_eq!(info("Status").as_deref(), Some("Installed"));
    assert_eq!(info("Architecture").as_deref(), Some("x86_64"));
    assert_eq!(info("Depends on").as_deref(), Some("gtk4 libx11"));
    assert!(rows.iter().all(|row| !row.label().contains("Tab")));
  }

  #[test]
  fn install_and_remove_follow_the_open_package() {
    let mut app = app();
    app.page = PackagesPage::Search;
    app.packages.push(package("firefox", false));
    press(&mut app, Item::Package(0));
    let rows = app.rows();
    assert_eq!(row(&rows, Item::Install).label(), "Install");
    assert!(
      !row(&rows, Item::Remove).is_selectable(),
      "an uninstalled package cannot be removed"
    );
    // The details load runs; package operations wait for it, as before.
    press(&mut app, Item::Install);
    assert!(app.plan.is_none());
    app.job = None;
    press(&mut app, Item::Install);
    assert!(app.plan.is_some(), "installing plans the transaction first");

    let mut installed = PackagesApp::new(
      Lang::for_locale("en-US"),
      Theme::load(),
      Capabilities::default(),
    );
    installed.page = PackagesPage::Installed;
    installed.packages.push(package("zsh", true));
    press(&mut installed, Item::Package(0));
    installed.job = None;
    let rows = installed.rows();
    assert_eq!(row(&rows, Item::Install).label(), "Reinstall");
    press(&mut installed, Item::Remove);
    assert!(installed.plan.is_some());
  }

  #[test]
  fn aur_rows_open_the_filtered_package() {
    let mut app = app();
    app.page = PackagesPage::Aur;
    app.aur = vec![
      AurPackage {
        name: "yay-bin".into(),
        ..Default::default()
      },
      AurPackage {
        name: "paru-bin".into(),
        installed: true,
        ..Default::default()
      },
    ];
    for key in "paru".chars() {
      app.handle(KeyCode::Char(key));
    }
    app.handle(KeyCode::Enter);
    assert_eq!(app.page, PackagesPage::Details(1));
    assert_eq!(app.details_target().as_deref(), Some("paru-bin"));
    assert_eq!(app.details_parent, PackagesPage::Aur);
  }

  #[test]
  fn space_marks_orphans_and_the_marks_follow_the_filter() {
    let mut app = orphans_app();
    let rows = app.rows();
    assert!(!row(&rows, Item::RemoveMarked).is_selectable());
    for key in "lib".chars() {
      app.handle(KeyCode::Char(key));
    }
    app.handle(KeyCode::Down);
    app.handle(KeyCode::Char(' '));
    assert_eq!(app.marked, ["libbaz"]);
    assert_eq!(app.query, "lib", "Space marks instead of typing");
    let rows = app.rows();
    assert_eq!(
      row(&rows, Item::Orphan(2)).kind(),
      RowKind::Toggle { on: true }
    );
    assert_eq!(
      row(&rows, Item::RemoveMarked).label(),
      format!("{} (1)", tr(app.lang, "control_center.remove_marked"))
    );
    let text = screen(&mut app, 90, 25);
    assert!(text.contains("[x]"), "{text}");
    let mark = format!("Space {}", tr(app.lang, "control_center.mark"));
    assert!(text.contains(&mark), "{text}");
    press(&mut app, Item::RemoveMarked);
    assert!(app.plan.is_some(), "the marked set is planned for removal");
  }

  #[test]
  fn enter_on_an_orphan_opens_its_details() {
    let mut app = orphans_app();
    app.handle(KeyCode::Enter);
    assert_eq!(app.page, PackagesPage::Details(0));
    assert!(app.marked.is_empty());
    app.handle(KeyCode::Esc);
    assert_eq!(app.menu.selected_id(&app.rows()), Some(Item::Orphan(0)));
  }

  #[test]
  fn reloading_orphans_keeps_only_the_marks_still_listed() {
    let mut app = orphans_app();
    app.marked = vec!["libfoo".into(), "gone".into()];
    app.apply(Loaded::Packages(vec![package("libfoo", true)]));
    assert_eq!(app.marked, ["libfoo"]);
  }

  #[test]
  fn update_rows_confirm_one_package_or_plan_all() {
    let mut app = app();
    app.page = PackagesPage::Updates;
    app.updates.push(Update {
      name: "linux".into(),
      current: "1".into(),
      available: "2".into(),
      ..Default::default()
    });
    press(&mut app, Item::Update(0));
    assert_eq!(app.pending, Some(Action::UpgradePackage("linux".into())));
    app.handle(KeyCode::Esc);
    assert!(app.pending.is_none());
    press(&mut app, Item::RefreshDatabase);
    assert_eq!(app.pending, Some(Action::RefreshDatabase));
    app.handle(KeyCode::Esc);
    press(&mut app, Item::UpgradeAll);
    assert!(app.plan.is_some());
  }

  #[test]
  fn upgrade_all_is_disabled_without_updates() {
    let mut app = app();
    app.page = PackagesPage::Updates;
    let rows = app.rows();
    assert!(!row(&rows, Item::UpgradeAll).is_selectable());
    assert!(row(&rows, Item::RefreshDatabase).is_selectable());
  }

  #[test]
  fn cache_page_summarizes_and_cleans_from_the_danger_zone() {
    let mut app = app();
    app.page = PackagesPage::Cache;
    app.cache = vec![CachePackage {
      name: "firefox-155.0.1-1-x86_64.pkg.tar.zst".into(),
      bytes: 2_621_440,
      ..Default::default()
    }];
    let rows = app.rows();
    assert_eq!(rows.last().unwrap().id(), Some(&Item::CleanUninstalled));
    for (item, policy) in [
      (Item::CleanKeepThree, "keep-three"),
      (Item::CleanKeepOne, "keep-one"),
      (Item::CleanUninstalled, "uninstalled"),
    ] {
      assert_eq!(row(&rows, item).kind(), RowKind::Destructive);
      press(&mut app, item);
      assert_eq!(app.pending, Some(Action::CleanCache(policy)));
      app.handle(KeyCode::Esc);
    }
    press(&mut app, Item::CacheFiles);
    assert_eq!(app.page, PackagesPage::CacheFiles);
    let files = app.rows();
    assert_eq!(files[0].label(), "firefox-155.0.1-1-x86_64");
    assert_eq!(files[0].detail_text(), Some("2 MiB"));
    assert!(files.iter().all(|row| !row.is_selectable()));
    app.handle(KeyCode::Esc);
    assert_eq!(app.page, PackagesPage::Cache);
    assert_eq!(app.menu.selected_id(&app.rows()), Some(Item::CacheFiles));
  }

  #[test]
  fn downgrade_rows_confirm_the_cached_file() {
    let mut app = app();
    app.page = PackagesPage::Downgrade;
    app.cache = vec![CachePackage {
      name: "zsh".into(),
      version: "5.9-1".into(),
      path: "/var/cache/pacman/pkg/zsh-5.9-1-x86_64.pkg.tar.zst".into(),
      ..Default::default()
    }];
    let rows = app.rows();
    assert_eq!(rows[0].kind(), RowKind::Destructive);
    assert_eq!(rows[0].label(), "zsh 5.9-1");
    app.handle(KeyCode::Enter);
    assert_eq!(
      app.pending,
      Some(Action::Downgrade(
        "/var/cache/pacman/pkg/zsh-5.9-1-x86_64.pkg.tar.zst".into()
      ))
    );
  }

  #[test]
  fn history_is_read_only_and_has_no_cursor() {
    let mut app = app();
    app.page = PackagesPage::History;
    app.history = vec![HistoryEntry {
      timestamp: "2026-10-05T10:00".into(),
      action: "upgraded".into(),
      package: "linux".into(),
      old_version: Some("1".into()),
      new_version: Some("2".into()),
    }];
    let rows = app.rows();
    assert!(rows.iter().all(|row| !row.is_selectable()));
    assert_eq!(rows[0].detail_text(), Some("1 → 2"));
    app.handle(KeyCode::Enter);
    assert_eq!(app.page, PackagesPage::History);
  }

  #[test]
  fn package_confirmation_cancel_does_not_start_an_operation() {
    let mut app = app();
    app.pending = Some(Action::Upgrade);
    app.handle(KeyCode::Enter);
    assert!(app.pending.is_none());
    assert!(app.action.is_none());
  }

  #[test]
  fn configure_mirrors_is_disabled_without_reflector() {
    let mut app = app();
    app.page = PackagesPage::Mirrors;
    app.mirrors = vec![Mirror {
      server: "https://mirror.example/$repo/os/$arch".into(),
      enabled: true,
      ..Default::default()
    }];
    let rows = app.rows();
    let configure = row(&rows, Item::ConfigureMirrors);
    assert!(!configure.is_selectable());
    assert_eq!(
      configure.detail_text(),
      Some(tr(app.lang, "control_center.unavailable_no_reflector"))
    );
    assert_eq!(
      rows.last().unwrap().detail_text(),
      Some(tr(app.lang, "control_center.mirror_enabled"))
    );
  }

  #[test]
  fn mirror_editor_is_a_page_with_adjustable_options() {
    let caps = Capabilities {
      has_reflector: true,
      ..Default::default()
    };
    let mut app = PackagesApp::new(Lang::for_locale("en-US"), Theme::load(), caps);
    app.mirror_countries = Some(vec!["Brazil".into(), "Argentina".into()]);
    app.page = PackagesPage::Mirrors;
    app.handle(KeyCode::Tab);
    app.handle(KeyCode::Enter);
    assert_eq!(app.page, PackagesPage::MirrorEditor);
    let countries = |app: &PackagesApp| app.mirror_options.as_ref().unwrap().countries.clone();
    assert_eq!(countries(&app), ["Brazil"]);
    app.handle(KeyCode::Right);
    assert_eq!(countries(&app), ["Argentina"]);
    app.handle(KeyCode::Left);
    assert_eq!(countries(&app), ["Brazil"], "← moves back");
    app.handle(KeyCode::Char(' '));
    assert_eq!(countries(&app), ["Argentina"], "Space moves forward");

    app.handle(KeyCode::Down);
    app.handle(KeyCode::Down);
    app.handle(KeyCode::Left);
    assert_eq!(app.mirror_options.as_ref().unwrap().age_hours, 11);
    app.handle(KeyCode::Down);
    app.handle(KeyCode::Right);
    assert_eq!(app.mirror_options.as_ref().unwrap().count, 11);
    app.handle(KeyCode::Down);
    app.handle(KeyCode::Enter);
    assert_eq!(app.mirror_options.as_ref().unwrap().sort, "age");

    app.handle(KeyCode::Esc);
    assert_eq!(app.page, PackagesPage::Mirrors);
    assert!(app.mirror_options.is_none());
    assert_eq!(
      app.menu.selected_id(&app.rows()),
      Some(Item::ConfigureMirrors)
    );
  }

  #[test]
  fn mirror_numbers_open_a_bounded_field() {
    let mut app = app();
    app.page = PackagesPage::MirrorEditor;
    app.mirror_options = Some(ReflectorOptions::default());
    press(&mut app, Item::MirrorCount);
    assert!(app.captures_text());
    for key in ['1', 'x', '5', '0', '0'] {
      app.handle(KeyCode::Char(key));
    }
    assert_eq!(app.input.as_deref(), Some("150"));
    app.handle(KeyCode::Enter);
    assert_eq!(app.mirror_options.as_ref().unwrap().count, 10);
    assert_eq!(
      app.status.as_ref().map(|status| status.kind),
      Some(StatusKind::Error),
      "values out of 1–100 are rejected"
    );
    press(&mut app, Item::MirrorAge);
    for key in ['4', '8'] {
      app.handle(KeyCode::Char(key));
    }
    app.handle(KeyCode::Enter);
    assert_eq!(app.mirror_options.as_ref().unwrap().age_hours, 48);
  }

  #[test]
  fn mirror_preview_returns_to_mirrors_and_asks_for_confirmation() {
    let mut app = app();
    app.page = PackagesPage::MirrorEditor;
    app.mirror_options = Some(ReflectorOptions::default());
    app.apply(Loaded::MirrorPreview(
      "Server = https://a/$repo/os/$arch\n".into(),
    ));
    assert_eq!(app.page, PackagesPage::Mirrors);
    assert!(matches!(app.pending, Some(Action::ApplyMirrors(_))));
  }

  #[test]
  fn country_cycling_falls_back_to_a_small_static_list_until_reflector_loads() {
    let mut options = ReflectorOptions {
      countries: vec!["Brazil".into()],
      ..Default::default()
    };
    cycle_country(&mut options, &[], 1);
    assert_eq!(options.countries, ["United States"]);
    cycle_country(&mut options, &["Brazil".into(), "Argentina".into()], 1);
    assert_eq!(options.countries, ["Brazil"]);
    cycle_country(&mut options, &["Brazil".into(), "Argentina".into()], 1);
    assert_eq!(options.countries, ["Argentina"]);
    cycle_country(&mut options, &["Brazil".into(), "Argentina".into()], 1);
    assert!(options.countries.is_empty(), "wraps to every country");
    cycle_country(&mut options, &["Brazil".into(), "Argentina".into()], -1);
    assert_eq!(options.countries, ["Argentina"]);
  }

  #[test]
  fn transaction_window_scrolls_closes_and_generates_action_plans() {
    let mut app = app();
    let live = LiveProcess::new();
    for _ in 0..40 {
      live.push_line("line");
    }
    app.transaction_live = Some(live.clone());
    app.transaction_open = true;
    let total = transaction_wrapped_lines(&live.output());
    assert_eq!(total, 40);
    app.handle(KeyCode::Down);
    assert_eq!(app.transaction_scroll, 1);
    app.handle(KeyCode::Up);
    assert_eq!(app.transaction_scroll, 0);
    app.handle(KeyCode::PageDown);
    assert_eq!(app.transaction_scroll, 10);
    app.handle(KeyCode::Home);
    assert_eq!(app.transaction_scroll, 0);
    app.handle(KeyCode::End);
    assert_eq!(
      app.transaction_scroll as usize,
      40usize.saturating_sub(TRANSACTION_CONTENT_HEIGHT)
    );
    app.handle(KeyCode::Esc);
    assert!(!app.transaction_open);
    assert_eq!(app.transaction_scroll, 0);
  }

  #[test]
  fn start_pending_opens_the_process_window_immediately() {
    let mut app = app();
    // The operation is only checked as built; test builds start no process
    // (the core `testing` feature), so the job below runs nothing.
    let (_, process) = privileged_process(Action::RefreshDatabase);
    assert_eq!(process.args, ["system-settings", "package", "refresh-db"]);
    app.pending = Some(Action::RefreshDatabase);
    app.start_pending();
    assert!(
      app.transaction_open,
      "window must appear when the process starts"
    );
    assert!(app.transaction_live.is_some());
    assert!(app.transaction_follow);
    assert!(app.action.is_some());
    app.handle(KeyCode::Esc);
    assert!(!app.transaction_open);
  }

  #[test]
  fn transaction_bottom_offset_is_zero_for_short_output() {
    let mut app = app();
    let live = LiveProcess::new();
    live.push_line("done.");
    app.transaction_live = Some(live);
    app.transaction_open = true;
    app.handle(KeyCode::End);
    assert_eq!(app.transaction_scroll, 0);
    assert_eq!(transaction_bottom_offset("done."), 0);
  }

  #[test]
  fn transaction_window_uses_theme_background_and_border() {
    let theme = Theme::load();
    let mut app = PackagesApp::new(
      Lang::for_locale("en-US"),
      theme.clone(),
      Capabilities::default(),
    );
    let live = LiveProcess::new();
    live.push_line("ok");
    app.transaction_live = Some(live);
    app.transaction_open = true;
    let mut terminal = Terminal::new(TestBackend::new(120, 25)).unwrap();
    terminal.draw(|frame| app.draw(frame)).unwrap();
    let buffer = terminal.backend().buffer();
    let styled = buffer.content.iter().any(|cell| {
      cell.symbol() == "│" && cell.bg == theme.background && cell.fg == theme.border_active
    });
    assert!(
      styled,
      "border should contrast with the background behind it"
    );
  }

  #[test]
  fn confirmation_starts_on_cancel_and_answers_y_and_n() {
    let mut app = app();
    app.request(Action::RefreshDatabase);
    assert!(!app.confirmation.is_confirm_focused());
    assert_eq!(app.footer_hints(&app.rows()), confirm_hints(app.lang));
    app.handle(KeyCode::Char('n'));
    assert!(app.pending.is_none());
    assert!(app.action.is_none());
    app.request(Action::RefreshDatabase);
    // y starts the job, which runs no process in test builds (core
    // `testing` feature); the command itself is checked as built in
    // `package_operations_build_the_privileged_system_settings_command`.
    app.handle(KeyCode::Char('y'));
    assert!(app.pending.is_none());
    assert!(app.action.is_some(), "y runs the confirmed operation");
    assert!(app.transaction_open);
  }

  #[test]
  fn only_removals_cleanups_and_downgrades_use_the_danger_style() {
    for action in [
      Action::Remove(vec!["zsh".into()]),
      Action::CleanCache("keep-one"),
      Action::Downgrade("/tmp/zsh.pkg.tar.zst".into()),
    ] {
      assert!(action.is_destructive(), "{action:?}");
    }
    for action in [
      Action::Install(vec!["zsh".into()]),
      Action::Reinstall("zsh".into()),
      Action::UpgradePackage("zsh".into()),
      Action::Upgrade,
      Action::RefreshDatabase,
      Action::AurInstall("paru-bin".into()),
      Action::ApplyMirrors(String::new()),
    ] {
      assert!(!action.is_destructive(), "{action:?}");
    }
  }

  #[test]
  fn plan_counters_show_a_readable_download_size() {
    let lang = Lang::for_locale("en-US");
    let message = preview_message(
      lang,
      &TransactionPlan {
        install: vec![package("zsh", false)],
        download_bytes: 5 * 1024 * 1024,
        ..Default::default()
      },
    );
    assert!(message.contains(": 1\n"), "{message}");
    assert!(message.ends_with(": 5 MiB"), "{message}");
  }

  #[test]
  fn aur_install_confirms_with_the_pkgbuild_warning_only() {
    let mut app = app();
    app.page = PackagesPage::Aur;
    app.aur.push(AurPackage {
      name: "paru-bin".into(),
      ..Default::default()
    });
    app.handle(KeyCode::Enter);
    press(&mut app, Item::Install);
    assert!(app.plan.is_none(), "the AUR helper has no transaction plan");
    assert_eq!(app.pending, Some(Action::AurInstall("paru-bin".into())));
    assert!(app.preview.is_none());
    let text = screen(&mut app, 100, 30);
    assert!(text.contains("paru-bin"), "{text}");
    assert!(!text.contains(": 0"), "{text}");
  }

  /// The process a privileged operation would start, built without running
  /// it; the executable stands for the installed Control Center.
  fn privileged_process(action: Action) -> (String, ProcessRequest) {
    let Ok(ActionCommand::Privileged { echo, request }) =
      action_command(action, &Capabilities::default())
    else {
      panic!("not a privileged operation");
    };
    let process =
      SystemSettingsOperation::new(SystemProcessRunner, "/usr/bin/argvus-control-center")
        .process_for(&request)
        .unwrap();
    (echo, process)
  }

  #[test]
  fn package_operations_build_the_privileged_system_settings_command() {
    let cases = [
      (
        Action::Install(vec!["zsh".into(), "jq".into()]),
        "$ pacman -S --needed zsh jq",
        vec!["install", "zsh", "jq"],
      ),
      (
        Action::Remove(vec!["zsh".into()]),
        "$ pacman -Rns zsh",
        vec!["remove", "zsh"],
      ),
      (
        Action::Reinstall("zsh".into()),
        "$ pacman -S zsh",
        vec!["reinstall", "zsh"],
      ),
      (Action::Upgrade, "$ pacman -Syu", vec!["upgrade"]),
      (
        Action::UpgradePackage("linux".into()),
        "$ pacman -S linux",
        vec!["upgrade-package", "linux"],
      ),
      (Action::RefreshDatabase, "$ pacman -Syy", vec!["refresh-db"]),
      (
        Action::CleanCache("keep-one"),
        "$ paccache -r",
        vec!["clean-cache", "keep-one"],
      ),
      (
        Action::Downgrade("/var/cache/pacman/pkg/zsh-5.9-1-x86_64.pkg.tar.zst".into()),
        "$ pacman -U /var/cache/pacman/pkg/zsh-5.9-1-x86_64.pkg.tar.zst",
        vec![
          "downgrade",
          "/var/cache/pacman/pkg/zsh-5.9-1-x86_64.pkg.tar.zst",
        ],
      ),
      (
        Action::ApplyMirrors("Server = https://a/$repo/os/$arch\n".into()),
        "$ update mirrorlist",
        vec!["mirror-apply", "Server = https://a/$repo/os/$arch\n"],
      ),
    ];
    for (action, expected_echo, expected_args) in cases {
      let (echo, process) = privileged_process(action);
      assert_eq!(echo, expected_echo);
      assert_eq!(process.program, "/usr/bin/argvus-control-center");
      let mut args = vec!["system-settings", "package"];
      args.extend(expected_args);
      assert_eq!(process.args, args);
    }
  }

  #[test]
  fn aur_install_runs_the_helper_as_the_normal_user() {
    let caps = |paru, yay| Capabilities {
      has_paru: paru,
      has_yay: yay,
      ..Default::default()
    };
    for (caps, helper) in [(caps(true, true), "paru"), (caps(false, true), "yay")] {
      let Ok(ActionCommand::User { echo, process }) =
        action_command(Action::AurInstall("paru-bin".into()), &caps)
      else {
        panic!("the AUR install is not privileged");
      };
      assert_eq!(echo, format!("$ {helper} -S paru-bin"));
      assert_eq!(process.program, helper);
      assert_eq!(process.args, ["-S", "paru-bin"]);
    }
    assert!(action_command(Action::AurInstall("paru-bin".into()), &caps(false, false)).is_err());
  }

  #[test]
  fn packages_render_at_80x24() {
    let mut app = orphans_app();
    let text = screen(&mut app, 80, 24);
    assert!(text.contains("libfoo"));
    assert!(text.contains("Danger zone"));
  }
}
