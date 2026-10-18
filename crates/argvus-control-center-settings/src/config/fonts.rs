//! Implements font discovery and configuration in crate `argvus control center settings`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use std::collections::HashMap;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;
use std::process::Stdio;

use crate::config::paths;
use crate::error::SettingsError;
use crate::system::fonts::FontEntry;

/// Defines the constant `DEFAULT_FAMILY`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
const DEFAULT_FAMILY: &str = "IBM Plex Mono";
/// Defines the constant `MANAGED_START`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
const MANAGED_START: &str = "/* argvus-control-center-fonts:start */";
/// Defines the constant `MANAGED_END`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
const MANAGED_END: &str = "/* argvus-control-center-fonts:end */";
/// Defines the constant `LEGACY_MANAGED_START`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
const LEGACY_MANAGED_START: &str = "/* argvus-settings-fonts:start */";
/// Defines the constant `LEGACY_MANAGED_END`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
const LEGACY_MANAGED_END: &str = "/* argvus-settings-fonts:end */";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Defines `FontTarget`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub enum FontTarget {
  Taskbar,
  Sysinfo,
  ControlPanel,
  System,
  Apps,
  Terminal,
  Browser,
}

impl FontTarget {
  /// Defines the constant `ALL`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
  pub const ALL: [Self; 7] = [
    Self::Taskbar,
    Self::Sysinfo,
    Self::ControlPanel,
    Self::System,
    Self::Apps,
    Self::Terminal,
    Self::Browser,
  ];

  /// Executes the const function documented in this module. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
  pub const fn key(self) -> &'static str {
    match self {
      Self::Taskbar => "taskbar",
      Self::Sysinfo => "sysinfo",
      Self::ControlPanel => "control_panel",
      Self::System => "system",
      Self::Apps => "apps",
      Self::Terminal => "terminal",
      Self::Browser => "browser",
    }
  }

  /// Executes the const function documented in this module. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
  pub const fn default_size(self) -> u16 {
    match self {
      Self::Taskbar | Self::System | Self::Terminal => 13,
      Self::Sysinfo | Self::ControlPanel => 14,
      Self::Apps => 12,
      Self::Browser => 10,
    }
  }

  /// Executes the const function documented in this module. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
  const fn default_family(self) -> &'static str {
    DEFAULT_FAMILY
  }

  /// Executes the const function documented in this module. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
  const fn default_style(self) -> &'static str {
    "Regular"
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Defines `SettingKind`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub enum SettingKind {
  Antialiasing,
  Hinting,
  Subpixel,
  Dpi,
}

impl SettingKind {
  /// Defines the constant `ALL`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
  pub const ALL: [Self; 4] = [Self::Antialiasing, Self::Hinting, Self::Subpixel, Self::Dpi];
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Represents `FontSelection`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct FontSelection {
  pub family: String,
  pub style: String,
  pub size: u16,
}

impl FontSelection {
  /// Executes the `display_name` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn display_name(&self) -> String {
    FontEntry {
      family: self.family.clone(),
      style: self.style.clone(),
    }
    .display_name()
  }

  /// Executes the `value` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn value(&self) -> String {
    format!("{} {}", self.display_name(), self.size)
  }
}

#[derive(Debug, Clone)]
/// Represents `FontSettings`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct FontSettings {
  profile: HashMap<FontTarget, FontSelection>,
  pub antialias: bool,
  pub hinting: String,
  pub subpixel: String,
  pub custom_dpi: bool,
  pub dpi: u16,
}

