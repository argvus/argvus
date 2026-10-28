//! Visual effects of the desktop: transparency, blur and animations.
//!
//! The state lives in argvus-config (`/effects/...`) and in a per-theme
//! projection file for the values the store does not own. Applying a change
//! rewrites the surfaces that consume it: the Waybar stylesheets, the launcher
//! theme, dunst, the taskbar calendar, superfile and hyprlock. This is the
//! Rust home of the former `effects-toggle.sh`; `argvus effects-toggle` runs
//! `effects`.

use crate::accent::{Rule, apply_rules, edit_file};
use crate::json_pointers;
use crate::session_paths::SessionPaths;
use crate::user_copy;
use crate::{ConfigStore, text_edit};
use serde_json::{Map, Value};
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

mod commands;
mod css_text;
mod effect_settings;
mod projection;
mod surfaces;

use commands::{effects_status, raw_text};
use css_text::read_first_line;
pub use effect_settings::settings_valid;
pub(crate) use surfaces::apply_launcher;

/// Every surface that owns a transparency switch.
const TRANSPARENCY_SURFACES: [&str; 6] = [
  "taskbar",
  "control-panel",
  "widget-telemetry",
  "terminal",
  "launchers",
  "control-center",
];

/// The surfaces whose own transparency switch decides the EFFECTS status: the
/// Application Style pages (Terminal, Launchers) and the Taskbar, Widget
/// Telemetry and Control Panel pages. Control Center is written with them but
/// does not take part in the reading.
const EFFECT_SURFACES: [&str; 5] = [
  "taskbar",
  "control-panel",
  "widget-telemetry",
  "terminal",
  "launchers",
];

/// The blur parameters: `key minimum maximum kind default`.
const BLUR_SETTINGS: &str = "blur_size 1 64 integer 6
blur_passes 1 8 integer 2
blur_brightness 0 2 number 1
blur_noise 0 1 number 0
blur_contrast 0 2 number 0.9
blur_vibrancy 0 1 number 0.1
blur_vibrancy_darkness 0 1 number 0";

/// The animation parameters, in the same format.
const ANIMATION_SETTINGS: &str = "animation_opening_enabled - - boolean true
animation_opening_style - - enum:slide,popin,gnomed popin
animation_opening_speed 1 10 number 4.1
animation_opening_curve - - enum:default,bezier,spring,linear,ease_out default
animation_opening_popin 1 100 integer 87
animation_closing_style - - enum:slide,popin,gnomed popin
animation_closing_speed 1 10 number 1.49
animation_closing_curve - - enum:default,bezier,spring,linear,ease_out default
animation_closing_popin 1 100 integer 87
animation_moving_enabled - - boolean true
animation_moving_speed 1 10 number 4.79
animation_moving_curve - - enum:default,bezier,spring,linear,ease_out default
animation_workspace_in_style - - enum:slide,fade,slidefade,slidevert,slidefadevert slide
animation_workspace_in_speed 1 10 number 1.21
animation_workspace_in_curve - - enum:default,bezier,spring,linear,ease_out default
animation_workspace_out_style - - enum:slide,fade,slidefade,slidevert,slidefadevert slide
animation_workspace_out_speed 1 10 number 1.94
animation_workspace_out_curve - - enum:default,bezier,spring,linear,ease_out default
animation_special_workspace_style - - enum:slide,fade,slidefade,slidevert,slidefadevert slide
animation_special_workspace_speed 1 10 number 5
animation_special_workspace_curve - - enum:default,bezier,spring,linear,ease_out default
animation_layers_in_style - - enum:slide,popin,fade slide
animation_layers_in_speed 1 10 number 4
animation_layers_in_curve - - enum:default,bezier,spring,linear,ease_out default
animation_layers_out_style - - enum:slide,popin,fade slide
animation_layers_out_speed 1 10 number 1.5
animation_layers_out_curve - - enum:default,bezier,spring,linear,ease_out default
animation_fade_enabled - - boolean true
animation_fade_speed 1 10 number 3.03
animation_fade_curve - - enum:default,bezier,spring,linear,ease_out default
animation_fade_in_enabled - - boolean true
animation_fade_in_speed 1 10 number 1.73
animation_fade_in_curve - - enum:default,bezier,spring,linear,ease_out default
animation_fade_out_enabled - - boolean true
animation_fade_out_speed 1 10 number 1.46
animation_fade_out_curve - - enum:default,bezier,spring,linear,ease_out default
animation_border_enabled - - boolean true
animation_border_speed 1 10 number 5.39
animation_border_curve - - enum:default,bezier,spring,linear,ease_out default
animation_border_angle_enabled - - boolean true
animation_border_angle_speed 1 10 number 5
animation_border_angle_curve - - enum:default,bezier,spring,linear,ease_out default
animation_zoom_enabled - - boolean true
animation_zoom_speed 1 10 number 7
animation_zoom_curve - - enum:default,bezier,spring,linear,ease_out default
animation_monitor_added_enabled - - boolean true
animation_monitor_added_speed 1 10 number 5
animation_monitor_added_curve - - enum:default,bezier,spring,linear,ease_out default
animation_bezier_x1 0 1 number 0.23
animation_bezier_y1 -1 2 number 1
animation_bezier_x2 0 1 number 0.32
animation_bezier_y2 -1 2 number 1
animation_spring_mass 0.5 10 number 1
animation_spring_stiffness 0.5 2000 number 238.1191
animation_spring_dampening 0.5 200 number 24.21279333";

