//! Implements domain state and models consumed by the UI in crate `argvus control center packages`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackagesPage {
  Home,
  Search,
  Details(usize),
  Installed,
  Updates,
  Orphans,
  Cache,
  /// Read-only list of the files in the package cache, opened from Cache.
  CacheFiles,
  Aur,
  History,
  HistoryDetails(usize),
  Downgrade,
  Mirrors,
  /// Reflector options, opened from Mirrors; generating the preview asks
  /// for confirmation before the mirror list is replaced.
  MirrorEditor,
}
#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// Represents `Package`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct Package {
  pub name: String,
  pub version: String,
  pub repository: Option<String>,
  pub description: String,
  pub installed: bool,
  pub explicit: Option<bool>,
  pub size: Option<u64>,
  pub foreign: bool,
  pub update: Option<String>,
}
#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// Represents `Update`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct Update {
  pub name: String,
  pub current: String,
  pub available: String,
  pub repository: Option<String>,
  pub download_size: Option<u64>,
}
#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// Represents `CachePackage`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct CachePackage {
  pub name: String,
  pub version: String,
  pub path: String,
  pub bytes: u64,
  pub installed: bool,
}
#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// Represents `HistoryEntry`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct HistoryEntry {
  pub timestamp: String,
  pub action: String,
  pub package: String,
  pub old_version: Option<String>,
  pub new_version: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// Represents `PackageDetails`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct PackageDetails {
  pub package: Package,
  pub architecture: Option<String>,
  pub url: Option<String>,
  pub licenses: Vec<String>,
  pub installed_size: Option<u64>,
  pub download_size: Option<u64>,
  pub dependencies: Vec<String>,
  pub optional_dependencies: Vec<String>,
  pub required_by: Vec<String>,
  pub provides: Vec<String>,
  pub conflicts: Vec<String>,
  pub replaces: Vec<String>,
  pub groups: Vec<String>,
  pub install_date: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// Represents `TransactionPlan`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct TransactionPlan {
  pub install: Vec<Package>,
  pub remove: Vec<Package>,
  pub upgrade: Vec<Update>,
  pub replacements: Vec<PackageReplacement>,
  pub decisions: Vec<TransactionDecision>,
  pub download_bytes: u64,
  pub installed_size_delta: Option<i64>,
  pub requires_full_upgrade: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Represents `PackageReplacement`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct PackageReplacement {
  pub removed: Package,
  pub installed: Package,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Defines `TransactionDecision`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub enum TransactionDecision {
  ProviderSelection {
    dependency: String,
    providers: Vec<String>,
  },
  ReplacementConfirmation {
    removed: String,
    replacement: String,
  },
  Conflict {
    kind: ConflictKind,
    details: String,
  },
  SignatureError {
    kind: SignatureErrorKind,
    details: String,
  },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Defines `ConflictKind`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub enum ConflictKind {
  Dependency,
  Package,
  File,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Defines `SignatureErrorKind`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub enum SignatureErrorKind {
  InvalidSignature,
  UnknownKey,
  CorruptPackage,
  Keyring,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Represents `CachePreview`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct CachePreview {
  pub policy: CachePolicy,
  pub candidates: Vec<CachePackage>,
  pub bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Represents `ReflectorOptions`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct ReflectorOptions {
  pub countries: Vec<String>,
  pub protocols: Vec<String>,
  pub age_hours: u32,
  pub count: u32,
  pub sort: String,
}

impl Default for ReflectorOptions {
  /// Executes the `default` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn default() -> Self {
    Self {
      countries: Vec::new(),
      protocols: vec!["https".into()],
      age_hours: 12,
      count: 10,
      sort: "rate".into(),
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Defines `PackageFilter`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub enum PackageFilter {
  All,
  Explicit,
  Dependency,
  Foreign,
  Updates,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Defines `CachePolicy`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub enum CachePolicy {
  KeepThree,
  KeepOne,
  Uninstalled,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// Represents `Mirror`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct Mirror {
  pub server: String,
  pub protocol: String,
  pub enabled: bool,
  pub order: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// Represents `AurPackage`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct AurPackage {
  pub name: String,
  pub version: String,
  pub description: String,
  pub installed: bool,
  pub votes: Option<u64>,
  pub popularity: Option<String>,
}

/// Aggregate counters shown on the Packages home dashboard, mirroring the
/// snapshot collected by the Boot and Storage home screens.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PackageDashboard {
  pub installed_count: usize,
  pub update_count: usize,
  pub orphan_count: usize,
  pub cache_count: usize,
  pub cache_bytes: u64,
  pub history_count: usize,
  pub mirrors_total: usize,
  pub mirrors_enabled: usize,
  pub available_count: usize,
  pub aur_helper: Option<String>,
  /// Real failure reason instead of silently masking a query as zero.
  pub error: Option<String>,
}
