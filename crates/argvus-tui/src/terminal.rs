//! Terminal lifecycle management.
use std::io;
use std::panic;

use crossterm::{
  cursor::{Hide, Show},
  event::{DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture},
  execute,
  terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use crate::icon_scale::IconScaleBackend;

pub type TuiTerminal = Terminal<IconScaleBackend>;

pub struct TerminalGuard {
  terminal: TuiTerminal,
}

impl TerminalGuard {
  pub fn new() -> io::Result<Self> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    if let Err(error) = execute!(
      stdout,
      EnterAlternateScreen,
      EnableBracketedPaste,
      EnableMouseCapture,
      Hide
    ) {
      let _ = disable_raw_mode();
      return Err(error);
    }
    let backend = IconScaleBackend::new(CrosstermBackend::new(stdout));
    match Terminal::new(backend) {
      Ok(terminal) => Ok(Self { terminal }),
      Err(error) => {
        restore();
        Err(error)
      }
    }
  }

  pub fn terminal_mut(&mut self) -> &mut TuiTerminal {
    &mut self.terminal
  }
}

impl Drop for TerminalGuard {
  fn drop(&mut self) {
    let _ = disable_raw_mode();
    let _ = execute!(
      self.terminal.backend_mut(),
      DisableBracketedPaste,
      DisableMouseCapture,
      Show,
      LeaveAlternateScreen
    );
    let _ = self.terminal.show_cursor();
  }
}

pub fn install_panic_hook() {
  let previous = panic::take_hook();
  panic::set_hook(Box::new(move |info| {
    restore();
    previous(info);
  }));
}

fn restore() {
  let _ = disable_raw_mode();
  let _ = execute!(
    io::stdout(),
    DisableBracketedPaste,
    DisableMouseCapture,
    Show,
    LeaveAlternateScreen
  );
}
