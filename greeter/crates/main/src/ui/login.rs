//! Ratatui login screen and keyboard-first interaction state.
//!
//! The UI is intentionally a pure stateful view: it receives immutable user
//! and session discovery results, sends typed authentication commands, and
//! renders events returned by the worker. Keeping rendering and side effects
//! separated makes focus behavior testable and prevents the terminal thread
//! from blocking on greetd or D-Bus operations.

use crate::{
  config::Config,
  greetd::{AuthPrompt, Command, Event},
  i18n,
  power::{self, PowerAction},
  session::Session,
  users::User,
};
use argvus_i18n::I18n;
use argvus_theme::Theme;
use argvus_theme::loader::{DEFAULT_THEME, normalize_theme_name};
use argvus_tui::image::ImageSurface;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
  Frame,
  layout::{Alignment, Constraint, Layout, Margin, Rect},
  style::{Modifier, Style, Stylize},
  text::{Line, Span},
  widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Widget, Wrap},
};
use std::{
  fs,
  os::unix::fs::MetadataExt,
  path::{Path, PathBuf},
  sync::{
    Arc,
    mpsc::{Receiver, Sender},
  },
};
use time::{OffsetDateTime, macros::format_description};

// Compiled-in fallback keeps the login surface complete when no external
// avatar source exists or the source cannot be decoded.
const DEFAULT_AVATAR: &[u8] = include_bytes!("../../../../assets/avatar-default.svg");
/// Public per-account theme projection maintained by argvus-appearance.
const GREETER_THEME_STATE_DIR: &str = "/var/lib/argvus/greeter/themes";

fn valid_public_accent(value: &str) -> Option<String> {
  let value = value.trim();
  (value.len() == 7
    && value.starts_with('#')
    && value[1..]
      .chars()
      .all(|character| character.is_ascii_hexdigit()))
  .then(|| value.to_owned())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PublicThemeProjection {
  name: String,
  accent: Option<String>,
}

fn read_owned_regular_file(path: &Path, uid: u32) -> Option<String> {
  let metadata = fs::symlink_metadata(path).ok()?;
  (metadata.is_file() && metadata.uid() == uid).then(|| fs::read_to_string(path).ok())?
}

fn load_public_theme_projection(directory: &Path, uid: u32) -> PublicThemeProjection {
  let projection = directory.join(uid.to_string());
  let name = read_owned_regular_file(&projection, uid)
    .map(|value| normalize_theme_name(value.trim()).to_owned())
    .unwrap_or_else(|| DEFAULT_THEME.to_owned());
  let accent = read_owned_regular_file(&projection.with_extension("accent"), uid)
    .and_then(|value| valid_public_accent(&value));

  PublicThemeProjection { name, accent }
}

/// Interactive controls reachable through Tab and Shift-Tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Focus {
  /// Account selector and user list.
  User,
  /// Password/PAM response field.
  Password,
  /// Wayland session selector.
  Session,
  /// Login action.
  Login,
  /// Power action menu.
  Power,
}

impl Focus {
  // The array is the single source of truth for keyboard focus order.
  const ALL: [Self; 5] = [
    Self::User,
    Self::Password,
    Self::Session,
    Self::Login,
    Self::Power,
  ];

  /// Advances focus in the same order presented by the login form.
  fn next(self) -> Self {
    let index = Self::ALL.iter().position(|item| *item == self).unwrap_or(0);
    Self::ALL[(index + 1) % Self::ALL.len()]
  }

  /// Moves focus backwards, wrapping from the first control to the last.
  fn previous(self) -> Self {
    let index = Self::ALL.iter().position(|item| *item == self).unwrap_or(0);
    Self::ALL[(index + Self::ALL.len() - 1) % Self::ALL.len()]
  }
}