impl FontSettings {
  /// Retrieves data for `load` without mixing collection with TUI rendering. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn load() -> Self {
    let mut state = parse_state(&fs::read_to_string(paths::fonts_file()).unwrap_or_default());
    if let Some(fonts) = canonical_fonts() {
      overlay_canonical_fonts(&mut state, &fonts);
    }
    let mut profile = HashMap::new();
    for target in FontTarget::ALL {
      let key = target.key();
      let family = state
        .get(&format!("{key}_family"))
        .filter(|value| !value.trim().is_empty())
        .cloned()
        .unwrap_or_else(|| DEFAULT_FAMILY.to_string());
      let style = state
        .get(&format!("{key}_style"))
        .cloned()
        .unwrap_or_else(|| "Regular".to_string());
      let size = state
        .get(&format!("{key}_size"))
        .and_then(|value| value.parse().ok())
        .unwrap_or_else(|| target.default_size())
        .clamp(8, 32);
      profile.insert(
        target,
        FontSelection {
          family,
          style,
          size,
        },
      );
    }
    Self {
      profile,
      antialias: state.get("antialias").is_none_or(|value| value == "true"),
      hinting: state
        .get("hinting")
        .cloned()
        .unwrap_or_else(|| "full".to_string()),
      subpixel: state
        .get("subpixel")
        .cloned()
        .unwrap_or_else(|| "none".to_string()),
      custom_dpi: state.get("custom_dpi").is_some_and(|value| value == "true"),
      dpi: state
        .get("dpi")
        .and_then(|value| value.parse().ok())
        .unwrap_or(96)
        .clamp(72, 240),
    }
  }

  /// Executes the `get` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn get(&self, target: FontTarget) -> &FontSelection {
    self
      .profile
      .get(&target)
      .expect("every font target is initialized")
  }

  /// Applies the `apply_font` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn apply_font(
    &mut self,
    target: FontTarget,
    font: &FontEntry,
    size: u16,
  ) -> Result<(), SettingsError> {
    let previous = self.get(target).clone();
    self.profile.insert(
      target,
      FontSelection {
        family: font.family.clone(),
        style: font.style.clone(),
        size: size.clamp(8, 32),
      },
    );
    let result = self.apply_target(target);
    if result.is_err() {
      self.profile.insert(target, previous);
      let _ = self.write_state();
    }
    result.map_err(SettingsError::Fonts)
  }

  /// Executes the `reset_font` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn reset_font(&mut self, target: FontTarget) -> Result<(), SettingsError> {
    let previous = self.get(target).clone();
    self.profile.insert(
      target,
      FontSelection {
        family: target.default_family().to_string(),
        style: target.default_style().to_string(),
        size: target.default_size(),
      },
    );
    let result = self.apply_target(target);
    if result.is_err() {
      self.profile.insert(target, previous);
      let _ = self.write_state();
    }
    result.map_err(SettingsError::Fonts)
  }

  /// Executes the `reset_setting` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn reset_setting(&mut self, setting: SettingKind) -> Result<(), SettingsError> {
    let value = match setting {
      SettingKind::Antialiasing => "enabled",
      SettingKind::Hinting => "full",
      SettingKind::Subpixel => "none",
      SettingKind::Dpi => "automatic",
    };
    self.apply_setting(setting, value)
  }

  /// Executes the `reset_all` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn reset_all(&mut self) -> Result<(), SettingsError> {
    let previous = self.clone();
    *self = Self::defaults();
    let result = (|| {
      self.write_state()?;
      for target in FontTarget::ALL {
        self.apply_target(target)?;
      }
      self.apply_rendering()?;
      self.refresh_runtime();
      Ok(())
    })();
    if result.is_err() {
      *self = previous;
      let _ = self.write_state();
    }
    result.map_err(SettingsError::Fonts)
  }

  /// Applies the `apply_setting` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn apply_setting(&mut self, setting: SettingKind, value: &str) -> Result<(), SettingsError> {
    let previous = self.clone();
    match setting {
      SettingKind::Antialiasing => self.antialias = value == "enabled",
      SettingKind::Hinting => self.hinting = value.to_string(),
      SettingKind::Subpixel => self.subpixel = value.to_string(),
      SettingKind::Dpi => {
        if value == "automatic" {
          self.custom_dpi = false;
        } else {
          self.dpi = value
            .parse::<u16>()
            .map_err(|error| SettingsError::Fonts(error.to_string()))?
            .clamp(72, 240);
          self.custom_dpi = true;
        }
      }
    }
    let result = (|| {
      self.write_state()?;
      if setting != SettingKind::Dpi {
        self.apply_rendering()?;
      }
      self.refresh_runtime();
      Ok(())
    })();
    if result.is_err() {
      *self = previous;
      let _ = self.write_state();
    }
    result.map_err(SettingsError::Fonts)
  }

  /// Executes the `setting_value` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn setting_value(&self, setting: SettingKind) -> String {
    match setting {
      SettingKind::Antialiasing => if self.antialias {
        "enabled"
      } else {
        "disabled"
      }
      .to_string(),
      SettingKind::Hinting => self.hinting.clone(),
      SettingKind::Subpixel => self.subpixel.clone(),
      SettingKind::Dpi => {
        if self.custom_dpi {
          self.dpi.to_string()
        } else {
          "automatic".to_string()
        }
      }
    }
  }

  /// Applies the `apply_target` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn apply_target(&self, target: FontTarget) -> Result<(), String> {
    self.write_state()?;
    match target {
      FontTarget::Taskbar | FontTarget::Sysinfo => self.write_waybar_settings()?,
      FontTarget::ControlPanel => {}
      FontTarget::System => gsettings_set(
        "org.gnome.desktop.wm.preferences",
        "titlebar-font",
        &self.get(target).value(),
      )?,
      FontTarget::Apps => {
        let value = self.get(target).value();
        gsettings_set("org.gnome.desktop.interface", "font-name", &value)?;
        gsettings_set("org.gnome.desktop.interface", "document-font-name", &value)?;
        self.write_gtk_settings()?;
        self.write_rofi_settings()?;
      }
      FontTarget::Terminal => {
        gsettings_set(
          "org.gnome.desktop.interface",
          "monospace-font-name",
          &self.get(target).value(),
        )?;
        self.write_terminal_settings()?;
      }
      FontTarget::Browser => self.write_browser_settings()?,
    }
    self.refresh_runtime();
    Ok(())
  }

  /// Executes the `defaults` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn defaults() -> Self {
    let mut profile = HashMap::new();
    for target in FontTarget::ALL {
      profile.insert(
        target,
        FontSelection {
          family: target.default_family().to_string(),
          style: target.default_style().to_string(),
          size: target.default_size(),
        },
      );
    }
    Self {
      profile,
      antialias: true,
      hinting: "full".to_string(),
      subpixel: "none".to_string(),
      custom_dpi: false,
      dpi: 96,
    }
  }

  /// Applies the `apply_rendering` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn apply_rendering(&self) -> Result<(), String> {
    gsettings_set(
      "org.gnome.desktop.interface",
      "font-antialiasing",
      if self.antialias { "grayscale" } else { "none" },
    )?;
    gsettings_set("org.gnome.desktop.interface", "font-hinting", &self.hinting)?;
    if let Some(order) = rgba_order(&self.subpixel) {
      gsettings_set("org.gnome.desktop.interface", "font-rgba-order", order)?;
    }
    Ok(())
  }

  /// Applies the `write_state` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn write_state(&self) -> Result<(), String> {
    let mut output = String::from("# Written by argvus-control-center. Edit with care.\n");
    for target in FontTarget::ALL {
      let font = self.get(target);
      let key = target.key();
      let _ = writeln!(output, "{key}_family={}", font.family);
      let _ = writeln!(output, "{key}_style={}", font.style);
      let _ = writeln!(output, "{key}_size={}", font.size);
      let _ = writeln!(output, "{key}_name={}", font.display_name());
    }
    let apps = self.get(FontTarget::Apps);
    let terminal = self.get(FontTarget::Terminal);
    let _ = writeln!(output, "default_family={}", apps.family);
    let _ = writeln!(output, "default_style={}", apps.style);
    let _ = writeln!(output, "default_size={}", apps.size);
    let _ = writeln!(output, "default_name={}", apps.display_name());
    let _ = writeln!(output, "monospace_family={}", terminal.family);
    let _ = writeln!(output, "monospace_style={}", terminal.style);
    let _ = writeln!(output, "monospace_size={}", terminal.size);
    let _ = writeln!(output, "monospace_name={}", terminal.display_name());
    let _ = writeln!(output, "antialias={}", self.antialias);
    let _ = writeln!(output, "hinting={}", self.hinting);
    let _ = writeln!(output, "subpixel={}", self.subpixel);
    let _ = writeln!(output, "custom_dpi={}", self.custom_dpi);
    let _ = writeln!(output, "dpi={}", self.dpi);
    let targets = FontTarget::ALL
      .into_iter()
      .map(|target| {
        let font = self.get(target);
        (
          target.key().to_owned(),
          serde_json::json!({
            "family": font.family,
            "style": font.style,
            "size": font.size,
          }),
        )
      })
      .collect::<serde_json::Map<_, _>>();
    let patch = serde_json::json!({
      "/fonts/targets": targets,
      "/fonts/rendering": {
        "antialias": self.antialias,
        "hinting": self.hinting,
        "subpixel": self.subpixel,
        "custom_dpi": self.custom_dpi,
        "dpi": self.dpi,
      },
      "/fonts/legacy": output,
    });
    let patch = serde_json::to_string(&patch).map_err(|error| error.to_string())?;
    run_checked("argvus", &["config", "patch", &patch])?;
    argvus_control_center_core::config::reload_argvus_config_service()
  }

  /// Applies the `write_gtk_settings` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn write_gtk_settings(&self) -> Result<(), String> {
    let value = self.get(FontTarget::Apps).value();
    for directory in [
      paths::config_home().join("gtk-3.0"),
      paths::config_home().join("gtk-4.0"),
      paths::argvus_data_home().join("gtk-3.0"),
      paths::argvus_data_home().join("gtk-4.0"),
    ] {
      replace_ini_setting(&directory.join("settings.ini"), "gtk-font-name", &value)?;
    }
    Ok(())
  }

  /// Applies the `write_rofi_settings` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn write_rofi_settings(&self) -> Result<(), String> {
    let system = paths::system_config_root().join("launcher/config/theme.rasi");
    let user_theme = paths::argvus_data_home().join("rofi/theme.rasi");
    let theme = if user_theme.exists() {
      user_theme
    } else {
      system
    };
    let font = escape(&self.get(FontTarget::Apps).value());
    let generated = paths::argvus_data_home().join("generated/rofi/config.rasi");
    write_file(
      &generated,
      &format!(
        "@theme \"{}\"\n\nconfiguration {{\n    font: \"{font}\";\n    show-icons: false;\n    sort: true;\n    case-sensitive: false;\n}}\n",
        escape(&theme.display().to_string())
      ),
    )?;
    let user = paths::argvus_data_home().join("rofi/config.rasi");
    if user.exists() {
      write_managed_block(
        &user,
        &format!("configuration {{\n    font: \"{font}\";\n}}\n"),
      )?;
    }
    Ok(())
  }

  /// Applies the `write_waybar_settings` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn write_waybar_settings(&self) -> Result<(), String> {
    let taskbar = self.get(FontTarget::Taskbar);
    let widget_telemetry = self.get(FontTarget::Sysinfo);
    self.write_waybar_profile(
      "argvus-taskbar.css",
      &format!(
        "* {{\n  font-family: \"{}\", \"Symbols Nerd Font Mono\", monospace;\n  font-size: {}px;\n}}\n\n#custom-icon-window,\n#network,\n#bluetooth,\n#custom-bluetooth,\n#custom-expand-icon,\n#custom-removable-devices,\n#custom-recording,\n#custom-search,\n#pulseaudio,\n#power-profiles-daemon,\n#custom-settings,\n#custom-power {{\n  font-family: \"Symbols Nerd Font Mono\";\n}}\n",
        escape(&taskbar.family), taskbar.size
      ),
    )?;
    self.write_waybar_profile(
      "argvus-widget-telemetry.css",
      &format!(
        "* {{\n  font-family: \"{}\", \"Symbols Nerd Font Mono\", monospace;\n  font-size: {}px;\n}}\n",
        escape(&widget_telemetry.family), widget_telemetry.size
      ),
    )
  }

  /// Applies the `write_waybar_profile` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn write_waybar_profile(&self, name: &str, block: &str) -> Result<(), String> {
    let user = paths::argvus_data_home().join("waybar").join(name);
    if user.exists() {
      return write_managed_block(&user, block);
    }
    let project = match name {
      "argvus-taskbar.css" => "taskbar",
      "argvus-widget-telemetry.css" => "widget-telemetry",
      _ => "taskbar",
    };
    let system = paths::system_config_root()
      .join(project)
      .join("config")
      .join(name);
    let generated = paths::argvus_data_home()
      .join("generated/waybar")
      .join(name);
    write_file(
      &generated,
      &format!(
        "@import url(\"{}\");\n\n{MANAGED_START}\n{block}{MANAGED_END}\n",
        escape(&system.display().to_string())
      ),
    )
  }

  /// Applies the `write_terminal_settings` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn write_terminal_settings(&self) -> Result<(), String> {
    // The terminal family and size were already written to fonts.conf by
    // write_state; argvus-terminal --apply folds them into the Kitty profiles
    // consumed by the ARGVUS terminal windows.
    spawn_if_available("argvus", &["terminal", "--apply"]);
    Ok(())
  }

  /// Applies the `write_browser_settings` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn write_browser_settings(&self) -> Result<(), String> {
    let font = self.get(FontTarget::Browser);
    let mut matches = String::new();
    for process in [
      "firefox",
      "chromium",
      "google-chrome",
      "google-chrome-stable",
      "brave",
      "brave-browser",
    ] {
      let _ = writeln!(
        matches,
        "  <match target=\"pattern\">\n    <test name=\"prgname\" compare=\"eq\"><string>{process}</string></test>\n    <edit name=\"family\" mode=\"prepend\" binding=\"strong\"><string>{}</string></edit>\n    <edit name=\"size\" mode=\"assign\"><double>{}</double></edit>\n  </match>",
        xml_escape(&font.family),
        font.size
      );
    }
    let file = paths::config_home().join("fontconfig/conf.d/52-argvus-browser-font.conf");
    write_file(
      &file,
      &format!(
        "<?xml version=\"1.0\"?>\n<!DOCTYPE fontconfig SYSTEM \"urn:fontconfig:fonts.dtd\">\n<fontconfig>\n{matches}</fontconfig>\n"
      ),
    )?;
    spawn_if_available("fc-cache", &["-f"]);
    Ok(())
  }

  /// Executes the `refresh_runtime` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn refresh_runtime(&self) {
    spawn_if_available("hyprctl", &["reload"]);
    spawn_if_available("argvus", &["sessionctl", "restart", "waybar", "shell"]);
    let script = paths::system_config_root().join("lock/sh/hyprlock-theme.sh");
    if script.is_file() {
      let script = script.to_string_lossy();
      spawn_if_available("sh", &[script.as_ref(), "--invalidate"]);
    }
  }
}

