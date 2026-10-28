//! Rows and labels of the Power page lists.

use super::*;

impl PowerApp {
  pub(super) fn row_count(&self) -> usize {
    let Some(state) = &self.state else {
      return 0;
    };
    if state.is_laptop { 7 } else { 3 }
  }

  pub(super) fn rows(&self) -> Vec<String> {
    let Some(state) = &self.state else {
      return vec![tr(self.lang, "control_center.loading").into()];
    };
    let mut rows = Vec::new();

    if state.is_laptop {
      rows.push(format!(
        " {}  {}  ·  ▸ {}",
        AppConfig::icon(argvus_tui::icons::MONITOR),
        tr(self.lang, "control_center.on_lid_close_battery"),
        behavior_label(self.lang, state.lid[0].value()),
      ));
      rows.push(format!(
        " {}  {}  ·  ▸ {}",
        AppConfig::icon(argvus_tui::icons::ETHERNET),
        tr(self.lang, "control_center.on_lid_close_ac"),
        behavior_label(self.lang, state.lid[1].value()),
      ));
      rows.push(format!(
        " {}  {}  ·  ▸ {}",
        AppConfig::icon(argvus_tui::icons::POWER),
        tr(self.lang, "control_center.power_button"),
        behavior_label(self.lang, state.power_button.value()),
      ));
    }

    rows.push(if state.screen_off_supported {
      format!(
        " {}  {}  ·  ▸ {}",
        AppConfig::icon(argvus_tui::icons::MONITOR),
        tr(self.lang, "control_center.screen_off_after"),
        idle_label(state.screen_off_minutes.unwrap_or(0)),
      )
    } else {
      format!(
        " {}  {}  ·  {}",
        AppConfig::icon(argvus_tui::icons::MONITOR),
        tr(self.lang, "control_center.screen_off_after"),
        tr(self.lang, "control_center.via_argvus_hypridle"),
      )
    });

    rows.push(if state.lock_supported {
      format!(
        " {}  {}  ·  ▸ {}",
        AppConfig::icon(argvus_tui::icons::LOCK),
        tr(self.lang, "control_center.lock_screen_after"),
        idle_label(state.lock_minutes.unwrap_or(0)),
      )
    } else {
      format!(
        " {}  {}  ·  {}",
        AppConfig::icon(argvus_tui::icons::LOCK),
        tr(self.lang, "control_center.lock_screen_after"),
        tr(self.lang, "control_center.via_argvus_hypridle"),
      )
    });

    rows.push(format!(
      " {}  {}  ·  ▸ {}",
      AppConfig::icon(argvus_tui::icons::MONITOR),
      tr(self.lang, "control_center.keep_awake"),
      tr(
        self.lang,
        if state.keep_awake {
          "control_center.enabled"
        } else {
          "control_center.disabled"
        }
      ),
    ));

    if state.is_laptop {
      rows.push(self.capabilities_line());
    }

    rows
  }
}

pub(super) fn behavior_label(lang: Lang, value: &str) -> String {
  match value {
    "ignore" => tr(lang, "control_center.ignore").into(),
    "poweroff" => tr(lang, "control_center.power_off").into(),
    "reboot" => tr(lang, "control_center.reboot").into(),
    "halt" => tr(lang, "control_center.halt").into(),
    "suspend" => tr(lang, "control_center.suspend").into(),
    "hibernate" => tr(lang, "control_center.hibernate").into(),
    "suspend-then-hibernate" => tr(lang, "control_center.suspend_hibernate").into(),
    "lock" => tr(lang, "control_center.lock").into(),
    other => other.into(),
  }
}

pub(super) fn idle_label(minutes: u32) -> String {
  if minutes == 0 {
    String::from("Nunca")
  } else if minutes < 60 {
    format!("{minutes} min")
  } else {
    format!("{} h", minutes / 60)
  }
}
