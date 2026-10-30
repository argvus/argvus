//! Ratatui shell which embeds the themed btop screen through a PTY.

use argvus_i18n::I18n;
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
  style::{Modifier, Style},
  text::Line,
  widgets::{Paragraph, Wrap},
};
use std::{
  io::{Read, Write},
  process::ExitCode,
  sync::mpsc::{self, Receiver},
  thread::{self, JoinHandle},
  time::Duration,
};

const EVENT_POLL: Duration = Duration::from_millis(30);
const GPU_PANEL_TIMEOUT: Duration = Duration::from_secs(3);
const GPU_BOX_MARKER: &str = "gpu0";

pub struct Catalog(Option<I18n>);

impl Catalog {
  pub fn load() -> Self {
    Self(
      I18n::new("system-monitor")
        .inspect_err(|error| log::error!("cannot load system-monitor translations: {error}"))
        .ok(),
    )
  }

  pub fn tr(&self, key: &str) -> String {
    self
      .0
      .as_ref()
      .map_or_else(|| key.to_owned(), |catalog| catalog.tr(key))
  }
}

enum PanelState {
  Running,
  LaunchError,
  MetricsUnavailable,
}

struct RunningBtop {
  child: Box<dyn Child + Send + Sync>,
  master: Box<dyn MasterPty + Send>,
  writer: Box<dyn Write + Send>,
  output: Receiver<Vec<u8>>,
  reader_thread: Option<JoinHandle<()>>,
}

