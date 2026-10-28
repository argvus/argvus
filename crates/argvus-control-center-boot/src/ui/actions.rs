//! Boot actions: choosing the default entry, typed values and the privileged requests they send.

use super::*;

impl BootApp {
  /// Runs the row `item`.
  pub(super) fn activate(&mut self, item: Item) {
    match item {
      Item::Summary => self.open_section(BootPage::Summary),
      Item::Kernels => self.open_section(BootPage::Kernel),
      Item::Bootloader => self.open_section(BootPage::Bootloader),
      Item::Initramfs => self.open_section(BootPage::Initramfs),
      Item::Plymouth => self.open_section(BootPage::Plymouth),
      Item::Kernel(index) => self.go(BootPage::KernelDetail(index)),
      Item::Entry(index) => self.go(BootPage::BootloaderDetail(index)),
      Item::Preset(index) => self.go(BootPage::InitramfsDetail(index)),
      Item::SetDefault => self.request_default(),
      Item::Timeout => {
        self.timeout_input = Some(String::new());
        self.input_mode = Some(InputMode::Timeout);
      }
      Item::KernelCmdline => {
        self.timeout_input = Some(self.grub_value("GRUB_CMDLINE_LINUX_DEFAULT"));
        self.input_mode = Some(InputMode::GrubCmdline);
      }
      Item::RegenerateGrub => self.request(BootAction::GrubRegenerate),
      Item::RegenerateInitramfs => self.request(BootAction::Initramfs),
      Item::Theme(index) => {
        if let Some(theme) = self.snapshot.plymouth.themes.get(index) {
          self.request(BootAction::Plymouth(theme.clone()));
        }
      }
    }
  }

  /// Opens a page from the Boot home and reloads the snapshot, as before.
  pub(super) fn open_section(&mut self, page: BootPage) {
    self.go(page);
    self.reload();
  }

  /// Asks for confirmation before running `action`.
  pub(super) fn request(&mut self, action: BootAction) {
    self.confirmation = ConfirmState::new();
    self.pending = Some(Pending::Action(action));
  }

  /// The systemd-boot entry that `Set default` uses on the open page.
  pub(super) fn default_target(&self) -> Option<&crate::model::BootEntry> {
    match self.page {
      BootPage::KernelDetail(index) => self
        .snapshot
        .kernels
        .get(index)
        .and_then(|kernel| self.entry_for_kernel(kernel)),
      BootPage::BootloaderDetail(index) => self
        .snapshot
        .bootloader_info
        .entries
        .get(index)
        .filter(|_| self.snapshot.bootloader == BootloaderKind::SystemdBoot),
      _ => None,
    }
  }

  /// Why `Set default` is unavailable on the open page, if it is.
  pub(super) fn default_unavailable_reason(&self) -> Option<&'static str> {
    if self.default_target().is_some() {
      None
    } else if self.snapshot.bootloader == BootloaderKind::Unknown {
      Some(tr(
        self.lang,
        "control_center.unavailable_unknown_bootloader",
      ))
    } else {
      Some(tr(
        self.lang,
        "control_center.unavailable_no_systemd_boot_entry",
      ))
    }
  }

  pub(super) fn request_default(&mut self) {
    let action = self
      .default_target()
      .map(|entry| BootAction::SystemdDefault(entry.id.clone()));
    if let Some(action) = action {
      self.pending = Some(Pending::Action(action));
    } else {
      self.error(tr(
        self.lang,
        "control_center.a_boot_entry_could_not_be_mapped_safely",
      ));
    }
  }

  pub(super) fn entry_for_kernel(
    &self,
    kernel: &crate::model::KernelInfo,
  ) -> Option<&crate::model::BootEntry> {
    (self.snapshot.bootloader == BootloaderKind::SystemdBoot)
      .then_some(())
      .and_then(|_| {
        self.snapshot.bootloader_info.entries.iter().find(|entry| {
          entry
            .linux
            .as_deref()
            .is_some_and(|linux| linux.ends_with(&format!("vmlinuz-{}", kernel.package)))
        })
      })
  }

  pub(super) fn apply_timeout_input(&mut self) {
    let value = self.timeout_input.take().unwrap_or_default();
    let Ok(seconds) = value.parse::<u32>() else {
      self.error(tr(self.lang, "control_center.invalid_timeout"));
      return;
    };
    if seconds > 60 {
      self.error(tr(
        self.lang,
        "control_center.timeout_must_be_between_0_and_60_seconds",
      ));
      return;
    }
    let action = match self.snapshot.bootloader {
      BootloaderKind::SystemdBoot => BootAction::SystemdTimeout(seconds),
      BootloaderKind::Grub => BootAction::GrubTimeout(seconds),
      BootloaderKind::Unknown => {
        self.error(tr(
          self.lang,
          "control_center.bootloader_is_undetermined_no_change_was_made",
        ));
        return;
      }
    };
    self.pending = Some(Pending::Action(action));
  }

  pub(super) fn apply_input(&mut self) {
    if self.input_mode == Some(InputMode::GrubCmdline) {
      let value = self.timeout_input.take().unwrap_or_default();
      self.input_mode = None;
      if value.len() > KERNEL_CMDLINE_MAX
        || value
          .chars()
          .any(|character| character.is_control() || matches!(character, '"' | '\\'))
      {
        self.error(tr(self.lang, "control_center.invalid_kernel_command_line"));
      } else {
        self.pending = Some(Pending::Action(BootAction::GrubCmdline(value)));
      }
    } else {
      self.apply_timeout_input();
      self.input_mode = None;
    }
  }
}

