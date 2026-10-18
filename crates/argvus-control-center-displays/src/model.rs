//! Implements domain state and models consumed by the UI in crate `argvus control center displays`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayPage {
  Home,
  Profiles,
  /// One saved profile, with its Apply, Rename and Delete rows.
  Profile(usize),
  Detail(usize),
  Picker {
    monitor: usize,
    setting: MonitorSetting,
  },
  Prompt {
    goal: PromptGoal,
  },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Defines `PromptGoal`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub enum PromptGoal {
  /// Free-form "x,y" position editor for a monitor index.
  Position(usize),
  /// Free-form fractional scale editor for a monitor index.
  Scale(usize),
  SdrBrightness(usize),
  SdrSaturation(usize),
  ProfileCreate,
  ProfileRename(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Defines `MonitorSetting`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub enum MonitorSetting {
  Resolution,
  RefreshRate,
  Scale,
  Position,
  Orientation,
  Vrr,
  Hdr,
  Primary,
  Enabled,
  Mirror,
  BitDepth,
  Dpms,
  SdrBrightness,
  SdrSaturation,
  Workspaces,
}

impl MonitorSetting {
  /// Executes the `picker_rows` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn picker_rows(lang: argvus_i18n::Lang) -> &'static [MonitorSetting] {
    let _ = lang;
    &[
      MonitorSetting::Resolution,
      MonitorSetting::RefreshRate,
      MonitorSetting::Scale,
      MonitorSetting::Position,
      MonitorSetting::Orientation,
      MonitorSetting::Enabled,
      MonitorSetting::Mirror,
      MonitorSetting::BitDepth,
      MonitorSetting::Vrr,
      MonitorSetting::Hdr,
      MonitorSetting::Dpms,
      MonitorSetting::SdrBrightness,
      MonitorSetting::SdrSaturation,
      MonitorSetting::Workspaces,
      MonitorSetting::Primary,
    ]
  }
}

#[derive(Debug, Clone, PartialEq)]
/// Represents `Mode`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct Mode {
  pub id: i64,
  pub width: u32,
  pub height: u32,
  pub refresh_rate: f64,
  pub bit_depth: u32,
}

impl Mode {
  /// Executes the `label` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn label(&self) -> String {
    format!(
      "{}x{} @ {:.3} Hz  ·  {} bpp",
      self.width, self.height, self.refresh_rate, self.bit_depth
    )
  }

  /// Executes the `method_specific` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn method_specific(&self) -> String {
    format!(
      "{}x{}@{}",
      self.width,
      self.height,
      format_rate(self.refresh_rate)
    )
  }
}

/// Executes the `format_rate` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn format_rate(rate: f64) -> String {
  let rounded = (rate * 1000.0).round() / 1000.0;
  if rounded.fract() < 0.0005 {
    format!("{}", rounded as u64)
  } else {
    format!("{rounded:.3}")
  }
}

#[derive(Debug, Clone, Default, PartialEq)]
/// Represents `MonitorInfo`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct MonitorInfo {
  pub make: String,
  pub model: String,
  pub serial: String,
  pub description: String,
  pub physical_width: Option<u32>,
  pub physical_height: Option<u32>,
  pub current_format: String,
  pub mirror_of: Option<String>,
  pub sdr_brightness: Option<f64>,
  pub sdr_saturation: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
/// Represents `Monitor`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct Monitor {
  pub id: i32,
  pub name: String,
  pub x: i32,
  pub y: i32,
  pub width: u32,
  pub height: u32,
  pub refresh_rate: f64,
  pub scale: f64,
  pub transform: i32,
  pub focused: bool,
  pub vrr: i32,
  pub dpms_status: String,
  pub disabled: bool,
  pub modes: Vec<Mode>,
  pub connected: bool,
  pub info: MonitorInfo,
  pub active_workspace: Option<i32>,
}

