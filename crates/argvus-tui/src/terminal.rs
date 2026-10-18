//! Implements terminal lifecycle management in crate `argvus tui`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
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

/// Names the type `TuiTerminal`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub type TuiTerminal = Terminal<IconScaleBackend>;

/// Represents `TerminalGuard`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct TerminalGuard {
  terminal: TuiTerminal,
}

impl TerminalGuard {
  /// Constructs `new` with this module's expected initial state. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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

  /// Executes the `terminal_mut` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn terminal_mut(&mut self) -> &mut TuiTerminal {
    &mut self.terminal
  }
}

impl Drop for TerminalGuard {
  /// Executes the `drop` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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

/// Executes the `install_panic_hook` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn install_panic_hook() {
  let previous = panic::take_hook();
  panic::set_hook(Box::new(move |info| {
    restore();
    previous(info);
  }));
}

/// Executes the `restore` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
