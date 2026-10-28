//! Parsers of pacman, AUR helper and reflector output, and the classification of pacman failures.

use super::validation::validate_package_name;
use super::*;

pub fn parse_installed(input: &str) -> Vec<Package> {
  input
    .lines()
    .filter_map(|line| {
      let mut fields = line.split_whitespace();
      let package = fields.next()?;
      let version = fields.next()?;
      if fields.next().is_some() {
        return None;
      }
      Some(Package {
        name: terminal_text(package),
        version: terminal_text(version),
        installed: true,
        foreign: false,
        ..Default::default()
      })
    })
    .collect()
}

pub fn parse_updates(input: &str) -> Vec<Update> {
  input
    .lines()
    .filter_map(|line| {
      let (left, right) = line.split_once(" -> ")?;
      let mut current_fields = left.split_whitespace();
      let name = current_fields.next()?.to_string();
      let current = current_fields.next()?.to_string();
      let mut available_fields = right.split_whitespace();
      Some(Update {
        name,
        current,
        available: available_fields.next()?.to_string(),
        repository: None,
        download_size: None,
      })
    })
    .collect()
}

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

/// The installed packages that block removing `targets`, from pacman's
/// English `:: removing X breaks dependency 'Y' required by Z` lines, in
/// order and without repetition.
pub fn parse_breaking_dependents(stderr: &str, targets: &[String]) -> Vec<String> {
  let mut dependents: Vec<String> = Vec::new();
  for line in stderr
    .lines()
    .filter(|line| line.contains("breaks dependency"))
  {
    let Some((_, name)) = line.rsplit_once("required by ") else {
      continue;
    };
    let name = terminal_text(name.trim());
    if !name.is_empty() && !targets.contains(&name) && !dependents.contains(&name) {
      dependents.push(name);
    }
  }
  dependents
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

pub fn cached_versions(package: &str, entries: &[CachePackage]) -> Vec<CachePackage> {
  entries
    .iter()
    .filter(|entry| entry.name == package)
    .cloned()
    .collect()
}

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

pub fn parse_history(input: &str) -> Vec<HistoryEntry> {
  input
    .lines()
    .filter_map(|line| {
      let (timestamp, rest) = line.split_once(']')?;
      let mut words = rest.split_whitespace();
      let action = words.next()?.trim_matches(&['[', ']', '*'][..]).to_string();
      let package = words.next()?.to_string();
      let versions = rest
        .split_once('(')
        .and_then(|(_, value)| value.strip_suffix(')'));
      let (old_version, new_version) = versions
        .and_then(|version_change| version_change.split_once(" -> "))
        .map(|(old, new)| (Some(old.trim().to_owned()), Some(new.trim().to_owned())))
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

pub(super) fn parse_details(input: &str) -> Option<PackageDetails> {
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

pub(super) fn parse_size(value: Option<&String>) -> Option<u64> {
  value
    .and_then(|v| v.split_whitespace().next()?.parse::<f64>().ok())
    .map(|v| v as u64)
}

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
