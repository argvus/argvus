//! Value prompts and confirmations: typing a number, color or path, and answering a confirmation.

use super::*;

impl AppearanceApp {
  pub(super) fn open_prompt(&mut self, goal: PromptGoal) {
    self.prompt_back = Some(self.page);
    self.page = AppearancePage::Prompt { goal };
    self.prompt_buffer.clear();
    self.prompt_error = None;
  }

  pub(super) fn prompt_key(&mut self, key: KeyCode) -> bool {
    if self.page == AppearancePage::AccentEdit {
      match key {
        KeyCode::Enter if self.action.is_none() => {
          if let Some(color) = normalize_hex_color(&self.prompt_buffer) {
            self.apply(
              tr(self.lang, "control_center.accent_applied").into(),
              move || backend::set_accent(&color),
            );
            self.go(AppearancePage::Accents);
          } else {
            self.prompt_error = Some(tr(self.lang, "control_center.accent_invalid").into());
          }
        }
        KeyCode::Esc | KeyCode::Left => self.go(AppearancePage::Accents),
        KeyCode::Backspace | KeyCode::Delete => {
          self.prompt_buffer.pop();
          self.prompt_error = None;
        }
        KeyCode::Char('#') if self.prompt_buffer.is_empty() => self.prompt_buffer.push('#'),
        KeyCode::Char(c)
          if c.is_ascii_hexdigit() && self.prompt_buffer.trim_start_matches('#').len() < 6 =>
        {
          self.prompt_buffer.push(c.to_ascii_uppercase());
          self.prompt_error = None;
        }
        _ => {}
      }
      return false;
    }
    match key {
      KeyCode::Enter if self.action.is_none() => {
        let AppearancePage::Prompt { goal } = self.page else {
          return false;
        };
        let value = self.prompt_buffer.trim().to_string();
        if let PromptGoal::BlurValue(field) = goal {
          self.commit_blur_value(field, &value);
          return false;
        }
        if let PromptGoal::AnimationValue(field) = goal {
          self.commit_animation_value(field, &value);
          return false;
        }
        if goal == PromptGoal::ExportProfile {
          let back = self.prompt_back.take().unwrap_or(AppearancePage::Themes);
          if value.is_empty() {
            self.prompt_error =
              Some(tr(self.lang, "control_center.theme_profile_invalid_name").into());
            self.prompt_back = Some(back);
            return false;
          }
          let message = tr(self.lang, "control_center.theme_profile_exported").to_string();
          self.apply_result(move || {
            let path = backend::export_theme(&value)?;
            Ok(format!("{message}: {}", path.display()))
          });
          self.go(back);
          return false;
        }
        let (min, max) = goal.range();
        let valid = value
          .parse::<i32>()
          .ok()
          .is_some_and(|v| (min..=max).contains(&v));
        if !valid {
          self.prompt_error = Some(format!(
            "{} ({}–{})",
            tr(self.lang, "control_center.enter_a_valid_integer"),
            min,
            max
          ));
          return false;
        }
        let back = self.prompt_back.take().unwrap_or(AppearancePage::Home);
        if goal == PromptGoal::DraftValue {
          if let Ok(value) = value.parse::<i32>() {
            self.set_draft_value(back, value);
          }
          return false;
        }
        let key_name = goal.key();
        let message_key = if matches!(goal, PromptGoal::Rounding) {
          "control_center.border_settings_applied"
        } else if matches!(goal, PromptGoal::Thickness) {
          "control_center.thickness_applied"
        } else {
          "control_center.spacing_applied"
        };
        self.apply(tr(self.lang, message_key).into(), move || match goal {
          PromptGoal::Rounding | PromptGoal::Thickness => backend::set_border(key_name, &value),
          _ => backend::set_spacing(key_name, &value),
        });
        self.go(back);
        false
      }
      KeyCode::Esc | KeyCode::Left => {
        let back = self.prompt_back.take().unwrap_or(AppearancePage::Home);
        self.go(back);
        false
      }
      KeyCode::Char(c)
        if matches!(
          self.page,
          AppearancePage::Prompt {
            goal: PromptGoal::ExportProfile
          }
        ) && !c.is_control()
          && self.prompt_buffer.len() < 512 =>
      {
        self.prompt_buffer.push(c);
        false
      }
      KeyCode::Char(c)
        if c.is_ascii_digit() && self.prompt_buffer.len() < self.prompt_digit_limit() =>
      {
        self.prompt_buffer.push(c);
        false
      }
      KeyCode::Char('.') if self.decimal_prompt() && !self.prompt_buffer.contains('.') => {
        self.prompt_buffer.push('.');
        false
      }
      KeyCode::Char('-')
        if self
          .animation_prompt_field()
          .is_some_and(AnimationField::allows_negative)
          && self.prompt_buffer.is_empty() =>
      {
        self.prompt_buffer.push('-');
        false
      }
      KeyCode::Backspace => {
        self.prompt_buffer.pop();
        false
      }
      _ => false,
    }
  }

