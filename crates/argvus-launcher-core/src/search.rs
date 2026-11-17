//! Pure ranking and typed activation targets. No user text is evaluated as shell.
use crate::settings::{ApplicationSort, Settings};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum Action {
  Application(String),
  Window(String),
  Settings(String),
  Project(String),
  Ssh(String),
  File(String),
  AudioOutputs,
  AudioOutput(String),
  Configure,
  Help,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
  pub id: String,
  pub title: String,
  #[serde(default)]
  pub description: String,
  pub category: String,
  #[serde(default)]
  pub icon: String,
  #[serde(default)]
  pub keywords: Vec<String>,
  #[serde(default)]
  pub is_active: bool,
  #[serde(default)]
  pub usage_id: String,
  pub action: Action,
}

pub fn normalize(text: &str) -> String {
  text
    .to_lowercase()
    .chars()
    .map(|ch| match ch {
      'á' | 'à' | 'ã' | 'â' | 'ä' => 'a',
      'é' | 'è' | 'ê' | 'ë' => 'e',
      'í' | 'ì' | 'î' | 'ï' => 'i',
      'ó' | 'ò' | 'õ' | 'ô' | 'ö' => 'o',
      'ú' | 'ù' | 'û' | 'ü' => 'u',
      'ç' => 'c',
      _ => ch,
    })
    .collect::<String>()
    .split_whitespace()
    .collect::<Vec<_>>()
    .join(" ")
}

fn score(entry: &Entry, query: &str) -> Option<u8> {
  let title = normalize(&entry.title);
  if title == query {
    return Some(0);
  }
  if title.starts_with(query) {
    return Some(1);
  }
  if title.split_whitespace().any(|word| word == query) {
    return Some(2);
  }
  if title.contains(query) {
    return Some(3);
  }
  let keywords = entry
    .keywords
    .iter()
    .map(|word| normalize(word))
    .collect::<Vec<_>>();
  if keywords.iter().any(|word| word == query) {
    return Some(4);
  }
  let text = format!(
    "{} {} {}",
    title,
    normalize(&entry.description),
    keywords.join(" ")
  );
  if query.split_whitespace().all(|word| text.contains(word)) {
    return Some(5);
  }
  // Subsequence matching is deliberately last: exact intent aliases win.
  let mut chars = title.chars();
  query
    .chars()
    .all(|needle| chars.by_ref().any(|ch| ch == needle))
    .then_some(6)
}

pub fn rank(entries: &[Entry], query: &str, category: &str, settings: &Settings) -> Vec<Entry> {
  rank_with_recent_order(entries, query, category, settings, &[])
}

pub(crate) fn rank_with_recent_order(
  entries: &[Entry],
  query: &str,
  category: &str,
  settings: &Settings,
  recent_order: &[String],
) -> Vec<Entry> {
  let query = normalize(query);
  let is_initial_query = query.is_empty();
  let mut seen = std::collections::HashSet::new();
  let mut matches: Vec<_> = entries
    .iter()
    .filter(|entry| {
      // Open now suggestions ignore the saved category only when explicitly enabled.
      let is_open_window = is_initial_query
        && settings.show_open_now
        && category != "windows"
        && entry.category == "windows";
      settings.enabled(&entry.category)
        && (category.is_empty() || entry.category == category || is_open_window)
        && (entry.category != "windows"
          || !is_initial_query
          || category == "windows"
          || is_open_window)
        && seen.insert(&entry.id)
    })
    .filter_map(|entry| {
      let favorite = settings.favorites.contains(&entry.id);
      let is_open_window = is_initial_query
        && settings.show_open_now
        && category != "windows"
        && entry.category == "windows";
      let rank = if query.is_empty() {
        if category.is_empty()
          && !matches!(entry.category.as_str(), "applications")
          && !is_open_window
          && !favorite
        {
          return None;
        }
        0
      } else {
        score(entry, &query)?
      };
      let priority = settings
        .sources
        .iter()
        .position(|source| source == &entry.category)
        .unwrap_or(usize::MAX);
      let window_priority = usize::from(
        is_initial_query
          && settings.show_open_now
          && category != "windows"
          && entry.category != "windows",
      );
      let active_priority = usize::from(
        !(is_initial_query
          && settings.show_open_now
          && entry.category == "windows"
          && entry.is_active),
      );
      let application_order = (entry.category == "applications").then(|| {
        let recent_position = match &entry.action {
          Action::Application(id) => recent_order
            .iter()
            .position(|recent_id| recent_id == id)
            .unwrap_or(usize::MAX),
          _ => usize::MAX,
        };
        let title = normalize(&entry.title);
        match settings.application_sort {
          ApplicationSort::Alphabetical => (0, title),
          ApplicationSort::Recent => (recent_position, title),
        }
      });
      Some(RankedEntry {
        rank,
        window_priority,
        active_priority,
        application_order,
        favorite,
        priority,
        title: normalize(&entry.title),
        entry,
      })
    })
    .collect();
  matches.sort_by(|a, b| {
    (a.rank, a.window_priority, a.active_priority)
      .cmp(&(b.rank, b.window_priority, b.active_priority))
      .then_with(|| a.priority.cmp(&b.priority))
      .then_with(|| a.application_order.cmp(&b.application_order))
      .then_with(|| {
        (!a.favorite, a.priority, &a.title, &a.entry.id).cmp(&(
          !b.favorite,
          b.priority,
          &b.title,
          &b.entry.id,
        ))
      })
  });
  matches
    .into_iter()
    .take(100)
    .map(|ranked| ranked.entry.clone())
    .collect()
}

struct RankedEntry<'a> {
  rank: u8,
  window_priority: usize,
  active_priority: usize,
  application_order: Option<(usize, String)>,
  favorite: bool,
  priority: usize,
  title: String,
  entry: &'a Entry,
}