/// The state the effects read and write, resolved once per command.
pub struct Effects {
  paths: SessionPaths,
  store: Option<ConfigStore>,
  commands_dir: PathBuf,
  state_home: PathBuf,
}

/// A failed command: the exit status the shell would have used.
pub type Failure = i32;

/// Directory of the effects state: `ARGVUS_STATE_HOME`, or `<data_home>/state`.
pub(crate) fn effects_state_home(data_home: &Path) -> PathBuf {
  env::var_os("ARGVUS_STATE_HOME")
    .filter(|value| !value.is_empty())
    .map(PathBuf::from)
    .unwrap_or_else(|| data_home.join("state"))
}

/// The generated effects file of `theme` under the effects state directory.
pub(crate) fn theme_effects_path(state_home: &Path, theme: &str) -> PathBuf {
  state_home.join("effects").join(format!("{theme}.conf"))
}

impl Effects {
  pub fn from_environment() -> Option<Self> {
    let paths = SessionPaths::from_environment()?;
    let state_home = effects_state_home(&paths.data_home);
    Some(Self {
      store: ConfigStore::from_environment().ok(),
      commands_dir: env::var_os("ARGVUS_COMMANDS_DIR")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/usr/lib/argvus/commands")),
      state_home,
      paths,
    })
  }

  /// The live copy of a packaged file, as the shell's mutable `paths_config`
  /// returned it. `None` when it cannot be made.
  fn live(&self, relative: &str) -> Option<PathBuf> {
    user_copy::config_path(&self.paths, relative)
  }

  fn active_theme(&self) -> String {
    let theme = read_first_line(&self.paths.config_home.join("argvus/data/.active-theme"));
    if theme.is_empty() {
      "argvus-dark".to_string()
    } else {
      theme
    }
  }

  fn theme_effects_file(&self) -> PathBuf {
    theme_effects_path(&self.state_home, &self.active_theme())
  }

  /// Reads one raw effective value from the store, as `get --effective --raw`.
  fn store_raw(&self, pointer: &str) -> Option<String> {
    let document = self.store.as_ref()?.load_effective().ok()?;
    Some(raw_text(document.get(pointer)))
  }

  /// The store value, when it is usable as a status word.
  fn store_status(&self, pointer: &str) -> Option<&'static str> {
    match self.store_raw(pointer)?.as_str() {
      "true" | "enabled" => Some("enabled"),
      "false" | "disabled" => Some("disabled"),
      _ => None,
    }
  }

  fn set_store(&self, pointer: &str, value: Value) -> io::Result<()> {
    let store = self
      .store
      .as_ref()
      .ok_or_else(|| io::Error::other("argvus-config store unavailable"))?;
    store
      .update(pointer, value)
      .map(|_| ())
      .map_err(io::Error::other)
  }

  fn patch_store(&self, values: &Map<String, Value>) -> io::Result<()> {
    let store = self
      .store
      .as_ref()
      .ok_or_else(|| io::Error::other("argvus-config store unavailable"))?;
    store.patch(values).map(|_| ()).map_err(io::Error::other)
  }

  /// Persists a value when the store is reachable. Failures are tolerated, as
  /// the projection still works without the store.
  fn config_set_value(&self, pointer: &str, value: &str) {
    let parsed = serde_json::from_str(value).unwrap_or_else(|_| Value::String(value.to_string()));
    let _ = self.set_store(pointer, parsed);
  }

  fn config_set_flag(&self, pointer: &str, status: &str) {
    let _ = self.set_store(pointer, Value::Bool(status == "enabled"));
  }

  fn write_flag(&self, pointer: &str, status: &str) -> io::Result<()> {
    self.set_store(pointer, Value::Bool(status == "enabled"))
  }

  fn write_transparency(&self, status: &str) -> io::Result<()> {
    for surface in TRANSPARENCY_SURFACES {
      self.write_flag(&format!("/effects/transparency_{surface}_enabled"), status)?;
    }
    Ok(())
  }

  /// The EFFECTS master switch: global blur and every transparency surface.
  /// Animations are a separate master switch and are never touched here.
  fn write_effects(&self, status: &str) -> io::Result<()> {
    self.write_flag(json_pointers::EFFECTS_BLUR_GLOBAL_ENABLED, status)?;
    self.write_transparency(status)
  }

  /// Asks the compositor-facing services to reload the configuration.
  fn request_config_reload(&self) -> io::Result<()> {
    if env::var("ARGVUS_CONFIG_SERVICE").as_deref() == Ok("1")
      || env::var("ARGVUS_PROJECTING").as_deref() == Ok("1")
    {
      return Ok(());
    }
    if !on_path("systemctl") {
      return Ok(());
    }
    let status = Command::new("systemctl")
      .args(["--user", "reload", "argvus-config.service"])
      .status()?;
    if status.success() {
      Ok(())
    } else {
      Err(io::Error::other("reload failed"))
    }
  }

  // ---- default and canonical status ------------------------------------

  /// Every effect is on until the user turns it off, on virtual machines and
  /// low-power sessions too.
  fn default_status(&self) -> &'static str {
    "enabled"
  }

  fn global_effect_enabled(&self) -> &'static str {
    match self
      .store_raw(json_pointers::EFFECTS_BLUR_GLOBAL_ENABLED)
      .as_deref()
    {
      Some("true" | "enabled") => "enabled",
      Some("false" | "disabled") => "disabled",
      _ => self.default_status(),
    }
  }

  fn canonical_status(&self, component: &str) -> Option<&'static str> {
    self.store_status(&format!("/effects/{component}"))
  }

  fn status(&self, component: &str) -> &'static str {
    match component {
      "blur" => self.global_effect_enabled(),
      "transparency" => self.transparency_status(),
      "effects" => effects_status(self.any_transparency_status(), self.global_effect_enabled()),
      _ => self
        .canonical_status(component)
        .unwrap_or_else(|| self.default_status()),
    }
  }

  /// Transparency is one switch per surface. The aggregate is enabled only
  /// when every surface is enabled.
  fn transparency_status(&self) -> &'static str {
    for surface in TRANSPARENCY_SURFACES {
      if self.effect_enabled(&format!("{surface}.transparency")) != "enabled" {
        return "disabled";
      }
    }
    "enabled"
  }

  /// `enabled` when at least one effect surface keeps its transparency on.
  /// The surfaces are independent, so one is enough for EFFECTS to read as on.
  fn any_transparency_status(&self) -> &'static str {
    let any_enabled = EFFECT_SURFACES
      .iter()
      .any(|surface| self.effect_enabled(&format!("{surface}.transparency")) == "enabled");
    if any_enabled { "enabled" } else { "disabled" }
  }

  // ---- projection file ---------------------------------------------------

  // ---- settings tables -------------------------------------------------

  // ---- surfaces --------------------------------------------------------

  // ---- commands --------------------------------------------------------
}

