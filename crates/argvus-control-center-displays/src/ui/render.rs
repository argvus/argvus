//! Drawing of the displays pages: list, prompt, overlays, footer hints and the breadcrumb.

use super::rows::setting_label;
use super::*;

impl DisplaysApp {
  pub(super) fn breadcrumb(&self) -> String {
    let root = format!(
      "{} > {}",
      HomeCardId::DesktopInput.label(self.lang),
      tr(self.lang, "control_center.displays")
    );
    let root = if matches!(
      self.page,
      DisplayPage::Prompt {
        goal: PromptGoal::NightTemperature | PromptGoal::NightStart | PromptGoal::NightEnd
      }
    ) {
      format!(
        "{root} > {}",
        tr(self.lang, "control_center.night_light.title")
      )
    } else {
      root
    };
    match self.page {
      DisplayPage::Home => root,
      DisplayPage::NightLight => format!(
        "{root} > {}",
        tr(self.lang, "control_center.night_light.title")
      ),
      DisplayPage::Detail(index) => {
        let name = self
          .monitors
          .get(index)
          .map(|monitor| monitor.name.clone())
          .or_else(|| self.stale_name(index))
          .unwrap_or_else(|| tr(self.lang, "control_center.monitor").into());
        format!("{root} > {name}")
      }
      DisplayPage::Picker { setting, .. } => {
        format!("{root} > {}", setting_label(self.lang, setting))
      }
      DisplayPage::Profiles => format!("{root} > {}", tr(self.lang, "control_center.profiles")),
      DisplayPage::Profile(index) => format!(
        "{root} > {} > {}",
        tr(self.lang, "control_center.profiles"),
        self
          .state
          .profiles
          .get(index)
          .map(|profile| profile.name.as_str())
          .unwrap_or_default()
      ),
      DisplayPage::Prompt { goal } => format!(
        "{root} > {}",
        match goal {
          PromptGoal::NightTemperature => tr(self.lang, "control_center.night_light.temperature"),
          PromptGoal::NightStart => tr(self.lang, "control_center.night_light.start"),
          PromptGoal::NightEnd => tr(self.lang, "control_center.night_light.end"),
          PromptGoal::Position(_) => tr(self.lang, "control_center.position"),
          PromptGoal::Scale(_) => tr(self.lang, "control_center.scale"),
          PromptGoal::SdrBrightness(_) => tr(self.lang, "control_center.sdr_brightness"),
          PromptGoal::SdrSaturation(_) => tr(self.lang, "control_center.sdr_saturation"),
          PromptGoal::ProfileCreate => tr(self.lang, "control_center.new_profile"),
          PromptGoal::ProfileRename(_) => tr(self.lang, "control_center.rename_profile"),
        }
      ),
    }
  }

  pub(super) fn draw_prompt(&mut self, frame: &mut Frame, area: Rect, goal: PromptGoal) {
    let width = area.width.saturating_sub(6).clamp(34, 76);
    let height = 9u16.min(area.height);
    let x = area.x + (area.width - width).saturating_div(2);
    let y = area.y + (area.height.saturating_sub(height)).saturating_div(2);
    let box_area = Rect::new(x, y, width, height);
    frame.render_widget(Clear, box_area);
    let title = match goal {
      PromptGoal::NightTemperature => tr(self.lang, "control_center.night_light.temperature"),
      PromptGoal::NightStart => tr(self.lang, "control_center.night_light.start"),
      PromptGoal::NightEnd => tr(self.lang, "control_center.night_light.end"),
      PromptGoal::Position(_) => tr(self.lang, "control_center.position_x_y"),
      PromptGoal::Scale(_) => tr(self.lang, "control_center.scale"),
      PromptGoal::SdrBrightness(_) => tr(self.lang, "control_center.sdr_brightness"),
      PromptGoal::SdrSaturation(_) => tr(self.lang, "control_center.sdr_saturation"),
      PromptGoal::ProfileCreate => tr(self.lang, "control_center.new_profile"),
      PromptGoal::ProfileRename(_) => tr(self.lang, "control_center.rename_profile"),
    };
    let display = if self.prompt_buffer.is_empty() {
      " ".into()
    } else {
      self.prompt_buffer.clone()
    };
    let mut lines = vec![Line::from("")];
    lines.push(Line::from(vec![
      Span::styled("> ", Style::new().fg(self.theme.accent)),
      Span::styled(
        display,
        Style::new()
          .fg(self.theme.foreground)
          .add_modifier(Modifier::BOLD),
      ),
    ]));
    lines.push(Line::from(""));
    if let Some(error) = &self.prompt_error {
      lines.push(Line::from(vec![Span::styled(
        argvus_tui::icons::icon_label(argvus_tui::icons::WARNING, error),
        Style::new().fg(self.theme.error),
      )]));
    }
    frame.render_widget(
      Paragraph::new(lines)
        .block(
          Block::bordered()
            .title(format!(" {title} "))
            .border_style(Style::new().fg(self.theme.border_active)),
        )
        .style(Style::new().bg(self.theme.background)),
      box_area,
    );
    self.overlays(frame, area);
  }

