//! Rows of every list of the page, built from the loaded data and the current filter.

use super::*;

impl PackagesApp {
  /// Keeps the cursor on a selectable row after the rows changed.
  pub(super) fn normalize(&mut self) {
    let rows = self.rows();
    self.menu.normalize(&rows);
  }

  /// Name of the package whose details page is open.
  pub(super) fn details_target(&self) -> Option<String> {
    if self.page != PackagesPage::Details {
      return None;
    }
    self
      .details
      .as_ref()
      .map(|details| details.package.name.clone())
      .or_else(|| self.details_name.clone())
  }

  /// Whether an AUR helper (paru or yay) is installed, which shows the AUR menu.
  pub(super) fn has_aur_helper(&self) -> bool {
    self.capabilities.has_paru || self.capabilities.has_yay
  }

  pub(super) fn matches_query(&self, name: &str, description: &str) -> bool {
    let query = self.query.trim().to_ascii_lowercase();
    query.is_empty()
      || name.to_ascii_lowercase().contains(&query)
      || description.to_ascii_lowercase().contains(&query)
  }

  /// Packages matching the filter, with their index in `packages`.
  pub(super) fn visible_packages(&self) -> Vec<(usize, &Package)> {
    self
      .packages
      .iter()
      .enumerate()
      .filter(|(_, package)| self.matches_query(&package.name, &package.description))
      .collect()
  }

  /// Updates matching the filter, with their index in `updates`.
  pub(super) fn visible_updates(&self) -> Vec<(usize, &Update)> {
    self
      .updates
      .iter()
      .enumerate()
      .filter(|(_, update)| self.matches_query(&update.name, ""))
      .collect()
  }

  /// AUR results matching the filter, with their index in `aur`.
  pub(super) fn visible_aur(&self) -> Vec<(usize, &AurPackage)> {
    self
      .aur
      .iter()
      .enumerate()
      .filter(|(_, package)| self.matches_query(&package.name, &package.description))
      .collect()
  }

