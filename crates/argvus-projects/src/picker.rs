//! Project labels and their mapping to the shared picker.

use std::path::Path;

use argvus_i18n::{Lang, tr};
use argvus_theme_selector::picker::{self, Choice, SearchPicker};

pub fn choose(projects: &[String], prompt: &str) -> Result<Option<String>, gtk::glib::BoolError> {
  let locale = argvus_i18n::user_locale();
  let lang = Lang::for_domain(&locale, "launcher");
  let t = |key| tr(lang, key);
  let shortcuts: Vec<_> = (1..=projects.len().min(9))
    .map(|slot| format!("Super + Ctrl + {slot}"))
    .collect();
  let rows: Vec<_> = projects
    .iter()
    .enumerate()
    .map(|(index, path)| Choice {
      swatch: None,
      icon: "folder-symbolic",
      title: Path::new(path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(path),
      detail: path,
      shortcut: shortcuts.get(index).map_or("", String::as_str),
    })
    .collect();
  let options = SearchPicker {
    namespace: "argvus-projects",
    title: t("project.picker_title"),
    prompt: if prompt.is_empty() {
      t("project.picker_search")
    } else {
      prompt
    },
    empty: t("project.picker_empty"),
    no_results: t("project.picker_no_results"),
    activate: t("search.project"),
  };
  picker::choose(&locale, lang.direction(), options, &rows)
    .map(|index| index.and_then(|index| projects.get(index).cloned()))
}
