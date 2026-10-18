//! Implements isolated integration with system tools and APIs in crate `argvus control center packages`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use crate::model::*;
use argvus_control_center_core::{
  capabilities::{Capabilities, resolve_executable},
  process::{ProcessError, ProcessRequest, ProcessRunner},
  sanitize::terminal_text,
};
use std::{
  fs,
  path::{Path, PathBuf},
  time::Duration,
};
use thiserror::Error;
#[derive(Debug, Error)]
/// Defines `PackageError`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
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
}
/// Represents `PackageBackend`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct PackageBackend<R> {
  pub runner: R,
  pub capabilities: Capabilities,
  /// Absolute path to pacman so it keeps working even when the desktop
  /// session stripped `/usr/bin` from the inherited `PATH` (see
  /// `resolve_executable` in capabilities.rs).
  pacman: String,
}
impl<R: ProcessRunner> PackageBackend<R> {
  /// Constructs `new` with this module's expected initial state. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
  /// Executes the `run` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
  /// Executes the `run_plan` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
      .timeout(Duration::from_secs(60));
    let output = self.runner.run(&request)?;
    if output.timed_out {
      return Err(PackageError::Command(
        "transaction planning timed out".into(),
      ));
    }
    if output.status != Some(0) {
      return Err(PackageError::Command(terminal_text(
        &String::from_utf8_lossy(&output.stderr),
      )));
    }
    Ok(terminal_text(&String::from_utf8_lossy(&output.stdout)))
  }
  /// Executes the `plan_install` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn plan_install(&self, packages: &[String]) -> Result<TransactionPlan, PackageError> {
    validate_package_names(packages)?;
    let mut args = vec![
      "-S".into(),
      "--print".into(),
      "--needed".into(),
      "--print-format".into(),
      "%n\\t%v\\t%s".into(),
    ];
    args.extend(packages.iter().cloned());
    Ok(parse_transaction_plan(&self.run_plan(&args)?, true))
  }
  /// Executes the `plan_remove` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn plan_remove(&self, packages: &[String]) -> Result<TransactionPlan, PackageError> {
    validate_package_names(packages)?;
    let mut args = vec![
      "-R".into(),
      "--print".into(),
      "--print-format".into(),
      "%n\\t%v\\t%s".into(),
    ];
    args.extend(packages.iter().cloned());
    Ok(parse_transaction_plan(&self.run_plan(&args)?, false))
  }
  /// Executes the `plan_upgrade` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn plan_upgrade(&self) -> Result<TransactionPlan, PackageError> {
    Ok(parse_transaction_plan(
      &self.run_plan(&[
        "-Syu".into(),
        "--print".into(),
        "--print-format".into(),
        "%n\\t%v\\t%s".into(),
      ])?,
      true,
    ))
  }
  /// Executes the `installed` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
  /// Applies the `updates` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn updates(&self) -> Result<Vec<Update>, PackageError> {
    let text = self.run(&["-Qu"])?;
    Ok(parse_updates(&text))
  }
  /// Executes the `search` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn search(&self, query: &str) -> Result<Vec<Package>, PackageError> {
    validate_search(query)?;
    let text = self.run(&["-Ss", query])?;
    Ok(parse_search(&text))
  }
  /// Executes the `details` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn details(&self, name: &str) -> Result<PackageDetails, PackageError> {
    validate_package_name(name)?;
    let text = self
      .run(&["-Qi", name])
      .or_else(|_| self.run(&["-Si", name]))?;
    parse_details(&text)
      .ok_or_else(|| PackageError::Command("package metadata was incomplete".into()))
  }
  /// Executes the `orphans` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
  /// Executes the `history` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn history(&self) -> Result<Vec<HistoryEntry>, PackageError> {
    let text = fs::read_to_string("/var/log/pacman.log")
      .map_err(|e| PackageError::Command(e.to_string()))?;
    Ok(parse_history(&text))
  }
  /// Executes the `cache` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn cache(&self) -> Vec<CachePackage> {
    cache_entries(&cache_dir())
  }
  /// Executes the `mirrors` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn mirrors(&self) -> Vec<Mirror> {
    parse_mirrors(
      &mirrorlist_path()
        .and_then(|path| fs::read_to_string(path).ok())
        .unwrap_or_default(),
    )
  }
  /// Executes the `aur_helper` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
  /// Executes the `aur_search` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
