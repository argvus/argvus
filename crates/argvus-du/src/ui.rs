use crate::i18n::Messages;
use argvus_tui::{terminal::TerminalGuard, terminal_view::TerminalView};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use portable_pty::{CommandBuilder, MasterPty, PtySize, native_pty_system};
use ratatui::{
  Frame,
  layout::{Constraint, Layout, Rect},
  style::{Modifier, Style},
  text::{Line, Span},
  widgets::{Block, Borders, Paragraph},
};
use std::{
  ffi::CString,
  io::{self, Read, Write},
  os::unix::ffi::OsStrExt,
  process::{Command, Stdio},
  sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc,
  },
  thread,
  time::{Duration, Instant},
};
use unicode_width::UnicodeWidthStr;

const OPTIONS: [(&str, &str); 6] = [
  ("option_one_file_system", "--one-file-system"),
  ("option_exclude_caches", "--exclude-caches"),
  ("option_exclude_kernfs", "--exclude-kernfs"),
  ("option_apparent_size", "--apparent-size"),
  ("option_si", "--si"),
  ("option_itemcount", "--show-itemcount"),
];
const ANALYZE_FOCUS: usize = OPTIONS.len() + 2;
const FOCUS_COUNT: usize = ANALYZE_FOCUS + 1;
#[derive(Clone, Copy, PartialEq, Eq)]
enum PickerMode {
  Superfile,
  Other,
}
struct Picker {
  child: std::process::Child,
  selection: Option<tempfile::NamedTempFile>,
  mode: PickerMode,
  stop_focus: Arc<AtomicBool>,
  stop_focus_on_drop: bool,
  detached_gui: bool,
}
impl Picker {
  fn spawn(path: &str, _title: &str) -> io::Result<Self> {
    let system = std::env::var_os("ARGVUS_SYSTEM_CONFIG")
      .map(std::path::PathBuf::from)
      .unwrap_or_else(|| "/usr/share/argvus".into());
    let output = Command::new(system.join("session/sh/get-default.sh"))
      .arg("file_manager")
      .output()?;
    if !output.status.success() {
      return Err(io::Error::other("cannot determine default file manager"));
    }
    let manager = String::from_utf8(output.stdout).map_err(io::Error::other)?;
    let words = manager.split_whitespace().collect::<Vec<_>>();
    let first = words
      .first()
      .ok_or_else(|| io::Error::other("empty file manager command"))?;
    let base = std::path::Path::new(first)
      .file_name()
      .and_then(|name| name.to_str())
      .unwrap_or(first);
    let app = if base == "argvus" {
      words.get(1).copied().unwrap_or("")
    } else {
      base
    };
    let mode = match app {
      "spf" | "--spf" | "superfile" | "--superfile" => PickerMode::Superfile,
      _ => PickerMode::Other,
    };
    let start = if std::path::Path::new(path).is_dir() {
      std::path::PathBuf::from(path)
    } else {
      std::env::var_os("HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| "/".into())
    };
    if mode == PickerMode::Other {
      let tui = matches!(
        base,
        "argvus" | "yazi" | "ranger" | "lf" | "joshuto" | "broot" | "mc" | "nnn"
      );
      let mut command = if tui {
        let mut command = Command::new("argvus");
        command.args([
          "tui-terminal",
          "--class",
          "argvus-du-picker",
          "--term",
          "kitty",
          "--",
        ]);
        command.args(&words);
        command
      } else {
        let mut command = Command::new(first);
        command.args(words.iter().skip(1));
        command
      };
      let child = command
        .arg(start)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()?;
      let stop_focus = Arc::new(AtomicBool::new(false));
      let class = if tui {
        Some("argvus-du-picker")
      } else {
        gui_manager_class(base)
      };
      crate::window::place(child.id(), class, None, Arc::clone(&stop_focus));
      return Ok(Self {
        child,
        selection: None,
        mode,
        stop_focus,
        stop_focus_on_drop: true,
        detached_gui: !tui,
      });
    }
    let cache = std::env::var_os("XDG_CACHE_HOME")
      .map(std::path::PathBuf::from)
      .or_else(|| {
        std::env::var_os("HOME").map(|home| std::path::PathBuf::from(home).join(".cache"))
      })
      .unwrap_or_else(|| "/tmp/argvus".into())
      .join("argvus");
    std::fs::create_dir_all(&cache)?;
    let selection = tempfile::Builder::new()
      .prefix("du-picker-")
      .tempfile_in(cache)?;
    let executable = std::env::current_exe()?;
    let child = Command::new("argvus")
      .args([
        "tui-terminal",
        "--class",
        "argvus-du-picker",
        "--term",
        "kitty",
        "--",
      ])
      .arg(executable)
      .arg("--directory-picker")
      .arg(selection.path())
      .arg(start)
      .args(words.iter().skip(if base == "argvus" { 2 } else { 1 }))
      .stdin(Stdio::null())
      .stdout(Stdio::null())
      .stderr(Stdio::piped())
      .spawn()?;
    let stop_focus = Arc::new(AtomicBool::new(false));
    crate::window::place(child.id(), None, None, Arc::clone(&stop_focus));
    Ok(Self {
      child,
      selection: Some(selection),
      mode,
      stop_focus,
      stop_focus_on_drop: true,
      detached_gui: false,
    })
  }
  fn finish(mut self) -> Result<(Option<String>, PickerMode), String> {
    let status = self.child.wait().map_err(|e| e.to_string())?;
    if !status.success() {
      let mut error = String::new();
      if let Some(stderr) = self.child.stderr.as_mut() {
        let _ = stderr.read_to_string(&mut error);
      }
      return Err(if error.trim().is_empty() {
        format!("file manager exited: {status}")
      } else {
        error.trim().to_owned()
      });
    }
    let Some(selection) = &self.selection else {
      if self.detached_gui {
        self.stop_focus_on_drop = false;
      }
      return Ok((None, PickerMode::Other));
    };
    let bytes = std::fs::read(selection.path()).map_err(|e| e.to_string())?;
    if bytes.is_empty() {
      return Ok((None, self.mode));
    }
    let path = String::from_utf8(bytes).map_err(|e| e.to_string())?;
    if !std::path::Path::new(&path).is_dir() {
      return Err("selected path is not a directory".into());
    }
    Ok((Some(path), self.mode))
  }
}
impl Drop for Picker {
  fn drop(&mut self) {
    if self.stop_focus_on_drop {
      self.stop_focus.store(true, Ordering::Relaxed);
    }
    let _ = self.child.kill();
    let _ = self.child.wait();
  }
}
fn gui_manager_class(name: &str) -> Option<&'static str> {
  match name {
    "nautilus" => Some("org.gnome.Nautilus"),
    "nemo" => Some("org.nemo.Nemo"),
    "thunar" => Some("Thunar"),
    "dolphin" => Some("org.kde.dolphin"),
    "pcmanfm" | "pcmanfm-qt" => Some("pcmanfm"),
    "caja" => Some("org.mate.Caja"),
    _ => None,
  }
}
struct Child {
  process: Box<dyn portable_pty::Child + Send + Sync>,
  master: Box<dyn MasterPty + Send>,
  writer: Box<dyn Write + Send>,
  output: mpsc::Receiver<Vec<u8>>,
  screen: TerminalView,
  size: (u16, u16),
}
impl Drop for Child {
  fn drop(&mut self) {
    let _ = self.process.kill();
    let _ = self.process.wait();
  }
}
fn size(area: Rect) -> PtySize {
  PtySize {
    rows: area.height.max(1),
    cols: area.width.max(1),
    pixel_width: 0,
    pixel_height: 0,
  }
}
fn arguments(options: &[bool; OPTIONS.len()], path: &str) -> Vec<String> {
  let mut args = vec![
    "--ignore-config",
    "--disable-shell",
    "--enable-delete",
    "--confirm-delete",
    "--color=dark",
    "--show-graph",
    "--show-percent",
    "--group-directories-first",
  ]
  .into_iter()
  .map(str::to_owned)
  .collect::<Vec<_>>();
  for (enabled, (_, flag)) in options.iter().zip(OPTIONS) {
    if *enabled {
      args.push(flag.into());
    }
  }
  args.push("--".into());
  args.push(path.into());
  args
}
fn validate(path: &str, messages: &Messages) -> Result<(), String> {
  if path.is_empty() {
    return Err(messages.get("empty").into());
  }
  let p = std::path::Path::new(path);
  let meta = std::fs::metadata(p).map_err(|e| format!("{}: {e}", messages.get("inaccessible")))?;
  if !meta.is_dir() {
    return Err(messages.get("not_directory").into());
  }
  let path =
    CString::new(p.as_os_str().as_bytes()).map_err(|_| messages.get("inaccessible").to_string())?;
  // SAFETY: path is a valid NUL-terminated string for the duration of this call.
  if unsafe { libc::access(path.as_ptr(), libc::R_OK | libc::X_OK) } != 0 {
    return Err(format!(
      "{}: {}",
      messages.get("unreadable"),
      io::Error::last_os_error()
    ));
  }
  std::fs::read_dir(p).map_err(|e| format!("{}: {e}", messages.get("unreadable")))?;
  Ok(())
}
impl Child {
  fn spawn(
    path: &str,
    options: &[bool; OPTIONS.len()],
    area: Rect,
    messages: &Messages,
  ) -> Result<Self, String> {
    Self::spawn_command("ncdu", &arguments(options, path), area)
      .map_err(|e| format!("{}: {e}", messages.get("missing_ncdu")))
  }
  fn spawn_command(program: &str, args: &[String], area: Rect) -> Result<Self, String> {
    Self::spawn_command_with_config(program, args, area, None)
  }
  fn spawn_command_with_config(
    program: &str,
    args: &[String],
    area: Rect,
    config_home: Option<&std::path::Path>,
  ) -> Result<Self, String> {
    let pair = native_pty_system()
      .openpty(size(area))
      .map_err(|e| e.to_string())?;
    let mut cmd = CommandBuilder::new(program);
    cmd.args(args);
    if let Some(config_home) = config_home {
      cmd.env("XDG_CONFIG_HOME", config_home);
    }
    cmd.env("TERM", "xterm-256color");
    let mut reader = pair.master.try_clone_reader().map_err(|e| e.to_string())?;
    let writer = pair.master.take_writer().map_err(|e| e.to_string())?;
    let process = pair.slave.spawn_command(cmd).map_err(|e| e.to_string())?;
    drop(pair.slave);
    let (tx, output) = mpsc::channel();
    thread::spawn(move || {
      let mut buf = [0; 8192];
      while let Ok(n) = reader.read(&mut buf) {
        if n == 0 || tx.send(buf[..n].to_vec()).is_err() {
          break;
        }
      }
    });
    Ok(Self {
      process,
      master: pair.master,
      writer,
      output,
      screen: TerminalView::new(area.height, area.width),
      size: (area.height.max(1), area.width.max(1)),
    })
  }
  fn resize(&mut self, area: Rect) -> Result<(), String> {
    let dimensions = (area.height.max(1), area.width.max(1));
    if dimensions != self.size {
      self.master.resize(size(area)).map_err(|e| e.to_string())?;
      self.screen.resize(dimensions.0, dimensions.1);
      self.size = dimensions;
    }
    Ok(())
  }
}
struct App {
  path: String,
  cursor: usize,
  options: [bool; OPTIONS.len()],
  focus: usize,
  child: Option<Child>,
  picker: Option<Picker>,
  status: String,
  messages: Messages,
  theme: crate::theme::Theme,
}
fn layout(area: Rect) -> std::rc::Rc<[Rect]> {
  Layout::vertical([
    Constraint::Length(3),
    Constraint::Length(3),
    Constraint::Length(8),
    Constraint::Min(3),
    Constraint::Length(2),
    Constraint::Length(1),
  ])
  .split(area)
}
fn results(area: Rect) -> Rect {
  layout(area)[3].inner(ratatui::layout::Margin {
    horizontal: 1,
    vertical: 1,
  })
}
fn draw(f: &mut Frame, a: &App) {
  let area = f.area();
  if area.width < 40 || area.height < 22 {
    f.render_widget(Paragraph::new(a.messages.get("small")), area);
    return;
  }
  let rows = layout(area);
  let theme = &a.theme;
  f.render_widget(
    Block::default().style(Style::default().fg(theme.foreground).bg(theme.background)),
    area,
  );
  let accent = Style::default().fg(theme.accent);
  let focus = |n| {
    if a.focus == n && a.child.is_none() {
      accent.add_modifier(Modifier::BOLD)
    } else {
      Style::default().fg(theme.foreground)
    }
  };
  f.render_widget(
    Paragraph::new(Line::from(vec![
      Span::styled(
        " ARGVUS ",
        Style::default()
          .fg(theme.background)
          .bg(theme.accent)
          .add_modifier(Modifier::BOLD),
      ),
      Span::styled(
        format!("  {}", a.messages.get("title")),
        Style::default().add_modifier(Modifier::BOLD),
      ),
    ]))
    .block(
      Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(theme.accent)),
    ),
    rows[0],
  );
  let prefix = &a.path[..a.cursor];
  let directory_areas =
    Layout::horizontal([Constraint::Min(1), Constraint::Length(16)]).split(rows[1]);
  let width = usize::from(directory_areas[0].width.saturating_sub(2));
  let mut start = 0;
  for (offset, _) in prefix.char_indices() {
    if UnicodeWidthStr::width(&prefix[start..]) < width {
      break;
    }
    start = offset
      + a.path[offset..]
        .chars()
        .next()
        .map(char::len_utf8)
        .unwrap_or(0);
  }
  f.render_widget(
    Paragraph::new(&a.path[start..]).block(
      Block::default()
        .title(format!(" {} ", a.messages.get("directory")))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.accent)),
    ),
    directory_areas[0],
  );
  let browse_area = Layout::vertical([Constraint::Length(1)])
    .flex(ratatui::layout::Flex::Center)
    .split(directory_areas[1])[0];
  f.render_widget(
    Paragraph::new(format!("[ {} ]", a.messages.get("browse"))).style(focus(1)),
    browse_area,
  );
  if a.focus == 0 && a.child.is_none() && width > 0 {
    let column = UnicodeWidthStr::width(&a.path[start..a.cursor]).min(width - 1);
    f.set_cursor_position((rows[1].x + 1 + column as u16, rows[1].y + 1));
  }
  let opts = (0..OPTIONS.len())
    .map(|i| {
      Line::from(Span::styled(
        format!(
          "{} {}",
          if a.options[i] { "[x]" } else { "[ ]" },
          a.messages.get(OPTIONS[i].0)
        ),
        focus(i + 2),
      ))
    })
    .collect::<Vec<_>>();
  f.render_widget(
    Paragraph::new(opts).block(
      Block::default()
        .title(format!(" {} ", a.messages.get("options")))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.accent)),
    ),
    rows[2],
  );
  f.render_widget(
    Block::default()
      .title(format!(" {} ", a.messages.get("results")))
      .borders(Borders::ALL)
      .border_style(Style::default().fg(theme.accent)),
    rows[3],
  );
  let inner = results(area);
  if let Some(child) = &a.child {
    render_child(f, child, inner, theme);
  } else {
    f.render_widget(
      Paragraph::new(if a.status.is_empty() {
        a.messages.get("initial")
      } else {
        &a.status
      }),
      inner,
    );
  }
  f.render_widget(
    Paragraph::new(format!("[ {} ]", a.messages.get("analyze"))).style(focus(ANALYZE_FOCUS)),
    rows[4],
  );
  f.render_widget(
    Paragraph::new(if a.child.is_some() {
      a.messages.get("footer_results")
    } else {
      a.messages.get("footer_main")
    })
    .style(Style::default().fg(theme.muted)),
    rows[5],
  );
}
fn render_child(f: &mut Frame, child: &Child, inner: Rect, theme: &crate::theme::Theme) {
  child.screen.render(f, inner, theme);
  child.screen.render_cursor(f, inner);
}
fn key_bytes(key: KeyEvent, application_cursor: bool) -> Option<Vec<u8>> {
  let code = key.code;
  let mut bytes = match code {
    KeyCode::Char(c) if key.modifiers.contains(KeyModifiers::CONTROL) && c.is_ascii() => {
      vec![(c as u8) & 31]
    }
    KeyCode::Char(c) => c.to_string().into_bytes(),
    KeyCode::Enter => vec![13],
    KeyCode::Backspace => vec![127],
    KeyCode::Tab => vec![9],
    KeyCode::Up => if application_cursor {
      b"\x1bOA"
    } else {
      b"\x1b[A"
    }
    .to_vec(),
    KeyCode::Down => if application_cursor {
      b"\x1bOB"
    } else {
      b"\x1b[B"
    }
    .to_vec(),
    KeyCode::Right => if application_cursor {
      b"\x1bOC"
    } else {
      b"\x1b[C"
    }
    .to_vec(),
    KeyCode::Left => if application_cursor {
      b"\x1bOD"
    } else {
      b"\x1b[D"
    }
    .to_vec(),
    KeyCode::PageUp => b"\x1b[5~".to_vec(),
    KeyCode::PageDown => b"\x1b[6~".to_vec(),
    KeyCode::Home => b"\x1b[H".to_vec(),
    KeyCode::End => b"\x1b[F".to_vec(),
    KeyCode::Delete => b"\x1b[3~".to_vec(),
    KeyCode::F(n @ 1..=4) => vec![27, b'O', b'P' + n - 1],
    KeyCode::F(n @ 5..=12) => format!(
      "\x1b[{}~",
      [15, 17, 18, 19, 20, 21, 23, 24][usize::from(n - 5)]
    )
    .into_bytes(),
    _ => return None,
  };
  if key.modifiers.contains(KeyModifiers::ALT) {
    bytes.insert(0, 27);
  }
  Some(bytes)
}
pub fn run() -> Result<(), String> {
  let stopped = Arc::new(AtomicBool::new(false));
  for signal in [
    signal_hook::consts::SIGTERM,
    signal_hook::consts::SIGHUP,
    signal_hook::consts::SIGINT,
  ] {
    signal_hook::flag::register(signal, Arc::clone(&stopped)).map_err(|e| e.to_string())?;
  }
  let messages = Messages::load();
  argvus_tui::terminal::install_panic_hook();
  let mut terminal_guard =
    TerminalGuard::new().map_err(|error| format!("cannot open TUI: {error}"))?;
  let terminal = terminal_guard.terminal_mut();
  let path = std::env::var("HOME").unwrap_or_default();
  let mut a = App {
    cursor: path.len(),
    path,
    options: [false; OPTIONS.len()],
    focus: 0,
    child: None,
    picker: None,
    status: String::new(),
    messages,
    theme: crate::theme::Theme::load(),
  };
  let mut theme_checked = Instant::now();
  loop {
    if theme_checked.elapsed() >= Duration::from_secs(1) {
      a.theme = crate::theme::Theme::load();
      theme_checked = Instant::now();
    }
    if stopped.load(Ordering::Relaxed) {
      break;
    }
    let s = terminal.size().map_err(|e| e.to_string())?;
    let area = results(Rect::new(0, 0, s.width, s.height));
    if let Some(child) = a.child.as_mut() {
      child.resize(area)?;
      while let Ok(bytes) = child.output.try_recv() {
        child.screen.process(&bytes);
      }
      if let Some(status) = child.process.try_wait().map_err(|e| e.to_string())? {
        a.status = format!("{}: {status}", a.messages.get("child_exited"));
        a.child = None;
      }
    }
    if let Some(picker) = a.picker.as_mut()
      && picker
        .child
        .try_wait()
        .map_err(|e| e.to_string())?
        .is_some()
      && let Some(picker) = a.picker.take()
    {
      match picker.finish() {
        Ok((Some(path), _)) => {
          a.path = path;
          a.cursor = a.path.len();
          a.status.clear();
        }
        Ok((None, PickerMode::Other)) => a.status = a.messages.get("manager_no_selection").into(),
        Ok((None, _)) => a.status.clear(),
        Err(error) => {
          a.status = format!("{}: {error}", a.messages.get("browse_error"));
        }
      }
    }
    terminal.draw(|f| draw(f, &a)).map_err(|e| e.to_string())?;
    if !event::poll(Duration::from_millis(30)).map_err(|e| e.to_string())? {
      continue;
    }
    let event = event::read().map_err(|e| e.to_string())?;
    if let Event::Paste(text) = &event {
      if let Some(child) = a.child.as_mut() {
        let data = if child.screen.bracketed_paste() {
          format!("\x1b[200~{text}\x1b[201~")
        } else {
          text.clone()
        };
        child
          .writer
          .write_all(data.as_bytes())
          .map_err(|e| e.to_string())?;
      } else if a.focus == 0 && a.picker.is_none() {
        a.path.insert_str(a.cursor, text);
        a.cursor += text.len();
      }
      continue;
    }
    if let Event::Key(k) = event {
      if k.kind == KeyEventKind::Release {
        continue;
      }
      if a.picker.is_some() {
        if k.code == KeyCode::Esc {
          a.picker = None;
          a.status.clear();
        }
        continue;
      }
      if let Some(child) = a.child.as_mut() {
        if k.code == KeyCode::Esc {
          a.child = None;
          a.status = a.messages.get("stopped").into();
        } else if let Some(bytes) = key_bytes(k, child.screen.application_cursor()) {
          child.writer.write_all(&bytes).map_err(|e| e.to_string())?;
        }
        continue;
      }
      match k.code {
        KeyCode::Esc => break,
        KeyCode::Tab | KeyCode::Down => a.focus = (a.focus + 1) % FOCUS_COUNT,
        KeyCode::BackTab | KeyCode::Up => a.focus = (a.focus + FOCUS_COUNT - 1) % FOCUS_COUNT,
        KeyCode::Enter if a.focus <= 1 => {
          match Picker::spawn(&a.path, a.messages.get("browse_title")) {
            Ok(picker) => {
              a.picker = Some(picker);
              a.status = a.messages.get("browsing").into();
            }
            Err(error) => a.status = format!("{}: {error}", a.messages.get("browse_error")),
          }
        }
        KeyCode::Enter | KeyCode::Char(' ') if a.focus >= 2 => {
          if a.focus < ANALYZE_FOCUS {
            a.options[a.focus - 2] = !a.options[a.focus - 2];
          } else {
            match validate(&a.path, &a.messages)
              .and_then(|()| Child::spawn(&a.path, &a.options, area, &a.messages))
            {
              Ok(child) => {
                a.child = Some(child);
                a.status.clear();
              }
              Err(e) => a.status = e,
            }
          }
        }
        KeyCode::Char(c) if a.focus == 0 => {
          a.path.insert(a.cursor, c);
          a.cursor += c.len_utf8();
        }
        KeyCode::Backspace if a.focus == 0 && a.cursor > 0 => {
          let p = a.path[..a.cursor]
            .char_indices()
            .last()
            .map(|(i, _)| i)
            .unwrap_or(0);
          a.path.drain(p..a.cursor);
          a.cursor = p;
        }
        KeyCode::Delete if a.focus == 0 && a.cursor < a.path.len() => {
          a.path.remove(a.cursor);
        }
        KeyCode::Left if a.focus == 0 => {
          a.cursor = a.path[..a.cursor]
            .char_indices()
            .last()
            .map(|(i, _)| i)
            .unwrap_or(0)
        }
        KeyCode::Right if a.focus == 0 => {
          if let Some(c) = a.path[a.cursor..].chars().next() {
            a.cursor += c.len_utf8();
          }
        }
        KeyCode::Left => a.focus = (a.focus + FOCUS_COUNT - 1) % FOCUS_COUNT,
        KeyCode::Right => a.focus = (a.focus + 1) % FOCUS_COUNT,
        KeyCode::Home if a.focus == 0 => a.cursor = 0,
        KeyCode::End if a.focus == 0 => a.cursor = a.path.len(),
        _ => {}
      }
    }
  }
  Ok(())
}
/// Run Superfile in a dedicated ARGVUS terminal and return its explicitly
/// selected current directory through the private chooser file.
pub fn run_directory_picker(
  selection: &std::ffi::OsStr,
  start: &std::ffi::OsStr,
  manager_args: &[std::ffi::OsString],
) -> Result<(), String> {
  let stopped = Arc::new(AtomicBool::new(false));
  for signal in [
    signal_hook::consts::SIGTERM,
    signal_hook::consts::SIGHUP,
    signal_hook::consts::SIGINT,
  ] {
    signal_hook::flag::register(signal, Arc::clone(&stopped)).map_err(|e| e.to_string())?;
  }
  let messages = Messages::load();
  argvus_tui::terminal::install_panic_hook();
  let mut terminal_guard =
    TerminalGuard::new().map_err(|error| format!("cannot open TUI: {error}"))?;
  let terminal = terminal_guard.terminal_mut();
  let s = terminal.size().map_err(|e| e.to_string())?;
  let area = Rect::new(0, 0, s.width, s.height.saturating_sub(1));
  let start = start.to_str().ok_or("directory path is not UTF-8")?;
  let keys = crate::superfile::prepare(manager_args)?;
  let mut args = vec!["--spf".to_owned()];
  args.extend(keys.args.iter().cloned());
  args.extend(["--print-last-dir".to_owned(), start.to_owned()]);
  let mut child = Child::spawn_command_with_config("argvus", &args, area, Some(&keys.config_home))?;
  let mut tail = Vec::new();
  let mut confirmed = false;
  loop {
    if stopped.load(Ordering::Relaxed) {
      return Ok(());
    }
    while let Ok(bytes) = child.output.try_recv() {
      child.screen.process(&bytes);
      tail.extend_from_slice(&bytes);
      if tail.len() > 131072 {
        tail.drain(..65536);
      }
    }
    if let Some(status) = child.process.try_wait().map_err(|e| e.to_string())? {
      while let Ok(bytes) = child.output.recv_timeout(Duration::from_millis(100)) {
        tail.extend_from_slice(&bytes);
      }
      if !status.success() {
        return Err(format!("Superfile exited: {status}"));
      }
      if confirmed && let Some(path) = selected_directory(&tail) {
        std::fs::write(selection, path.as_bytes()).map_err(|e| e.to_string())?;
      }
      return Ok(());
    }
    let size = terminal.size().map_err(|e| e.to_string())?;
    let area = Rect::new(0, 0, size.width, size.height.saturating_sub(1));
    child.resize(area)?;
    let theme = crate::theme::Theme::load();
    terminal
      .draw(|frame| {
        render_child(frame, &child, area, &theme);
        let footer = Rect::new(0, area.height, size.width, 1);
        frame.render_widget(
          Paragraph::new(messages.get("picker_help"))
            .style(Style::default().fg(theme.accent).bg(theme.background)),
          footer,
        );
      })
      .map_err(|e| e.to_string())?;
    if event::poll(Duration::from_millis(30)).map_err(|e| e.to_string())? {
      match event::read().map_err(|e| e.to_string())? {
        Event::Key(key) if key.kind != KeyEventKind::Release => {
          if key.code == KeyCode::Esc {
            return Ok(());
          }
          confirmed =
            key.code == KeyCode::Char(' ') && !key.modifiers.contains(KeyModifiers::CONTROL);
          if let Some(bytes) = key_bytes(key, child.screen.application_cursor()) {
            child.writer.write_all(&bytes).map_err(|e| e.to_string())?;
          }
        }
        Event::Paste(text) => child
          .writer
          .write_all(text.as_bytes())
          .map_err(|e| e.to_string())?,
        _ => {}
      }
    }
  }
}