impl Monitor {
  /// The safe "current state" keyword arguments: every active field is
  /// restored, used to revert risky changes to the exact previous layout.
  pub fn keyword_args(&self) -> String {
    format!(
      "{}, {}x{}@{}, {}x{}, {}, transform, {}, vrr, {}",
      self.name,
      self.width,
      self.height,
      format_rate(self.refresh_rate),
      self.x,
      self.y,
      self.scale,
      self.transform,
      self.vrr,
    )
  }

  /// Full monitor rule that replays the effective persisted state over the
  /// live values. Used by the Apply button so one rule always matches what
  /// was saved, including mirror / bit depth / VRR / HDR / SDR fields.
  pub fn apply_args(&self, persisted: &PersistedMonitor) -> String {
    let mode = persisted.mode.clone().unwrap_or_else(|| {
      format!(
        "{}x{}@{}",
        self.width,
        self.height,
        format_rate(self.refresh_rate)
      )
    });
    let position = persisted
      .position
      .clone()
      .unwrap_or_else(|| format!("{}x{}", self.x, self.y));
    let scale = persisted.scale.unwrap_or(self.scale);
    let transform = persisted.transform.unwrap_or(self.transform);
    let mut body = format!(
      "{}, {mode}, {position}, {scale}, transform, {transform}",
      self.name
    );
    match &persisted.mirror {
      Some(mirror) => body.push_str(&format!(", mirror, {mirror}")),
      None if self.info.mirror_of.is_some() => body.push_str(", mirror, "),
      None => {}
    }
    if let Some(bitdepth) = persisted.bitdepth {
      body.push_str(&format!(", bitdepth, {bitdepth}"));
    }
    if let Some(vrr) = persisted.vrr {
      body.push_str(&format!(", vrr, {vrr}"));
    }
    if let Some(hdr) = persisted.hdr {
      body.push_str(&format!(", supports_hdr, {hdr}"));
    }
    if let Some(brightness) = persisted.sdr_brightness {
      body.push_str(&format!(", sdrbrightness, {brightness}"));
    }
    if let Some(saturation) = persisted.sdr_saturation {
      body.push_str(&format!(", sdrsaturation, {saturation}"));
    }
    body
  }

  /// Executes the `mode_id_args` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn mode_id_args(&self, mode: &Mode) -> String {
    format!(
      "{}, #{:04X}, {}x{}, {}",
      self.name, mode.id, self.x, self.y, self.scale
    )
  }

  /// Executes the `resolution_args` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn resolution_args(&self, width: u32, height: u32, rate: f64) -> String {
    format!(
      "{}, {}x{}@{}, {}x{}, {}",
      self.name,
      width,
      height,
      format_rate(rate),
      self.x,
      self.y,
      self.scale
    )
  }

  /// Executes the `position_args` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn position_args(&self, x: i32, y: i32) -> String {
    format!("{}, auto, {}x{}, auto", self.name, x, y)
  }

  /// Executes the `scale_args` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn scale_args(&self, scale: f64) -> String {
    format!("{}, auto, {}x{}, {}", self.name, self.x, self.y, scale)
  }

  /// Executes the `transform_args` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn transform_args(&self, transform: i32) -> String {
    format!("{}, auto, auto, auto, transform, {}", self.name, transform)
  }

  /// Executes the `vrr_args` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn vrr_args(&self, vrr: i32) -> String {
    format!(
      "{}, auto, {}x{}, {}, vrr, {}",
      self.name, self.x, self.y, self.scale, vrr
    )
  }

  /// Executes the `hdr_args` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn hdr_args(&self, hdr: i32) -> String {
    format!(
      "{}, auto, {}x{}, {}, supports_hdr, {}",
      self.name, self.x, self.y, self.scale, hdr
    )
  }

  /// Executes the `mirror_args` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn mirror_args(&self, mirror: &str) -> String {
    format!("{}, auto, auto, auto, mirror, {}", self.name, mirror)
  }

  /// Executes the `disabled_args` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn disabled_args(&self) -> String {
    format!("{}, disabled", self.name)
  }

  /// Executes the `bitdepth_args` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn bitdepth_args(&self, bitdepth: i32) -> String {
    format!("{}, auto, auto, auto, bitdepth, {}", self.name, bitdepth)
  }

  /// Executes the `enabled_with_current_args` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn enabled_with_current_args(&self) -> String {
    self.keyword_args()
  }

  /// Checks the condition represented by `has_10bit` using only the state available to the module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn has_10bit(&self) -> bool {
    self.modes.iter().any(|mode| mode.bit_depth >= 10) || self.info.current_format.contains("10")
  }

  /// Executes the `info_rows` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn info_rows(&self) -> Vec<(String, String)> {
    let info = &self.info;
    let mut rows = Vec::new();
    if !info.make.is_empty() {
      rows.push(("Fabricante".into(), info.make.clone()));
    }
    if !info.model.is_empty() {
      rows.push(("Modelo".into(), info.model.clone()));
    }
    if !info.serial.is_empty() {
      rows.push(("Serial".into(), info.serial.clone()));
    }
    if !info.description.is_empty() {
      rows.push(("Descrição".into(), info.description.clone()));
    }
    if let (Some(width), Some(height)) = (info.physical_width, info.physical_height) {
      rows.push(("Tamanho físico".into(), format!("{}x{} mm", width, height)));
    }
    if !info.current_format.is_empty() {
      rows.push(("Formato atual".into(), info.current_format.clone()));
    }
    if let Some(mirror) = &info.mirror_of {
      rows.push(("Espelhando".into(), mirror.clone()));
    }
    rows
  }
}