/// Executes the `settle_count` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn settle_count(
  name: &str,
  result: Result<Result<usize, PackageError>, Box<dyn std::any::Any + Send>>,
  slot: &mut usize,
  error: &mut Option<String>,
) {
  match result {
    Ok(Ok(value)) => *slot = value,
    Ok(Err(failure)) => {
      if error.is_none() {
        *error = Some(failure.to_string());
      }
    }
    Err(_) => {
      if error.is_none() {
        *error = Some(format!("dashboard {name} query panicked"));
      }
    }
  }
}
/// Executes the `validate_package_name` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn validate_package_name(v: &str) -> Result<(), PackageError> {
  if v.is_empty()
    || v.len() > 128
    || v
      .chars()
      .any(|c| !c.is_ascii_alphanumeric() && !matches!(c, '@' | '.' | '_' | '+' | '-'))
  {
    Err(PackageError::InvalidPackageName)
  } else {
    Ok(())
  }
}
/// Executes the `validate_search` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn validate_search(v: &str) -> Result<(), PackageError> {
  if v.trim().is_empty()
    || v.len() > 128
    || v
      .chars()
      .any(|c| c.is_control() || matches!(c, '\'' | '"' | ';' | '|' | '&'))
  {
    Err(PackageError::InvalidPackageName)
  } else {
    Ok(())
  }
}
/// Executes the `validate_aur_search` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn validate_aur_search(v: &str) -> Result<(), PackageError> {
  validate_search(v)?;
  if v.trim().chars().count() < 2 {
    Err(PackageError::InvalidPackageName)
  } else {
    Ok(())
  }
}
/// Executes the `validate_package_names` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn validate_package_names(values: &[String]) -> Result<(), PackageError> {
  if values.is_empty() {
    return Err(PackageError::InvalidPackageName);
  }
  values
    .iter()
    .try_for_each(|value| validate_package_name(value))
}
/// Converts input data into `parse_installed` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn parse_installed(input: &str) -> Vec<Package> {
  input
    .lines()
    .filter_map(|l| {
      let mut fields = l.split_whitespace();
      let n = fields.next()?;
      let v = fields.next()?;
      if fields.next().is_some() {
        return None;
      }
      Some(Package {
        name: terminal_text(n),
        version: terminal_text(v),
        installed: true,
        foreign: false,
        ..Default::default()
      })
    })
    .collect()
}
/// Converts input data into `parse_updates` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn parse_updates(input: &str) -> Vec<Update> {
  input
    .lines()
    .filter_map(|l| {
      let (left, right) = l.split_once(" -> ")?;
      let mut p = left.split_whitespace();
      let name = p.next()?.to_string();
      let current = p.next()?.to_string();
      let mut q = right.split_whitespace();
      Some(Update {
        name,
        current,
        available: q.next()?.to_string(),
        repository: None,
        download_size: None,
      })
    })
    .collect()
}
/// Converts input data into `parse_transaction_plan` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn parse_transaction_plan(input: &str, install: bool) -> TransactionPlan {
  let mut plan = TransactionPlan {
    requires_full_upgrade: install,
    ..Default::default()
  };
  for line in input.lines() {
    let mut fields = line.split('\t');
    let Some(name) = fields.next().map(str::trim).filter(|v| !v.is_empty()) else {
      continue;
    };
    let version = fields.next().unwrap_or_default().trim().to_owned();
    let size = fields
      .next()
      .and_then(|value| value.trim().parse::<u64>().ok());
    let package = Package {
      name: terminal_text(name),
      version: terminal_text(&version),
      installed: !install,
      size,
      ..Default::default()
    };
    if install {
      plan.download_bytes = plan.download_bytes.saturating_add(size.unwrap_or_default());
      plan.install.push(package);
    } else {
      plan.remove.push(package);
    }
  }
  plan
}

