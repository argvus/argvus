//! Launcher preferences: asynchronous canonical persistence and explicit file roots.
use argvus_i18n::{Lang, tr};
use argvus_launcher_core::settings::{ApplicationSort, SOURCES, Settings};
use argvus_theme::Theme;
use argvus_tui::page::{list, shell};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
  Frame,
  layout::{Constraint, Layout},
  widgets::Paragraph,
};
use std::sync::mpsc::{self, Receiver};

#[derive(Clone)]
enum Row {
  AppearanceHeading,
  Source(String),
  ApplicationOrder,
  OpenNow,
  Recent,
  Index,
  Pause,
  Hidden,
  Motion,
  Icons,
  Root(usize),
  Exclude(usize),
  Favorite(usize),
  AddRoot,
  AddExclude,
  Rebuild,
  Clear,
  ClearUsage,
}
#[derive(Clone, Copy)]
enum Operation {
  Save,
  RebuildIndex,
  ClearUsage,
}
enum Work {
  Loaded(Settings, (usize, u64)),
  Failed(String),
}

#[derive(Default)]
pub struct LauncherPage {
  settings: Option<Settings>,
  selected: usize,
  editing: Option<(bool, String)>,
  receiver: Option<Receiver<Work>>,
  error: bool,
  index: (usize, u64),
}

impl LauncherPage {
  pub fn open(&mut self) {
    self.selected = 0;
    self.editing = None;
    self.start(None, Operation::Save);
  }
  fn start(&mut self, settings: Option<Settings>, operation: Operation) {
    if self.receiver.is_some() {
      return;
    }
    let previous = self.settings.clone();
    let (sender, receiver) = mpsc::channel();
    self.receiver = Some(receiver);
    self.error = false;
    std::thread::spawn(move || {
      let result = (|| {
        if let Some(settings) = settings {
          if let Some(previous) = previous {
            settings.save_changes(&previous)?;
          } else {
            settings.save()?;
          }
        }
        match operation {
          Operation::Save => {}
          Operation::RebuildIndex => argvus_launcher_core::rebuild_index()?,
          Operation::ClearUsage => argvus_launcher_core::clear_usage_history()?,
        }
        Ok::<_, String>((Settings::load()?, argvus_launcher_core::index_status()))
      })();
      let work = match result {
        Ok((settings, index)) => Work::Loaded(settings, index),
        Err(error) => Work::Failed(error),
      };
      let _ = sender.send(work);
    });
  }
  pub fn poll(&mut self) -> bool {
    let Some(receiver) = &self.receiver else {
      return false;
    };
    match receiver.try_recv() {
      Ok(work) => {
        self.receiver = None;
        match work {
          Work::Loaded(settings, index) => {
            self.settings = Some(settings);
            self.index = index;
          }
          Work::Failed(error) => {
            eprintln!("launcher settings: {error}");
            self.error = true;
          }
        }
        true
      }
      Err(mpsc::TryRecvError::Disconnected) => {
        self.receiver = None;
        self.error = true;
        true
      }
      _ => false,
    }
  }
  pub fn busy(&self) -> bool {
    self.receiver.is_some()
  }

  pub fn paste(&mut self, text: &str) {
    if let Some((_, input)) = &mut self.editing {
      input.extend(text.chars().filter(|character| !character.is_control()));
    }
  }

