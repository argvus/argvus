//! Drawing of the login screen: clock, user list, login panel, messages, footer and power menu.

use super::*;

impl LoginApp {
  /// Renders the complete screen, including the optional power overlay.
  ///
  /// The small-terminal guard prevents Ratatui widgets and image protocols
  /// from competing for a region too small to remain legible.
  pub fn draw(&mut self, frame: &mut Frame) {
    let area = frame.area();
    if area.width < argvus_tui::MIN_WIDTH || area.height < argvus_tui::MIN_HEIGHT {
      argvus_tui::chrome::draw_too_small(
        frame,
        area,
        &self.theme,
        &self.i18n.tr("terminal_window_is_too_small"),
        &self.i18n.tr("minimum_size"),
        &self.i18n.tr("current_size"),
      );
      return;
    }

    // The outer frame establishes a consistent ARGVUS background and border
    // before child widgets draw their own focused panels.
    Block::new()
      .borders(Borders::ALL)
      .border_style(Style::new().fg(self.theme.border_active))
      .bg(self.theme.background)
      .render(area, frame.buffer_mut());
    let inner = area.inner(Margin::new(1, 1));
    // Header, clock, body, message box, and footer use explicit regions. The
    // message box is intentionally outside the form so every status appears
    // in one predictable location only.
    let rows = Layout::vertical([
      Constraint::Length(1),
      Constraint::Length(2),
      Constraint::Min(1),
      Constraint::Length(3),
      Constraint::Length(2),
    ])
    .split(inner);
    argvus_tui::chrome::draw_header(
      frame,
      rows[0],
      &self.theme,
      argvus_tui::chrome::Header {
        title: &self.i18n.tr("title"),
        version: None,
        version_label: "",
      },
    );
    self.draw_clock(frame, rows[1]);
    self.draw_body(frame, rows[2]);
    self.draw_message_box(frame, rows[3]);
    self.draw_footer(frame, rows[4]);

    if self.power_open {
      self.draw_power_menu(frame, area);
    }
  }

  /// Draws the optional local clock and date without affecting form state.
  pub(super) fn draw_clock(&self, frame: &mut Frame, area: Rect) {
    if !self.config.appearance.show_clock {
      return;
    }
    // Local time is best effort because the login process may lack a complete
    // timezone environment. The epoch fallback keeps rendering infallible.
    let now = OffsetDateTime::now_local().unwrap_or(OffsetDateTime::UNIX_EPOCH);
    let time = now
      .format(&format_description!("[hour]:[minute]"))
      .unwrap_or_else(|_| "00:00".to_string());
    let date = if self.config.appearance.show_date {
      now
        .format(&format_description!("[year]-[month]-[day]"))
        .unwrap_or_else(|_| "1970-01-01".to_string())
    } else {
      String::new()
    };
    frame.render_widget(
      Paragraph::new(Line::from(vec![
        Span::styled(
          time,
          Style::new()
            .fg(self.theme.foreground)
            .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
          if date.is_empty() {
            String::new()
          } else {
            format!("  {date}")
          },
          Style::new().fg(self.theme.muted),
        ),
      ])),
      area,
    );
  }

  /// Splits the body between account navigation and authentication controls.
  pub(super) fn draw_body(&mut self, frame: &mut Frame, area: Rect) {
    let columns =
      Layout::horizontal([Constraint::Percentage(38), Constraint::Percentage(62)]).split(area);
    self.draw_users(frame, columns[0]);
    self.draw_login_panel(frame, columns[1]);
  }

