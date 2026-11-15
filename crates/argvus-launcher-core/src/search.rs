//! Pure ranking and typed activation targets. No user text is evaluated as shell.
use crate::settings::Settings;
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
  let query = normalize(query);
  let mut seen = std::collections::HashSet::new();
  let mut matches: Vec<_> = entries
    .iter()
    .filter(|entry| {
      settings.enabled(&entry.category)
        && (category.is_empty() || entry.category == category)
        && seen.insert(&entry.id)
    })
    .filter_map(|entry| {
      let favorite = settings.favorites.contains(&entry.id);
      let rank = if query.is_empty() {
        if category.is_empty() && entry.category != "applications" && !favorite {
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
      Some((rank, !favorite, priority, entry))
    })
    .collect();
  matches.sort_by(|a, b| {
    (a.0, a.1, a.2, &a.3.title, &a.3.id).cmp(&(b.0, b.1, b.2, &b.3.title, &b.3.id))
  });
  matches
    .into_iter()
    .take(100)
    .map(|(_, _, _, entry)| entry.clone())
    .collect()
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
}
