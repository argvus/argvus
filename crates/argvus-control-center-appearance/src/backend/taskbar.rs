//! Taskbar position, shape, icons, date and time formats, and reloading it with the apply overlay.

use super::commands::{read_first, run_script, script};
use super::themes::active_theme_file;
use super::*;

/// Persists the taskbar utility-group mode (the `group/right-2` drawer's
/// expanded/collapsed presentation) and regenerates its managed Waybar
/// configuration. Reload is left to the caller, which also applies the rest
/// of the taskbar icon/date/time preferences in the same action and restarts
/// the taskbar exactly once.
pub fn set_taskbar_utility_group(mode: TaskbarUtilityGroupMode) -> Result<(), String> {
  run_script(&script("taskbar-right-2-mode.sh"), &["set", mode.value()])
}

/// The loading splash shown while an `Apply` recreates a surface (a Waybar bar
/// or the Control Panel). It is up before the first write and stays until the
/// guard is dropped after the last step, so no change is visible before it.
/// Changes that reload in place (styles, blur, transparency) show no splash.
///
/// The overlay follows the process that started it, so it is held by
/// `argvus config transition-overlay hold`, which keeps it up until its stdin
/// closes. Dropping the guard closes that pipe, and so does the Control Center
/// exiting, so the splash is never left covering the screen.
pub struct ApplyOverlay {
  pub(super) holder: Option<Child>,
}

impl ApplyOverlay {
  /// Shows the splash when `needed` and returns once it is on screen. When it
  /// is not needed, cannot be shown, or another owner already shows one, the
  /// guard holds nothing.
  pub fn start_if(needed: bool) -> Self {
    if !needed {
      return Self { holder: None };
    }
    let theme = canonical_theme_id(&read_first(&active_theme_file(), DEFAULT_THEME));
    let holder = argvus_control_center_core::process::command("argvus")
      .args(["config", "transition-overlay", "hold", &theme])
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      // The Control Center is a full-screen terminal UI; keep its screen clean.
      .stderr(Stdio::null())
      .spawn()
      .ok()
      .and_then(|mut holder| {
        // `hold` prints `started` once the splash is on screen.
        let started = holder.stdout.take().is_some_and(|stdout| {
          let mut line = String::new();
          BufReader::new(stdout).read_line(&mut line).is_ok() && line.trim() == "started"
        });
        if started {
          Some(holder)
        } else {
          drop(holder.stdin.take());
          let _ = holder.wait();
          None
        }
      });
    Self { holder }
  }
}

impl Drop for ApplyOverlay {
  fn drop(&mut self) {
    if let Some(mut holder) = self.holder.take() {
      // Closing stdin tells `hold` to stop the splash; waiting lets it fade out.
      drop(holder.stdin.take());
      let _ = holder.wait();
    }
  }
}

/// Writes the taskbar height and border to the live taskbar files. A border that
/// is off is written as width `0`. It does not reload or restart anything: the
/// border is a style the bar reloads by itself, and the `Apply` restarts the
/// taskbar once when the height changed.
pub fn set_taskbar_shape(
  height: u32,
  border_enabled: bool,
  border_thickness: u32,
) -> Result<(), String> {
  let border_width = if border_enabled { border_thickness } else { 0 };
  run_taskbar_config(&["taskbar-height", &height.to_string()])?;
  run_taskbar_config(&["taskbar-border", &border_width.to_string()])
}

pub(super) fn run_taskbar_config(arguments: &[&str]) -> Result<(), String> {
  let status = argvus_control_center_core::process::command("argvus")
    .arg("config")
    .args(arguments)
    .status()
    .map_err(|error| format!("failed to set the taskbar shape: {error}"))?;
  if !status.success() {
    return Err("argvus config rejected the taskbar shape".into());
  }
  Ok(())
}

/// Persists the taskbar icon/date/time preferences to the canonical document
/// and re-derives the Waybar config from them. It does not reload
/// `argvus-config.service` nor restart the taskbar: the `Apply` of the taskbar
/// surface restarts it once, under the splash, when any of them changed.
pub fn apply_taskbar_icons_and_format(
  audio_player_enabled: bool,
  launcher_enabled: bool,
  utility_widgets: &TaskbarUtilityWidgets,
  date_format: TaskbarDateFormat,
  time_seconds_enabled: bool,
  time_format: TaskbarTimeFormat,
) -> Result<(), String> {
  let mut patch = serde_json::Map::new();
  patch.insert(
    "/taskbar/icons/audio_player_enabled".into(),
    Value::Bool(audio_player_enabled),
  );
  patch.insert(
    "/taskbar/icons/launcher_enabled".into(),
    Value::Bool(launcher_enabled),
  );
  for widget in TaskbarUtilityWidget::ALL {
    patch.insert(
      format!("/taskbar/icons/{}_enabled", widget.key()),
      Value::Bool(utility_widgets.enabled(widget)),
    );
  }
  patch.insert(
    "/taskbar/date/format".into(),
    Value::String(date_format.value().into()),
  );
  patch.insert(
    "/taskbar/time/seconds_enabled".into(),
    Value::Bool(time_seconds_enabled),
  );
  patch.insert(
    "/taskbar/time/format".into(),
    Value::String(time_format.value().into()),
  );
  persist_taskbar_patch(patch)?;
  run_script(&script("taskbar-widgets-mode.sh"), &["apply"])
}

/// Restarts the taskbar (Waybar) process so it re-parses its generated
/// JSONC from scratch. `argvus-sessionctl reload` is not sufficient here: it
/// only reload-or-restarts `argvus-config.service` and never touches
/// `argvus-taskbar.service` (confirmed in `de/argvus-session`'s
/// `argvus-sessionctl`/`test-session-reload.py`). A full restart is also the
/// only way Waybar re-reads anything beyond CSS — `reload_style_on_change`
/// only watches the stylesheet, and the SIGUSR1 binding in Hyprland merely
/// toggles bar visibility, not a config reload.
pub fn reload_taskbar() -> Result<(), String> {
  let output = SystemProcessRunner
    .run(
      &ProcessRequest::new("argvus")
        .arg("sessionctl")
        .arg("restart")
        .arg("taskbar"),
    )
    .map_err(|error| error.to_string())?;
  if output.status.is_none_or(|status| status != 0) {
    let stderr = terminal_text(&String::from_utf8_lossy(&output.stderr))
      .trim()
      .to_string();
    return Err(if stderr.is_empty() {
      "argvus-sessionctl restart taskbar falhou".into()
    } else {
      stderr
    });
  }
  Ok(())
}

/// Writes the taskbar patch without reloading the configuration (see
/// `apply_taskbar_icons_and_format`).
pub(super) fn persist_taskbar_patch(patch: serde_json::Map<String, Value>) -> Result<(), String> {
  let patch = serde_json::to_string(&patch).map_err(|error| error.to_string())?;
  let status = argvus_control_center_core::process::command("argvus")
    .arg("config")
    .args(["patch", &patch])
    .status()
    .map_err(|error| format!("failed to persist taskbar settings: {error}"))?;
  if !status.success() {
    return Err("argvus config rejected the taskbar settings".into());
  }
  Ok(())
}