impl RunningBtop {
  fn spawn(program: &str, arguments: &[String], area: Rect) -> Result<Self, String> {
    let system = native_pty_system();
    let pair = system
      .openpty(pty_size(area))
      .map_err(|error| format!("cannot open btop PTY: {error}"))?;
    let mut command = CommandBuilder::new(program);
    command.args(arguments);
    command.env("TERM", "xterm-256color");
    command.env("COLORTERM", "truecolor");
    let reader = pair
      .master
      .try_clone_reader()
      .map_err(|error| format!("cannot read btop PTY: {error}"))?;
    let writer = pair
      .master
      .take_writer()
      .map_err(|error| format!("cannot write to btop PTY: {error}"))?;
    let (sender, output) = mpsc::channel();
    let reader_thread = thread::Builder::new()
      .name("argvus-btop-output".into())
      .spawn(move || read_output(reader, sender))
      .map_err(|error| format!("cannot create btop reader: {error}"))?;
    let child = match pair.slave.spawn_command(command) {
      Ok(child) => child,
      Err(error) => {
        drop(pair.slave);
        let _ = reader_thread.join();
        return Err(format!("cannot start btop: {error}"));
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
      .map_err(|error| format!("cannot resize btop PTY: {error}"))
  }

  fn write_input(&mut self, input: &[u8]) -> Result<(), String> {
    self
      .writer
      .write_all(input)
      .map_err(|error| format!("cannot send input to btop: {error}"))
  }

  fn stop(&mut self) {
    let _ = self.child.kill();
    let _ = self.child.wait();
    if let Some(reader_thread) = self.reader_thread.take() {
      let _ = reader_thread.join();
    }
  }
}

impl Drop for RunningBtop {
  fn drop(&mut self) {
    self.stop();
  }
}

pub fn run(arguments: &[String], title: String, catalog: Catalog) -> Result<ExitCode, String> {
  run_panel(Some(arguments), false, title, catalog)
}

/// Runs btop while checking that it actually rendered a GPU box.
pub fn run_gpu(arguments: &[String], title: String, catalog: Catalog) -> Result<ExitCode, String> {
  run_panel(Some(arguments), true, title, catalog)
}

/// Shows a branded, closable message when no usable GPU panel can be selected.
pub fn run_unavailable(title: String, catalog: Catalog) -> Result<ExitCode, String> {
  run_panel(None, false, title, catalog)
}

struct PanelOptions<'a> {
  expect_gpu: bool,
  title: &'a str,
  catalog: &'a Catalog,
  theme: &'a Theme,
}

fn run_panel(
  arguments: Option<&[String]>,
  expect_gpu: bool,
  title: String,
  catalog: Catalog,
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
  let (mut state, mut child) = match arguments {
    Some(arguments) => start_btop("btop", arguments, initial_area),
    None => (PanelState::MetricsUnavailable, None),
  };
  let result = event_loop(
    terminal,
    &mut child,
    &mut view,
    PanelOptions {
      expect_gpu,
      title: &title,
      catalog: &catalog,
      theme: &theme,
    },
    &mut state,
  );
  result.map(|()| ExitCode::SUCCESS)
}

fn start_btop(
  program: &str,
  arguments: &[String],
  area: Rect,
) -> (PanelState, Option<RunningBtop>) {
  match RunningBtop::spawn(program, arguments, area) {
    Ok(child) => (PanelState::Running, Some(child)),
    Err(error) => {
      log::error!("cannot start embedded btop: {error}");
      (PanelState::LaunchError, None)
    }
  }
}

fn event_loop(
  terminal: &mut argvus_tui::terminal::TuiTerminal,
  child: &mut Option<RunningBtop>,
  view: &mut TerminalView,
  options: PanelOptions<'_>,
  state: &mut PanelState,
) -> Result<(), String> {
  let started_at = std::time::Instant::now();
  loop {
    if let Some(running) = child.as_mut() {
      while let Ok(output) = running.output.try_recv() {
        view.process(&output);
      }
      if running
        .child
        .try_wait()
        .map_err(|error| format!("cannot check btop process: {error}"))?
        .is_some()
      {
        return Ok(());
      }
    }
    if options.expect_gpu
      && matches!(state, PanelState::Running)
      && started_at.elapsed() >= GPU_PANEL_TIMEOUT
      && !gpu_box_is_visible(view)
    {
      if let Some(running) = child.as_mut() {
        running.stop();
      }
      *child = None;
      *state = PanelState::MetricsUnavailable;
    }
    terminal
      .draw(|frame| {
        draw(
          frame,
          view,
          options.title,
          options.catalog,
          options.theme,
          state,
        )
      })
      .map_err(|error| format!("cannot render btop panel: {error}"))?;
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
            *state = PanelState::LaunchError;
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

fn gpu_box_is_visible(view: &TerminalView) -> bool {
  view.contains_text(GPU_BOX_MARKER)
}

fn draw(
  frame: &mut Frame,
  view: &TerminalView,
  title: &str,
  catalog: &Catalog,
  theme: &Theme,
  state: &PanelState,
) {
  let areas = Layout::vertical([
    Constraint::Length(3),
    Constraint::Min(1),
    Constraint::Length(1),
  ])
  .split(frame.area());
  let live_status = matches!(state, PanelState::Running).then(|| catalog.tr("system-monitor.live"));
  draw_live_header(frame, areas[0], theme, title, live_status.as_deref());
  match state {
    PanelState::Running => view.render(frame, areas[1], theme),
    PanelState::LaunchError => frame.render_widget(
      Paragraph::new(Line::from(catalog.tr("system-monitor.unavailable")))
        .style(Style::new().fg(theme.warning).add_modifier(Modifier::BOLD)),
      areas[1],
    ),
    PanelState::MetricsUnavailable => frame.render_widget(
      Paragraph::new(Line::from(catalog.tr("system-monitor.gpu.unavailable")))
        .wrap(Wrap { trim: true })
        .style(Style::new().fg(theme.warning).add_modifier(Modifier::BOLD)),
      areas[1],
    ),
  }
  draw_live_footer(
    frame,
    areas[2],
    theme,
    &catalog.tr("system-monitor.footer.close"),
  );
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
  matches!(key.code, KeyCode::Esc | KeyCode::Char('q')) && key.kind != KeyEventKind::Release
}

fn encode_key(key: KeyEvent) -> Option<Vec<u8>> {
  let mut bytes = match key.code {
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
  use super::{PanelState, closes_panel, encode_key, gpu_box_is_visible, pty_size, start_btop};
  use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
  use ratatui::layout::Rect;

  #[test]
  fn escape_and_q_close_the_panel() {
    let escape = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
    let quit = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
    assert!(closes_panel(escape));
    assert!(closes_panel(quit));
    assert_eq!(encode_key(quit), Some(b"q".to_vec()));
  }

  #[test]
  fn pty_dimensions_are_nonzero_when_terminal_is_too_small() {
    let size = pty_size(Rect::new(0, 0, 0, 0));
    assert_eq!((size.cols, size.rows), (1, 1));
  }

  #[test]
  fn key_release_does_not_close_panel() {
    let mut key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
    key.kind = KeyEventKind::Release;
    assert!(!closes_panel(key));
    key.code = KeyCode::Char('q');
    assert!(!closes_panel(key));
  }

  #[test]
  fn btop_must_render_a_gpu_slot_in_gpu_mode() {
    let mut view = argvus_tui::terminal_view::TerminalView::new(2, 24);
    view.process(b"CPU MEM NET PROC");
    assert!(!gpu_box_is_visible(&view));
    view.process(b"\x1b[1;1H\x1b[2K\x1b[1mgpu0");
    assert!(gpu_box_is_visible(&view));
  }

  #[test]
  fn missing_btop_is_a_recoverable_panel_state() {
    let (state, child) = start_btop(
      "argvus-btop-command-that-does-not-exist",
      &[],
      Rect::new(0, 0, 80, 20),
    );
    assert!(matches!(state, PanelState::LaunchError));
    assert!(child.is_none());
  }
}