/// Complete in-memory state for the login screen.
///
/// Fields that represent external effects (`commands`, `events`) are kept
/// alongside visual state so a single event loop can coordinate rendering and
/// authentication without sharing mutable state across threads.
pub struct LoginApp {
  /// Read-only presentation and session preferences.
  config: Config,
  /// Accounts discovered before the TUI started.
  users: Vec<User>,
  /// Validated Wayland sessions available to greetd.
  sessions: Vec<Session>,
  /// Shared translation catalog for all user-facing text.
  i18n: Arc<I18n>,
  /// Commands sent to the background greetd worker.
  commands: Sender<Command>,
  /// Authentication and lifecycle events from the worker.
  events: Receiver<Event>,
  /// Resolved ARGVUS semantic colors.
  theme: Theme,
  /// UID whose public theme projection is currently loaded.
  theme_user_uid: Option<u32>,
  /// Theme identifier passed to the post-login handoff splash.
  theme_name: String,
  /// Optional validated accent passed to the post-login handoff splash.
  theme_accent: Option<String>,
  /// Current decoded avatar or the built-in fallback.
  avatar: Option<ImageSurface>,
  /// Source path used to avoid decoding the same avatar on every frame.
  avatar_key: Option<PathBuf>,
  /// Current keyboard focus.
  focus: Focus,
  /// Selected account index in `users`.
  selected_user: usize,
  /// Selected session index in `sessions`.
  selected_session: usize,
  /// Current response text; cleared after submission or failure.
  password: String,
  /// Most recent prompt metadata returned by greetd.
  prompt: AuthPrompt,
  /// Whether the next Enter submits a response to an active prompt.
  waiting_for_prompt: bool,
  /// Initial password entered before greetd emits its secret prompt.
  pending_secret_response: Option<String>,
  /// Prevents a second CreateSession request while greetd is processing login.
  ///
  /// greetd permits only one session configuration per worker connection. A
  /// repeated Enter event must therefore be ignored until the current attempt
  /// reaches a terminal event; otherwise the server reports that a session is
  /// already being configured and the visible login flow becomes misleading.
  authentication_in_progress: bool,
  /// Status message and semantic severity shown below the form.
  status: Option<(String, StatusKind)>,
  /// Whether the power overlay is currently visible.
  power_open: bool,
  /// Selected action inside the power overlay.
  power_selected: usize,
  /// Signals the outer application loop to leave the terminal.
  pub quit: bool,
  /// Prevents keyboard interaction once the session overlay owns the display.
  handoff_active: bool,
}

/// Semantic class used to select status colors without embedding colors in
/// authentication logic.
#[derive(Debug, Clone, Copy)]
enum StatusKind {
  /// Non-error progress or informational text.
  Info,
  /// A diagnostic or backend warning that does not necessarily stop login.
  Warn,
  /// A recoverable authentication or system error.
  Error,
  /// A successfully requested or completed action.
  Success,
}

impl LoginApp {
  /// Creates a login state and initializes the first avatar/empty-state view.
  ///
  /// The default session is selected by discovery metadata, while user focus
  /// starts at the account selector so keyboard users have a predictable entry
  /// point.
  #[allow(clippy::too_many_arguments)]
  pub fn new(
    config: Config,
    users: Vec<User>,
    sessions: Vec<Session>,
    i18n: Arc<I18n>,
    commands: Sender<Command>,
    events: Receiver<Event>,
  ) -> Self {
    // Discovery already computed the configured default, so this lookup keeps
    // configuration policy out of the rendering layer.
    let selected_session = sessions
      .iter()
      .position(|session| session.is_default)
      .unwrap_or(0);
    // Build the complete state before loading the avatar so all fallback paths
    // can use the final selected-user index.
    let mut app = Self {
      config,
      users,
      sessions,
      i18n,
      commands,
      events,
      // Theme::load is the shared ARGVUS source of semantic UI colors.
      theme: Theme::load(),
      theme_user_uid: None,
      theme_name: String::new(),
      theme_accent: None,
      avatar: None,
      avatar_key: None,
      focus: Focus::User,
      selected_user: 0,
      selected_session,
      password: String::new(),
      prompt: AuthPrompt {
        message: String::new(),
        secret: true,
      },
      waiting_for_prompt: false,
      pending_secret_response: None,
      authentication_in_progress: false,
      status: None,
      power_open: false,
      power_selected: 0,
      quit: false,
      handoff_active: false,
    };
    // The account area always renders an image, using the default asset when
    // the selected user has no usable external avatar.
    app.refresh_avatar();
    app.refresh_theme();
    app.set_empty_state();
    app
  }

