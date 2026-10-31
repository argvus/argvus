//! Onboarding navigation hosted in the Control Center's existing page tree.
use crate::app::{App, Route};
use crate::home::HomeDestination;
use argvus_i18n::tr;
use argvus_welcome::state::{Progress, StateStore, StepStatus};
use crossterm::event::KeyCode;
use ratatui::{
  Frame,
  layout::{Constraint, Layout},
  style::Style,
  widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};

pub struct Step {
  pub id: &'static str,
  pub destination: HomeDestination,
}

pub fn steps() -> Vec<Step> {
  vec![
    #[cfg(feature = "locale")]
    Step {
      id: "language",
      destination: HomeDestination::LanguageRegion,
    },
    #[cfg(feature = "displays")]
    Step {
      id: "displays",
      destination: HomeDestination::Displays,
    },
    #[cfg(feature = "hardware")]
    Step {
      id: "hardware",
      destination: HomeDestination::HardwareOverview,
    },
    #[cfg(feature = "network")]
    Step {
      id: "network",
      destination: HomeDestination::Network,
    },
    #[cfg(feature = "audio")]
    Step {
      id: "audio",
      destination: HomeDestination::Audio,
    },
    #[cfg(feature = "bluetooth")]
    Step {
      id: "bluetooth",
      destination: HomeDestination::Bluetooth,
    },
    #[cfg(feature = "appearance")]
    Step {
      id: "appearance",
      destination: HomeDestination::DesktopStyle,
    },
    #[cfg(feature = "power")]
    Step {
      id: "power",
      destination: HomeDestination::Power,
    },
    #[cfg(feature = "apps")]
    Step {
      id: "applications",
      destination: HomeDestination::DefaultApplications,
    },
    #[cfg(feature = "session")]
    Step {
      id: "session",
      destination: HomeDestination::Session,
    },
  ]
}

pub struct Welcome {
  pub progress: Progress,
  pub selected: usize,
  pub active: bool,
  pub reviewing: bool,
  pub done_focus: bool,
  pub done_blocked: bool,
  pub error: bool,
  store: Option<StateStore>,
}

impl Default for Welcome {
  fn default() -> Self {
    let store = StateStore::new().ok();
    let (progress, warning) = store.as_ref().map(StateStore::load).unwrap_or_default();
    let selected = steps()
      .iter()
      .position(|step| step.id == progress.current_category)
      .unwrap_or(0);
    Self {
      progress,
      selected,
      active: false,
      reviewing: false,
      done_focus: false,
      done_blocked: false,
      error: warning.is_some() || store.is_none(),
      store,
    }
  }
}

impl Welcome {
  fn finish_current(&mut self, status: StepStatus) -> Option<bool> {
    let steps = steps();
    let step = steps.get(self.selected)?;
    let previous = self.progress.clone();
    self.progress.set_status(step.id, status);
    let next = steps.get(self.selected + 1);
    if let Some(next) = next {
      self.progress.current_category = next.id.to_owned();
    }
    if !self.save() {
      self.progress = previous;
      return None;
    }
    self.reviewing = false;
    if next.is_some() {
      self.selected += 1;
    }
    Some(next.is_some())
  }

  fn save(&mut self) -> bool {
    self.error = self
      .store
      .as_ref()
      .is_none_or(|store| store.save(&mut self.progress).is_err());
    !self.error
  }
}

impl App {
  pub fn poll_welcome_request(&mut self) -> bool {
    if self.welcome.active {
      if let Some(store) = &self.welcome.store {
        let _ = store.take_open_request();
      }
      return false;
    }
    if !matches!(self.route, Route::Home | Route::Welcome) {
      return false;
    }
    if self
      .welcome
      .store
      .as_ref()
      .is_some_and(|store| store.take_open_request().unwrap_or(false))
    {
      self.open_welcome();
      return true;
    }
    false
  }

  pub fn open_welcome(&mut self) {
    self.welcome.active = false;
    self.welcome.reviewing = false;
    self.route = Route::Welcome;
    self.clear_global_search();
  }

  fn open_welcome_step(&mut self) {
    let Some(step) = steps()
      .get(self.welcome.selected)
      .map(|step| (step.id, step.destination))
    else {
      return;
    };
    self.welcome.progress.started = true;
    self.welcome.progress.current_category = step.0.to_owned();
    if !self.welcome.save() {
      return;
    }
    self.open_destination(step.1);
    self.welcome.done_blocked = false;
    self.welcome.done_focus = false;
    self.welcome.active = true;
    self.welcome.reviewing = false;
    // Settings and appearance use this when returning from their root page.
    self.return_route = Route::Home;
  }