  pub fn captures_text(&self) -> bool {
    self.editing.is_some()
  }
  fn rows(&self, lang: Lang) -> Vec<(Row, String)> {
    let Some(settings) = &self.settings else {
      return Vec::new();
    };
    let text = |key: &str| tr(lang, key).to_string();
    let mut ordered = settings.sources.clone();
    for source in SOURCES {
      if !ordered.iter().any(|s| s == source) {
        ordered.push(source.into());
      }
    }
    let mut rows: Vec<_> = ordered
      .into_iter()
      .map(|source| {
        let title = format!(
          "[{}] {}",
          if settings.enabled(&source) { "x" } else { " " },
          tr(lang, &format!("control_center.launcher_source_{source}"))
        );
        (Row::Source(source), title)
      })
      .collect();
    let application_sort = match settings.application_sort {
      ApplicationSort::Alphabetical => "control_center.launcher_sort_alphabetical",
      ApplicationSort::Recent => "control_center.launcher_sort_recent",
    };
    rows.push((
      Row::ApplicationOrder,
      format!(
        "{}: {}",
        tr(lang, "control_center.launcher_application_order"),
        tr(lang, application_sort)
      ),
    ));
    rows.push((
      Row::OpenNow,
      format!(
        "[{}] {}",
        if settings.show_open_now { "x" } else { " " },
        tr(lang, "control_center.launcher_open_now")
      ),
    ));
    for (row, key, enabled) in [
      (Row::Recent, "recent", settings.recent_files),
      (Row::Index, "index", settings.index_enabled),
      (Row::Pause, "pause", settings.index_paused),
      (Row::Hidden, "hidden", settings.include_hidden),
      (Row::Motion, "motion", settings.reduced_motion),
      (Row::Icons, "icons", settings.show_icons),
    ] {
      if matches!(&row, Row::Motion) {
        rows.push((
          Row::AppearanceHeading,
          tr(lang, "control_center.appearance").into(),
        ));
      }
      rows.push((
        row,
        format!(
          "[{}] {}",
          if enabled { "x" } else { " " },
          tr(lang, &format!("control_center.launcher_{key}"))
        ),
      ));
    }
    for (index, path) in settings.roots.iter().enumerate() {
      rows.push((
        Row::Root(index),
        format!(
          "{}: {}",
          text("control_center.launcher_root"),
          path.display()
        ),
      ));
    }
    rows.push((Row::AddRoot, text("control_center.launcher_add_root")));
    for (index, path) in settings.exclusions.iter().enumerate() {
      rows.push((
        Row::Exclude(index),
        format!(
          "{}: {}",
          text("control_center.launcher_exclude"),
          path.display()
        ),
      ));
    }
    rows.push((Row::AddExclude, text("control_center.launcher_add_exclude")));
    for (index, id) in settings.favorites.iter().enumerate() {
      rows.push((
        Row::Favorite(index),
        format!("{}: {}", text("control_center.launcher_favorite"), id),
      ));
    }
    rows.push((Row::Rebuild, text("control_center.launcher_rebuild")));
    rows.push((Row::Clear, text("control_center.launcher_clear")));
    rows.push((Row::ClearUsage, text("control_center.launcher_clear_usage")));
    rows
  }
  pub fn handle(&mut self, key: KeyEvent, lang: Lang) -> bool {
    if let Some((exclude, input)) = &mut self.editing {
      match key.code {
        KeyCode::Esc => {
          self.editing = None;
        }
        KeyCode::Backspace => {
          input.pop();
        }
        KeyCode::Char(ch) => input.push(ch),
        KeyCode::Enter => {
          let path = std::path::PathBuf::from(input.trim());
          if !path.is_absolute() {
            self.error = true;
            return false;
          }
          let Some(mut settings) = self.settings.clone() else {
            return false;
          };
          let paths = if *exclude {
            &mut settings.exclusions
          } else {
            &mut settings.roots
          };
          if !paths.contains(&path) {
            paths.push(path);
          }
          self.editing = None;
          self.start(Some(settings), Operation::Save);
        }
        _ => {}
      }
      return false;
    }
    if key.code == KeyCode::Esc {
      return true;
    }
    if self.receiver.is_some() {
      return false;
    }
    let rows: Vec<_> = self
      .rows(lang)
      .into_iter()
      .filter(|(row, _)| !matches!(row, Row::AppearanceHeading))
      .collect();
    let Some((row, _)) = rows.get(self.selected).cloned() else {
      if key.code == KeyCode::Enter {
        self.open();
      }
      return false;
    };
    let Some(mut settings) = self.settings.clone() else {
      return false;
    };
    if key.modifiers.contains(KeyModifiers::CONTROL)
      && matches!(key.code, KeyCode::Up | KeyCode::Down)
    {
      if let Row::Source(source) = row
        && let Some(index) = settings.sources.iter().position(|s| s == &source)
      {
        let next = if key.code == KeyCode::Up {
          index.saturating_sub(1)
        } else {
          (index + 1).min(settings.sources.len() - 1)
        };
        settings.sources.swap(index, next);
        self.selected = next;
        self.start(Some(settings), Operation::Save);
      }
      return false;
    }
    match key.code {
      KeyCode::Up => self.selected = self.selected.saturating_sub(1),
      KeyCode::Down => self.selected = (self.selected + 1).min(rows.len().saturating_sub(1)),
      KeyCode::Enter | KeyCode::Char(' ') => {
        match row {
          Row::Source(source) => {
            if settings.enabled(&source) {
              settings.sources.retain(|s| s != &source);
            } else {
              settings.sources.push(source);
            }
          }
          Row::Recent => settings.recent_files = !settings.recent_files,
          Row::Index => settings.index_enabled = !settings.index_enabled,
          Row::Pause => settings.index_paused = !settings.index_paused,
          Row::Hidden => settings.include_hidden = !settings.include_hidden,
          Row::Motion => settings.reduced_motion = !settings.reduced_motion,
          Row::Icons => settings.show_icons = !settings.show_icons,
          Row::OpenNow => settings.show_open_now = !settings.show_open_now,
          Row::ApplicationOrder => {
            settings.application_sort = match settings.application_sort {
              ApplicationSort::Alphabetical => ApplicationSort::Recent,
              ApplicationSort::Recent => ApplicationSort::Alphabetical,
            };
          }
          Row::AddRoot => {
            self.editing = Some((false, String::new()));
            return false;
          }
          Row::AddExclude => {
            self.editing = Some((true, String::new()));
            return false;
          }
          Row::Rebuild => {
            settings.index_enabled = true;
            settings.index_paused = false;
            self.start(Some(settings), Operation::RebuildIndex);
            return false;
          }
          Row::Clear => {
            settings.index_paused = true;
            self.start(Some(settings), Operation::RebuildIndex);
            return false;
          }
          Row::ClearUsage => {
            self.start(None, Operation::ClearUsage);
            return false;
          }
          _ => return false,
        }
        self.start(Some(settings), Operation::Save);
      }
      KeyCode::Delete => {
        match row {
          Row::Root(i) => {
            settings.roots.remove(i);
          }
          Row::Exclude(i) => {
            settings.exclusions.remove(i);
          }
          Row::Favorite(i) => {
            settings.favorites.remove(i);
          }
          _ => return false,
        }
        self.selected = self.selected.saturating_sub(1);
        self.start(Some(settings), Operation::Save);
      }
      KeyCode::F(5) => self.open(),
      _ => {}
    }
    false
  }
  pub fn breadcrumb(lang: Lang) -> String {
    format!(
      "{} > {}",
      argvus_control_center_core::navigation::HomeCardId::Preferences.label(lang),
      tr(lang, "control_center.launcher_search")
    )
  }
  pub fn draw(&self, frame: &mut Frame, lang: Lang, theme: &Theme) {
    self.draw_in(frame, lang, theme, frame.area());
  }

