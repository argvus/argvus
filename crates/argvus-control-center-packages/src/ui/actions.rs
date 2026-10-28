//! Package actions: activating rows, planning and running pacman or AUR transactions, and the mirror editor.

use super::input::step_within;
use super::*;

impl PackagesApp {
  /// Runs the row `item`.
  pub(super) fn activate(&mut self, item: Item) {
    if item.waits_for_jobs() && self.busy() {
      return;
    }
    if !item.is_mirror_option() && self.destructive() {
      return;
    }
    match item {
      Item::Open(page) => self.open_page(page),
      Item::Package(_) | Item::Aur(_) | Item::Update(_) => self.run_task(item),
      Item::Orphan(_) => self.open_details(item),
      Item::CacheFile(index) => {
        if let Some(file) = self.cache.get(index) {
          let path = file.path.clone();
          self.request(Action::Downgrade(path));
        }
      }
      Item::Install => self.install_open_package(),
      Item::Remove => self.remove_open_package(),
      Item::UpgradeAll => match self.page {
        PackagesPage::Task(PackageSource::Aur, _) => self.request(Action::AurUpgrade),
        _ => self.begin_plan(Action::Upgrade),
      },
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
      Item::RepoField(field) => self.activate_repository_field(field),
      Item::RepoReview => self.start_repository_review(),
      Item::Repository(index) => self.go(PackagesPage::RepositoryDetails(index)),
      Item::RepoToggle => self.toggle_open_repository(),
      Item::RepoRemove => self.remove_open_repository(),
    }
  }

