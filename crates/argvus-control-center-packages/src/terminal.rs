//! A retained package terminal. Process I/O runs off the UI thread.
use argvus_control_center_core::process::{ProcessRequest, command};
use argvus_tui::terminal_view::TerminalView;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use portable_pty::{Child, ChildKiller, CommandBuilder, MasterPty, PtySize, native_pty_system};
use std::{
  io::{Read, Write},
  sync::{
    Arc, Mutex,
    mpsc::{self, Receiver, SyncSender},
  },
  thread,
};

const HISTORY_LINES: usize = 10_000;
const OUTPUT_CHUNKS: usize = 64;
const INPUT_MESSAGES: usize = 32;
const MAX_INPUT_BYTES: usize = 64 * 1024;

struct Control {
  master: Option<Box<dyn MasterPty + Send>>,
  size: PtySize,
  closed: bool,
  killer: Option<Box<dyn ChildKiller + Send + Sync>>,
}

enum Input {
  Bytes(Vec<u8>),
  Stop,
}

/// The UI owns the emulated screen and bounded queues, never a blocking pipe.
pub(super) struct PackageTerminal {
  pub view: TerminalView,
  output: Receiver<Vec<u8>>,
  input: SyncSender<Input>,
  control: Arc<Mutex<Control>>,
  pub result: Option<bool>,
  pub notice: Option<String>,
}

/// Sent to the existing background job that starts and waits for the command.
pub(super) struct TerminalIo {
  output: SyncSender<Vec<u8>>,
  input: Receiver<Input>,
  stop_input: SyncSender<Input>,
  control: Arc<Mutex<Control>>,
}

impl PackageTerminal {
  pub fn new(rows: u16, columns: u16) -> (Self, TerminalIo) {
    let (output_sender, output) = mpsc::sync_channel(OUTPUT_CHUNKS);
    let (input, input_receiver) = mpsc::sync_channel(INPUT_MESSAGES);
    let control = Arc::new(Mutex::new(Control {
      master: None,
      size: size(rows, columns),
      closed: false,
      killer: None,
    }));
    (
      Self {
        view: TerminalView::with_scrollback(rows, columns, HISTORY_LINES),
        output,
        input: input.clone(),
        control: control.clone(),
        result: None,
        notice: None,
      },
      TerminalIo {
        output: output_sender,
        input: input_receiver,
        stop_input: input,
        control,
      },
    )
  }

  /// Drain at most 64 KiB per tick, including while the terminal is hidden.
  pub fn poll(&mut self) -> bool {
    let mut changed = false;
    for _ in 0..16 {
      let Ok(bytes) = self.output.try_recv() else {
        break;
      };
      self.view.process(&bytes);
      changed = true;
    }
    changed
  }

  pub fn resize(&mut self, rows: u16, columns: u16) -> Result<(), String> {
    let next = size(rows, columns);
    let mut control = self.control.lock().map_err(|error| error.to_string())?;
    if control.size.rows != next.rows || control.size.cols != next.cols {
      if let Some(master) = &control.master {
        master
          .resize(next)
          .map_err(|error| format!("terminal resize: {error}"))?;
      }
      control.size = next;
      self.view.resize(next.rows, next.cols);
    }
    Ok(())
  }

  pub fn write(&mut self, bytes: Vec<u8>) -> Result<(), String> {
    if self.result.is_some() {
      return Ok(());
    }
    if bytes.len() > MAX_INPUT_BYTES {
      return Err("control_center.package_terminal_input_full".into());
    }
    self
      .input
      .try_send(Input::Bytes(bytes))
      .map_err(|_| "control_center.package_terminal_input_full".to_owned())?;
    self.notice = None;
    Ok(())
  }

  pub fn paste(&mut self, text: &str) -> Result<(), String> {
    self.view.set_scrollback(0);
    let bytes = if self.view.bracketed_paste() {
      // A pasted escape must not terminate the bracketed-paste envelope.
      format!("\x1b[200~{}\x1b[201~", text.replace('\x1b', "")).into_bytes()
    } else {
      text.replace("\r\n", "\n").replace('\n', "\r").into_bytes()
    };
    self.write(bytes)
  }
}

impl TerminalIo {
  pub fn run(self, request: &ProcessRequest, banner: &str) -> Result<String, String> {
    self
      .output
      .send(format!("{}\r\n", banner.replace('\n', "\r\n")).into_bytes())
      .map_err(|_| "package terminal was closed".to_owned())?;
    // Preserve the core's testing feature: Control Center tests cannot start
    // pkexec, pacman, helpers or the Control Center executable itself.
    let guarded = command(&request.program);
    let mut builder = CommandBuilder::new(guarded.get_program());
    builder.args(&request.args);
    for (key, value) in &request.env {
      builder.env(key, value);
    }
    self.run_command(builder)
  }