  pub fn draw_in(&self, frame: &mut Frame, lang: Lang, theme: &Theme, area: ratatui::layout::Rect) {
    let breadcrumb = Self::breadcrumb(lang);
    let body = shell(
      frame,
      area,
      theme,
      lang,
      &breadcrumb,
      tr(lang, "control_center.launcher_hints"),
    );
    let areas = Layout::vertical([Constraint::Min(1), Constraint::Length(2)]).split(body);
    let model = self.rows(lang);
    let visual_selected = model
      .iter()
      .enumerate()
      .filter(|(_, (row, _))| !matches!(row, Row::AppearanceHeading))
      .nth(self.selected)
      .map_or(0, |(index, _)| index);
    let rows: Vec<_> = model.into_iter().map(|(_, title)| title).collect();
    list(frame, areas[0], theme, &rows, visual_selected);
    let status = if let Some((_, input)) = &self.editing {
      format!("{}: {}", tr(lang, "control_center.launcher_path"), input)
    } else if self.error {
      tr(lang, "control_center.launcher_error").into()
    } else if self.receiver.is_some() {
      tr(lang, "control_center.loading").into()
    } else {
      format!(
        "{}: {}",
        tr(lang, "control_center.launcher_index_count"),
        self.index.0
      )
    };
    frame.render_widget(Paragraph::new(status), areas[1]);
  }
}

impl crate::app::App {
  /// Dispatches navigation requested while the existing window is focused.
  pub fn poll_launcher_request(&mut self) -> bool {
    let Some(directory) = std::env::var_os("XDG_RUNTIME_DIR") else {
      return false;
    };
    let path = std::path::PathBuf::from(directory).join("argvus-control-center-search.json");
    let Ok(bytes) = std::fs::read(&path) else {
      return false;
    };
    if std::fs::remove_file(&path).is_err() {
      return false;
    }
    let Ok(arguments) = serde_json::from_slice::<Vec<String>>(&bytes) else {
      return false;
    };
    if arguments.as_slice() == ["launcher"] {
      self.open_launcher();
      return true;
    }
    if arguments.len() == 2 && arguments[0] == "--search-id" {
      let route = self
        .search_registry
        .entries()
        .iter()
        .find(|entry| entry.id == arguments[1])
        .map(|entry| entry.route.clone());
      if let Some(route) = route {
        self.open_search_route(&route);
        return true;
      }
    }
    false
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn breadcrumb_and_cli_match_preferences_destination() {
    let lang = Lang::for_locale("en-US");
    assert_eq!(
      LauncherPage::breadcrumb(lang),
      "Preferences > Launcher and Search"
    );
    assert_eq!(
      crate::cli::parse(&["launcher".into()]).unwrap(),
      Some(crate::app::InitialRoute::Launcher)
    );
    let rows = crate::home::catalog::home_rows(lang, crate::home::HomeAvailability::default());
    assert!(rows.iter().any(|row| matches!(
      row,
      crate::home::HomeRow::Item {
        destination: crate::home::HomeDestination::Launcher,
        ..
      }
    )));
  }

  #[test]
  fn launcher_search_preferences_include_open_now_and_application_order() {
    let page = LauncherPage {
      settings: Some(Settings::default()),
      ..LauncherPage::default()
    };

    let rows = page.rows(Lang::for_locale("en-US"));

    assert!(
      rows
        .iter()
        .any(|(row, label)| { matches!(row, Row::OpenNow) && label.contains("Open now") })
    );
    assert!(rows.iter().any(|(row, label)| {
      matches!(row, Row::ApplicationOrder) && label.contains("Recently opened")
    }));
  }
}
