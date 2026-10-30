//! Rows of the monitor list and the monitor detail page, and the labels of each setting.

use super::*;

impl DisplaysApp {
  /// Rows of the current page.
  pub(super) fn rows(&self) -> Vec<Row<Item>> {
    match self.page {
      DisplayPage::Home => self.home_rows(),
      DisplayPage::NightLight => self.night_rows(),
      DisplayPage::Detail(index) => self.detail_rows(index),
      DisplayPage::Picker { monitor, setting } => self.picker_rows(monitor, setting),
      DisplayPage::Profiles => self.profile_rows(),
      DisplayPage::Profile(index) => self.profile_page_rows(index),
      DisplayPage::Prompt { .. } => Vec::new(),
    }
  }

  pub(super) fn stale_start(&self) -> usize {
    self.monitors.len()
  }

  /// Names of saved configurations whose monitor is not present.
  pub(super) fn stale_names(&self) -> Vec<String> {
    self
      .config
      .monitors
      .iter()
      .filter(|(name, _)| !self.monitors.iter().any(|monitor| &monitor.name == name))
      .map(|(name, _)| name.clone())
      .collect()
  }

  pub(super) fn stale_name(&self, index: usize) -> Option<String> {
    let offset = index.checked_sub(self.stale_start())?;
    self.stale_names().get(offset).cloned()
  }

  pub(super) fn detail_settings(&self) -> Vec<MonitorSetting> {
    let index = match self.monitor_index() {
      Some(index) => index,
      None => return vec![],
    };
    let Some(monitor) = self.monitors.get(index) else {
      return vec![];
    };
    if !monitor.connected {
      return vec![];
    }
    let mut settings = vec![
      MonitorSetting::Resolution,
      MonitorSetting::RefreshRate,
      MonitorSetting::Scale,
      MonitorSetting::Position,
      MonitorSetting::Orientation,
      MonitorSetting::Enabled,
      MonitorSetting::Mirror,
      MonitorSetting::BitDepth,
    ];
    if self.vrr_supported() {
      settings.push(MonitorSetting::Vrr);
    }
    if self.hdr_supported() && monitor.has_10bit() {
      settings.push(MonitorSetting::Hdr);
    }
    settings.push(MonitorSetting::Dpms);
    if self.hdr_supported() && monitor.has_10bit() {
      settings.push(MonitorSetting::SdrBrightness);
      settings.push(MonitorSetting::SdrSaturation);
    }
    settings.push(MonitorSetting::Workspaces);
    settings.push(MonitorSetting::Primary);
    settings
  }

  pub(super) fn monitor_index(&self) -> Option<usize> {
    match self.page {
      DisplayPage::Detail(index) | DisplayPage::Picker { monitor: index, .. } => Some(index),
      DisplayPage::Home
      | DisplayPage::NightLight
      | DisplayPage::Profiles
      | DisplayPage::Profile(_)
      | DisplayPage::Prompt { .. } => None,
    }
  }

  /// Home: connected monitors, saved configurations of absent monitors,
  /// Profiles and Refresh.
  pub(super) fn home_rows(&self) -> Vec<Row<Item>> {
    let lang = self.lang;
    let mut rows: Vec<Row<Item>> = Vec::new();
    for (index, monitor) in self.monitors.iter().enumerate() {
      let role = if self.config.primary_monitor.as_deref() == Some(monitor.name.as_str()) {
        tr(lang, "control_center.primary_ead365")
      } else if monitor.focused {
        tr(lang, "control_center.focused")
      } else {
        ""
      };
      let power = if monitor.disabled || monitor.dpms_status == "off" {
        tr(lang, "control_center.off")
      } else {
        tr(lang, "control_center.on")
      };
      let mut detail = format!(
        "{}x{} @ {:.3} Hz  ·  {}x{}  ·  {} {}  ·  {power}",
        monitor.width,
        monitor.height,
        monitor.refresh_rate,
        monitor.x,
        monitor.y,
        tr(lang, "control_center.scale"),
        monitor.scale,
      );
      if !role.is_empty() {
        detail.push_str(&format!("  ·  {role}"));
      }
      rows.push(
        Row::submenu(Item::Monitor(index), monitor.name.clone())
          .icon(icons::MONITOR)
          .detail(detail),
      );
    }
    for (offset, name) in self.stale_names().into_iter().enumerate() {
      let persisted = self.config.persisted(&name);
      rows.push(
        Row::submenu(Item::Stale(offset), name)
          .icon(icons::LINK_OFF)
          .detail(format!(
            "{}  ·  {}",
            tr(lang, "control_center.disconnected"),
            persisted
              .mode
              .as_deref()
              .unwrap_or(tr(lang, "control_center.configured")),
          )),
      );
    }
    if rows.is_empty() {
      rows.push(Row::info(
        tr(lang, "control_center.no_monitors_found_is_hyprland_running"),
        "",
      ));
    }
    rows.push(
      Row::submenu(
        Item::NightLight,
        tr(lang, "control_center.night_light.title"),
      )
      .icon(icons::BRIGHTNESS),
    );
    rows.push(
      Row::submenu(Item::Profiles, tr(lang, "control_center.profiles"))
        .icon(icons::PROFILE)
        .detail(self.state.profiles.len().to_string()),
    );
    rows.push(Row::separator());
    rows.push(Row::action(Item::Refresh, tr(lang, "control_center.refresh")).icon(icons::REFRESH));
    rows
  }

