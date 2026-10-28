//! Keys and text input on the Boot page.

use super::render::transaction_bottom_offset;
use super::*;

impl BootApp {
  /// Whether typed characters currently go to the timeout/kernel command
  /// line field, so `q`/`?` must not act as the global quit/help keys. The
  /// confirmation sits above the field in [`Self::handle`] and does not take
  /// text.
  pub fn captures_text(&self) -> bool {
    self.pending.is_none() && self.timeout_input.is_some()
  }

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

  /// Esc/`←`: one level up, with the cursor back on the row that opened the
  /// page; `true` leaves the Boot home.
  pub(super) fn back(&mut self) -> bool {
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
}

/// Whether the open field takes `character`: the timeout takes up to two
/// digits; the kernel command line takes any text up to its limit, and the
/// characters the backend rejects are reported when the value is applied.
pub(super) fn accepts_input_char(mode: Option<InputMode>, input: &str, character: char) -> bool {
  match mode {
    Some(InputMode::GrubCmdline) => input.len() + character.len_utf8() <= KERNEL_CMDLINE_MAX,
    _ => character.is_ascii_digit() && input.len() < TIMEOUT_MAX_DIGITS,
  }
}

/// The end of `input` that fits in `width` cells next to the cursor, so the
/// typing position stays visible in long values.
pub(super) fn input_tail(input: &str, width: usize) -> String {
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
