//! Drawing of the page: lists, search box, update actions, overlays, the transaction window and the footer hints.

use super::actions::{action_message, choice_label, preview_message};
use super::*;

pub(super) const TRANSACTION_POPUP_WIDTH: u16 = 100;

pub(super) const TRANSACTION_POPUP_HEIGHT: u16 = 20;

pub(super) const TRANSACTION_CONTENT_WIDTH: usize = TRANSACTION_POPUP_WIDTH as usize - 2;

pub(super) const TRANSACTION_CONTENT_HEIGHT: usize = TRANSACTION_POPUP_HEIGHT as usize - 2;

pub(super) fn transaction_wrapped_lines(output: &str) -> usize {
  output
    .lines()
    .map(|line| (argvus_tui::text::display_width(line).div_ceil(TRANSACTION_CONTENT_WIDTH)).max(1))
    .sum()
}

pub(super) fn transaction_bottom_offset(output: &str) -> u16 {
  transaction_wrapped_lines(output).saturating_sub(TRANSACTION_CONTENT_HEIGHT) as u16
}

/// Whether typed characters filter the list on `page` (D8): letters,
/// including `j`, `k`, `r`, `q` and `?`, become text of the filter.
pub(super) fn filters_while_typing(page: PackagesPage) -> bool {
  matches!(page, PackagesPage::Task(..) | PackagesPage::Orphans)
}

/// The Update lists, which keep the action bar under the list.
pub(super) fn has_action_bar(page: PackagesPage) -> bool {
  matches!(page, PackagesPage::Task(_, PackageTask::Update))
}

impl PackagesApp {
  /// The footer derived from the selected row.
  pub(super) fn footer_hints(&self, rows: &[Row<Item>]) -> String {
    if self.discard.is_some() {
      return confirm_hints(self.lang);
    }
    if let Some(pending) = &self.pending {
      return if pending.choices().is_some() {
        choice_hints(self.lang)
      } else {
        confirm_hints(self.lang)
      };
    }
    if let PackagesPage::Task(_, task) = self.page {
      return self.task_footer_hints(rows, task);
    }
    let mut menu = self.menu;
    menu.normalize(rows);
    let typing = filters_while_typing(self.page);
    // Enter opens an orphan's details; Space marks it.
    let row = match menu.selected_kind(rows) {
      Some(RowKind::Toggle { .. }) if self.page == PackagesPage::Orphans => Some(RowKind::Submenu),
      kind => kind,
    };
    let mark = [("Space", tr(self.lang, "control_center.mark"))];
    hints(
      self.lang,
      &HintContext {
        row,
        can_go_back: true,
        search: typing,
        refresh: !typing && self.page != PackagesPage::MirrorEditor,
        extra: if self.page == PackagesPage::Orphans {
          &mark
        } else {
          &[]
        },
        ..HintContext::default()
      },
    )
  }

