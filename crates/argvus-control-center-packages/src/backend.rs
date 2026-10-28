//! Isolated integration with system tools and APIs of the Packages page.
use crate::model::*;
use argvus_control_center_core::{
  capabilities::{Capabilities, resolve_executable},
  pacman_conf::{self, KeyInfo, PACMAN_CONF, Repository},
  process::{ProcessError, ProcessRequest, ProcessRunner},
  sanitize::terminal_text,
};
use std::{
  fs,
  path::{Path, PathBuf},
  time::Duration,
};
use thiserror::Error;

mod cache;
mod parsers;
mod validation;

use cache::{cache_dir, write_private_file};
pub use cache::{cache_entries, cache_preview, mirrorlist_path};
use parsers::parse_details;
pub use parsers::{
  cached_versions, classify_pacman_failure, parse_breaking_dependents, parse_cached_metadata,
  parse_history, parse_installed, parse_mirrors, parse_reflector_countries, parse_search,
  parse_transaction_plan, parse_updates,
};
use validation::settle_count;
pub use validation::{
  reflector_args, validate_aur_search, validate_package_name, validate_package_names,
  validate_reflector_options, validate_search,
};
#[derive(Debug, Error)]
pub enum PackageError {
  #[error("pacman is unavailable")]
  BackendUnavailable,
  #[error("process failed: {0}")]
  Process(#[from] ProcessError),
  #[error("pacman command failed: {0}")]
  Command(String),
  #[error("database lock detected")]
  DatabaseLocked,
  #[error("invalid package name")]
  InvalidPackageName,
  #[error("invalid reflector option")]
  InvalidReflectorOption,
  /// Only `https://` addresses are downloaded.
  #[error("only https:// addresses are accepted")]
  NotHttps,
  /// Removing the targets would break these installed packages, which
  /// depend on them; only a cascade removal can take them away together.
  #[error("required by: {}", .0.join(", "))]
  BreaksDependencies(Vec<String>),
}
/// Largest file downloaded for a repository (its configuration or its key).
const MAX_DOWNLOAD_BYTES: usize = 256 * 1024;

/// Fields of each package of a `--print` plan: name, version and size,
/// separated by real tabs (pacman prints the format as given, without
/// expanding a written backslash-t).
const PLAN_FORMAT: &str = "%n\t%v\t%s";

pub struct PackageBackend<R> {
  pub runner: R,
  pub capabilities: Capabilities,
  /// Absolute path to pacman so it keeps working even when the desktop
  /// session stripped `/usr/bin` from the inherited `PATH` (see
  /// `resolve_executable` in capabilities.rs).
  pacman: String,
}
impl<R: ProcessRunner> PackageBackend<R> {
  pub fn new(runner: R, capabilities: Capabilities) -> Self {
    let pacman = resolve_executable("pacman")
      .map(|path| path.to_string_lossy().into_owned())
      .unwrap_or_else(|| "pacman".into());
    Self {
      runner,
      capabilities,
      pacman,
    }
  }
  fn run(&self, args: &[&str]) -> Result<String, PackageError> {
    if !self.capabilities.has_pacman {
      return Err(PackageError::BackendUnavailable);
    }
    let req = args
      .iter()
      .fold(ProcessRequest::new(&self.pacman), |r, a| r.arg(*a))
      .timeout(Duration::from_secs(30));
    let o = self.runner.run(&req)?;
    if o.timed_out {
      return Err(PackageError::Command("timeout".into()));
    }
    if o.status != Some(0) {
      return Err(PackageError::Command(terminal_text(
        &String::from_utf8_lossy(&o.stderr),
      )));
    }
    Ok(terminal_text(&String::from_utf8_lossy(&o.stdout)))
  }
  /// Runs a `--print` transaction plan. pacman answers in English
  /// (`LC_ALL=C`) so its failures can be classified, as
  /// [`classify_pacman_failure`] and [`parse_breaking_dependents`] expect.
  fn run_plan(&self, args: &[String]) -> Result<String, PackageError> {
    if !self.capabilities.has_pacman {
      return Err(PackageError::BackendUnavailable);
    }
    if Path::new("/var/lib/pacman/db.lck").exists() {
      return Err(PackageError::DatabaseLocked);
    }
    let request = args
      .iter()
      .fold(ProcessRequest::new(&self.pacman), |request, argument| {
        request.arg(argument)
      })
      .env("LC_ALL", "C")
      .timeout(Duration::from_secs(60));
    let output = self.runner.run(&request)?;
    if output.timed_out {
      return Err(PackageError::Command(
        "transaction planning timed out".into(),
      ));
    }
    let stdout = terminal_text(&String::from_utf8_lossy(&output.stdout));
    if output.status != Some(0) {
      // pacman prints the `error:` line on stderr and its details (such as
      // `:: removing X breaks dependency 'Y' required by Z`) on stdout.
      let stderr = terminal_text(&String::from_utf8_lossy(&output.stderr));
      let details = [stderr.trim(), stdout.trim()]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
      return Err(PackageError::Command(details));
    }
    Ok(stdout)
  }
  pub fn plan_install(&self, packages: &[String]) -> Result<TransactionPlan, PackageError> {
    validate_package_names(packages)?;
    let mut args = vec![
      "-S".into(),
      "--print".into(),
      "--needed".into(),
      "--print-format".into(),
      PLAN_FORMAT.into(),
    ];
    args.extend(packages.iter().cloned());
    Ok(parse_transaction_plan(&self.run_plan(&args)?, true))
  }
  /// Plans the removal of `packages` with the dependencies nothing else
  /// needs (`-Rs`, as the `-Rns` that runs it). Fails with
  /// [`PackageError::BreaksDependencies`] when other installed packages
  /// depend on them.
  pub fn plan_remove(&self, packages: &[String]) -> Result<TransactionPlan, PackageError> {
    self
      .plan_removal("-Rs", packages)
      .map_err(|error| match error {
        PackageError::Command(stderr) => {
          let dependents = parse_breaking_dependents(&stderr, packages);
          if dependents.is_empty() {
            PackageError::Command(stderr)
          } else {
            PackageError::BreaksDependencies(dependents)
          }
        }
        error => error,
      })
  }
  /// Plans the removal of `packages` together with every package that
  /// depends on them (`-Rcs`, as the `-Rcns` that runs it).
  pub fn plan_remove_cascade(&self, packages: &[String]) -> Result<TransactionPlan, PackageError> {
    self.plan_removal("-Rcs", packages)
  }
  /// A removal plan with the operation flags `flags`.
  fn plan_removal(
    &self,
    flags: &str,
    packages: &[String],
  ) -> Result<TransactionPlan, PackageError> {
    validate_package_names(packages)?;
    let mut args = vec![
      flags.into(),
      "--print".into(),
      "--print-format".into(),
      PLAN_FORMAT.into(),
    ];
    args.extend(packages.iter().cloned());
    Ok(parse_transaction_plan(&self.run_plan(&args)?, false))
  }
  pub fn plan_upgrade(&self) -> Result<TransactionPlan, PackageError> {
    Ok(parse_transaction_plan(
      &self.run_plan(&[
        "-Syu".into(),
        "--print".into(),
        "--print-format".into(),
        PLAN_FORMAT.into(),
      ])?,
      true,
    ))
  }
  pub fn installed(&self) -> Result<Vec<Package>, PackageError> {
    let (installed, foreign) = std::thread::scope(|scope| {
      let installed = scope.spawn(|| self.run(&["-Q"]));
      let foreign = scope.spawn(|| self.run(&["-Qmq"]));
      (installed.join(), foreign.join())
    });
    let text = installed.unwrap_or_else(|_| {
      Err(PackageError::Command(
        "installed query thread panicked".into(),
      ))
    })?;
    let mut packages = parse_installed(&text);
    let foreign: std::collections::BTreeSet<String> = foreign
      .unwrap_or_else(|_| Ok(String::new()))
      .unwrap_or_default()
      .lines()
      .map(str::trim)
      .filter(|name| !name.is_empty())
      .map(str::to_owned)
      .collect();
    for package in &mut packages {
      package.foreign = foreign.contains(&package.name);
    }
    Ok(packages)
  }
  /// Installed packages that no sync repository provides (`pacman -Qm`),
  /// usually built from the AUR. pacman exits with 1 and prints nothing when
  /// there is none, which is an empty list rather than a failure.
  pub fn foreign(&self) -> Result<Vec<Package>, PackageError> {
    let text = match self.run(&["-Qm"]) {
      Err(PackageError::Command(message)) if message.trim().is_empty() => String::new(),
      result => result?,
    };
    let mut packages = parse_installed(&text);
    for package in &mut packages {
      package.foreign = true;
    }
    Ok(packages)
  }
  /// Pending AUR updates reported by `helper -Qua`, run as the normal user.
  /// The helpers exit with 1 and print nothing when every package is current.
  pub fn aur_updates(&self, helper: &str) -> Result<Vec<Update>, PackageError> {
    if !matches!(helper, "paru" | "yay") {
      return Err(PackageError::InvalidPackageName);
    }
    let request = ProcessRequest::new(helper)
      .arg("-Qua")
      .timeout(Duration::from_secs(60));
    let output = self.runner.run(&request)?;
    if output.timed_out {
      return Err(PackageError::Command("timeout".into()));
    }
    let stdout = terminal_text(&String::from_utf8_lossy(&output.stdout));
    let stderr = terminal_text(&String::from_utf8_lossy(&output.stderr));
    match output.status {
      Some(0) => Ok(parse_updates(&stdout)),
      Some(1) if stdout.trim().is_empty() && stderr.trim().is_empty() => Ok(Vec::new()),
      _ => Err(PackageError::Command(stderr)),
    }
  }
  pub fn updates(&self) -> Result<Vec<Update>, PackageError> {
    let text = self.run(&["-Qu"])?;
    Ok(parse_updates(&text))
  }
  pub fn search(&self, query: &str) -> Result<Vec<Package>, PackageError> {
    validate_search(query)?;
    let text = self.run(&["-Ss", query])?;
    Ok(parse_search(&text))
  }
  pub fn details(&self, name: &str) -> Result<PackageDetails, PackageError> {
    validate_package_name(name)?;
    let text = self
      .run(&["-Qi", name])
      .or_else(|_| self.run(&["-Si", name]))?;
    parse_details(&text)
      .ok_or_else(|| PackageError::Command("package metadata was incomplete".into()))
  }
  pub fn orphans(&self) -> Result<Vec<Package>, PackageError> {
    let text = self.run(&["-Qdtq"])?;
    Ok(
      text
        .lines()
        .filter_map(|name| {
          let name = name.trim();
          (!name.is_empty()).then(|| Package {
            name: terminal_text(name),
            installed: true,
            ..Default::default()
          })
        })
        .collect(),
    )
  }
  pub fn history(&self) -> Result<Vec<HistoryEntry>, PackageError> {
    let text = fs::read_to_string("/var/log/pacman.log")
      .map_err(|e| PackageError::Command(e.to_string()))?;
    Ok(parse_history(&text))
  }
  pub fn cache(&self) -> Vec<CachePackage> {
    cache_entries(&cache_dir())
  }
  pub fn mirrors(&self) -> Vec<Mirror> {
    parse_mirrors(
      &mirrorlist_path()
        .and_then(|path| fs::read_to_string(path).ok())
        .unwrap_or_default(),
    )
  }
  /// The unofficial repositories of `/etc/pacman.conf` and the files it
  /// includes, read without running anything.
  pub fn repositories(&self) -> Result<Vec<Repository>, PackageError> {
    let text = fs::read_to_string(PACMAN_CONF)
      .map_err(|error| PackageError::Command(format!("{PACMAN_CONF}: {error}")))?;
    Ok(pacman_conf::list_repositories(&text, |path| {
      fs::read_to_string(path).ok()
    }))
  }
  /// Downloads `url` as the user, following redirects only to https and
  /// refusing files larger than a repository configuration or key.
  pub fn fetch_https(&self, url: &str) -> Result<Vec<u8>, PackageError> {
    if !url.starts_with("https://") || url.chars().any(|c| c.is_whitespace() || c.is_control()) {
      return Err(PackageError::NotHttps);
    }
    let request = ProcessRequest::new("curl")
      .arg("--fail")
      .arg("--silent")
      .arg("--show-error")
      .arg("--location")
      .arg("--proto")
      .arg("=https")
      .arg("--proto-redir")
      .arg("=https")
      .arg("--max-filesize")
      .arg(MAX_DOWNLOAD_BYTES.to_string())
      .arg("--max-time")
      .arg("30")
      .arg("--")
      .arg(url)
      .timeout(Duration::from_secs(40));
    let output = self.runner.run(&request)?;
    if output.timed_out {
      return Err(PackageError::Command("timeout".into()));
    }
    if output.status != Some(0) {
      return Err(PackageError::Command(terminal_text(
        String::from_utf8_lossy(&output.stderr).trim(),
      )));
    }
    if output.stdout.len() > MAX_DOWNLOAD_BYTES {
      return Err(PackageError::Command(format!("{url}: file too large")));
    }
    Ok(output.stdout)
  }
  /// The public keys of `key` (armored or binary), read with
  /// `gpg --show-keys` as the user without importing anything.
  pub fn key_info(&self, key: &[u8]) -> Result<Vec<KeyInfo>, PackageError> {
    let directory = std::env::var_os("XDG_RUNTIME_DIR")
      .map(PathBuf::from)
      .unwrap_or_else(std::env::temp_dir);
    let key_file = directory.join(format!(
      "argvus-repository-key-{}-{}.gpg",
      std::process::id(),
      key.len()
    ));
    write_private_file(&key_file, key)?;
    let request = ProcessRequest::new("gpg")
      .arg("--batch")
      .arg("--no-autostart")
      .arg("--with-colons")
      .arg("--show-keys")
      .arg(key_file.to_string_lossy().into_owned())
      .timeout(Duration::from_secs(30));
    let output = self.runner.run(&request);
    let _ = fs::remove_file(&key_file);
    let output = output?;
    if output.status != Some(0) {
      return Err(PackageError::Command(terminal_text(
        String::from_utf8_lossy(&output.stderr).trim(),
      )));
    }
    Ok(pacman_conf::parse_key_listing(&String::from_utf8_lossy(
      &output.stdout,
    )))
  }
  pub fn aur_helper(&self) -> Option<&'static str> {
    self
      .capabilities
      .has_paru
      .then_some("paru")
      .or_else(|| self.capabilities.has_yay.then_some("yay"))
  }
  /// Collects the aggregate counters shown on the Packages home dashboard.
  /// Independent commands run in parallel so the home screen loads in roughly
  /// the time of the slowest query instead of the sum of all of them. Each
  /// query degrades independently, and a failure (or an unavailable pacman)
  /// is reported through `error` instead of being masked as a silent zero.
  pub fn dashboard(&self) -> PackageDashboard {
    if !self.capabilities.has_pacman {
      return PackageDashboard {
        error: Some("pacman is unavailable (not found in PATH)".into()),
        ..Default::default()
      };
    }
    let cache = self.cache();
    let mirrors = self.mirrors();
    let mut dashboard = PackageDashboard {
      repository_count: self
        .repositories()
        .map(|list| list.len())
        .unwrap_or_default(),
      cache_count: cache.len(),
      cache_bytes: cache.iter().map(|entry| entry.bytes).sum(),
      mirrors_total: mirrors.len(),
      mirrors_enabled: mirrors.iter().filter(|mirror| mirror.enabled).count(),
      aur_helper: self.aur_helper().map(str::to_owned),
      ..Default::default()
    };
    let mut failure: Option<String> = None;
    std::thread::scope(|scope| {
      let installed = scope.spawn(|| self.installed().map(|v| v.len()));
      let foreign = scope.spawn(|| self.foreign().map(|v| v.len()));
      let updates = scope.spawn(|| self.updates().map(|v| v.len()));
      let orphans = scope.spawn(|| self.orphans().map(|v| v.len()));
      let history = scope.spawn(|| self.history().map(|v| v.len()));
      let available = scope.spawn(|| {
        self
          .run(&["-Sl"])
          .map(|text| text.lines().filter(|line| !line.trim().is_empty()).count())
      });
      settle_count(
        "installed",
        installed.join(),
        &mut dashboard.installed_count,
        &mut failure,
      );
      settle_count(
        "foreign",
        foreign.join(),
        &mut dashboard.foreign_count,
        &mut failure,
      );
      settle_count(
        "updates",
        updates.join(),
        &mut dashboard.update_count,
        &mut failure,
      );
      settle_count(
        "orphans",
        orphans.join(),
        &mut dashboard.orphan_count,
        &mut failure,
      );
      settle_count(
        "history",
        history.join(),
        &mut dashboard.history_count,
        &mut failure,
      );
      settle_count(
        "available",
        available.join(),
        &mut dashboard.available_count,
        &mut failure,
      );
    });
    if let Some(failure) = failure {
      dashboard.error = Some(failure);
    }
    dashboard
  }
  pub fn aur_search(&self, helper: &str, query: &str) -> Result<Vec<AurPackage>, PackageError> {
    if !matches!(helper, "paru" | "yay") {
      return Err(PackageError::InvalidPackageName);
    }
    validate_aur_search(query)?;
    let req = ProcessRequest::new(helper)
      .arg("-Ssq")
      .arg(query)
      .timeout(Duration::from_secs(30));
    let output = self.runner.run(&req)?;
    if output.status != Some(0) {
      return Err(PackageError::Command(terminal_text(
        &String::from_utf8_lossy(&output.stderr),
      )));
    }
    Ok(
      output
        .stdout
        .split(|b| *b == b'\n')
        .filter_map(|line| {
          let name = terminal_text(String::from_utf8_lossy(line).trim());
          (!name.is_empty()).then(|| AurPackage {
            name,
            ..Default::default()
          })
        })
        .collect(),
    )
  }
}

#[cfg(test)]
mod tests;