  /// The Blur field whose value the open prompt is typing, if any.
  pub(super) fn blur_prompt_field(&self) -> Option<BlurField> {
    match self.page {
      AppearancePage::Prompt {
        goal: PromptGoal::BlurValue(field),
      } => Some(field),
      _ => None,
    }
  }

  /// The Animations field whose value the open prompt is typing, if any.
  pub(super) fn animation_prompt_field(&self) -> Option<AnimationField> {
    match self.page {
      AppearancePage::Prompt {
        goal: PromptGoal::AnimationValue(field),
      } => Some(field),
      _ => None,
    }
  }

  /// Whether the open prompt types a decimal value: Blur parameters and the
  /// Animations curve parameters.
  pub(super) fn decimal_prompt(&self) -> bool {
    self.blur_prompt_field().is_some()
      || self
        .animation_prompt_field()
        .is_some_and(|field| !field.is_integer())
  }

  /// Longest text a numeric prompt accepts. Decimal values need room for a
  /// value such as `0.900000`.
  pub(super) fn prompt_digit_limit(&self) -> usize {
    if self.decimal_prompt() { 8 } else { 3 }
  }

  pub(super) fn prompt_label(&self, goal: PromptGoal) -> &'static str {
    match goal {
      PromptGoal::ExportProfile => "control_center.theme_profile_export_name",
      PromptGoal::WaybarTop => "control_center.top",
      PromptGoal::WaybarLeft => "control_center.left",
      PromptGoal::WaybarRight => "control_center.right",
      PromptGoal::WaybarBottom => "control_center.bottom",
      PromptGoal::GapsIn => "control_center.inner_gap",
      PromptGoal::GapsOutTop => "control_center.outer_gap_top",
      PromptGoal::GapsOutLeft => "control_center.outer_gap_left",
      PromptGoal::GapsOutRight => "control_center.outer_gap_right",
      PromptGoal::GapsOutBottom => "control_center.outer_gap_bottom",
      PromptGoal::Rounding | PromptGoal::Thickness => "control_center.value",
      PromptGoal::DraftValue => "control_center.value",
      PromptGoal::BlurValue(field) => field.label_key(),
      PromptGoal::AnimationValue(field) => field.label_key(),
    }
  }

  /// Runs the confirmed decision. Returns `true` when Appearance is left.
  pub(super) fn resolve_confirmation(&mut self, confirmation: Confirmation) -> bool {
    match confirmation {
      Confirmation::DiscardDraft => self.navigate_back(),
      Confirmation::DeleteTheme => {
        if let Some(theme) = self.delete_theme.take() {
          self.apply(
            tr(self.lang, "control_center.theme_profile_deleted").into(),
            move || backend::delete_custom_theme(&theme),
          );
        }
        self.go(AppearancePage::Themes);
        false
      }
      Confirmation::ReplaceImport => {
        self.start_pending_import();
        self.go(AppearancePage::Themes);
        false
      }
    }
  }

  /// Cancelling a theme confirmation returns to the Themes page, as the old
  /// confirmation pages did; cancelling a discard keeps the page and draft.
  pub(super) fn cancel_confirmation(&mut self, confirmation: Confirmation) {
    match confirmation {
      Confirmation::DiscardDraft => {}
      Confirmation::DeleteTheme => {
        self.delete_theme = None;
        self.go(AppearancePage::Themes);
      }
      Confirmation::ReplaceImport => {
        self.pending_import = None;
        self.pending_import_name = None;
        self.go(AppearancePage::Themes);
      }
    }
  }
}
