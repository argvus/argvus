//! Detail lines of a kernel, the bootloader and the initramfs.

use super::*;

impl BootApp {
  /// `Current · Default` badges of a kernel, or an empty string.
  pub(super) fn kernel_badges(&self, kernel: &crate::model::KernelInfo) -> String {
    let mut badges = Vec::new();
    if kernel.current {
      badges.push(tr(self.lang, "control_center.current"));
    }
    if kernel.default {
      badges.push(tr(self.lang, "control_center.default"));
    }
    badges.join(" · ")
  }

  pub(super) fn kernel_detail(&self, index: usize) -> Vec<Row<Item>> {
    let Some(k) = self.snapshot.kernels.get(index) else {
      return vec![Row::info(
        tr(self.lang, "control_center.kernel_not_found"),
        "",
      )];
    };
    let badges = self.kernel_badges(k);
    vec![
      Row::section(tr(self.lang, "control_center.kernel")),
      Row::info(self.field("control_center.package"), k.package.clone()),
      Row::info(
        self.field("control_center.version_20bc85"),
        k.version.clone(),
      ),
      Row::info(
        self.field("control_center.status"),
        if badges.is_empty() {
          MISSING.into()
        } else {
          badges
        },
      ),
      Row::section(tr(self.lang, "control_center.section_files")),
      Row::info(
        self.field("control_center.image"),
        k.image.as_deref().unwrap_or(MISSING),
      ),
      Row::info(
        self.field("control_center.initramfs_6d7381"),
        k.initramfs.as_deref().unwrap_or(MISSING),
      ),
      Row::info(
        self.field("control_center.fallback"),
        k.fallback.as_deref().unwrap_or(MISSING),
      ),
      Row::info(
        self.field("control_center.headers"),
        yes_no(self.lang, k.headers),
      ),
      Row::info(
        self.field("control_center.preset"),
        k.preset.as_deref().unwrap_or(MISSING),
      ),
      Row::info(
        self.field("control_center.uki"),
        k.uki.as_deref().unwrap_or(MISSING),
      ),
      Row::section(tr(self.lang, "control_center.section_actions")),
      self.set_default_row(),
    ]
  }

  pub(super) fn bootloader_detail(&self, index: usize) -> Vec<Row<Item>> {
    let Some(e) = self.snapshot.bootloader_info.entries.get(index) else {
      return vec![Row::info(
        tr(self.lang, "control_center.entry_not_found"),
        "",
      )];
    };
    vec![
      Row::section(tr(self.lang, "control_center.section_entry")),
      Row::info(self.field("control_center.title"), e.title.clone()),
      Row::info(self.field("control_center.id"), e.id.clone()),
      Row::info(
        self.field("control_center.status"),
        if e.is_default {
          tr(self.lang, "control_center.default")
        } else {
          MISSING
        },
      ),
      Row::info(
        self.field("control_center.linux_efi"),
        e.linux.as_deref().unwrap_or(MISSING),
      ),
      Row::info(self.field("control_center.initrd"), joined(&e.initrd)),
      Row::info(
        self.field("control_center.options"),
        e.options.as_deref().unwrap_or(MISSING),
      ),
      Row::section(tr(self.lang, "control_center.section_actions")),
      self.set_default_row(),
    ]
  }

  pub(super) fn initramfs_detail(&self, index: usize) -> Vec<Row<Item>> {
    let initramfs = &self.snapshot.initramfs;
    vec![
      Row::section(tr(self.lang, "control_center.initramfs")),
      Row::info(
        self.field("control_center.preset"),
        initramfs
          .presets
          .get(index)
          .map(String::as_str)
          .unwrap_or(MISSING),
      ),
      Row::info(
        self.field("control_center.config"),
        initramfs.config_path.as_deref().unwrap_or(MISSING),
      ),
      Row::section(tr(self.lang, "control_center.configuration")),
      Row::info(
        tr(self.lang, "control_center.initramfs_modules"),
        joined(&initramfs.modules),
      ),
      Row::info(
        tr(self.lang, "control_center.initramfs_binaries"),
        joined(&initramfs.binaries),
      ),
      Row::info(
        tr(self.lang, "control_center.initramfs_files"),
        joined(&initramfs.files),
      ),
      Row::info(
        tr(self.lang, "control_center.initramfs_hooks"),
        joined(&initramfs.hooks),
      ),
    ]
  }

  pub(super) fn grub_value(&self, key: &str) -> String {
    self
      .snapshot
      .bootloader_info
      .grub_values
      .iter()
      .find_map(|(name, value)| (name == key).then(|| value.clone()))
      .unwrap_or_default()
  }
}

/// Space-separated values, or [`MISSING`] when there are none.
pub(super) fn joined(values: &[String]) -> String {
  if values.is_empty() {
    MISSING.into()
  } else {
    values.join(" ")
  }
}

pub(super) fn yes_no(lang: Lang, value: bool) -> String {
  tr(
    lang,
    if value {
      "control_center.yes"
    } else {
      "control_center.no"
    },
  )
  .into()
}