  /// Footer: the revert keys while the countdown runs, the editor keys in a
  /// prompt, otherwise derived from the selected row.
  pub(super) fn footer_hints(&self, rows: &[Row<Item>]) -> String {
    let label = |key: &str| tr(self.lang, key);
    if self.revert.is_some() {
      // Keeping is the explicit choice, so `y` comes first.
      return [
        ("y", label("control_center.keep")),
        ("n/Esc", label("control_center.revert")),
        ("↑/↓", label("control_center.hint.select")),
      ]
      .map(|(keys, action)| format!("{keys} {action}"))
      .join(FOOTER_GAP);
    }
    if self.confirm.is_some() {
      return confirm_hints(self.lang);
    }
    if let DisplayPage::Prompt { .. } = self.page {
      return [
        ("Enter", label("control_center.hint.confirm")),
        ("Esc", label("control_center.hint.back")),
      ]
      .map(|(keys, action)| format!("{keys} {action}"))
      .join(FOOTER_GAP);
    }
    let mut extra = Vec::new();
    if matches!(self.page, DisplayPage::Picker { .. }) {
      extra.push(("r", label("control_center.cancel")));
    }
    let mut menu = self.menu;
    menu.normalize(rows);
    hints(
      self.lang,
      &HintContext {
        row: menu.selected_kind(rows),
        can_go_back: true,
        refresh: matches!(
          self.page,
          DisplayPage::Home
            | DisplayPage::NightLight
            | DisplayPage::Detail(_)
            | DisplayPage::Profiles
            | DisplayPage::Profile(_)
        ),
        extra: &extra,
        ..HintContext::default()
      },
    )
  }

  pub fn draw(&mut self, frame: &mut Frame) {
    let area = frame.area();
    let rows = self.rows();
    self.menu.normalize(&rows);
    let body = shell(
      frame,
      area,
      &self.theme,
      self.lang,
      &self.breadcrumb(),
      &self.footer_hints(&rows),
    );
    if let DisplayPage::Prompt { goal } = self.page {
      self.draw_prompt(frame, body, goal);
      return;
    }
    self.list_height = body.height;
    draw_menu(
      frame,
      body,
      &self.theme,
      &rows,
      &mut self.menu,
      MenuStyle {
        icons: AppConfig::icons_enabled(),
      },
    );
    self.overlays(frame, area);
  }

  pub(super) fn overlays(&self, frame: &mut Frame, area: Rect) {
    if let Some(revert) = &self.revert {
      let message = format!(
        "{}\n{}",
        revert.name,
        tr(self.lang, "control_center.keep_configuration_description")
      );
      let dialog = ConfirmDialog {
        title: tr(self.lang, "control_center.keep_configuration_title"),
        message: &message,
        confirm: tr(self.lang, "control_center.keep"),
        cancel: tr(self.lang, "control_center.revert"),
        danger: false,
        deadline: Some(revert.deadline.saturating_duration_since(Instant::now())),
      };
      draw_confirm(frame, area, &self.theme, dialog, &self.revert_focus);
      return;
    }
    if let Some((confirmation, focus)) = &self.confirm {
      let (title, message, confirm) = match confirmation {
        Confirmation::DeleteProfile(index) => (
          "control_center.delete_profile",
          format!(
            "{}\n{}",
            tr(self.lang, "control_center.delete_this_profile"),
            self
              .state
              .profiles
              .get(*index)
              .map(|profile| profile.name.as_str())
              .unwrap_or("")
          ),
          "control_center.delete",
        ),
        Confirmation::RemoveConfig => (
          "control_center.remove_config",
          format!(
            "{}\n{}",
            self
              .monitor_index()
              .and_then(|index| {
                self
                  .monitors
                  .get(index)
                  .map(|monitor| monitor.name.clone())
                  .or_else(|| self.stale_name(index))
              })
              .unwrap_or_default(),
            tr(self.lang, "control_center.remove_config_description")
          ),
          "control_center.remove_config",
        ),
      };
      let dialog = ConfirmDialog {
        title: tr(self.lang, title),
        message: &message,
        confirm: tr(self.lang, confirm),
        cancel: tr(self.lang, "control_center.cancel"),
        danger: true,
        deadline: None,
      };
      draw_confirm(frame, area, &self.theme, dialog, focus);
      return;
    }
    if let Some(s) = &self.status {
      status(frame, area, &self.theme, s);
    }
  }
}
