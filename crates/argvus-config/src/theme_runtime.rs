//! The running desktop takes a new theme without restarting anything.
//!
//! Each consumer says in the theme registry (`theme/consumers.d`) how it picks
//! up a new build: a rewrite of the stylesheet Waybar watches, a signal, or a
//! command. Hyprland's colours are evaluated in place, and the wallpaper goes
//! through hyprpaper's IPC. The compositor reloads its configuration only when
//! the switch changed something else it reads: the Sticky/Float layout, the
//! per-theme effects or the projected state. A layout change also restarts the
//! two Waybar bars, whose margins are layer-shell settings. A surface whose
//! consumer is not registered is restarted, and Hyprland reloads when its own
//! consumer is missing, so a partly upgraded desktop still takes the theme.

use crate::session_paths::SessionPaths;
use crate::{ConfigStore, accent, hyprland_ipc, theme_build, theme_surfaces};
use argvus_theme_core::registry::{self, Reload, ReloadStage};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread::sleep;
use std::time::{Duration, Instant};

/// How long the switch waits for the reload commands of the consumers.
const HOOK_WAIT_LIMIT: Duration = Duration::from_secs(5);
const POLL_INTERVAL: Duration = Duration::from_millis(20);

/// The first line of a stylesheet that imports the build; a new value makes
/// Waybar's `reload_style_on_change` re-read the style.
const BUILD_MARKER_PREFIX: &str = "/* argvus theme build: ";

/// The bars whose margins live in their JSONC: a layout change restarts them.
const BAR_UNITS: [&str; 2] = ["argvus-taskbar.service", "argvus-widget-telemetry.service"];

/// The surfaces that show the theme, by the consumer that reloads them in
/// place. A switch restarts the unit of a consumer that is not registered (its
/// package is older than the registry), as switches did before it.
const SURFACE_UNITS: [(&str, &str); 4] = [
  ("taskbar", "argvus-taskbar.service"),
  ("widget-telemetry", "argvus-widget-telemetry.service"),
  ("control-panel", "argvus-control-panel.service"),
  ("notifications", "argvus-dunst.service"),
];

/// The consumer that evaluates the theme's colours in Hyprland. Without it the
/// compositor reloads its configuration instead.
const HYPRLAND_CONSUMER: &str = "hyprland";

/// Restarted as well when no consumer at all is registered: the wallpaper
/// service of the components that predate the registry.
const WALLPAPER_UNIT: &str = "argvus-wallpaper.service";

/// The Hyprland state lines a theme switch changes on purpose and applies
/// itself: the splash colour, the accent ownership and the layout.
const SWITCH_OWNED_STATE: [&str; 3] = ["background_color", "accent_custom", "layout_variant"];

/// What the compositor read before the switch, to tell whether it must reload.
pub struct Before {
  theme: String,
  effects: Option<String>,
  state: Option<String>,
}

impl Before {
  /// Reads the active theme, its effects and the projected Hyprland state.
  pub fn capture(paths: &SessionPaths) -> Self {
    let theme = theme_build::displayed_theme(paths);
    Self {
      effects: theme_effects(paths, &theme),
      state: compositor_state(paths),
      theme,
    }
  }
}