  /// Renders the complete screen, including the optional power overlay.
  ///
  /// The small-terminal guard prevents Ratatui widgets and image protocols
  /// from competing for a region too small to remain legible.
  pub fn draw(&mut self, frame: &mut Frame) {
    let area = frame.area();
    if area.width < argvus_tui::MIN_WIDTH || area.height < argvus_tui::MIN_HEIGHT {
      argvus_tui::chrome::draw_too_small(
        frame,
        area,
        &self.theme,
        &self.i18n.tr("terminal_window_is_too_small"),
        &self.i18n.tr("minimum_size"),
        &self.i18n.tr("current_size"),
      );
      return;
    }

    // The outer frame establishes a consistent ARGVUS background and border
    // before child widgets draw their own focused panels.
    Block::new()
      .borders(Borders::ALL)
      .border_style(Style::new().fg(self.theme.border_active))
      .bg(self.theme.background)
      .render(area, frame.buffer_mut());
    let inner = area.inner(Margin::new(1, 1));
    // Header, clock, body, message box, and footer use explicit regions. The
    // message box is intentionally outside the form so every status appears
    // in one predictable location only.
    let rows = Layout::vertical([
      Constraint::Length(1),
      Constraint::Length(2),
      Constraint::Min(1),
      Constraint::Length(3),
      Constraint::Length(2),
    ])
    .split(inner);
    argvus_tui::chrome::draw_header(
      frame,
      rows[0],
      &self.theme,
      argvus_tui::chrome::Header {
        title: &self.i18n.tr("title"),
        version: None,
        version_label: "",
      },
    );
    self.draw_clock(frame, rows[1]);
    self.draw_body(frame, rows[2]);
    self.draw_message_box(frame, rows[3]);
    self.draw_footer(frame, rows[4]);

    if self.power_open {
      self.draw_power_menu(frame, area);
    }
  }

  /// Draws the optional local clock and date without affecting form state.
  fn draw_clock(&self, frame: &mut Frame, area: Rect) {
    if !self.config.appearance.show_clock {
      return;
    }
    // Local time is best effort because the login process may lack a complete
    // timezone environment. The epoch fallback keeps rendering infallible.
    let now = OffsetDateTime::now_local().unwrap_or(OffsetDateTime::UNIX_EPOCH);
    let time = now
      .format(&format_description!("[hour]:[minute]"))
      .unwrap_or_else(|_| "00:00".to_string());
    let date = if self.config.appearance.show_date {
      now
        .format(&format_description!("[year]-[month]-[day]"))
        .unwrap_or_else(|_| "1970-01-01".to_string())
    } else {
      String::new()
    };
    frame.render_widget(
      Paragraph::new(Line::from(vec![
        Span::styled(
          time,
          Style::new()
            .fg(self.theme.foreground)
            .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
          if date.is_empty() {
            String::new()
          } else {
            format!("  {date}")
          },
          Style::new().fg(self.theme.muted),
        ),
      ])),
      area,
    );
  }

  /// Splits the body between account navigation and authentication controls.
  fn draw_body(&mut self, frame: &mut Frame, area: Rect) {
    let columns =
      Layout::horizontal([Constraint::Percentage(38), Constraint::Percentage(62)]).split(area);
    self.draw_users(frame, columns[0]);
    self.draw_login_panel(frame, columns[1]);
  }

  /// Renders the account list and its focused selection state.
  fn draw_users(&self, frame: &mut Frame, area: Rect) {
    let title = format!(" {} ", self.i18n.tr("users"));
    let items = if self.users.is_empty() {
      vec![ListItem::new(self.i18n.tr("no_local_login_users"))]
    } else {
      self
        .users
        .iter()
        .enumerate()
        .map(|(index, user)| {
          // The icon communicates avatar availability without exposing the
          // underlying filesystem path to the person logging in.
          let avatar = if user.avatar.is_some() {
            "󰀄 "
          } else {
            "󰀉 "
          };
          let style = if index == self.selected_user && self.focus == Focus::User {
            Style::new()
              .fg(self.theme.selected_foreground)
              .bg(self.theme.selected_background)
              .add_modifier(Modifier::BOLD)
          } else {
            Style::new().fg(self.theme.foreground)
          };
          ListItem::new(format!("{avatar}{}", user.display_name)).style(style)
        })
        .collect()
    };
    let border = if self.focus == Focus::User {
      self.theme.border_active
    } else {
      self.theme.border
    };
    frame.render_widget(
      List::new(items)
        .block(
          Block::bordered()
            .title(title)
            .border_style(Style::new().fg(border)),
        )
        .highlight_style(
          Style::new()
            .fg(self.theme.selected_foreground)
            .bg(self.theme.selected_background),
        ),
      area,
    );
  }

