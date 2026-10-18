//! Implements terminal UI rendering and interaction in crate `argvus control center boot`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
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

#[derive(Debug, Clone)]
/// Defines `Pending`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
enum Pending {
  Action(BootAction),
}
#[derive(Debug, Clone)]
/// Defines `BootAction`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
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
/// Defines `ActionResult`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
enum ActionResult {
  Success,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Defines `InputMode`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
enum InputMode {
  Timeout,
  GrubCmdline,
}

/// Defines the constant `TRANSACTION_POPUP_WIDTH`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
const TRANSACTION_POPUP_WIDTH: u16 = 100;
/// Defines the constant `TRANSACTION_POPUP_HEIGHT`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
const TRANSACTION_POPUP_HEIGHT: u16 = 20;
/// Defines the constant `TRANSACTION_CONTENT_WIDTH`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
const TRANSACTION_CONTENT_WIDTH: usize = TRANSACTION_POPUP_WIDTH as usize - 2;
/// Defines the constant `TRANSACTION_CONTENT_HEIGHT`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
const TRANSACTION_CONTENT_HEIGHT: usize = TRANSACTION_POPUP_HEIGHT as usize - 2;

/// Executes the `transaction_wrapped_lines` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn transaction_wrapped_lines(output: &str) -> usize {
  output
    .lines()
    .map(|line| (text::display_width(line).div_ceil(TRANSACTION_CONTENT_WIDTH)).max(1))
    .sum()
}

/// Executes the `transaction_bottom_offset` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn transaction_bottom_offset(output: &str) -> u16 {
  transaction_wrapped_lines(output).saturating_sub(TRANSACTION_CONTENT_HEIGHT) as u16
}

/// Longest kernel command line accepted, the same limit the privileged
/// `grub-cmdline` operation enforces.
const KERNEL_CMDLINE_MAX: usize = 2048;
/// Digits accepted by the timeout field (0–60 seconds).
const TIMEOUT_MAX_DIGITS: usize = 2;
/// Widest the input popup grows, for long kernel command lines.
const INPUT_POPUP_MAX_WIDTH: u16 = 100;

/// Whether the open field takes `character`: the timeout takes up to two
/// digits; the kernel command line takes any text up to its limit, and the
/// characters the backend rejects are reported when the value is applied.
fn accepts_input_char(mode: Option<InputMode>, input: &str, character: char) -> bool {
  match mode {
    Some(InputMode::GrubCmdline) => input.len() + character.len_utf8() <= KERNEL_CMDLINE_MAX,
    _ => character.is_ascii_digit() && input.len() < TIMEOUT_MAX_DIGITS,
  }
}

/// The end of `input` that fits in `width` cells next to the cursor, so the
/// typing position stays visible in long values.
fn input_tail(input: &str, width: usize) -> String {
  let room = width.saturating_sub(1);
  let mut used = 0;
  let mut start = input.len();
  for (index, character) in input.char_indices().rev() {
    let cells = text::display_width(character.encode_utf8(&mut [0; 4]));
    if used + cells > room {
      break;
    }
    used += cells;
    start = index;
  }
  format!("{}_", &input[start..])
}

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

