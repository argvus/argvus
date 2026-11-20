//! Internal GTK picker protocol for specialized launcher scripts.

use std::{ffi::OsString, io, process::ExitCode};

use argvus_i18n::{Lang, tr};
use argvus_theme_selector::picker::{self, Choice, SearchPicker};

use crate::exec::{Failure, Outcome};

pub fn run(arguments: &[OsString]) -> Outcome {
  let locale = argvus_i18n::user_locale();
  let lang = Lang::for_domain(&locale, "launcher");
  let t = |key| tr(lang, key);
  let (namespace, title, icon, prompt, empty) = match arguments {
    [kind] if kind == "ssh" => (
      "argvus-ssh",
      "SSH",
      "network-server-symbolic",
      "search.ssh",
      "error.no_ssh_hosts",
    ),
    [kind] if kind == "snippets" => (
      "argvus-snippets",
      t("picker.snippets"),
      "text-x-generic-symbolic",
      "search.snippet",
      "error.no_snippets",
    ),
    _ => return Err(Failure::new("usage: argvus launcher-picker ssh|snippets")),
  };

  let names: Vec<String> = serde_json::from_reader(io::stdin().lock())
    .map_err(|error| Failure::new(format!("invalid picker input: {error}")))?;
  let rows: Vec<_> = names
    .iter()
    .map(|name| Choice {
      swatch: None,
      icon,
      title: name,
      detail: "",
      shortcut: "",
    })
    .collect();
  let options = SearchPicker {
    namespace,
    title,
    prompt: t(prompt),
    empty: t(empty),
    no_results: t("picker.no_results"),
    activate: t(prompt),
  };
  if let Some(index) = picker::choose(&locale, lang.direction(), options, &rows)
    .map_err(|error| Failure::new(format!("GTK: {error}")))?
  {
    println!("{index}");
  }
  Ok(ExitCode::SUCCESS)
}