  /// Renders the account preview, password response, session selector, and
  /// login action. Status messages are rendered by [`Self::draw_message_box`]
  /// so they cannot be duplicated inside the form.
  fn draw_login_panel(&mut self, frame: &mut Frame, area: Rect) {
    let inner = area.inner(Margin::new(2, 1));
    // The account row is taller than the other controls so the avatar can be
    // framed without collapsing its image protocol target to one cell.
    let rows = Layout::vertical([
      Constraint::Length(5),
      Constraint::Length(3),
      Constraint::Length(3),
      Constraint::Length(3),
    ])
    .split(inner);
    let selected_name = self
      .users
      .get(self.selected_user)
      .map(|user| user.display_name.as_str())
      .unwrap_or("ARGVUS");
    // Account focus is represented by the password/login controls because the
    // account preview itself is informational after selection.
    let border = if matches!(self.focus, Focus::Password | Focus::Login) {
      self.theme.border_active
    } else {
      self.theme.border
    };
    frame.render_widget(
      Block::bordered()
        .title(self.i18n.tr("account"))
        .border_style(Style::new().fg(border)),
      rows[0],
    );
    // Kitty preserves the avatar's square aspect ratio in terminal-cell
    // coordinates. A six-cell inner target at this row height matches the
    // visible image width, keeping the frame adjacent to the actual pixels
    // instead of surrounding unused horizontal letterboxing space.
    let account_columns = Layout::horizontal([Constraint::Length(8), Constraint::Min(1)])
      .split(rows[0].inner(Margin::new(1, 0)));
    if let Some(avatar) = self.avatar.as_mut() {
      // Draw the frame first and render the image into its inner rectangle so
      // the border remains visible for both Kitty graphics and text fallback.
      let avatar_area = account_columns[0];
      frame.render_widget(
        Block::bordered().border_style(Style::new().fg(border)),
        avatar_area,
      );
      avatar.render(
        frame,
        avatar_area.inner(Margin::new(1, 1)),
        self.theme.background,
        self.theme.accent,
      );
    }
    // Only the selected account name is shown here. The actual avatar source
    // is an implementation detail and must not be presented as login text.
    frame.render_widget(
      Paragraph::new(Line::from(Span::styled(
        selected_name,
        Style::new()
          .fg(self.theme.foreground)
          .add_modifier(Modifier::BOLD),
      )))
      .alignment(Alignment::Left),
      account_columns[1],
    );
    let prompt = if self.prompt.message.is_empty() {
      self.i18n.tr("password")
    } else {
      i18n::auth_message(&self.i18n, &self.prompt.message)
    };
    let password = if self.prompt.secret {
      "•".repeat(self.password.chars().count())
    } else {
      self.password.clone()
    };
    frame.render_widget(
      Paragraph::new(password).block(Block::bordered().title(prompt).border_style(
        Style::new().fg(if self.focus == Focus::Password {
          self.theme.border_active
        } else {
          self.theme.border
        }),
      )),
      rows[1],
    );
    let session = self
      .sessions
      .get(self.selected_session)
      .map(|item| item.name.as_str())
      .unwrap_or("-");
    let session_style = if self.focus == Focus::Session {
      Style::new()
        .fg(self.theme.selected_foreground)
        .bg(self.theme.selected_background)
    } else {
      Style::new().fg(self.theme.foreground)
    };
    frame.render_widget(
      Paragraph::new(Line::from(Span::styled(
        format!("‹ {session} ›"),
        session_style,
      )))
      .block(
        Block::bordered()
          .title(self.i18n.tr("session"))
          .border_style(Style::new().fg(if self.focus == Focus::Session {
            self.theme.border_active
          } else {
            self.theme.border
          })),
      ),
      rows[2],
    );
    let login_style = if self.focus == Focus::Login {
      Style::new()
        .fg(self.theme.selected_foreground)
        .bg(self.theme.accent)
        .add_modifier(Modifier::BOLD)
    } else {
      Style::new()
        .fg(self.theme.foreground)
        .bg(self.theme.surface)
    };
    frame.render_widget(
      Paragraph::new(Line::from(Span::styled(
        format!("  {}  ", self.i18n.tr("login")),
        login_style,
      )))
      .alignment(Alignment::Center)
      .block(
        Block::bordered().border_style(Style::new().fg(if self.focus == Focus::Login {
          self.theme.accent
        } else {
          self.theme.border
        })),
      ),
      rows[3],
    );
  }

