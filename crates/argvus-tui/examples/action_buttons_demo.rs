use argvus_theme::Theme;
use argvus_tui::action_buttons::{ActionButton, draw, height};
use crossterm::{
  event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},
  execute,
  terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
  Frame, Terminal,
  backend::CrosstermBackend,
  layout::{Constraint, Direction, Layout, Rect},
  style::{Modifier, Style},
  text::{Line, Span},
  widgets::{Block, Borders, Paragraph},
};
use std::io;

struct Demo {
  selected: usize,
  buttons: Vec<ActionButton>,
  message: Option<String>,
}

impl Demo {
  fn new() -> Self {
    let buttons = vec![
      ActionButton::primary("💾", "Save changes", "Ctrl+S"),
      ActionButton::secondary("🔐", "Change password", "Ctrl+P"),
      ActionButton::secondary("🔒", "Lock/Unlock", "Ctrl+L"),
      ActionButton::secondary("👤", "Avatar", "Ctrl+A"),
      ActionButton::danger("❌", "Delete user", "Del"),
      ActionButton::secondary("❓", "Help", "F1"),
    ];

    Self {
      selected: 0,
      buttons,
      message: Some("Use Tab/Shift+Tab to navigate, Enter to select, Esc to exit".to_string()),
    }
  }

  fn handle_input(&mut self, key: KeyEvent) -> bool {
    match key.code {
      KeyCode::Tab => {
        self.selected = (self.selected + 1) % self.buttons.len();
        true
      }
      KeyCode::BackTab => {
        self.selected = if self.selected > 0 {
          self.selected - 1
        } else {
          self.buttons.len() - 1
        };
        true
      }
      KeyCode::Enter => {
        let button = &self.buttons[self.selected];
        self.message = Some(format!("Pressed: {} ({})", button.label, button.shortcut));
        true
      }
      KeyCode::Char('s')
        if key
          .modifiers
          .contains(crossterm::event::KeyModifiers::CONTROL) =>
      {
        self.message = Some("✓ Changes saved successfully".to_string());
        true
      }
      KeyCode::Char('p')
        if key
          .modifiers
          .contains(crossterm::event::KeyModifiers::CONTROL) =>
      {
        self.message = Some("Password change dialog would open".to_string());
        true
      }
      KeyCode::Char('l')
        if key
          .modifiers
          .contains(crossterm::event::KeyModifiers::CONTROL) =>
      {
        self.message = Some("User locked/unlocked".to_string());
        true
      }
      KeyCode::Char('a')
        if key
          .modifiers
          .contains(crossterm::event::KeyModifiers::CONTROL) =>
      {
        self.message = Some("Avatar selector would open".to_string());
        true
      }
      KeyCode::Delete => {
        self.message = Some("⚠️  Confirm deletion? Press Y to confirm, N to cancel".to_string());
        true
      }
      KeyCode::F(1) => {
        self.message = Some(
          "Help: Navigate with Tab/Shift+Tab, Select with Enter, Use keyboard shortcuts"
            .to_string(),
        );
        true
      }
      KeyCode::Esc => false,
      _ => true,
    }
  }

  fn draw(&self, f: &mut Frame) {
    let theme = Theme::load();
    let chunks = Layout::default()
      .direction(Direction::Vertical)
      .constraints([
        Constraint::Length(3),
        Constraint::Min(10),
        Constraint::Length(6),
        Constraint::Length(1),
      ])
      .split(f.area());

    // Title
    let title = Paragraph::new("ARGVUS Action Buttons Demo")
      .style(
        Style::default()
          .fg(theme.accent)
          .add_modifier(Modifier::BOLD),
      )
      .block(
        Block::default()
          .borders(Borders::ALL)
          .border_type(ratatui::widgets::BorderType::Rounded),
      );
    f.render_widget(title, chunks[0]);

    // User details section
    let user_info = vec![
      Line::from(Span::styled(
        "User Details",
        Style::default()
          .fg(theme.accent)
          .add_modifier(Modifier::BOLD),
      )),
      Line::from(""),
      Line::from(vec![
        Span::raw("Username: "),
        Span::styled("john_doe", Style::default().fg(theme.selected_foreground)),
      ]),
      Line::from(vec![
        Span::raw("User ID: "),
        Span::styled("1000", Style::default().fg(theme.muted)),
      ]),
      Line::from(vec![
        Span::raw("Shell: "),
        Span::styled("/bin/zsh", Style::default().fg(theme.muted)),
      ]),
      Line::from(vec![
        Span::raw("Home: "),
        Span::styled("/home/john_doe", Style::default().fg(theme.muted)),
      ]),
    ];
    let user_widget = Paragraph::new(user_info).block(
      Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .title("Account Info"),
    );
    f.render_widget(user_widget, chunks[1]);

    // Action Buttons
    let btn_height = height(&self.buttons, chunks[2].width);
    let button_area = Rect {
      y: chunks[2].y,
      height: btn_height.min(chunks[2].height),
      ..chunks[2]
    };

    let buttons_block = Block::default()
      .borders(Borders::ALL)
      .border_type(ratatui::widgets::BorderType::Rounded)
      .title("Actions (Tab/Shift+Tab to navigate)");
    f.render_widget(buttons_block, chunks[2]);

    // Leave space for block borders
    let inner_area = Rect {
      x: button_area.x + 1,
      y: button_area.y + 1,
      width: button_area.width.saturating_sub(2),
      height: button_area.height.saturating_sub(2),
    };

    draw(f, inner_area, &self.buttons, self.selected, &theme);

    // Status/Message
    let msg_text = self
      .message
      .as_ref()
      .map(|m| Paragraph::new(m.as_str()).style(Style::default().fg(theme.selected_foreground)))
      .unwrap_or_else(|| Paragraph::new("Ready").style(Style::default().fg(theme.muted)));
    f.render_widget(msg_text, chunks[3]);
  }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
  // Setup terminal
  enable_raw_mode()?;
  let mut stdout = io::stdout();
  execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
  let backend = CrosstermBackend::new(stdout);
  let mut terminal = Terminal::new(backend)?;

  // Create app and run
  let app = Demo::new();
  let res = run_app(&mut terminal, app);

  // Restore terminal
  disable_raw_mode()?;
  execute!(
    terminal.backend_mut(),
    LeaveAlternateScreen,
    DisableMouseCapture
  )?;
  terminal.show_cursor()?;

  if let Err(err) = res {
    println!("Error: {:?}", err);
  }

  Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, mut app: Demo) -> io::Result<()> {
  loop {
    terminal.draw(|f| app.draw(f))?;

    if crossterm::event::poll(std::time::Duration::from_millis(250))?
      && let Event::Key(key) = event::read()?
      && !app.handle_input(key)
    {
      return Ok(());
    }
  }
}
