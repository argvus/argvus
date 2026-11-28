use super::*;

struct Setting {
  key: &'static str,
  label: &'static str,
  values: &'static [&'static str],
  default: usize,
}

const SETTINGS: &[Setting] = &[
  Setting {
    key: "position",
    label: "position",
    values: &["bottom", "top", "left", "right"],
    default: 0,
  },
  Setting {
    key: "theme",
    label: "theme",
    values: &[
      "native",
      "macos-glass",
      "macos-dark",
      "frosted-glass",
      "dark-minimal",
      "light-minimal",
      "transparent",
      "matte",
      "graphite",
      "neon",
      "classic-plank",
      "retro",
    ],
    default: 0,
  },
  Setting {
    key: "alignment",
    label: "dock_alignment",
    values: &["start", "center", "end", "fill"],
    default: 1,
  },
  Setting {
    key: "visibility",
    label: "dock_visibility",
    values: &[
      "always_visible",
      "auto_hide",
      "intelligent_hide",
      "dodge_active",
      "dodge_any",
      "dodge_maximized",
    ],
    default: 0,
  },
  Setting {
    key: "icon_size",
    label: "dock_icon_size",
    values: &["24", "32", "40", "48", "56", "64", "80", "96"],
    default: 3,
  },
  Setting {
    key: "magnification",
    label: "dock_magnification",
    values: &["false", "true"],
    default: 1,
  },
  Setting {
    key: "magnification_strength",
    label: "dock_strength",
    values: &["1.0", "1.2", "1.5", "1.8"],
    default: 2,
  },
  Setting {
    key: "running_indicators",
    label: "dock_indicators",
    values: &["false", "true"],
    default: 1,
  },
  Setting {
    key: "monitor_policy",
    label: "dock_monitor",
    values: &["primary", "active", "all"],
    default: 2,
  },
  Setting {
    key: "launcher",
    label: "launcher",
    values: &["false", "true"],
    default: 0,
  },
  Setting {
    key: "lock_items",
    label: "lock",
    values: &["false", "true"],
    default: 0,
  },
  Setting {
    key: "show_running",
    label: "running",
    values: &["false", "true"],
    default: 1,
  },
  Setting {
    key: "show_pinned",
    label: "dock_pinned",
    values: &["false", "true"],
    default: 1,
  },
  Setting {
    key: "hide_delay",
    label: "dock_hide_delay",
    values: &["0", "150", "250", "500", "1000", "2000", "3000"],
    default: 2,
  },
];

impl AppearanceApp {
  fn dock_value(&self, setting: &Setting) -> String {
    self
      .state
      .dock
      .get(setting.key)
      .map(|value| {
        value
          .as_str()
          .map_or_else(|| value.to_string(), str::to_owned)
      })
      .unwrap_or_else(|| setting.values[setting.default].into())
  }

  pub(super) fn dock_rows(&self) -> Vec<argvus_tui::menu::Row<Item>> {
    let mut rows = vec![self.submenu(Item::DockSettings, "control_center.dock_settings")];
    for (index, setting) in SETTINGS.iter().enumerate() {
      let value = self.dock_value(setting);
      let label = format!("control_center.{}", setting.label);
      let value_key = match value.as_str() {
        "true" => "enabled",
        "false" => "disabled",
        "primary" => "primary_monitor",
        value => value,
      };
      let detail = if setting.key == "theme" {
        tr(self.lang, &format!("control_center.dock_theme_{value}")).to_owned()
      } else if setting.key == "alignment" {
        tr(self.lang, &format!("control_center.dock_align_{value}")).to_owned()
      } else if value_key
        .bytes()
        .next()
        .is_some_and(|byte| byte.is_ascii_digit())
      {
        value
      } else {
        tr(self.lang, &format!("control_center.{value_key}")).to_owned()
      };
      rows.push(self.submenu(Item::DockOption(index), &label).detail(detail));
    }
    rows.push(self.submenu(Item::DockReset, "control_center.reset_defaults"));
    rows
  }

  pub(super) fn change_dock(&mut self, index: Option<usize>) {
    if index.is_none() {
      self.apply_result(|| {
        backend::commands::run_argvus_route("config", &["dock-reset"], &[])?;
        Ok(String::new())
      });
      return;
    }
    let mut patch = serde_json::Map::new();
    for (i, setting) in SETTINGS.iter().enumerate() {
      if index.is_some_and(|index| index != i) {
        continue;
      }
      let next = if index.is_some() {
        let current = self.dock_value(setting);
        setting
          .values
          .iter()
          .position(|value| *value == current)
          .map_or(setting.default, |index| (index + 1) % setting.values.len())
      } else {
        setting.default
      };
      let value = setting.values[next];
      patch.insert(
        format!("/dock/{}", setting.key),
        serde_json::from_str(value).unwrap_or_else(|_| serde_json::Value::String(value.into())),
      );
    }
    self.apply_result(move || {
      backend::commands::run_argvus_route(
        "config",
        &["patch", &serde_json::Value::Object(patch).to_string()],
        &[],
      )?;
      Ok(String::new())
    });
  }
}