/// Applies the switch to `theme` (`<id>` or `<id>-float`) on the running
/// desktop. Failures are reported and do not stop the other consumers.
pub fn apply(paths: &SessionPaths, before: &Before, theme: &str) {
  let consumers = registered_consumers(paths);
  let registered = |id: &str| consumers.iter().any(|consumer| consumer.id == id);
  let layout_changed = before.theme.ends_with("-float") != theme.ends_with("-float");
  let mut restart: Vec<&str> = SURFACE_UNITS
    .iter()
    .filter(|(id, _)| !registered(id))
    .map(|(_, unit)| *unit)
    .collect();
  if consumers.is_empty() {
    restart.push(WALLPAPER_UNIT);
  }
  if layout_changed {
    for unit in BAR_UNITS {
      if !restart.contains(&unit) {
        restart.push(unit);
      }
    }
  }
  let reload = !registered(HYPRLAND_CONSUMER)
    || layout_changed
    || before.effects != theme_effects(paths, theme)
    || before.state != compositor_state(paths);
  // Without a compositor (a test, a TTY) there is nothing to reload.
  if reload && std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some() {
    report(
      hyprland_ipc::reload()
        .map(|_| ())
        .map_err(|error| error.to_string()),
    );
  }
  reload_consumers(paths, Some(ReloadStage::Files));
  if !restart.is_empty() {
    let _ = Command::new("systemctl")
      .args(["--user", "try-restart"])
      .args(restart)
      .stdout(Stdio::null())
      .stderr(Stdio::null())
      .status();
  }
}

/// Runs the reload of the registered consumers of `stage` (all of them with
/// `None`), together, and waits for them at most `HOOK_WAIT_LIMIT`. A switch
/// runs the `Build` stage as soon as the build is active and the `Files` stage
/// at its end; an accent change runs them all.
pub fn reload_consumers(paths: &SessionPaths, stage: Option<ReloadStage>) {
  let consumers = registered_consumers(paths);
  let build = build_name(&theme_build::theme_paths(paths).current());
  let mut children = Vec::new();
  for consumer in consumers
    .iter()
    .filter(|consumer| stage.is_none_or(|stage| consumer.reload_stage == stage))
  {
    match &consumer.reload {
      Reload::None => {}
      Reload::Rewrite { path } => {
        report(mark_stylesheet(&paths.config_home.join(path), &build));
      }
      Reload::Signal { process, signal } => children.extend(spawn(
        Command::new("pkill")
          .args([
            "--signal",
            signal,
            "-u",
            &theme_surfaces::current_user_id().to_string(),
          ])
          .args(["-x", process]),
      )),
      Reload::Command { argv } => {
        let argv: Vec<String> = argv.iter().map(|word| expand(paths, word)).collect();
        children.extend(spawn(Command::new(&argv[0]).args(&argv[1..])));
      }
    }
  }
  wait_all(children, HOOK_WAIT_LIMIT);
}

/// The consumers of the theme registry; an unreadable registry is reported and
/// counts as empty.
fn registered_consumers(paths: &SessionPaths) -> Vec<registry::Consumer> {
  let theme_paths = theme_build::theme_paths(paths);
  registry::load(&theme_paths.system_root, &theme_paths.registry()).unwrap_or_else(|error| {
    eprintln!("argvus-config: theme registry: {error}");
    Vec::new()
  })
}

/// Evaluates the colours of the active theme in Hyprland: the `theme.lua` the
/// configuration loads, with the user's accent on the active borders when the
/// user chose one, as `hyprland.lua` does.
pub fn apply_hyprland_colors(paths: &SessionPaths) -> Result<(), String> {
  let theme = active_theme(paths);
  let path =
    hyprland_theme_file(paths, &theme).ok_or_else(|| format!("no theme.lua for {theme}"))?;
  let text = fs::read_to_string(&path).map_err(|error| format!("{}: {error}", path.display()))?;
  let mut values = lua_values(&text);
  if let Some(accent) = custom_accent(paths) {
    let border = format!(
      "\"rgba({}ff)\"",
      accent.trim_start_matches('#').to_lowercase()
    );
    values.retain(|(key, _)| key != "border_active" && key != "groupbar_active");
    values.push(("border_active".into(), border.clone()));
    values.push(("groupbar_active".into(), border));
  }
  let lua = colors_lua(&values).ok_or_else(|| format!("{}: missing colours", path.display()))?;
  hyprland_ipc::eval(&lua)
    .map(|_| ())
    .map_err(|error| error.to_string())
}

