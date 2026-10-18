//! Implements isolated integration with system tools and APIs in crate `argvus control center boot`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use crate::model::*;
use argvus_control_center_core::{
  capabilities::Capabilities,
  process::{ProcessError, ProcessRequest, ProcessRunner},
  sanitize::terminal_text,
};
use std::{
  collections::BTreeMap,
  fs,
  path::{Path, PathBuf},
  time::Duration,
};
use thiserror::Error;

#[derive(Debug, Error)]
/// Defines `BootError`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub enum BootError {
  #[error("boot information unavailable")]
  Unavailable,
  #[error("process failed: {0}")]
  Process(#[from] ProcessError),
  #[error("invalid boot configuration")]
  InvalidConfig,
  #[error("command failed: {0}")]
  Command(String),
}

/// Represents `BootBackend`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct BootBackend<R> {
  pub runner: R,
  pub capabilities: Capabilities,
}
impl<R: ProcessRunner> BootBackend<R> {
  /// Constructs `new` with this module's expected initial state. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn new(runner: R, capabilities: Capabilities) -> Self {
    Self {
      runner,
      capabilities,
    }
  }
  /// Executes the `collect` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn collect(&self) -> BootSnapshot {
    let firmware = if self.capabilities.is_uefi {
      "UEFI"
    } else {
      "BIOS/Legacy"
    }
    .into();
    let (bootloader, bootloader_info) = detect_bootloader(&self.capabilities);
    let current_kernel = uname(&self.runner).unwrap_or_else(|| "Não disponível".into());
    let kernels = detect_kernels(&current_kernel, &bootloader_info);
    let initramfs = read_initramfs(&self.capabilities);
    let plymouth = read_plymouth(&self.capabilities, &self.runner);
    BootSnapshot {
      firmware,
      bootloader,
      esp: detect_esp(),
      current_kernel,
      kernels,
      bootloader_info,
      initramfs,
      plymouth,
      secure_boot: detect_secure_boot(),
    }
  }
}

