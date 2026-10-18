//! Implements domain state and models consumed by the UI in crate `argvus control center boot`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootPage {
  Home,
  Summary,
  Kernel,
  KernelDetail(usize),
  Bootloader,
  BootloaderDetail(usize),
  Initramfs,
  InitramfsDetail(usize),
  Plymouth,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// Defines `BootloaderKind`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub enum BootloaderKind {
  SystemdBoot,
  Grub,
  #[default]
  Unknown,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// Represents `BootSnapshot`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct BootSnapshot {
  pub firmware: String,
  pub bootloader: BootloaderKind,
  pub esp: Option<String>,
  pub current_kernel: String,
  pub kernels: Vec<KernelInfo>,
  pub bootloader_info: BootloaderInfo,
  pub initramfs: InitramfsInfo,
  pub plymouth: PlymouthInfo,
  pub secure_boot: Option<bool>,
}
#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// Represents `KernelInfo`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct KernelInfo {
  pub package: String,
  pub version: String,
  pub current: bool,
  pub headers: bool,
  pub image: Option<String>,
  pub initramfs: Option<String>,
  pub fallback: Option<String>,
  pub preset: Option<String>,
  pub default: bool,
  pub uki: Option<String>,
}
#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// Represents `BootloaderInfo`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct BootloaderInfo {
  pub version: Option<String>,
  pub default_entry: Option<String>,
  pub timeout: Option<u32>,
  pub entries: Vec<BootEntry>,
  pub grub_values: Vec<(String, String)>,
}
#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// Represents `BootEntry`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct BootEntry {
  pub id: String,
  pub title: String,
  pub linux: Option<String>,
  pub initrd: Vec<String>,
  pub options: Option<String>,
  pub is_default: bool,
}
#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// Represents `InitramfsInfo`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct InitramfsInfo {
  pub available: bool,
  pub version: Option<String>,
  pub modules: Vec<String>,
  pub binaries: Vec<String>,
  pub files: Vec<String>,
  pub hooks: Vec<String>,
  pub presets: Vec<String>,
  pub config_path: Option<String>,
}
#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// Represents `PlymouthInfo`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct PlymouthInfo {
  pub installed: bool,
  pub current_theme: Option<String>,
  pub themes: Vec<String>,
}