/// The `hl.config` call for the colour keys of a theme table, or `None` when
/// a colour is missing.
fn colors_lua(values: &[(String, String)]) -> Option<String> {
  let get = |key: &str| {
    values
      .iter()
      .rev()
      .find(|(name, _)| name == key)
      .map(|(_, value)| value.as_str())
  };
  let active = get("border_active")?;
  let inactive = get("border_inactive")?;
  let group_active = get("groupbar_active")?;
  let group_inactive = get("groupbar_inactive")?;
  let shadow = get("shadow_color")?;
  let shadow_inactive = get("shadow_color_inactive")?;
  let rounding_power = get("rounding_power")
    .map(|value| format!(", rounding_power = {value}"))
    .unwrap_or_default();
  Some(format!(
    "hl.config({{ general = {{ col = {{ active_border = {active}, inactive_border = {inactive} }} }}, \
group = {{ col = {{ border_active = {active}, border_inactive = {inactive}, \
border_locked_active = {active}, border_locked_inactive = {inactive} }}, \
groupbar = {{ col = {{ active = {group_active}, inactive = {group_inactive}, \
locked_active = {group_active}, locked_inactive = {group_inactive} }} }} }}, \
decoration = {{ shadow = {{ color = {shadow}, color_inactive = {shadow_inactive} }}{rounding_power} }} }})"
  ))
}

/// The `key = value` lines of a returned Lua table, values kept as Lua text.
fn lua_values(text: &str) -> Vec<(String, String)> {
  text
    .lines()
    .filter_map(|line| {
      let line = line.split("--").next()?.trim().trim_end_matches(',');
      let (key, value) = line.split_once('=')?;
      let key = key.trim();
      let value = value.trim();
      let valid_key = !key.is_empty()
        && key
          .bytes()
          .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_');
      (valid_key && is_lua_literal(value)).then(|| (key.to_owned(), value.to_owned()))
    })
    .collect()
}

/// A plain string without inner quotes, a `0x` hex integer or a number: the only
/// values a theme table may hand to `hl.config`.
fn is_lua_literal(value: &str) -> bool {
  let string = value.len() >= 2
    && value.starts_with('"')
    && value.ends_with('"')
    && !value[1..value.len() - 1].contains(['"', '\\']);
  let hex = value.strip_prefix("0x").is_some_and(|digits| {
    !digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_hexdigit())
  });
  string || hex || value.parse::<f64>().is_ok_and(f64::is_finite)
}

/// The `theme.lua` Hyprland loads for `theme`: the one of the theme build.
fn hyprland_theme_file(paths: &SessionPaths, theme: &str) -> Option<PathBuf> {
  theme_build::current_file_for(paths, theme, "hypr/theme.lua")
}

/// The accent the user chose (`#RRGGBB`), or `None` when the theme owns it.
fn custom_accent(paths: &SessionPaths) -> Option<String> {
  accent::canonical_accent(Some(&ConfigStore::at(&paths.config_home)))
    .and_then(|value| accent::normalize(&value))
    .map(|accent| accent.color)
}

/// Shows `wallpaper` on every monitor through hyprpaper. Without hyprpaper
/// (the swaybg fallback), the wallpaper service starts again with it.
pub fn show_wallpaper(paths: &SessionPaths, wallpaper: &str) {
  let running = Command::new("pgrep")
    .args([
      "-u",
      &theme_surfaces::current_user_id().to_string(),
      "-x",
      "hyprpaper",
    ])
    .stdout(Stdio::null())
    .status()
    .is_ok_and(|status| status.success());
  if !running {
    let _ = Command::new("systemctl")
      .args(["--user", "try-restart", "argvus-wallpaper.service"])
      .stdout(Stdio::null())
      .stderr(Stdio::null())
      .status();
    return;
  }
  let fit = hyprpaper_setting(paths, "fit_mode")
    .filter(|value| value.bytes().all(|byte| byte.is_ascii_lowercase()))
    .unwrap_or_else(|| "cover".to_owned());
  for monitor in monitor_names() {
    let _ = Command::new("hyprctl")
      .args([
        "hyprpaper",
        "wallpaper",
        &format!("{monitor},{wallpaper},{fit}"),
      ])
      .stdout(Stdio::null())
      .stderr(Stdio::null())
      .status();
  }
}

