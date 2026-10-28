//! Option pickers of each monitor setting: the choices offered, the current one and applying a selection.

use super::rows::{hdr_label, rate_of, transform_label, trimmed_rate, vrr_label};
use super::*;

impl DisplaysApp {
  pub(super) fn exit_picker(&mut self) {
    if let DisplayPage::Picker { monitor, .. } = self.page {
      self.go(DisplayPage::Detail(monitor));
    }
  }

  /// Options of the open setting: Choice rows with `●` on the current
  /// value, Toggle rows for workspaces, and a Value row for a custom value.
  pub(super) fn picker_rows(
    &self,
    monitor_index: usize,
    setting: MonitorSetting,
  ) -> Vec<Row<Item>> {
    let lang = self.lang;
    let Some(monitor) = self.monitors.get(monitor_index) else {
      return vec![Row::info(
        tr(lang, "control_center.no_options_available"),
        "",
      )];
    };
    let mut rows = Vec::new();
    let mut custom = None;
    for (index, option) in self
      .options_for(monitor_index, setting)
      .into_iter()
      .enumerate()
    {
      if option.prompt.is_some() {
        custom = Some(option);
        continue;
      }
      let item = Item::Option(index);
      rows.push(match (&option.persist, setting) {
        (PersistChange::Workspace(workspace, target), MonitorSetting::Workspaces) => {
          let (label, detail) = option
            .label
            .split_once("  ·  ")
            .map(|(label, detail)| (label.to_string(), detail.to_string()))
            .unwrap_or((workspace.to_string(), String::new()));
          // Bound here means selecting it unbinds (target `None`).
          Row::toggle(item, label, target.is_none()).detail(detail)
        }
        (PersistChange::None, _)
          if option.label == tr(lang, "control_center.unsupported_7c5d5f") =>
        {
          // "Unsupported" placeholder of a gated setting.
          Row::info(option.label, "")
        }
        _ => {
          let current = self.option_is_current(monitor, setting, index, &option);
          Row::choice(item, option.label, current)
        }
      });
    }
    if let Some(option) = custom {
      rows.push(Row::separator());
      rows.push(
        Row::value(
          Item::Custom,
          option.label.trim_start_matches("✎ ").to_string(),
          self.setting_value(monitor, setting),
          None,
        )
        .icon(icons::EDIT),
      );
    }
    if rows.is_empty() {
      rows.push(Row::info(
        tr(lang, "control_center.no_options_available"),
        "",
      ));
    }
    rows
  }

  /// Whether `option` is the value currently in effect for `setting`.
  pub(super) fn option_is_current(
    &self,
    monitor: &Monitor,
    setting: MonitorSetting,
    index: usize,
    option: &PickerOption,
  ) -> bool {
    let persisted = self.config.persisted(&monitor.name);
    let current_mode = persisted.mode.clone().unwrap_or_else(|| {
      format!(
        "{}x{}@{}",
        monitor.width,
        monitor.height,
        trimmed_rate(monitor.refresh_rate)
      )
    });
    let size_of = |mode: &str| mode.split('@').next().unwrap_or_default().to_string();
    match (&option.persist, setting) {
      (PersistChange::Mode(mode), MonitorSetting::Resolution) => {
        size_of(mode) == size_of(&current_mode)
      }
      (PersistChange::Mode(mode), _) => {
        size_of(mode) == size_of(&current_mode)
          && match (rate_of(mode), rate_of(&current_mode)) {
            (Some(left), Some(right)) => (left - right).abs() < 0.01,
            _ => false,
          }
      }
      (PersistChange::Scale(value), _) => {
        (value - persisted.scale.unwrap_or(monitor.scale)).abs() < 0.001
      }
      (PersistChange::Disabled(disabled), _) => *disabled == (persisted.disabled == Some(true)),
      (PersistChange::Mirror(mirror), _) => {
        *mirror == self.current_mirror(monitor).unwrap_or_default()
      }
      (PersistChange::BitDepth(depth), _) => {
        depth.to_string() == self.bitdepth_label(monitor, &persisted)
      }
      (PersistChange::Vrr(value), _) => *value == persisted.vrr.unwrap_or(monitor.vrr),
      (PersistChange::Hdr(value), _) => *value == persisted.hdr.unwrap_or(0),
      (PersistChange::Dpms(on), _) => *on == (monitor.dpms_status != "off"),
      (PersistChange::Primary, _) => {
        self.config.primary_monitor.as_deref() == Some(option.args.as_str())
      }
      (PersistChange::None, MonitorSetting::Orientation) => {
        index as i32 == persisted.transform.unwrap_or(monitor.transform)
      }
      _ => false,
    }
  }

