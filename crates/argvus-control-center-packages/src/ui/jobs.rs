//! Background work of the page: loading each view, polling the jobs and reporting their results.

use super::render::transaction_bottom_offset;
use super::*;

impl PackagesApp {
  pub(super) fn busy(&self) -> bool {
    self.job.is_some() || self.plan.is_some() || self.action.is_some()
  }

  pub(super) fn destructive(&self) -> bool {
    self.plan.is_some() || self.action.is_some()
  }

  pub fn reload(&mut self) {
    self.start_reload(false);
  }

  pub(super) fn reload_force(&mut self) {
    self.start_reload(true);
  }

  pub(super) fn reload_key(&self) -> String {
    format!(
      "{:?}|{}|{}",
      self.page,
      self.query,
      self.details_target().unwrap_or_default()
    )
  }

  pub(super) fn start_reload(&mut self, force: bool) {
    if self.destructive() {
      return;
    }
    if self.job.is_some() {
      return;
    }
    if let PackagesPage::Task(source, PackageTask::Install) = self.page
      && self.query.trim().is_empty()
    {
      self.status = Some(StatusMessage {
        kind: StatusKind::Info,
        text: match source {
          PackageSource::Aur => tr(self.lang, "control_center.press_to_search_the_aur"),
          PackageSource::Official => tr(self.lang, "control_center.press_to_search_packages"),
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
    // With the AUR menu on the Home, its packages are removed from there.
    let hide_foreign = self.has_aur_helper();
    self.job = Some(self.jobs.spawn(move |_| {
      let b = PackageBackend::new(SystemProcessRunner, caps);
      let aur_helper = || b.aur_helper().ok_or("AUR helper is unavailable");
      let data = match page {
        PackagesPage::Task(PackageSource::Official, PackageTask::Install) => {
          Loaded::Packages(b.search(&query).map_err(|e| e.to_string())?)
        }
        PackagesPage::Task(PackageSource::Official, PackageTask::Remove) => {
          let mut packages = b.installed().map_err(|e| e.to_string())?;
          if hide_foreign {
            packages.retain(|package| !package.foreign);
          }
          Loaded::Packages(packages)
        }
        PackagesPage::Task(PackageSource::Official, PackageTask::Update) => {
          Loaded::Updates(b.updates().map_err(|e| e.to_string())?)
        }
        PackagesPage::Task(PackageSource::Aur, PackageTask::Install) => Loaded::Aur(
          b.aur_search(aur_helper()?, &query)
            .map_err(|e| e.to_string())?,
        ),
        PackagesPage::Task(PackageSource::Aur, PackageTask::Remove) => {
          Loaded::Packages(b.foreign().map_err(|e| e.to_string())?)
        }
        PackagesPage::Task(PackageSource::Aur, PackageTask::Update) => {
          Loaded::Updates(b.aur_updates(aur_helper()?).map_err(|e| e.to_string())?)
        }
        PackagesPage::Orphans => Loaded::Packages(b.orphans().map_err(|e| e.to_string())?),
        PackagesPage::Cache | PackagesPage::CacheFiles | PackagesPage::Downgrade => {
          Loaded::Cache(b.cache())
        }
        PackagesPage::History => Loaded::History(b.history().map_err(|e| e.to_string())?),
        PackagesPage::Mirrors | PackagesPage::MirrorEditor => Loaded::Mirrors(b.mirrors()),
        PackagesPage::Details | PackagesPage::HistoryDetails(_) => Loaded::Details(Box::new(
          b.details(&name.ok_or("package not found")?)
            .map_err(|e| e.to_string())?,
        )),
        PackagesPage::Home | PackagesPage::Source(_) => Loaded::Dashboard(b.dashboard()),
        PackagesPage::Repositories
        | PackagesPage::RepositoryAdd
        | PackagesPage::RepositoryList
        | PackagesPage::RepositoryDetails(_) => {
          Loaded::Repositories(b.repositories().map_err(|e| e.to_string())?)
        }
      };
      Ok(Ok(data))
    }));
    self.status = Some(StatusMessage {
      kind: StatusKind::Info,
      text: tr(self.lang, "control_center.loading_packages").into(),
    });
  }

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

  pub(super) fn poll_action(&mut self) -> bool {
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
        self.after_success();
        self.reload_force();
        true
      }
      JobState::Finished(Ok(Err(e))) | JobState::Finished(Err(e)) => {
        self.action = None;
        self.running = None;
        self.error(e);
        true
      }
    }
  }

  /// What follows a successful operation: after adding a repository, the
  /// draft is cleared, the List opens and the upgrade is offered; after
  /// removing one, its page closes.
  pub(super) fn after_success(&mut self) {
    match self.running.take() {
      Some(Action::RepoAdd(_)) => {
        self.repo_draft = RepositoryDraft::default();
        if self.page == PackagesPage::RepositoryAdd {
          self.go(PackagesPage::RepositoryList);
        }
        self.follow_up = Some(Action::UpgradeAfterRepoAdd);
      }
      Some(Action::RepoRemove { .. })
        if matches!(self.page, PackagesPage::RepositoryDetails(_)) =>
      {
        self.go(PackagesPage::RepositoryList);
      }
      _ => {}
    }
    if !self.transaction_open {
      self.ask_follow_up();
    }
  }

  /// Opens the confirmation that follows an operation, once its output is
  /// no longer on screen.
  pub(super) fn ask_follow_up(&mut self) {
    if let Some(next) = self.follow_up.take() {
      self.request(next);
    }
  }

  pub(super) fn apply(&mut self, d: Loaded) -> bool {
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
      Loaded::Repositories(list) => self.repositories = list,
      Loaded::RepositoryReview(review) => {
        self.request(Action::RepoAdd(review));
        self.status = None;
        // Not a list refresh: skip the "refreshed" status under the
        // confirmation.
        reported_error = true;
      }
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

  pub(super) fn error(&mut self, e: impl Into<String>) {
    self.status = Some(StatusMessage {
      kind: StatusKind::Error,
      text: terminal_text(&e.into()),
    });
  }

  pub(super) fn success(&mut self, e: impl Into<String>) {
    self.status = Some(StatusMessage {
      kind: StatusKind::Success,
      text: e.into(),
    });
  }

  /// Drops the list still loading for the previous page, so its result never
  /// fills the list that opens now.
  pub(super) fn cancel_load(&mut self) {
    if let Some(job) = self.job.take() {
      job.cancel();
    }
  }

  /// Enter on a package of a task list: the operation of that list, after
  /// the same plan and confirmation as before.
  pub(super) fn run_task(&mut self, item: Item) {
    let PackagesPage::Task(source, task) = self.page else {
      return;
    };
    match (item, task) {
      (Item::Package(index), PackageTask::Install) => {
        if let Some(package) = self.packages.get(index) {
          let (name, installed) = (package.name.clone(), package.installed);
          self.install_package(name, installed, source);
        }
      }
      (Item::Aur(index), PackageTask::Install) => {
        if let Some(package) = self.aur.get(index) {
          let (name, installed) = (package.name.clone(), package.installed);
          self.install_package(name, installed, PackageSource::Aur);
        }
      }
      (Item::Package(index), PackageTask::Remove) => {
        if let Some(name) = self.packages.get(index).map(|package| package.name.clone()) {
          self.begin_plan(Action::Remove(vec![name]));
        }
      }
      (Item::Update(index), PackageTask::Update) => {
        if let Some(name) = self.updates.get(index).map(|update| update.name.clone()) {
          self.request(match source {
            PackageSource::Official => Action::UpgradePackage(name),
            PackageSource::Aur => Action::AurUpgradePackage(name),
          });
        }
      }
      _ => {}
    }
  }
}