  /// `←/→` (and Space) on a reflector option: cycles the choices or moves
  /// the number one step, within the same bounds as before.
  pub(super) fn adjust(&mut self, item: Item, step: i32) {
    if let Item::RepoField(field) = item {
      self.adjust_repository_field(field, step);
      return;
    }
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
  pub(super) fn request(&mut self, action: Action) {
    self.confirmation = ConfirmState::new();
    self.choice = ChoiceState::new();
    self.preview = None;
    self.pending = Some(action);
  }

  pub(super) fn open_mirror_editor(&mut self) {
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

  pub(super) fn spawn_country_list(&mut self) {
    let capabilities = self.capabilities.clone();
    self.job = Some(
      self
        .jobs
        .spawn(move |_| Ok(fetch_reflector_countries(capabilities).map(Loaded::MirrorCountries))),
    );
  }

  pub(super) fn start_mirror_preview(&mut self, options: ReflectorOptions) {
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
  pub(super) fn open_number_input(&mut self, mode: InputMode) {
    self.input = Some(String::new());
    self.input_mode = Some(mode);
  }

  pub(super) fn start_pending(&mut self) {
    let Some(a) = self.pending.take() else { return };
    self.running = Some(a.clone());
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

  pub(super) fn begin_plan(&mut self, action: Action) {
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
      if let Action::Remove(packages) = &action {
        // Packages other installed packages need cannot be removed alone:
        // the confirmation offers removing them together (cascade).
        return Ok(match backend.plan_remove(packages) {
          Ok(preview) => Ok((action, preview)),
          Err(PackageError::BreaksDependencies(dependents)) => backend
            .plan_remove_cascade(packages)
            .map(|preview| {
              let packages = packages.clone();
              (
                Action::RemoveCascade {
                  packages,
                  dependents,
                },
                preview,
              )
            })
            .map_err(|error| error.to_string()),
          Err(error) => Err(error.to_string()),
        });
      }
      let preview = match &action {
        Action::Install(packages) => backend.plan_install(packages),
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
  pub(super) fn open_package_installed(&self) -> bool {
    if let Some(details) = &self.details {
      return details.package.installed;
    }
    match self.details_origin {
      Some(Item::Aur(index)) => self.aur.get(index).is_some_and(|package| package.installed),
      Some(Item::Package(index) | Item::Orphan(index)) => self
        .packages
        .get(index)
        .is_some_and(|package| package.installed),
      // Only installed packages have pending updates.
      Some(Item::Update(_)) => true,
      _ => false,
    }
  }

  /// The source of the list that opened the details page.
  pub(super) fn details_source(&self) -> PackageSource {
    match self.details_parent {
      PackagesPage::Task(source, _) => source,
      _ => PackageSource::Official,
    }
  }

  /// `Install`/`Reinstall` on the package details page.
  pub(super) fn install_open_package(&mut self) {
    let Some(name) = self.details_target() else {
      return;
    };
    let installed = self.open_package_installed();
    self.install_package(name, installed, self.details_source());
  }

  /// Installs `name`, or reinstalls it when `installed`. The AUR helper has
  /// no transaction plan: its confirmation shows only the PKGBUILD warning,
  /// without the empty 0/0/0 counters, and it also rebuilds an installed
  /// AUR package, which `pacman -S` cannot find (D58).
  pub(super) fn install_package(&mut self, name: String, installed: bool, source: PackageSource) {
    match (source, installed) {
      (PackageSource::Aur, _) => self.request(Action::AurInstall(name)),
      (PackageSource::Official, true) => self.begin_plan(Action::Reinstall(name)),
      (PackageSource::Official, false) => self.begin_plan(Action::Install(vec![name])),
    }
  }

  /// `Remove` on the package details page; only installed packages.
  pub(super) fn remove_open_package(&mut self) {
    if let Some(name) = self.details_target()
      && self.open_package_installed()
    {
      self.begin_plan(Action::Remove(vec![name]));
    }
  }
}

pub(super) fn action_message(lang: Lang, a: &Action) -> String {
  match a {
    Action::Install(v) => format!("{}: {}", tr(lang, "control_center.install"), v.join(", ")),
    Action::Remove(v) => format!("{}: {}", tr(lang, "control_center.remove"), v.join(", ")),
    Action::RemoveOnly(v) => format!(
      "{}: {}",
      tr(lang, "control_center.remove_only_title"),
      v.join(", ")
    ),
    Action::RemoveCascade {
      packages,
      dependents,
    } => tr(lang, "control_center.remove_with_dependents")
      .replace("{packages}", &packages.join(", "))
      .replace("{dependents}", &shortened_list(dependents)),
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
    Action::AurUpgradePackage(v) => format!(
      "{}: {v}? {}",
      tr(lang, "control_center.update_aur_package"),
      tr(
        lang,
        "control_center.pkgbuilds_execute_build_instructions_as_the_normal_user"
      )
    ),
    Action::AurUpgrade => format!(
      "{}? {}",
      tr(lang, "control_center.upgrade_all_aur"),
      tr(
        lang,
        "control_center.pkgbuilds_execute_build_instructions_as_the_normal_user"
      )
    ),
    Action::RepoAdd(review) => repositories::add_message(lang, review),
    Action::UpgradeAfterRepoAdd => tr(lang, "control_center.repo_sync_now").into(),
    Action::RepoEnable(name) => {
      tr(lang, "control_center.repo_enable_message").replace("{name}", name)
    }
    Action::RepoDisable(name) => {
      tr(lang, "control_center.repo_disable_message").replace("{name}", name)
    }
    Action::RepoRemove { name, .. } => {
      tr(lang, "control_center.repo_remove_message").replace("{name}", name)
    }
    Action::ApplyMirrors(content) => format!(
      "{}\n{}",
      tr(lang, "control_center.apply_these_mirrors"),
      mirror_preview_summary(content)
    ),
  }
}

/// The option row of `choice` in the removal choice: only the packages, or
/// the cascade with the number of packages its `plan` removes.
pub(super) fn choice_label(lang: Lang, choice: &Action, plan: Option<&TransactionPlan>) -> String {
  match choice {
    Action::RepoRemove {
      delete_key: false, ..
    } => tr(lang, "control_center.repo_remove_only").into(),
    Action::RepoRemove {
      delete_key: true, ..
    } => tr(lang, "control_center.repo_remove_with_key").into(),
    Action::RemoveOnly(packages) => {
      tr(lang, "control_center.remove_only_selected").replace("{packages}", &packages.join(", "))
    }
    _ => tr(lang, "control_center.remove_with_dependents_option").replace(
      "{count}",
      &plan.map_or(0, |plan| plan.remove.len()).to_string(),
    ),
  }
}

/// Most dependents named in the removal choice; the cascade option gives
/// the number of packages it removes.
pub(super) const DEPENDENTS_SHOWN: usize = 8;

/// `names` joined for a message, cut after [`DEPENDENTS_SHOWN`] with `…`.
pub(super) fn shortened_list(names: &[String]) -> String {
  let mut text = names
    .iter()
    .take(DEPENDENTS_SHOWN)
    .cloned()
    .collect::<Vec<_>>()
    .join(", ");
  if names.len() > DEPENDENTS_SHOWN {
    text.push_str(", …");
  }
  text
}

/// The transaction plan counters shown under the confirmation message, with
/// the download size in readable units.
pub(super) fn preview_message(lang: Lang, plan: &TransactionPlan) -> String {
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
pub(super) enum ActionCommand {
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
pub(super) fn action_command(a: Action, caps: &Capabilities) -> Result<ActionCommand, String> {
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
    Action::RemoveOnly(v) => privileged(format!("$ pacman -Rdd {}", v.join(" ")), "remove-only", v),
    Action::RemoveCascade { packages, .. } => privileged(
      format!("$ pacman -Rcns {}", packages.join(" ")),
      "remove-cascade",
      packages,
    ),
    Action::Reinstall(v) => privileged(format!("$ pacman -S {v}"), "reinstall", vec![v]),
    Action::Upgrade => privileged("$ pacman -Syu".into(), "upgrade", vec![]),
    Action::UpgradePackage(v) => privileged(format!("$ pacman -S {v}"), "upgrade-package", vec![v]),
    Action::RefreshDatabase => privileged("$ pacman -Syy".into(), "refresh-db", vec![]),
    Action::CleanCache(v) => privileged("$ paccache -r".into(), "clean-cache", vec![v.into()]),
    Action::Downgrade(v) => privileged(format!("$ pacman -U {v}"), "downgrade", vec![v]),
    Action::RepoAdd(review) => {
      let name = review.name().to_owned();
      let file = argvus_control_center_core::pacman_conf::repository_file(&name);
      let mut echo = format!(
        "$ write {} and Include it in /etc/pacman.conf",
        file.display()
      );
      let mut arguments = vec![name, review.conf];
      if let Some(key) = review.key {
        echo = format!(
          "$ pacman-key --add <key> && pacman-key --lsign-key {}\n{echo}",
          key.info.fingerprint
        );
        arguments.extend([key.info.fingerprint, key.hex]);
      }
      privileged(echo, "repo-add", arguments)
    }
    Action::UpgradeAfterRepoAdd => privileged("$ pacman -Syu".into(), "upgrade", vec![]),
    Action::RepoEnable(name) => privileged(
      format!("$ enable [{name}] in /etc/pacman.conf"),
      "repo-enable",
      vec![name],
    ),
    Action::RepoDisable(name) => privileged(
      format!("$ disable [{name}] in /etc/pacman.conf"),
      "repo-disable",
      vec![name],
    ),
    Action::RepoRemove {
      name,
      key,
      delete_key,
    } => {
      let mut echo = format!("$ remove [{name}] from /etc/pacman.conf");
      let mut arguments = vec![name];
      if delete_key && let Some(key) = key {
        echo.push_str(&format!(" && pacman-key --delete {key}"));
        arguments.push("delete-key".into());
      }
      privileged(echo, "repo-remove", arguments)
    }
    Action::ApplyMirrors(content) => {
      privileged("$ update mirrorlist".into(), "mirror-apply", vec![content])
    }
    Action::AurInstall(name) | Action::AurUpgradePackage(name) => aur_command(caps, &["-S", &name]),
    Action::AurUpgrade => aur_command(caps, &["-Sua"]),
  }
}

/// The AUR helper (paru first, then yay) with `args`, run as the normal user.
pub(super) fn aur_command(caps: &Capabilities, args: &[&str]) -> Result<ActionCommand, String> {
  let helper = if caps.has_paru {
    "paru"
  } else if caps.has_yay {
    "yay"
  } else {
    return Err("AUR helper is unavailable".into());
  };
  Ok(ActionCommand::User {
    echo: format!("$ {helper} {}", args.join(" ")),
    process: args
      .iter()
      .fold(ProcessRequest::new(helper), |process, arg| {
        process.arg(*arg)
      }),
  })
}

/// Runs the command of `a`, streaming its output into `live`.
pub(super) fn run_action(
  a: Action,
  caps: Capabilities,
  live: LiveProcess,
) -> Result<String, String> {
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

pub(super) fn generate_mirror_preview(
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

pub(super) fn fetch_reflector_countries(caps: Capabilities) -> Result<Vec<String>, String> {
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

pub(super) fn mirror_preview_summary(content: &str) -> String {
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
pub(super) fn cycled(index: usize, step: i32, len: usize) -> usize {
  (index as i64 + i64::from(step)).rem_euclid(len.max(1) as i64) as usize
}

/// Moves the reflector country by `step`: forward with `→`/Space, back
/// with `←`. Until reflector lists its countries, a short fallback list is
/// used; `None` means every country.
pub(super) fn cycle_country(options: &mut ReflectorOptions, countries: &[String], step: i32) {
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
pub(super) fn cycle_protocol(options: &mut ReflectorOptions, step: i32) {
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
pub(super) fn cycle_sort(options: &mut ReflectorOptions, step: i32) {
  const SORTS: &[&str] = &["rate", "age", "score", "delay", "country"];
  let index = SORTS
    .iter()
    .position(|sort| *sort == options.sort)
    .unwrap_or(0);
  options.sort = SORTS[cycled(index, step, SORTS.len())].into();
}