  /// Renders the single status channel for progress, warnings, errors, and
  /// successful operations directly above the navigation footer.
  fn draw_message_box(&self, frame: &mut Frame, area: Rect) {
    let (message, color) = match self.status.as_ref() {
      Some((text, StatusKind::Info)) => (text.clone(), self.theme.muted),
      Some((text, StatusKind::Warn)) => (text.clone(), self.theme.warning),
      Some((text, StatusKind::Error)) => (text.clone(), self.theme.error),
      Some((text, StatusKind::Success)) => (text.clone(), self.theme.success),
      None => (self.i18n.tr("select_user_help"), self.theme.muted),
    };
    frame.render_widget(
      Paragraph::new(message)
        .wrap(Wrap { trim: true })
        .style(Style::new().fg(color))
        .block(
          Block::bordered()
            .title(self.i18n.tr("status"))
            .border_style(Style::new().fg(color)),
        ),
      area,
    );
  }

  /// Renders the keyboard help line shared by all normal login states.
  fn draw_footer(&self, frame: &mut Frame, area: Rect) {
    frame.render_widget(
      Paragraph::new(self.i18n.tr("navigation_help"))
        .alignment(Alignment::Right)
        .style(Style::new().fg(self.theme.muted).bg(self.theme.surface)),
      area,
    );
  }

  /// Draws a centered power menu above the normal screen contents.
  fn draw_power_menu(&self, frame: &mut Frame, area: Rect) {
    // Clear prevents the underlying form from visually leaking through the
    // modal while retaining the terminal's current background.
    let popup = argvus_tui::chrome::centered(area, area.width.min(42), 9);
    frame.render_widget(Clear, popup);
    let actions = [
      PowerAction::Shutdown,
      PowerAction::Restart,
      PowerAction::Suspend,
    ];
    let items = actions
      .iter()
      .enumerate()
      .map(|(index, action)| {
        let style = if index == self.power_selected {
          Style::new()
            .fg(self.theme.selected_foreground)
            .bg(self.theme.selected_background)
            .add_modifier(Modifier::BOLD)
        } else {
          Style::new().fg(self.theme.foreground)
        };
        ListItem::new(action.label(&self.i18n)).style(style)
      })
      .collect::<Vec<_>>();
    frame.render_widget(
      List::new(items).block(
        Block::bordered()
          .title(self.i18n.tr("power"))
          .border_style(Style::new().fg(self.theme.border_active)),
      ),
      popup,
    );
  }

  /// Reuses the decoded image when the selected account has not changed.
  ///
  /// The fallback is loaded from bytes so no user-controlled path is needed
  /// for the default presentation.
  fn refresh_avatar(&mut self) {
    let key = self
      .users
      .get(self.selected_user)
      .and_then(|user| user.avatar.clone());
    // Avoid decoding and reinitializing the Kitty/iTerm/sixel protocol on
    // every frame when keyboard input did not change the selected account.
    if key == self.avatar_key && self.avatar.is_some() {
      return;
    }
    self.avatar_key = key.clone();
    self.avatar = key
      .as_deref()
      .and_then(ImageSurface::from_path)
      .or_else(|| ImageSurface::from_bytes(DEFAULT_AVATAR, true));
  }

