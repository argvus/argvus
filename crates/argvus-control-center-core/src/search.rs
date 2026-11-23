//! Search interaction.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct SearchEntry {
  pub id: String,
  pub category: String,
  pub title: String,
  pub keywords: Vec<String>,
  pub route: String,
}

#[derive(Debug, Default, Clone)]
pub struct SearchRegistry {
  entries: Vec<SearchEntry>,
}

impl SearchRegistry {
  pub fn register(&mut self, entry: SearchEntry) -> Result<(), String> {
    if entry.id.trim().is_empty() || self.entries.iter().any(|current| current.id == entry.id) {
      return Err("search entry id must be unique and non-empty".into());
    }
    self.entries.push(entry);
    Ok(())
  }
  pub fn entry_for_route_mut(&mut self, route: &str) -> Option<&mut SearchEntry> {
    self.entries.iter_mut().find(|entry| entry.route == route)
  }

  pub fn remove_account_entries(&mut self) {
    self
      .entries
      .retain(|entry| !entry.id.starts_with("account:"));
  }

  pub fn entries(&self) -> &[SearchEntry] {
    &self.entries
  }
  pub fn search(&self, query: &str) -> Vec<&SearchEntry> {
    let query = normalize(query);
    if query.is_empty() {
      return Vec::new();
    }
    let mut matches: Vec<(u8, usize, &SearchEntry)> = self
      .entries
      .iter()
      .enumerate()
      .filter_map(|(index, entry)| {
        let rank = match_rank(&query, entry);
        rank.map(|rank| (rank, index, entry))
      })
      .collect();
    matches.sort_by_key(|(rank, index, _)| (*rank, *index));
    matches.into_iter().map(|(_, _, entry)| entry).collect()
  }
}

fn normalize(value: &str) -> String {
  value
    .trim()
    .to_lowercase()
    .chars()
    .map(|character| match character {
      'á' | 'à' | 'ã' | 'â' | 'ä' | 'Á' | 'À' | 'Ã' | 'Â' | 'Ä' => 'a',
      'é' | 'è' | 'ê' | 'ë' | 'É' | 'È' | 'Ê' | 'Ë' => 'e',
      'í' | 'ì' | 'î' | 'ï' | 'Í' | 'Ì' | 'Î' | 'Ï' => 'i',
      'ó' | 'ò' | 'õ' | 'ô' | 'ö' | 'Ó' | 'Ò' | 'Õ' | 'Ô' | 'Ö' => 'o',
      'ú' | 'ù' | 'û' | 'ü' | 'Ú' | 'Ù' | 'Û' | 'Ü' => 'u',
      'ç' | 'Ç' => 'c',
      other => other.to_ascii_lowercase(),
    })
    .collect()
}

fn match_rank(query: &str, entry: &SearchEntry) -> Option<u8> {
  let title = normalize(&entry.title);
  let category = normalize(&entry.category);
  if title == query {
    return Some(0);
  }
  if title.starts_with(query) {
    return Some(1);
  }
  if title.split_whitespace().any(|word| word.starts_with(query)) {
    return Some(2);
  }
  if title.contains(query) {
    return Some(3);
  }
  if title.split_whitespace().any(|word| word.contains(query))
    || category.split_whitespace().any(|word| word.contains(query))
  {
    return Some(4);
  }
  let keywords = entry.keywords.iter().map(|keyword| normalize(keyword));
  if keywords.clone().any(|keyword| keyword == query) {
    return Some(5);
  }
  if entry
    .keywords
    .iter()
    .map(|keyword| normalize(keyword))
    .any(|keyword| keyword.starts_with(query))
  {
    return Some(6);
  }
  if entry
    .keywords
    .iter()
    .map(|keyword| normalize(keyword))
    .any(|keyword| keyword.contains(query))
  {
    return Some(7);
  }
  if category.contains(query) || normalize(&entry.id).contains(query) {
    return Some(8);
  }
  if query.split_whitespace().count() > 1
    && query
      .split_whitespace()
      .all(|word| match_rank(word, entry).is_some())
  {
    return Some(9);
  }
  None
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn finds_by_keyword_and_rejects_duplicate_ids() {
    let mut registry = SearchRegistry::default();
    let entry = SearchEntry {
      id: "network.wifi".into(),
      category: "network".into(),
      title: "Wi-Fi".into(),
      keywords: vec!["wireless".into()],
      route: "network/wifi".into(),
    };
    registry.register(entry.clone()).unwrap();
    assert_eq!(registry.search("wireless").len(), 1);
    assert!(registry.register(entry).is_err());
  }

  #[test]
  fn finds_each_account_by_field_label_and_combines_path_title_and_content() {
    let mut registry = SearchRegistry::default();
    for name in ["ana", "bea"] {
      registry
        .register(SearchEntry {
          id: format!("account:{name}"),
          category: "System Administration > Users & Groups > Users".into(),
          title: name.into(),
          keywords: vec!["Shell".into()],
          route: format!("account/{name}"),
        })
        .unwrap();
    }
    for query in ["shell", "Shell", "SHELL", "users shell"] {
      let titles: Vec<_> = registry
        .search(query)
        .into_iter()
        .map(|entry| entry.title.as_str())
        .collect();
      assert_eq!(titles, ["ana", "bea"]);
    }
    assert_eq!(registry.search("users shell ANA")[0].title, "ana");
  }

  #[test]
  fn ranks_exact_title_before_keyword_and_empty_query_is_empty() {
    let mut registry = SearchRegistry::default();
    registry
      .register(SearchEntry {
        id: "audio".into(),
        category: "audio".into(),
        title: "Audio".into(),
        keywords: vec!["sound".into()],
        route: "audio/summary".into(),
      })
      .unwrap();
    registry
      .register(SearchEntry {
        id: "audio.output".into(),
        category: "audio".into(),
        title: "Output".into(),
        keywords: vec!["audio".into()],
        route: "audio/output".into(),
      })
      .unwrap();
    assert_eq!(registry.search("audio")[0].title, "Audio");
    assert!(registry.search(" ").is_empty());
    assert_eq!(registry.search("ÁUDIO")[0].title, "Audio");
  }
}
