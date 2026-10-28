//! Drawing of the Network & Wi-Fi page: lists, overlays, footer hints and the breadcrumb.

use super::*;

impl NetworkApp {
  pub(super) fn breadcrumb(&self) -> String {
    let root = format!(
      "{} > {}",
      HomeCardId::Hardware.label(self.lang),
      tr(self.lang, "control_center.network_wifi")
    );
    let connections = tr(self.lang, "control_center.net_connections");
    match self.page {
      NetworkPage::Home => root,
      NetworkPage::AddConnection => format!(
        "{root} > {connections} > {}",
        tr(self.lang, "control_center.net_add_connection")
      ),
      page if page.is_editor() => {
        format!(
          "{root} > {connections} > {}",
          self.editor_breadcrumb().join(" > ")
        )
      }
      _ => format!("{root} > {}", self.page_label()),
    }
  }

  /// Footer hints for the current row. `i Details` appears only on a Wi-Fi
  /// network or a VPN row, where it opens the details page.
  pub(super) fn footer_hints(&self, rows: &[Row<Item>]) -> String {
    if self.confirm_forget || self.editor_confirm.is_some() {
      return confirm_hints(self.lang);
    }
    let mut menu = self.menu;
    menu.normalize(rows);
    let details = [("i", tr(self.lang, "control_center.details"))];
    let selected = menu.selected_id(rows);
    let on_network = matches!(
      selected,
      Some(Item::WifiNetwork(_) | Item::VpnConnection(_))
    );
    let toggle_label = match selected {
      Some(Item::Profile(index)) if self.page == NetworkPage::Connections => {
        self.snapshot.profiles.get(index).map(|profile| {
          tr(
            self.lang,
            if profile.active {
              "control_center.disconnect"
            } else {
              "control_center.connect"
            },
          )
        })
      }
      _ => None,
    };
    let activation = [("a", toggle_label.unwrap_or_default())];
    let extra: &[(&str, &str)] = if on_network {
      &details
    } else if toggle_label.is_some() {
      &activation
    } else {
      &[]
    };
    hints(
      self.lang,
      &HintContext {
        row: menu.selected_kind(rows),
        can_go_back: true,
        search: self.is_list_page(),
        refresh: !self.page.is_editor(),
        extra,
        ..HintContext::default()
      },
    )
  }

  /// Renders the page: the single menu list, then the text fields, the
  /// confirmation and the status.
  pub fn draw(&mut self, f: &mut Frame) {
    let area = f.area();
    let rows = self.rows();
    self.menu.normalize(&rows);
    let body = shell(
      f,
      area,
      &self.theme,
      self.lang,
      &self.breadcrumb(),
      &self.footer_hints(&rows),
    );
    self.list_height = body.height;
    draw_menu(
      f,
      body,
      &self.theme,
      &rows,
      &mut self.menu,
      MenuStyle {
        icons: AppConfig::icons_enabled(),
      },
    );
    if let Some(input) = &self.dns_input {
      let popup = argvus_tui::chrome::centered(area, 48, 7);
      f.render_widget(Clear, popup);
      f.render_widget(
        Paragraph::new(vec![
          Line::from(tr(self.lang, "control_center.dns_servers_comma_separated")),
          Line::from(format!("{input}_")),
          Line::from(tr(self.lang, "control_center.enter_apply_esc_cancel")),
        ])
        .block(Block::bordered().title(tr(self.lang, "control_center.dns_servers"))),
        popup,
      );
    } else if let Some(password) = &self.password {
      let popup = argvus_tui::chrome::centered(area, 48, 7);
      f.render_widget(Clear, popup);
      let shown = "•".repeat(argvus_tui::text::display_width(password));
      f.render_widget(
        Paragraph::new(vec![
          Line::from(tr(self.lang, "control_center.wi_fi_network_password")),
          Line::from(format!("{shown}_")),
          Line::from(tr(self.lang, "control_center.enter_connect_esc_cancel")),
        ])
        .block(Block::bordered().title(tr(self.lang, "control_center.wi_fi_password"))),
        popup,
      );
    }
    if self.confirm_forget {
      let name = self
        .current_index()
        .and_then(|i| self.snapshot.wifi.get(i))
        .map(|w| terminal_text(&w.ssid))
        .unwrap_or_default();
      draw_confirm(
        f,
        area,
        &self.theme,
        ConfirmDialog {
          title: tr(self.lang, "control_center.forget_network"),
          message: &format!(
            "{} {}?",
            tr(self.lang, "control_center.forget_network_58478a"),
            name
          ),
          confirm: tr(self.lang, "control_center.forget"),
          cancel: tr(self.lang, "control_center.cancel"),
          danger: true,
          deadline: None,
        },
        &self.confirmation,
      );
    }
    if let Some((confirm, state)) = &self.editor_confirm {
      let (title, message, confirm_label, danger) = self.editor_confirm_text(*confirm);
      draw_confirm(
        f,
        area,
        &self.theme,
        ConfirmDialog {
          title: &title,
          message: &message,
          confirm: &confirm_label,
          cancel: tr(self.lang, "control_center.cancel"),
          danger,
          deadline: None,
        },
        state,
      );
    }
    if let Some((_, editor)) = &self.text_edit {
      editor.draw(
        f,
        area,
        &self.theme,
        tr(self.lang, "control_center.enter_apply_esc_cancel"),
      );
    }
    if let Some(status_message) = &self.status {
      status(f, body, &self.theme, status_message);
    }
    if self.job.is_some() {
      status(
        f,
        body,
        &self.theme,
        &StatusMessage {
          kind: StatusKind::Info,
          text: tr(
            self.lang,
            if self.scan {
              "control_center.scanning_for_networks"
            } else {
              "control_center.refreshing_network"
            },
          )
          .into(),
        },
      );
    }
  }
}