  /// Loads the selected account's pre-login theme projection.
  ///
  /// Only the UID-named projection is read here. The greeter intentionally
  /// never traverses a user's private home because it runs before PAM has
  /// authenticated that account.
  fn refresh_theme(&mut self) {
    let Some(user) = self.users.get(self.selected_user) else {
      return;
    };
    if self.theme_user_uid == Some(user.uid) {
      return;
    }

    let projection = load_public_theme_projection(Path::new(GREETER_THEME_STATE_DIR), user.uid);
    self.theme =
      Theme::load_for_theme_name_and_accent(&projection.name, projection.accent.as_deref());
    self.theme_name = projection.name;
    self.theme_accent = projection.accent;
    self.theme_user_uid = Some(user.uid);
  }

  /// Handles one terminal key event and updates only local UI state.
  ///
  /// Authentication and power actions are sent to their respective owners;
  /// this method never performs blocking I/O directly on the TUI thread.
  pub fn handle_key(&mut self, key: KeyEvent) {
    if self.handoff_active {
      return;
    }
    if key.kind != KeyEventKind::Press {
      return;
    }
    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
      // Ctrl+C is deliberately consumed. Exiting the login process leaves the
      // greetd compositor without a surface and produces a blank screen.
      return;
    }
    if self.power_open {
      // The modal owns all keys while open so normal form controls cannot be
      // activated behind it.
      self.handle_power_key(key.code);
      return;
    }
    match key.code {
      KeyCode::Tab => self.focus = self.focus.next(),
      KeyCode::BackTab => self.focus = self.focus.previous(),
      KeyCode::Up | KeyCode::Char('k') => self.move_vertical(-1),
      KeyCode::Down | KeyCode::Char('j') => self.move_vertical(1),
      KeyCode::Left | KeyCode::Char('h') if self.focus == Focus::Session => self.move_session(-1),
      KeyCode::Right | KeyCode::Char('l') if self.focus == Focus::Session => self.move_session(1),
      KeyCode::Char(character) if self.focus == Focus::Password => self.password.push(character),
      KeyCode::Backspace if self.focus == Focus::Password => {
        self.password.pop();
      }
      KeyCode::Enter => self.activate_focus(),
      KeyCode::Esc => self.cancel_input(),
      _ => {}
    }
    // Avatar refresh is cheap when the selected user is unchanged and ensures
    // selection changes become visible in the same event cycle.
    self.refresh_avatar();
    self.refresh_theme();
  }

  /// Moves the active indexed control vertically, with wrap-around behavior.
  fn move_vertical(&mut self, delta: isize) {
    match self.focus {
      Focus::User => self.selected_user = move_index(self.selected_user, self.users.len(), delta),
      Focus::Session => {
        self.selected_session = move_index(self.selected_session, self.sessions.len(), delta)
      }
      Focus::Power => self.power_selected = move_index(self.power_selected, 3, delta),
      _ => {}
    }
  }

  /// Changes the selected session when focus is on the session control.
  fn move_session(&mut self, delta: isize) {
    self.selected_session = move_index(self.selected_session, self.sessions.len(), delta);
  }

  /// Executes the action associated with the currently focused control.
  fn activate_focus(&mut self) {
    match self.focus {
      Focus::User => self.focus = Focus::Password,
      Focus::Password => self.submit(),
      Focus::Session => self.focus = Focus::Login,
      Focus::Login => self.submit(),
      Focus::Power => self.power_open = true,
    }
  }

  /// Cancels the current password entry or returns focus to account selection.
  fn cancel_input(&mut self) {
    if !self.password.is_empty() {
      self.password.clear();
      self.status = None;
    } else {
      self.focus = Focus::User;
    }
  }

  /// Handles navigation and activation inside the power modal.
  fn handle_power_key(&mut self, key: KeyCode) {
    match key {
      KeyCode::Esc => self.power_open = false,
      KeyCode::Up | KeyCode::Char('k') => {
        self.power_selected = move_index(self.power_selected, 3, -1)
      }
      KeyCode::Down | KeyCode::Char('j') => {
        self.power_selected = move_index(self.power_selected, 3, 1)
      }
      KeyCode::Enter => {
        // The selected index is bounded by move_index, making this fixed array
        // safe while keeping the menu order explicit.
        let action = [
          PowerAction::Shutdown,
          PowerAction::Restart,
          PowerAction::Suspend,
        ][self.power_selected];
        match power::request(action) {
          Ok(()) => self.status = Some((self.i18n.tr("power_requested"), StatusKind::Success)),
          Err(error) => {
            self.status = Some((
              format!("{}: {error}", self.i18n.tr("power_failed")),
              StatusKind::Error,
            ))
          }
        }
        self.power_open = false;
      }
      _ => {}
    }
  }

  /// Starts authentication or posts the response to the active PAM prompt.
  fn submit(&mut self) {
    if self.waiting_for_prompt {
      // `take` clears the secret immediately after ownership moves to the
      // worker, minimizing how long credentials remain in UI state.
      let response = std::mem::take(&mut self.password);
      self.waiting_for_prompt = false;
      if self
        .commands
        .send(Command::AuthResponse(Some(response)))
        .is_err()
      {
        self.authentication_in_progress = false;
        self.set_error("internal_channel_unavailable");
      }
      return;
    }
    // A key repeat or an impatient second Enter must not create another greetd
    // session while the first request is still waiting for PAM. The worker
    // remains the owner of the active socket until it emits a final event.
    if self.authentication_in_progress {
      return;
    }
    let Some(user) = self.users.get(self.selected_user) else {
      self.set_error("no_login_user");
      return;
    };
    let Some(session) = self.sessions.get(self.selected_session) else {
      self.set_error("no_wayland_session");
      return;
    };
    // Some PAM stacks request a password only after CreateSession; retain an
    // initial response so it can be posted automatically when that prompt
    // arrives.
    let initial = std::mem::take(&mut self.password);
    self.pending_secret_response = (!initial.is_empty()).then_some(initial);
    self.authentication_in_progress = true;
    self.status = Some((self.i18n.tr("starting_authentication"), StatusKind::Info));
    self.focus = Focus::Password;
    if self
      .commands
      .send(Command::Begin {
        username: user.username.clone(),
        session: session.clone(),
        theme: self.theme_name.clone(),
        accent: self.theme_accent.clone(),
      })
      .is_err()
    {
      self.authentication_in_progress = false;
      self.set_error("internal_channel_unavailable");
    }
  }

  /// Drains all currently available worker events without blocking rendering.
  pub fn poll_events(&mut self) {
    while let Ok(event) = self.events.try_recv() {
      self.handle_event(event);
    }
  }

  /// Applies one worker event to the visible login state.
  fn handle_event(&mut self, event: Event) {
    match event {
      Event::Ready => {}
      Event::AuthMessage(prompt) => {
        // A secret prompt may arrive after the user already entered a
        // password; post that pending response without displaying it again.
        if prompt.secret
          && let Some(response) = self.pending_secret_response.take()
        {
          let _ = self.commands.send(Command::AuthResponse(Some(response)));
          return;
        }
        self.prompt = prompt;
        self.waiting_for_prompt = true;
        self.focus = Focus::Password;
        self.status = None;
      }
      // Authentication messages are normalized at the UI boundary so backend
      // wording does not leak into the rest of the state machine.
      Event::Info(message) => {
        self.status = Some((i18n::auth_message(&self.i18n, &message), StatusKind::Warn))
      }
      Event::Error(message) => {
        self.password.clear();
        self.waiting_for_prompt = false;
        self.pending_secret_response = None;
        self.authentication_in_progress = false;
        self.set_status(i18n::auth_message(&self.i18n, &message), StatusKind::Error);
      }
      Event::AuthFailed(message) => {
        self.password.clear();
        self.waiting_for_prompt = false;
        self.pending_secret_response = None;
        self.authentication_in_progress = false;
        let text = if message.is_empty() {
          self.i18n.tr("authentication_failed")
        } else {
          i18n::auth_message(&self.i18n, &message)
        };
        self.set_status(text, StatusKind::Error);
      }
      Event::AuthPromptUnavailable => {
        self.waiting_for_prompt = false;
        self.pending_secret_response = None;
        self.authentication_in_progress = false;
        self.set_error("no_active_auth_prompt");
      }
      Event::GreeterUnavailable => {
        self.waiting_for_prompt = false;
        self.pending_secret_response = None;
        self.authentication_in_progress = false;
        self.set_error("login_service_unavailable");
      }
      Event::SessionStarting => {
        self.set_status(self.i18n.tr("starting_session"), StatusKind::Info);
        self.waiting_for_prompt = false;
      }
      Event::SessionStarted => {
        self.set_status(self.i18n.tr("session_started"), StatusKind::Success);
        self.authentication_in_progress = false;
        // Keep Kitty and the greeter compositor alive under the mapped overlay
        // until greetd sends SIGTERM. Returning here would restore the terminal
        // and create the visible gap this handoff prevents.
        self.handoff_active = true;
      }
    }
  }

  /// Displays an initial state error when discovery found no usable choices.
  fn set_empty_state(&mut self) {
    if self.users.is_empty() {
      self.set_error("no_local_login_users");
    } else if self.sessions.is_empty() {
      self.set_error("no_wayland_sessions");
    }
  }

  /// Translates a catalog key into an error status.
  fn set_error(&mut self, key: &str) {
    self.set_status(self.i18n.tr(key), StatusKind::Error);
  }

  /// Stores a status message together with its semantic severity.
  fn set_status(&mut self, text: String, kind: StatusKind) {
    self.status = Some((text, kind));
  }
}