  fn run_command(self, mut builder: CommandBuilder) -> Result<String, String> {
    builder.env("TERM", "xterm-256color");
    builder.env("COLORTERM", "truecolor");
    let initial = self.control.lock().map_err(|error| error.to_string())?.size;
    let pair = native_pty_system()
      .openpty(initial)
      .map_err(|error| format!("terminal PTY: {error}"))?;
    let mut reader = pair
      .master
      .try_clone_reader()
      .map_err(|error| error.to_string())?;
    let mut writer = pair
      .master
      .take_writer()
      .map_err(|error| error.to_string())?;
    let child = pair
      .slave
      .spawn_command(builder)
      .map_err(|error| format!("terminal command: {error}"))?;
    let mut child = RunningChild {
      child,
      completed: false,
    };
    drop(pair.slave);
    {
      let mut control = self.control.lock().map_err(|error| error.to_string())?;
      if control.closed {
        return Err("package terminal was closed".into());
      }
      pair
        .master
        .resize(control.size)
        .map_err(|error| error.to_string())?;
      control.killer = Some(child.child.clone_killer());
      control.master = Some(pair.master);
    }
    let output = self.output.clone();
    let read_thread = thread::Builder::new()
      .name("package-terminal-output".into())
      .spawn(move || {
        let mut bytes = [0; 4096];
        loop {
          match reader.read(&mut bytes) {
            Ok(0) => break,
            Ok(count) => {
              if output.send(bytes[..count].to_vec()).is_err() {
                break;
              }
            }
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            // Unix PTYs report EIO when the last slave closes.
            Err(error) if error.raw_os_error() == Some(libc::EIO) => break,
            Err(error) => {
              let _ = output.send(format!("\r\nterminal output: {error}\r\n").into_bytes());
              break;
            }
          }
        }
      });
    if let Err(error) = read_thread {
      let _ = child.child.kill();
      let _ = child.wait();
      return Err(format!("terminal output reader: {error}"));
    }
    let input = self.input;
    let output = self.output.clone();
    let write_thread = thread::Builder::new()
      .name("package-terminal-input".into())
      .spawn(move || {
        while let Ok(Input::Bytes(bytes)) = input.recv() {
          if let Err(error) = writer.write_all(&bytes).and_then(|()| writer.flush()) {
            let _ = output.send(format!("\r\nterminal input: {error}\r\n").into_bytes());
            break;
          }
        }
      });
    if let Err(error) = write_thread {
      let _ = child.child.kill();
      let _ = child.wait();
      return Err(format!("terminal input writer: {error}"));
    }
    let status = child.wait();
    // A full queue need not block completion: all queued input will be drained
    // or the writer will see the closed PTY. Closing this sender also ends it.
    let _ = self.stop_input.try_send(Input::Stop);
    if let Ok(mut control) = self.control.lock() {
      control.master = None;
      control.killer = None;
    }
    let status = status.map_err(|error| format!("terminal command: {error}"))?;
    if status.success() {
      Ok(String::new())
    } else if status.signal().is_some() {
      Err("control_center.package_transaction_interrupted".into())
    } else {
      Err("control_center.package_transaction_failed".into())
    }
  }
}

/// Reap a child if PTY/thread setup fails after it was spawned.
struct RunningChild {
  child: Box<dyn Child + Send + Sync>,
  completed: bool,
}

impl RunningChild {
  fn wait(&mut self) -> std::io::Result<portable_pty::ExitStatus> {
    let status = self.child.wait()?;
    self.completed = true;
    Ok(status)
  }
}

impl Drop for RunningChild {
  fn drop(&mut self) {
    if !self.completed {
      let _ = self.child.kill();
      let _ = self.child.wait();
    }
  }
}

impl Drop for PackageTerminal {
  fn drop(&mut self) {
    // Normal hiding retains this object. Only teardown abandons an active PTY.
    if let Ok(mut control) = self.control.lock() {
      control.closed = true;
      if self.result.is_none() {
        let _ = self.input.try_send(Input::Bytes(vec![3]));
        if let Some(killer) = &mut control.killer {
          let _ = killer.kill();
        }
      }
    }
  }
}

fn size(rows: u16, columns: u16) -> PtySize {
  PtySize {
    rows: rows.max(1),
    cols: columns.max(1),
    pixel_width: 0,
    pixel_height: 0,
  }
}

/// Encode terminal keys without allowing the surrounding UI to consume them.
pub(super) fn encode_key(key: KeyEvent, application_cursor: bool) -> Option<Vec<u8>> {
  if key.kind == KeyEventKind::Release {
    return None;
  }
  let mut bytes = match key.code {
    KeyCode::Char(character) if key.modifiers.contains(KeyModifiers::CONTROL) => {
      if !character.is_ascii() {
        return None;
      }
      vec![(character as u8) & 0x1f]
    }
    KeyCode::Char(character) => character.to_string().into_bytes(),
    KeyCode::Enter => b"\r".to_vec(),
    KeyCode::Backspace => vec![0x7f],
    KeyCode::Tab => vec![b'\t'],
    KeyCode::BackTab => b"\x1b[Z".to_vec(),
    KeyCode::Up | KeyCode::Down | KeyCode::Right | KeyCode::Left => {
      let suffix = match key.code {
        KeyCode::Up => 'A',
        KeyCode::Down => 'B',
        KeyCode::Right => 'C',
        _ => 'D',
      };
      format!("\x1b{}{suffix}", if application_cursor { 'O' } else { '[' }).into_bytes()
    }
    KeyCode::Home => b"\x1b[H".to_vec(),
    KeyCode::End => b"\x1b[F".to_vec(),
    KeyCode::PageUp => b"\x1b[5~".to_vec(),
    KeyCode::PageDown => b"\x1b[6~".to_vec(),
    KeyCode::Delete => b"\x1b[3~".to_vec(),
    KeyCode::Insert => b"\x1b[2~".to_vec(),
    KeyCode::F(number @ 1..=4) => vec![0x1b, b'O', b'P' + number - 1],
    KeyCode::F(number @ 5..=12) if number != 8 => {
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

#[cfg(test)]
impl TerminalIo {
  pub fn emit(&self, bytes: &[u8]) {
    self.output.send(bytes.to_vec()).unwrap();
  }

  pub fn next_input(&self) -> Option<Vec<u8>> {
    match self.input.try_recv().ok()? {
      Input::Bytes(bytes) => Some(bytes),
      Input::Stop => None,
    }
  }
}

#[cfg(test)]
mod tests;