  /// Detail of a monitor: its information, one row per setting and the
  /// actions; a disconnected monitor only offers to remove its saved
  /// configuration.
  pub(super) fn detail_rows(&self, index: usize) -> Vec<Row<Item>> {
    let lang = self.lang;
    let connected = self.monitors.get(index).filter(|monitor| monitor.connected);
    let Some(monitor) = connected else {
      return vec![
        Row::info(tr(lang, "control_center.monitor_disconnected"), ""),
        Row::section(tr(lang, "control_center.danger_zone")),
        Row::destructive(Item::RemoveConfig, tr(lang, "control_center.remove_config"))
          .icon(icons::DELETE),
      ];
    };
    let mut rows = Vec::new();
    let info = monitor.info_rows();
    if !info.is_empty() {
      rows.push(Row::section(tr(lang, "control_center.monitor")));
      rows.extend(
        info
          .into_iter()
          .map(|(label, value)| Row::info(label, value)),
      );
    }
    rows.push(Row::section(tr(lang, "control_center.configuration")));
    rows.extend(self.detail_settings().into_iter().map(|setting| {
      Row::submenu(Item::Setting(setting), setting_label(lang, setting))
        .icon(setting_icon(setting))
        .detail(self.setting_value(monitor, setting))
    }));
    rows.push(Row::section(tr(lang, "control_center.section_actions")));
    rows.push(Row::action(Item::Apply, tr(lang, "control_center.apply")).icon(icons::APPLY));
    rows.push(
      Row::action(Item::Reset, tr(lang, "control_center.restore_default")).icon(icons::RESTORE),
    );
    rows
  }

  /// Current value of `setting`, preferring the saved override.
  pub(super) fn setting_value(&self, monitor: &Monitor, setting: MonitorSetting) -> String {
    let lang = self.lang;
    let persisted = self.config.persisted(&monitor.name);
    match setting {
      MonitorSetting::Resolution => persisted
        .mode
        .clone()
        .and_then(|mode| mode.split('@').next().map(str::to_string))
        .unwrap_or_else(|| format!("{}x{}", monitor.width, monitor.height)),
      MonitorSetting::RefreshRate => format!(
        "{:.3} Hz",
        persisted
          .mode
          .as_deref()
          .and_then(rate_of)
          .unwrap_or(monitor.refresh_rate)
      ),
      MonitorSetting::Scale => persisted.scale.unwrap_or(monitor.scale).to_string(),
      MonitorSetting::Position => persisted
        .position
        .clone()
        .unwrap_or(format!("{}x{}", monitor.x, monitor.y)),
      MonitorSetting::Orientation => {
        transform_label(lang, persisted.transform.unwrap_or(monitor.transform))
      }
      MonitorSetting::Enabled => on_off(lang, persisted.disabled != Some(true)),
      MonitorSetting::Mirror => self
        .current_mirror(monitor)
        .filter(|mirror| !mirror.is_empty())
        .unwrap_or_else(|| tr(lang, "control_center.none").into()),
      MonitorSetting::BitDepth => self.bitdepth_label(monitor, &persisted),
      MonitorSetting::Vrr => vrr_label(lang, persisted.vrr.unwrap_or(monitor.vrr)),
      MonitorSetting::Hdr => hdr_label(lang, persisted.hdr.unwrap_or(0)),
      MonitorSetting::Dpms => if monitor.dpms_status == "off" {
        tr(lang, "control_center.off")
      } else {
        tr(lang, "control_center.on")
      }
      .into(),
      MonitorSetting::SdrBrightness => persisted
        .sdr_brightness
        .or(monitor.info.sdr_brightness)
        .map(|value| format!("{value:.2}"))
        .unwrap_or_else(|| tr(lang, "control_center.auto").into()),
      MonitorSetting::SdrSaturation => persisted
        .sdr_saturation
        .or(monitor.info.sdr_saturation)
        .map(|value| format!("{value:.2}"))
        .unwrap_or_else(|| tr(lang, "control_center.auto").into()),
      MonitorSetting::Workspaces => {
        let workspaces = self.config.workspaces_of(&monitor.name);
        if workspaces.is_empty() {
          tr(lang, "control_center.unbound").into()
        } else {
          workspaces
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<_>>()
            .join(", ")
        }
      }
      MonitorSetting::Primary => on_off(
        lang,
        self.config.primary_monitor.as_deref() == Some(monitor.name.as_str()),
      ),
    }
  }