/// Classifies pacman's stable error vocabulary without attempting to answer a
/// prompt. The caller must keep the operation aborted until a corresponding
/// decision is explicitly modelled by the UI.
pub fn classify_pacman_failure(stderr: &str) -> Option<TransactionDecision> {
  let text = terminal_text(stderr);
  let lower = text.to_ascii_lowercase();
  if lower.contains("conflicting files") || lower.contains("exists in filesystem") {
    return Some(TransactionDecision::Conflict {
      kind: ConflictKind::File,
      details: text,
    });
  }
  if lower.contains("conflicts with") || lower.contains("conflicting packages") {
    return Some(TransactionDecision::Conflict {
      kind: ConflictKind::Package,
      details: text,
    });
  }
  if lower.contains("failed to satisfy a dependency")
    || lower.contains("could not satisfy dependencies")
  {
    return Some(TransactionDecision::Conflict {
      kind: ConflictKind::Dependency,
      details: text,
    });
  }
  let kind = if lower.contains("unknown trust") || lower.contains("unknown key") {
    SignatureErrorKind::UnknownKey
  } else if lower.contains("invalid or corrupted package") || lower.contains("corrupted package") {
    SignatureErrorKind::CorruptPackage
  } else if lower.contains("invalid signature") || lower.contains("signature from") {
    SignatureErrorKind::InvalidSignature
  } else if lower.contains("keyring") || lower.contains("key could not be looked up") {
    SignatureErrorKind::Keyring
  } else {
    return None;
  };
  Some(TransactionDecision::SignatureError {
    kind,
    details: text,
  })
}

/// Executes the `validate_reflector_options` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn validate_reflector_options(options: &ReflectorOptions) -> Result<(), PackageError> {
  if options.age_hours == 0
    || options.age_hours > 24 * 365
    || options.count == 0
    || options.count > 100
  {
    return Err(PackageError::InvalidReflectorOption);
  }
  if options.countries.iter().any(|country| {
    country.is_empty()
      || country.len() > 64
      || country
        .chars()
        .any(|value| !value.is_ascii_alphabetic() && value != ' ' && value != '-')
  }) {
    return Err(PackageError::InvalidReflectorOption);
  }
  if options
    .protocols
    .iter()
    .any(|protocol| !matches!(protocol.as_str(), "http" | "https" | "rsync"))
    || !matches!(
      options.sort.as_str(),
      "rate" | "age" | "score" | "delay" | "country"
    )
  {
    return Err(PackageError::InvalidReflectorOption);
  }
  Ok(())
}

/// Executes the `reflector_args` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn reflector_args(options: &ReflectorOptions) -> Result<Vec<String>, PackageError> {
  validate_reflector_options(options)?;
  let mut args = Vec::new();
  if !options.countries.is_empty() {
    args.push("--country".into());
    args.push(options.countries.join(","));
  }
  if !options.protocols.is_empty() {
    args.push("--protocol".into());
    args.push(options.protocols.join(","));
  }
  args.push("--age".into());
  args.push(options.age_hours.to_string());
  args.push("--latest".into());
  args.push(options.count.to_string());
  args.push("--sort".into());
  args.push(options.sort.clone());
  Ok(args)
}

/// Parses the fixed-width table emitted by `reflector --list-countries`:
/// `Country  Code  Count` with the country name kept whole, so multi-word
/// names such as "United States" survive the parse.
pub fn parse_reflector_countries(input: &str) -> Vec<String> {
  let mut countries = Vec::new();
  for line in input.lines() {
    let mut tokens: Vec<&str> = line.split_whitespace().collect();
    let Some(count) = tokens.pop() else {
      continue;
    };
    let Some(code) = tokens.pop() else {
      continue;
    };
    if code.len() != 2
      || !code.chars().all(|value| value.is_ascii_uppercase())
      || !count.chars().all(|value| value.is_ascii_digit())
    {
      continue;
    }
    let name = tokens.join(" ");
    if !name.is_empty() {
      countries.push(name);
    }
  }
  countries
}