/// Converts input data into `parse_state` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn canonical_fonts() -> Option<serde_json::Value> {
  let output = argvus_control_center_core::process::command("argvus")
    .arg("config")
    .args(["get", "/fonts", "--effective"])
    .output()
    .ok()?;
  output
    .status
    .success()
    .then(|| serde_json::from_slice(&output.stdout).ok())
    .flatten()
}

fn overlay_canonical_fonts(state: &mut HashMap<String, String>, fonts: &serde_json::Value) {
  if let Some(targets) = fonts.get("targets").and_then(serde_json::Value::as_object) {
    for (target, selection) in targets {
      for key in ["family", "style"] {
        if let Some(value) = selection.get(key).and_then(serde_json::Value::as_str) {
          state.insert(format!("{target}_{key}"), value.to_owned());
        }
      }
      if let Some(size) = selection.get("size").and_then(serde_json::Value::as_u64) {
        state.insert(format!("{target}_size"), size.to_string());
      }
    }
  }
  if let Some(rendering) = fonts
    .get("rendering")
    .and_then(serde_json::Value::as_object)
  {
    for key in ["antialias", "custom_dpi"] {
      if let Some(value) = rendering.get(key).and_then(serde_json::Value::as_bool) {
        state.insert(key.into(), value.to_string());
      }
    }
    for key in ["hinting", "subpixel"] {
      if let Some(value) = rendering.get(key).and_then(serde_json::Value::as_str) {
        state.insert(key.into(), value.to_owned());
      }
    }
    if let Some(value) = rendering.get("dpi").and_then(serde_json::Value::as_u64) {
      state.insert("dpi".into(), value.to_string());
    }
  }
}