  pub(super) fn current_mirror(&self, monitor: &Monitor) -> Option<String> {
    self
      .config
      .persisted(&monitor.name)
      .mirror
      .or_else(|| monitor.info.mirror_of.clone())
  }

  pub(super) fn bitdepth_label(&self, monitor: &Monitor, persisted: &PersistedMonitor) -> String {
    if !monitor.has_10bit() {
      return tr(self.lang, "control_center.unsupported").into();
    }
    match persisted.bitdepth {
      Some(10) => "10".into(),
      Some(8) => "8".into(),
      _ if monitor.info.current_format.contains("10") => "10".into(),
      _ => "8".into(),
    }
  }

  pub(super) fn vrr_supported(&self) -> bool {
    self.version >= (0, 31)
  }

  pub(super) fn hdr_supported(&self) -> bool {
    self.version >= (0, 42)
  }

  pub(super) fn remove_button(&mut self) {
    let index = self.monitor_index();
    let name = match index {
      Some(index) if index < self.monitors.len() => Some(self.monitors[index].name.clone()),
      Some(index) => self.stale_name(index),
      None => None,
    };
    let Some(name) = name else {
      return;
    };
    self
      .config
      .monitors
      .retain(|(existing, _)| existing != &name);
    self
      .config
      .workspaces
      .retain(|(monitor, _)| monitor != &name);
    if self.config.primary_monitor.as_deref() == Some(&name) {
      self.config.primary_monitor = None;
      self.state.primary_monitor = None;
    }
    let config = self.config.clone();
    let state = self.state.clone();
    let lang = self.lang;
    let removed = tr(lang, "control_center.config_removed").to_string();
    self.action = Some(self.manager.spawn(move |_| {
      backend::save_config(&config)?;
      backend::save_state(&state)?;
      Ok(JobData::Action(format!("{removed} · {name}")))
    }));
    self.go(DisplayPage::Home);
  }

  pub(super) fn reset_button(&mut self) {
    let Some(index) = self.monitor_index() else {
      return;
    };
    let Some(monitor) = self.monitors.get(index) else {
      return;
    };
    let name = monitor.name.clone();
    self
      .config
      .monitors
      .retain(|(existing, _)| existing != &name);
    let config = self.config.clone();
    self.spawn_apply_with_config(
      config,
      format!(
        "{} · {name}",
        tr(self.lang, "control_center.default_applied")
      ),
    );
  }

  pub(super) fn apply_button(&mut self) {
    let Some(index) = self.monitor_index() else {
      return;
    };
    let Some(monitor) = self.monitors.get(index) else {
      return;
    };
    let name = monitor.name.clone();
    let _persisted = self.config.persisted(&name);
    let previous = monitor.clone();
    let previous_config = self.config.clone();
    self.arm_revert(name.clone(), previous, previous_config, Instant::now());
    self.spawn_apply_with_config(
      self.config.clone(),
      format!("{} · {name}", tr(self.lang, "control_center.applied")),
    );
  }
}

