//! Per-theme effects projection: the values the configuration store does not own, kept next to the active theme.

use super::commands::{config_enabled_pointer, config_value_pointer};
use super::*;

impl Effects {
  /// Creates the projection file with its defaults and renames the old
  /// launcher keys, as the shell did on every read.
  /// Completes the theme's effects projection with the defaults it lacks.
  ///
  /// Every reader calls this, and the Control Center runs its readers
  /// concurrently, so the file is only written when something was added, and
  /// then through a rename: a reader never sees it truncated, and never
  /// writes back the defaults over a half-read file.
  pub(super) fn ensure_theme_effects(&self) -> io::Result<()> {
    let file = self.theme_effects_file();
    if let Some(parent) = file.parent() {
      fs::create_dir_all(parent)?;
    }
    let original = match fs::read_to_string(&file) {
      Ok(text) => Some(text),
      Err(error) if error.kind() == io::ErrorKind::NotFound => None,
      Err(error) => return Err(error),
    };
    let mut text = original.clone().unwrap_or_default();
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
      "launchers.transparency=15",
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
    if original.as_deref() == Some(text.as_str()) {
      return Ok(());
    }
    let temporary = file.with_extension(format!("tmp.{}", std::process::id()));
    fs::write(&temporary, text)?;
    fs::rename(&temporary, &file)
  }

  pub(super) fn projection_value(&self, key: &str) -> Option<String> {
    let text = fs::read_to_string(self.theme_effects_file()).ok()?;
    let prefix = format!("{key}=");
    text
      .lines()
      .find_map(|line| line.strip_prefix(&prefix))
      .map(str::to_string)
  }

  /// The transparency value of a surface, clamped to 0..=100.
  pub(super) fn effect_value(&self, key: &str) -> u32 {
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
  pub(super) fn effect_enabled(&self, name: &str) -> &'static str {
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
  pub(super) fn set_projection(&self, key: &str, value: &str) -> io::Result<()> {
    self.set_projection_pairs(&[(key.to_string(), value.to_string())])
  }

  pub(super) fn set_projection_pairs(&self, pairs: &[(String, String)]) -> io::Result<()> {
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

  pub(super) fn set_effect_value(&self, key: &str, value: &str) -> Result<(), Failure> {
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
}