fn selected_directory(output: &[u8]) -> Option<String> {
  // Superfile prints its selected path after leaving the alternate screen;
  // a regular quit emits no path. Strip the terminal reset sequences only.
  let marker = b"\x1b[?1049l";
  let index = output
    .windows(marker.len())
    .rposition(|window| window == marker)?
    + marker.len();
  let mut plain = Vec::new();
  let mut bytes = output[index..].iter().copied().peekable();
  while let Some(byte) = bytes.next() {
    if byte != 0x1b {
      plain.push(byte);
      continue;
    }
    match bytes.next() {
      Some(b'[') => {
        for byte in bytes.by_ref() {
          if (0x40..=0x7e).contains(&byte) {
            break;
          }
        }
      }
      Some(b']') => {
        while let Some(byte) = bytes.next() {
          if byte == 7 {
            break;
          }
          if byte == 0x1b && bytes.peek() == Some(&b'\\') {
            bytes.next();
            break;
          }
        }
      }
      _ => {}
    }
  }
  while matches!(plain.last(), Some(b'\r' | b'\n')) {
    plain.pop();
  }
  let path = String::from_utf8(plain).ok()?;
  (!path.is_empty() && std::path::Path::new(&path).is_dir()).then_some(path)
}

#[cfg(test)]
mod tests {
  use super::*;
  use ratatui::Terminal;
  #[test]
  fn arguments_keep_defaults_enable_confirmation_and_respect_options() {
    for bits in 0..(1 << OPTIONS.len()) {
      let mut options = [false; OPTIONS.len()];
      for (index, value) in options.iter_mut().enumerate() {
        *value = bits & (1 << index) != 0;
      }
      let args = arguments(&options, "-path with spaces;雪");
      for flag in [
        "--disable-shell",
        "--enable-delete",
        "--confirm-delete",
        "--color=dark",
        "--show-graph",
        "--show-percent",
        "--group-directories-first",
      ] {
        assert!(args.iter().any(|arg| arg == flag));
      }
      for (index, (_, flag)) in OPTIONS.iter().enumerate() {
        assert_eq!(args.iter().any(|arg| arg == flag), options[index]);
      }
      assert_eq!(&args[args.len() - 2..], &["--", "-path with spaces;雪"]);
    }
  }
  #[test]
  fn draws_lines_with_full_accent_color() {
    let backend = ratatui::backend::TestBackend::new(100, 30);
    let mut terminal = Terminal::new(backend).unwrap();
    let accent = ratatui::style::Color::Rgb(100, 70, 130);
    let mut theme = crate::theme::Theme::load();
    theme.accent = accent;
    let app = App {
      path: "/tmp".into(),
      cursor: 4,
      options: [false; OPTIONS.len()],
      focus: 0,
      child: None,
      picker: None,
      status: String::new(),
      messages: Messages::load(),
      theme,
    };
    terminal.draw(|frame| draw(frame, &app)).unwrap();
    let buffer = terminal.backend().buffer();
    assert_eq!(buffer[(0, 2)].fg, accent);
    assert_eq!(buffer[(0, 6)].fg, accent);
    assert_eq!(buffer[(0, 14)].fg, accent);
  }
  #[test]
  fn browse_button_shows_only_its_bracketed_label_without_a_frame() {
    let backend = ratatui::backend::TestBackend::new(100, 30);
    let mut terminal = Terminal::new(backend).expect("test backend creates a terminal");
    let app = App {
      path: "/tmp".into(),
      cursor: 4,
      options: [false; OPTIONS.len()],
      focus: 1,
      child: None,
      picker: None,
      status: String::new(),
      messages: Messages::load(),
      theme: crate::theme::Theme::load(),
    };
    terminal
      .draw(|frame| draw(frame, &app))
      .expect("the form renders");
    let button_area = Layout::horizontal([Constraint::Min(1), Constraint::Length(16)])
      .split(layout(Rect::new(0, 0, 100, 30))[1])[1];
    let buffer = terminal.backend().buffer();
    assert_eq!(buffer[(button_area.x, button_area.y)].symbol(), " ");
    assert_eq!(buffer[(button_area.x, button_area.y + 1)].symbol(), "[");
    assert_eq!(
      buffer[(button_area.x + button_area.width - 1, button_area.y)].symbol(),
      " "
    );
  }
  #[test]
  fn cursor_encoding_tracks_terminal_mode() {
    let key = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
    assert_eq!(key_bytes(key, true).as_deref(), Some(&b"\x1bOB"[..]));
    assert_eq!(key_bytes(key, false).as_deref(), Some(&b"\x1b[B"[..]));
  }
  #[test]
  fn rejects_unreadable_directory() {
    use std::os::unix::fs::PermissionsExt;
    // SAFETY: geteuid has no arguments or preconditions.
    if unsafe { libc::geteuid() } == 0 {
      return;
    }
    let root = std::path::Path::new("/tmp/argvus");
    std::fs::create_dir_all(root).unwrap();
    let path = root.join(format!("du-unreadable-{}", std::process::id()));
    std::fs::create_dir(&path).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o000)).unwrap();
    let result = validate(path.to_str().unwrap(), &Messages::load());
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
    std::fs::remove_dir(&path).unwrap();
    assert!(result.is_err());
  }
  #[test]
  fn rejects_invalid_paths() {
    let messages = Messages::load();
    assert!(validate("", &messages).is_err());
    assert!(validate("/dev/null", &messages).is_err());
    assert!(validate("/nonexistent-argvus-du-test", &messages).is_err());
  }
}