/// The item's own icon for each monitor setting.
pub(super) fn setting_icon(setting: MonitorSetting) -> &'static str {
  match setting {
    MonitorSetting::Resolution => icons::ASPECT_RATIO,
    MonitorSetting::RefreshRate => icons::SINE_WAVE,
    MonitorSetting::Scale => icons::ZOOM,
    MonitorSetting::Position => icons::ARROW_ALL,
    MonitorSetting::Orientation => icons::ROTATE,
    MonitorSetting::Enabled => icons::POWER,
    MonitorSetting::Mirror => icons::MONITOR_MULTIPLE,
    MonitorSetting::BitDepth => icons::PALETTE,
    MonitorSetting::Vrr => icons::SYNC,
    MonitorSetting::Hdr => icons::HDR,
    MonitorSetting::Dpms => icons::SLEEP,
    MonitorSetting::SdrBrightness => icons::BRIGHTNESS,
    MonitorSetting::SdrSaturation => icons::CONTRAST,
    MonitorSetting::Workspaces => icons::GRID,
    MonitorSetting::Primary => icons::STAR,
  }
}

pub(super) fn trimmed_rate(rate: f64) -> String {
  let rounded = (rate * 1000.0).round() / 1000.0;
  if rounded.fract() < 0.0005 {
    format!("{}", rounded as u64)
  } else {
    format!("{rounded:.3}")
  }
}

pub(super) fn rate_of(mode: &str) -> Option<f64> {
  mode.rsplit('@').next()?.parse::<f64>().ok()
}

pub(super) fn setting_label(lang: Lang, setting: MonitorSetting) -> String {
  match setting {
    MonitorSetting::Resolution => tr(lang, "control_center.resolution").into(),
    MonitorSetting::RefreshRate => tr(lang, "control_center.refresh_rate").into(),
    MonitorSetting::Scale => tr(lang, "control_center.scale").into(),
    MonitorSetting::Position => tr(lang, "control_center.position").into(),
    MonitorSetting::Primary => tr(lang, "control_center.primary_monitor").into(),
    MonitorSetting::Enabled => tr(lang, "control_center.monitor_enabled").into(),
    MonitorSetting::Mirror => tr(lang, "control_center.mirror").into(),
    MonitorSetting::BitDepth => tr(lang, "control_center.color_depth").into(),
    MonitorSetting::Vrr => tr(lang, "control_center.vrr").into(),
    MonitorSetting::Hdr => tr(lang, "control_center.hdr").into(),
    MonitorSetting::Dpms => "DPMS".into(),
    MonitorSetting::SdrBrightness => tr(lang, "control_center.sdr_brightness").into(),
    MonitorSetting::SdrSaturation => tr(lang, "control_center.sdr_saturation").into(),
    MonitorSetting::Workspaces => tr(lang, "control_center.workspaces").into(),
    MonitorSetting::Orientation => tr(lang, "control_center.orientation").into(),
  }
}

pub(super) fn transform_label(lang: Lang, transform: i32) -> String {
  match transform {
    0 => tr(lang, "control_center.normal").into(),
    1 => tr(lang, "control_center.90").into(),
    2 => tr(lang, "control_center.180").into(),
    3 => tr(lang, "control_center.270").into(),
    4 => tr(lang, "control_center.flipped").into(),
    5 => tr(lang, "control_center.flipped_90").into(),
    6 => tr(lang, "control_center.flipped_180").into(),
    7 => tr(lang, "control_center.flipped_270").into(),
    _ => format!("{transform}"),
  }
}

pub(super) fn vrr_label(lang: Lang, vrr: i32) -> String {
  match vrr {
    0 => tr(lang, "control_center.disabled").into(),
    1 => tr(lang, "control_center.enabled").into(),
    2 => tr(lang, "control_center.fullscreen_only").into(),
    _ => format!("{vrr}"),
  }
}

pub(super) fn hdr_label(lang: Lang, hdr: i32) -> String {
  if hdr != 0 {
    tr(lang, "control_center.forced").into()
  } else {
    tr(lang, "control_center.auto_d2c2e5").into()
  }
}

pub(super) fn on_off(lang: Lang, on: bool) -> String {
  if on {
    tr(lang, "control_center.yes").into()
  } else {
    tr(lang, "control_center.no").into()
  }
}