/// Computes the exact candidate set for the three supported paccache
/// policies. It is intentionally independent of filesystem mutation so the
/// result can be shown before confirmation and recalculated immediately
/// before execution.
pub fn cache_preview(mut entries: Vec<CachePackage>, policy: CachePolicy) -> CachePreview {
  entries.sort_by(|a, b| a.name.cmp(&b.name).then(a.version.cmp(&b.version)));
  let mut candidates = Vec::new();
  let mut groups: std::collections::BTreeMap<String, Vec<CachePackage>> =
    std::collections::BTreeMap::new();
  for entry in entries {
    groups.entry(entry.name.clone()).or_default().push(entry);
  }
  for (_name, mut versions) in groups {
    versions.sort_by(|a, b| b.version.cmp(&a.version));
    let keep = match policy {
      CachePolicy::KeepThree => 3,
      CachePolicy::KeepOne => 1,
      CachePolicy::Uninstalled => 0,
    };
    for (index, entry) in versions.into_iter().enumerate() {
      if (policy == CachePolicy::Uninstalled && entry.installed) || index < keep {
        continue;
      }
      candidates.push(entry);
    }
  }
  let bytes = candidates.iter().map(|entry| entry.bytes).sum();
  CachePreview {
    policy,
    candidates,
    bytes,
  }
}

/// Parses the metadata fields emitted by pacman `-Qp --print-format`. The
/// format is deliberately kept separate from filename parsing because Arch
/// package names and versions may contain hyphens.
pub fn parse_cached_metadata(input: &str) -> Option<(String, String, String)> {
  let mut fields = input.trim().split('\t');
  let name = fields.next()?.trim();
  let version = fields.next()?.trim();
  let arch = fields.next()?.trim();
  if name.is_empty() || version.is_empty() || arch.is_empty() {
    return None;
  }
  validate_package_name(name).ok()?;
  Some((name.into(), version.into(), arch.into()))
}

/// Executes the `cached_versions` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn cached_versions(package: &str, entries: &[CachePackage]) -> Vec<CachePackage> {
  entries
    .iter()
    .filter(|entry| entry.name == package)
    .cloned()
    .collect()
}
/// Converts input data into `parse_search` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn parse_search(input: &str) -> Vec<Package> {
  let mut out: Vec<Package> = Vec::new();
  for line in input.lines() {
    if line.starts_with(char::is_whitespace) {
      if let Some(package) = out.last_mut() {
        let description = terminal_text(line.trim());
        if !description.is_empty() {
          package.description = description;
        }
      }
      continue;
    }
    let mut fields = line.split_whitespace();
    let Some(repo_name) = fields.next() else {
      continue;
    };
    let Some((repo, name)) = repo_name.split_once('/') else {
      continue;
    };
    let Some(version) = fields.next() else {
      continue;
    };
    out.push(Package {
      name: terminal_text(name),
      version: terminal_text(version),
      repository: Some(terminal_text(repo)),
      installed: fields.any(|field| field == "[installed]"),
      ..Default::default()
    })
  }
  out
}
/// Converts input data into `parse_history` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn parse_history(input: &str) -> Vec<HistoryEntry> {
  input
    .lines()
    .filter_map(|l| {
      let (timestamp, rest) = l.split_once(']')?;
      let mut p = rest.split_whitespace();
      let action = p.next()?.trim_matches(&['[', ']', '*'][..]).to_string();
      let package = p.next()?.to_string();
      let versions = rest
        .split_once('(')
        .and_then(|(_, value)| value.strip_suffix(')'));
      let (old_version, new_version) = versions
        .and_then(|v| v.split_once(" -> "))
        .map(|(a, b)| (Some(a.trim().to_owned()), Some(b.trim().to_owned())))
        .unwrap_or((None, None));
      Some(HistoryEntry {
        timestamp: timestamp.trim_start_matches('[').into(),
        action,
        package,
        old_version,
        new_version,
      })
    })
    .collect()
}
/// Executes the `cache_entries` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn cache_entries(dir: &Path) -> Vec<CachePackage> {
  fs::read_dir(dir)
    .ok()
    .into_iter()
    .flatten()
    .filter_map(Result::ok)
    .filter_map(|e| {
      let m = e.metadata().ok()?;
      let n = e.file_name().to_string_lossy().into_owned();
      (n.ends_with(".pkg.tar.zst") || n.ends_with(".pkg.tar.xz") || n.ends_with(".pkg.tar.gz"))
        .then(|| CachePackage {
          name: n.clone(),
          version: String::new(),
          path: e.path().to_string_lossy().into_owned(),
          bytes: m.len(),
          installed: false,
        })
    })
    .collect()
}

