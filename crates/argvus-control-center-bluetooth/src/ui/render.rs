//! Drawing of the Bluetooth page: lists, overlays, footer hints and the breadcrumb.

use super::*;

impl BluetoothApp {
  pub(super) fn footer_hints(&self) -> &'static str {
    match self.page {
      BluetoothPage::Home => tr(
        self.lang,
        "control_center.navigate_enter_open_esc_back_r_refresh_help",
      ),
      BluetoothPage::State | BluetoothPage::Devices | BluetoothPage::Pair => tr(
        self.lang,
        "control_center.navigate_tab_actions_move_enter_activate_r_refresh_esc_back_help",
      ),
    }
  }

  pub fn draw(&self, f: &mut Frame) {
    // Bluetooth is listed under the Hardware card, so its path starts there.
    let page = match self.page {
      BluetoothPage::Home => tr(self.lang, "control_center.bluetooth"),
      BluetoothPage::State => tr(self.lang, "control_center.bluetooth_state"),
      BluetoothPage::Devices => tr(self.lang, "control_center.bluetooth_devices"),
      BluetoothPage::Pair => tr(self.lang, "control_center.bluetooth_pair"),
    };
    let breadcrumb = join(&[HomeCardId::Hardware.label(self.lang), page]);
    let area = shell(
      f,
      f.area(),
      &self.theme,
      self.lang,
      &breadcrumb,
      self.footer_hints(),
    );
    let buttons = self.buttons();
    let raw_buttons: Vec<Button> = buttons.iter().map(|(_, button)| button.clone()).collect();
    let (body, button_area) = if raw_buttons.is_empty() {
      (area, None)
    } else {
      let button_height = argvus_tui::buttons::height(&raw_buttons, area.width).min(area.height);
      let split =
        Layout::vertical([Constraint::Min(1), Constraint::Length(button_height)]).split(area);
      (split[0], Some(split[1]))
    };
    let rows: Vec<String> = match self.page {
      BluetoothPage::Home => vec![
        tr(self.lang, "control_center.state").into(),
        tr(self.lang, "control_center.devices").into(),
        tr(self.lang, "control_center.pair").into(),
      ],
      BluetoothPage::State => {
        let a = self.snapshot.adapter.as_ref();
        vec![
          format!(
            "{}  {}",
            tr(self.lang, "control_center.adapter"),
            a.map(|x| x.name.as_str()).unwrap_or("—")
          ),
          format!(
            "{}  {}   {}",
            tr(self.lang, "control_center.powered"),
            a.map(|x| {
              if x.powered {
                tr(self.lang, "control_center.yes")
              } else {
                tr(self.lang, "control_center.no")
              }
            })
            .unwrap_or("—"),
            if self.scanning { "[SCAN]" } else { "" }
          ),
        ]
      }
      BluetoothPage::Devices | BluetoothPage::Pair => self
        .snapshot
        .devices
        .iter()
        .map(|d| {
          let state = if d.connected {
            tr(self.lang, "control_center.connected")
          } else if d.paired {
            tr(self.lang, "control_center.paired")
          } else {
            tr(self.lang, "control_center.known")
          };
          let trusted = if d.trusted {
            tr(self.lang, "control_center.trusted")
          } else {
            ""
          };
          format!(
            "{}  {}  {}{}{}",
            if d.name.is_empty() {
              tr(self.lang, "control_center.unnamed")
            } else {
              &d.name
            },
            d.address,
            state,
            trusted,
            d.battery.map(|v| format!(" {}%", v)).unwrap_or_default()
          )
        })
        .collect(),
    };
    list(
      f,
      body,
      &self.theme,
      &rows,
      self.selected.index.min(rows.len().saturating_sub(1)),
    );
    if let Some(button_area) = button_area {
      let focus = if self.on_buttons {
        self.button_selected
      } else {
        usize::MAX
      };
      argvus_tui::buttons::draw(f, button_area, &raw_buttons, focus, &self.theme);
    }
    if let Some(s) = &self.status {
      draw_status(
        f,
        Rect::new(
          body.x,
          body.y + body.height.saturating_sub(1),
          body.width,
          1,
        ),
        &self.theme,
        s,
      );
    }
    if self.confirm_remove {
      draw_confirmation(
        f,
        f.area(),
        &self.theme,
        ConfirmationDialog {
          title: tr(self.lang, "control_center.remove_device"),
          message: tr(
            self.lang,
            "control_center.the_device_will_be_removed_from_known_devices",
          ),
          confirm_label: tr(self.lang, "control_center.remove"),
          cancel_label: tr(self.lang, "control_center.cancel"),
          confirm_selected: self.confirmation.confirm_selected,
        },
      );
    }
    if let Some(prompt) = &self.prompt {
      self.draw_agent_prompt(f, prompt);
    }
  }

  pub(super) fn draw_agent_prompt(&self, f: &mut Frame, prompt: &AgentPrompt) {
    let device_name = prompt.device();
    let selected = |label: &str, active: bool| {
      if active {
        Span::styled(
          format!("[ {label} ]"),
          Style::new()
            .fg(self.theme.selected_foreground)
            .bg(self.theme.selected_background)
            .add_modifier(Modifier::BOLD),
        )
      } else {
        Span::styled(format!("[ {label} ]"), Style::new().fg(self.theme.accent))
      }
    };
    let input_line = |label: &str, value: &str| {
      Line::from(vec![
        Span::styled(format!("{label} "), Style::new().fg(self.theme.muted)),
        Span::styled(format!("{value}_"), Style::new().fg(self.theme.foreground)),
      ])
    };
    let mut lines = vec![Line::from("")];
    let (title, height) = match prompt {
      AgentPrompt::Pin { input, .. } => {
        lines.push(Line::from(tr(
          self.lang,
          "control_center.enter_the_pin_shown_by_the_device",
        )));
        lines.push(Line::from(""));
        lines.push(input_line("PIN:", input));
        lines.push(Line::from(""));
        (tr(self.lang, "control_center.device_pin").to_string(), 9)
      }
      AgentPrompt::Passkey { input, .. } => {
        lines.push(Line::from(tr(
          self.lang,
          "control_center.enter_the_verification_code_from_the_device",
        )));
        lines.push(Line::from(""));
        lines.push(input_line("Código:", input));
        lines.push(Line::from(""));
        (
          tr(self.lang, "control_center.verification_code").to_string(),
          9,
        )
      }
      AgentPrompt::DisplayPasskey { passkey, .. } => {
        lines.push(Line::from(self.lang.tr_args(
          "control_center.showing_the_number_check_it_on",
          [("name", device_name)],
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
          format!("{passkey:06}"),
          Style::new()
            .fg(self.theme.accent)
            .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
          selected(
            tr(self.lang, "control_center.continue"),
            self.prompt_confirm,
          ),
          Span::raw("  "),
          selected(tr(self.lang, "control_center.cancel"), !self.prompt_confirm),
        ]));
        (
          tr(self.lang, "control_center.verification_number").to_string(),
          9,
        )
      }
      AgentPrompt::Confirm { passkey, .. } => {
        lines.push(Line::from(self.lang.tr_args(
          "control_center.device_shows_the_number",
          [("name", device_name)],
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
          format!("{passkey:06}"),
          Style::new()
            .fg(self.theme.accent)
            .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
          selected(tr(self.lang, "control_center.yes"), self.prompt_confirm),
          Span::raw("  "),
          selected(tr(self.lang, "control_center.no"), !self.prompt_confirm),
        ]));
        (
          tr(self.lang, "control_center.confirm_pairing").to_string(),
          9,
        )
      }
      AgentPrompt::Authorize { .. } => {
        lines.push(Line::from(self.lang.tr_args(
          "control_center.allow_device_to_be_paired",
          [("name", device_name)],
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
          selected(tr(self.lang, "control_center.allow"), self.prompt_confirm),
          Span::raw("  "),
          selected(tr(self.lang, "control_center.cancel"), !self.prompt_confirm),
        ]));
        (
          tr(self.lang, "control_center.authorize_pairing").to_string(),
          8,
        )
      }
      AgentPrompt::Service { uuid, .. } => {
        lines.push(Line::from(self.lang.tr_args(
          "control_center.device_requests_access_to_service",
          [("name", device_name)],
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
          uuid,
          Style::new().fg(self.theme.foreground),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
          selected(
            tr(self.lang, "control_center.allow_d0cc72"),
            self.prompt_confirm,
          ),
          Span::raw("  "),
          selected(tr(self.lang, "control_center.deny"), !self.prompt_confirm),
        ]));
        (
          tr(self.lang, "control_center.service_access").to_string(),
          9,
        )
      }
    };
    lines.push(Line::from(tr(
      self.lang,
      "control_center.enter_ok_esc_cancel",
    )));
    let popup = centered(
      f.area(),
      f.area().width.saturating_sub(8).clamp(36, 72),
      height,
    );
    f.render_widget(Clear, popup);
    f.render_widget(
      Paragraph::new(lines)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true })
        .block(
          Block::bordered()
            .title(format!(" {title} "))
            .border_style(Style::new().fg(self.theme.border_active))
            .style(Style::new().bg(self.theme.background)),
        ),
      popup,
    );
  }
}