/// A `key = value` setting of the user's hyprpaper configuration, which the
/// switch has just written with the theme's wallpaper.
fn hyprpaper_setting(paths: &SessionPaths, key: &str) -> Option<String> {
  let text = fs::read_to_string(paths.data_home.join("hypr/hyprpaper.conf")).ok()?;
  text
    .lines()
    .find_map(|line| {
      let (name, value) = line.split_once('=')?;
      (name.trim() == key).then(|| value.trim().to_owned())
    })
    .filter(|value| !value.is_empty() && !value.contains(','))
}

fn monitor_names() -> Vec<String> {
  let Ok(output) = Command::new("hyprctl").args(["-j", "monitors"]).output() else {
    return Vec::new();
  };
  serde_json::from_slice::<serde_json::Value>(&output.stdout)
    .ok()
    .and_then(|value| value.as_array().cloned())
    .unwrap_or_default()
    .iter()
    .filter_map(|monitor| monitor.get("name")?.as_str().map(str::to_owned))
    .collect()
}

/// Writes `marker` as the first line of a stylesheet, replacing an older one.
/// A stylesheet that is missing is left alone.
fn mark_stylesheet(path: &Path, build: &str) -> Result<(), String> {
  let Ok(text) = fs::read_to_string(path) else {
    return Ok(());
  };
  let rest = match text.split_once('\n') {
    Some((first, rest)) if first.starts_with(BUILD_MARKER_PREFIX) => rest,
    _ => text.as_str(),
  };
  let marked = format!("{BUILD_MARKER_PREFIX}{build} */\n{rest}");
  crate::text_edit::replace_file(path, &marked)
    .map_err(|error| format!("{}: {error}", path.display()))
}

/// The name of the build behind `current`, or the time of this switch when
/// there is none, so the stylesheet still changes.
fn build_name(current: &Path) -> String {
  fs::read_link(current)
    .ok()
    .and_then(|target| {
      target
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
    })
    .unwrap_or_else(|| {
      let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
      format!("none-{}", now.as_millis())
    })
}

/// `${ARGVUS_DATA_HOME}` and `${ARGVUS_CONFIG_HOME}` in a registry command.
fn expand(paths: &SessionPaths, word: &str) -> String {
  word
    .replace("${ARGVUS_DATA_HOME}", &paths.data_home.to_string_lossy())
    .replace(
      "${ARGVUS_CONFIG_HOME}",
      &paths.config_home.to_string_lossy(),
    )
}

fn spawn(command: &mut Command) -> Option<Child> {
  command
    .stdin(Stdio::null())
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .spawn()
    .ok()
}

/// Waits for the children until `limit`; the ones still running are killed.
fn wait_all(mut children: Vec<Child>, limit: Duration) {
  let started = Instant::now();
  while !children.is_empty() {
    children.retain_mut(|child| matches!(child.try_wait(), Ok(None)));
    if started.elapsed() >= limit {
      for child in &mut children {
        let _ = child.kill();
        let _ = child.wait();
      }
      return;
    }
    if !children.is_empty() {
      sleep(POLL_INTERVAL);
    }
  }
}

fn active_theme(paths: &SessionPaths) -> String {
  fs::read_to_string(paths.data_home.join(".active-theme"))
    .map(|text| text.trim().to_owned())
    .unwrap_or_default()
}

fn theme_effects(paths: &SessionPaths, theme: &str) -> Option<String> {
  fs::read_to_string(
    paths
      .data_home
      .join("state/effects")
      .join(format!("{theme}.conf")),
  )
  .ok()
}

