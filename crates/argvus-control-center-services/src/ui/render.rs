//! Drawing of the Services page: lists, overlays, footer hints and the breadcrumb.

use super::rows::action_label;
use super::*;

impl ServicesApp {
  pub fn breadcrumb(&self) -> String {
    let root = format!(
      "{} > {}",
      HomeCardId::System.label(self.lang),
      tr(self.lang, "control_center.services")
    );
    if self.page == ServicePage::Home {
      root
    } else {
      format!("{root} > {}", self.label(self.page))
    }
  }

  pub(super) fn footer_hints(&self) -> String {
    match self.page {
      ServicePage::LogDetail(_) => tr(self.lang, "control_center.scroll_esc_back_help").into(),
      ServicePage::Home => tr(
        self.lang,
        "control_center.navigate_enter_open_r_refresh_esc_back_help",
      )
      .into(),
      ServicePage::System | ServicePage::User | ServicePage::Failed => {
        let mut menu = self.menu;
        let rows = self.list_rows();
        menu.normalize(&rows);
        hints(
          self.lang,
          &HintContext {
            row: menu.selected_kind(&rows),
            can_go_back: true,
            search: true,
            refresh: true,
            extra: &[("f", tr(self.lang, "control_center.filter"))],
            ..HintContext::default()
          },
        )
      }
      ServicePage::Logs => hints(
        self.lang,
        &HintContext {
          can_go_back: true,
          search: true,
          refresh: true,
          extra: &[
            ("b", tr(self.lang, "control_center.boot")),
            ("u", tr(self.lang, "control_center.service")),
            ("p", tr(self.lang, "control_center.priority")),
          ],
          ..HintContext::default()
        },
      ),
      ServicePage::Detail => {
        let mut menu = self.menu;
        let rows = self.detail_rows();
        menu.normalize(&rows);
        hints(
          self.lang,
          &HintContext {
            row: menu.selected_kind(&rows),
            can_go_back: true,
            refresh: true,
            ..HintContext::default()
          },
        )
      }
    }
  }

  pub fn draw(&mut self, frame: &mut Frame) {
    let area = frame.area();
    let body = shell(
      frame,
      area,
      &self.theme,
      self.lang,
      &self.breadcrumb(),
      &self.footer_hints(),
    );
    if self.is_list_page() {
      self.draw_list(frame, area, body);
      return;
    }
    if self.page == ServicePage::Logs {
      self.draw_logs(frame, area, body);
      return;
    }
    if self.page == ServicePage::Detail {
      self.draw_detail(frame, area, body);
      return;
    }
    let lines: Vec<String> = if self.page == ServicePage::Home {
      self.home_rows()
    } else if let ServicePage::LogDetail(index) = self.page {
      self
        .log_detail_lines(index)
        .into_iter()
        .map(|l| l.to_string())
        .collect()
    } else {
      Vec::new()
    };
    let list_selection = if self.job.is_some() {
      usize::MAX
    } else {
      self.selected.min(lines.len().saturating_sub(1))
    };
    list(frame, body, &self.theme, &lines, list_selection);
    self.draw_overlays(frame, area);
  }

  /// Renders a unit list in the shared layout: the filter line, the search
  /// box and the units in a `Services:` block.
  pub(super) fn draw_list(&mut self, frame: &mut Frame, area: Rect, body: Rect) {
    let rows = self.list_rows();
    self.menu.normalize(&rows);
    let [filter_area, search_area, units_area] = Layout::vertical([
      Constraint::Length(1),
      Constraint::Length(3),
      Constraint::Min(3),
    ])
    .areas(body);
    filter_line(
      frame,
      filter_area,
      &self.theme,
      tr(self.lang, "control_center.filter"),
      self.filter.label(self.lang),
    );
    search_box(
      frame,
      search_area,
      &self.theme,
      tr(self.lang, "control_center.search_2c43ee"),
      &self.search,
      self.searching,
    );
    let menu_area = titled_block(
      frame,
      units_area,
      &self.theme,
      tr(self.lang, "control_center.services_list"),
    );
    draw_menu(
      frame,
      menu_area,
      &self.theme,
      &rows,
      &mut self.menu,
      MenuStyle {
        icons: AppConfig::icons_enabled(),
      },
    );
    self.draw_overlays(frame, area);
  }

  /// Renders the journal in the same layout as the unit lists: the boot,
  /// unit and priority filters on the filter line, the search over the
  /// messages, then the entries in a `Logs:` block.
  pub(super) fn draw_logs(&mut self, frame: &mut Frame, area: Rect, body: Rect) {
    let [filter_area, search_area, entries_area] = Layout::vertical([
      Constraint::Length(1),
      Constraint::Length(3),
      Constraint::Min(3),
    ])
    .areas(body);
    filter_line(
      frame,
      filter_area,
      &self.theme,
      tr(self.lang, "control_center.filter"),
      &self.logs_filter_summary(),
    );
    search_box(
      frame,
      search_area,
      &self.theme,
      tr(self.lang, "control_center.search_2c43ee"),
      &self.log_search,
      self.searching,
    );
    let lines: Vec<String> = self
      .filtered_logs()
      .into_iter()
      .map(|entry| log_line(entry).to_string())
      .collect();
    let list_area = titled_block(
      frame,
      entries_area,
      &self.theme,
      tr(self.lang, "control_center.logs_list"),
    );
    let list_selection = if self.job.is_some() {
      usize::MAX
    } else {
      self.selected.min(lines.len().saturating_sub(1))
    };
    list(frame, list_area, &self.theme, &lines, list_selection);
    self.draw_overlays(frame, area);
  }

  /// Renders the details page: the service information and its actions.
  pub(super) fn draw_detail(&mut self, frame: &mut Frame, area: Rect, body: Rect) {
    let rows = self.detail_rows();
    self.menu.normalize(&rows);
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
    self.draw_overlays(frame, area);
  }

  /// Status line, loading indicator and the confirmation, shared by the
  /// menu pages.
  pub(super) fn draw_overlays(&self, frame: &mut Frame, area: Rect) {
    if let Some(s) = &self.status {
      status(frame, area, &self.theme, s)
    }
    if self.job.is_some() {
      status(
        frame,
        area,
        &self.theme,
        &StatusMessage {
          kind: StatusKind::Info,
          text: tr(self.lang, "control_center.loading_services").into(),
        },
      );
    }
    if let Some(Pending::Action(a, u)) = &self.pending {
      self.draw_pending(frame, area, a, u)
    }
  }

  pub(super) fn draw_pending(
    &self,
    frame: &mut Frame,
    area: ratatui::layout::Rect,
    action: &str,
    unit: &str,
  ) {
    draw_confirmation(
      frame,
      area,
      &self.theme,
      ConfirmationDialog {
        title: tr(self.lang, "control_center.confirm_action"),
        message: &format!("{} {}?", action_label(self.lang, action), unit),
        confirm_label: tr(self.lang, "control_center.continue"),
        cancel_label: tr(self.lang, "control_center.cancel"),
        confirm_selected: self.confirmation.confirm_selected,
      },
    )
  }
}