/// Represents `BootApp`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
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

  /// Constructs `new` with this module's expected initial state. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
  /// Executes the `reload` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn reload(&mut self) {
    if self.job.is_some() || self.action.is_some() {
      return;
    }
    let capabilities = self.capabilities.clone();
    self.job = Some(self.jobs.spawn(move |_| {
      Ok(Ok(
        BootBackend::new(SystemProcessRunner, capabilities).collect(),
      ))
    }));
    self.status = Some(StatusMessage {
      kind: StatusKind::Info,
      text: tr(self.lang, "control_center.loading_boot").into(),
    });
  }
  /// Executes the `poll` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn poll(&mut self) -> bool {
    let mut changed = false;
    if let Some(job) = &self.job
      && let JobState::Finished(result) = job.try_state()
    {
      self.job = None;
      match result {
        Ok(Ok(snapshot)) => {
          self.snapshot = snapshot;
          self.normalize();
          self.success(tr(self.lang, "control_center.boot_refreshed"));
        }
        Ok(Err(error)) | Err(error) => self.error(error),
      }
      changed = true;
    }
    if let Some(live) = &self.transaction_live
      && self.transaction_open
      && self.transaction_follow
      && self
        .action
        .as_ref()
        .is_some_and(|job| matches!(job.try_state(), JobState::Running))
    {
      self.transaction_scroll = transaction_bottom_offset(&live.output());
      changed = true;
    }
    if let Some(job) = &self.action
      && let JobState::Finished(result) = job.try_state()
    {
      self.action = None;
      match result {
        Ok((outcome, _)) => match outcome {
          ActionResult::Success => self.success(tr(
            self.lang,
            "control_center.operation_completed_the_change_will_be_used_on_the_next_boot",
          )),
        },
        Err(error) => self.error(error),
      }
      self.reload();
      changed = true;
    }
    changed
  }
  /// Executes the `success` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn success(&mut self, text: impl Into<String>) {
    self.status = Some(StatusMessage {
      kind: StatusKind::Success,
      text: text.into(),
    });
  }
  /// Executes the `error` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn error(&mut self, text: impl Into<String>) {
    self.status = Some(StatusMessage {
      kind: StatusKind::Error,
      text: text.into(),
    });
  }
  /// Executes the `busy` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn busy(&self) -> bool {
    self.job.is_some() || self.action.is_some()
  }
  /// Executes the `normalize` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn normalize(&mut self) {
    let rows = self.rows();
    self.menu.normalize(&rows);
  }
  /// Whether typed characters currently go to the timeout/kernel command
  /// line field, so `q`/`?` must not act as the global quit/help keys. The
  /// confirmation sits above the field in [`Self::handle`] and does not take
  /// text.
  pub fn captures_text(&self) -> bool {
    self.pending.is_none() && self.timeout_input.is_some()
  }

  /// Processes `handle` in this module's event flow. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn handle(&mut self, key: KeyCode) -> bool {
    if self.pending.is_some() {
      match self.confirmation.handle(key) {
        ConfirmOutcome::Confirmed => {
          self.confirmation = ConfirmState::new();
          self.start_pending();
        }
        ConfirmOutcome::Cancelled => {
          self.pending = None;
          self.confirmation = ConfirmState::new();
        }
        ConfirmOutcome::Pending => {}
      }
      return false;
    }
    if let Some(input) = self.timeout_input.as_mut() {
      match key {
        KeyCode::Char(c) if accepts_input_char(self.input_mode, input, c) => input.push(c),
        KeyCode::Backspace => {
          input.pop();
        }
        KeyCode::Enter => self.apply_input(),
        KeyCode::Esc => {
          self.timeout_input = None;
          self.input_mode = None;
        }
        _ => {}
      }
      return false;
    }
    if self.transaction_open {
      match key {
        KeyCode::Esc => {
          self.transaction_open = false;
          self.transaction_live = None;
          self.transaction_scroll = 0;
          self.transaction_follow = false;
        }
        KeyCode::Up | KeyCode::Char('k') => {
          self.transaction_follow = false;
          self.transaction_scroll = self.transaction_scroll.saturating_sub(1);
        }
        KeyCode::Down | KeyCode::Char('j') => {
          self.transaction_follow = false;
          self.transaction_scroll = self.transaction_scroll.saturating_add(1);
        }
        KeyCode::PageUp => {
          self.transaction_follow = false;
          self.transaction_scroll = self.transaction_scroll.saturating_sub(10);
        }
        KeyCode::PageDown => {
          self.transaction_follow = false;
          self.transaction_scroll = self.transaction_scroll.saturating_add(10);
        }
        KeyCode::Home => {
          self.transaction_follow = false;
          self.transaction_scroll = 0;
        }
        KeyCode::End => {
          if let Some(live) = &self.transaction_live {
            self.transaction_scroll = transaction_bottom_offset(&live.output());
          }
        }
        _ => {}
      }
      return false;
    }
    let rows = self.rows();
    match key {
      // Tab only switches tabs or panes; Boot has none.
      KeyCode::Tab | KeyCode::BackTab => return false,
      KeyCode::Char('r') => {
        self.reload();
        return false;
      }
      _ => {}
    }
    let page_size = usize::from(self.list_height.max(1));
    match self.menu.handle(key, &rows, page_size) {
      MenuEvent::Back => return self.back(),
      // While a snapshot or a boot change runs, pages still open but no
      // boot change starts.
      MenuEvent::Activate(item) | MenuEvent::Toggle(item) | MenuEvent::Confirm(item)
        if item.opens_page() || !self.busy() =>
      {
        self.activate(item)
      }
      MenuEvent::Activate(_) | MenuEvent::Toggle(_) | MenuEvent::Confirm(_) => {}
      MenuEvent::Adjust(..) | MenuEvent::Moved | MenuEvent::None => {}
    }
    false
  }

  /// Opens `page` with the cursor on its first selectable row.
  fn go(&mut self, page: BootPage) {
    self.page = page;
    self.menu = MenuState::default();
  }

  /// Esc/`←`: one level up, with the cursor back on the row that opened the
  /// page; `true` leaves the Boot home.
  fn back(&mut self) -> bool {
    let (parent, origin) = match self.page {
      BootPage::Home => return true,
      BootPage::Summary => (BootPage::Home, Item::Summary),
      BootPage::Kernel => (BootPage::Home, Item::Kernels),
      BootPage::Bootloader => (BootPage::Home, Item::Bootloader),
      BootPage::Initramfs => (BootPage::Home, Item::Initramfs),
      BootPage::Plymouth => (BootPage::Home, Item::Plymouth),
      BootPage::KernelDetail(index) => (BootPage::Kernel, Item::Kernel(index)),
      BootPage::BootloaderDetail(index) => (BootPage::Bootloader, Item::Entry(index)),
      BootPage::InitramfsDetail(index) => (BootPage::Initramfs, Item::Preset(index)),
    };
    self.go(parent);
    let rows = self.rows();
    self.menu.select(&rows, &origin);
    false
  }

  /// Runs the row `item`.
  fn activate(&mut self, item: Item) {
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
  fn open_section(&mut self, page: BootPage) {
    self.go(page);
    self.reload();
  }

  /// Asks for confirmation before running `action`.
  fn request(&mut self, action: BootAction) {
    self.confirmation = ConfirmState::new();
    self.pending = Some(Pending::Action(action));
  }

  /// The systemd-boot entry that `Set default` uses on the open page.
  fn default_target(&self) -> Option<&crate::model::BootEntry> {
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
  fn default_unavailable_reason(&self) -> Option<&'static str> {
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

  /// Executes the `request_default` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn request_default(&mut self) {
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
  /// Executes the `entry_for_kernel` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn entry_for_kernel(
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
  /// Applies the `apply_timeout_input` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn apply_timeout_input(&mut self) {
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
  /// Applies the `apply_input` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn apply_input(&mut self) {
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
  /// Executes the `grub_value` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn grub_value(&self, key: &str) -> String {
    self
      .snapshot
      .bootloader_info
      .grub_values
      .iter()
      .find_map(|(name, value)| (name == key).then(|| value.clone()))
      .unwrap_or_default()
  }
  /// Executes the `start_pending` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn start_pending(&mut self) {
    let Some(Pending::Action(action)) = self.pending.take() else {
      return;
    };
    self.status = Some(StatusMessage {
      kind: StatusKind::Info,
      text: tr(self.lang, "control_center.applying_boot_change").into(),
    });
    let live = LiveProcess::new();
    self.transaction_live = Some(live.clone());
    self.transaction_open = true;
    self.transaction_scroll = 0;
    self.transaction_follow = true;
    self.action = Some(self.jobs.spawn(move |_| run_action(action, live)));
  }
  /// The footer derived from the selected row.
  fn footer_hints(&self, rows: &[Row<Item>]) -> String {
    if self.pending.is_some() {
      return confirm_hints(self.lang);
    }
    let mut menu = self.menu;
    menu.normalize(rows);
    hints(
      self.lang,
      &HintContext {
        row: menu.selected_kind(rows),
        can_go_back: true,
        refresh: true,
        ..HintContext::default()
      },
    )
  }
  /// Renders `draw` while respecting the current domain state and semantic theme. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn draw(&mut self, frame: &mut Frame) {
    let area = frame.area();
    let rows = self.rows();
    self.menu.normalize(&rows);
    let body = shell(
      frame,
      area,
      &self.theme,
      self.lang,
      &self.breadcrumb(),
      &self.footer_hints(&rows),
    );
    self.list_height = body.height;
    draw_menu(
      frame,
      body,
      &self.theme,
      &rows,
      &mut self.menu,
      MenuStyle {
        icons: AppConfig::icons_enabled(),
      },
    );
    self.overlays(frame, area, body);
  }
  /// Draws the input field, the confirmation, the status and the live
  /// transaction output over the page.
  fn overlays(&self, frame: &mut Frame, area: ratatui::layout::Rect, body: ratatui::layout::Rect) {
    if let Some(input) = &self.timeout_input {
      let width = if self.input_mode == Some(InputMode::GrubCmdline) {
        area
          .width
          .saturating_sub(4)
          .clamp(48, INPUT_POPUP_MAX_WIDTH)
      } else {
        48
      };
      let popup = argvus_tui::chrome::centered(area, width, 7);
      frame.render_widget(Clear, popup);
      frame.render_widget(
        Paragraph::new(vec![
          Line::from(if self.input_mode == Some(InputMode::GrubCmdline) {
            tr(self.lang, "control_center.new_kernel_command_line")
          } else {
            tr(self.lang, "control_center.new_timeout_in_seconds")
          }),
          Line::from(input_tail(
            input,
            usize::from(popup.width.saturating_sub(2)),
          )),
          Line::from(tr(self.lang, "control_center.enter_apply_esc_cancel")),
        ])
        .block(Block::bordered().title(
          if self.input_mode == Some(InputMode::GrubCmdline) {
            tr(self.lang, "control_center.kernel_command_line")
          } else {
            tr(self.lang, "control_center.timeout")
          },
        )),
        popup,
      );
    }
    if let Some(Pending::Action(action)) = &self.pending {
      let message = action_message(self.lang, action);
      draw_confirm(
        frame,
        area,
        &self.theme,
        ConfirmDialog {
          title: tr(self.lang, "control_center.confirm_boot_operation"),
          message: &message,
          confirm: tr(self.lang, "control_center.continue"),
          cancel: tr(self.lang, "control_center.cancel"),
          danger: action.is_destructive(),
          deadline: None,
        },
        &self.confirmation,
      );
    }
    if let Some(status_message) = &self.status {
      status(frame, body, &self.theme, status_message);
    }
    if self.transaction_open
      && let Some(output) = self.transaction_live.as_ref().map(|live| live.output())
    {
      let popup = chrome::centered(
        frame.area(),
        TRANSACTION_POPUP_WIDTH,
        TRANSACTION_POPUP_HEIGHT,
      );
      let total = transaction_wrapped_lines(&output);
      let shown_total = total.max(1);
      let current = (self.transaction_scroll as usize + 1).min(shown_total);
      frame.render_widget(Clear, popup);
      frame.render_widget(
        Paragraph::new(output.as_str())
          .block(
            Block::new()
              .borders(Borders::ALL)
              .title(Line::styled(
                format!(
                  " {} · {} {} {} {} ",
                  tr(self.lang, "control_center.process"),
                  tr(self.lang, "control_center.line"),
                  current,
                  tr(self.lang, "control_center.of"),
                  shown_total,
                ),
                Style::new().fg(self.theme.accent),
              ))
              .border_style(Style::new().fg(self.theme.border_active))
              .style(Style::new().bg(self.theme.background))
              .title_bottom(Line::from(tr(
                self.lang,
                "control_center.scroll_pgup_pgdn_home_end_esc_close",
              ))),
          )
          .scroll((self.transaction_scroll, 0))
          .wrap(Wrap { trim: false }),
        popup,
      );
    }
  }
  /// Rows of the current page.
  fn rows(&self) -> Vec<Row<Item>> {
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
  /// A translated field label without the trailing colon some catalog
  /// entries carry, since Info rows draw label and value in columns.
  fn field(&self, key: &str) -> &'static str {
    tr(self.lang, key).trim_end_matches(':')
  }
  /// Executes the `home_rows` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn home_rows(&self) -> Vec<Row<Item>> {
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
  fn summary_rows(&self) -> Vec<Row<Item>> {
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
  /// `Current · Default` badges of a kernel, or an empty string.
  fn kernel_badges(&self, kernel: &crate::model::KernelInfo) -> String {
    let mut badges = Vec::new();
    if kernel.current {
      badges.push(tr(self.lang, "control_center.current"));
    }
    if kernel.default {
      badges.push(tr(self.lang, "control_center.default"));
    }
    badges.join(" · ")
  }
  /// One submenu per installed kernel, with its badges on the right.
  fn kernel_rows(&self) -> Vec<Row<Item>> {
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
  fn set_default_row(&self) -> Row<Item> {
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
  /// Executes the `kernel_detail` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn kernel_detail(&self, index: usize) -> Vec<Row<Item>> {
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
  /// Entries, the timeout and, with GRUB, the kernel command line and the
  /// regeneration in the Danger zone.
  fn bootloader_rows(&self) -> Vec<Row<Item>> {
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
  /// Executes the `bootloader_detail` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn bootloader_detail(&self, index: usize) -> Vec<Row<Item>> {
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
  /// Presets and, in the Danger zone, the regeneration of every image.
  fn initramfs_rows(&self) -> Vec<Row<Item>> {
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
  /// Executes the `initramfs_detail` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn initramfs_detail(&self, index: usize) -> Vec<Row<Item>> {
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
  /// One choice per installed theme; `●` marks the current one.
  fn plymouth_rows(&self) -> Vec<Row<Item>> {
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
  /// Retrieves data for `loader_label` without mixing collection with TUI rendering. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn loader_label(&self) -> &'static str {
    match self.snapshot.bootloader {
      BootloaderKind::SystemdBoot => "systemd-boot",
      BootloaderKind::Grub => "GRUB",
      BootloaderKind::Unknown => tr(self.lang, "control_center.unknown"),
    }
  }
  /// Executes the `breadcrumb` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn breadcrumb(&self) -> String {
    let root = format!(
      "{} > {}",
      HomeCardId::System.label(self.lang),
      tr(self.lang, "control_center.boot")
    );
    if self.page == BootPage::Home {
      root
    } else {
      format!("{root} > {}", self.page_label())
    }
  }
  /// Executes the `page_label` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn page_label(&self) -> &'static str {
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

/// Shown for a missing value.
const MISSING: &str = "—";

/// Space-separated values, or [`MISSING`] when there are none.
fn joined(values: &[String]) -> String {
  if values.is_empty() {
    MISSING.into()
  } else {
    values.join(" ")
  }
}

/// Executes the `yes_no` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn yes_no(lang: Lang, value: bool) -> String {
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
/// Executes the `action_message` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn action_message(lang: Lang, action: &BootAction) -> String {
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
fn boot_request(action: BootAction) -> Result<(String, PrivilegedRequest), String> {
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

/// Runs `action` through `<executable> system-settings boot ...`, which
/// elevates itself with `pkexec`, streaming its output into `live`.
fn run_action(action: BootAction, live: LiveProcess) -> Result<(ActionResult, String), String> {
  let (echo, request) = boot_request(action)?;
  live.push_line(&echo);
  let executable = std::env::current_exe()
    .map_err(|error| error.to_string())?
    .to_string_lossy()
    .into_owned();
  let output =
    SystemSettingsOperation::new(SystemProcessRunner, executable).execute_live(&request, &live)?;
  if output.status == Some(0) {
    let stdout = terminal_text(&String::from_utf8_lossy(&output.stdout))
      .trim()
      .to_owned();
    Ok((
      ActionResult::Success,
      if stdout.is_empty() {
        "ok".to_owned()
      } else {
        stdout
      },
    ))
  } else {
    Err(
      terminal_text(&String::from_utf8_lossy(&output.stderr))
        .trim()
        .into(),
    )
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::model::{BootEntry, KernelInfo};
  use argvus_tui::menu::RowKind;
  use ratatui::{Terminal, backend::TestBackend};
  use std::time::Duration;

  fn app() -> BootApp {
    BootApp::new(
      Lang::for_locale("en-US"),
      Theme::load(),
      Capabilities::default(),
    )
  }

  fn screen(app: &mut BootApp, width: u16, height: u16) -> String {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal.draw(|frame| app.draw(frame)).unwrap();
    terminal
      .backend()
      .buffer()
      .content
      .iter()
      .map(|cell| cell.symbol())
      .collect()
  }

  /// Moves the cursor to `item` and presses Enter.
  fn press(app: &mut BootApp, item: Item) {
    let rows = app.rows();
    assert!(app.menu.select(&rows, &item), "{item:?} is not selectable");
    app.handle(KeyCode::Enter);
  }

  fn row(rows: &[Row<Item>], item: Item) -> &Row<Item> {
    rows
      .iter()
      .find(|row| row.id() == Some(&item))
      .unwrap_or_else(|| panic!("{item:?} is missing"))
  }

  fn ids(rows: &[Row<Item>]) -> Vec<Item> {
    rows.iter().filter_map(|row| row.id().copied()).collect()
  }

  fn systemd_app() -> BootApp {
    let mut app = app();
    app.snapshot.bootloader = BootloaderKind::SystemdBoot;
    app.snapshot.bootloader_info.entries = vec![BootEntry {
      id: "arch.conf".into(),
      title: "Arch".into(),
      linux: Some("/vmlinuz-linux".into()),
      is_default: true,
      ..Default::default()
    }];
    app.snapshot.kernels = vec![KernelInfo {
      package: "linux".into(),
      version: "6.1".into(),
      current: true,
      uki: Some("/boot/EFI/Linux/linux.efi".into()),
      ..Default::default()
    }];
    app
  }

  /// A snapshot job that stays running long enough for the test.
  fn running_job(app: &mut BootApp) {
    app.job = Some(app.jobs.spawn(|_| {
      std::thread::sleep(Duration::from_secs(2));
      Ok(Ok(BootSnapshot::default()))
    }));
  }

  #[test]
  fn boot_home_rows_are_submenus_with_their_state() {
    let mut app = app();
    app.snapshot.firmware = "UEFI".into();
    app.snapshot.current_kernel = "linux-lts 6.18".into();
    app.snapshot.secure_boot = Some(false);
    app.snapshot.bootloader = BootloaderKind::SystemdBoot;
    app.snapshot.bootloader_info.timeout = Some(3);
    app.snapshot.initramfs.available = true;
    app.snapshot.initramfs.presets = vec!["linux-lts.preset".into()];
    let rows = app.rows();
    assert_eq!(
      ids(&rows),
      [
        Item::Summary,
        Item::Kernels,
        Item::Bootloader,
        Item::Initramfs,
        Item::Plymouth
      ]
    );
    assert!(rows.iter().all(|row| row.kind() == RowKind::Submenu));
    let detail = |item| {
      row(&rows, item)
        .detail_text()
        .unwrap_or_default()
        .to_owned()
    };
    assert!(detail(Item::Summary).contains("UEFI"));
    assert!(detail(Item::Kernels).contains("linux-lts 6.18"));
    assert!(
      detail(Item::Bootloader).contains("systemd-boot") && detail(Item::Bootloader).contains("3 s")
    );
    assert!(detail(Item::Initramfs).contains('1'));
    assert!(
      detail(Item::Plymouth).contains("Not installed")
        || detail(Item::Plymouth).contains("Não instalado")
    );
    assert_eq!(row(&rows, Item::Bootloader).icon_glyph(), Some(icons::BOOT));
  }

  #[test]
  fn boot_home_dashboard_uses_theme_and_falls_back_to_installed_flag() {
    let mut app = app();
    app.snapshot.plymouth.installed = true;
    let rows = app.rows();
    let plymouth = row(&rows, Item::Plymouth).detail_text().unwrap_or_default();
    assert!(plymouth.contains("Installed") || plymouth.contains("Instalado"));
    app.snapshot.plymouth.current_theme = Some("argvus".into());
    let rows = app.rows();
    assert_eq!(row(&rows, Item::Plymouth).detail_text(), Some("argvus"));
  }

  #[test]
  fn boot_kernel_rows_carry_current_and_default_badges() {
    let mut app = app();
    app.snapshot.kernels = vec![
      KernelInfo {
        package: "linux".into(),
        version: "6.1".into(),
        current: true,
        ..Default::default()
      },
      KernelInfo {
        package: "linux-lts".into(),
        version: "6.18".into(),
        default: true,
        ..Default::default()
      },
      KernelInfo {
        package: "linux-zen".into(),
        version: "6.19".into(),
        ..Default::default()
      },
    ];
    app.page = BootPage::Kernel;
    let rows = app.rows();
    assert_eq!(
      ids(&rows),
      [Item::Kernel(0), Item::Kernel(1), Item::Kernel(2)]
    );
    assert_eq!(rows[0].label(), "linux 6.1");
    assert_eq!(
      rows[0].detail_text(),
      Some(tr(app.lang, "control_center.current"))
    );
    assert_eq!(
      rows[1].detail_text(),
      Some(tr(app.lang, "control_center.default"))
    );
    assert_eq!(rows[2].detail_text(), None);
    assert!(rows.iter().all(|row| row.icon_glyph().is_none()));
  }

  #[test]
  fn kernel_detail_shows_info_and_sets_the_mapped_entry_as_default() {
    let mut app = systemd_app();
    app.page = BootPage::KernelDetail(0);
    let rows = app.rows();
    assert!(
      rows
        .iter()
        .any(|row| row.detail_text() == Some("/boot/EFI/Linux/linux.efi"))
    );
    assert!(
      rows
        .iter()
        .filter(|row| row.id().is_none())
        .all(|row| !row.is_selectable())
    );
    let last = rows.last().unwrap();
    assert_eq!(last.id(), Some(&Item::SetDefault));
    assert_eq!(last.kind(), RowKind::Action);
    assert_eq!(last.icon_glyph(), Some(icons::STAR));
    assert!(last.is_enabled());
    press(&mut app, Item::SetDefault);
    assert!(matches!(
      &app.pending,
      Some(Pending::Action(BootAction::SystemdDefault(id))) if id == "arch.conf"
    ));
  }

  #[test]
  fn set_default_is_disabled_with_the_reason_when_no_entry_maps() {
    let mut app = systemd_app();
    app.snapshot.bootloader_info.entries[0].linux = Some("/vmlinuz-linux-zen".into());
    app.page = BootPage::KernelDetail(0);
    let rows = app.rows();
    let set_default = row(&rows, Item::SetDefault);
    assert!(!set_default.is_enabled());
    assert_eq!(
      set_default.detail_text(),
      Some(tr(
        app.lang,
        "control_center.unavailable_no_systemd_boot_entry"
      ))
    );

    app.snapshot.bootloader = BootloaderKind::Unknown;
    let rows = app.rows();
    assert_eq!(
      row(&rows, Item::SetDefault).detail_text(),
      Some(tr(
        app.lang,
        "control_center.unavailable_unknown_bootloader"
      ))
    );
    // Disabled rows are skipped: the page has no cursor.
    app.handle(KeyCode::Enter);
    assert!(app.pending.is_none());
  }

  #[test]
  fn boot_pages_are_keyboard_navigable_and_back_returns_to_the_origin() {
    let mut app = systemd_app();
    app.handle(KeyCode::Down);
    app.handle(KeyCode::Enter);
    assert_eq!(app.page, BootPage::Kernel);
    app.job = None;
    app.handle(KeyCode::Right);
    assert_eq!(app.page, BootPage::KernelDetail(0));
    app.handle(KeyCode::Esc);
    assert_eq!(app.page, BootPage::Kernel);
    assert_eq!(app.menu.selected_id(&app.rows()), Some(Item::Kernel(0)));
    app.handle(KeyCode::Left);
    assert_eq!(app.page, BootPage::Home);
    assert_eq!(app.menu.selected_id(&app.rows()), Some(Item::Kernels));
    assert!(app.handle(KeyCode::Esc));
  }

  #[test]
  fn systemd_entry_default_goes_through_the_confirmation() {
    let mut app = systemd_app();
    app.page = BootPage::Bootloader;
    press(&mut app, Item::Entry(0));
    assert_eq!(app.page, BootPage::BootloaderDetail(0));
    press(&mut app, Item::SetDefault);
    assert!(app.pending.is_some());
    app.handle(KeyCode::Tab);
    assert!(app.confirmation.is_confirm_focused());
    app.handle(KeyCode::BackTab);
    assert!(!app.confirmation.is_confirm_focused());
    app.handle(KeyCode::Enter);
    assert!(app.pending.is_none());
    assert!(app.action.is_none());
    app.handle(KeyCode::Enter);
    assert!(app.pending.is_some());
    app.handle(KeyCode::Esc);
    assert!(app.pending.is_none());
  }

  #[test]
  fn confirmation_starts_on_cancel_and_n_cancels() {
    let mut app = app();
    app.page = BootPage::Initramfs;
    press(&mut app, Item::RegenerateInitramfs);
    assert!(app.pending.is_some());
    assert!(!app.confirmation.is_confirm_focused());
    assert_eq!(app.footer_hints(&app.rows()), confirm_hints(app.lang));
    app.handle(KeyCode::Char('n'));
    assert!(app.pending.is_none());
    assert!(app.action.is_none());
    press(&mut app, Item::RegenerateInitramfs);
    app.handle(KeyCode::Char('k'));
    assert!(app.confirmation.is_confirm_focused());
    app.handle(KeyCode::Esc);
    assert!(app.pending.is_none());
    // A new confirmation starts on Cancel again.
    press(&mut app, Item::RegenerateInitramfs);
    assert!(!app.confirmation.is_confirm_focused());
  }

  #[test]
  fn only_the_regenerations_use_the_danger_style() {
    assert!(BootAction::GrubRegenerate.is_destructive());
    assert!(BootAction::Initramfs.is_destructive());
    assert!(!BootAction::SystemdDefault("arch.conf".into()).is_destructive());
    assert!(!BootAction::SystemdTimeout(3).is_destructive());
    assert!(!BootAction::GrubTimeout(3).is_destructive());
    assert!(!BootAction::GrubCmdline("quiet".into()).is_destructive());
    assert!(!BootAction::Plymouth("argvus".into()).is_destructive());
  }

  #[test]
  fn confirmation_renders_the_boot_change() {
    let mut app = app();
    app.snapshot.plymouth.themes = vec!["argvus".into()];
    app.page = BootPage::Plymouth;
    press(&mut app, Item::Theme(0));
    let text = screen(&mut app, 90, 25);
    assert!(text.contains(tr(app.lang, "control_center.confirm_boot_operation")));
    assert!(text.contains("argvus"));
    assert!(text.contains(tr(app.lang, "control_center.continue")));
  }

  #[test]
  fn timeout_input_is_bounded_and_cancelable() {
    let mut app = app();
    app.snapshot.bootloader = BootloaderKind::SystemdBoot;
    app.page = BootPage::Bootloader;
    press(&mut app, Item::Timeout);
    assert_eq!(app.timeout_input.as_deref(), Some(""));
    app.handle(KeyCode::Char('6'));
    app.handle(KeyCode::Char('1'));
    app.handle(KeyCode::Enter);
    assert!(app.pending.is_none());
    assert!(
      app
        .status
        .as_ref()
        .is_some_and(|status| status.kind == StatusKind::Error)
    );
    press(&mut app, Item::Timeout);
    app.handle(KeyCode::Char('5'));
    app.handle(KeyCode::Enter);
    assert!(matches!(
      app.pending,
      Some(Pending::Action(BootAction::SystemdTimeout(5)))
    ));
    app.handle(KeyCode::Esc);
    press(&mut app, Item::Timeout);
    app.handle(KeyCode::Esc);
    assert!(app.timeout_input.is_none());
    assert_eq!(app.page, BootPage::Bootloader);
  }

  #[test]
  fn bootloader_rows_follow_the_detected_bootloader() {
    let mut app = systemd_app();
    app.snapshot.bootloader_info.timeout = Some(4);
    app.page = BootPage::Bootloader;
    let rows = app.rows();
    assert_eq!(ids(&rows), [Item::Entry(0), Item::Timeout]);
    let timeout = row(&rows, Item::Timeout);
    assert_eq!(timeout.kind(), RowKind::Value { step: None });
    assert_eq!(timeout.detail_text(), Some("4 s"));
    assert_eq!(timeout.icon_glyph(), Some(icons::TIMER));

    let mut grub = super::tests::app();
    grub.snapshot.bootloader = BootloaderKind::Grub;
    grub.snapshot.bootloader_info.grub_values =
      vec![("GRUB_CMDLINE_LINUX_DEFAULT".into(), "quiet".into())];
    grub.page = BootPage::Bootloader;
    let rows = grub.rows();
    assert_eq!(
      ids(&rows),
      [Item::Timeout, Item::KernelCmdline, Item::RegenerateGrub]
    );
    assert_eq!(row(&rows, Item::KernelCmdline).detail_text(), Some("quiet"));
    assert_eq!(
      row(&rows, Item::KernelCmdline).icon_glyph(),
      Some(icons::TERMINAL)
    );
    // The Danger zone closes the page.
    let danger = &rows[rows.len() - 2];
    assert!(danger.is_section());
    assert_eq!(danger.label(), tr(grub.lang, "control_center.danger_zone"));
    let regenerate = rows.last().unwrap();
    assert_eq!(regenerate.kind(), RowKind::Destructive);
    assert_eq!(regenerate.icon_glyph(), Some(icons::SYNC));
    press(&mut grub, Item::RegenerateGrub);
    assert!(matches!(
      grub.pending,
      Some(Pending::Action(BootAction::GrubRegenerate))
    ));
    grub.handle(KeyCode::Esc);
    press(&mut grub, Item::KernelCmdline);
    assert_eq!(grub.timeout_input.as_deref(), Some("quiet"));
    assert_eq!(grub.input_mode, Some(InputMode::GrubCmdline));
  }

  #[test]
  fn timeout_is_disabled_with_the_reason_for_an_unknown_bootloader() {
    let mut app = app();
    app.page = BootPage::Bootloader;
    let rows = app.rows();
    let timeout = row(&rows, Item::Timeout);
    assert!(!timeout.is_enabled());
    assert_eq!(
      timeout.detail_text(),
      Some(tr(
        app.lang,
        "control_center.unavailable_unknown_bootloader"
      ))
    );
    app.handle(KeyCode::Enter);
    assert!(app.timeout_input.is_none());
  }

  #[test]
  fn initramfs_regenerates_from_the_danger_zone() {
    let mut app = app();
    app.snapshot.initramfs.presets = vec!["linux.preset".into()];
    app.page = BootPage::Initramfs;
    let rows = app.rows();
    assert_eq!(ids(&rows), [Item::Preset(0), Item::RegenerateInitramfs]);
    assert!(rows[rows.len() - 2].is_section());
    assert_eq!(rows.last().unwrap().kind(), RowKind::Destructive);
    press(&mut app, Item::RegenerateInitramfs);
    assert!(matches!(
      app.pending,
      Some(Pending::Action(BootAction::Initramfs))
    ));
    app.handle(KeyCode::Esc);
    press(&mut app, Item::Preset(0));
    assert_eq!(app.page, BootPage::InitramfsDetail(0));
    assert!(app.rows().iter().all(|row| !row.is_selectable()));
    app.handle(KeyCode::Esc);
    assert_eq!(app.menu.selected_id(&app.rows()), Some(Item::Preset(0)));
  }

  #[test]
  fn plymouth_themes_are_confirmed_choices() {
    let mut app = app();
    app.snapshot.plymouth.themes = vec!["argvus".into(), "spinner".into()];
    app.snapshot.plymouth.current_theme = Some("argvus".into());
    app.page = BootPage::Plymouth;
    let rows = app.rows();
    assert_eq!(rows[0].kind(), RowKind::Choice { current: true });
    assert_eq!(rows[1].kind(), RowKind::Choice { current: false });
    press(&mut app, Item::Theme(1));
    assert!(matches!(
      &app.pending,
      Some(Pending::Action(BootAction::Plymouth(theme))) if theme == "spinner"
    ));
  }

  #[test]
  fn summary_is_read_only_and_has_no_cursor() {
    let mut app = systemd_app();
    app.page = BootPage::Summary;
    let rows = app.rows();
    assert!(rows.iter().all(|row| !row.is_selectable()));
    assert!(
      rows
        .iter()
        .any(|row| row.detail_text() == Some("systemd-boot"))
    );
    app.normalize();
    assert!(app.menu.is_scroll_only());
  }

  #[test]
  fn boot_changes_wait_for_a_running_snapshot_but_pages_open() {
    let mut app = app();
    app.snapshot.initramfs.presets = vec!["linux.preset".into()];
    app.page = BootPage::Initramfs;
    running_job(&mut app);
    press(&mut app, Item::RegenerateInitramfs);
    assert!(app.pending.is_none());
    press(&mut app, Item::Preset(0));
    assert_eq!(app.page, BootPage::InitramfsDetail(0));
    assert!(!app.handle(KeyCode::Esc));
    assert!(!app.handle(KeyCode::Esc));
    assert_eq!(app.page, BootPage::Home);
    press(&mut app, Item::Kernels);
    assert_eq!(app.page, BootPage::Kernel);
  }

  #[test]
  fn tab_does_nothing_without_tabs() {
    let mut app = systemd_app();
    app.page = BootPage::Bootloader;
    app.handle(KeyCode::Down);
    let before = app.menu;
    app.handle(KeyCode::Tab);
    app.handle(KeyCode::BackTab);
    assert_eq!(app.menu, before);
    assert_eq!(app.page, BootPage::Bootloader);
  }

  #[test]
  fn boot_uses_shared_chrome_and_contextual_footer() {
    let mut app = app();
    let text = screen(&mut app, 90, 25);
    assert!(text.contains("ARGVUS"));
    assert!(text.contains("Boot"));
    assert!(text.contains(tr(app.lang, "control_center.hint.open")));
    assert!(text.contains(tr(app.lang, "control_center.hint.refresh")));
    assert!(text.contains("Summary") || text.contains("Resumo"));
  }

  #[test]
  fn boot_pages_have_no_button_bar() {
    let mut app = app();
    app.snapshot.bootloader = BootloaderKind::Grub;
    app.page = BootPage::Bootloader;
    let text = screen(&mut app, 90, 25);
    assert!(!text.contains("[ "), "{text}");
    assert!(text.contains(tr(app.lang, "control_center.kernel_command_line")));
  }

  #[test]
  fn kernel_command_line_field_takes_text() {
    let mut app = BootApp::new(
      Lang::for_locale("en-US"),
      Theme::load(),
      Capabilities::default(),
    );
    app.snapshot.bootloader = BootloaderKind::Grub;
    app.snapshot.bootloader_info.grub_values =
      vec![("GRUB_CMDLINE_LINUX_DEFAULT".into(), "loglevel=3".into())];
    app.timeout_input = Some(app.grub_value("GRUB_CMDLINE_LINUX_DEFAULT"));
    app.input_mode = Some(InputMode::GrubCmdline);
    for character in " quiet splash".chars() {
      app.handle(KeyCode::Char(character));
    }
    assert_eq!(
      app.timeout_input.as_deref(),
      Some("loglevel=3 quiet splash")
    );
    app.handle(KeyCode::Backspace);
    app.handle(KeyCode::Char('h'));
    app.handle(KeyCode::Enter);
    assert!(matches!(
      &app.pending,
      Some(Pending::Action(BootAction::GrubCmdline(value))) if value == "loglevel=3 quiet splash"
    ));
  }

  #[test]
  fn kernel_command_line_field_stops_at_the_backend_limit() {
    let full = "a".repeat(KERNEL_CMDLINE_MAX);
    assert!(!accepts_input_char(
      Some(InputMode::GrubCmdline),
      &full,
      'b'
    ));
    assert!(accepts_input_char(
      Some(InputMode::GrubCmdline),
      &full[1..],
      'b'
    ));
    assert!(!accepts_input_char(Some(InputMode::Timeout), "12", '3'));
    assert!(!accepts_input_char(Some(InputMode::Timeout), "", 'a'));
  }

  #[test]
  fn kernel_command_line_rejects_quotes_when_applied() {
    let mut app = BootApp::new(
      Lang::for_locale("en-US"),
      Theme::load(),
      Capabilities::default(),
    );
    app.timeout_input = Some(String::new());
    app.input_mode = Some(InputMode::GrubCmdline);
    app.handle(KeyCode::Char('"'));
    app.handle(KeyCode::Enter);
    assert!(app.pending.is_none());
    assert!(
      app
        .status
        .as_ref()
        .is_some_and(|status| status.kind == StatusKind::Error)
    );
  }

  #[test]
  fn input_tail_keeps_the_cursor_visible() {
    assert_eq!(input_tail("quiet", 10), "quiet_");
    assert_eq!(input_tail("loglevel=3 quiet splash", 7), "splash_");
  }

  #[test]
  fn boot_actions_build_the_privileged_system_settings_command() {
    let cases = [
      (
        BootAction::SystemdDefault("arch.conf".into()),
        "$ bootctl set-default arch.conf",
        vec!["systemd-default", "arch.conf"],
      ),
      (
        BootAction::SystemdTimeout(5),
        "$ boot timeout = 5s",
        vec!["systemd-timeout", "5"],
      ),
      (
        BootAction::GrubTimeout(3),
        "$ GRUB_TIMEOUT = 3s",
        vec!["grub-timeout", "3"],
      ),
      (
        BootAction::GrubCmdline("quiet splash".into()),
        "$ GRUB_CMDLINE_LINUX_DEFAULT",
        vec!["grub-cmdline", "quiet splash"],
      ),
      (
        BootAction::GrubRegenerate,
        "$ grub-mkconfig",
        vec!["grub-regenerate"],
      ),
      (
        BootAction::Initramfs,
        "$ mkinitcpio -P",
        vec!["initramfs-regenerate"],
      ),
      (
        BootAction::Plymouth("bgrt".into()),
        "$ plymouth-set-default-theme -R bgrt",
        vec!["plymouth-theme", "bgrt"],
      ),
    ];
    for (action, expected_echo, expected_args) in cases {
      let (echo, request) = boot_request(action).unwrap();
      assert_eq!(echo, expected_echo);
      let process =
        SystemSettingsOperation::new(SystemProcessRunner, "/usr/bin/argvus-control-center")
          .process_for(&request)
          .unwrap();
      assert_eq!(process.program, "/usr/bin/argvus-control-center");
      let mut args = vec!["system-settings", "boot"];
      args.extend(expected_args);
      assert_eq!(process.args, args);
    }
  }

  #[test]
  /// Executes the `boot_start_pending_opens_the_process_window_immediately` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn boot_start_pending_opens_the_process_window_immediately() {
    let mut app = BootApp::new(
      Lang::for_locale("en-US"),
      Theme::load(),
      Capabilities::default(),
    );
    // The operation is only checked as built; test builds start no process
    // (the core `testing` feature), so the job below runs nothing.
    let (_, request) = boot_request(BootAction::Initramfs).unwrap();
    let process =
      SystemSettingsOperation::new(SystemProcessRunner, "/usr/bin/argvus-control-center")
        .process_for(&request)
        .unwrap();
    assert_eq!(
      process.args,
      ["system-settings", "boot", "initramfs-regenerate"]
    );
    app.pending = Some(Pending::Action(BootAction::Initramfs));
    app.start_pending();
    assert!(
      app.transaction_open,
      "window must appear when the process starts"
    );
    assert!(app.transaction_live.is_some(), "live sink must be shared");
    assert!(app.transaction_follow);
    assert!(app.action.is_some());
    app.handle(KeyCode::Esc);
    assert!(!app.transaction_open);
  }

  #[test]
  /// Executes the `boot_transaction_window_scrolls_and_closes` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn boot_transaction_window_scrolls_and_closes() {
    let mut app = BootApp::new(
      Lang::for_locale("en-US"),
      Theme::load(),
      Capabilities::default(),
    );
    let live = LiveProcess::new();
    for _ in 0..40 {
      live.push_line("line");
    }
    app.transaction_live = Some(live.clone());
    app.transaction_open = true;
    assert!(app.transaction_open);
    app.handle(KeyCode::Down);
    assert_eq!(app.transaction_scroll, 1);
    app.handle(KeyCode::Up);
    assert_eq!(app.transaction_scroll, 0);
    app.handle(KeyCode::PageDown);
    assert_eq!(app.transaction_scroll, 10);
    app.handle(KeyCode::PageUp);
    assert_eq!(app.transaction_scroll, 0);
    app.handle(KeyCode::End);
    assert_eq!(
      app.transaction_scroll as usize,
      transaction_bottom_offset(&live.output()) as usize
    );
    app.handle(KeyCode::Esc);
    assert!(!app.transaction_open);
    assert_eq!(app.transaction_scroll, 0);
  }

  #[test]
  /// Executes the `boot_transaction_window_renders_process_title_and_output` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn boot_transaction_window_renders_process_title_and_output() {
    let mut app = BootApp::new(
      Lang::for_locale("en-US"),
      Theme::load(),
      Capabilities::default(),
    );
    let live = LiveProcess::new();
    live.push_line("$ plymouth-set-default-theme -R argvus");
    live.push_line("==> Building image");
    app.transaction_live = Some(live);
    app.transaction_open = true;
    let mut terminal = Terminal::new(TestBackend::new(110, 25)).unwrap();
    terminal.draw(|frame| app.draw(frame)).unwrap();
    let text = terminal
      .backend()
      .buffer()
      .content
      .iter()
      .map(|cell| cell.symbol())
      .collect::<String>();
    assert!(text.contains("Process") || text.contains("Processo"));
    assert!(text.contains("$ plymouth-set-default-theme -R argvus"));
    assert!(text.contains("scr") || text.contains("rol"));
  }
}
