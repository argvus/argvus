//! The highlight color command, `argvus config accent`.
//!
//! The rules are in `accent`. This module is the entry point the callers use:
//! it resolves the icon theme, applies the color, and tells the user what
//! happened, through argvus-i18n and a desktop notification. With no argument
//! it opens the highlight picker first.
//!
//! Exit codes: 0 on success or when the picker is closed, 1 when the color is
//! invalid, the theme has no default accent, or the state cannot be written.

use crate::ConfigStore;
use crate::accent;
use crate::i18n::{self, Stream};
use crate::icons::{self, IconNames};
use crate::session_paths::SessionPaths;
use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, ExitCode, Stdio};

/// The picker entries in menu order: the translation key of each name and its
/// color.
const CHOICES: [(&str, &str); 9] = [
  ("accent.color.blue", "#3590bd"),
  ("accent.color.slate_blue", "#7391a5"),
  ("accent.color.brown", "#996548"),
  ("accent.color.green", "#17d174"),
  ("accent.color.magenta", "#cb17d1"),
  ("accent.color.red", "#d1174f"),
  ("accent.color.yellow", "#d1ce17"),
  ("accent.color.purple", "#9617d1"),
  ("accent.color.silver", "#595959"),
];

/// Why a change did not apply.
enum Failure {
  /// The requested value is not a `#RRGGBB` color.
  Invalid(String),
  /// The theme has no default accent; carries the theme id.
  DefaultMissing(String),
  /// The accent state could not be written; carries the error text.
  State(String),
}

/// What an applied change reports.
struct Applied {
  color: String,
  theme: String,
  notify: bool,
}

/// Runs `argvus config accent [COLOR|--apply|--startup|--apply-static|--theme-default]`.
/// With no argument the picker chooses the color.
pub fn run(arguments: &[String]) -> ExitCode {
  let Some(paths) = SessionPaths::from_environment() else {
    eprintln!("argvus-config: cannot resolve the configuration directory");
    return ExitCode::from(1);
  };
  let requested = match arguments.first() {
    Some(first) => first.clone(),
    None => match pick(&paths) {
      Some(choice) => choice,
      None => return ExitCode::SUCCESS,
    },
  };
  // The icon theme is resolved for the active theme. Its warnings are not
  // repeated on every accent change, as the shell did.
  let icons = icons::resolve(&paths, "", Stdio::null());
  match apply(&paths, &requested, &icons) {
    Ok(applied) => {
      if applied.notify {
        notify(&applied.color);
      }
      i18n::print(
        "accent.applied",
        &[
          format!("color={}", applied.color),
          format!("theme={}", applied.theme),
        ],
        Stream::Stdout,
      );
      ExitCode::SUCCESS
    }
    Err(Failure::Invalid(value)) => {
      i18n::print(
        "accent.invalid",
        &[format!("value={value}")],
        Stream::Stderr,
      );
      ExitCode::from(1)
    }
    Err(Failure::DefaultMissing(theme)) => {
      i18n::print(
        "accent.default_missing",
        &[format!("theme={theme}")],
        Stream::Stderr,
      );
      ExitCode::from(1)
    }
    Err(Failure::State(error)) => {
      eprintln!("argvus-config: accent state: {error}");
      ExitCode::from(1)
    }
  }
}

/// Opens the highlight picker and returns the chosen color. The rofi line ends
/// with the color, so the last field is the value. `None` when the picker was
/// closed or cannot run.
fn pick(paths: &SessionPaths) -> Option<String> {
  let entries: Vec<String> = CHOICES
    .iter()
    .enumerate()
    .map(|(index, (key, color))| {
      let label = i18n::text(key, &[]);
      format!("{:02} - {label} {color}", index + 1)
    })
    .collect();
  let mut child = Command::new("rofi")
    .arg("-config")
    .arg(paths.read_config("launcher/config/config.rasi"))
    .args(["-dmenu", "-i", "-p"])
    .arg(i18n::text("accent.title", &[]))
    .args(["-theme-str", "listview {lines: 9;}"])
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .spawn()
    .ok()?;
  if let Some(mut stdin) = child.stdin.take() {
    let _ = stdin.write_all(entries.join("\n").as_bytes());
  }
  let output = child.wait_with_output().ok()?;
  choice_from_selection(&String::from_utf8_lossy(&output.stdout))
}

/// The color of a picker line: the last field, after the menu number and the
/// name. An empty selection (the picker was closed) gives `None`.
fn choice_from_selection(selection: &str) -> Option<String> {
  selection.split_whitespace().last().map(str::to_string)
}