fn run_checked(command: &str, arguments: &[&str]) -> Result<(), String> {
  let status = argvus_control_center_core::process::command(command)
    .args(arguments)
    .status()
    .map_err(|error| format!("failed to run {command}: {error}"))?;
  status
    .success()
    .then_some(())
    .ok_or_else(|| format!("{command} exited with {status}"))
}

pub fn parse_state(contents: &str) -> HashMap<String, String> {
  contents
    .lines()
    .filter_map(|line| {
      let line = line.trim();
      if line.is_empty() || line.starts_with('#') {
        return None;
      }
      line
        .split_once('=')
        .map(|(key, value)| (key.trim().to_string(), value.trim().to_string()))
    })
    .collect()
}

/// Executes the `rgba_order` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn rgba_order(subpixel: &str) -> Option<&str> {
  match subpixel {
    "rgba" | "rgb" | "bgr" | "vrgb" | "vbgr" => Some(subpixel),
    _ => None,
  }
}

/// Executes the `gsettings_set` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn gsettings_set(schema: &str, key: &str, value: &str) -> Result<(), String> {
  let status = argvus_control_center_core::process::command("gsettings")
    .args(["set", schema, key, value])
    .stdin(Stdio::null())
    .stdout(Stdio::null())
    .stderr(Stdio::piped())
    .status()
    .map_err(|error| format!("gsettings unavailable: {error}"))?;
  if status.success() {
    Ok(())
  } else {
    Err(format!("gsettings exited with {status}"))
  }
}

