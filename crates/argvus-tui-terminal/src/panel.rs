//! Shared framed terminal for interactive child commands.

use argvus_theme::Theme;
use argvus_tui::{
  chrome::{draw_live_footer, draw_live_header},
  terminal::TerminalGuard,
  terminal_view::TerminalView,
};
use crossterm::event::{
  self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent,
};
use portable_pty::{Child, CommandBuilder, MasterPty, PtySize, native_pty_system};
use ratatui::{
  Frame,
  layout::{Constraint, Layout, Rect},
};
use std::{
  io::{Read, Write},
  process::ExitCode,
  sync::mpsc::{self, Receiver},
  thread::{self, JoinHandle},
  time::Duration,
};

const EVENT_POLL: Duration = Duration::from_millis(30);

struct RunningChild {
  child: Box<dyn Child + Send + Sync>,
  master: Box<dyn MasterPty + Send>,
  writer: Box<dyn Write + Send>,
  output: Receiver<Vec<u8>>,
  reader_thread: Option<JoinHandle<()>>,
}

impl RunningChild {
  fn spawn(program: &str, arguments: &[String], area: Rect) -> Result<Self, String> {
    let system = native_pty_system();
    let pair = system
      .openpty(pty_size(area))
      .map_err(|error| format!("cannot open child command PTY: {error}"))?;
    let mut command = CommandBuilder::new(program);
    command.args(arguments);
    command.cwd(std::env::current_dir().map_err(|error| error.to_string())?);
    command.env("TERM", "xterm-256color");
    command.env("COLORTERM", "truecolor");
    let reader = pair
      .master
      .try_clone_reader()
      .map_err(|error| format!("cannot read child command PTY: {error}"))?;
    let writer = pair
      .master
      .take_writer()
      .map_err(|error| format!("cannot write to child command PTY: {error}"))?;
    let (sender, output) = mpsc::channel();
    let reader_thread = thread::Builder::new()
      .name("argvus-framed-terminal-output".into())
      .spawn(move || read_output(reader, sender))
      .map_err(|error| format!("cannot create child command reader: {error}"))?;
    let child = match pair.slave.spawn_command(command) {
      Ok(child) => child,
      Err(error) => {
        drop(pair.slave);
        drop(reader_thread);
        return Err(format!("cannot start child command: {error}"));
      }
    };
    drop(pair.slave);
    Ok(Self {
      child,
      master: pair.master,
      writer,
      output,
      reader_thread: Some(reader_thread),
    })
  }

  fn resize(&mut self, area: Rect) -> Result<(), String> {
    self
      .master
      .resize(pty_size(area))
      .map_err(|error| format!("cannot resize child command PTY: {error}"))
  }

  fn write_input(&mut self, input: &[u8]) -> Result<(), String> {
    self
      .writer
      .write_all(input)
      .map_err(|error| format!("cannot send input to child command: {error}"))
  }

  fn stop(&mut self) {
    let _ = self.child.kill();
    let _ = self.child.wait();
    if let Some(reader_thread) = self.reader_thread.take() {
      drop(reader_thread);
    }
  }
}

impl Drop for RunningChild {
  fn drop(&mut self) {
    self.stop();
  }
}

pub fn run(
  program: &str,
  arguments: &[String],
  title: String,
  footer: String,
) -> Result<ExitCode, String> {
  run_panel(program, arguments, title, footer)
}

struct PanelOptions<'a> {
  title: &'a str,
  footer: &'a str,
  theme: &'a Theme,
}

fn run_panel(
  program: &str,
  arguments: &[String],
  title: String,
  footer: String,
) -> Result<ExitCode, String> {
  argvus_tui::terminal::install_panic_hook();
  let theme = Theme::load();
  let mut guard = TerminalGuard::new().map_err(|error| format!("cannot open TUI: {error}"))?;
  let terminal = guard.terminal_mut();
  let initial_size = terminal
    .size()
    .map_err(|error| format!("cannot read terminal size: {error}"))?;
  let initial_area = content_area(Rect::new(0, 0, initial_size.width, initial_size.height));
  let mut view = TerminalView::new(initial_area.height, initial_area.width);
  let mut child = Some(RunningChild::spawn(program, arguments, initial_area)?);
  let result = event_loop(
    terminal,
    &mut child,
    &mut view,
    PanelOptions {
      title: &title,
      footer: &footer,
      theme: &theme,
    },
  );
  result.map(|()| ExitCode::SUCCESS)
}