  pub(super) fn picker_options(&self) -> Vec<PickerOption> {
    match self.monitor_ctx() {
      Some((monitor_index, setting)) => self.options_for(monitor_index, setting),
      None => vec![],
    }
  }

  /// Options offered for `setting` of the monitor `monitor_index`.
  pub(super) fn options_for(
    &self,
    monitor_index: usize,
    setting: MonitorSetting,
  ) -> Vec<PickerOption> {
    let Some(monitor) = self.monitors.get(monitor_index) else {
      return vec![];
    };
    match setting {
      MonitorSetting::Resolution => resolution_options(monitor),
      MonitorSetting::RefreshRate => refresh_options(monitor),
      MonitorSetting::Scale => {
        let mut options = scale_options(monitor);
        options.push(PickerOption {
          label: tr(self.lang, "control_center.custom").into(),
          args: String::new(),
          persist: PersistChange::None,
          prompt: Some(PromptGoal::Scale(monitor_index)),
        });
        options
      }
      MonitorSetting::Position => {
        let mut options = self.position_options(monitor_index);
        options.push(PickerOption {
          label: tr(self.lang, "control_center.custom").into(),
          args: String::new(),
          persist: PersistChange::None,
          prompt: Some(PromptGoal::Position(monitor_index)),
        });
        options
      }
      MonitorSetting::Primary => self.primary_options(),
      MonitorSetting::Enabled => enabled_options(monitor, self.lang),
      MonitorSetting::Mirror => mirror_options(monitor, &self.monitors, self.lang),
      MonitorSetting::BitDepth if monitor.has_10bit() => bitdepth_options(monitor, self.lang),
      MonitorSetting::Vrr if self.vrr_supported() => vrr_options(monitor, self.lang),
      MonitorSetting::Hdr if self.hdr_supported() && monitor.has_10bit() => {
        hdr_options(monitor, self.lang)
      }
      MonitorSetting::Dpms => dpms_options(monitor, self.lang),
      MonitorSetting::Orientation => orientation_options(monitor, self.lang),
      MonitorSetting::SdrBrightness if self.hdr_supported() && monitor.has_10bit() => {
        let mut options = sdr_brightness_options(self.lang, monitor);
        options.push(PickerOption {
          label: tr(self.lang, "control_center.custom").into(),
          args: String::new(),
          persist: PersistChange::None,
          prompt: Some(PromptGoal::SdrBrightness(monitor_index)),
        });
        options
      }
      MonitorSetting::SdrSaturation if self.hdr_supported() && monitor.has_10bit() => {
        let mut options = sdr_saturation_options(self.lang, monitor);
        options.push(PickerOption {
          label: tr(self.lang, "control_center.custom").into(),
          args: String::new(),
          persist: PersistChange::None,
          prompt: Some(PromptGoal::SdrSaturation(monitor_index)),
        });
        options
      }
      MonitorSetting::Workspaces => self.workspace_options(monitor_index),
      MonitorSetting::BitDepth => vec![PickerOption {
        label: tr(self.lang, "control_center.unsupported_7c5d5f").into(),
        args: String::new(),
        persist: PersistChange::None,
        prompt: None,
      }],
      MonitorSetting::Vrr | MonitorSetting::Hdr => vec![PickerOption {
        label: tr(self.lang, "control_center.unsupported_7c5d5f").into(),
        args: String::new(),
        persist: PersistChange::None,
        prompt: None,
      }],
      MonitorSetting::SdrBrightness | MonitorSetting::SdrSaturation => vec![PickerOption {
        label: tr(self.lang, "control_center.unsupported_7c5d5f").into(),
        args: String::new(),
        persist: PersistChange::None,
        prompt: None,
      }],
    }
  }

