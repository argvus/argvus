//! Domain state and models consumed by the UI of the Boot page.
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
pub enum BootloaderKind {
  SystemdBoot,
  Grub,
  #[default]
  Unknown,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
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
pub struct BootloaderInfo {
  pub version: Option<String>,
  pub default_entry: Option<String>,
  pub timeout: Option<u32>,
  pub entries: Vec<BootEntry>,
  pub grub_values: Vec<(String, String)>,
}
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BootEntry {
  pub id: String,
  pub title: String,
  pub linux: Option<String>,
  pub initrd: Vec<String>,
  pub options: Option<String>,
  pub is_default: bool,
}
#[derive(Debug, Clone, Default, PartialEq, Eq)]
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
pub struct PlymouthInfo {
  pub installed: bool,
  pub current_theme: Option<String>,
  pub themes: Vec<String>,
}
