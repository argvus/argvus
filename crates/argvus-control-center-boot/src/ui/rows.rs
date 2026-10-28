//! Rows and labels of the Boot page lists.

use super::details::yes_no;
use super::*;

impl BootApp {
  /// Rows of the current page.
  pub(super) fn rows(&self) -> Vec<Row<Item>> {
    match self.page {
      BootPage::Home => self.home_rows(),
      BootPage::Summary => self.summary_rows(),
      BootPage::Kernel => self.kernel_rows(),
      BootPage::KernelDetail(index) => self.kernel_detail(index),
      BootPage::Bootloader => self.bootloader_rows(),
      BootPage::BootloaderDetail(index) => self.bootloader_detail(index),
      BootPage::Initramfs => self.initramfs_rows(),
      BootPage::InitramfsDetail(index) => self.initramfs_detail(index),
      BootPage::Plymouth => self.plymouth_rows(),
    }
  }

  pub(super) fn home_rows(&self) -> Vec<Row<Item>> {
    let secure = match self.snapshot.secure_boot {
      Some(value) => format!(
        "{}: {}",
        tr(self.lang, "control_center.secure_boot"),
        yes_no(self.lang, value)
      ),
      None => tr(self.lang, "control_center.secure_boot_n_a").into(),
    };
    let timeout = self
      .snapshot
      .bootloader_info
      .timeout
      .map(|t| format!("{}: {} s", tr(self.lang, "control_center.timeout"), t))
      .unwrap_or_else(|| tr(self.lang, "control_center.timeout_default").into());
    let initramfs = if self.snapshot.initramfs.available {
      format!(
        "{}: {}",
        yes_no(self.lang, true),
        self.snapshot.initramfs.presets.len()
      )
    } else {
      tr(self.lang, "control_center.not_installed").into()
    };
    let plymouth = match &self.snapshot.plymouth.current_theme {
      Some(theme) => theme.clone(),
      None if self.snapshot.plymouth.installed => tr(self.lang, "control_center.installed").into(),
      None => tr(self.lang, "control_center.not_installed").into(),
    };
    vec![
      Row::submenu(Item::Summary, tr(self.lang, "control_center.summary"))
        .icon(icons::INFO)
        .detail(format!("{} · {}", self.snapshot.firmware, secure)),
      Row::submenu(Item::Kernels, tr(self.lang, "control_center.kernels"))
        .icon(icons::CPU)
        .detail(format!(
          "{} · {}",
          self.snapshot.kernels.len(),
          self.snapshot.current_kernel
        )),
      Row::submenu(
        Item::Bootloader,
        tr(self.lang, "control_center.bootloader_f1a3c5"),
      )
      .icon(icons::BOOT)
      .detail(format!("{} · {}", self.loader_label(), timeout)),
      Row::submenu(Item::Initramfs, tr(self.lang, "control_center.initramfs"))
        .icon(icons::PACKAGES)
        .detail(initramfs),
      Row::submenu(Item::Plymouth, tr(self.lang, "control_center.plymouth"))
        .icon(icons::IMAGE)
        .detail(plymouth),
    ]
  }

  /// Read-only overview of the boot chain.
  pub(super) fn summary_rows(&self) -> Vec<Row<Item>> {
    let lang = self.lang;
    let timeout = self
      .snapshot
      .bootloader_info
      .timeout
      .map(|t| format!("{t} s"))
      .unwrap_or_else(|| tr(lang, "control_center.default").into());
    let secure_boot = self
      .snapshot
      .secure_boot
      .map(|v| yes_no(lang, v))
      .unwrap_or_else(|| tr(lang, "control_center.unavailable").into());
    let default_entry = self
      .snapshot
      .bootloader_info
      .entries
      .iter()
      .find(|entry| entry.is_default)
      .map(|entry| entry.title.clone())
      .or_else(|| self.snapshot.bootloader_info.default_entry.clone())
      .unwrap_or_else(|| MISSING.into());
    let plymouth_theme = self
      .snapshot
      .plymouth
      .current_theme
      .clone()
      .unwrap_or_else(|| MISSING.into());
    vec![
      Row::section(tr(lang, "control_center.section_base_system")),
      Row::info(
        tr(lang, "control_center.firmware"),
        self.snapshot.firmware.clone(),
      ),
      Row::info(tr(lang, "control_center.secure_boot"), secure_boot),
      Row::info(
        tr(lang, "control_center.kernel"),
        self.snapshot.current_kernel.clone(),
      ),
      Row::section(tr(lang, "control_center.bootloader_f1a3c5")),
      Row::info(tr(lang, "control_center.boot_manager"), self.loader_label()),
      Row::info(tr(lang, "control_center.default_entry"), default_entry),
      Row::info(
        tr(lang, "control_center.section_entries"),
        self.snapshot.bootloader_info.entries.len().to_string(),
      ),
      Row::info(
        tr(lang, "control_center.esp_path"),
        self.snapshot.esp.as_deref().unwrap_or(MISSING),
      ),
      Row::info(tr(lang, "control_center.timeout"), timeout),
      Row::section(tr(lang, "control_center.section_components")),
      Row::info(
        tr(lang, "control_center.kernels"),
        self.snapshot.kernels.len().to_string(),
      ),
      Row::info(
        "mkinitcpio",
        yes_no(lang, self.snapshot.initramfs.available),
      ),
      Row::info(
        tr(lang, "control_center.plymouth"),
        yes_no(lang, self.snapshot.plymouth.installed),
      ),
      Row::info(tr(lang, "control_center.theme"), plymouth_theme),
    ]
  }