// ---- helpers --------------------------------------------------------------

fn on_path(program: &str) -> bool {
  env::var_os("PATH")
    .map(|paths| env::split_paths(&paths).any(|dir| dir.join(program).is_file()))
    .unwrap_or(false)
}

/// Entry point of `argvus config effects …`. Returns the exit status.
pub fn run(arguments: &[String]) -> i32 {
  let Some(effects) = Effects::from_environment() else {
    eprintln!("argvus-config: cannot resolve the configuration directory");
    return 1;
  };
  let rest = &arguments[arguments.len().min(1)..];
  let result = match arguments.first().map(String::as_str).unwrap_or("status") {
    "animations" | "transparency" | "blur" | "effects" => {
      let component = arguments[0].as_str();
      effects.component_command(
        component,
        rest.first().map(String::as_str).unwrap_or("status"),
      )
    }
    "transparency-value" | "blur-value" => {
      let kind = arguments[0].trim_end_matches("-value");
      let surface = rest.first().map(String::as_str).unwrap_or("");
      let operation = rest.get(1).map(String::as_str).unwrap_or("get");
      effects.surface_value_command(kind, surface, operation, rest.get(2).map(String::as_str))
    }
    "blur-settings" => match effects.settings_get_or_set(BLUR_SETTINGS, "blur_", rest) {
      Ok(lines) => {
        lines.iter().for_each(|line| println!("{line}"));
        Ok(())
      }
      Err(code) => Err(code),
    },
    "animations-settings" => {
      match effects.settings_get_or_set(ANIMATION_SETTINGS, "animation_", rest) {
        Ok(lines) => {
          lines.iter().for_each(|line| println!("{line}"));
          Ok(())
        }
        Err(code) => Err(code),
      }
    }
    "surface-apply" => effects.surface_apply_command(rest),
    "effect-enabled" => effects.effect_enabled_command(
      rest.first().map(String::as_str).unwrap_or(""),
      rest.get(1).map(String::as_str).unwrap_or(""),
    ),
    "status" => {
      println!("{}", effects.all_effects_status());
      Ok(())
    }
    "enable" | "on" | "enabled" => effects.set_all_status("enabled"),
    "disable" | "off" | "disabled" => effects.set_all_status("disabled"),
    "toggle" => effects.toggle_all(),
    "apply" => effects
      .ensure_theme_effects()
      .map_err(|_| 1)
      .and_then(|()| {
        effects
          .apply_surfaces(effects.status("transparency"), effects.status("animations"))
          .map_err(|_| 1)
      }),
    "runtime" => {
      effects.apply_runtime();
      Ok(())
    }
    _ => {
      effects.usage();
      Err(64)
    }
  };
  match result {
    Ok(()) => 0,
    Err(code) => code,
  }
}

#[cfg(test)]
mod tests;
