//! Keyboard-first launcher for the optional ARGVUS AI integration.
mod view;

mod style;

use argvus_i18n::I18n;
use glib::Unichar;
use gtk::{gdk::Key as KeyCode, glib};
use std::{
  path::PathBuf,
  process::{Command, ExitCode},
};

const CATALOG: [(&str, &str, &str); 9] = [
  ("opencode", "OpenCode", "opencode.svg"),
  ("codex", "OpenAI Codex", "codex.png"),
  ("agy", "Google Antigravity", "antigravity.svg"),
  ("qwen", "Qwen Code", "qwen.svg"),
  ("gemini", "Gemini CLI", "gemini.svg"),
  ("claude", "Claude Code", "claude.svg"),
  ("hermes", "Hermes", "hermes.svg"),
  ("cursor", "Cursor CLI", "cursor.svg"),
  ("kimi", "Kimi Code", "kimi.svg"),
];

struct Agent {
  id: &'static str,
  name: &'static str,
  description: String,
  searchable: String,
  icon: PathBuf,
}

fn search_key(text: &str) -> String {
  glib::normalize(glib::casefold(text), glib::NormalizeMode::Default)
    .chars()
    .filter(|character| character.combining_class() == 0)
    .collect()
}

fn matching(agents: &[Agent], query: &str) -> Vec<usize> {
  let query = search_key(query);
  agents
    .iter()
    .enumerate()
    .filter_map(|(index, agent)| {
      query
        .split_whitespace()
        .all(|word| agent.searchable.contains(word))
        .then_some(index)
    })
    .collect()
}

fn move_selection(selected: usize, count: usize, columns: usize, key: KeyCode) -> usize {
  if count == 0 {
    return 0;
  }
  match key {
    KeyCode::Left if !selected.is_multiple_of(columns) => selected - 1,
    KeyCode::Right if selected % columns + 1 < columns => (selected + 1).min(count - 1),
    KeyCode::Up if selected >= columns => selected - columns,
    KeyCode::Down if selected / columns < (count - 1) / columns => {
      (selected + columns).min(count - 1)
    }
    _ => selected,
  }
}

fn main() -> ExitCode {
  let arguments: Vec<_> = std::env::args().skip(1).collect();
  let [flag, ids @ ..] = arguments.as_slice() else {
    return ExitCode::from(2);
  };
  if flag != "--selector"
    || ids
      .iter()
      .any(|id| !CATALOG.iter().any(|entry| entry.0 == id))
  {
    return ExitCode::from(2);
  }
  let i18n = match I18n::new("ai") {
    Ok(i18n) => i18n,
    Err(error) => {
      eprintln!("{error}");
      return ExitCode::FAILURE;
    }
  };
  let root = std::env::var_os("ARGVUS_SYSTEM_CONFIG")
    .map_or_else(|| PathBuf::from("/usr/share/argvus"), PathBuf::from)
    .join("ai-agents");
  let agents = CATALOG
    .into_iter()
    .filter(|entry| ids.iter().any(|id| id == entry.0))
    .map(|(id, name, icon)| {
      let description = i18n.tr(&format!("selector.description.{id}"));
      let keywords = i18n.tr(&format!("selector.keywords.{id}"));
      let searchable = search_key(&format!("{name} {description} {keywords}"));
      Agent {
        id,
        name,
        description,
        searchable,
        icon: root.join(icon),
      }
    })
    .collect();
  match view::select(agents, &i18n) {
    Ok(Some(id)) => match Command::new("argvus-ai").args(["launch", id]).spawn() {
      Ok(_) => ExitCode::SUCCESS,
      Err(error) => {
        eprintln!("{} ({error})", i18n.tr("terminal_error"));
        ExitCode::FAILURE
      }
    },
    Ok(None) => ExitCode::SUCCESS,
    Err(code) => code,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn searches_all_terms_without_accents_or_case() {
    let agent = Agent {
      id: "codex",
      name: "Codex",
      description: String::new(),
      searchable: search_key("Codex automação código"),
      icon: PathBuf::new(),
    };
    assert_eq!(matching(&[agent], "CÓDEX automacao"), [0]);
    assert!(matching(&[], "codex").is_empty());
  }

  #[test]
  fn down_clamps_to_the_last_card_in_an_incomplete_row() {
    assert_eq!(move_selection(2, 5, 3, KeyCode::Down), 4);
    assert_eq!(move_selection(4, 5, 3, KeyCode::Right), 4);
    assert_eq!(move_selection(4, 5, 3, KeyCode::Up), 1);
  }
}