  pub(crate) fn finish_welcome_step(&mut self, status: StepStatus) {
    match self.welcome.finish_current(status) {
      Some(true) => self.open_welcome_step(),
      Some(false) => {
        self.welcome.active = false;
        self.route = Route::Welcome;
      }
      None => {}
    }
  }

  pub fn handle_welcome(&mut self, key: KeyCode) {
    match key {
      KeyCode::Up => {
        self.welcome.selected = self.welcome.selected.saturating_sub(1);
        self.welcome.reviewing = false;
      }
      KeyCode::Down => {
        self.welcome.selected = (self.welcome.selected + 1).min(steps().len().saturating_sub(1));
        self.welcome.reviewing = false;
      }
      KeyCode::Enter if self.welcome.reviewing => self.finish_welcome_step(StepStatus::Reviewed),
      KeyCode::Enter | KeyCode::Char('o') => self.open_welcome_step(),
      KeyCode::Char('s') => self.finish_welcome_step(StepStatus::Skipped),
      KeyCode::Char('u') => self.finish_welcome_step(StepStatus::NotApplicable),
      KeyCode::Char('f') => {
        let completed = self.welcome.progress.completed;
        self.welcome.progress.completed = true;
        if self.welcome.save() {
          self.welcome.active = false;
          self.welcome.reviewing = false;
          self.route = Route::Home;
        } else {
          self.welcome.progress.completed = completed;
        }
      }
      KeyCode::Esc => {
        self.welcome.active = false;
        self.welcome.reviewing = false;
        self.route = Route::Home;
      }
      _ => {}
    }
  }
}