  pub(super) fn primary_options(&self) -> Vec<PickerOption> {
    self
      .monitors
      .iter()
      .filter(|monitor| monitor.connected)
      .map(|monitor| PickerOption {
        label: monitor.name.clone(),
        args: monitor.name.clone(),
        persist: PersistChange::Primary,
        prompt: None,
      })
      .collect()
  }

  pub(super) fn workspace_options(&self, index: usize) -> Vec<PickerOption> {
    let Some(monitor) = self.monitors.get(index) else {
      return vec![];
    };
    let name = monitor.name.clone();
    let owned = self.config.workspaces_of(&name);
    let mut options = Vec::new();
    for workspace in 1..=10 {
      let bound_here = owned.contains(&workspace);
      let bound_elsewhere = self
        .config
        .workspaces
        .iter()
        .filter(|(other, _)| *other != name)
        .any(|(_, ids)| ids.contains(&workspace));
      let label = if bound_here {
        format!(
          "{}  ·  {}",
          workspace,
          tr(self.lang, "control_center.this_monitor")
        )
      } else if bound_elsewhere {
        format!(
          "{}  ·  {}",
          workspace,
          tr(self.lang, "control_center.another_monitor")
        )
      } else {
        format!(
          "{}  ·  {}",
          workspace,
          tr(self.lang, "control_center.unbound")
        )
      };
      let persist = if bound_here {
        PersistChange::Workspace(workspace, None)
      } else {
        PersistChange::Workspace(workspace, Some(name.clone()))
      };
      options.push(PickerOption {
        label,
        args: String::new(),
        persist,
        prompt: None,
      });
    }
    options
  }

  pub(super) fn position_options(&self, index: usize) -> Vec<PickerOption> {
    let Some(monitor) = self.monitors.get(index) else {
      return vec![];
    };
    let mut options = vec![PickerOption {
      label: tr(self.lang, "control_center.top_left_corner_0_0").into(),
      args: monitor.position_args(0, 0),
      persist: PersistChange::Position(0, 0),
      prompt: None,
    }];
    for (other_index, other) in self.monitors.iter().enumerate() {
      if other_index == index || !other.connected {
        continue;
      }
      let placements = [
        (
          tr(self.lang, "control_center.to_the_right_of"),
          other.x + other.width as i32,
          other.y,
        ),
        (
          tr(self.lang, "control_center.to_the_left_of"),
          other.x - monitor.width as i32,
          other.y,
        ),
        (
          tr(self.lang, "control_center.below"),
          other.x,
          other.y + other.height as i32,
        ),
        (
          tr(self.lang, "control_center.above"),
          other.x,
          other.y - monitor.height as i32,
        ),
      ];
      for (label, x, y) in placements {
        options.push(PickerOption {
          label: format!("{label} {}", other.name),
          args: monitor.position_args(x, y),
          persist: PersistChange::Position(x, y),
          prompt: None,
        });
      }
    }
    options.sort_by(|left, right| left.label.cmp(&right.label));
    options
  }