fn event_loop(
  terminal: &mut argvus_tui::terminal::TuiTerminal,
  child: &mut Option<RunningChild>,
  view: &mut TerminalView,
  options: PanelOptions<'_>,
) -> Result<(), String> {
  loop {
    if let Some(running) = child.as_mut() {
      while let Ok(output) = running.output.try_recv() {
        view.process(&output);
      }
      if running
        .child
        .try_wait()
        .map_err(|error| format!("cannot check child command process: {error}"))?
        .is_some()
      {
        return Ok(());
      }
    }
    terminal
      .draw(|frame| draw(frame, view, options.title, options.footer, options.theme))
      .map_err(|error| format!("cannot render child command panel: {error}"))?;
    if event::poll(EVENT_POLL).map_err(|error| format!("cannot read TUI events: {error}"))? {
      match event::read().map_err(|error| format!("cannot read TUI event: {error}"))? {
        Event::Key(key) if key.kind != KeyEventKind::Release => {
          if closes_panel(key) {
            return Ok(());
          }
          if let (Some(running), Some(input)) = (child.as_mut(), encode_key(key))
            && let Err(error) = running.write_input(&input)
          {
            log::error!("{error}");
            return Err(error);
          }
        }
        Event::Paste(text) => {
          if let Some(running) = child.as_mut() {
            if view.bracketed_paste() {
              running.write_input(format!("\x1b[200~{text}\x1b[201~").as_bytes())?;
            } else {
              running.write_input(text.as_bytes())?;
            }
          }
        }
        Event::Mouse(mouse) => {
          if let Some(running) = child.as_mut() {
            let terminal_area = terminal.size().map_err(|error| error.to_string())?;
            let content = content_area(Rect::new(0, 0, terminal_area.width, terminal_area.height));
            if let Some(input) = encode_mouse(mouse, content, view_mouse_enabled(view))
              && let Err(error) = running.write_input(&input)
            {
              log::error!("{error}");
            }
          }
        }
        Event::Resize(_, _) => {
          let terminal_area = terminal.size().map_err(|error| error.to_string())?;
          let content = content_area(Rect::new(0, 0, terminal_area.width, terminal_area.height));
          view.resize(content.height, content.width);
          if let Some(running) = child.as_mut() {
            running.resize(content).map_err(|error| {
              log::error!("{error}");
              error
            })?;
          }
          terminal.clear().map_err(|error| error.to_string())?;
        }
        _ => {}
      }
    }
  }
}

fn draw(frame: &mut Frame, view: &TerminalView, title: &str, footer: &str, theme: &Theme) {
  let areas = Layout::vertical([
    Constraint::Length(3),
    Constraint::Min(1),
    Constraint::Length(1),
  ])
  .split(frame.area());
  let live_status: Option<String> = None;
  draw_live_header(frame, areas[0], theme, title, live_status.as_deref());
  view.render(frame, areas[1], theme);
  view.render_cursor(frame, areas[1]);
  draw_live_footer(frame, areas[2], theme, footer);
}

fn content_area(area: Rect) -> Rect {
  Layout::vertical([
    Constraint::Length(3),
    Constraint::Min(1),
    Constraint::Length(1),
  ])
  .split(area)[1]
}

fn pty_size(area: Rect) -> PtySize {
  PtySize {
    rows: area.height.max(1),
    cols: area.width.max(1),
    pixel_width: 0,
    pixel_height: 0,
  }
}

fn read_output(mut reader: Box<dyn Read + Send>, sender: mpsc::Sender<Vec<u8>>) {
  let mut buffer = [0_u8; 8192];
  loop {
    match reader.read(&mut buffer) {
      Ok(0) | Err(_) => break,
      Ok(count) => {
        if sender.send(buffer[..count].to_vec()).is_err() {
          break;
        }
      }
    }
  }
}

fn closes_panel(key: KeyEvent) -> bool {
  key.code == KeyCode::Char('e')
    && key.modifiers.contains(KeyModifiers::CONTROL)
    && key.kind != KeyEventKind::Release
}