/// Sends the notification of a color change. A missing notification daemon is
/// not an error of the change.
fn notify(color: &str) {
  let title = i18n::text("accent.notification.title", &[]);
  let message = i18n::text("accent.notification.message", &[format!("color={color}")]);
  let _ = Command::new("notify-send")
    .arg(title)
    .arg(message)
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .status();
}

/// Applies one request. The mode words keep the contract of the shell script:
/// `--startup` and `--apply-static` write without the runtime, `--apply` also
/// reloads the runtime, and `--theme-default` takes the theme's own accent.
/// Any other value is a color, which is also notified.
fn apply(
  paths: &SessionPaths,
  requested_mode: &str,
  icons: &IconNames,
) -> Result<Applied, Failure> {
  let themes_dir = paths.system_config("appearance/themes.d");
  let state_dir = paths.config_home.join("argvus/data");
  let active_file = state_dir.join(".active-theme");
  let accent_file = state_dir.join(".accent-color");
  let store = ConfigStore::from_environment().ok();
  let saved_accent = || {
    accent::state_accent(
      &themes_dir,
      &state_dir,
      accent::canonical_accent(store.as_ref()),
    )
  };
  let (mut runtime, mut notify, requested) = match requested_mode {
    "--startup" | "--apply-static" => (false, false, saved_accent()),
    "--apply" => (true, false, saved_accent()),
    "--theme-default" => {
      let active =
        accent::canonical_theme_id(&accent::read_state(&active_file, accent::DEFAULT_THEME))
          .to_string();
      match accent::theme_default_accent(&themes_dir, &active) {
        Some(default) => (false, false, default),
        None => return Err(Failure::DefaultMissing(active)),
      }
    }
    color => (true, true, color.to_string()),
  };
  if env::var("ARGVUS_NO_RUNTIME").as_deref() == Ok("1") {
    runtime = false;
    notify = false;
  }
  let Some(accent_value) = accent::normalize(&requested) else {
    return Err(Failure::Invalid(requested));
  };
  let active = accent::canonical_theme_id(&accent::read_state(&active_file, accent::DEFAULT_THEME))
    .to_string();
  let theme = accent::applied_theme(&themes_dir, &active);
  let greeter_state_dir = env::var_os("ARGVUS_GREETER_THEME_STATE_DIR")
    .filter(|value| !value.is_empty())
    .map(PathBuf::from)
    .unwrap_or_else(|| PathBuf::from("/var/lib/argvus/greeter/themes"));
  let home = PathBuf::from(env::var_os("HOME").unwrap_or_default());
  let xdg_config_home = env::var_os("XDG_CONFIG_HOME")
    .filter(|value| !value.is_empty())
    .map(PathBuf::from)
    .unwrap_or_else(|| home.join(".config"));
  let cache_home = env::var_os("XDG_CACHE_HOME")
    .filter(|value| !value.is_empty())
    .map(PathBuf::from)
    .unwrap_or_else(|| home.join(".cache"));
  let written = fs::create_dir_all(&state_dir)
    .and_then(|()| fs::write(&active_file, format!("{theme}\n")))
    .and_then(|()| fs::write(&accent_file, format!("{}\n", accent_value.color)));
  if let Err(error) = written {
    return Err(Failure::State(error.to_string()));
  }
  report_result(accent::publish_greeter_accent(
    &greeter_state_dir,
    &accent_value,
  ));
  let context = accent::Context {
    paths,
    themes_dir: &themes_dir,
    accent: &accent_value,
    theme: &theme,
    icon_theme: &icons.current,
    icon_default_dark: &icons.dark,
    icon_default_light: &icons.light,
    runtime,
    theme_switch: env::var("ARGVUS_THEME_SWITCH").as_deref() == Ok("1"),
    greeter_state_dir: &greeter_state_dir,
    cache_home: &cache_home,
    xdg_config_home: &xdg_config_home,
  };
  accent::apply(&context);
  Ok(Applied {
    color: accent_value.color.clone(),
    theme,
    notify,
  })
}

/// Reports a write that failed without stopping the change: the greeter
/// projection is best effort, as before.
fn report_result(result: std::io::Result<()>) {
  if let Err(error) = result {
    eprintln!("argvus-config: {error}");
  }
}

#[cfg(test)]
mod tests {
  use super::{CHOICES, choice_from_selection};

  #[test]
  fn picker_line_yields_its_color() {
    let selection = "01 - Azul  #3590bd\n";
    assert_eq!(choice_from_selection(selection).as_deref(), Some("#3590bd"));
  }

  #[test]
  fn closed_picker_yields_nothing() {
    assert_eq!(choice_from_selection(""), None);
    assert_eq!(choice_from_selection("\n"), None);
  }

  #[test]
  fn every_choice_is_a_valid_color() {
    for (_, color) in CHOICES {
      assert!(crate::accent::normalize(color).is_some(), "{color}");
    }
  }
}