  /// Applies the picker option `index` (immediate, as before; risky options
  /// arm the revert countdown) and goes back to the monitor.
  pub(super) fn apply_picker_selection(&mut self, index: usize) {
    let Some((monitor_index, setting)) = self.monitor_ctx() else {
      return;
    };
    let options = self.picker_options();
    let Some(option) = options.get(index).cloned() else {
      return;
    };
    if let Some(prompt) = option.prompt {
      self.open_prompt(prompt);
      return;
    }
    if setting == MonitorSetting::Primary {
      if let Some(monitor) = self.monitors.get(monitor_index) {
        let name = monitor.name.clone();
        self.config.primary_monitor = Some(name.clone());
        self.state.primary_monitor = Some(name.clone());
        let config = self.config.clone();
        let state = self.state.clone();
        let message = format!(
          "{}: {name}",
          tr(self.lang, "control_center.primary_monitor")
        );
        self.action = Some(self.manager.spawn(move |_| {
          backend::save_config(&config)?;
          backend::save_state(&state)?;
          Ok(JobData::Action(message))
        }));
      }
      self.go(DisplayPage::Detail(monitor_index));
      return;
    }
    let Some(monitor) = self.monitors.get(monitor_index).cloned() else {
      return;
    };
    let name = monitor.name.clone();
    let previous = monitor.clone();
    let prev_config = self.config.clone();
    let mut config = self.config.clone();
    Self::apply_persist(&mut config, &name, &option.persist);
    if option_is_risky(&option.persist) {
      self.arm_revert(name.clone(), previous, prev_config, Instant::now());
    }
    let persist = option.persist.clone();
    let message = format!(
      "{} · {}",
      monitor.name,
      option.label.split("  ·  ").next().unwrap_or("")
    );
    self.config = config.clone();
    self.go(DisplayPage::Detail(monitor_index));
    self.action = Some(self.manager.spawn(move |_| {
      match persist {
        PersistChange::Dpms(true) => backend::set_dpms(&name, true)?,
        PersistChange::Dpms(_) => backend::set_dpms(&name, false)?,
        PersistChange::Workspace(workspace, target) => {
          backend::apply_workspace(workspace, target.as_deref())?;
          backend::save_config(&config)?;
        }
        _ => {
          backend::apply_all(&config)?;
        }
      }
      Ok(JobData::Action(message))
    }));
  }
}

pub(super) fn resolution_options(monitor: &Monitor) -> Vec<PickerOption> {
  let mut unique: Vec<(u32, u32, f64)> = Vec::new();
  for mode in &monitor.modes {
    if !unique
      .iter()
      .any(|(w, h, _)| *w == mode.width && *h == mode.height)
    {
      unique.push((mode.width, mode.height, mode.refresh_rate));
    }
  }
  unique.sort_by(|left, right| {
    (right.0 * right.1).cmp(&(left.0 * left.1)).then(
      right
        .2
        .partial_cmp(&left.2)
        .unwrap_or(std::cmp::Ordering::Equal),
    )
  });
  unique
    .into_iter()
    .map(|(width, height, rate)| PickerOption {
      label: format!("{width}x{height}  ·  {:.3} Hz", rate),
      args: monitor.resolution_args(width, height, rate),
      persist: PersistChange::Mode(format!("{}x{}@{}", width, height, trimmed_rate(rate))),
      prompt: None,
    })
    .collect()
}

pub(super) fn refresh_options(monitor: &Monitor) -> Vec<PickerOption> {
  let mut modes = monitor.modes.clone();
  modes.sort_by(|left, right| {
    right
      .refresh_rate
      .partial_cmp(&left.refresh_rate)
      .unwrap_or(std::cmp::Ordering::Equal)
  });
  modes
    .into_iter()
    .map(|mode: crate::model::Mode| PickerOption {
      label: mode.label(),
      args: monitor.mode_id_args(&mode),
      persist: PersistChange::Mode(format!(
        "{}x{}@{}",
        mode.width,
        mode.height,
        trimmed_rate(mode.refresh_rate)
      )),
      prompt: None,
    })
    .collect()
}

pub(super) fn scale_options(monitor: &Monitor) -> Vec<PickerOption> {
  [1.0_f64, 0.75, 1.1, 1.25, 1.5, 2.0]
    .into_iter()
    .map(|scale| PickerOption {
      label: format!("{scale}"),
      args: monitor.scale_args(scale),
      persist: PersistChange::Scale(scale),
      prompt: None,
    })
    .collect()
}

