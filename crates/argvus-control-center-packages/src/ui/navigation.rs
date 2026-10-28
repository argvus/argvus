//! Page navigation: opening pages and package details, going back, and the breadcrumb.

use super::*;

impl PackagesApp {
  /// The pages listed on the Packages home, in order.
  pub(super) fn home_pages(&self) -> Vec<PackagesPage> {
    let mut pages = vec![PackagesPage::Source(PackageSource::Official)];
    if self.has_aur_helper() {
      pages.push(PackagesPage::Source(PackageSource::Aur));
    }
    pages.extend([
      PackagesPage::Cache,
      PackagesPage::History,
      PackagesPage::Downgrade,
      PackagesPage::Mirrors,
    ]);
    pages
  }

  /// Opens `page` with the cursor on its first selectable row.
  pub(super) fn go(&mut self, page: PackagesPage) {
    self.page = page;
    self.menu = MenuState::default();
    self.actions = ActionFocus::default();
  }

  /// Esc/`←`: one level up, with the cursor back on the row that opened the
  /// page; `true` leaves the Packages home.
  pub(super) fn back(&mut self) -> bool {
    let (parent, origin) = match self.page {
      PackagesPage::Home => return true,
      PackagesPage::Details => (self.details_parent, self.details_origin),
      PackagesPage::Task(source, task) => (
        PackagesPage::Source(source),
        Some(Item::Open(PackagesPage::Task(source, task))),
      ),
      PackagesPage::Orphans => {
        // The orphans replaced the installed list: load it again.
        self.cancel_load();
        self.packages.clear();
        let remove = PackagesPage::Task(PackageSource::Official, PackageTask::Remove);
        self.go(remove);
        self.query.clear();
        self.reload_force();
        let rows = self.rows();
        self.menu.select(&rows, &Item::Open(PackagesPage::Orphans));
        return false;
      }
      PackagesPage::HistoryDetails(_) => (PackagesPage::History, None),
      PackagesPage::CacheFiles => (PackagesPage::Cache, Some(Item::CacheFiles)),
      PackagesPage::MirrorEditor => {
        self.mirror_options = None;
        (PackagesPage::Mirrors, Some(Item::ConfigureMirrors))
      }
      PackagesPage::Repositories => (
        PackagesPage::Source(PackageSource::Official),
        Some(Item::Open(PackagesPage::Repositories)),
      ),
      PackagesPage::RepositoryAdd if self.repo_draft.is_touched() && self.discard.is_none() => {
        // Leaving drops the draft: ask first (the confirmation calls back
        // here once the draft is cleared).
        self.discard = Some(ConfirmState::new());
        return false;
      }
      page @ (PackagesPage::RepositoryAdd | PackagesPage::RepositoryList) => {
        (PackagesPage::Repositories, Some(Item::Open(page)))
      }
      PackagesPage::RepositoryDetails(index) => {
        (PackagesPage::RepositoryList, Some(Item::Repository(index)))
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

  /// Opens `page` from outside the Packages pages (a CLI route or the global
  /// search), loading its data as the rows that open it do.
  pub fn open(&mut self, page: PackagesPage) {
    self.open_page(page);
    if matches!(page, PackagesPage::Source(_)) {
      self.reload();
    }
  }

  /// Opens `page` from a row of the Home, a source menu or the Remove list.
  pub(super) fn open_page(&mut self, page: PackagesPage) {
    self.go(page);
    match page {
      // The dashboard loaded with the Home already has the counters.
      PackagesPage::Source(_) => {}
      // Each list loads from scratch, so another list never shows its
      // packages while this one loads.
      PackagesPage::Task(..) | PackagesPage::Orphans => {
        self.cancel_load();
        self.query.clear();
        self.packages.clear();
        self.updates.clear();
        self.aur.clear();
        self.reload_force();
      }
      _ => self.reload(),
    }
  }

  /// `→` on a package of a task list opens its details; `false` when the
  /// selected row is not a package.
  pub(super) fn open_selected_details(&mut self) -> bool {
    if !matches!(self.page, PackagesPage::Task(..)) || self.destructive() {
      return false;
    }
    let rows = self.rows();
    self.menu.normalize(&rows);
    match self.menu.selected_id(&rows) {
      Some(item) if item.is_list_entry() => {
        self.open_details(item);
        true
      }
      _ => false,
    }
  }

  /// Opens the details page of the package of `item`, remembering the list
  /// and the row to return to.
  pub(super) fn open_details(&mut self, item: Item) {
    let name = match item {
      Item::Package(index) | Item::Orphan(index) => {
        self.packages.get(index).map(|package| package.name.clone())
      }
      Item::Update(index) => self.updates.get(index).map(|update| update.name.clone()),
      Item::Aur(index) => self.aur.get(index).map(|package| package.name.clone()),
      _ => None,
    };
    let Some(name) = name else {
      return;
    };
    self.details_parent = self.page;
    self.details_origin = Some(item);
    self.details_name = Some(name);
    self.details = None;
    self.go(PackagesPage::Details);
    if let Item::Aur(index) = item
      && let Some(package) = self.aur.get(index)
    {
      // AUR search results are not in the local database: the page shows
      // what the search returned, as before.
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
    } else {
      self.reload();
    }
  }

  pub(super) fn breadcrumb(&self) -> String {
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

  /// The path of the current page under `Packages`.
  pub(super) fn page_label(&self) -> String {
    self.label_of(self.page)
  }
}
