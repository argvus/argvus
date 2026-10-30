//! Typed values (custom mode, position, scale, workspaces): opening, editing and committing a prompt.

use super::*;

impl DisplaysApp {
  pub(super) fn prompt_key(&mut self, key: KeyCode) -> bool {
    let DisplayPage::Prompt { goal } = self.page else {
      return false;
    };
    match key {
      KeyCode::Esc => {
        self.restore_prompt(goal);
        false
      }
      KeyCode::Backspace => {
        self.prompt_buffer.pop();
        false
      }
      KeyCode::Char(c) if prompt_allowed(goal, c) => {
        self.prompt_buffer.push(c);
        false
      }
      KeyCode::Enter => {
        self.commit_prompt(goal);
        false
      }
      _ => false,
    }
  }

  pub(super) fn restore_prompt(&mut self, goal: PromptGoal) {
    self.prompt_buffer.clear();
    self.prompt_error = None;
    let page = self.prompt_back.take().unwrap_or(match goal {
      PromptGoal::Position(i)
      | PromptGoal::Scale(i)
      | PromptGoal::SdrBrightness(i)
      | PromptGoal::SdrSaturation(i) => DisplayPage::Detail(i),
      PromptGoal::NightTemperature | PromptGoal::NightStart | PromptGoal::NightEnd => {
        DisplayPage::NightLight
      }
      PromptGoal::ProfileCreate => DisplayPage::Profiles,
      PromptGoal::ProfileRename(i) => DisplayPage::Profile(i),
    });
    self.go(page);
  }

  pub(super) fn commit_prompt(&mut self, goal: PromptGoal) {
    let buffer = self.prompt_buffer.clone();
    match goal {
      PromptGoal::NightTemperature | PromptGoal::NightStart | PromptGoal::NightEnd => {
        if !self.commit_night_prompt(goal, &buffer) {
          return;
        }
        self.leave_prompt(DisplayPage::NightLight);
      }
      PromptGoal::Position(index) => {
        let Some((x, y)) = parse_position(&buffer) else {
          self.prompt_error = Some(
            tr(
              self.lang,
              "control_center.invalid_position_use_x_y_e_g_1920_0",
            )
            .into(),
          );
          return;
        };
        let Some(monitor) = self.monitors.get(index).cloned() else {
          return;
        };
        let name = monitor.name.clone();
        let mut config = self.config.clone();
        let mut persisted = config.persisted(&name);
        persisted.position = Some(format!("{x}x{y}"));
        config.set_monitor(&name, persisted);
        let prev = self.config.clone();
        self.config = config.clone();
        self.arm_revert(name.clone(), monitor.clone(), prev, Instant::now());
        let message = format!(
          "{} · {name} ({x}, {y})",
          tr(self.lang, "control_center.position")
        );
        self.leave_prompt(DisplayPage::Detail(index));
        self.spawn_apply_with_config(config, message);
      }
      PromptGoal::Scale(index) => {
        let Some(value) = parse_number(&buffer) else {
          self.prompt_error = Some(
            tr(
              self.lang,
              "control_center.invalid_value_use_a_number_e_g_1_25",
            )
            .into(),
          );
          return;
        };
        let Some(monitor) = self.monitors.get(index).cloned() else {
          return;
        };
        let name = monitor.name.clone();
        let mut config = self.config.clone();
        let mut persisted = config.persisted(&name);
        persisted.scale = Some(value);
        config.set_monitor(&name, persisted);
        self.config = config.clone();
        let message = format!(
          "{} · {name}: {value}",
          tr(self.lang, "control_center.scale")
        );
        self.leave_prompt(DisplayPage::Detail(index));
        self.spawn_apply_with_config(config, message);
      }
      PromptGoal::SdrBrightness(index) => {
        let Some(value) = parse_number(&buffer) else {
          self.prompt_error =
            Some(tr(self.lang, "control_center.invalid_value_use_0_0_to_1_0").into());
          return;
        };
        self.apply_sdr(index, |persisted| {
          persisted.sdr_brightness = Some(value.clamp(0.0, 1.0))
        });
      }
      PromptGoal::SdrSaturation(index) => {
        let Some(value) = parse_number(&buffer) else {
          self.prompt_error =
            Some(tr(self.lang, "control_center.invalid_value_use_0_0_to_2_0").into());
          return;
        };
        self.apply_sdr(index, |saturation| {
          saturation.sdr_saturation = Some(value.clamp(0.0, 2.0))
        });
      }
      PromptGoal::ProfileCreate => {
        if buffer.trim().is_empty() {
          self.prompt_error =
            Some(tr(self.lang, "control_center.profile_name_cannot_be_empty").into());
          return;
        }
        if self.state.profile_index(buffer.trim()).is_some() {
          self.prompt_error = Some(
            tr(
              self.lang,
              "control_center.a_profile_with_this_name_already_exists",
            )
            .into(),
          );
          return;
        }
        let profile = MonitorProfile {
          name: buffer.trim().to_string(),
          apply_wallpapers: false,
          config: self.config.clone(),
        };
        self.state.profiles.push(profile);
        self.state.active_profile = None;
        let state = self.state.clone();
        self.spawn_save_state(
          state,
          format!(
            "{} · {}",
            tr(self.lang, "control_center.profile_created"),
            buffer.trim()
          ),
        );
        self.leave_prompt(DisplayPage::Profiles);
        let created = Item::Profile(self.state.profiles.len().saturating_sub(1));
        let rows = self.rows();
        self.menu.select(&rows, &created);
      }
      PromptGoal::ProfileRename(index) => {
        if buffer.trim().is_empty() {
          self.prompt_error =
            Some(tr(self.lang, "control_center.profile_name_cannot_be_empty").into());
          return;
        }
        if let Some(profile) = self.state.profiles.get_mut(index) {
          profile.name = buffer.trim().to_string();
        }
        let state = self.state.clone();
        self.spawn_save_state(
          state,
          format!(
            "{} · {}",
            tr(self.lang, "control_center.profile_renamed"),
            buffer.trim()
          ),
        );
        self.leave_prompt(DisplayPage::Profile(index));
      }
    }
  }