  /// Renders the account list and its focused selection state.
  pub(super) fn draw_users(&self, frame: &mut Frame, area: Rect) {
    let title = format!(" {} ", self.i18n.tr("users"));
    let items = if self.users.is_empty() {
      vec![ListItem::new(self.i18n.tr("no_local_login_users"))]
    } else {
      self
        .users
        .iter()
        .enumerate()
        .map(|(index, user)| {
          // The icon communicates avatar availability without exposing the
          // underlying filesystem path to the person logging in.
          let avatar = if user.avatar.is_some() {
            "󰀄 "
          } else {
            "󰀉 "
          };
          let style = if index == self.selected_user && self.focus == Focus::User {
            Style::new()
              .fg(self.theme.selected_foreground)
              .bg(self.theme.selected_background)
              .add_modifier(Modifier::BOLD)
          } else {
            Style::new().fg(self.theme.foreground)
          };
          ListItem::new(format!("{avatar}{}", user.display_name)).style(style)
        })
        .collect()
    };
    let border = if self.focus == Focus::User {
      self.theme.border_active
    } else {
      self.theme.border
    };
    frame.render_widget(
      List::new(items)
        .block(
          Block::bordered()
            .title(title)
            .border_style(Style::new().fg(border)),
        )
        .highlight_style(
          Style::new()
            .fg(self.theme.selected_foreground)
            .bg(self.theme.selected_background),
        ),
      area,
    );
  }

  /// Renders the account preview, password response, session selector, and
  /// login action. Status messages are rendered by [`Self::draw_message_box`]
  /// so they cannot be duplicated inside the form.
  pub(super) fn draw_login_panel(&mut self, frame: &mut Frame, area: Rect) {
    let inner = area.inner(Margin::new(2, 1));
    // The account row is taller than the other controls so the avatar can be
    // framed without collapsing its image protocol target to one cell.
    let rows = Layout::vertical([
      Constraint::Length(5),
      Constraint::Length(3),
      Constraint::Length(3),
      Constraint::Length(3),
    ])
    .split(inner);
    let selected_name = self
      .users
      .get(self.selected_user)
      .map(|user| user.display_name.as_str())
      .unwrap_or("ARGVUS");
    // Account focus is represented by the password/login controls because the
    // account preview itself is informational after selection.
    let border = if matches!(self.focus, Focus::Password | Focus::Login) {
      self.theme.border_active
    } else {
      self.theme.border
    };
    frame.render_widget(
      Block::bordered()
        .title(self.i18n.tr("account"))
        .border_style(Style::new().fg(border)),
      rows[0],
    );
    // Kitty preserves the avatar's square aspect ratio in terminal-cell
    // coordinates. A six-cell inner target at this row height matches the
    // visible image width, keeping the frame adjacent to the actual pixels
    // instead of surrounding unused horizontal letterboxing space.
    let account_columns = Layout::horizontal([Constraint::Length(8), Constraint::Min(1)])
      .split(rows[0].inner(Margin::new(1, 0)));
    if let Some(avatar) = self.avatar.as_mut() {
      // Draw the frame first and render the image into its inner rectangle so
      // the border remains visible for both Kitty graphics and text fallback.
      let avatar_area = account_columns[0];
      frame.render_widget(
        Block::bordered().border_style(Style::new().fg(border)),
        avatar_area,
      );
      avatar.render(
        frame,
        avatar_area.inner(Margin::new(1, 1)),
        self.theme.background,
        self.theme.accent,
      );
    }
    // Only the selected account name is shown here. The actual avatar source
    // is an implementation detail and must not be presented as login text.
    frame.render_widget(
      Paragraph::new(Line::from(Span::styled(
        selected_name,
        Style::new()
          .fg(self.theme.foreground)
          .add_modifier(Modifier::BOLD),
      )))
      .alignment(Alignment::Left),
      account_columns[1],
    );
    let prompt = if self.prompt.message.is_empty() {
      self.i18n.tr("password")
    } else {
      i18n::auth_message(&self.i18n, &self.prompt.message)
    };
    let password = if self.prompt.secret {
      "•".repeat(self.password.chars().count())
    } else {
      self.password.clone()
    };
    frame.render_widget(
      Paragraph::new(password).block(Block::bordered().title(prompt).border_style(
        Style::new().fg(if self.focus == Focus::Password {
          self.theme.border_active
        } else {
          self.theme.border
        }),
      )),
      rows[1],
    );
    let session = self
      .sessions
      .get(self.selected_session)
      .map(|item| item.name.as_str())
      .unwrap_or("-");
    let session_style = if self.focus == Focus::Session {
      Style::new()
        .fg(self.theme.selected_foreground)
        .bg(self.theme.selected_background)
    } else {
      Style::new().fg(self.theme.foreground)
    };
    frame.render_widget(
      Paragraph::new(Line::from(Span::styled(
        format!("‹ {session} ›"),
        session_style,
      )))
      .block(
        Block::bordered()
          .title(self.i18n.tr("session"))
          .border_style(Style::new().fg(if self.focus == Focus::Session {
            self.theme.border_active
          } else {
            self.theme.border
          })),
      ),
      rows[2],
    );
    let login_style = if self.focus == Focus::Login {
      Style::new()
        .fg(self.theme.selected_foreground)
        .bg(self.theme.accent)
        .add_modifier(Modifier::BOLD)
    } else {
      Style::new()
        .fg(self.theme.foreground)
        .bg(self.theme.surface)
    };
    frame.render_widget(
      Paragraph::new(Line::from(Span::styled(
        format!("  {}  ", self.i18n.tr("login")),
        login_style,
      )))
      .alignment(Alignment::Center)
      .block(
        Block::bordered().border_style(Style::new().fg(if self.focus == Focus::Login {
          self.theme.accent
        } else {
          self.theme.border
        })),
      ),
      rows[3],
    );
  }

