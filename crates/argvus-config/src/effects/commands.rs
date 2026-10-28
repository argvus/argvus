//! Subcommands of `argvus config effects`: status, toggles, per-surface values and the settings table.

use super::surfaces::{signal_kitty, spawn_runtime};
use super::*;

impl Effects {
  pub(super) fn set_status(&self, component: &str, status: &str) -> Result<(), Failure> {
    self.require_store()?;
    match component {
      "blur" => self
        .write_flag(json_pointers::EFFECTS_BLUR_GLOBAL_ENABLED, status)
        .map_err(|_| 1)?,
      "animations" => self
        .write_flag(json_pointers::EFFECTS_ANIMATIONS, status)
        .map_err(|_| 1)?,
      "transparency" => self.write_transparency(status).map_err(|_| 1)?,
      "effects" => self.write_effects(status).map_err(|_| 1)?,
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

  pub(super) fn set_all_status(&self, status: &str) -> Result<(), Failure> {
    self.require_store()?;
    self
      .write_flag(json_pointers::EFFECTS_ANIMATIONS, status)
      .map_err(|_| 1)?;
    self.write_effects(status).map_err(|_| 1)?;
    self.request_config_reload().map_err(|_| 1)?;
    self.apply_surfaces(status, status).map_err(|_| 1)?;
    println!("{status}");
    spawn_runtime();
    Ok(())
  }

  pub(super) fn require_store(&self) -> Result<(), Failure> {
    if self.store.is_some() {
      Ok(())
    } else {
      eprintln!("argvus is required to change effects");
      Err(1)
    }
  }

  pub(super) fn toggle_component(&self, component: &str) -> Result<(), Failure> {
    match self.status(component) {
      "enabled" => self.set_status(component, "disabled"),
      _ => self.set_status(component, "enabled"),
    }
  }

  /// `enabled` only when animations are enabled and EFFECTS reads as enabled
  /// (blur or any effect surface transparency).
  pub(super) fn all_effects_status(&self) -> &'static str {
    all_effects_status(self.status("animations"), self.status("effects"))
  }

  /// Switches every effect at once, from the same aggregate that `status`
  /// prints, so a toggle always moves away from what `status` reported.
  pub(super) fn toggle_all(&self) -> Result<(), Failure> {
    self.set_all_status(toggle_target(self.all_effects_status()))
  }

  pub(super) fn component_command(&self, component: &str, operation: &str) -> Result<(), Failure> {
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
  pub(super) fn surface_value_command(
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
            .store_raw(json_pointers::EFFECTS_CONTROL_CENTER_TRANSPARENCY)
            .unwrap_or_default();
          match raw.parse::<u32>() {
            Ok(number) if number <= 100 && !raw.starts_with('+') => {
              println!("{number}")
            }
            _ => println!("15"),
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
              json_pointers::EFFECTS_CONTROL_CENTER_TRANSPARENCY,
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
  pub(super) fn surface_apply_command(&self, arguments: &[String]) -> Result<(), Failure> {
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
            json_pointers::EFFECTS_CONTROL_CENTER_TRANSPARENCY,
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
        // The blur of a surface is a layer rule of the compositor. The bars
        // reload the stylesheets just rewritten, and the configuration reload
        // reaches the Control Panel: nothing is restarted.
        if on_path("argvus") {
          let _ = Command::new("argvus")
            .args(["config", "hyprland", "reload"])
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
  pub(super) fn set_launcher_projection(&self, enabled: &str, value: &str) -> Result<(), Failure> {
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
  pub(super) fn effect_enabled_command(&self, surface: &str, name: &str) -> Result<(), Failure> {
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

  /// The usage line of the effects switch, translated when argvus-i18n is installed.
  pub(super) fn usage(&self) {
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
  pub(super) fn settings_get_or_set(
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

pub(super) struct Row {
  pub(super) minimum: String,
  pub(super) maximum: String,
  pub(super) kind: String,
  pub(super) default: String,
}

pub(super) fn table_row(table: &str, key: &str) -> Option<Row> {
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

pub(super) fn all_effects_status(animations: &str, effects: &str) -> &'static str {
  if animations == "enabled" && effects == "enabled" {
    "enabled"
  } else {
    "disabled"
  }
}

/// `enabled` when blur or at least one effect surface transparency is enabled;
/// the animations master switch does not take part.
pub(super) fn effects_status(transparency: &str, blur: &str) -> &'static str {
  if transparency == "enabled" || blur == "enabled" {
    "enabled"
  } else {
    "disabled"
  }
}

/// What a toggle switches every effect to: off when all are on, on otherwise.
pub(super) fn toggle_target(aggregate: &str) -> &'static str {
  if aggregate == "enabled" {
    "disabled"
  } else {
    "enabled"
  }
}

pub(super) fn raw_text(value: Option<&Value>) -> String {
  match value {
    Some(Value::String(text)) => text.clone(),
    Some(other) => other.to_string(),
    None => "null".to_string(),
  }
}

pub(super) fn config_value_pointer(surface: &str) -> Option<String> {
  match surface {
    "taskbar" | "control-panel" | "widget-telemetry" | "terminal" | "launchers" => {
      Some(format!("/effects/transparency_{surface}_value"))
    }
    _ => None,
  }
}

pub(super) fn config_enabled_pointer(surface: &str, kind: &str) -> Option<String> {
  match (surface, kind) {
    (
      "taskbar" | "control-panel" | "widget-telemetry" | "terminal" | "launchers",
      "transparency",
    ) => Some(format!("/effects/transparency_{surface}_enabled")),
    _ => None,
  }
}

/// The master switch of the effect whose settings carry `prefix`.
pub(super) fn master_switch(prefix: &str) -> Option<&'static str> {
  match prefix {
    "blur_" => Some(json_pointers::EFFECTS_BLUR_GLOBAL_ENABLED),
    "animation_" => Some(json_pointers::EFFECTS_ANIMATIONS),
    "shadow_" => Some(json_pointers::EFFECTS_SHADOW_ENABLED),
    _ => None,
  }
}
