//! Drawing of the Power page: lists, overlays, footer hints and the breadcrumb.

use super::*;

impl PowerApp {
  pub fn draw(&mut self, frame: &mut Frame) {
    let area = frame.area();
    let mut lines = self.rows();
    if let Some(state) = &self.state
      && state.is_laptop
    {
      lines.push(self.capabilities_line());
    }
    let shell_body = shell(
      frame,
      area,
      &self.theme,
      self.lang,
      &format!(
        "{} > {}",
        HomeCardId::Preferences.label(self.lang),
        tr(self.lang, "control_center.power")
      ),
      tr(
        self.lang,
        "control_center.navigate_tab_actions_enter_list_r_refresh_esc_back_help",
      ),
    );
    let buttons: Vec<ActionButton> = self
      .buttons()
      .into_iter()
      .map(|(_, button)| button)
      .collect();
    // The action bar stays fixed under the list, as in Time Zone: a bordered
    // block titled `Actions · hint`, with the list above it.
    let (body, action_area) = if buttons.is_empty() {
      (shell_body, None)
    } else {
      let split = Layout::vertical([Constraint::Min(1), Constraint::Length(3)]).split(shell_body);
      (split[0], Some(split[1]))
    };
    let list_selection = if self.job.is_some() {
      usize::MAX
    } else {
      self.selected.min(lines.len().saturating_sub(1))
    };
    list(frame, body, &self.theme, &lines, list_selection);
    if let Some(action_area) = action_area {
      let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(self.theme.border_active))
        .title(format!(
          "{} · {}",
          tr(self.lang, "control_center.actions"),
          tr(self.lang, "control_center.actions_hint")
        ));
      let inner = block.inner(action_area);
      frame.render_widget(block, action_area);
      let focus = self.actions.cursor().unwrap_or(usize::MAX);
      draw_aligned_transparent(frame, inner, &buttons, focus, &self.theme, Alignment::Left);
    }
    if let Some(s) = &self.status {
      status(frame, area, &self.theme, s);
    }
    if let Some(pending) = &self.pending {
      let action = match pending {
        Pending::Suspend => tr(self.lang, "control_center.suspend"),
        Pending::Hibernate => tr(self.lang, "control_center.hibernate"),
      };
      draw_confirmation(
        frame,
        area,
        &self.theme,
        ConfirmationDialog {
          title: tr(self.lang, "control_center.confirm_action"),
          message: &format!(
            "{} {}?",
            tr(self.lang, "control_center.do_you_want_to"),
            action.to_lowercase(),
          ),
          confirm_label: tr(self.lang, "control_center.continue"),
          cancel_label: tr(self.lang, "control_center.cancel"),
          confirm_selected: self.confirmation.confirm_selected,
        },
      )
    }
    if let Some(picker) = &self.picker {
      draw_picker(frame, area, &self.theme, self.lang, picker);
    }
  }
}

pub(super) fn draw_picker(
  frame: &mut Frame,
  area: Rect,
  theme: &Theme,
  lang: Lang,
  picker: &Picker,
) {
  let options = picker_options(picker.target, lang);
  let title = picker_title(lang, picker.target);
  let width = area.width.saturating_sub(8).clamp(28, 48);
  let height = (options.len() as u16).saturating_add(2).min(area.height);
  let popup = centered(area, width, height.max(4));
  frame.render_widget(Clear, popup);
  let lines: Vec<Line> = options
    .iter()
    .enumerate()
    .map(|(index, option)| {
      let chosen = index == picker.selected;
      let style = if chosen {
        Style::new()
          .fg(theme.selected_foreground)
          .bg(theme.selected_background)
          .add_modifier(Modifier::BOLD)
      } else {
        Style::new().fg(theme.foreground)
      };
      Line::from(Span::styled(
        format!(" {} {} ", if chosen { ">" } else { " " }, option),
        style,
      ))
    })
    .collect();
  frame.render_widget(
    Paragraph::new(lines).block(
      Block::bordered()
        .title(format!(" {title} "))
        .border_style(Style::new().fg(theme.border_active))
        .style(Style::new().bg(theme.background)),
    ),
    popup,
  );
}