  /// The footer of a task list: Enter runs the task on the package and `→`
  /// opens its details; on the Update lists, `Tab` reaches the action bar.
  pub(super) fn task_footer_hints(&self, rows: &[Row<Item>], task: PackageTask) -> String {
    let label = |key| tr(self.lang, key);
    let bar_focused = self.actions.cursor().is_some();
    let mut menu = self.menu;
    menu.normalize(rows);
    let selected = menu.selected_id(rows);
    let mut extra = Vec::new();
    if bar_focused {
      extra.push(("Enter", label("control_center.hint.run")));
    } else if selected.is_some_and(Item::is_list_entry) {
      extra.push(("Enter", self.task_label(task)));
      extra.push(("→", label("control_center.details")));
    } else if selected.is_some() {
      extra.push(("→/Enter", label("control_center.hint.open")));
    }
    if has_action_bar(self.page) && !bar_focused && !self.enabled_action_indexes().is_empty() {
      extra.push(("Tab", label("control_center.actions")));
    }
    hints(
      self.lang,
      &HintContext {
        // The row's own segment would name `→/Enter`; the task is listed above.
        row: Some(RowKind::Info),
        can_go_back: true,
        search: !bar_focused,
        extra: &extra,
        ..HintContext::default()
      },
    )
  }

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
    let body = argvus_tui::page::admin_warning(
      f,
      body,
      &self.theme,
      self.lang,
      !matches!(
        self.page,
        PackagesPage::Home
          | PackagesPage::Source(_)
          | PackagesPage::CacheFiles
          | PackagesPage::History
          | PackagesPage::HistoryDetails(_)
          | PackagesPage::Repositories
      ),
    );
    if filters_while_typing(self.page) {
      self.draw_filtered_list(f, body, &rows);
    } else {
      self.list_height = body.height;
      self.draw_list(f, body, &rows);
    }
    self.overlays(f, body);
  }

  /// Draws `rows` as the page list, with the menu style of the Packages pages.
  pub(super) fn draw_list(&mut self, f: &mut Frame, area: Rect, rows: &[Row<Item>]) {
    draw_menu(
      f,
      area,
      &self.theme,
      rows,
      &mut self.menu,
      MenuStyle {
        icons: AppConfig::icons_enabled(),
      },
    );
  }

  /// The lists that filter while typing, in the layout of Time Zone: the
  /// filter fixed on top and the list in a bordered block (`Results`, or
  /// `Updates` with its action bar at the bottom). The six task lists share
  /// this screen.
  pub(super) fn draw_filtered_list(&mut self, f: &mut Frame, area: Rect, rows: &[Row<Item>]) {
    let has_action_bar = has_action_bar(self.page);
    let [search, list, bar] = Layout::vertical([
      Constraint::Length(3),
      Constraint::Min(3),
      Constraint::Length(if has_action_bar { 3 } else { 0 }),
    ])
    .areas(area);
    self.draw_search_box(f, search);
    let title = if has_action_bar {
      "control_center.updates"
    } else {
      "control_center.search_results"
    };
    // As on Time Zone, the list and the bar keep the active border; the focus
    // shows on the selected row or on the focused button.
    let block = Block::bordered()
      .border_style(Style::new().fg(self.theme.border_active))
      .title(tr(self.lang, title));
    let inner = block.inner(list);
    f.render_widget(block, list);
    self.list_height = inner.height;
    self.draw_list(f, inner, rows);
    if has_action_bar {
      self.draw_updates_actions(f, bar);
    }
  }

  /// The filter of a list: a bordered `Search:` line that shows the typed
  /// text, highlighted while the letters go to it (the list has the focus).
  pub(super) fn draw_search_box(&self, f: &mut Frame, area: Rect) {
    let (border, cursor) = if self.actions.cursor().is_none() && self.captures_text() {
      (self.theme.border_active, "▌")
    } else {
      (self.theme.border, "")
    };
    let line = Line::from(vec![
      Span::styled(
        format!("{}: ", tr(self.lang, "control_center.search_2c43ee")),
        Style::new()
          .fg(self.theme.accent)
          .add_modifier(Modifier::BOLD),
      ),
      Span::styled(
        format!("{}{cursor}", self.query),
        Style::new().fg(self.theme.foreground),
      ),
    ]);
    f.render_widget(
      Paragraph::new(line)
        .style(Style::new().bg(self.theme.background))
        .block(Block::bordered().border_style(Style::new().fg(border))),
      area,
    );
  }

  /// The Updates action bar: a bordered `Actions` block with one button per
  /// action row; the focused button is highlighted while the bar has the focus.
  pub(super) fn draw_updates_actions(&self, f: &mut Frame, area: Rect) {
    let rows = self.update_action_rows();
    let block = Block::bordered()
      .border_style(Style::new().fg(self.theme.border_active))
      .title(format!(
        "{} · {}",
        tr(self.lang, "control_center.actions"),
        tr(self.lang, "control_center.actions_hint")
      ));
    let inner = block.inner(area);
    f.render_widget(block, area);
    let buttons: Vec<ActionButton> = rows
      .iter()
      .map(|row| {
        let icon = if AppConfig::icons_enabled() {
          row.icon_glyph().unwrap_or("")
        } else {
          ""
        };
        if row.is_selectable() {
          ActionButton::primary(icon, row.label(), "")
        } else {
          ActionButton::secondary(icon, row.label(), "")
        }
      })
      .collect();
    draw_aligned_transparent(
      f,
      inner,
      &buttons,
      self.actions.cursor().unwrap_or(usize::MAX),
      &self.theme,
      Alignment::Left,
    );
  }

  /// Draws the input field, the confirmation, the status and the live
  /// transaction output over the page.
  pub(super) fn overlays(&self, f: &mut Frame, body: Rect) {
    if let Some(input) = &self.input {
      let mode = self.input_mode.unwrap_or(InputMode::Search);
      let (title, prompt) = match mode {
        InputMode::Search => (
          tr(self.lang, "control_center.search_2c43ee"),
          tr(self.lang, "control_center.search_packages"),
        ),
        InputMode::MirrorAge => (
          tr(self.lang, "control_center.maximum_age"),
          tr(self.lang, "control_center.input_5ca63b"),
        ),
        InputMode::MirrorCount => (
          tr(self.lang, "control_center.count"),
          tr(self.lang, "control_center.input_5ca63b"),
        ),
      };
      let popup = chrome::centered(f.area(), 50, 7);
      f.render_widget(Clear, popup);
      f.render_widget(
        Paragraph::new(vec![
          Line::from(prompt),
          Line::from(format!("{input}_")),
          Line::from(tr(self.lang, "control_center.enter_apply_esc_cancel")),
        ])
        .block(Block::bordered().title(title)),
        popup,
      );
    }
    if let Some(a) = &self.pending
      && let Some(choices) = a.choices()
    {
      // Each option names what it removes, so the plan counters (which
      // describe only the cascade) are left out.
      let labels = choices.map(|choice| choice_label(self.lang, &choice, self.preview.as_ref()));
      draw_choice(
        f,
        f.area(),
        &self.theme,
        ChoiceDialog {
          title: tr(self.lang, "control_center.confirm_transaction"),
          message: &action_message(self.lang, a),
          options: &labels.each_ref().map(String::as_str),
          cancel: tr(self.lang, "control_center.cancel"),
          danger: true,
        },
        &self.choice,
      );
    } else if let Some(a) = &self.pending {
      let message = format!(
        "{}{}",
        action_message(self.lang, a),
        self
          .preview
          .as_ref()
          .map(|plan| preview_message(self.lang, plan))
          .unwrap_or_default()
      );
      draw_confirm(
        f,
        f.area(),
        &self.theme,
        ConfirmDialog {
          title: tr(self.lang, "control_center.confirm_transaction"),
          message: &message,
          confirm: tr(self.lang, "control_center.apply"),
          cancel: tr(self.lang, "control_center.cancel"),
          danger: a.is_destructive(),
          deadline: None,
        },
        &self.confirmation,
      );
    }
    if let Some(state) = &self.discard {
      draw_confirm(
        f,
        f.area(),
        &self.theme,
        ConfirmDialog {
          title: tr(self.lang, "control_center.repository_add"),
          message: tr(self.lang, "control_center.repo_discard"),
          confirm: tr(self.lang, "control_center.repo_discard_confirm"),
          cancel: tr(self.lang, "control_center.cancel"),
          danger: false,
          deadline: None,
        },
        state,
      );
    }
    if let Some((_, editor)) = &self.editor {
      let hint = tr(
        self.lang,
        if editor.is_multiline() {
          "control_center.repo_editor_multi"
        } else {
          "control_center.repo_editor_single"
        },
      );
      editor.draw(f, f.area(), &self.theme, hint);
    }
    if let Some(s) = &self.status {
      status(f, body, &self.theme, s)
    }
    if self.transaction_open
      && let Some(output) = self.transaction_live.as_ref().map(|live| live.output())
    {
      let popup = chrome::centered(f.area(), TRANSACTION_POPUP_WIDTH, TRANSACTION_POPUP_HEIGHT);
      let total = transaction_wrapped_lines(&output);
      let shown_total = total.max(1);
      let current = (self.transaction_scroll as usize + 1).min(shown_total);
      f.render_widget(Clear, popup);
      f.render_widget(
        Paragraph::new(output.as_str())
          .block(
            Block::new()
              .borders(Borders::ALL)
              .title(Line::styled(
                format!(
                  " {} · {} {} {} {} ",
                  tr(self.lang, "control_center.transaction_output"),
                  tr(self.lang, "control_center.line"),
                  current,
                  tr(self.lang, "control_center.of"),
                  shown_total,
                ),
                Style::new().fg(self.theme.accent),
              ))
              .border_style(Style::new().fg(self.theme.border_active))
              .style(Style::new().bg(self.theme.background))
              .title_bottom(Line::from(tr(
                self.lang,
                "control_center.scroll_pgup_pgdn_home_end_esc_close",
              ))),
          )
          .scroll((self.transaction_scroll, 0))
          .wrap(Wrap { trim: false }),
        popup,
      );
    }
  }

  /// The path of `page` under `Packages`, built from the rows that open it.
  pub(super) fn label_of(&self, page: PackagesPage) -> String {
    let label = |key| tr(self.lang, key).to_owned();
    match page {
      PackagesPage::Source(source) => self.source_label(source).to_owned(),
      PackagesPage::Task(source, task) => {
        format!("{} > {}", self.source_label(source), self.task_label(task))
      }
      PackagesPage::Orphans => format!(
        "{} > {}",
        self.label_of(PackagesPage::Task(
          PackageSource::Official,
          PackageTask::Remove
        )),
        label("control_center.orphans_29aae8")
      ),
      PackagesPage::Details => format!(
        "{} > {}",
        self.label_of(self.details_parent),
        self.details_target().unwrap_or_default()
      ),
      PackagesPage::Cache => label("control_center.cache"),
      PackagesPage::CacheFiles => format!(
        "{} > {}",
        label("control_center.cache"),
        label("control_center.cache_files")
      ),
      PackagesPage::History | PackagesPage::HistoryDetails(_) => label("control_center.history"),
      PackagesPage::Downgrade => label("control_center.downgrade"),
      PackagesPage::Mirrors => label("control_center.mirrors"),
      PackagesPage::MirrorEditor => format!(
        "{} > {}",
        label("control_center.mirrors"),
        label("control_center.configure_mirrors")
      ),
      PackagesPage::Home => label("control_center.packages"),
      PackagesPage::Repositories => format!(
        "{} > {}",
        self.source_label(PackageSource::Official),
        label("control_center.repositories")
      ),
      PackagesPage::RepositoryAdd => format!(
        "{} > {}",
        self.label_of(PackagesPage::Repositories),
        label("control_center.repository_add")
      ),
      PackagesPage::RepositoryList => format!(
        "{} > {}",
        self.label_of(PackagesPage::Repositories),
        label("control_center.repository_list")
      ),
      PackagesPage::RepositoryDetails(index) => format!(
        "{} > {}",
        self.label_of(PackagesPage::RepositoryList),
        self
          .repositories
          .get(index)
          .map(|repository| repository.name.as_str())
          .unwrap_or_default()
      ),
    }
  }
}