  /// A translated field label without the trailing colon some catalog
  /// entries carry, since Info rows draw label and value in columns.
  pub(super) fn field(&self, key: &str) -> &'static str {
    tr(self.lang, key).trim_end_matches(':')
  }

  /// Rows of the current page.
  pub(super) fn rows(&self) -> Vec<Row<Item>> {
    match self.page {
      PackagesPage::Home => self.home_rows(),
      PackagesPage::Source(source) => self.source_rows(source),
      PackagesPage::Task(_, PackageTask::Install | PackageTask::Remove) => self.package_rows(),
      PackagesPage::Task(_, PackageTask::Update) => self.update_rows(),
      PackagesPage::Orphans => self.orphan_rows(),
      PackagesPage::Details => self.details_rows(),
      PackagesPage::Cache => self.cache_rows(),
      PackagesPage::CacheFiles => self.cache_file_rows(),
      PackagesPage::Downgrade => self.downgrade_rows(),
      PackagesPage::History | PackagesPage::HistoryDetails(_) => self.history_rows(),
      PackagesPage::Mirrors => self.mirror_rows(),
      PackagesPage::MirrorEditor => self.mirror_editor_rows(),
      PackagesPage::Repositories => self.repositories_rows(),
      PackagesPage::RepositoryAdd => self.repository_add_rows(),
      PackagesPage::RepositoryList => self.repository_list_rows(),
      PackagesPage::RepositoryDetails(index) => self.repository_details_rows(index),
    }
  }

  /// The row of an empty filtered list, saying why it is empty. `None` while
  /// the list is still loading, so the loading status is not contradicted.
  pub(super) fn empty_list_row(&self) -> Option<Row<Item>> {
    if self.job.is_some() {
      return None;
    }
    let message = match self.page {
      PackagesPage::Task(PackageSource::Official, PackageTask::Install)
        if self.query.is_empty() =>
      {
        "control_center.press_to_search_packages"
      }
      PackagesPage::Task(PackageSource::Aur, PackageTask::Install) if self.query.is_empty() => {
        "control_center.press_to_search_the_aur"
      }
      PackagesPage::Task(_, PackageTask::Update) if self.query.is_empty() => {
        "control_center.no_pending_updates"
      }
      PackagesPage::Task(_, PackageTask::Install) => "control_center.search_no_results",
      _ if self.query.is_empty() => "control_center.search_no_results",
      _ => "control_center.no_matching_items",
    };
    Some(Row::info(tr(self.lang, message), ""))
  }

  /// The translated name of `source`, as the Home lists it.
  pub(super) fn source_label(&self, source: PackageSource) -> &'static str {
    tr(
      self.lang,
      match source {
        PackageSource::Official => "control_center.official_repositories",
        PackageSource::Aur => "control_center.aur_repository",
      },
    )
  }

  /// The translated name of `task`, as the source menus list it.
  pub(super) fn task_label(&self, task: PackageTask) -> &'static str {
    tr(
      self.lang,
      match task {
        PackageTask::Install => "control_center.install",
        PackageTask::Remove => "control_center.remove",
        PackageTask::Update => "control_center.update",
      },
    )
  }

  /// Installed packages of the sync repositories. Without the AUR menu the
  /// Official Remove list also keeps the AUR packages, so it counts them.
  pub(super) fn official_installed_count(&self) -> usize {
    let dashboard = &self.dashboard;
    if self.has_aur_helper() {
      dashboard
        .installed_count
        .saturating_sub(dashboard.foreign_count)
    } else {
      dashboard.installed_count
    }
  }

  /// The Packages home: one row per source, then the shared tools.
  pub(super) fn home_rows(&self) -> Vec<Row<Item>> {
    let dashboard = &self.dashboard;
    let lang = self.lang;
    let packages = tr(lang, "control_center.packages_c945db");
    self
      .home_pages()
      .into_iter()
      .map(|page| {
        let (icon, label, detail) = match page {
          PackagesPage::Source(PackageSource::Official) => (
            icons::PACKAGES,
            self.source_label(PackageSource::Official),
            format!(
              "{} {packages} · {} {}",
              self.official_installed_count(),
              dashboard.update_count,
              tr(lang, "control_center.pending")
            ),
          ),
          PackagesPage::Source(PackageSource::Aur) => (
            icons::USERS,
            self.source_label(PackageSource::Aur),
            format!(
              "{} · {} {packages}",
              dashboard.aur_helper.as_deref().unwrap_or("—"),
              dashboard.foreign_count
            ),
          ),
          PackagesPage::Cache => (
            icons::DATABASE,
            tr(lang, "control_center.cache"),
            format!(
              "{} {} · {}",
              dashboard.cache_count,
              tr(lang, "control_center.files_7093b3"),
              human_bytes(dashboard.cache_bytes)
            ),
          ),
          PackagesPage::History => (
            icons::HISTORY,
            tr(lang, "control_center.history"),
            format!(
              "{} {}",
              dashboard.history_count,
              tr(lang, "control_center.entries")
            ),
          ),
          PackagesPage::Downgrade => (
            icons::DOWNGRADE,
            tr(lang, "control_center.downgrade"),
            format!(
              "{} {}",
              dashboard.cache_count,
              tr(lang, "control_center.versions")
            ),
          ),
          _ => (
            icons::NETWORK,
            tr(lang, "control_center.mirrors"),
            format!(
              "{}/{} {}",
              dashboard.mirrors_enabled,
              dashboard.mirrors_total,
              tr(lang, "control_center.active_ae7190")
            ),
          ),
        };
        Row::submenu(Item::Open(page), label)
          .icon(icon)
          .detail(detail)
      })
      .collect()
  }

  /// Install, Remove and Update of one source, with their counters.
  pub(super) fn source_rows(&self, source: PackageSource) -> Vec<Row<Item>> {
    let dashboard = &self.dashboard;
    let lang = self.lang;
    let packages = tr(lang, "control_center.packages_c945db");
    [
      (PackageTask::Install, icons::ADD),
      (PackageTask::Remove, icons::DELETE),
      (PackageTask::Update, icons::UPDATE),
    ]
    .into_iter()
    .map(|(task, icon)| {
      let detail = match (source, task) {
        (PackageSource::Official, PackageTask::Install) => {
          Some(format!("{} {packages}", dashboard.available_count))
        }
        (PackageSource::Official, PackageTask::Remove) => {
          Some(format!("{} {packages}", self.official_installed_count()))
        }
        (PackageSource::Official, PackageTask::Update) => Some(format!(
          "{} {}",
          dashboard.update_count,
          tr(lang, "control_center.pending")
        )),
        (PackageSource::Aur, PackageTask::Install) => dashboard.aur_helper.clone(),
        (PackageSource::Aur, PackageTask::Remove) => {
          Some(format!("{} {packages}", dashboard.foreign_count))
        }
        // Counting the AUR updates queries the AUR: the list does it on open.
        (PackageSource::Aur, PackageTask::Update) => None,
      };
      let row = Row::submenu(
        Item::Open(PackagesPage::Task(source, task)),
        self.task_label(task),
      )
      .icon(icon);
      match detail {
        Some(detail) => row.detail(detail),
        None => row,
      }
    })
    .chain((source == PackageSource::Official).then(|| {
      Row::submenu(
        Item::Open(PackagesPage::Repositories),
        tr(lang, "control_center.repositories"),
      )
      .icon(icons::STORAGE)
      .detail(self.repository_count_text(dashboard.repository_count))
    }))
    .collect()
  }

  /// Install and Remove lists: Enter installs or removes the package, `→`
  /// opens its details. Official Remove starts with the Orphans page.
  pub(super) fn package_rows(&self) -> Vec<Row<Item>> {
    let lang = self.lang;
    let installed = tr(lang, "control_center.installed");
    let mut rows: Vec<Row<Item>> = Vec::new();
    if self.page == PackagesPage::Task(PackageSource::Official, PackageTask::Remove) {
      rows.push(
        Row::submenu(
          Item::Open(PackagesPage::Orphans),
          tr(lang, "control_center.orphans_29aae8"),
        )
        .icon(icons::CLEAN)
        .detail(format!(
          "{} {}",
          self.dashboard.orphan_count,
          tr(lang, "control_center.orphans")
        )),
      );
      rows.push(Row::section(installed));
    }
    if self.page == PackagesPage::Task(PackageSource::Aur, PackageTask::Install) {
      rows.extend(self.visible_aur().into_iter().map(|(index, package)| {
        let mut parts = vec![package.version.clone(), "AUR".into()];
        if package.installed {
          parts.push(installed.into());
        }
        Row::action(Item::Aur(index), package.name.clone()).detail(joined_parts(&parts))
      }));
    } else {
      let searching = matches!(self.page, PackagesPage::Task(_, PackageTask::Install));
      rows.extend(self.visible_packages().into_iter().map(|(index, package)| {
        let mut parts = vec![package.version.clone()];
        if searching {
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
        Row::action(Item::Package(index), package.name.clone()).detail(joined_parts(&parts))
      }));
    }
    if !rows
      .iter()
      .any(|row| row.id().is_some_and(|item| item.is_list_entry()))
    {
      rows.extend(self.empty_list_row());
    }
    rows
  }

  /// Orphans: Space marks (`[x]`), Enter opens the details; the marked set
  /// is removed from the Danger zone.
  pub(super) fn orphan_rows(&self) -> Vec<Row<Item>> {
    let mut rows: Vec<Row<Item>> = Vec::new();
    rows.extend(self.visible_packages().into_iter().map(|(index, package)| {
      let marked = self.marked.contains(&package.name);
      let row = Row::toggle(Item::Orphan(index), package.name.clone(), marked);
      if package.version.is_empty() {
        row
      } else {
        row.detail(package.version.clone())
      }
    }));
    if rows.is_empty() {
      rows.extend(self.empty_list_row());
    }
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

  /// Update lists: one row per pending update (Enter updates it after the
  /// confirmation, `→` opens its details); Upgrade all and Refresh database
  /// live in the action bar (see `update_action_rows`). An empty list says
  /// why, except while the list is still loading.
  pub(super) fn update_rows(&self) -> Vec<Row<Item>> {
    let mut rows: Vec<Row<Item>> = self
      .visible_updates()
      .into_iter()
      .map(|(index, update)| {
        Row::action(Item::Update(index), update.name.clone())
          .detail(format!("{} → {}", update.current, update.available))
      })
      .collect();
    if rows.is_empty() {
      rows.extend(self.empty_list_row());
    }
    rows
  }

  /// The action bar of the Update lists: Upgrade all (enabled with pending
  /// updates), and Refresh database on the Official list only, since the
  /// AUR helper refreshes its own metadata.
  pub(super) fn update_action_rows(&self) -> Vec<Row<Item>> {
    let has_updates = !self.visible_updates().is_empty();
    if let PackagesPage::Task(PackageSource::Aur, _) = self.page {
      return vec![
        Row::action(
          Item::UpgradeAll,
          tr(self.lang, "control_center.upgrade_all_aur"),
        )
        .icon(icons::UPDATE)
        .enabled(has_updates),
      ];
    }
    vec![
      Row::action(
        Item::UpgradeAll,
        tr(self.lang, "control_center.upgrade_all"),
      )
      .icon(icons::UPDATE)
      .enabled(has_updates),
      Row::action(
        Item::RefreshDatabase,
        tr(self.lang, "control_center.refresh_database"),
      )
      .icon(icons::SYNC),
    ]
  }

  /// Details of one package: its actions first, the metadata in sections,
  /// and `Remove` in the Danger zone.
  pub(super) fn details_rows(&self) -> Vec<Row<Item>> {
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
  pub(super) fn cache_rows(&self) -> Vec<Row<Item>> {
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
  pub(super) fn cache_file_rows(&self) -> Vec<Row<Item>> {
    self
      .cache
      .iter()
      .map(|file| Row::info(cached_display_name(file), human_bytes(file.bytes)))
      .collect()
  }

  /// Downgrade: every cached file installs that version after the
  /// confirmation.
  pub(super) fn downgrade_rows(&self) -> Vec<Row<Item>> {
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
  pub(super) fn history_rows(&self) -> Vec<Row<Item>> {
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
  pub(super) fn mirror_rows(&self) -> Vec<Row<Item>> {
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
  pub(super) fn mirror_editor_rows(&self) -> Vec<Row<Item>> {
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
}

/// Non-empty parts of a list row's detail, joined with ` · `.
pub(super) fn joined_parts(parts: &[String]) -> String {
  parts
    .iter()
    .filter(|part| !part.is_empty())
    .cloned()
    .collect::<Vec<_>>()
    .join(" · ")
}

pub(super) fn join_or_dash(values: &[String]) -> String {
  if values.is_empty() {
    "—".into()
  } else {
    values.join(" ")
  }
}

pub(super) fn cached_display_name(package: &CachePackage) -> String {
  if !package.version.is_empty() {
    return format!("{} {}", package.name, package.version);
  }
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
