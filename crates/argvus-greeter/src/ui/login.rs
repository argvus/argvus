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

mod appearance;
mod authentication;
mod input;
mod render;

// Compiled-in fallback keeps the login surface complete when no external
// avatar source exists or the source cannot be decoded.
const DEFAULT_AVATAR: &[u8] = include_bytes!("../../assets/avatar-default.svg");
/// Public per-account theme projection maintained by argvus-appearance.
const GREETER_THEME_STATE_DIR: &str = "/var/lib/argvus/greeter/themes";

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
}

#[cfg(test)]
mod tests {
  //! Focus/index tests cover the pure navigation invariants without requiring
  //! a terminal, greetd socket, or graphical session.
  use super::appearance::{PublicThemeProjection, load_public_theme_projection};
  use super::input::move_index;
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

  fn input_test_app() -> LoginApp {
    let (commands, _) = std::sync::mpsc::channel();
    let (_, events) = std::sync::mpsc::channel();
    let catalog = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../resources/locales");
    LoginApp::new(
      Config::default(),
      Vec::new(),
      Vec::new(),
      Arc::new(I18n::from_locale(&catalog, "en-US", "greeter").unwrap()),
      commands,
      events,
    )
  }

  #[test]
  fn authentication_input_preserves_navigation_letters_and_backspace() {
    for secret in [true, false] {
      let mut app = input_test_app();
      app.focus = Focus::Password;
      app.prompt.secret = secret;
      for character in ['j', 'k', 'a', 'é'] {
        app.handle_key(KeyEvent::new(KeyCode::Char(character), KeyModifiers::NONE));
        assert!(app.password.ends_with(character));
      }
      assert_eq!(app.password.chars().count(), 4);
      app.handle_key(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE));
      assert_eq!(app.password.chars().count(), 3);
      app.handle_key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
      assert_eq!(app.focus, Focus::Session);
      app.handle_key(KeyEvent::new(KeyCode::BackTab, KeyModifiers::SHIFT));
      assert_eq!(app.focus, Focus::Password);
    }
  }

  #[test]
  fn navigation_letters_still_move_power_selection_outside_text_input() {
    let mut app = input_test_app();
    app.focus = Focus::Power;
    app.handle_key(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE));
    assert_eq!(app.power_selected, 1);
    app.handle_key(KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE));
    assert_eq!(app.power_selected, 0);
    assert!(app.password.is_empty());
  }

  fn account_list_text(app: &LoginApp, width: u16, height: u16) -> String {
    let mut terminal =
      ratatui::Terminal::new(ratatui::backend::TestBackend::new(width, height)).unwrap();
    terminal
      .draw(|frame| app.draw_users(frame, frame.area()))
      .unwrap();
    terminal
      .backend()
      .buffer()
      .content()
      .iter()
      .map(|cell| cell.symbol())
      .collect()
  }

  #[test]
  fn account_list_tracks_navigation_and_resize_without_reordering_accounts() {
    let mut app = input_test_app();
    app.users = (0..12)
      .map(|index| User {
        username: format!("account-{index:02}"),
        uid: 1000 + index,
        display_name: format!("Account{index:02}"),
        avatar: None,
      })
      .collect();
    let identities = app
      .users
      .iter()
      .map(|user| user.username.clone())
      .collect::<Vec<_>>();
    assert!(account_list_text(&app, 24, 5).contains("Account00"));
    for _ in 0..11 {
      app.move_vertical(1);
      let selected = format!("Account{:02}", app.selected_user);
      assert!(account_list_text(&app, 24, 5).contains(&selected));
    }
    assert_eq!(app.selected_user, 11);
    for height in [3, 8, 4, 16] {
      assert!(account_list_text(&app, 24, height).contains("Account11"));
    }
    app.focus = Focus::Password;
    assert!(account_list_text(&app, 24, 3).contains("Account11"));
    app.focus = Focus::User;
    app.move_vertical(1);
    assert_eq!(app.selected_user, 0);
    assert!(account_list_text(&app, 24, 3).contains("Account00"));
    assert_eq!(
      app
        .users
        .iter()
        .map(|user| user.username.clone())
        .collect::<Vec<_>>(),
      identities
    );
  }

  #[test]
  fn account_list_handles_empty_single_and_tiny_viewports() {
    let mut app = input_test_app();
    assert!(account_list_text(&app, 40, 4).contains(&app.i18n.tr("no_local_login_users")));
    app.users.push(User {
      username: "single".to_owned(),
      uid: 1000,
      display_name: "SingleAccount".to_owned(),
      avatar: None,
    });
    for delta in [1, -1] {
      app.move_vertical(delta);
      assert_eq!(app.selected_user, 0);
      assert!(account_list_text(&app, 24, 3).contains("SingleAccount"));
    }
    for height in [1, 2] {
      account_list_text(&app, 2, height);
    }
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