  /// One submenu per installed kernel, with its badges on the right.
  pub(super) fn kernel_rows(&self) -> Vec<Row<Item>> {
    self
      .snapshot
      .kernels
      .iter()
      .enumerate()
      .map(|(index, kernel)| {
        let row = Row::submenu(
          Item::Kernel(index),
          format!("{} {}", kernel.package, kernel.version.trim()),
        );
        let badges = self.kernel_badges(kernel);
        if badges.is_empty() {
          row
        } else {
          row.detail(badges)
        }
      })
      .collect()
  }

  /// `Set default` for the open kernel or entry, disabled with the reason
  /// on the right when no systemd-boot entry can be mapped.
  pub(super) fn set_default_row(&self) -> Row<Item> {
    let row = Row::action(
      Item::SetDefault,
      tr(self.lang, "control_center.set_default"),
    )
    .icon(icons::STAR);
    match self.default_unavailable_reason() {
      Some(reason) => row.enabled(false).detail(reason),
      None => row,
    }
  }

  /// Entries, the timeout and, with GRUB, the kernel command line and the
  /// regeneration in the Danger zone.
  pub(super) fn bootloader_rows(&self) -> Vec<Row<Item>> {
    let lang = self.lang;
    let mut rows = Vec::new();
    let entries = &self.snapshot.bootloader_info.entries;
    if !entries.is_empty() {
      rows.push(Row::section(tr(lang, "control_center.section_entries")));
      rows.extend(entries.iter().enumerate().map(|(index, entry)| {
        let row = Row::submenu(Item::Entry(index), entry.title.clone());
        if entry.is_default {
          row.detail(tr(lang, "control_center.default"))
        } else {
          row
        }
      }));
    }
    rows.push(Row::section(tr(lang, "control_center.configuration")));
    let timeout = Row::value(
      Item::Timeout,
      tr(lang, "control_center.timeout"),
      self
        .snapshot
        .bootloader_info
        .timeout
        .map(|t| format!("{t} s"))
        .unwrap_or_else(|| tr(lang, "control_center.default").into()),
      None,
    )
    .icon(icons::TIMER);
    rows.push(if self.snapshot.bootloader == BootloaderKind::Unknown {
      timeout
        .enabled(false)
        .detail(tr(lang, "control_center.unavailable_unknown_bootloader"))
    } else {
      timeout
    });
    if self.snapshot.bootloader == BootloaderKind::Grub {
      let cmdline = self.grub_value("GRUB_CMDLINE_LINUX_DEFAULT");
      rows.push(
        Row::value(
          Item::KernelCmdline,
          tr(lang, "control_center.kernel_command_line"),
          if cmdline.is_empty() {
            MISSING.into()
          } else {
            cmdline
          },
          None,
        )
        .icon(icons::TERMINAL),
      );
      rows.push(Row::section(tr(lang, "control_center.danger_zone")));
      rows.push(
        Row::destructive(
          Item::RegenerateGrub,
          tr(lang, "control_center.regenerate_grub"),
        )
        .icon(icons::SYNC),
      );
    }
    rows
  }

  /// Presets and, in the Danger zone, the regeneration of every image.
  pub(super) fn initramfs_rows(&self) -> Vec<Row<Item>> {
    let lang = self.lang;
    let mut rows = Vec::new();
    let presets = &self.snapshot.initramfs.presets;
    if !presets.is_empty() {
      rows.push(Row::section(tr(lang, "control_center.section_presets")));
      rows.extend(
        presets
          .iter()
          .enumerate()
          .map(|(index, preset)| Row::submenu(Item::Preset(index), preset.clone())),
      );
    }
    rows.push(Row::section(tr(lang, "control_center.danger_zone")));
    rows.push(
      Row::destructive(
        Item::RegenerateInitramfs,
        tr(lang, "control_center.regenerate_all_initramfs_images"),
      )
      .icon(icons::SYNC),
    );
    rows
  }

  /// One choice per installed theme; `●` marks the current one.
  pub(super) fn plymouth_rows(&self) -> Vec<Row<Item>> {
    let current = self.snapshot.plymouth.current_theme.as_deref();
    self
      .snapshot
      .plymouth
      .themes
      .iter()
      .enumerate()
      .map(|(index, theme)| Row::choice(Item::Theme(index), theme.clone(), current == Some(theme)))
      .collect()
  }

  pub(super) fn loader_label(&self) -> &'static str {
    match self.snapshot.bootloader {
      BootloaderKind::SystemdBoot => "systemd-boot",
      BootloaderKind::Grub => "GRUB",
      BootloaderKind::Unknown => tr(self.lang, "control_center.unknown"),
    }
  }

  pub(super) fn page_label(&self) -> &'static str {
    match self.page {
      BootPage::Summary => tr(self.lang, "control_center.summary"),
      BootPage::Kernel | BootPage::KernelDetail(_) => tr(self.lang, "control_center.kernel"),
      BootPage::Bootloader | BootPage::BootloaderDetail(_) => {
        tr(self.lang, "control_center.bootloader_f1a3c5")
      }
      BootPage::Initramfs | BootPage::InitramfsDetail(_) => {
        tr(self.lang, "control_center.initramfs")
      }
      BootPage::Plymouth => tr(self.lang, "control_center.plymouth"),
      BootPage::Home => tr(self.lang, "control_center.boot"),
    }
  }
}