/// Executes the `spawn_if_available` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn spawn_if_available(command: &str, args: &[&str]) {
  let _ = argvus_control_center_core::process::command(command)
    .args(args)
    .stdin(Stdio::null())
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .spawn();
}

/// Executes the `replace_ini_setting` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn replace_ini_setting(path: &Path, key: &str, value: &str) -> Result<(), String> {
  let mut lines: Vec<String> = if path.exists() {
    fs::read_to_string(path)
      .map_err(|error| error.to_string())?
      .lines()
      .map(str::to_string)
      .collect()
  } else {
    vec!["[Settings]".to_string()]
  };
  if !lines.iter().any(|line| line.trim() == "[Settings]") {
    lines.insert(0, "[Settings]".to_string());
  }
  replace_or_append(&mut lines, key, &format!("{key}={value}"));
  write_file(path, &(lines.join("\n") + "\n"))
}

/// Executes the `replace_or_append` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn replace_or_append(lines: &mut Vec<String>, key: &str, replacement: &str) {
  if let Some(line) = lines.iter_mut().find(|line| {
    let trimmed = line.trim_start();
    trimmed.starts_with(&format!("{key}=")) || trimmed.starts_with(&format!("{key} "))
  }) {
    *line = replacement.to_string();
  } else {
    lines.push(replacement.to_string());
  }
}