/// Moves an index by one step and wraps at either end of a non-empty list.
fn move_index(index: usize, length: usize, delta: isize) -> usize {
  // Empty lists are valid during discovery failures; returning zero avoids
  // underflow and lets the UI show its empty-state message.
  if length == 0 {
    return 0;
  }
  if delta < 0 {
    if index == 0 { length - 1 } else { index - 1 }
  } else if index + 1 >= length {
    0
  } else {
    index + 1
  }
}

#[cfg(test)]
mod tests {
  //! Focus/index tests cover the pure navigation invariants without requiring
  //! a terminal, greetd socket, or graphical session.
  use super::*;

  #[test]
  /// Selection wraps in both directions and remains stable for empty lists.
  fn index_wraps_in_both_directions() {
    assert_eq!(move_index(0, 3, -1), 2);
    assert_eq!(move_index(2, 3, 1), 0);
    assert_eq!(move_index(1, 0, 1), 0);
  }

  #[test]
  /// Tab order includes the first and last controls as a closed cycle.
  fn focus_cycles_with_tab() {
    assert_eq!(Focus::User.next(), Focus::Password);
    assert_eq!(Focus::User.previous(), Focus::Power);
  }

  #[test]
  fn public_projection_accepts_owned_official_theme_and_accent() {
    let directory = temporary_projection_directory();
    let uid = current_uid();
    fs::write(directory.join(uid.to_string()), "argvus-light\n").unwrap();
    fs::write(directory.join(format!("{uid}.accent")), " #12aBcD \n").unwrap();

    assert_eq!(
      load_public_theme_projection(&directory, uid),
      PublicThemeProjection {
        name: "argvus-light".to_owned(),
        accent: Some("#12aBcD".to_owned()),
      }
    );
    fs::remove_dir_all(directory).unwrap();
  }

