//! Visual effects of the desktop: transparency, blur and animations.
//!
//! The state lives in argvus-config (`/effects/...`) and in a per-theme
//! projection file for the values the store does not own. Applying a change
//! rewrites the surfaces that consume it: the Waybar stylesheets, the launcher
//! theme, dunst, the taskbar calendar, superfile and hyprlock. This is the
//! Rust home of the former `effects-toggle.sh`; `argvus effects-toggle` runs
//! `effects`.

use crate::accent::{Rule, apply_rules, edit_file};
use crate::session_paths::SessionPaths;
use crate::user_copy;
use crate::{ConfigStore, text_edit};
use serde_json::{Map, Value};
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Every surface that owns a transparency switch.
const TRANSPARENCY_SURFACES: [&str; 6] = [
  "taskbar",
  "control-panel",
  "widget-telemetry",
  "terminal",
  "launchers",
  "control-center",
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
animation_opening_style - - enum:slide,popin,slidefade,slidevert,slidefadevert,gnomed popin
animation_opening_speed 1 10 number 4.1
animation_opening_curve - - enum:default,bezier,spring,linear,ease_out default
animation_opening_popin 1 100 integer 87
animation_closing_style - - enum:slide,popin,slidefade,slidevert,slidefadevert,gnomed popin
animation_closing_speed 1 10 number 1.49
animation_closing_curve - - enum:default,bezier,spring,linear,ease_out default
animation_closing_popin 1 100 integer 87
animation_moving_enabled - - boolean true
animation_moving_speed 1 10 number 4.79
animation_moving_curve - - enum:default,bezier,spring,linear,ease_out default
animation_switching_style - - enum:slide,fade,slidefade,slidevert,slidefadevert fade
animation_switching_speed 1 10 number 1.94
animation_switching_curve - - enum:default,bezier,spring,linear,ease_out default
animation_workspace_in_style - - enum:slide,fade,slidefade,slidevert,slidefadevert fade
animation_workspace_in_speed 1 10 number 1.21
animation_workspace_in_curve - - enum:default,bezier,spring,linear,ease_out default
animation_workspace_out_style - - enum:slide,fade,slidefade,slidevert,slidefadevert fade
animation_workspace_out_speed 1 10 number 1.94
animation_workspace_out_curve - - enum:default,bezier,spring,linear,ease_out default
animation_special_workspace_style - - enum:slide,fade,slidefade,slidevert,slidefadevert slide
animation_special_workspace_speed 1 10 number 5
animation_special_workspace_curve - - enum:default,bezier,spring,linear,ease_out default
animation_layers_enabled - - boolean true
animation_layers_speed 1 10 number 3.81
animation_layers_curve - - enum:default,bezier,spring,linear,ease_out default
animation_layers_in_style - - enum:slide,popin,fade,slidefade,slidevert,slidefadevert fade
animation_layers_in_speed 1 10 number 4
animation_layers_in_curve - - enum:default,bezier,spring,linear,ease_out default
animation_layers_out_style - - enum:slide,popin,fade,slidefade,slidevert,slidefadevert fade
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
animation_bezier_y1 -2 3 number 1
animation_bezier_x2 0 1 number 0.32
animation_bezier_y2 -2 3 number 1
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

impl Effects {
  pub fn from_environment() -> Option<Self> {
    let paths = SessionPaths::from_environment()?;
    let state_home = env::var_os("ARGVUS_STATE_HOME")
      .filter(|value| !value.is_empty())
      .map(PathBuf::from)
      .unwrap_or_else(|| paths.data_home.join("state"));
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
    self
      .state_home
      .join("effects")
      .join(format!("{}.conf", self.active_theme()))
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

  fn default_status(&self) -> &'static str {
    if env::var("ARGVUS_LOW_POWER").as_deref() == Ok("1") || detect_vm() {
      "disabled"
    } else {
      "enabled"
    }
  }

  fn global_effect_enabled(&self) -> &'static str {
    match self.store_raw("/effects/blur_global_enabled").as_deref() {
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

  // ---- projection file ---------------------------------------------------

  /// Creates the projection file with its defaults and renames the old
  /// launcher keys, as the shell did on every read.
  fn ensure_theme_effects(&self) -> io::Result<()> {
    let file = self.theme_effects_file();
    if let Some(parent) = file.parent() {
      fs::create_dir_all(parent)?;
    }
    if !file.exists() {
      fs::write(&file, "")?;
    }
    let mut text = fs::read_to_string(&file)?;
    if text.lines().any(|line| line.starts_with("launcher.")) {
      text = text
        .split_inclusive('\n')
        .map(|line| match line.strip_prefix("launcher.") {
          Some(rest) => format!("launchers.{rest}"),
          None => line.to_string(),
        })
        .collect();
    }
    const DEFAULTS: [&str; 16] = [
      "taskbar.transparency=50",
      "control-panel.transparency=50",
      "widget-telemetry.transparency=50",
      "taskbar.transparency.enabled=enabled",
      "control-panel.transparency.enabled=enabled",
      "widget-telemetry.transparency.enabled=enabled",
      "taskbar.blur=50",
      "control-panel.blur=50",
      "widget-telemetry.blur=50",
      "taskbar.blur.enabled=enabled",
      "control-panel.blur.enabled=enabled",
      "widget-telemetry.blur.enabled=enabled",
      "terminal.transparency=50",
      "terminal.transparency.enabled=enabled",
      "launchers.transparency=50",
      "launchers.transparency.enabled=enabled",
    ];
    for entry in DEFAULTS {
      let key = entry.split('=').next().unwrap_or("");
      let present = text
        .lines()
        .any(|line| line.starts_with(&format!("{key}=")));
      if !present {
        text.push_str(entry);
        text.push('\n');
      }
    }
    fs::write(&file, text)
  }

  fn projection_value(&self, key: &str) -> Option<String> {
    let text = fs::read_to_string(self.theme_effects_file()).ok()?;
    let prefix = format!("{key}=");
    text
      .lines()
      .find_map(|line| line.strip_prefix(&prefix))
      .map(str::to_string)
  }

  /// The transparency value of a surface, clamped to 0..=100.
  fn effect_value(&self, key: &str) -> u32 {
    let _ = self.ensure_theme_effects();
    let (surface, kind) = key.split_once('.').unwrap_or((key, ""));
    if kind == "transparency" {
      let stored = config_value_pointer(surface)
        .and_then(|pointer| self.store_raw(&pointer))
        .and_then(|value| value.parse::<u32>().ok().filter(|number| *number <= 100));
      if let Some(number) = stored {
        return number;
      }
    }
    let value = self.projection_value(key).unwrap_or_default();
    match value.parse::<u32>() {
      Ok(number) if number <= 100 => number,
      Ok(_) => 100,
      Err(_) => 0,
    }
  }

  /// `enabled` or `disabled` for one surface switch, `name` being `surface.kind`.
  fn effect_enabled(&self, name: &str) -> &'static str {
    let _ = self.ensure_theme_effects();
    let (surface, kind) = name.split_once('.').unwrap_or((name, ""));
    if let Some(pointer) = config_enabled_pointer(surface, kind) {
      match self.store_raw(&pointer).as_deref() {
        Some("true" | "enabled") => return "enabled",
        Some("false" | "disabled") => return "disabled",
        _ => {}
      }
    }
    match self.projection_value(&format!("{name}.enabled")).as_deref() {
      Some("enabled") => "enabled",
      Some("disabled") => "disabled",
      _ => match name {
        "taskbar.transparency"
        | "control-panel.transparency"
        | "widget-telemetry.transparency"
        | "taskbar.blur"
        | "control-panel.blur"
        | "widget-telemetry.blur" => "enabled",
        // A per-surface transparency switch must not fall back to the
        // aggregate status, which would call back into this function.
        _ if kind == "transparency" => self.default_status(),
        _ => self.status(kind),
      },
    }
  }

  /// Replaces or appends `key=value` in the projection file.
  fn set_projection(&self, key: &str, value: &str) -> io::Result<()> {
    self.set_projection_pairs(&[(key.to_string(), value.to_string())])
  }

  fn set_projection_pairs(&self, pairs: &[(String, String)]) -> io::Result<()> {
    self.ensure_theme_effects()?;
    let file = self.theme_effects_file();
    let text = fs::read_to_string(&file)?;
    let mut seen = vec![false; pairs.len()];
    let mut output = String::new();
    for line in text.split_inclusive('\n') {
      let content = line.strip_suffix('\n').unwrap_or(line);
      let name = content.split('=').next().unwrap_or("");
      match pairs.iter().position(|(key, _)| key == name) {
        Some(slot) => {
          seen[slot] = true;
          output.push_str(&format!("{}={}\n", pairs[slot].0, pairs[slot].1));
        }
        None => {
          output.push_str(line);
          if !line.ends_with('\n') {
            output.push('\n');
          }
        }
      }
    }
    for (slot, (key, value)) in pairs.iter().enumerate() {
      if !seen[slot] {
        output.push_str(&format!("{key}={value}\n"));
      }
    }
    let temporary = file.with_extension(format!("tmp.{}", std::process::id()));
    fs::write(&temporary, output)?;
    fs::rename(&temporary, &file)
  }

  fn set_effect_value(&self, key: &str, value: &str) -> Result<(), Failure> {
    let number: u32 = value.parse().map_err(|_| 64)?;
    if number > 100 || value.starts_with('+') {
      return Err(64);
    }
    self.set_projection(key, value).map_err(|_| 1)?;
    let (surface, kind) = key.split_once('.').unwrap_or((key, ""));
    if let (true, Some(pointer)) = (kind == "transparency", config_value_pointer(surface)) {
      self.config_set_value(&pointer, value);
    }
    Ok(())
  }

  // ---- settings tables -------------------------------------------------

  fn settings_read(&self, table: &str, key: &str) -> Option<String> {
    let row = table_row(table, key)?;
    let value = self
      .store_raw(&format!("/effects/{key}"))
      .unwrap_or_else(|| "null".to_string());
    if settings_valid(&row.kind, &row.minimum, &row.maximum, &value) {
      Some(value)
    } else {
      Some(row.default)
    }
  }

  /// `get`: one `name=value` line for every row of the table.
  fn settings_get(&self, table: &str, prefix: &str) -> Vec<String> {
    table
      .lines()
      .filter_map(|line| line.split_whitespace().next())
      .map(|key| {
        format!(
          "{}={}",
          key.strip_prefix(prefix).unwrap_or(key),
          self.settings_read(table, key).unwrap_or_default()
        )
      })
      .collect()
  }

  /// `set name=value ...`: validates every pair, then writes them in one patch.
  fn settings_set(
    &self,
    table: &str,
    prefix: &str,
    pairs: &[String],
  ) -> Result<Vec<String>, Failure> {
    if pairs.is_empty() || self.store.is_none() {
      return Err(64);
    }
    let mut patch = Map::new();
    for argument in pairs {
      let (name, value) = argument.split_once('=').ok_or(64)?;
      let key = format!("{prefix}{name}");
      let row = table_row(table, &key).ok_or(64)?;
      if !settings_valid(&row.kind, &row.minimum, &row.maximum, value) {
        return Err(64);
      }
      let literal = if row.kind.starts_with("enum:") {
        Value::String(value.to_string())
      } else {
        serde_json::from_str(value).map_err(|_| 64)?
      };
      patch.insert(format!("/effects/{key}"), literal);
    }
    self.patch_store(&patch).map_err(|_| 1)?;
    self.request_config_reload().map_err(|_| 1)?;
    let output = self.settings_get(table, prefix);
    spawn_runtime();
    Ok(output)
  }

  // ---- surfaces --------------------------------------------------------

  fn apply_taskbar_surface(&self, animations: &str) -> io::Result<()> {
    let css = self.live("taskbar/config/argvus-taskbar.css");
    ensure_waybar_namespace(
      self.live("taskbar/config/argvus-taskbar.jsonc").as_deref(),
      "argvus-taskbar",
    )?;
    let Some(css) = css.filter(|path| path.is_file()) else {
      return Ok(());
    };
    let mut text = fs::read_to_string(&css)?;
    if !text
      .lines()
      .any(|line| line.starts_with("@define-color th-background-effective "))
    {
      text = insert_after(
        &text,
        "@define-color transparent transparent;",
        "@define-color th-background-effective @th-background-rgba;",
      );
      text = replace_in_lines(
        &text,
        "background: @th-background-rgba;",
        "background: @th-background-effective;",
      );
    }
    if !text
      .lines()
      .any(|line| line.starts_with("@define-color th-mpris-bg-effective "))
    {
      text = insert_after(
        &text,
        "@define-color th-background-effective ",
        "@define-color th-mpris-bg-effective @th-mpris-bg;",
      );
      text = replace_in_lines(
        &text,
        "background: @th-mpris-bg;",
        "background: @th-mpris-bg-effective;",
      );
    }
    if self.effect_enabled("taskbar.transparency") == "disabled" {
      text = set_define(&text, "th-background-effective", "@th-background;");
      text = set_define(&text, "th-mpris-bg-effective", "@th-right1-bg;");
    } else {
      let opacity = opacity_factor(self.effect_value("taskbar.transparency"));
      text = set_define(
        &text,
        "th-background-effective",
        &format!("alpha(@th-background, {opacity});"),
      );
      text = set_define(
        &text,
        "th-mpris-bg-effective",
        &format!("alpha(@th-mpris-bg, {opacity});"),
      );
    }
    const MARK_START: &str = "/* ARGVUS animations disabled */";
    const MARK_END: &str = "/* ARGVUS animations disabled end */";
    if animations == "disabled" {
      if !text.lines().any(|line| line == MARK_START) {
        text.push_str(&format!(
                    "{MARK_START}\n#workspaces button, #right-2, #custom-expand-icon {{\n  transition: none;\n}}\n#mpris, #custom-spotify-mpris, #custom-recording.recording {{\n  animation: none;\n}}\n{MARK_END}\n"
                ));
      }
    } else {
      text = delete_range(&text, MARK_START, MARK_END);
    }
    fs::write(&css, text)
  }

  fn apply_widget_telemetry_surface(&self) -> io::Result<()> {
    let css = self.live("widget-telemetry/config/argvus-widget-telemetry.css");
    ensure_waybar_namespace(
      self
        .live("widget-telemetry/config/argvus-widget-telemetry.jsonc")
        .as_deref(),
      "argvus-widget-telemetry",
    )?;
    let Some(css) = css.filter(|path| path.is_file()) else {
      return Ok(());
    };
    let mut text = fs::read_to_string(&css)?;
    if !text
      .lines()
      .any(|line| line.starts_with("@define-color th-window-bg-effective "))
    {
      text = insert_after(
        &text,
        "/*  Waybar — Painel vertical",
        "@define-color th-window-bg-effective @th-window-bg-rgba;",
      );
      text = replace_in_lines(
        &text,
        "background: @th-window-bg;",
        "background: @th-window-bg-effective;",
      );
    }
    if self.effect_enabled("widget-telemetry.transparency") == "disabled" {
      text = set_define(&text, "th-window-bg-effective", "@th-window-bg;");
    } else {
      let opacity = opacity_factor(self.effect_value("widget-telemetry.transparency"));
      text = set_define(
        &text,
        "th-window-bg-effective",
        &format!("alpha(@th-window-bg, {opacity});"),
      );
    }
    fs::write(&css, text)
  }

  fn apply_launcher_surface(&self) -> io::Result<()> {
    let Some(theme) = self
      .live("launcher/config/theme.rasi")
      .filter(|path| path.is_file())
    else {
      return Ok(());
    };
    let background = if self.effect_enabled("launchers.transparency") == "disabled" {
      theme_background(&self.active_theme()).to_string()
    } else {
      // The alpha is the inverse of the transparency, rounded as the shell
      // rounded it: the opacity is printed with two decimals first.
      let opacity: f64 = opacity_factor(self.effect_value("launchers.transparency"))
        .parse()
        .unwrap_or(1.0);
      let text = fs::read_to_string(&theme)?;
      let import = text
        .lines()
        .find_map(|line| {
          line
            .strip_prefix("@import \"")
            .and_then(|rest| rest.split('"').next())
        })
        .map(str::to_string);
      let color = import
        .and_then(|path| fs::read_to_string(path).ok())
        .and_then(|imported| {
          imported.lines().find_map(|line| {
            let rest = line.trim_start().strip_prefix("th-bg:")?.trim_start();
            rest.strip_suffix(';').map(str::to_string)
          })
        })
        .unwrap_or_default();
      let channels: Vec<String> = color
        .replace("rgba", "")
        .replace("rgb", "")
        .replace(['(', ')', '%'], "")
        .split(',')
        .flat_map(str::split_whitespace)
        .map(str::to_string)
        .collect();
      match channels.len() {
        3 | 4 => {
          let alpha = (opacity * 100.0).round() as i64;
          format!(
            "rgba({}, {}, {}, {}%)",
            channels[0], channels[1], channels[2], alpha
          )
        }
        _ => "@th-bg".to_string(),
      }
    };
    let rules = [Rule::literal(
      r"^[[:space:]]*bg:[[:space:]].*;",
      format!("    bg:               {background};"),
    )];
    edit_file(Some(&theme), |text| apply_rules(text, &rules))
  }

  fn apply_dunst_surface(&self, status: &str) -> io::Result<()> {
    let Some(dunstrc) = self
      .live("notifications/config/dunstrc")
      .filter(|path| path.is_file())
    else {
      return Ok(());
    };
    let backup = self.state_home.join("effects-dunst-transparency");
    let text = fs::read_to_string(&dunstrc)?;
    if status == "disabled" {
      if !backup.is_file() {
        let current = text
          .lines()
          .find_map(|line| {
            let rest = line.trim_start().strip_prefix("transparency")?.trim_start();
            let digits: String = rest
              .strip_prefix('=')?
              .trim_start()
              .chars()
              .take_while(char::is_ascii_digit)
              .collect();
            (!digits.is_empty()).then_some(digits)
          })
          .filter(|digits| digits.chars().all(|digit| digit.is_ascii_digit()))
          .unwrap_or_else(|| "10".to_string());
        if let Some(parent) = backup.parent() {
          fs::create_dir_all(parent)?;
        }
        fs::write(&backup, format!("{current}\n"))?;
      }
      let rules = [Rule::literal(
        r"^[[:space:]]*transparency[[:space:]]*=.*",
        "    transparency = 0".to_string(),
      )];
      return fs::write(&dunstrc, apply_rules(&text, &rules));
    }
    let current = fs::read_to_string(&backup)
      .ok()
      .and_then(|value| value.lines().next().map(str::to_string))
      .filter(|value| !value.is_empty() && value.chars().all(|digit| digit.is_ascii_digit()))
      .unwrap_or_else(|| "10".to_string());
    let rules = [Rule::literal(
      r"^[[:space:]]*transparency[[:space:]]*=.*",
      format!("    transparency = {current}"),
    )];
    fs::write(&dunstrc, apply_rules(&text, &rules))?;
    let _ = fs::remove_file(&backup);
    Ok(())
  }

  fn apply_calendar_surface(&self, status: &str) -> io::Result<()> {
    let cache = env::var_os("XDG_CACHE_HOME")
      .filter(|value| !value.is_empty())
      .map(PathBuf::from)
      .unwrap_or_else(|| PathBuf::from(env::var_os("HOME").unwrap_or_default()).join(".cache"));
    let css = cache.join("argvus-taskbar-calendar/theme.css");
    if !css.is_file() {
      return Ok(());
    }
    let mut text = fs::read_to_string(&css)?;
    if !text
      .lines()
      .any(|line| line.starts_with("@define-color argvus_bg_effective "))
    {
      text = insert_after(
        &text,
        "@define-color argvus_bg_alpha ",
        "@define-color argvus_bg_effective @argvus_bg_alpha;",
      );
      text = replace_in_lines(
        &text,
        "background: @argvus_bg_alpha;",
        "background: @argvus_bg_effective;",
      );
    }
    let value = if status == "disabled" {
      "@argvus_bg;"
    } else {
      "@argvus_bg_alpha;"
    };
    text = set_define(&text, "argvus_bg_effective", value);
    fs::write(&css, text)
  }

  fn apply_superfile_surface(&self, status: &str) -> io::Result<()> {
    let Some(config) = self
      .live("app-profiles/config/superfile/config.toml")
      .filter(|path| path.is_file())
    else {
      return Ok(());
    };
    let transparent = if status == "disabled" {
      "false"
    } else {
      "true"
    };
    let text = fs::read_to_string(&config)?;
    if text.lines().any(|line| {
      line.trim_start().starts_with("transparent_background")
        && line.trim_start()["transparent_background".len()..]
          .trim_start()
          .starts_with('=')
    }) {
      let rules = [Rule::literal(
        r"^[[:space:]]*transparent_background[[:space:]]*=.*",
        format!("transparent_background = {transparent}"),
      )];
      return fs::write(&config, apply_rules(&text, &rules));
    }
    let mut output = text;
    output.push_str(&format!("\ntransparent_background = {transparent}\n"));
    fs::write(&config, output)
  }

  /// Runs the hyprlock theme helper, when the package has one.
  fn apply_hyprlock_effects(&self) {
    if let Some(script) = self
      .live("lock/sh/hyprlock-theme.sh")
      .filter(|path| path.is_file())
    {
      let _ = Command::new("sh")
        .arg(script)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    }
  }

  fn apply_surfaces(&self, transparency: &str, animations: &str) -> io::Result<()> {
    self.apply_taskbar_surface(animations)?;
    self.apply_widget_telemetry_surface()?;
    self.apply_launcher_surface()?;
    self.apply_dunst_surface(transparency)?;
    self.apply_calendar_surface(transparency)?;
    self.apply_superfile_surface(transparency)?;
    self.apply_hyprlock_effects();
    Ok(())
  }

  /// The services and palettes that follow an effects change. It runs in the
  /// background after the command answered, as the shell did.
  fn apply_runtime(&self) {
    if on_path("argvus") {
      let _ = Command::new("argvus")
        .args(["config", "hyprland", "reload"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    }
    let sessionctl = self.commands_dir.join("sessionctl");
    if sessionctl.is_file() {
      for component in ["waybar", "dunst", "control-panel"] {
        let _ = Command::new(&sessionctl)
          .args(["restart", component])
          .stdout(Stdio::null())
          .stderr(Stdio::null())
          .status();
      }
    }
    let calendar = self.commands_dir.join("taskbar-calendar");
    if calendar.is_file() {
      let _ = Command::new(&calendar)
        .arg("reload")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    }
    if on_path("argvus") {
      let _ = Command::new("argvus")
        .args(["terminal", "--apply", &self.active_theme()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
      signal_kitty();
    }
  }

  fn rematerialize_control_center(&self) {
    // The launcher is the `argvus` command itself; without it nothing is rewritten.
    {
      let _ = Command::new("argvus")
        .args([
          "tui-terminal",
          "--materialize-kitty",
          "--profile",
          "control-center",
          "--class",
          "argvus-control-center",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    }
    signal_kitty();
  }

  // ---- commands --------------------------------------------------------

  fn set_status(&self, component: &str, status: &str) -> Result<(), Failure> {
    self.require_store()?;
    match component {
      "blur" => self
        .write_flag("/effects/blur_global_enabled", status)
        .map_err(|_| 1)?,
      "animations" => self
        .write_flag("/effects/animations", status)
        .map_err(|_| 1)?,
      "transparency" => self.write_transparency(status).map_err(|_| 1)?,
      _ => return Err(64),
    }
    self.request_config_reload().map_err(|_| 1)?;
    let transparency = self.status("transparency");
    let animations = self.status("animations");
    self
      .apply_surfaces(transparency, animations)
      .map_err(|_| 1)?;
    println!("{status}");
    spawn_runtime();
    Ok(())
  }

  fn set_all_status(&self, status: &str) -> Result<(), Failure> {
    self.require_store()?;
    self
      .write_flag("/effects/animations", status)
      .map_err(|_| 1)?;
    self
      .write_flag("/effects/blur_global_enabled", status)
      .map_err(|_| 1)?;
    self.write_transparency(status).map_err(|_| 1)?;
    self.request_config_reload().map_err(|_| 1)?;
    self.apply_surfaces(status, status).map_err(|_| 1)?;
    println!("{status}");
    spawn_runtime();
    Ok(())
  }

  fn require_store(&self) -> Result<(), Failure> {
    if self.store.is_some() {
      Ok(())
    } else {
      eprintln!("argvus is required to change effects");
      Err(1)
    }
  }

  fn toggle_component(&self, component: &str) -> Result<(), Failure> {
    match self.status(component) {
      "enabled" => self.set_status(component, "disabled"),
      _ => self.set_status(component, "enabled"),
    }
  }

  fn component_command(&self, component: &str, operation: &str) -> Result<(), Failure> {
    match operation {
      "status" => {
        println!("{}", self.status(component));
        Ok(())
      }
      "enable" | "on" | "enabled" => self.set_status(component, "enabled"),
      "disable" | "off" | "disabled" => self.set_status(component, "disabled"),
      "toggle" => self.toggle_component(component),
      _ => Err(64),
    }
  }

  /// `transparency-value|blur-value SURFACE get|set [VALUE]`.
  fn surface_value_command(
    &self,
    kind: &str,
    surface: &str,
    operation: &str,
    value: Option<&str>,
  ) -> Result<(), Failure> {
    if !TRANSPARENCY_SURFACES.contains(&surface) {
      return Err(64);
    }
    if surface == "control-center" {
      if kind != "transparency" {
        return Err(64);
      }
      return match operation {
        "get" => {
          let raw = self
            .store_raw("/effects/transparency_control-center_value")
            .unwrap_or_default();
          match raw.parse::<u32>() {
            Ok(number) if number <= 100 && !raw.starts_with('+') => {
              println!("{number}")
            }
            _ => println!("50"),
          }
          Ok(())
        }
        "set" => {
          let value = value.ok_or(64)?;
          let number: u32 = value.parse().map_err(|_| 64)?;
          if number > 100 || value.starts_with('+') {
            return Err(64);
          }
          self.require_store().map_err(|_| 64)?;
          self
            .set_store(
              "/effects/transparency_control-center_value",
              Value::from(number),
            )
            .map_err(|_| 1)?;
          self.request_config_reload().map_err(|_| 1)?;
          self.rematerialize_control_center();
          println!("{number}");
          Ok(())
        }
        _ => Err(64),
      };
    }
    let key = format!("{surface}.{kind}");
    match operation {
      "get" => {
        println!("{}", self.effect_value(&key));
        Ok(())
      }
      "set" => {
        self.set_effect_value(&key, value.unwrap_or(""))?;
        self.request_config_reload().map_err(|_| 1)?;
        let transparency = self.status("transparency");
        let animations = self.status("animations");
        self
          .apply_surfaces(transparency, animations)
          .map_err(|_| 1)?;
        println!("{}", self.effect_value(&key));
        spawn_runtime();
        Ok(())
      }
      _ => Err(64),
    }
  }

  /// `surface-apply SURFACE TRANSPARENCY_ENABLED TRANSPARENCY_VALUE BLUR_ENABLED BLUR_VALUE`.
  fn surface_apply_command(&self, arguments: &[String]) -> Result<(), Failure> {
    let [
      surface,
      transparency_enabled,
      transparency_value,
      blur_enabled,
      blur_value,
    ] = arguments
    else {
      return Err(64);
    };
    if !TRANSPARENCY_SURFACES.contains(&surface.as_str()) {
      return Err(64);
    }
    for state in [transparency_enabled, blur_enabled] {
      if state != "enabled" && state != "disabled" {
        return Err(64);
      }
    }
    for number in [transparency_value, blur_value] {
      if number.is_empty() || !number.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(64);
      }
      if number.parse::<u32>().map_err(|_| 64)? > 100 {
        return Err(64);
      }
    }
    let flag = |status: &str| Value::Bool(status == "enabled");
    match surface.as_str() {
      "control-center" => {
        self.require_store()?;
        self
          .set_store(
            "/effects/transparency_control-center_enabled",
            flag(transparency_enabled),
          )
          .map_err(|_| 1)?;
        self
          .set_store(
            "/effects/transparency_control-center_value",
            Value::from(transparency_value.parse::<u32>().unwrap_or(0)),
          )
          .map_err(|_| 1)?;
        self
          .set_store("/effects/blur_control-center_enabled", flag(blur_enabled))
          .map_err(|_| 1)?;
        self.request_config_reload().map_err(|_| 1)?;
        self.rematerialize_control_center();
        println!("{transparency_value}");
        Ok(())
      }
      "terminal" => {
        self.require_store()?;
        self
          .set_store(
            "/effects/transparency_terminal_enabled",
            flag(transparency_enabled),
          )
          .map_err(|_| 1)?;
        self
          .set_store(
            "/effects/transparency_terminal_value",
            Value::from(transparency_value.parse::<u32>().unwrap_or(0)),
          )
          .map_err(|_| 1)?;
        if on_path("argvus") {
          let _ = Command::new("argvus")
            .args(["terminal", "--apply", &self.active_theme()])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        }
        signal_kitty();
        self.request_config_reload().map_err(|_| 1)?;
        println!("{transparency_value}");
        Ok(())
      }
      "launchers" => {
        self.set_launcher_projection(transparency_enabled, transparency_value)?;
        if self.store.is_some() {
          self.config_set_flag(
            "/effects/transparency_launchers_enabled",
            transparency_enabled,
          );
          self.config_set_value("/effects/transparency_launchers_value", transparency_value);
        }
        self.request_config_reload().map_err(|_| 1)?;
        self.apply_launcher_surface().map_err(|_| 1)?;
        println!("{transparency_value}");
        Ok(())
      }
      _ => {
        self.ensure_theme_effects().map_err(|_| 1)?;
        self
          .set_projection_pairs(&[
            (
              format!("{surface}.transparency.enabled"),
              transparency_enabled.clone(),
            ),
            (
              format!("{surface}.transparency"),
              transparency_value.clone(),
            ),
            (format!("{surface}.blur.enabled"), blur_enabled.clone()),
            (format!("{surface}.blur"), blur_value.clone()),
          ])
          .map_err(|_| 1)?;
        self.config_set_flag(
          &format!("/effects/transparency_{surface}_enabled"),
          transparency_enabled,
        );
        self.config_set_value(
          &format!("/effects/transparency_{surface}_value"),
          transparency_value,
        );
        self.request_config_reload().map_err(|_| 1)?;
        self
          .apply_taskbar_surface(self.status("animations"))
          .map_err(|_| 1)?;
        self.apply_widget_telemetry_surface().map_err(|_| 1)?;
        if on_path("argvus") {
          let _ = Command::new("argvus")
            .args(["config", "hyprland", "reload"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        }
        let sessionctl = self.commands_dir.join("sessionctl");
        let restart = match surface.as_str() {
          "taskbar" => Some("waybar"),
          "widget-telemetry" => Some("widget-telemetry"),
          "control-panel" => Some("control-panel"),
          _ => None,
        };
        if let (Some(component), true) = (restart, sessionctl.is_file()) {
          let _ = Command::new(&sessionctl)
            .args(["restart", component])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        }
        Ok(())
      }
    }
  }

  /// The launcher keeps its switch and value in the projection; this is the
  /// same write the shell did for it.
  fn set_launcher_projection(&self, enabled: &str, value: &str) -> Result<(), Failure> {
    self.ensure_theme_effects().map_err(|_| 1)?;
    self
      .set_projection_pairs(&[
        (
          "launchers.transparency.enabled".to_string(),
          enabled.to_string(),
        ),
        ("launchers.transparency".to_string(), value.to_string()),
      ])
      .map_err(|_| 1)
  }

  /// `effect-enabled SURFACE NAME`, for the Control Center.
  fn effect_enabled_command(&self, surface: &str, name: &str) -> Result<(), Failure> {
    match surface {
      "taskbar" | "control-panel" | "widget-telemetry" | "terminal" | "launchers" => {
        println!("{}", self.effect_enabled(&format!("{surface}.{name}")));
        Ok(())
      }
      "control-center" => {
        let raw = self
          .store_raw(&format!("/effects/{name}_control-center_enabled"))
          .unwrap_or_default();
        let status = match raw.as_str() {
          "true" | "enabled" => "enabled",
          "false" | "disabled" => "disabled",
          _ => "enabled",
        };
        println!("{status}");
        Ok(())
      }
      _ => Err(64),
    }
  }
}

// ---- helpers --------------------------------------------------------------

struct Row {
  minimum: String,
  maximum: String,
  kind: String,
  default: String,
}

fn table_row(table: &str, key: &str) -> Option<Row> {
  table.lines().find_map(|line| {
    let fields: Vec<&str> = line.split_whitespace().collect();
    (fields.first() == Some(&key)).then(|| Row {
      minimum: fields.get(1).unwrap_or(&"-").to_string(),
      maximum: fields.get(2).unwrap_or(&"-").to_string(),
      kind: fields.get(3).unwrap_or(&"").to_string(),
      default: fields.get(4).unwrap_or(&"").to_string(),
    })
  })
}

/// Checks a value against its row. The rules are the shell's, including its
/// leading-zero test, which also refuses a lone `0`.
pub fn settings_valid(kind: &str, minimum: &str, maximum: &str, value: &str) -> bool {
  match kind {
    "boolean" => return value == "true" || value == "false",
    _ if kind.starts_with("enum:") => {
      return kind["enum:".len()..]
        .split(',')
        .any(|choice| choice == value);
    }
    _ => {}
  }
  if is_leading_zero(value) {
    return false;
  }
  let digits = value.strip_prefix('-').unwrap_or(value);
  let shape_ok = match kind {
    "integer" => !digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit()),
    "number" => {
      !digits.is_empty()
        && digits
          .bytes()
          .all(|byte| byte.is_ascii_digit() || byte == b'.')
        && !digits.starts_with('.')
        && !digits.ends_with('.')
        && digits.matches('.').count() <= 1
    }
    _ => false,
  };
  if !shape_ok {
    return false;
  }
  match (
    value.parse::<f64>(),
    minimum.parse::<f64>(),
    maximum.parse::<f64>(),
  ) {
    (Ok(number), Ok(low), Ok(high)) => number >= low && number <= high,
    _ => false,
  }
}

/// `0[0-9]*` or `-0[0-9]*`, the shell's pattern: it matches a lone `0` too.
fn is_leading_zero(value: &str) -> bool {
  let rest = match value.strip_prefix("-0") {
    Some(rest) => rest,
    None => match value.strip_prefix('0') {
      Some(rest) => rest,
      None => return false,
    },
  };
  rest.bytes().all(|byte| byte.is_ascii_digit())
}

fn raw_text(value: Option<&Value>) -> String {
  match value {
    Some(Value::String(text)) => text.clone(),
    Some(other) => other.to_string(),
    None => "null".to_string(),
  }
}

fn config_value_pointer(surface: &str) -> Option<String> {
  match surface {
    "taskbar" | "control-panel" | "widget-telemetry" | "terminal" | "launchers" => {
      Some(format!("/effects/transparency_{surface}_value"))
    }
    _ => None,
  }
}

fn config_enabled_pointer(surface: &str, kind: &str) -> Option<String> {
  match (surface, kind) {
    (
      "taskbar" | "control-panel" | "widget-telemetry" | "terminal" | "launchers",
      "transparency",
    ) => Some(format!("/effects/transparency_{surface}_enabled")),
    _ => None,
  }
}

fn opacity_factor(transparency: u32) -> String {
  format!("{:.2}", 1.0 - f64::from(transparency) / 100.0)
}

fn theme_background(theme: &str) -> &'static str {
  match theme {
    "dracula" | "dracula-float" => "#282A36",
    "slate-dark" | "slate-dark-float" => "#2f3541",
    "argvus-light" | "argvus-light-float" => "#f7f7f7",
    "github-light" | "github-light-float" => "#ffffff",
    "one-light" | "one-light-float" => "#FAFAFA",
    "everforest-light" | "everforest-light-float" => "#FDF6E3",
    "solarized-light" | "solarized-light-float" => "#FDF6E3",
    "catppuccin-latte" | "catppuccin-latte-float" => "#EFF1F5",
    "gruvbox-light" | "gruvbox-light-float" => "#FBF1C7",
    "universe" | "universe-float" => "#000000",
    "gruvbox-high-dark" | "gruvbox-high-dark-float" | "gruvbox-dark" | "gruvbox-dark-float" => {
      "#282828"
    }
    "solitude" | "solitude-float" => "#101315",
    "sunset" | "sunset-float" => "#0F0F0F",
    "matrix" | "matrix-float" => "#0B0C16",
    "monokai-dark" | "monokai-dark-float" => "#2D2A2E",
    "one-dark" | "one-dark-float" => "#282C34",
    _ => "#111316",
  }
}

/// `sed "s|^\(key\)…"` on every `@define-color NAME …` line.
fn set_define(text: &str, name: &str, value: &str) -> String {
  let rules = [Rule::literal(
    &format!(r"^@define-color {name} .*"),
    format!("@define-color {name} {value}"),
  )];
  apply_rules(text, &rules)
}

fn replace_in_lines(text: &str, from: &str, to: &str) -> String {
  let rules = [Rule::literal(&regex::escape(from), to.to_string())];
  apply_rules(text, &rules)
}

/// `sed '/exact/a text'`: inserts `insert` after each line that starts with
/// `after`, the first match on the line counting as in `sed`.
fn insert_after(text: &str, after: &str, insert: &str) -> String {
  let mut output = String::new();
  for line in text.split_inclusive('\n') {
    output.push_str(line);
    let content = line.strip_suffix('\n').unwrap_or(line);
    if content.starts_with(after) {
      if !line.ends_with('\n') {
        output.push('\n');
      }
      output.push_str(insert);
      output.push('\n');
    }
  }
  output
}

/// `sed '/start/,/end/d'`: removes each block from a `start` line through the
/// next `end` line.
fn delete_range(text: &str, start: &str, end: &str) -> String {
  let mut inside = false;
  let mut output = String::new();
  for line in text.split_inclusive('\n') {
    let content = line.strip_suffix('\n').unwrap_or(line);
    if !inside && content == start {
      inside = true;
    }
    if inside {
      if content == end {
        inside = false;
      }
      continue;
    }
    output.push_str(line);
  }
  output
}

/// Sets the `"name"` field of a Waybar JSONC file, or adds it after the first line.
fn ensure_waybar_namespace(path: Option<&Path>, namespace: &str) -> io::Result<()> {
  let Some(path) = path.filter(|path| path.is_file()) else {
    return Ok(());
  };
  let text = fs::read_to_string(path)?;
  let has_name = text.lines().any(|line| {
    line.trim_start().starts_with("\"name\"")
      && line.trim_start()["\"name\"".len()..]
        .trim_start()
        .starts_with(':')
  });
  let edited = if has_name {
    let rules = [Rule::literal(
      r#"^[[:space:]]*"name"[[:space:]]*:.*"#,
      format!("  \"name\": \"{namespace}\","),
    )];
    apply_rules(&text, &rules)
  } else {
    insert_after_first_line(&text, &format!("  \"name\": \"{namespace}\","))
  };
  if edited != text {
    text_edit::replace_file(path, &edited)?;
  }
  Ok(())
}

fn insert_after_first_line(text: &str, insert: &str) -> String {
  let mut lines = text.split_inclusive('\n');
  let mut output = String::new();
  if let Some(first) = lines.next() {
    output.push_str(first);
    if !first.ends_with('\n') {
      output.push('\n');
    }
  }
  output.push_str(insert);
  output.push('\n');
  for line in lines {
    output.push_str(line);
  }
  output
}

fn read_first_line(path: &Path) -> String {
  fs::read_to_string(path)
    .ok()
    .and_then(|text| text.lines().next().map(|line| line.trim().to_string()))
    .unwrap_or_default()
}

/// `systemd-detect-virt --vm` succeeds only inside a virtual machine.
fn detect_vm() -> bool {
  if !on_path("systemd-detect-virt") {
    return false;
  }
  Command::new("systemd-detect-virt")
    .arg("--vm")
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .status()
    .map(|status| status.success())
    .unwrap_or(false)
}

fn on_path(program: &str) -> bool {
  env::var_os("PATH")
    .map(|paths| env::split_paths(&paths).any(|dir| dir.join(program).is_file()))
    .unwrap_or(false)
}

/// Asks running kitty instances to reload their generated config.
fn signal_kitty() {
  let Ok(output) = Command::new("pgrep")
    .args(["-x", "kitty"])
    .stderr(Stdio::null())
    .output()
  else {
    return;
  };
  for pid in String::from_utf8_lossy(&output.stdout).lines() {
    if let Ok(pid) = pid.trim().parse::<i32>() {
      // SAFETY: SIGUSR1 to a pid that pgrep just reported.
      unsafe {
        libc::kill(pid, libc::SIGUSR1);
      }
    }
  }
}

/// Runs the runtime refresh in a detached process, so the command answers
/// before the components restart.
fn spawn_runtime() {
  if let Ok(executable) = env::current_exe() {
    let _ = Command::new(executable)
      .args(["config", "effects", "runtime"])
      .stdin(Stdio::null())
      .stdout(Stdio::null())
      .stderr(Stdio::null())
      .spawn();
  }
}

/// Entry point of `argvus config effects …`. Returns the exit status.
pub fn run(arguments: &[String]) -> i32 {
  let Some(effects) = Effects::from_environment() else {
    eprintln!("argvus-config: cannot resolve the configuration directory");
    return 1;
  };
  let rest = &arguments[arguments.len().min(1)..];
  let result = match arguments.first().map(String::as_str).unwrap_or("status") {
    "animations" | "transparency" | "blur" => {
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
      let on = effects.status("animations") == "enabled"
        && effects.status("transparency") == "enabled"
        && effects.status("blur") == "enabled";
      println!("{}", if on { "enabled" } else { "disabled" });
      Ok(())
    }
    "enable" | "on" | "enabled" => effects.set_all_status("enabled"),
    "disable" | "off" | "disabled" => effects.set_all_status("disabled"),
    "toggle" => effects.toggle_component("animations"),
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

impl Effects {
  /// The usage line of the effects switch, translated when argvus-i18n is installed.
  fn usage(&self) {
    if on_path("argvus") {
      let _ = Command::new("argvus")
        .args([
          "i18n",
          "get",
          "session",
          "usage.effects",
          "command=argvus effects-toggle",
        ])
        .stderr(Stdio::inherit())
        .status();
    } else {
      eprintln!("usage.effects");
    }
  }

  /// `get`, or `set name=value ...` followed by the new values.
  fn settings_get_or_set(
    &self,
    table: &str,
    prefix: &str,
    rest: &[String],
  ) -> Result<Vec<String>, Failure> {
    match rest.first().map(String::as_str).unwrap_or("get") {
      "get" => Ok(self.settings_get(table, prefix)),
      "set" => self.settings_set(table, prefix, &rest[1..]),
      _ => Err(64),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn booleans_and_enums_accept_only_their_choices() {
    assert!(settings_valid("boolean", "-", "-", "true"));
    assert!(!settings_valid("boolean", "-", "-", "yes"));
    assert!(settings_valid("enum:slide,popin", "-", "-", "popin"));
    assert!(!settings_valid("enum:slide,popin", "-", "-", "pop"));
    assert!(!settings_valid("enum:slide,popin", "-", "-", ""));
  }

  #[test]
  fn numbers_keep_their_shape_and_bounds() {
    assert!(settings_valid("integer", "1", "64", "10"));
    assert!(!settings_valid("integer", "1", "64", "99"));
    assert!(!settings_valid("integer", "1", "64", "4.5"));
    assert!(settings_valid("number", "0", "2", "1.25"));
    assert!(!settings_valid("number", "0", "2", "1.2.3"));
    assert!(!settings_valid("number", "0", "2", "."));
    assert!(settings_valid("number", "-2", "3", "-1.5"));
  }

  #[test]
  fn the_leading_zero_test_refuses_a_lone_zero_as_the_shell_did() {
    assert!(is_leading_zero("0"));
    assert!(is_leading_zero("007"));
    assert!(is_leading_zero("-0"));
    assert!(!is_leading_zero("0.5"));
    assert!(!settings_valid("number", "0", "1", "0"));
    assert!(settings_valid("number", "0", "1", "0.5"));
  }

  #[test]
  fn surface_lines_are_replaced_or_inserted_after_their_anchor() {
    let text = "@define-color transparent transparent;\nbody\n";
    assert_eq!(
      insert_after(
        text,
        "@define-color transparent transparent;",
        "@define-color x y;"
      ),
      "@define-color transparent transparent;\n@define-color x y;\nbody\n"
    );
    assert_eq!(
      set_define("@define-color th-power old;\n", "th-power", "new;"),
      "@define-color th-power new;\n"
    );
  }

  #[test]
  fn animation_block_is_removed_between_its_markers() {
    let text = "a\n/* ARGVUS animations disabled */\nx\n/* ARGVUS animations disabled end */\nb\n";
    assert_eq!(
      delete_range(
        text,
        "/* ARGVUS animations disabled */",
        "/* ARGVUS animations disabled end */"
      ),
      "a\nb\n"
    );
  }

  #[test]
  fn opacity_follows_the_inverse_of_the_transparency() {
    assert_eq!(opacity_factor(20), "0.80");
    assert_eq!(opacity_factor(0), "1.00");
    assert_eq!(opacity_factor(100), "0.00");
  }
}