/// Converts input data into `parse_details` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn parse_details(input: &str) -> Option<PackageDetails> {
  let mut values = std::collections::BTreeMap::<String, String>::new();
  for line in input.lines() {
    if let Some((key, value)) = line.split_once(':') {
      let key = match key.trim() {
        "Nome" | "Name" => "Name",
        "Versão" | "Version" => "Version",
        "Descrição" | "Description" => "Description",
        "Arquitetura" | "Architecture" => "Architecture",
        "Repositório" | "Repository" => "Repository",
        "URL" => "URL",
        "Licenças" | "Licenses" => "Licenses",
        "Tamanho instalado" | "Installed Size" => "Installed Size",
        "Tamanho de download" | "Download Size" => "Download Size",
        "Depende de" | "Depends On" => "Depends On",
        "Depend. opcionais" | "Optional Deps" => "Optional Deps",
        "Necessário para" | "Required By" => "Required By",
        "Provê" | "Provides" => "Provides",
        "Conflita com" | "Conflicts With" => "Conflicts With",
        "Substitui" | "Replaces" => "Replaces",
        "Grupos" | "Optional For" => "Optional For",
        "Data de instalação" | "Install Date" => "Install Date",
        other => other,
      };
      values.insert(key.into(), terminal_text(value.trim()));
    }
  }
  let name = values.get("Name")?.clone();
  let package = Package {
    name,
    version: values.get("Version").cloned().unwrap_or_default(),
    repository: values.get("Repository").cloned(),
    description: values.get("Description").cloned().unwrap_or_default(),
    installed: values.contains_key("Install Date"),
    ..Default::default()
  };
  let list = |key: &str| {
    values
      .get(key)
      .map(|v| v.split_whitespace().map(str::to_owned).collect())
      .unwrap_or_default()
  };
  Some(PackageDetails {
    package,
    architecture: values.get("Architecture").cloned(),
    url: values.get("URL").cloned(),
    licenses: list("Licenses"),
    installed_size: parse_size(values.get("Installed Size")),
    download_size: parse_size(values.get("Download Size")),
    dependencies: list("Depends On"),
    optional_dependencies: list("Optional Deps"),
    required_by: list("Required By"),
    provides: list("Provides"),
    conflicts: list("Conflicts With"),
    replaces: list("Replaces").into_iter().collect(),
    groups: list("Optional For").into_iter().collect(),
    install_date: values.get("Install Date").cloned(),
  })
}
/// Converts input data into `parse_size` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn parse_size(value: Option<&String>) -> Option<u64> {
  value
    .and_then(|v| v.split_whitespace().next()?.parse::<f64>().ok())
    .map(|v| v as u64)
}
/// Executes the `cache_dir` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn cache_dir() -> PathBuf {
  fs::read_to_string("/etc/pacman.conf")
    .ok()
    .and_then(|text| {
      text
        .lines()
        .find_map(|line| line.trim().strip_prefix("CacheDir = ").map(PathBuf::from))
    })
    .unwrap_or_else(|| PathBuf::from("/var/cache/pacman/pkg"))
}
/// Executes the `mirrorlist_path` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn mirrorlist_path() -> Option<PathBuf> {
  let config = fs::read_to_string("/etc/pacman.conf").ok()?;
  config
    .lines()
    .find_map(|line| {
      let value = line.trim().strip_prefix("Include = ")?;
      value.contains("mirrorlist").then(|| PathBuf::from(value))
    })
    .or_else(|| Some(PathBuf::from("/etc/pacman.d/mirrorlist")))
}
/// Converts input data into `parse_mirrors` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn parse_mirrors(input: &str) -> Vec<Mirror> {
  input
    .lines()
    .enumerate()
    .filter_map(|(index, line)| {
      let trimmed = line.trim();
      let enabled = !trimmed.starts_with('#');
      let server = trimmed
        .trim_start_matches('#')
        .trim()
        .strip_prefix("Server = ")?
        .trim()
        .to_owned();
      let protocol = server.split("://").next().unwrap_or("unknown").to_owned();
      Some(Mirror {
        server,
        protocol,
        enabled,
        order: index + 1,
      })
    })
    .collect()
}
#[cfg(test)]
mod tests {
  use super::*;
  use argvus_control_center_core::process::{ProcessOutput, ProcessRequest};
  use std::sync::Mutex;