/// Executes the `uname` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn uname<R: ProcessRunner>(runner: &R) -> Option<String> {
  let o = runner
    .run(
      &ProcessRequest::new("uname")
        .arg("-r")
        .timeout(Duration::from_secs(2)),
    )
    .ok()?;
  (o.status == Some(0) && !o.timed_out)
    .then(|| {
      terminal_text(&String::from_utf8_lossy(&o.stdout))
        .trim()
        .to_owned()
    })
    .filter(|v| !v.is_empty())
}
/// Retrieves data for `detect_esp` without mixing collection with TUI rendering. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn detect_esp() -> Option<String> {
  ["/boot", "/efi", "/boot/efi"]
    .iter()
    .find(|p| Path::new(p).join("EFI").is_dir())
    .map(|p| (*p).into())
}
/// Retrieves data for `detect_bootloader` without mixing collection with TUI rendering. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn detect_bootloader(cap: &Capabilities) -> (BootloaderKind, BootloaderInfo) {
  let mut info = BootloaderInfo::default();
  let systemd = Path::new("/boot/loader/loader.conf").is_file()
    || Path::new("/efi/loader/loader.conf").is_file()
    || Path::new("/boot/EFI/Linux").is_dir();
  let grub = Path::new("/boot/grub/grub.cfg").is_file()
    || Path::new("/boot/grub2/grub.cfg").is_file()
    || Path::new("/etc/default/grub").is_file();
  if systemd && !grub {
    let root = if Path::new("/efi/loader/loader.conf").is_file() {
      PathBuf::from("/efi")
    } else {
      PathBuf::from("/boot")
    };
    info = parse_systemd_boot(&root);
    return (BootloaderKind::SystemdBoot, info);
  }
  if grub && !systemd {
    info = parse_grub_defaults(Path::new("/etc/default/grub")).unwrap_or_default();
    return (BootloaderKind::Grub, info);
  }
  if cap.has_systemd_boot && !grub && (Path::new("/boot/loader/entries").is_dir()) {
    return (
      BootloaderKind::SystemdBoot,
      parse_systemd_boot(Path::new("/boot")),
    );
  }
  (BootloaderKind::Unknown, info)
}
/// Converts input data into `parse_systemd_boot` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn parse_systemd_boot(root: &Path) -> BootloaderInfo {
  let loader = root.join("loader/loader.conf");
  let mut info = BootloaderInfo::default();
  if let Ok(text) = fs::read_to_string(loader) {
    for line in text.lines().map(str::trim) {
      if line.starts_with('#') {
        continue;
      }
      let mut p = line.split_whitespace();
      match p.next() {
        Some("default") => info.default_entry = p.next().map(str::to_owned),
        Some("timeout") => info.timeout = p.next().and_then(|v| v.parse().ok()),
        _ => {}
      }
    }
  }
  let dir = root.join("loader/entries");
  if let Ok(entries) = fs::read_dir(dir) {
    for entry in entries
      .flatten()
      .filter(|e| e.path().extension().is_some_and(|v| v == "conf"))
    {
      if let Ok(text) = fs::read_to_string(entry.path()) {
        info.entries.push(parse_entry(
          entry.file_name().to_string_lossy().into_owned(),
          &text,
          info.default_entry.as_deref(),
        ));
      }
    }
  }
  info.entries.sort_by(|a, b| {
    (b.is_default as u8)
      .cmp(&(a.is_default as u8))
      .then_with(|| a.id.cmp(&b.id))
  });
  info
}
/// Converts input data into `parse_entry` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn parse_entry(id: String, text: &str, default: Option<&str>) -> BootEntry {
  let mut e = BootEntry {
    id: id.clone(),
    is_default: default.is_some_and(|v| v == id || v == id.trim_end_matches(".conf")),
    ..Default::default()
  };
  for line in text.lines().map(str::trim) {
    let mut p = line.splitn(2, char::is_whitespace);
    let key = p.next().unwrap_or("");
    let value = p.next().unwrap_or("").trim();
    match key {
      "title" => e.title = terminal_text(value),
      "linux" => e.linux = Some(terminal_text(value)),
      "initrd" => e.initrd.push(terminal_text(value)),
      "options" => e.options = Some(terminal_text(value)),
      _ => {}
    }
  }
  e
}
/// Converts input data into `parse_grub_defaults` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn parse_grub_defaults(path: &Path) -> Option<BootloaderInfo> {
  let text = fs::read_to_string(path).ok()?;
  let mut values = BTreeMap::new();
  for line in text.lines().map(str::trim) {
    if line.is_empty() || line.starts_with('#') {
      continue;
    }
    if let Some((k, v)) = line.split_once('=') {
      let value = v.trim().trim_matches('"').trim_matches('\'').to_string();
      values.insert(k.to_string(), value);
    }
  }
  Some(BootloaderInfo {
    default_entry: values.get("GRUB_DEFAULT").cloned(),
    timeout: values.get("GRUB_TIMEOUT").and_then(|v| v.parse().ok()),
    grub_values: values.into_iter().collect(),
    ..Default::default()
  })
}
/// Retrieves data for `detect_kernels` without mixing collection with TUI rendering. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn detect_kernels(current: &str, bootloader: &BootloaderInfo) -> Vec<KernelInfo> {
  let mut out = Vec::new();
  if let Ok(entries) = fs::read_dir("/usr/lib/modules") {
    for e in entries.flatten() {
      let package = e.file_name().to_string_lossy().into_owned();
      let dir = e.path();
      let image = Path::new("/boot").join(format!("vmlinuz-{package}"));
      let init = Path::new("/boot").join(format!("initramfs-{package}.img"));
      let fallback = Path::new("/boot").join(format!("initramfs-{package}-fallback.img"));
      let default = bootloader.entries.iter().any(|entry| {
        entry.is_default
          && entry
            .linux
            .as_deref()
            .is_some_and(|linux| linux.ends_with(&format!("vmlinuz-{package}")))
      });
      let uki = ["/boot", "/efi", "/boot/efi"].iter().find_map(|root| {
        let path = Path::new(root).join(format!("EFI/Linux/{package}.efi"));
        path.is_file().then(|| path.to_string_lossy().into_owned())
      });
      out.push(KernelInfo {
        package: package.clone(),
        version: package.clone(),
        current: package == current,
        headers: dir.join("build").exists(),
        image: image
          .is_file()
          .then(|| image.to_string_lossy().into_owned()),
        initramfs: init.is_file().then(|| init.to_string_lossy().into_owned()),
        fallback: fallback
          .is_file()
          .then(|| fallback.to_string_lossy().into_owned()),
        preset: Path::new("/etc/mkinitcpio.d")
          .join(format!("{package}.preset"))
          .is_file()
          .then(|| format!("/etc/mkinitcpio.d/{package}.preset")),
        default,
        uki,
      });
    }
  }
  out.sort_by(|a, b| {
    (b.current as u8, b.default as u8)
      .cmp(&(a.current as u8, a.default as u8))
      .then_with(|| a.package.cmp(&b.package))
  });
  out
}
/// Retrieves data for `read_initramfs` without mixing collection with TUI rendering. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn read_initramfs(cap: &Capabilities) -> InitramfsInfo {
  let mut i = InitramfsInfo {
    available: cap.has_mkinitcpio,
    config_path: Path::new("/etc/mkinitcpio.conf")
      .is_file()
      .then(|| "/etc/mkinitcpio.conf".into()),
    ..Default::default()
  };
  if let Ok(text) = fs::read_to_string("/etc/mkinitcpio.conf") {
    for line in text.lines().map(str::trim) {
      if let Some((k, v)) = line.split_once('=') {
        let values = v
          .trim()
          .trim_matches('"')
          .split_whitespace()
          .map(str::to_owned)
          .collect();
        match k {
          "MODULES" => i.modules = values,
          "BINARIES" => i.binaries = values,
          "FILES" => i.files = values,
          "HOOKS" => i.hooks = values,
          _ => {}
        }
      }
    }
  }
  if let Ok(entries) = fs::read_dir("/etc/mkinitcpio.d") {
    i.presets = entries
      .flatten()
      .filter_map(|e| {
        e.path()
          .extension()
          .is_some_and(|v| v == "preset")
          .then(|| e.file_name().to_string_lossy().into_owned())
      })
      .collect()
  }
  i
}
/// Retrieves data for `read_plymouth` without mixing collection with TUI rendering. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn read_plymouth<R: ProcessRunner>(cap: &Capabilities, runner: &R) -> PlymouthInfo {
  let installed = cap.has_plymouth;
  let current = runner
    .run(&ProcessRequest::new("plymouth-set-default-theme").arg("--get-default"))
    .ok()
    .filter(|o| o.status == Some(0) && !o.timed_out)
    .map(|o| {
      terminal_text(&String::from_utf8_lossy(&o.stdout))
        .trim()
        .to_owned()
    })
    .filter(|v| !v.is_empty())
    .or_else(|| default_theme_from("/etc/plymouth/plymouthd.conf"))
    .or_else(|| default_theme_from("/usr/share/plymouth/plymouthd.defaults"));
  let themes = fs::read_dir("/usr/share/plymouth/themes")
    .ok()
    .into_iter()
    .flatten()
    .filter_map(Result::ok)
    .filter(|e| e.path().is_dir())
    .map(|e| e.file_name().to_string_lossy().into_owned())
    .collect();
  PlymouthInfo {
    installed,
    current_theme: current,
    themes,
  }
}
/// Fall back to the `Theme=` value from the plymouth daemon configuration,
/// which is the archival source on distros whose `plymouth-set-default-theme`
/// lacks the `--get-default` flag (e.g. Arch).
fn default_theme_from(path: &str) -> Option<String> {
  let contents = fs::read_to_string(path).ok()?;
  let mut in_daemon = false;
  for line in contents.lines() {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
      continue;
    }
    if line.starts_with('[') && line.ends_with(']') {
      in_daemon = line == "[Daemon]";
      continue;
    }
    if in_daemon
      && let Some(value) = line.strip_prefix("Theme=")
      && let Some(theme) = value.split_whitespace().next()
    {
      return Some(theme.to_owned());
    }
  }
  None
}