pub(super) fn action_message(lang: Lang, action: &BootAction) -> String {
  match action {
    BootAction::SystemdDefault(id) => format!(
      "{} '{}' {}",
      tr(lang, "control_center.set_entry"),
      id,
      tr(lang, "control_center.as_default_for_the_next_boot")
    ),
    BootAction::SystemdTimeout(v) | BootAction::GrubTimeout(v) => {
      format!("{} {} s?", tr(lang, "control_center.change_timeout_to"), v)
    }
    BootAction::GrubCmdline(value) => format!(
      "{} '{}' ?",
      tr(lang, "control_center.apply_kernel_command_line"),
      value
    ),
    BootAction::GrubRegenerate => tr(lang, "control_center.regenerate_grub_configuration").into(),
    BootAction::Initramfs => tr(
      lang,
      "control_center.regenerate_all_initramfs_images_an_invalid_configuration_may_affect_th",
    )
    .into(),
    BootAction::Plymouth(theme) => format!(
      "{} '{}' {}",
      tr(lang, "control_center.apply_plymouth_theme"),
      theme,
      tr(lang, "control_center.and_regenerate_initramfs")
    ),
  }
}

/// The privileged request of `action` and the line echoed at the top of
/// the live output, built without running anything so the exact operation
/// can be checked.
pub(super) fn boot_request(action: BootAction) -> Result<(String, PrivilegedRequest), String> {
  let (echo, name, args) = match action {
    BootAction::SystemdDefault(id) => (
      format!("$ bootctl set-default {id}"),
      "systemd-default",
      vec![id],
    ),
    BootAction::SystemdTimeout(seconds) => (
      format!("$ boot timeout = {seconds}s"),
      "systemd-timeout",
      vec![seconds.to_string()],
    ),
    BootAction::GrubTimeout(seconds) => (
      format!("$ GRUB_TIMEOUT = {seconds}s"),
      "grub-timeout",
      vec![seconds.to_string()],
    ),
    BootAction::GrubCmdline(value) => (
      "$ GRUB_CMDLINE_LINUX_DEFAULT".to_owned(),
      "grub-cmdline",
      vec![value],
    ),
    BootAction::GrubRegenerate => ("$ grub-mkconfig".to_owned(), "grub-regenerate", vec![]),
    BootAction::Initramfs => ("$ mkinitcpio -P".to_owned(), "initramfs-regenerate", vec![]),
    BootAction::Plymouth(theme) => (
      format!("$ plymouth-set-default-theme -R {theme}"),
      "plymouth-theme",
      vec![theme],
    ),
  };
  Ok((echo, PrivilegedRequest::new("boot", name, args)?))
}
