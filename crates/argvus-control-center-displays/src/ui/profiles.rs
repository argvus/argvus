//! Saved display profiles: listing, applying and deleting them, and matching them to the connected monitors.

use super::*;

impl DisplaysApp {
  pub(super) fn delete_profile(&mut self, index: usize) {
    if index < self.state.profiles.len() {
      self.state.profiles.remove(index);
    }
    let state = self.state.clone();
    self.action = Some(self.manager.spawn(move |_| {
      backend::save_state(&state)?;
      Ok(JobData::Action(String::new()))
    }));
  }

  /// Saved profiles (Enter opens one) and New profile.
  pub(super) fn profile_rows(&self) -> Vec<Row<Item>> {
    let lang = self.lang;
    let mut rows: Vec<Row<Item>> = self
      .state
      .profiles
      .iter()
      .enumerate()
      .map(|(index, profile)| {
        let row = Row::submenu(Item::Profile(index), profile.name.clone());
        if self.state.active_profile.as_deref() == Some(profile.name.as_str()) {
          row.detail(tr(lang, "control_center.active"))
        } else {
          row
        }
      })
      .collect();
    if !rows.is_empty() {
      rows.push(Row::separator());
    }
    rows
      .push(Row::action(Item::NewProfile, tr(lang, "control_center.new_profile")).icon(icons::ADD));
    rows
  }

  /// One profile: Apply and Rename, with Delete in the Danger zone (D4).
  pub(super) fn profile_page_rows(&self, index: usize) -> Vec<Row<Item>> {
    let lang = self.lang;
    let Some(profile) = self.state.profiles.get(index) else {
      return vec![Row::info(
        tr(lang, "control_center.no_options_available"),
        "",
      )];
    };
    let mut rows = vec![
      Row::info(tr(lang, "control_center.profiles"), profile.name.clone()),
      Row::info(
        tr(lang, "control_center.profile_monitors"),
        profile.config.monitors.len().to_string(),
      ),
    ];
    if self.state.active_profile.as_deref() == Some(profile.name.as_str()) {
      rows.push(Row::info(tr(lang, "control_center.active"), ""));
    }
    rows.extend([
      Row::section(tr(lang, "control_center.section_actions")),
      Row::action(Item::ApplyProfile, tr(lang, "control_center.apply_profile")).icon(icons::APPLY),
      Row::action(Item::RenameProfile, tr(lang, "control_center.rename")).icon(icons::EDIT),
      Row::section(tr(lang, "control_center.danger_zone")),
      Row::destructive(
        Item::DeleteProfile,
        tr(lang, "control_center.delete_profile"),
      )
      .icon(icons::DELETE),
    ]);
    rows
  }

  pub(super) fn apply_profile(&mut self, index: usize) {
    let Some(profile) = self.state.profiles.get(index).cloned() else {
      self.status = Some(StatusMessage {
        kind: StatusKind::Warning,
        text: tr(self.lang, "control_center.select_a_profile_to_apply").into(),
      });
      return;
    };
    self.config = profile.config.clone();
    self.state.primary_monitor = profile.config.primary_monitor.clone();
    self.state.active_profile = Some(profile.name.clone());
    let config = self.config.clone();
    let state = self.state.clone();
    let rules: Vec<String> = profile
      .config
      .monitors
      .iter()
      .map(|(name, persisted)| rule_from_persisted(name, persisted))
      .collect();
    let workspaces: Vec<(u32, String)> = profile
      .config
      .workspaces
      .iter()
      .flat_map(|(monitor, ids)| ids.iter().map(|id| (*id, monitor.clone())))
      .collect();
    let lang = self.lang;
    let profile_applied = tr(lang, "control_center.profile_applied").to_string();
    self.action = Some(self.manager.spawn(move |_| {
      for rule in &rules {
        backend::apply_keyword(rule)?;
      }
      for (workspace, monitor) in &workspaces {
        backend::apply_workspace(*workspace, Some(monitor))?;
      }
      backend::save_config(&config)?;
      backend::save_state(&state)?;
      Ok(JobData::Action(format!(
        "{profile_applied} · {}",
        state.active_profile.as_deref().unwrap_or("")
      )))
    }));
    self.refresh();
  }
}

pub(super) fn rule_from_persisted(name: &str, persisted: &PersistedMonitor) -> String {
  let mode = persisted.mode.clone().unwrap_or_else(|| "auto".into());
  let position = persisted.position.clone().unwrap_or_else(|| "auto".into());
  let scale = persisted
    .scale
    .map(|value| value.to_string())
    .unwrap_or_else(|| "auto".into());
  let transform = persisted
    .transform
    .map(|value| value.to_string())
    .unwrap_or_else(|| "auto".into());
  let mut body = format!("{name}, {mode}, {position}, {scale}, transform, {transform}");
  if let Some(mirror) = &persisted.mirror {
    body.push_str(&format!(", mirror, {mirror}"));
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
  if persisted.disabled == Some(true) {
    body = format!("{name}, disabled");
  }
  body
}

pub(super) fn fingerprint(monitors: &[Monitor]) -> Vec<String> {
  monitors
    .iter()
    .map(|monitor| {
      format!(
        "{}:{}x{}:{}x{}:{}:{}",
        monitor.name,
        monitor.width,
        monitor.height,
        monitor.x,
        monitor.y,
        monitor.disabled,
        monitor.dpms_status
      )
    })
    .collect()
}