/// Retrieves data for `detect_secure_boot` without mixing collection with TUI rendering. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn detect_secure_boot() -> Option<bool> {
  let entry = fs::read_dir("/sys/firmware/efi/efivars")
    .ok()?
    .find_map(|entry| {
      entry
        .ok()
        .filter(|e| e.file_name().to_string_lossy().starts_with("SecureBoot-"))
    })?;
  let data = fs::read(entry.path()).ok()?;
  data.last().map(|v| *v == 1)
}
#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  /// Retrieves data for `loader_preserves_unknown_and_parses_fields` without mixing collection with TUI rendering. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn loader_preserves_unknown_and_parses_fields() {
    let root = std::env::temp_dir().join(format!("argvus-boot-{}", std::process::id()));
    let _ = fs::create_dir_all(root.join("loader/entries"));
    fs::write(
      root.join("loader/loader.conf"),
      "default arch.conf\ntimeout 5\nunknown x\n",
    )
    .unwrap();
    fs::write(
      root.join("loader/entries/arch.conf"),
      "title Arch Linux\nlinux /vmlinuz-linux\ninitrd /initramfs.img\noptions quiet\n",
    )
    .unwrap();
    let x = parse_systemd_boot(&root);
    assert_eq!(x.default_entry.as_deref(), Some("arch.conf"));
    assert_eq!(x.timeout, Some(5));
    assert_eq!(x.entries[0].title, "Arch Linux");
    let _ = fs::remove_dir_all(root);
  }
  #[test]
  /// Executes the `grub_quotes_parse` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn grub_quotes_parse() {
    let p = std::env::temp_dir().join(format!("argvus-grub-{}", std::process::id()));
    fs::write(&p, "GRUB_DEFAULT=\"saved\"\nGRUB_TIMEOUT=7\n# x\n").unwrap();
    let x = parse_grub_defaults(&p).unwrap();
    assert_eq!(x.default_entry.as_deref(), Some("saved"));
    assert_eq!(x.timeout, Some(7));
    let _ = fs::remove_file(p);
  }

  #[test]
  /// Executes the `plymouth_fallback_reads_only_the_daemon_theme` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn plymouth_fallback_reads_only_the_daemon_theme() {
    let path = std::env::temp_dir().join(format!("argvus-plymouth-{}", std::process::id()));
    fs::write(
      &path,
      "# comment\n[Daemon]\nTheme=argvus\nShowDelay=0\n#Theme=fade-in\n",
    )
    .unwrap();
    assert_eq!(
      default_theme_from(path.to_str().unwrap()).as_deref(),
      Some("argvus")
    );
    let _ = fs::remove_file(path);

    let absent =
      std::env::temp_dir().join(format!("argvus-plymouth-absent-{}", std::process::id()));
    assert_eq!(default_theme_from(absent.to_str().unwrap()), None);
  }
  #[test]
  /// Executes the `systemd_entries_are_sorted_default_first` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn systemd_entries_are_sorted_default_first() {
    let root = std::env::temp_dir().join(format!("argvus-boot-sort-{}", std::process::id()));
    let _ = fs::create_dir_all(root.join("loader/entries"));
    fs::write(
      root.join("loader/loader.conf"),
      "default b.conf\ntimeout 3\n",
    )
    .unwrap();
    fs::write(
      root.join("loader/entries/a.conf"),
      "title A\nlinux /vmlinuz-a\n",
    )
    .unwrap();
    fs::write(
      root.join("loader/entries/b.conf"),
      "title B\nlinux /vmlinuz-b\n",
    )
    .unwrap();
    let x = parse_systemd_boot(&root);
    assert_eq!(x.entries.len(), 2);
    assert_eq!(x.entries[0].id, "b.conf");
    assert!(x.entries[0].is_default);
    let _ = fs::remove_dir_all(root);
  }
}
