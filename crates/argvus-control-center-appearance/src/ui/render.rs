//! Drawing of the appearance pages: the list, confirmations, prompts, the accent editor and the footer hints.

use super::*;

impl AppearanceApp {
  /// Footer of the current page. List pages derive it from the selected
  /// row's kind (`argvus_tui::hints`); the text editors list their own keys
  /// with the same translated action names.
  pub(super) fn hints(&self, rows: &[Row<Item>]) -> String {
    let label = |key: &str| tr(self.lang, key);
    if self.confirm.is_some() {
      return confirm_hints(self.lang);
    }
    match self.page {
      AppearancePage::AccentEdit => [
        ("#/0-9/A-F", label("control_center.hex_color")),
        ("Enter", label("control_center.apply")),
        ("Esc", label("control_center.hint.back")),
      ]
      .map(|(keys, action)| format!("{keys} {action}"))
      .join(FOOTER_GAP),
      AppearancePage::Prompt { goal } => {
        let mut segments = Vec::new();
        if goal != PromptGoal::ExportProfile {
          segments.push(format!("0-9 {}", label("control_center.hint.edit")));
        }
        segments.push(format!("Enter {}", label("control_center.hint.confirm")));
        segments.push(format!("Esc {}", label("control_center.hint.back")));
        segments.join(FOOTER_GAP)
      }
      _ => {
        let mut extra = Vec::new();
        if self.page == AppearancePage::CustomThemes {
          extra.push(("d", label("control_center.delete")));
        }
        if matches!(
          self.page,
          AppearancePage::Themes | AppearancePage::CustomThemes
        ) {
          extra.push(("e", label("control_center.export")));
          extra.push(("i", label("control_center.import")));
        }
        let movable_row = matches!(
          self.menu.selected_id(rows),
          Some(Item::TelemetryBlock(_) | Item::PanelCard(_))
        );
        if movable_row
          && matches!(
            self.page,
            AppearancePage::SurfaceSection {
              surface: EffectSurface::WidgetTelemetry | EffectSurface::ControlPanel,
              section: SurfaceSection::Sessions,
            }
          )
        {
          extra.push(("⇧↑/⇧↓", label("control_center.move_block")));
        }
        hints(
          self.lang,
          &HintContext {
            row: self.menu.selected_kind(rows),
            can_go_back: true,
            refresh: true,
            extra: &extra,
            ..HintContext::default()
          },
        )
      }
    }
  }

  /// Renders the current page with the semantic theme.
  pub fn draw(&mut self, frame: &mut Frame) {
    let rows = self.rows();
    self.menu.normalize(&rows);
    // The Icons page draws its own footer (status left, shortcuts right).
    let icons_page = self.page == AppearancePage::Icons;
    let hints = if icons_page {
      String::new()
    } else {
      self.hints(&rows)
    };
    let area = shell(
      frame,
      frame.area(),
      &self.theme,
      self.lang,
      &self.breadcrumb(),
      &hints,
    );
    match self.page {
      AppearancePage::AccentEdit => self.draw_accent_editor(frame, area),
      AppearancePage::Prompt { goal } => self.draw_prompt(frame, area, goal),
      AppearancePage::Icons => self.draw_icons(frame, area, &rows),
      _ => {
        self.list_height = area.height;
        draw_menu(
          frame,
          area,
          &self.theme,
          &rows,
          &mut self.menu,
          MenuStyle {
            icons: AppConfig::icons_enabled(),
          },
        );
      }
    }
    if let Some((confirmation, state)) = &self.confirm {
      self.draw_confirmation(frame, area, *confirmation, state);
    }
    if icons_page {
      self.draw_icons_footer(frame, area);
      return;
    }
    if let Some(message) = &self.status {
      status(frame, area, &self.theme, message);
    }
    if self.status_loading {
      status(
        frame,
        area,
        &self.theme,
        &StatusMessage {
          kind: StatusKind::Info,
          text: tr(self.lang, "control_center.loading_appearance").into(),
        },
      );
    }
  }