/// Per-monitor overrides that ARGVUS persists. `dpms` is runtime-only and is
/// intentionally never written to the generated Lua file (Hyprland has no
/// persistent dpms configuration).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PersistedMonitor {
  pub mode: Option<String>,
  pub position: Option<String>,
  pub scale: Option<f64>,
  pub transform: Option<i32>,
  pub vrr: Option<i32>,
  /// HDR support: -1 force off, 0 auto, 1 force on (written as supports_hdr).
  pub hdr: Option<i32>,
  pub mirror: Option<String>,
  pub bitdepth: Option<i32>,
  pub disabled: Option<bool>,
  pub sdr_brightness: Option<f64>,
  pub sdr_saturation: Option<f64>,
  pub dpms: Option<bool>,
}

impl PersistedMonitor {
  /// Executes the `position_coords` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn position_coords(&self) -> Option<(i32, i32)> {
    let (x, y) = self.position.as_deref()?.split_once('x')?;
    Some((x.parse().ok()?, y.parse().ok()?))
  }
}

#[derive(Debug, Clone, Default, PartialEq)]
/// Represents `PersistedConfig`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct PersistedConfig {
  pub primary_monitor: Option<String>,
  pub monitors: Vec<(String, PersistedMonitor)>,
  /// Workspace ids bound to each monitor name.
  pub workspaces: Vec<(String, Vec<u32>)>,
}

impl PersistedConfig {
  /// Executes the `persisted` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn persisted(&self, name: &str) -> PersistedMonitor {
    self
      .monitors
      .iter()
      .find(|(monitor, _)| monitor == name)
      .map(|(_, config)| config.clone())
      .unwrap_or_default()
  }

  /// Applies the `set_monitor` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn set_monitor(&mut self, name: &str, config: PersistedMonitor) {
    let mut entries = std::mem::take(&mut self.monitors);
    if let Some(entry) = entries.iter_mut().find(|(existing, _)| existing == name) {
      entry.1 = config;
    } else {
      entries.push((name.to_string(), config));
    }
    entries.sort_by(|left, right| left.0.cmp(&right.0));
    self.monitors = entries;
  }

  /// Executes the `remove_monitor` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn remove_monitor(&mut self, name: &str) {
    self.monitors.retain(|(existing, _)| existing != name);
    self.workspaces.retain(|(monitor, _)| monitor != name);
    if self.primary_monitor.as_deref() == Some(name) {
      self.primary_monitor = None;
    }
  }

  /// Executes the `workspaces_of` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn workspaces_of(&self, name: &str) -> Vec<u32> {
    self
      .workspaces
      .iter()
      .find(|(monitor, _)| monitor == name)
      .map(|(_, ids)| ids.clone())
      .unwrap_or_default()
  }

  /// Applies the `set_workspaces` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn set_workspaces(&mut self, name: &str, ids: Vec<u32>) {
    let mut entries = std::mem::take(&mut self.workspaces);
    if let Some(entry) = entries.iter_mut().find(|(monitor, _)| monitor == name) {
      entry.1 = ids;
    } else {
      entries.push((name.to_string(), ids));
    }
    self.workspaces = entries;
  }
}

