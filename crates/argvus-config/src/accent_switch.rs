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
use crate::json_pointers;
use crate::session_paths::SessionPaths;
use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, ExitCode, Stdio};

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
      Ok(Some(choice)) => choice,
      Ok(None) => return ExitCode::SUCCESS,
      Err(error) => {
        eprintln!("argvus-config: accent picker: {error}");
        return ExitCode::from(1);
      }
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

/// Opens the picker with the current effective color.
fn pick(paths: &SessionPaths) -> Result<Option<String>, String> {
  let picker = env::current_exe()
    .ok()
    .filter(|path| path.file_name().is_some_and(|name| name == "argvus"))
    .unwrap_or_else(|| PathBuf::from("argvus"));
  let store = ConfigStore::from_environment().ok();
  let current = accent::state_accent(
    &crate::theme_build::theme_paths(paths),
    &paths.config_home.join("argvus/data"),
    accent::canonical_accent(store.as_ref()),
  );
  let initial = serde_json::to_vec(&current).map_err(|error| error.to_string())?;
  let mut child = Command::new(picker)
    .args(["appearance-picker", "accent"])
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .spawn()
    .map_err(|error| error.to_string())?;
  let write_result = child
    .stdin
    .take()
    .ok_or_else(|| "missing picker stdin".to_owned())
    .and_then(|mut input| input.write_all(&initial).map_err(|error| error.to_string()));
  let output = child
    .wait_with_output()
    .map_err(|error| error.to_string())?;
  write_result?;
  if !output.status.success() {
    return Err(output.status.to_string());
  }
  let selection = std::str::from_utf8(&output.stdout).map_err(|error| error.to_string())?;
  selected_color(selection)
}

fn selected_color(selection: &str) -> Result<Option<String>, String> {
  let selection = selection.trim();
  if selection.is_empty() {
    return Ok(None);
  }
  accent::normalize(selection)
    .map(|accent| Some(accent.color))
    .ok_or_else(|| "invalid accent color from picker".to_owned())
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
  let theme_paths = crate::theme_build::theme_paths(paths);
  let state_dir = paths.config_home.join("argvus/data");
  let active_file = state_dir.join(".active-theme");
  let accent_file = state_dir.join(".accent-color");
  let store = ConfigStore::from_environment().ok();
  let saved_accent = || {
    accent::state_accent(
      &theme_paths,
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
      match accent::theme_default_accent(&theme_paths, &active) {
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
  // A chosen color is the user's custom accent; `--theme-default` returns to the
  // theme's own accent. Both are written to the canonical document in one step,
  // so a later theme switch or projection keeps the choice. The startup and
  // apply modes only re-apply what is already stored and write nothing.
  let custom_choice = match requested_mode {
    "--theme-default" => Some(false),
    "--startup" | "--apply-static" | "--apply" => None,
    _ => Some(true),
  };
  if let Some(custom) = custom_choice {
    let color = accent_value.color.clone();
    let store =
      ConfigStore::from_environment().map_err(|error| Failure::State(error.to_string()))?;
    store
      .modify(|document| {
        document.set(
          json_pointers::APPEARANCE_ACCENT,
          serde_json::Value::String(color.clone()),
        )?;
        document.set(
          json_pointers::APPEARANCE_ACCENT_CUSTOM,
          serde_json::Value::Bool(custom),
        )
      })
      .map_err(|error| Failure::State(error.to_string()))?;
  }
  let active = accent::canonical_theme_id(&accent::read_state(&active_file, accent::DEFAULT_THEME))
    .to_string();
  let theme = accent::applied_theme(&theme_paths, &active);
  let greeter_state_dir = env::var_os("ARGVUS_GREETER_THEME_STATE_DIR")
    .filter(|value| !value.is_empty())
    .map(PathBuf::from)
    .unwrap_or_else(|| PathBuf::from("/var/lib/argvus/greeter/themes"));
  let home = PathBuf::from(env::var_os("HOME").unwrap_or_default());
  let xdg_config_home = env::var_os("XDG_CONFIG_HOME")
    .filter(|value| !value.is_empty())
    .map(PathBuf::from)
    .unwrap_or_else(|| home.join(".config"));
  let written = fs::create_dir_all(&state_dir)
    .and_then(|()| crate::text_edit::replace_file(&active_file, &format!("{theme}\n")))
    .and_then(|()| {
      crate::text_edit::replace_file(&accent_file, &format!("{}\n", accent_value.color))
    });
  if let Err(error) = written {
    return Err(Failure::State(error.to_string()));
  }
  report_result(accent::publish_greeter_accent(
    &greeter_state_dir,
    &accent_value,
  ));
  // The theme build carries the accent: every themed file follows it.
  let theme_switch = env::var("ARGVUS_THEME_SWITCH").as_deref() == Ok("1");
  // The transaction has already rendered the default palette. Custom accents
  // still need their own build; an explicit accent command always refreshes it.
  if !theme_switch || requested_mode != "--theme-default" {
    crate::theme_build::refresh_or_report(paths, &theme, Some(&accent_value.color));
  }
  let context = accent::Context {
    paths,
    theme: &theme,
    icon_theme: &icons.current,
    icon_default_dark: &icons.dark,
    icon_default_light: &icons.light,
    runtime,
    theme_switch,
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
  use super::selected_color;

  #[test]
  fn accepts_custom_color_but_rejects_non_colors() {
    assert_eq!(selected_color("#12ab9f\n"), Ok(Some("#12AB9F".into())));
    assert!(selected_color("#abc").is_err());
    assert!(selected_color("--theme-default").is_err());
  }

  #[test]
  fn closed_picker_yields_nothing() {
    assert_eq!(selected_color(""), Ok(None));
    assert_eq!(selected_color("\n"), Ok(None));
  }
}