fn encode_key(key: KeyEvent) -> Option<Vec<u8>> {
  let mut bytes = match key.code {
    // Escape is a single byte in the child PTY. Keep it distinct from the
    // frame's Ctrl+E close shortcut so agent prompts can be cancelled.
    KeyCode::Esc => b"\x1b".to_vec(),
    KeyCode::Char(character) => {
      if key.modifiers.contains(KeyModifiers::CONTROL) {
        let control = (character as u32 & 0x1f) as u8;
        vec![control]
      } else {
        character.to_string().into_bytes()
      }
    }
    KeyCode::Enter => b"\r".to_vec(),
    KeyCode::Backspace => b"\x7f".to_vec(),
    KeyCode::Tab => b"\t".to_vec(),
    KeyCode::BackTab => b"\x1b[Z".to_vec(),
    KeyCode::Up => b"\x1b[A".to_vec(),
    KeyCode::Down => b"\x1b[B".to_vec(),
    KeyCode::Right => b"\x1b[C".to_vec(),
    KeyCode::Left => b"\x1b[D".to_vec(),
    KeyCode::Home => b"\x1b[H".to_vec(),
    KeyCode::End => b"\x1b[F".to_vec(),
    KeyCode::PageUp => b"\x1b[5~".to_vec(),
    KeyCode::PageDown => b"\x1b[6~".to_vec(),
    KeyCode::Insert => b"\x1b[2~".to_vec(),
    KeyCode::Delete => b"\x1b[3~".to_vec(),
    KeyCode::F(number @ 1..=4) => vec![b'\x1b', b'O', b'P' + number - 1],
    KeyCode::F(number @ 5..=12) => {
      let code = [15, 17, 18, 19, 20, 21, 23, 24][usize::from(number - 5)];
      format!("\x1b[{code}~").into_bytes()
    }
    KeyCode::Null => vec![0],
    _ => return None,
  };
  if key.modifiers.contains(KeyModifiers::ALT) {
    bytes.insert(0, 0x1b);
  }
  Some(bytes)
}

fn view_mouse_enabled(view: &TerminalView) -> bool {
  view.mouse_enabled()
}

fn encode_mouse(event: MouseEvent, area: Rect, enabled: bool) -> Option<Vec<u8>> {
  if !enabled
    || event.column < area.x
    || event.row < area.y
    || event.column >= area.x + area.width
    || event.row >= area.y + area.height
  {
    return None;
  }
  let x = event.column - area.x + 1;
  let y = event.row - area.y + 1;
  let mut button = match event.kind {
    event::MouseEventKind::Down(MouseButton::Left) => 0,
    event::MouseEventKind::Down(MouseButton::Middle) => 1,
    event::MouseEventKind::Down(MouseButton::Right) => 2,
    event::MouseEventKind::Up(_) => 3,
    event::MouseEventKind::Drag(MouseButton::Left) => 32,
    event::MouseEventKind::Drag(MouseButton::Middle) => 33,
    event::MouseEventKind::Drag(MouseButton::Right) => 34,
    event::MouseEventKind::Moved => 35,
    event::MouseEventKind::ScrollUp => 64,
    event::MouseEventKind::ScrollDown => 65,
    event::MouseEventKind::ScrollLeft => 66,
    event::MouseEventKind::ScrollRight => 67,
  };
  if event.modifiers.contains(KeyModifiers::SHIFT) {
    button += 4;
  }
  if event.modifiers.contains(KeyModifiers::ALT) {
    button += 8;
  }
  if event.modifiers.contains(KeyModifiers::CONTROL) {
    button += 16;
  }
  let ending = if matches!(event.kind, event::MouseEventKind::Up(_)) {
    'm'
  } else {
    'M'
  };
  Some(format!("\x1b[<{button};{x};{y}{ending}").into_bytes())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn escape_is_forwarded_to_the_child_while_control_e_closes_the_frame() {
    assert_eq!(
      encode_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)),
      Some(vec![0x1b])
    );
    assert!(closes_panel(KeyEvent::new(
      KeyCode::Char('e'),
      KeyModifiers::CONTROL
    )));
  }

  #[test]
  fn only_control_e_closes_the_agent() {
    assert!(closes_panel(KeyEvent::new(
      KeyCode::Char('e'),
      KeyModifiers::CONTROL
    )));
    for code in [KeyCode::Esc, KeyCode::Char('q'), KeyCode::Char('e')] {
      assert!(!closes_panel(KeyEvent::new(code, KeyModifiers::NONE)));
    }
    assert!(!closes_panel(KeyEvent::new(
      KeyCode::Char('c'),
      KeyModifiers::CONTROL
    )));
  }
}