#[cfg(test)]
mod tests {
  use super::*;
  fn entry(id: &str, title: &str) -> Entry {
    Entry {
      id: id.into(),
      title: title.into(),
      description: String::new(),
      category: "settings".into(),
      icon: String::new(),
      keywords: vec!["internet sem acesso".into()],
      is_active: false,
      usage_id: String::new(),
      action: Action::Configure,
    }
  }
  #[test]
  fn ranks_exact_before_intent_and_deduplicates() {
    let entries = vec![
      entry("network", "Rede"),
      entry("exact", "Internet sem acesso"),
      entry("network", "Rede"),
    ];
    let results = rank(&entries, "INTERNET SEM ACESSO", "", &Settings::default());
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].id, "exact");
  }
  #[test]
  fn accents_fuzzy_and_disabled_sources() {
    let entries = vec![entry("audio", "Áudio")];
    assert_eq!(rank(&entries, "audio", "", &Settings::default()).len(), 1);
    assert_eq!(rank(&entries, "ado", "", &Settings::default()).len(), 1);
    assert!(rank(&entries, "audio", "applications", &Settings::default()).is_empty());
    let settings = Settings {
      sources: vec![],
      ..Settings::default()
    };
    assert!(rank(&entries, "audio", "", &settings).is_empty());
  }

  #[test]
  fn open_now_is_disabled_by_default_and_can_be_enabled() {
    let mut active_window = entry("window:1", "Editor: notes.md");
    active_window.category = "windows".into();
    active_window.is_active = true;
    active_window.action = Action::Window("1".into());

    let mut other_window = entry("window:2", "Browser: docs");
    other_window.category = "windows".into();
    other_window.action = Action::Window("2".into());

    let mut last_window = entry("window:3", "Mail: inbox");
    last_window.category = "windows".into();
    last_window.action = Action::Window("3".into());

    let mut application = entry("app:editor", "Editor");
    application.category = "applications".into();
    application.action = Action::Application("editor.desktop".into());

    let mut settings = Settings::default();
    let default_results = rank(
      &[
        application.clone(),
        last_window.clone(),
        other_window.clone(),
        active_window.clone(),
      ],
      "",
      "applications",
      &settings,
    );
    assert_eq!(default_results.len(), 1);
    assert_eq!(default_results[0].id, "app:editor");
    settings.show_open_now = true;
    let results = rank(
      &[application, last_window, other_window, active_window],
      "",
      "applications",
      &settings,
    );

    assert_eq!(
      results
        .iter()
        .map(|entry| entry.id.as_str())
        .collect::<Vec<_>>(),
      ["window:1", "window:2", "window:3", "app:editor"]
    );
  }

  #[test]
  fn initial_all_category_hides_windows_when_open_now_is_disabled() {
    let mut window = entry("window:1", "Browser: docs");
    window.category = "windows".into();
    window.action = Action::Window("1".into());
    let mut application = entry("app:browser", "Browser");
    application.category = "applications".into();
    application.action = Action::Application("browser.desktop".into());

    let results = rank(&[application, window], "", "", &Settings::default());

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "app:browser");
  }

  #[test]
  fn typed_search_respects_the_selected_category() {
    let mut window = entry("window:1", "Editor");
    window.category = "windows".into();
    window.action = Action::Window("1".into());
    let mut application = entry("app:editor", "Editor");
    application.category = "applications".into();
    application.action = Action::Application("editor.desktop".into());

    let results = rank(
      &[window, application],
      "editor",
      "applications",
      &Settings::default(),
    );

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "app:editor");
  }

  #[test]
  fn disabled_windows_source_hides_open_windows_from_initial_results() {
    let mut window = entry("window:1", "Editor");
    window.category = "windows".into();
    window.action = Action::Window("1".into());
    let settings = Settings {
      sources: vec!["applications".into()],
      ..Settings::default()
    };

    assert!(rank(&[window], "", "applications", &settings).is_empty());
  }

  #[test]
  fn application_order_supports_recent_and_alphabetical_modes() {
    let mut browser = entry("app:browser", "Browser");
    browser.category = "applications".into();
    browser.keywords.push("suite".into());
    browser.action = Action::Application("browser.desktop".into());
    let mut editor = entry("app:editor", "Editor");
    editor.category = "applications".into();
    editor.keywords.push("suite".into());
    editor.action = Action::Application("editor.desktop".into());
    let recent_order = vec!["editor.desktop".into(), "browser.desktop".into()];
    let results = rank_with_recent_order(
      &[editor.clone(), browser.clone()],
      "",
      "applications",
      &Settings::default(),
      &recent_order,
    );
    assert_eq!(results[0].id, "app:editor");
    let query_results = rank_with_recent_order(
      &[editor.clone(), browser.clone()],
      "suite",
      "applications",
      &Settings::default(),
      &recent_order,
    );
    assert_eq!(query_results[0].id, "app:editor");
    let alphabetical = Settings {
      application_sort: ApplicationSort::Alphabetical,
      ..Settings::default()
    };
    let alphabetical_results = rank_with_recent_order(
      &[editor, browser],
      "",
      "applications",
      &alphabetical,
      &recent_order,
    );
    assert_eq!(alphabetical_results[0].id, "app:browser");
  }

  #[test]
  fn all_search_includes_windows_and_keeps_category_order_stable() {
    let mut window = entry("window:1", "Editor window");
    window.category = "windows".into();
    window.action = Action::Window("1".into());
    let mut application = entry("app:editor", "Editor application");
    application.category = "applications".into();
    application.action = Action::Application("editor.desktop".into());
    let results = rank(&[window, application], "editor", "", &Settings::default());
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].category, "applications");
    assert_eq!(results[1].category, "windows");
  }
}