/// The projected Hyprland state without the lines the switch applies itself.
fn compositor_state(paths: &SessionPaths) -> Option<String> {
  let text = fs::read_to_string(paths.data_home.join("generated/hypr/state.lua")).ok()?;
  Some(
    text
      .lines()
      .filter(|line| {
        !SWITCH_OWNED_STATE
          .iter()
          .any(|key| line.trim_start().starts_with(key))
      })
      .collect::<Vec<_>>()
      .join("\n"),
  )
}

fn report(result: Result<(), String>) {
  if let Err(error) = result {
    eprintln!("argvus-config: theme runtime: {error}");
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  const THEME_LUA: &str = r#"-- Theme: sunset
return {
  border_active = "rgba(e2be8aff)",
  border_inactive = "rgba(3a332dff)",
  groupbar_active = "rgba(e2be8aff)",
  groupbar_inactive = "rgba(3a332dcc)",
  shadow_color = 0xCC000000,
  shadow_color_inactive = 0x660F0F0F,
  rounding = 4,
  rounding_power = 2,
  border_size = 1,
}
"#;

  #[test]
  fn reads_the_theme_table_values_as_lua_text() {
    let values = lua_values(THEME_LUA);
    assert!(values.contains(&("border_active".into(), "\"rgba(e2be8aff)\"".into())));
    assert!(values.contains(&("shadow_color".into(), "0xCC000000".into())));
    assert!(values.contains(&("rounding_power".into(), "2".into())));
    assert!(!values.iter().any(|(key, _)| key == "return"));
  }

  #[test]
  fn refuses_values_that_are_not_plain_literals() {
    let values =
      lua_values("return {\n  border_active = os.execute(\"x\"),\n  a = \"b\" .. \"c\",\n}\n");
    assert!(values.is_empty(), "{values:?}");
  }

  #[test]
  fn builds_one_hl_config_call_with_every_colour() {
    let lua = colors_lua(&lua_values(THEME_LUA)).unwrap();
    assert!(lua.starts_with("hl.config({ general = { col = { active_border = \"rgba(e2be8aff)\""));
    assert!(
      lua.contains(
        "groupbar = { col = { active = \"rgba(e2be8aff)\", inactive = \"rgba(3a332dcc)\""
      )
    );
    assert!(lua.contains(
      "shadow = { color = 0xCC000000, color_inactive = 0x660F0F0F }, rounding_power = 2"
    ));
    assert!(
      colors_lua(&lua_values(
        "return {\n  border_active = \"rgba(000000ff)\",\n}\n"
      ))
      .is_none()
    );
  }

  #[test]
  fn marks_a_stylesheet_once_and_replaces_its_marker() {
    let dir = std::env::temp_dir().join(format!("argvus-theme-runtime-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let css = dir.join("argvus-taskbar.css");
    fs::write(&css, "@import url(\"a\");\n").unwrap();
    mark_stylesheet(&css, "sunset-sticky-1").unwrap();
    mark_stylesheet(&css, "dracula-sticky-2").unwrap();
    assert_eq!(
      fs::read_to_string(&css).unwrap(),
      "/* argvus theme build: dracula-sticky-2 */\n@import url(\"a\");\n"
    );
    mark_stylesheet(&dir.join("missing.css"), "x").unwrap();
    fs::remove_dir_all(&dir).unwrap();
  }

  #[test]
  fn expands_only_the_documented_placeholders() {
    let paths = SessionPaths {
      config_home: PathBuf::from("/home/u/.config"),
      data_home: PathBuf::from("/home/u/.config/argvus/data"),
      system_config: PathBuf::from("/usr/share/argvus"),
    };
    assert_eq!(
      expand(&paths, "${ARGVUS_DATA_HOME}/dunst/dunstrc"),
      "/home/u/.config/argvus/data/dunst/dunstrc"
    );
    assert_eq!(expand(&paths, "${HOME}/x"), "${HOME}/x");
  }
}