  /// Closes the prompt after a committed value and opens `page`.
  pub(super) fn leave_prompt(&mut self, page: DisplayPage) {
    self.prompt_buffer.clear();
    self.prompt_error = None;
    self.prompt_back = None;
    self.go(page);
  }

  /// The free-form editor of the open setting.
  pub(super) fn custom_prompt(&self) -> Option<PromptGoal> {
    self
      .picker_options()
      .into_iter()
      .find_map(|option| option.prompt)
  }

  pub(super) fn open_prompt(&mut self, goal: PromptGoal) {
    self.prompt_back = Some(self.page);
    self.prompt_buffer = self.prompt_prefill(goal);
    self.prompt_error = None;
    self.page = DisplayPage::Prompt { goal };
  }

  pub(super) fn prompt_prefill(&self, goal: PromptGoal) -> String {
    match goal {
      PromptGoal::NightTemperature => self.night_light.settings.temperature.to_string(),
      PromptGoal::NightStart => self.night_light.settings.start.clone(),
      PromptGoal::NightEnd => self.night_light.settings.end.clone(),
      PromptGoal::Position(index) => self
        .monitors
        .get(index)
        .map(|monitor| {
          self
            .config
            .persisted(&monitor.name)
            .position
            .clone()
            .unwrap_or_else(|| format!("{},{}", monitor.x, monitor.y))
        })
        .unwrap_or_default(),
      PromptGoal::Scale(index) => self
        .monitors
        .get(index)
        .map(|monitor| {
          self
            .config
            .persisted(&monitor.name)
            .scale
            .map(|value| value.to_string())
            .unwrap_or_else(|| monitor.scale.to_string())
        })
        .unwrap_or_default(),
      PromptGoal::SdrBrightness(index) => self
        .monitors
        .get(index)
        .and_then(|monitor| {
          self
            .config
            .persisted(&monitor.name)
            .sdr_brightness
            .or(monitor.info.sdr_brightness)
            .map(|value| value.to_string())
        })
        .unwrap_or_else(|| "1.0".into()),
      PromptGoal::SdrSaturation(index) => self
        .monitors
        .get(index)
        .and_then(|monitor| {
          self
            .config
            .persisted(&monitor.name)
            .sdr_saturation
            .or(monitor.info.sdr_saturation)
            .map(|value| value.to_string())
        })
        .unwrap_or_else(|| "1.0".into()),
      PromptGoal::ProfileCreate => String::new(),
      PromptGoal::ProfileRename(index) => self
        .state
        .profiles
        .get(index)
        .map(|profile| profile.name.clone())
        .unwrap_or_default(),
    }
  }
}

pub(super) fn parse_position(buffer: &str) -> Option<(i32, i32)> {
  let (x, y) = buffer.split_once(',')?;
  Some((x.trim().parse().ok()?, y.trim().parse().ok()?))
}

pub(super) fn parse_number(buffer: &str) -> Option<f64> {
  buffer
    .trim()
    .parse::<f64>()
    .ok()
    .filter(|value| value.is_finite())
}

pub(super) fn prompt_allowed(goal: PromptGoal, character: char) -> bool {
  match goal {
    PromptGoal::NightTemperature => character.is_ascii_digit(),
    PromptGoal::NightStart | PromptGoal::NightEnd => character.is_ascii_digit() || character == ':',
    PromptGoal::Position(_) => {
      character.is_ascii_digit() || character == 'x' || character == ',' || character == '-'
    }
    PromptGoal::Scale(_) | PromptGoal::SdrBrightness(_) | PromptGoal::SdrSaturation(_) => {
      character.is_ascii_digit() || character == '.'
    }
    PromptGoal::ProfileCreate | PromptGoal::ProfileRename(_) => !character.is_control(),
  }
}