  #[test]
  fn public_projection_falls_back_for_missing_invalid_or_foreign_state() {
    let directory = temporary_projection_directory();
    let uid = current_uid();
    fs::write(directory.join(uid.to_string()), "not-a-theme\n").unwrap();
    fs::write(directory.join(format!("{uid}.accent")), "white\n").unwrap();

    assert_eq!(
      load_public_theme_projection(&directory, uid),
      PublicThemeProjection {
        name: DEFAULT_THEME.to_owned(),
        accent: None,
      }
    );

    let foreign_uid = uid.checked_add(1).unwrap_or(uid - 1);
    fs::write(directory.join(foreign_uid.to_string()), "argvus-light\n").unwrap();
    assert_eq!(
      load_public_theme_projection(&directory, foreign_uid),
      PublicThemeProjection {
        name: DEFAULT_THEME.to_owned(),
        accent: None,
      }
    );
    fs::remove_dir_all(directory).unwrap();
  }

  fn current_uid() -> u32 {
    std::os::unix::fs::MetadataExt::uid(&fs::metadata(".").unwrap())
  }

  fn temporary_projection_directory() -> PathBuf {
    let path = std::env::temp_dir().join(format!(
      "argvus-greeter-theme-test-{}-{}",
      std::process::id(),
      std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
    ));
    fs::create_dir(&path).unwrap();
    path
  }
}