/// Applies the `write_managed_block` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn write_managed_block(path: &Path, block: &str) -> Result<(), String> {
  let existing = fs::read_to_string(path).unwrap_or_default();
  let managed = format!("{MANAGED_START}\n{block}{MANAGED_END}\n");
  let markers = if existing.contains(MANAGED_START) {
    Some((MANAGED_START, MANAGED_END))
  } else if existing.contains(LEGACY_MANAGED_START) {
    Some((LEGACY_MANAGED_START, LEGACY_MANAGED_END))
  } else {
    None
  };
  let contents = if let Some((start_marker, end_marker)) = markers {
    if let Some(start) = existing.find(start_marker) {
      let relative_end = existing[start..].find(end_marker);
      if let Some(relative_end) = relative_end {
        let end = start + relative_end + end_marker.len();
        format!("{}{}{}", &existing[..start], managed, &existing[end..])
      } else {
        format!("{}\n{managed}", existing.trim_end())
      }
    } else {
      format!("{}\n{managed}", existing.trim_end())
    }
  } else {
    format!("{}\n{managed}", existing.trim_end())
  };
  write_file(path, &contents)
}

/// Applies the `write_file` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn write_file(path: &Path, contents: &str) -> Result<(), String> {
  if let Some(parent) = path.parent() {
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
  }
  fs::write(path, contents).map_err(|error| format!("{}: {error}", path.display()))
}

/// Executes the `escape` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn escape(value: &str) -> String {
  value.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Executes the `xml_escape` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn xml_escape(value: &str) -> String {
  value
    .replace('&', "&amp;")
    .replace('<', "&lt;")
    .replace('>', "&gt;")
    .replace('"', "&quot;")
    .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  /// Converts input data into `parses_state_and_ignores_comments` while applying local validation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn parses_state_and_ignores_comments() {
    let state = parse_state("# header\napps_family=Noto Sans\napps_size=12\n");
    assert_eq!(
      state.get("apps_family").map(String::as_str),
      Some("Noto Sans")
    );
    assert_eq!(state.len(), 2);
  }

  #[test]
  /// Executes the `all_targets_have_valid_defaults` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn all_targets_have_valid_defaults() {
    let settings = FontSettings::load();
    for target in FontTarget::ALL {
      assert!(!settings.get(target).family.is_empty());
      assert!((8..=32).contains(&settings.get(target).size));
    }
  }

  #[test]
  /// Executes the `xml_values_are_escaped` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn xml_values_are_escaped() {
    assert_eq!(xml_escape("A&B <Mono>"), "A&amp;B &lt;Mono&gt;");
  }

  #[test]
  /// Executes the `subpixel_none_is_not_mapped_to_an_invalid_rgba_order` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn subpixel_none_is_not_mapped_to_an_invalid_rgba_order() {
    assert_eq!(rgba_order("none"), None);
    assert_eq!(rgba_order("rgb"), Some("rgb"));
    assert_eq!(rgba_order("bgr"), Some("bgr"));
    assert_eq!(rgba_order("vrgb"), Some("vrgb"));
    assert_eq!(rgba_order("vbgr"), Some("vbgr"));
    assert_eq!(rgba_order("unknown"), None);
  }
}