pub(super) fn enabled_options(monitor: &Monitor, lang: Lang) -> Vec<PickerOption> {
  vec![
    PickerOption {
      label: tr(lang, "control_center.enable").into(),
      args: monitor.enabled_with_current_args(),
      persist: PersistChange::Disabled(false),
      prompt: None,
    },
    PickerOption {
      label: tr(lang, "control_center.disable").into(),
      args: monitor.disabled_args(),
      persist: PersistChange::Disabled(true),
      prompt: None,
    },
  ]
}

pub(super) fn orientation_options(monitor: &Monitor, lang: Lang) -> Vec<PickerOption> {
  (0..8)
    .map(|transform| PickerOption {
      label: transform_label(lang, transform),
      args: monitor.transform_args(transform),
      persist: PersistChange::None,
      prompt: None,
    })
    .collect()
}

pub(super) fn mirror_options(
  monitor: &Monitor,
  monitors: &[Monitor],
  lang: Lang,
) -> Vec<PickerOption> {
  let mut options = vec![PickerOption {
    label: tr(lang, "control_center.no_mirror").into(),
    args: monitor.mirror_args(""),
    persist: PersistChange::Mirror(String::new()),
    prompt: None,
  }];
  for other in monitors {
    if other.name != monitor.name && other.connected {
      options.push(PickerOption {
        label: other.name.clone(),
        args: monitor.mirror_args(&other.name),
        persist: PersistChange::Mirror(other.name.clone()),
        prompt: None,
      });
    }
  }
  options
}

pub(super) fn bitdepth_options(monitor: &Monitor, lang: Lang) -> Vec<PickerOption> {
  let _ = lang;
  [8_i32, 10]
    .into_iter()
    .map(|bitdepth| PickerOption {
      label: format!("{bitdepth} bpp"),
      args: monitor.bitdepth_args(bitdepth),
      persist: PersistChange::BitDepth(bitdepth),
      prompt: None,
    })
    .collect()
}

pub(super) fn vrr_options(monitor: &Monitor, lang: Lang) -> Vec<PickerOption> {
  [0_i32, 1, 2]
    .into_iter()
    .map(|value| PickerOption {
      label: vrr_label(lang, value),
      args: monitor.vrr_args(value),
      persist: PersistChange::Vrr(value),
      prompt: None,
    })
    .collect()
}

pub(super) fn hdr_options(monitor: &Monitor, lang: Lang) -> Vec<PickerOption> {
  [0_i32, 1]
    .into_iter()
    .map(|value| PickerOption {
      label: hdr_label(lang, value),
      args: monitor.hdr_args(value),
      persist: PersistChange::Hdr(value),
      prompt: None,
    })
    .collect()
}

pub(super) fn dpms_options(_monitor: &Monitor, lang: Lang) -> Vec<PickerOption> {
  vec![
    PickerOption {
      label: tr(lang, "control_center.on_5ddab3").into(),
      args: String::new(),
      persist: PersistChange::Dpms(true),
      prompt: None,
    },
    PickerOption {
      label: tr(lang, "control_center.off_688c4e").into(),
      args: String::new(),
      persist: PersistChange::Dpms(false),
      prompt: None,
    },
  ]
}

pub(super) fn sdr_brightness_options(_lang: Lang, _monitor: &Monitor) -> Vec<PickerOption> {
  [0.25, 0.5, 0.75, 1.0]
    .into_iter()
    .map(|value| PickerOption {
      label: format!("{value:.2}"),
      args: String::new(),
      persist: PersistChange::None,
      prompt: None,
    })
    .collect()
}

pub(super) fn sdr_saturation_options(_lang: Lang, _monitor: &Monitor) -> Vec<PickerOption> {
  [0.5, 1.0, 1.5, 2.0]
    .into_iter()
    .map(|value| PickerOption {
      label: format!("{value:.2}"),
      args: String::new(),
      persist: PersistChange::None,
      prompt: None,
    })
    .collect()
}

pub(super) fn option_is_risky(persist: &PersistChange) -> bool {
  matches!(
    persist,
    PersistChange::Mode(_)
      | PersistChange::Position(_, _)
      | PersistChange::Mirror(_)
      | PersistChange::Disabled(true)
      | PersistChange::BitDepth(_)
  )
}