pub fn draw(app: &mut App, frame: &mut Frame) {
  let area = frame.area();
  let title = format!(
    "{} > {} > {}",
    tr(app.lang, "control_center.argvus_control_center"),
    tr(app.lang, "control_center.card_tools"),
    tr(app.lang, "control_center.welcome_title")
  );
  let block = Block::default()
    .borders(Borders::ALL)
    .title(tr(app.lang, "control_center.welcome_title"))
    .style(
      Style::default()
        .fg(app.theme.foreground)
        .bg(app.theme.background),
    );
  let inner = block.inner(area);
  frame.render_widget(block, area);
  let rows = Layout::vertical([
    Constraint::Length(2),
    Constraint::Length(3),
    Constraint::Min(1),
    Constraint::Length(3),
    Constraint::Length(3),
  ])
  .split(inner);
  frame.render_widget(Paragraph::new(title).wrap(Wrap { trim: true }), rows[0]);
  frame.render_widget(
    Paragraph::new(tr(app.lang, "control_center.welcome_intro")).wrap(Wrap { trim: true }),
    rows[1],
  );
  let items: Vec<ListItem<'static>> = steps()
    .iter()
    .map(|step| {
      let (mark, status) = match app.welcome.progress.status(step.id) {
        StepStatus::Reviewed => ("[x]", "reviewed"),
        StepStatus::Skipped => ("[-]", "skipped"),
        StepStatus::NotApplicable => ("[-]", "not_applicable"),
        StepStatus::Pending => ("[ ]", "pending"),
      };
      ListItem::new(format!(
        "{} {} — {}",
        mark,
        tr(
          app.lang,
          &format!("control_center.welcome_step_{}", step.id)
        ),
        tr(app.lang, &format!("control_center.welcome_status_{status}"))
      ))
    })
    .collect();
  frame.render_stateful_widget(
    List::new(items)
      .highlight_style(Style::default().fg(app.theme.accent))
      .highlight_symbol("> "),
    rows[2],
    &mut ListState::default().with_selected(Some(app.welcome.selected)),
  );
  let message = if app.welcome.error {
    "control_center.welcome_save_error"
  } else if app.welcome.reviewing {
    "control_center.welcome_confirm"
  } else if app.welcome.progress.completed {
    "control_center.welcome_finished"
  } else {
    "control_center.welcome_return"
  };
  frame.render_widget(
    Paragraph::new(tr(app.lang, message)).wrap(Wrap { trim: true }),
    rows[3],
  );
  frame.render_widget(
    Paragraph::new(tr(app.lang, "control_center.welcome_keys")).wrap(Wrap { trim: true }),
    rows[4],
  );
  if !app.welcome.progress.opened {
    app.welcome.progress.opened = true;
    if !app.welcome.save() {
      app.welcome.progress.opened = false;
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn welcome(directory: &std::path::Path) -> Welcome {
    Welcome {
      progress: Progress::default(),
      selected: 0,
      active: false,
      reviewing: false,
      done_focus: false,
      done_blocked: false,
      error: false,
      store: Some(StateStore::at(directory.to_path_buf())),
    }
  }

  #[cfg(all(feature = "locale", feature = "displays"))]
  #[test]
  fn confirmation_checks_language_and_persists_the_next_step() {
    let directory = tempfile::tempdir().unwrap();
    let mut welcome = welcome(directory.path());
    assert_eq!(welcome.finish_current(StepStatus::Reviewed), Some(true));
    assert_eq!(welcome.progress.status("language"), StepStatus::Reviewed);
    assert_eq!(steps()[welcome.selected].id, "displays");
    let progress = welcome.store.as_ref().unwrap().load().0;
    assert_eq!(progress.current_category, "displays");
    assert_eq!(progress.status("language"), StepStatus::Reviewed);
  }

  #[cfg(feature = "locale")]
  #[test]
  fn skipping_advances_without_a_check_and_save_failure_keeps_the_step_pending() {
    let directory = tempfile::tempdir().unwrap();
    let mut guide = welcome(directory.path());
    guide.finish_current(StepStatus::Skipped);
    assert_eq!(guide.progress.status("language"), StepStatus::Skipped);
    let path = directory.path().join("file");
    std::fs::write(&path, "not a directory").unwrap();
    let mut guide = welcome(&path);
    assert_eq!(guide.finish_current(StepStatus::Reviewed), None);
    assert_eq!(guide.selected, 0);
    assert_eq!(guide.progress.status("language"), StepStatus::Pending);
    assert!(guide.error);
  }

  #[test]
  fn home_entry_and_return_from_settings_stay_in_the_same_app() {
    let directory = tempfile::tempdir().unwrap();
    let mut app = App::new(crate::app::InitialRoute::Home);
    app.welcome = welcome(directory.path());
    app.open_destination(HomeDestination::Welcome);
    assert_eq!(app.route, Route::Welcome);
    app.route = Route::Settings;
    app.welcome.active = true;
    app.settings.navigation = argvus_control_center_settings::navigation::Navigation::new(
      argvus_control_center_settings::Page::Main,
    );
    crate::event::handle(
      &mut app,
      crossterm::event::Event::Key(crossterm::event::KeyEvent::from(KeyCode::Esc)),
    );
    assert_eq!(app.route, Route::Welcome);
    assert!(app.welcome.reviewing);
    assert_eq!(app.welcome.progress.status("language"), StepStatus::Pending);
    app.handle_welcome(KeyCode::Esc);
    assert_eq!(app.route, Route::Home);
    assert!(!app.welcome.active);
  }

  #[cfg(feature = "locale")]
  #[test]
  fn checklist_renders_translated_checks_and_records_first_open() {
    for (locale, width, height) in [("en-US", 100, 24), ("pt-BR", 60, 18)] {
      let directory = tempfile::tempdir().unwrap();
      let mut app = App::new(crate::app::InitialRoute::Welcome);
      app.lang = argvus_i18n::Lang::for_locale(locale);
      app.welcome = welcome(directory.path());
      app
        .welcome
        .progress
        .set_status("language", StepStatus::Reviewed);
      let mut terminal =
        ratatui::Terminal::new(ratatui::backend::TestBackend::new(width, height)).unwrap();
      terminal.draw(|frame| draw(&mut app, frame)).unwrap();
      let contents: String = terminal
        .backend()
        .buffer()
        .content
        .iter()
        .map(|cell| cell.symbol())
        .collect();
      assert!(contents.contains("[x]"));
      assert!(contents.contains(tr(app.lang, "control_center.welcome_title")));
      assert!(contents.contains(tr(app.lang, "control_center.card_tools")));
      assert!(!contents.contains("control_center."));
      assert!(app.welcome.store.as_ref().unwrap().load().0.opened);
    }
  }

  #[test]
  fn request_waits_for_home_without_discarding_the_current_page() {
    let directory = tempfile::tempdir().unwrap();
    let mut app = App::new(crate::app::InitialRoute::Home);
    app.welcome = welcome(directory.path());
    app.welcome.store.as_ref().unwrap().request_open().unwrap();
    app.route = Route::Config;
    assert!(!app.poll_welcome_request());
    assert_eq!(app.route, Route::Config);
    app.route = Route::Home;
    assert!(app.poll_welcome_request());
    assert_eq!(app.route, Route::Welcome);
    assert!(
      !app
        .welcome
        .store
        .as_ref()
        .unwrap()
        .take_open_request()
        .unwrap()
    );
  }

  #[test]
  fn cli_route_enters_the_integrated_page() {
    assert_eq!(
      crate::cli::parse(&["welcome".into()]).unwrap(),
      Some(crate::app::InitialRoute::Welcome)
    );
  }
}