  /// Renders the single status channel for progress, warnings, errors, and
  /// successful operations directly above the navigation footer.
  pub(super) fn draw_message_box(&self, frame: &mut Frame, area: Rect) {
    let (message, color) = match self.status.as_ref() {
      Some((text, StatusKind::Info)) => (text.clone(), self.theme.muted),
      Some((text, StatusKind::Warn)) => (text.clone(), self.theme.warning),
      Some((text, StatusKind::Error)) => (text.clone(), self.theme.error),
      Some((text, StatusKind::Success)) => (text.clone(), self.theme.success),
      None => (self.i18n.tr("select_user_help"), self.theme.muted),
    };
    frame.render_widget(
      Paragraph::new(message)
        .wrap(Wrap { trim: true })
        .style(Style::new().fg(color))
        .block(
          Block::bordered()
            .title(self.i18n.tr("status"))
            .border_style(Style::new().fg(color)),
        ),
      area,
    );
  }

  /// Renders the keyboard help line shared by all normal login states.
  pub(super) fn draw_footer(&self, frame: &mut Frame, area: Rect) {
    frame.render_widget(
      Paragraph::new(self.i18n.tr("navigation_help"))
        .alignment(Alignment::Right)
        .style(Style::new().fg(self.theme.muted).bg(self.theme.surface)),
      area,
    );
  }

  /// Draws a centered power menu above the normal screen contents.
  pub(super) fn draw_power_menu(&self, frame: &mut Frame, area: Rect) {
    // Clear prevents the underlying form from visually leaking through the
    // modal while retaining the terminal's current background.
    let popup = argvus_tui::chrome::centered(area, area.width.min(42), 9);
    frame.render_widget(Clear, popup);
    let actions = [
      PowerAction::Shutdown,
      PowerAction::Restart,
      PowerAction::Suspend,
    ];
    let items = actions
      .iter()
      .enumerate()
      .map(|(index, action)| {
        let style = if index == self.power_selected {
          Style::new()
            .fg(self.theme.selected_foreground)
            .bg(self.theme.selected_background)
            .add_modifier(Modifier::BOLD)
        } else {
          Style::new().fg(self.theme.foreground)
        };
        ListItem::new(action.label(&self.i18n)).style(style)
      })
      .collect::<Vec<_>>();
    frame.render_widget(
      List::new(items).block(
        Block::bordered()
          .title(self.i18n.tr("power"))
          .border_style(Style::new().fg(self.theme.border_active)),
      ),
      popup,
    );
  }
}