  #[derive(Default)]
  /// Represents `InstalledRunner`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
  struct InstalledRunner(Mutex<Vec<ProcessRequest>>);
  impl ProcessRunner for InstalledRunner {
    /// Executes the `run` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
    fn run(&self, request: &ProcessRequest) -> Result<ProcessOutput, ProcessError> {
      self.0.lock().unwrap().push(request.clone());
      let stdout = if request.args == ["-Q"] {
        "linux 6.17.1.arch1-1\nargvus-control-center 0.1.0-1\n"
      } else if request.args == ["-Qmq"] {
        "argvus-control-center\n"
      } else {
        ""
      };
      Ok(ProcessOutput {
        stdout: stdout.as_bytes().to_vec(),
        stderr: Vec::new(),
        status: Some(0),
        timed_out: false,
      })
    }
  }
  #[test]
  /// Executes the `dashboard_reports_pacman_unavailable_instead_of_silent_zeros` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn dashboard_reports_pacman_unavailable_instead_of_silent_zeros() {
    let backend = PackageBackend::new(InstalledRunner::default(), Capabilities::default());
    let dashboard = backend.dashboard();
    assert_eq!(dashboard.installed_count, 0);
    assert!(
      dashboard
        .error
        .as_deref()
        .unwrap_or_default()
        .contains("pacman"),
      "{}",
      dashboard.error.as_deref().unwrap_or_default()
    );
  }
  #[test]
  /// Converts input data into `parses_installed` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn parses_installed() {
    let x = parse_installed("linux 6.1\nfoo 1-2\nrust 1:1.95.0-1");
    assert_eq!(x[0].name, "linux");
    assert_eq!(x[2].version, "1:1.95.0-1");
  }
  #[test]
  /// Executes the `dashboard_reports_query_failures_instead_of_silent_zeros` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn dashboard_reports_query_failures_instead_of_silent_zeros() {
    /// Represents `FailingRunner`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
    struct FailingRunner;
    impl ProcessRunner for FailingRunner {
      /// Executes the `run` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
      fn run(&self, _: &ProcessRequest) -> Result<ProcessOutput, ProcessError> {
        Err(ProcessError::EmptyProgram)
      }
    }
    let backend = PackageBackend::new(
      FailingRunner,
      Capabilities {
        has_pacman: true,
        ..Default::default()
      },
    );
    let dashboard = backend.dashboard();
    assert!(dashboard.error.is_some());
  }
  #[test]
  /// Executes the `installed_uses_two_compatible_queries_without_n_plus_one` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn installed_uses_two_compatible_queries_without_n_plus_one() {
    let capabilities = Capabilities {
      has_pacman: true,
      ..Default::default()
    };
    let backend = PackageBackend::new(InstalledRunner::default(), capabilities);
    let packages = backend.installed().unwrap();
    assert_eq!(packages.len(), 2);
    assert!(packages[1].foreign);
    let requests = backend.runner.0.lock().unwrap();
    assert_eq!(requests.len(), 2);
    let mut args = requests
      .iter()
      .map(|request| request.args.clone())
      .collect::<Vec<_>>();
    args.sort();
    assert_eq!(args, [vec![String::from("-Q")], vec![String::from("-Qmq")]]);
  }
  #[test]
  /// Converts input data into `parses_updates` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn parses_updates() {
    let x = parse_updates("linux 6.1 -> 6.2");
    assert_eq!(x[0].available, "6.2");
  }
  #[test]
  /// Converts input data into `parses_real_pacman_search_shape` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn parses_real_pacman_search_shape() {
    let packages = parse_search(
      "extra/firefox 155.0.1-1 [installed]\n    Fast, Private & Safe Web Browser\nextra/firefox-developer-edition 156.0b1-1\n    Developer Edition of the popular browser\n",
    );
    assert_eq!(packages.len(), 2);
    assert_eq!(packages[0].name, "firefox");
    assert_eq!(packages[0].repository.as_deref(), Some("extra"));
    assert!(packages[0].installed);
    assert_eq!(packages[0].description, "Fast, Private & Safe Web Browser");
  }
  #[test]
  /// Converts input data into `parses_localized_pacman_details_with_variable_spacing` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn parses_localized_pacman_details_with_variable_spacing() {
    let details = parse_details(
      "Nome                 : pacman\nVersão               : 7.1.0-1\nDescrição            : Package manager\nArquitetura           : x86_64\n",
    )
    .expect("metadata should be parsed");
    assert_eq!(details.package.name, "pacman");
    assert_eq!(details.package.version, "7.1.0-1");
    assert_eq!(details.architecture.as_deref(), Some("x86_64"));
  }
  #[test]
  /// Retrieves data for `detects_installed_via_localized_normalized_detail_field` without mixing collection with TUI rendering. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn detects_installed_via_localized_normalized_detail_field() {
    let installed = parse_details(
      "Nome                 : pacman\nVersão               : 7.1.0-1\nRepositório           : core\nData de instalação    : ter 01 jan 2020 00:00:00\n",
    )
    .expect("metadata should be parsed");
    assert!(installed.package.installed);
    let not_installed = parse_details(
      "Nome                 : pacman\nVersão               : 7.1.0-1\nRepositório           : core\n",
    )
    .expect("metadata should be parsed");
    assert!(!not_installed.package.installed);
  }
  #[test]
  /// Executes the `rejects_shell_input` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn rejects_shell_input() {
    assert!(validate_package_name("foo;bar").is_err());
  }
  #[test]
  /// Executes the `history_is_structured` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn history_is_structured() {
    let x = parse_history("[2026-09-10T10:00] [ALPM] upgraded linux (1 -> 2)");
    assert_eq!(x[0].action, "ALPM");
    assert_eq!(x[0].old_version.as_deref(), Some("1"));
    assert_eq!(x[0].new_version.as_deref(), Some("2"));
  }
  #[test]
  /// Executes the `mirror_parser_keeps_order_and_enabled_state` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn mirror_parser_keeps_order_and_enabled_state() {
    let mirrors = parse_mirrors(
      "#Server = https://old.example/$repo/os/$arch\nServer = https://new.example/$repo/os/$arch\n",
    );
    assert_eq!(mirrors.len(), 2);
    assert!(!mirrors[0].enabled);
    assert_eq!(mirrors[1].protocol, "https");
    assert_eq!(mirrors[1].order, 2);
  }
  #[test]
  /// Converts input data into `parses_real_reflector_country_table_with_multi_word_names` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn parses_real_reflector_country_table_with_multi_word_names() {
    let countries = parse_reflector_countries(
      "Country              Code Count\n-------------------- ---- -----\nAlbania                AL     1\nHong Kong              HK    11\nNew Zealand            NZ     8\nUnited States          US   196\n",
    );
    assert_eq!(
      countries,
      ["Albania", "Hong Kong", "New Zealand", "United States"]
    );
  }
  #[test]
  /// Executes the `rejects_code_and_header_lines_from_country_table` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn rejects_code_and_header_lines_from_country_table() {
    assert!(parse_reflector_countries("Country  Code  Count\n---- ---- -----\n").is_empty());
    assert!(parse_reflector_countries("Albania AL\n").is_empty());
  }
  #[test]
  /// Executes the `package_names_allow_arch_names_but_reject_shell_syntax` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn package_names_allow_arch_names_but_reject_shell_syntax() {
    assert!(validate_package_name("foo-bar@1").is_ok());
    assert!(validate_package_name("foo/bar").is_err());
    assert!(validate_search("mesa graphics").is_ok());
    assert!(validate_search("foo\nbar").is_err());
  }
  #[test]
  /// Executes the `transaction_plan_preserves_targets_and_sizes_without_running_a_transaction` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn transaction_plan_preserves_targets_and_sizes_without_running_a_transaction() {
    let plan = parse_transaction_plan("foo-bar\t2.0-1\t1024\nlibfoo\t1.0-2\t2048\n", true);
    assert_eq!(plan.install.len(), 2);
    assert_eq!(plan.download_bytes, 3072);
    assert!(plan.requires_full_upgrade);
    let removal = parse_transaction_plan("foo-bar\t2.0-1\t0\n", false);
    assert_eq!(removal.remove[0].name, "foo-bar");
    assert!(!removal.requires_full_upgrade);
  }

  #[test]
  /// Executes the `classifies_conflicts_and_signature_failures_without_bypassing_them` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn classifies_conflicts_and_signature_failures_without_bypassing_them() {
    assert!(matches!(
      classify_pacman_failure("error: conflicting files: /usr/bin/foo exists in filesystem"),
      Some(TransactionDecision::Conflict {
        kind: ConflictKind::File,
        ..
      })
    ));
    assert!(matches!(
      classify_pacman_failure("error: signature from key is unknown trust"),
      Some(TransactionDecision::SignatureError {
        kind: SignatureErrorKind::UnknownKey,
        ..
      })
    ));
  }

  #[test]
  /// Executes the `reflector_options_are_typed_and_shell_safe` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn reflector_options_are_typed_and_shell_safe() {
    let options = ReflectorOptions {
      countries: vec!["Brazil".into(), "United States".into()],
      protocols: vec!["https".into()],
      age_hours: 12,
      count: 10,
      sort: "rate".into(),
    };
    let args = reflector_args(&options).unwrap();
    assert!(
      args
        .windows(2)
        .any(|pair| pair == ["--country", "Brazil,United States"])
    );
    assert!(!args.iter().any(|arg| arg.contains(';')));
    assert!(
      validate_reflector_options(&ReflectorOptions {
        sort: "arbitrary-command".into(),
        ..options
      })
      .is_err()
    );
  }

  #[test]
  /// Executes the `cache_preview_matches_policy_and_counts_real_bytes` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn cache_preview_matches_policy_and_counts_real_bytes() {
    let entries = vec![
      CachePackage {
        name: "foo".into(),
        version: "3".into(),
        bytes: 30,
        installed: true,
        ..Default::default()
      },
      CachePackage {
        name: "foo".into(),
        version: "2".into(),
        bytes: 20,
        ..Default::default()
      },
      CachePackage {
        name: "foo".into(),
        version: "1".into(),
        bytes: 10,
        ..Default::default()
      },
    ];
    let preview = cache_preview(entries, CachePolicy::KeepOne);
    assert_eq!(preview.candidates.len(), 2);
    assert_eq!(preview.bytes, 30);
  }

  #[test]
  /// Executes the `cached_metadata_does_not_depend_on_hyphen_splitting` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn cached_metadata_does_not_depend_on_hyphen_splitting() {
    assert_eq!(
      parse_cached_metadata("foo-bar-baz\t2:1.4-3\tx86_64"),
      Some(("foo-bar-baz".into(), "2:1.4-3".into(), "x86_64".into()))
    );
    assert!(parse_cached_metadata("foo-bar\t\tx86_64").is_none());
  }
}