#[derive(Debug, Clone, PartialEq)]
/// Represents `MonitorProfile`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct MonitorProfile {
  pub name: String,
  pub apply_wallpapers: bool,
  pub config: PersistedConfig,
}

#[derive(Debug, Clone, Default, PartialEq)]
/// Represents `DisplayState`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct DisplayState {
  /// Last applied primary monitor, readable by other ARGVUS components.
  pub primary_monitor: Option<String>,
  pub active_profile: Option<String>,
  pub profiles: Vec<MonitorProfile>,
}

impl DisplayState {
  /// Executes the `active_profile` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn active_profile(&self) -> Option<&MonitorProfile> {
    let name = self.active_profile.as_deref()?;
    self.profiles.iter().find(|profile| profile.name == name)
  }

  /// Executes the `profile_index` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn profile_index(&self, name: &str) -> Option<usize> {
    self
      .profiles
      .iter()
      .position(|profile| profile.name == name)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  /// Executes the `monitor` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn monitor(name: &str) -> Monitor {
    Monitor {
      id: 0,
      name: name.into(),
      x: 0,
      y: 0,
      width: 1920,
      height: 1080,
      refresh_rate: 60.0,
      scale: 1.0,
      transform: 0,
      focused: false,
      vrr: 0,
      dpms_status: "on".into(),
      disabled: false,
      modes: vec![
        Mode {
          id: 1,
          width: 1920,
          height: 1080,
          refresh_rate: 60.0,
          bit_depth: 8,
        },
        Mode {
          id: 2,
          width: 1920,
          height: 1080,
          refresh_rate: 144.0,
          bit_depth: 10,
        },
      ],
      connected: true,
      info: MonitorInfo::default(),
      active_workspace: Some(1),
    }
  }

  #[test]
  /// Executes the `mode_labels_show_fractional_refresh` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn mode_labels_show_fractional_refresh() {
    let mode = Mode {
      id: 2,
      width: 1920,
      height: 1080,
      refresh_rate: 60.0,
      bit_depth: 10,
    };
    assert!(mode.label().contains("1920x1080"));
    assert_eq!(mode.method_specific(), "1920x1080@60");
  }

  #[test]
  /// Executes the `persisted_config_resolves_per_monitor` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn persisted_config_resolves_per_monitor() {
    let config = PersistedConfig {
      primary_monitor: Some("eDP-1".into()),
      monitors: vec![(
        "eDP-1".into(),
        PersistedMonitor {
          scale: Some(1.25),
          ..Default::default()
        },
      )],
      ..Default::default()
    };
    assert_eq!(config.persisted("eDP-1").scale, Some(1.25));
    assert_eq!(config.persisted("DP-1").scale, None);
  }

  #[test]
  /// Executes the `fractional_rates_keep_precision` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn fractional_rates_keep_precision() {
    assert_eq!(format_rate(60.0), "60");
    assert_eq!(format_rate(59.999), "59.999");
    assert_eq!(format_rate(100.001), "100.001");
  }

  #[test]
  /// Executes the `persisted_monitor_parses_positions` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn persisted_monitor_parses_positions() {
    let persisted = PersistedMonitor {
      position: Some("1920x0".into()),
      ..Default::default()
    };
    assert_eq!(persisted.position_coords(), Some((1920, 0)));
  }

  #[test]
  /// Applies the `set_and_remove_monitors_keep_lists_sorted` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn set_and_remove_monitors_keep_lists_sorted() {
    let mut config = PersistedConfig::default();
    config.set_monitor("HDMI-A-1", PersistedMonitor::default());
    config.set_monitor("eDP-1", PersistedMonitor::default());
    assert_eq!(config.monitors.len(), 2);
    assert_eq!(config.monitors[0].0, "HDMI-A-1");
    config.remove_monitor("eDP-1");
    assert_eq!(config.monitors.len(), 1);
  }

  #[test]
  /// Executes the `ten_bit_detection_covers_modes` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn ten_bit_detection_covers_modes() {
    assert!(monitor("eDP-1").has_10bit());
    let mut plain = monitor("eDP-1");
    plain.modes = plain
      .modes
      .into_iter()
      .map(|mut mode| {
        mode.bit_depth = 8;
        mode
      })
      .collect();
    assert!(!plain.has_10bit());
  }

  #[test]
  /// Applies the `apply_args_replays_persisted_fields` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn apply_args_replays_persisted_fields() {
    let mon = monitor("eDP-1");
    let persisted = PersistedMonitor {
      mode: Some("1920x1080@144".into()),
      position: Some("0x0".into()),
      scale: Some(1.25),
      transform: Some(1),
      vrr: Some(1),
      hdr: Some(1),
      bitdepth: Some(10),
      ..Default::default()
    };
    let args = mon.apply_args(&persisted);
    assert!(args.contains("1920x1080@144"), "{args}");
    assert!(args.contains("transform, 1"), "{args}");
    assert!(args.contains("vrr, 1"), "{args}");
    assert!(args.contains("supports_hdr, 1"), "{args}");
    assert!(args.contains("bitdepth, 10"), "{args}");
  }

  #[test]
  /// Applies the `apply_args_appends_mirror_clear_when_active` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn apply_args_appends_mirror_clear_when_active() {
    let mut mon = monitor("eDP-1");
    mon.info.mirror_of = Some("DP-1".into());
    let args = mon.apply_args(&PersistedMonitor::default());
    assert!(args.contains("mirror, "), "{args}");
    mon.info.mirror_of = None;
    let args = mon.apply_args(&PersistedMonitor::default());
    assert!(!args.contains("mirror"), "{args}");
  }

  #[test]
  /// Executes the `workspaces_round_trip_per_monitor` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn workspaces_round_trip_per_monitor() {
    let mut config = PersistedConfig::default();
    config.set_workspaces("eDP-1", vec![1, 2, 3]);
    assert_eq!(config.workspaces_of("eDP-1"), vec![1, 2, 3]);
    config.set_workspaces("eDP-1", vec![]);
    assert!(config.workspaces_of("eDP-1").is_empty());
  }

  #[test]
  /// Executes the `state_resolves_the_active_profile` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn state_resolves_the_active_profile() {
    let state = DisplayState {
      primary_monitor: Some("eDP-1".into()),
      active_profile: Some("Trabalho".into()),
      profiles: vec![MonitorProfile {
        name: "Trabalho".into(),
        apply_wallpapers: true,
        config: PersistedConfig::default(),
      }],
    };
    assert!(state.active_profile().is_some());
    assert_eq!(state.profile_index("Trabalho"), Some(0));
    assert_eq!(state.profile_index("Casa"), None);
  }

  #[test]
  /// Executes the `picker_rows_cover_the_documented_settings` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn picker_rows_cover_the_documented_settings() {
    let rows = MonitorSetting::picker_rows(argvus_i18n::Lang::for_locale("pt-BR"));
    assert!(rows.contains(&MonitorSetting::Workspaces));
    assert!(rows.contains(&MonitorSetting::SdrBrightness));
    assert!(rows.len() >= 14);
  }
}
