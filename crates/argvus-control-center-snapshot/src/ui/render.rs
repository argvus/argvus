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
      if self.retention_view {
        return label(self.lang, "retention_controls");
      }
      if self.diff.is_some() {
        return label(self.lang, "diff_help");
      }
      if self.restore_review.is_some() {
        return label(self.lang, "restore_review_help");
      }
      if self.comparison.is_some() {
        return label(self.lang, "comparison_help");
      }
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
      && self.diff.is_none()
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
        if self.comparison.is_some() {
          self
            .change_selection
            .index
            .saturating_add(3)
            .min(rows.len().saturating_sub(1))
        } else if self.retention_view {
          self
            .retention_selection
            .index
            .saturating_add(1)
            .min(rows.len().saturating_sub(1))
        } else {
          self.selection.index.min(rows.len().saturating_sub(1))
        },
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
        PendingAction::Retention { key, old, new } => (
          format!(
            "{}\n{key}: {old} → {new}",
            label(self.lang, "confirm_retention")
          ),
          true,
        ),
        PendingAction::Cleanup(algorithm) => {
          let count = match &self.state {
            Some(Integration::Ready { snapshots, .. }) => snapshots.len(),
            _ => 0,
          };
          (
            format!(
              "{}\n{} · {}: {}\n{}",
              label(self.lang, "confirm_cleanup"),
              label(self.lang, "cleanup_algorithm"),
              algorithm.as_snapper_arg(),
              count,
              label(self.lang, "cleanup_no_preview")
            ),
            true,
          )
        }
        PendingAction::Restore(review) => (
          format!(
            "{}\n{} #{} · {}\n{}\n{}",
            label(self.lang, "restore_confirm"),
            label(self.lang, "restore_source"),
            review.source_snapshot,
            review.path,
            label(self.lang, "restore_replacement_warning"),
            label(self.lang, "restore_safety_note")
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
            Phase::Comparing => "comparing",
            Phase::Diffing => "diffing",
            Phase::PreviewRestore => "restore_loading",
            Phase::Restoring => "restore_running",
          },
        )
        .into(),
      ];
    }
    if let Some(error) = &self.error {
      return vec![error_text(self.lang, error)];
    }
    if self.retention_view {
      let Some(config) = &self.retention else {
        return vec![label(self.lang, "loading").into()];
      };
      let mut rows = vec![label(self.lang, "retention_title").into()];
      rows.extend(config.values.iter().map(|(key, value)| {
        format!(
          "{key}: {}{}",
          value.value,
          if value.origin == SettingOrigin::SnapperDefault {
            format!(" ({})", label(self.lang, "snapper_default"))
          } else {
            String::new()
          }
        )
      }));
      rows.extend(
        config
          .unknown
          .iter()
          .map(|(key, value)| format!("{key}: {value} ({})", label(self.lang, "unknown_setting"))),
      );
      rows.push(format!(
        "snapper-timeline.timer: {}",
        self.timers.as_ref().map_or_else(
          || label(self.lang, "timer_unknown").into(),
          |timers| timer_label(self.lang, &timers.timeline)
        )
      ));
      rows.push(format!(
        "snapper-cleanup.timer: {}",
        self.timers.as_ref().map_or_else(
          || label(self.lang, "timer_unknown").into(),
          |timers| timer_label(self.lang, &timers.cleanup)
        )
      ));
      rows.push(label(self.lang, "retention_timer_note").into());
      rows.push(label(self.lang, "retention_controls").into());
      rows.push(format!(
        "{} {}",
        label(self.lang, "cleanup_algorithm"),
        ["number", "timeline", "empty-pre-post"][self.cleanup_selection]
      ));
      return rows;
    }
    if let Some(review) = &self.restore_review {
      let source_details = match &self.state {
        Some(Integration::Ready { snapshots, .. }) => snapshots
          .iter()
          .find(|snapshot| snapshot.number == review.source_snapshot)
          .map(|snapshot| {
            format!(
              "#{} · {} · {}",
              snapshot.number, snapshot.date, snapshot.description
            )
          })
          .unwrap_or_else(|| format!("#{}", review.source_snapshot)),
        _ => format!("#{}", review.source_snapshot),
      };
      let mut rows = vec![
        label(self.lang, "restore_review_title").into(),
        format!("{}: {source_details}", label(self.lang, "restore_source")),
        format!("{}: {}", label(self.lang, "restore_path"), review.path),
        label(
          self.lang,
          if review.destination.is_some() {
            "restore_destination_exists"
          } else {
            "restore_destination_missing"
          },
        )
        .into(),
        label(self.lang, "restore_regular_file_only").into(),
        label(self.lang, "restore_metadata_note").into(),
        label(self.lang, "restore_safety_note").into(),
        String::new(),
      ];
      match &review.diff {
        FileDiff::Text(text) => rows.extend(
          text
            .lines()
            .skip(self.diff_scroll)
            .take(512)
            .map(|line| line.chars().skip(self.diff_horizontal).collect()),
        ),
        FileDiff::Binary => rows.push(label(self.lang, "binary").into()),
        FileDiff::Truncated => rows.push(label(self.lang, "truncated").into()),
      }
      return rows;
    }
    if let Some(diff) = &self.diff {
      return match diff {
        FileDiff::Text(text) => text
          .lines()
          .skip(self.diff_scroll)
          .map(|line| line.chars().skip(self.diff_horizontal).collect())
          .collect(),
        FileDiff::Binary => vec![label(self.lang, "binary").into()],
        FileDiff::Truncated => vec![label(self.lang, "truncated").into()],
      };
    }
    if let Some(comparison) = &self.comparison {
      let snapshots = match &self.state {
        Some(Integration::Ready { snapshots, .. }) => snapshots.as_slice(),
        _ => &[],
      };
      let describe = |number| {
        snapshots
          .iter()
          .find(|snapshot| snapshot.number == number)
          .map(|snapshot| format!("#{number} · {} · {}", snapshot.date, snapshot.description))
          .unwrap_or_else(|| format!("#{number}"))
      };
      let mut rows = vec![
        format!(
          "{} {} → {} {}",
          label(self.lang, "origin"),
          describe(comparison.origin),
          label(self.lang, "destination"),
          describe(comparison.destination)
        ),
        format!(
          "{} {} · {} {} · {} {}",
          comparison
            .changes
            .iter()
            .filter(|change| change.kind == ChangeKind::Created)
            .count(),
          label(self.lang, "created_kind"),
          comparison
            .changes
            .iter()
            .filter(|change| change.kind == ChangeKind::Removed)
            .count(),
          label(self.lang, "removed_kind"),
          comparison
            .changes
            .iter()
            .filter(|change| change.kind == ChangeKind::Modified)
            .count(),
          label(self.lang, "modified_kind")
        ),
        String::new(),
      ];
      if comparison.changes.is_empty() {
        rows.push(label(self.lang, "no_changes").into());
      }
      rows.extend(
        comparison
          .changes
          .iter()
          .map(|change| format!("{} {}", change.indicators, change.path)),
      );
      return rows;
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
        let base = self.origin.as_ref().map(|snapshot| {
          format!(
            "{} #{} · {} · {}",
            label(self.lang, "origin"),
            snapshot.number,
            snapshot.date,
            snapshot.description
          )
        });
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
        let mut rows: Vec<String> = snapshots
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
          .collect();
        rows.push(label(self.lang, "retention_shortcut").into());
        if let Some(base) = base {
          rows.push(base);
        }
        rows
      }
      None => vec![],
    }
  }
}

fn timer_label(lang: Lang, state: &TimerState) -> String {
  match state {
    TimerState::Missing => label(lang, "timer_missing").into(),
    TimerState::Disabled => label(lang, "timer_disabled").into(),
    TimerState::EnabledInactive => label(lang, "timer_inactive").into(),
    TimerState::Active => label(lang, "timer_active").into(),
    TimerState::Unavailable(_) => label(lang, "timer_unavailable").into(),
  }
}