  pub(super) fn draw_confirmation(
    &self,
    frame: &mut Frame,
    area: Rect,
    confirmation: Confirmation,
    state: &ConfirmState,
  ) {
    let label = |key: &str| tr(self.lang, key);
    // The theme dialogs name the theme above the description.
    let named = |name: &str, description: &str| format!("{name}\n{}", label(description));
    let (title, message, confirm, danger) = match confirmation {
      Confirmation::DiscardDraft => (
        "control_center.discard_changes_title",
        label("control_center.discard_changes_description").to_string(),
        "control_center.discard",
        false,
      ),
      Confirmation::DeleteTheme => (
        "control_center.theme_profile_delete_title",
        named(
          self
            .delete_theme
            .as_ref()
            .map(|theme| theme.name.as_str())
            .unwrap_or_default(),
          "control_center.theme_profile_delete_description",
        ),
        "control_center.theme_profile_delete",
        true,
      ),
      Confirmation::ReplaceImport => (
        "control_center.theme_profile_duplicate_title",
        named(
          self.pending_import_name.as_deref().unwrap_or_default(),
          "control_center.theme_profile_duplicate_description",
        ),
        "control_center.theme_profile_duplicate_replace",
        true,
      ),
    };
    let dialog = ConfirmDialog {
      title: label(title),
      message: &message,
      confirm: label(confirm),
      cancel: label("control_center.cancel"),
      danger,
      deadline: None,
    };
    draw_confirm(frame, area, &self.theme, dialog, state);
  }

  pub(super) fn draw_prompt(&mut self, frame: &mut Frame, area: Rect, goal: PromptGoal) {
    let label = tr(self.lang, self.prompt_label(goal));
    let chunks = Layout::vertical([
      Constraint::Length(3),
      Constraint::Length(1),
      Constraint::Length(1),
      Constraint::Min(1),
    ])
    .split(area);
    frame.render_widget(
      Paragraph::new(Line::from(Span::styled(
        format!("{label}: {}", self.prompt_buffer),
        Style::new()
          .fg(self.theme.selected_foreground)
          .bg(self.theme.selected_background),
      ))),
      chunks[0],
    );
    // The keys are in the footer; this line only states what is accepted.
    let text = self.prompt_error.clone().unwrap_or_else(|| {
      if matches!(goal, PromptGoal::ExportProfile) {
        return tr(self.lang, "control_center.theme_profile_export_name").to_string();
      }
      let (min, max) = goal.range();
      format!("{min}..{max}")
    });
    frame.render_widget(
      Paragraph::new(Line::from(Span::styled(
        text,
        Style::new()
          .fg(if self.prompt_error.is_some() {
            self.theme.error
          } else {
            self.theme.foreground
          })
          .add_modifier(Modifier::DIM),
      ))),
      chunks[1],
    );
    if matches!(goal, PromptGoal::ExportProfile) {
      let output = backend::preview_theme_export_path(&self.prompt_buffer)
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "—".into());
      frame.render_widget(
        Paragraph::new(format!(
          "{}: {}",
          tr(self.lang, "control_center.theme_profile_output"),
          output
        ))
        .style(Style::new().fg(self.theme.muted)),
        chunks[2],
      );
    }
  }

  pub(super) fn draw_accent_editor(&mut self, frame: &mut Frame, area: Rect) {
    let color = self.prompt_buffer.parse::<HexColor>().ok();
    let chunks = Layout::vertical([
      Constraint::Length(1),
      Constraint::Length(3),
      Constraint::Length(1),
      Constraint::Length(3),
      Constraint::Min(1),
    ])
    .split(area);
    frame.render_widget(
      Paragraph::new(tr(self.lang, "control_center.preview")),
      chunks[0],
    );
    let preview_style = color
      .map(|color| {
        Style::new()
          .bg(ratatui::style::Color::Rgb(color.r, color.g, color.b))
          .fg(color.foreground())
      })
      .unwrap_or_else(|| Style::new().fg(self.theme.error));
    frame.render_widget(
      Paragraph::new(Line::from(Span::styled(
        format!("  {}  ", self.prompt_buffer),
        preview_style,
      ))),
      chunks[1],
    );
    frame.render_widget(
      Paragraph::new(tr(self.lang, "control_center.hex_color")),
      chunks[2],
    );
    frame.render_widget(
      Paragraph::new(Line::from(Span::styled(
        format!("> {}", self.prompt_buffer),
        Style::new()
          .fg(self.theme.selected_foreground)
          .bg(self.theme.selected_background),
      ))),
      chunks[3],
    );
    if let Some(error) = &self.prompt_error {
      frame.render_widget(
        Paragraph::new(Span::styled(error, Style::new().fg(self.theme.error))),
        chunks[4],
      );
    }
  }
}
