//! Terminal UI rendering and interaction of the Boot page.
use crate::{
  backend::BootBackend,
  model::{BootPage, BootSnapshot, BootloaderKind},
};
use argvus_control_center_core::navigation::HomeCardId;
use argvus_control_center_core::{
  capabilities::Capabilities,
  config::AppConfig,
  jobs::{JobHandle, JobManager, JobState},
  privileged::{PrivilegedRequest, SystemSettingsOperation},
  process::{LiveProcess, SystemProcessRunner},
  sanitize::terminal_text,
};
use argvus_i18n::{Lang, tr};
use argvus_theme::Theme;
use argvus_tui::{
  chrome,
  components::{StatusKind, StatusMessage},
  confirm::{ConfirmDialog, ConfirmOutcome, ConfirmState, draw_confirm},
  hints::{HintContext, confirm_hints, hints},
  icons,
  menu::{MenuEvent, MenuState, MenuStyle, Row, draw_menu},
  page::{shell, status},
  text,
};
use crossterm::event::KeyCode;
use ratatui::{
  Frame,
  style::Style,
  text::Line,
  widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

mod actions;
mod details;
mod input;
mod jobs;
mod render;
mod rows;

#[derive(Debug, Clone)]
enum Pending {
  Action(BootAction),
}
#[derive(Debug, Clone)]
enum BootAction {
  SystemdDefault(String),
  SystemdTimeout(u32),
  GrubTimeout(u32),
  GrubCmdline(String),
  GrubRegenerate,
  Initramfs,
  Plymouth(String),
}

impl BootAction {
  /// The regenerations listed in a Danger zone, confirmed in the danger
  /// style; the other changes use the plain confirmation.
  fn is_destructive(&self) -> bool {
    matches!(self, Self::GrubRegenerate | Self::Initramfs)
  }
}
#[derive(Debug, Clone, PartialEq, Eq)]
enum ActionResult {
  Success,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InputMode {
  Timeout,
  GrubCmdline,
}

const TRANSACTION_POPUP_WIDTH: u16 = 100;
const TRANSACTION_POPUP_HEIGHT: u16 = 20;
const TRANSACTION_CONTENT_WIDTH: usize = TRANSACTION_POPUP_WIDTH as usize - 2;
const TRANSACTION_CONTENT_HEIGHT: usize = TRANSACTION_POPUP_HEIGHT as usize - 2;

/// Longest kernel command line accepted, the same limit the privileged
/// `grub-cmdline` operation enforces.
const KERNEL_CMDLINE_MAX: usize = 2048;
/// Digits accepted by the timeout field (0–60 seconds).
const TIMEOUT_MAX_DIGITS: usize = 2;
/// Widest the input popup grows, for long kernel command lines.
const INPUT_POPUP_MAX_WIDTH: u16 = 100;

/// Stable identity of a Boot menu row. Kernels, entries, presets and
/// Plymouth themes keep their index in the snapshot list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Item {
  Summary,
  Kernels,
  Bootloader,
  Initramfs,
  Plymouth,
  /// Index into the installed kernels.
  Kernel(usize),
  /// Index into the bootloader entries.
  Entry(usize),
  /// Index into the mkinitcpio presets.
  Preset(usize),
  /// Makes the open kernel or entry the systemd-boot default (confirmed).
  SetDefault,
  /// Opens the timeout field (confirmed when applied).
  Timeout,
  /// Opens the GRUB kernel command line field (confirmed when applied).
  KernelCmdline,
  RegenerateGrub,
  RegenerateInitramfs,
  /// Index into the installed Plymouth themes.
  Theme(usize),
}

impl Item {
  /// Whether the row only opens another page (and starts no boot change).
  fn opens_page(self) -> bool {
    matches!(
      self,
      Self::Summary
        | Self::Kernels
        | Self::Bootloader
        | Self::Initramfs
        | Self::Plymouth
        | Self::Kernel(_)
        | Self::Entry(_)
        | Self::Preset(_)
    )
  }
}

pub struct BootApp {
  pub page: BootPage,
  menu: MenuState,
  /// Body height during the last draw, used as the PgUp/PgDn distance.
  list_height: u16,
  snapshot: BootSnapshot,
  job: Option<JobHandle<Result<BootSnapshot, String>>>,
  action: Option<JobHandle<(ActionResult, String)>>,
  pending: Option<Pending>,
  confirmation: ConfirmState,
  timeout_input: Option<String>,
  input_mode: Option<InputMode>,
  transaction_live: Option<LiveProcess>,
  transaction_open: bool,
  transaction_scroll: u16,
  transaction_follow: bool,
  jobs: JobManager,
  lang: Lang,
  theme: Theme,
  capabilities: Capabilities,
  pub status: Option<StatusMessage>,
}

impl BootApp {
  /// Replaces the semantic theme used by this page.
  pub fn set_theme(&mut self, theme: &Theme) {
    self.theme = theme.clone();
  }

  pub fn new(lang: Lang, theme: Theme, capabilities: Capabilities) -> Self {
    Self {
      page: BootPage::Home,
      menu: MenuState::default(),
      list_height: 0,
      snapshot: Default::default(),
      job: None,
      action: None,
      pending: None,
      confirmation: ConfirmState::new(),
      timeout_input: None,
      input_mode: None,
      transaction_live: None,
      transaction_open: false,
      transaction_scroll: 0,
      transaction_follow: false,
      jobs: JobManager::default(),
      lang,
      theme,
      capabilities,
      status: None,
    }
  }
  fn success(&mut self, text: impl Into<String>) {
    self.status = Some(StatusMessage {
      kind: StatusKind::Success,
      text: text.into(),
    });
  }
  fn error(&mut self, text: impl Into<String>) {
    self.status = Some(StatusMessage {
      kind: StatusKind::Error,
      text: text.into(),
    });
  }
  fn normalize(&mut self) {
    let rows = self.rows();
    self.menu.normalize(&rows);
  }

  /// Opens `page` with the cursor on its first selectable row.
  fn go(&mut self, page: BootPage) {
    self.page = page;
    self.menu = MenuState::default();
  }

  /// A translated field label without the trailing colon some catalog
  /// entries carry, since Info rows draw label and value in columns.
  fn field(&self, key: &str) -> &'static str {
    tr(self.lang, key).trim_end_matches(':')
  }
}

/// Shown for a missing value.
const MISSING: &str = "—";

#[cfg(test)]
mod tests;
