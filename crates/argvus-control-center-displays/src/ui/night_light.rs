//! Night-light rows and actions; the CLI remains the only owner of preferences.
use super::*;

impl DisplaysApp {
  pub(super) fn night_rows(&self) -> Vec<Row<Item>> {
    let label = |key| tr(self.lang, key);
    let snapshot = &self.night_light;
    let settings = &snapshot.settings;
    if !snapshot.loaded && !snapshot.available {
      return if let Some(reason) = &snapshot.unavailable_reason {
        vec![
          Row::info(
            label("control_center.night_light.unavailable"),
            reason.clone(),
          ),
          Row::action(Item::Refresh, label("control_center.refresh")),
        ]
      } else {
        vec![Row::info(label("control_center.night_light.loading"), "")]
      };
    }
    let mut rows = vec![];
    if !snapshot.available {
      rows.push(Row::info(
        label("control_center.night_light.unavailable"),
        snapshot.unavailable_reason.clone().unwrap_or_default(),
      ));
    }
    rows.extend([
      Row::toggle(
        Item::NightToggle,
        label("control_center.night_light.title"),
        snapshot.active.unwrap_or(false),
      )
      .enabled(snapshot.available),
      Row::info(
        label("control_center.night_light.saved_manual"),
        rows::on_off(self.lang, settings.enabled),
      ),
      Row::submenu(
        Item::NightTemperature,
        label("control_center.night_light.temperature"),
      )
      .detail(format!("{} K · 1000–20000 K", settings.temperature))
      .enabled(snapshot.available),
      Row::toggle(
        Item::NightSchedule,
        label("control_center.night_light.schedule"),
        settings.schedule_enabled,
      )
      .enabled(snapshot.available),
      Row::submenu(Item::NightStart, label("control_center.night_light.start"))
        .detail(&settings.start)
        .enabled(snapshot.available),
      Row::submenu(Item::NightEnd, label("control_center.night_light.end"))
        .detail(&settings.end)
        .enabled(snapshot.available),
      Row::info(
        label("control_center.night_light.effective"),
        snapshot
          .effective_temperature
          .map(|temperature| format!("{temperature} K"))
          .unwrap_or_else(|| {
            label(if snapshot.available {
              "control_center.off"
            } else {
              "control_center.unavailable"
            })
            .into()
          }),
      ),
      Row::info(
        label("control_center.night_light.next"),
        snapshot
          .next_transition
          .clone()
          .unwrap_or_else(|| label("control_center.none").into()),
      ),
    ]);
    if snapshot.manual_override {
      rows.push(Row::info(label("control_center.night_light.override"), ""));
    }
    if let Some(error) = &self.night_error {
      rows.push(Row::info(
        label("control_center.night_light.apply_failed"),
        error.clone(),
      ));
    }
    rows.push(Row::action(Item::Refresh, label("control_center.refresh")));
    rows
  }

  pub(super) fn night_action(&mut self, arguments: Vec<String>) {
    if !self.night_light.available || self.action.is_some() {
      return;
    }
    self.night_error = None;
    self.action = Some(self.manager.spawn(move |_| {
      let error = backend::night_light_command(&arguments).err();
      Ok(JobData::NightLight(backend::night_light_status(), error))
    }));
  }

  pub(super) fn night_schedule(&mut self, enabled: bool, start: String, end: String) {
    self.night_action(vec![
      "schedule".into(),
      if enabled { "on" } else { "off" }.into(),
      start,
      end,
    ]);
  }

  pub(super) fn commit_night_prompt(&mut self, goal: PromptGoal, buffer: &str) -> bool {
    if goal == PromptGoal::NightTemperature {
      if buffer
        .parse::<u32>()
        .ok()
        .is_none_or(|temperature| !(1000..=20000).contains(&temperature))
      {
        self.prompt_error = Some(tr(self.lang, "control_center.night_light.invalid").into());
        return false;
      }
      self.night_action(vec!["temperature".into(), buffer.into()]);
      return true;
    }
    let settings = &self.night_light.settings;
    let start = if goal == PromptGoal::NightStart {
      buffer
    } else {
      &settings.start
    };
    let end = if goal == PromptGoal::NightEnd {
      buffer
    } else {
      &settings.end
    };
    if !valid_time(start) || !valid_time(end) || start == end {
      self.prompt_error = Some(tr(self.lang, "control_center.night_light.invalid").into());
      return false;
    }
    self.night_schedule(settings.schedule_enabled, start.into(), end.into());
    true
  }
}

fn valid_time(text: &str) -> bool {
  let bytes = text.as_bytes();
  bytes.len() == 5
    && bytes[2] == b':'
    && bytes
      .iter()
      .enumerate()
      .all(|(index, byte)| index == 2 || byte.is_ascii_digit())
    && text[..2].parse::<u32>().is_ok_and(|hour| hour < 24)
    && text[3..].parse::<u32>().is_ok_and(|minute| minute < 60)
}
