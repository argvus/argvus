//! Snapshot lists, state messages and shared overlays. Rendering never performs I/O.
use super::*;

impl SnapshotApp {
  /// Available keys follow the detected integration, including in the global help overlay.
  pub fn help_text(&self) -> &'static str {
    if self.busy() {
      label(self.lang, "busy_help")
    } else if self
      .error
      .as_ref()
      .is_some_and(|error| error.kind == ErrorKind::Permission)
    {
      label(self.lang, "authorize_help")
    } else if self.error.is_none() && matches!(self.state, Some(Integration::Ready { .. })) {
      label(self.lang, "help")
    } else {
      label(self.lang, "refresh_help")
    }
  }

  pub fn draw(&mut self, frame: &mut Frame) {
    let hint = self.help_text();
    let body = shell(
      frame,
      frame.area(),
      &self.theme,
      self.lang,
      &self.breadcrumb(),
      hint,
    );
    let body = argvus_tui::page::admin_warning(frame, body, &self.theme, self.lang, true);
    let content = Rect::new(body.x, body.y, body.width, body.height.saturating_sub(2));
    let rows = self.rows();
    if !self.busy()
      && self.error.is_none()
      && !self.details
      && matches!(
        self.state,
        Some(Integration::Ready { .. } | Integration::ChooseConfig(_))
      )
      && !rows.is_empty()
      && !matches!(&self.state, Some(Integration::Ready { snapshots, .. }) if snapshots.is_empty())
    {
      list(
        frame,
        content,
        &self.theme,
        &rows,
        self.selection.index.min(rows.len().saturating_sub(1)),
      );
    } else {
      readonly(
        frame,
        content,
        &self.theme,
        &rows.into_iter().map(Line::from).collect::<Vec<_>>(),
      );
    }
    if let Some(status) = &self.status {
      draw_status(
        frame,
        Rect::new(body.x, body.bottom().saturating_sub(1), body.width, 1),
        &self.theme,
        status,
      );
    }
    if let Some(editor) = &self.editor {
      editor.draw(
        frame,
        frame.area(),
        &self.theme,
        label(self.lang, "editor_help"),
      );
    }
    if let Some((confirmation, action)) = &self.confirmation {
      let (message, danger) = match action {
        PendingAction::Create(description) => (
          format!("{}\n{description}", label(self.lang, "confirm_create")),
          false,
        ),
        PendingAction::Delete(snapshot) => (
          format!(
            "{}\n#{} — {}",
            label(self.lang, "confirm_delete"),
            snapshot.number,
            snapshot.description
          ),
          true,
        ),
      };
      draw_confirm(
        frame,
        frame.area(),
        &self.theme,
        ConfirmDialog {
          title: label(self.lang, "title"),
          message: &message,
          confirm: tr(self.lang, "control_center.confirm"),
          cancel: tr(self.lang, "control_center.cancel"),
          danger,
          deadline: None,
        },
        confirmation,
      );
    }
  }
  pub(super) fn rows(&self) -> Vec<String> {
    if self.busy() || (self.state.is_none() && self.error.is_none()) {
      return vec![
        label(
          self.lang,
          match self.phase {
            Phase::Detecting => "detecting",
            Phase::Loading => "loading",
            Phase::Changing => "changing",
          },
        )
        .into(),
      ];
    }
    if let Some(error) = &self.error {
      return vec![error_text(self.lang, error)];
    }
    match &self.state {
      Some(Integration::NonBtrfs(filesystem)) => vec![
        label(self.lang, "non_btrfs").into(),
        format!("/: {filesystem}"),
      ],
      Some(Integration::MissingSnapper) => vec![label(self.lang, "missing").into()],
      Some(Integration::NoConfig) => vec![label(self.lang, "no_config").into()],
      Some(Integration::ChooseConfig(configs)) => configs
        .iter()
        .map(|config| format!("{}: {config}", label(self.lang, "choose_config")))
        .collect(),
      Some(Integration::Ready { snapshots, config }) => {
        if self.details
          && let Some(snapshot) = self.selected()
        {
          return vec![
            format!("{}: {config}", label(self.lang, "config")),
            format!(
              "#{} · {} · {}",
              snapshot.number, snapshot.kind, snapshot.date
            ),
            snapshot.description.clone(),
            format!("{}: {}", label(self.lang, "cleanup"), snapshot.cleanup),
            format!(
              "{}: {} · {}: {}",
              label(self.lang, "active"),
              tr(
                self.lang,
                if snapshot.active {
                  "control_center.yes"
                } else {
                  "control_center.no"
                }
              ),
              label(self.lang, "default"),
              tr(
                self.lang,
                if snapshot.default {
                  "control_center.yes"
                } else {
                  "control_center.no"
                }
              )
            ),
          ];
        }
        if snapshots.is_empty() {
          return vec![label(self.lang, "empty").into()];
        }
        snapshots
          .iter()
          .map(|snapshot| {
            format!(
              "#{} {} · {} · {} · {}{}",
              snapshot.number,
              snapshot.kind,
              snapshot.date,
              snapshot.description,
              snapshot.cleanup,
              if snapshot.deletable() {
                ""
              } else {
                label(self.lang, "protected")
              }
            )
          })
          .collect()
      }
      None => vec![],
    }
  }
}
