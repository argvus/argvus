//! Rows and labels of the Services page lists.

use super::*;

impl ServicesApp {
  /// Builds the rows of a unit list: one row per unit with its Active and
  /// Enabled dots. The filter is not a row; it is cycled with `f`.
  pub(super) fn list_rows(&self) -> Vec<Row<Item>> {
    if self.page == ServicePage::Failed && self.filtered().is_empty() {
      return vec![Row::info(
        format!("[OK] {}", tr(self.lang, "control_center.no_failed_units")),
        "",
      )];
    }
    self
      .filtered()
      .iter()
      .enumerate()
      .map(|(index, unit)| {
        let mut label = unit.name.clone();
        if self.page == ServicePage::Failed && unit.scope == "user" {
          label.push_str(&format!(
            "   [{}]",
            tr(self.lang, "control_center.user_e7acba")
          ));
        }
        Row::submenu(Item::Unit(index), label).detail(self.status_cells(unit))
      })
      .collect()
  }

  pub(super) fn home_rows(&self) -> Vec<String> {
    let (system_active, system_enabled) = self.counts("system");
    let (user_active, user_enabled) = self.counts("user");
    let failed = self.units.iter().filter(|unit| unit.failed()).count();
    let boot = if self.previous_boot {
      tr(self.lang, "control_center.previous")
    } else {
      tr(self.lang, "control_center.current_c42247")
    };
    vec![
      format!(
        "{} {}  ·  {} {} · {} {}",
        AppConfig::icon(argvus_tui::icons::SETTINGS),
        tr(self.lang, "control_center.system"),
        system_active,
        tr(self.lang, "control_center.active_ae7190"),
        system_enabled,
        tr(self.lang, "control_center.enabled_26a3ab"),
      ),
      format!(
        "{} {}  ·  {} {} · {} {}",
        AppConfig::icon(argvus_tui::icons::USER),
        tr(self.lang, "control_center.user"),
        user_active,
        tr(self.lang, "control_center.active_ae7190"),
        user_enabled,
        tr(self.lang, "control_center.enabled_26a3ab"),
      ),
      format!(
        "{} {}  ·  {} {}",
        AppConfig::icon(argvus_tui::icons::WARNING),
        tr(self.lang, "control_center.failed"),
        failed,
        tr(self.lang, "control_center.failed_cc0486"),
      ),
      format!(
        "{} {}  ·  {} {} · {} {}",
        AppConfig::icon(argvus_tui::icons::LOGS),
        tr(self.lang, "control_center.logs"),
        tr(self.lang, "control_center.boot"),
        boot,
        tr(self.lang, "control_center.priority"),
        self.priority.as_deref().unwrap_or("0-7"),
      ),
    ]
  }

  /// Rows of the service details: the service information, then its
  /// actions. Info rows are read-only; actions reuse `detail_actions`.
  pub(super) fn detail_rows(&self) -> Vec<Row<Item>> {
    let mut rows = vec![Row::section(tr(self.lang, "control_center.service_bfe08e"))];
    let Some(u) = self.detail_or_selected_unit() else {
      rows.push(Row::info(
        tr(self.lang, "control_center.service_not_found"),
        "",
      ));
      return rows;
    };
    rows.extend([
      Row::info(tr(self.lang, "control_center.name"), u.name.clone()),
      Row::info(
        tr(self.lang, "control_center.description"),
        u.description.clone(),
      ),
      Row::info(tr(self.lang, "control_center.state"), state_value(u)),
      Row::info(
        tr(self.lang, "control_center.boot_f0640d"),
        file_state_label(self.lang, &u.file_state),
      ),
      Row::info(
        tr(self.lang, "control_center.main_pid"),
        u.main_pid
          .map(|v| v.to_string())
          .unwrap_or_else(|| "—".into()),
      ),
      Row::info(
        tr(self.lang, "control_center.unit_file"),
        u.fragment.clone().unwrap_or_else(|| "—".into()),
      ),
      Row::section(tr(self.lang, "control_center.section_actions")),
    ]);
    rows.extend(
      self
        .detail_actions()
        .into_iter()
        .map(|(action, label)| Row::action(Item::DetailAction(action), label)),
    );
    rows
  }

  /// The rows of the page that is currently a menu (lists and details).
  pub(super) fn menu_rows(&self) -> Vec<Row<Item>> {
    if self.page == ServicePage::Detail {
      self.detail_rows()
    } else {
      self.list_rows()
    }
  }

  pub(super) fn log_detail_lines(&self, index: usize) -> Vec<Line<'static>> {
    let filtered = self.filtered_logs();
    let Some(entry) = filtered.get(index) else {
      return vec![Line::from(tr(self.lang, "control_center.log_not_found"))];
    };
    let timestamp = entry.timestamp.clone().unwrap_or_else(|| "—".into());
    let unit = entry.unit.clone().unwrap_or_else(|| "kernel".into());
    let priority = entry.priority_name().to_string();
    let pid = entry.pid.clone().unwrap_or_else(|| "—".into());
    let boot = entry.boot_id.clone().unwrap_or_else(|| "—".into());
    let executable = entry.executable.clone().unwrap_or_else(|| "—".into());
    let message = entry.message.clone().unwrap_or_default();
    vec![
      Line::from(format!(
        "{}: {}",
        tr(self.lang, "control_center.timestamp"),
        timestamp
      )),
      Line::from(format!(
        "{}: {}",
        tr(self.lang, "control_center.unit"),
        unit
      )),
      Line::from(format!(
        "{}: {}",
        tr(self.lang, "control_center.priority"),
        priority
      )),
      Line::from(format!("PID: {}", pid)),
      Line::from(format!(
        "{}: {}",
        tr(self.lang, "control_center.executable"),
        executable
      )),
      Line::from(format!(
        "{}: {}",
        tr(self.lang, "control_center.boot"),
        boot
      )),
      Line::from(""),
      Line::from(message),
    ]
  }
}

pub(super) fn action_label(lang: Lang, action: &str) -> String {
  match action {
    "start" => tr(lang, "control_center.start"),
    "stop" => tr(lang, "control_center.stop"),
    "restart" => tr(lang, "control_center.restart"),
    "enable" => tr(lang, "control_center.enable"),
    "disable" => tr(lang, "control_center.disable"),
    "enable-now" => tr(lang, "control_center.enable_and_start"),
    "disable-now" => tr(lang, "control_center.disable_and_stop"),
    other => other,
  }
  .into()
}

pub(super) fn file_state_label(lang: Lang, state: &str) -> String {
  match state {
    "enabled" | "static" | "indirect" => tr(lang, "control_center.enabled_78438d").into(),
    "disabled" | "masked" => tr(lang, "control_center.disabled_483392").into(),
    _ => state.into(),
  }
}
