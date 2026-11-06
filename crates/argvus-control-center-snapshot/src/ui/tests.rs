use super::*;
use crate::backend::parse_retention_config;
use crossterm::event::KeyModifiers;
use ratatui::{Terminal, backend::TestBackend};
fn app() -> SnapshotApp {
  SnapshotApp::new(Lang::for_locale("en-US"), Theme::load())
}
fn snapshot(number: u32, active: bool) -> Snapshot {
  Snapshot {
    number,
    active,
    default: false,
    kind: "single".into(),
    date: "today".into(),
    description: "manual".into(),
    cleanup: "".into(),
  }
}
fn press(app: &mut SnapshotApp, key: KeyCode) {
  app.handle_key(KeyEvent::new(key, KeyModifiers::NONE));
}

#[test]
fn comparison_selection_rejects_same_snapshot_and_returns_from_diff() {
  let mut app = app();
  app.config = Some("system".into());
  app.state = Some(Integration::Ready {
    config: "system".into(),
    snapshots: vec![snapshot(1, false), snapshot(2, false)],
  });
  press(&mut app, KeyCode::Char('b'));
  assert_eq!(app.origin.as_ref().unwrap().number, 1);
  press(&mut app, KeyCode::Char('v'));
  assert!(app.job.is_none());
  assert!(app.status.is_some());
  press(&mut app, KeyCode::Down);
  app.comparison = Some(Comparison {
    unsupported: 0,
    truncated: false,
    total_seen: 1,
    origin: 1,
    destination: 2,
    changes: vec![ChangedPath {
      path: "/etc/a".into(),
      indicators: "c.....".into(),
      kind: ChangeKind::Modified,
      content: true,
      file_type: false,
      permissions: false,
      owner: false,
      group: false,
      xattrs: false,
      acl: false,
    }],
  });
  app.diff = Some(FileDiff::Text("line 1\nline 2".into()));
  press(&mut app, KeyCode::Down);
  assert_eq!(app.diff_scroll, 1);
  press(&mut app, KeyCode::Esc);
  assert!(app.diff.is_none() && app.comparison.is_some());
  press(&mut app, KeyCode::Esc);
  assert!(app.comparison.is_none() && app.origin.is_some());
}
#[test]
fn unsupported_state_never_offers_a_mutation() {
  let mut app = app();
  app.state = Some(Integration::NonBtrfs("ext4".into()));
  for key in [KeyCode::Char('c'), KeyCode::Char('d'), KeyCode::Enter] {
    press(&mut app, key);
  }
  assert!(app.editor.is_none() && app.confirmation.is_none() && app.job.is_none());
}
#[test]
fn creation_and_deletion_require_explicit_confirmation() {
  let mut app = app();
  app.config = Some("system".into());
  app.state = Some(Integration::Ready {
    config: "system".into(),
    snapshots: vec![snapshot(1, false)],
  });
  press(&mut app, KeyCode::Char('c'));
  assert!(app.editor.is_some());
  press(&mut app, KeyCode::Enter);
  assert!(app.confirmation.is_some());
  assert!(app.job.is_none());
  // Enter starts on Cancel, including after accepting the default description.
  press(&mut app, KeyCode::Enter);
  assert!(app.confirmation.is_none());
  assert!(app.job.is_none());
  press(&mut app, KeyCode::Char('d'));
  assert!(app.confirmation.is_some());
  press(&mut app, KeyCode::Esc);
  assert!(app.confirmation.is_none());
  assert!(app.job.is_none());
}
#[test]
fn active_and_zero_snapshots_cannot_open_delete_confirmation() {
  for snapshot in [snapshot(0, false), snapshot(2, true)] {
    let mut app = app();
    app.state = Some(Integration::Ready {
      config: "system".into(),
      snapshots: vec![snapshot],
    });
    press(&mut app, KeyCode::Char('d'));
    assert!(app.confirmation.is_none());
  }
}
#[test]
fn renders_all_integration_and_error_states_in_both_languages() {
  for locale in ["en-US", "pt-BR"] {
    let mut app = app();
    app.lang = Lang::for_locale(locale);
    for state in [
      Integration::NonBtrfs("ext4".into()),
      Integration::MissingSnapper,
      Integration::NoConfig,
      Integration::ChooseConfig(vec!["system".into(), "root".into()]),
      Integration::Ready {
        config: "system".into(),
        snapshots: vec![],
      },
      Integration::Ready {
        config: "system".into(),
        snapshots: vec![snapshot(1, false)],
      },
    ] {
      app.invalidate_rows();
      app.state = Some(state);
      let mut terminal = Terminal::new(TestBackend::new(110, 28)).unwrap();
      terminal.draw(|frame| app.draw(frame)).unwrap();
      let text = terminal
        .backend()
        .buffer()
        .content
        .iter()
        .map(|cell| cell.symbol())
        .collect::<String>();
      assert!(!text.contains("control_center.snapshot."));
      let expected_title = if locale == "pt-BR" {
        "Instantâneo"
      } else {
        "Snapshot"
      };
      assert!(
        text.contains(expected_title),
        "missing {expected_title} for {locale}"
      );
      assert!(
        text.contains("Experimental"),
        "missing experimental notice for {locale}"
      );
      assert!(!text.contains("ARGVUS Snapshot"));
      assert!(
        app
          .breadcrumb()
          .starts_with(HomeCardId::System.label(app.lang))
      );
    }
    for kind in [
      ErrorKind::Detection,
      ErrorKind::Permission,
      ErrorKind::Authentication,
      ErrorKind::Timeout,
      ErrorKind::Invalid,
      ErrorKind::Snapper,
      ErrorKind::Incompatible,
    ] {
      app.invalidate_rows();
      app.error = Some(SnapshotError::new(kind, "example"));
      assert!(!app.rows().join(" ").contains("control_center.snapshot."));
    }
  }
  assert_eq!(
    label(Lang::for_locale("pt-BR"), "non_btrfs"),
    "Este sistema de arquivos não é Btrfs"
  );
  assert_eq!(
    label(Lang::for_locale("en-US"), "non_btrfs"),
    "This filesystem is not Btrfs"
  );
}

#[test]
fn refresh_preserves_snapshot_number_and_details_escape_is_local() {
  let mut app = app();
  app.state = Some(Integration::Ready {
    config: "system".into(),
    snapshots: vec![snapshot(4, false), snapshot(7, false)],
  });
  app.selection.index = 1;
  press(&mut app, KeyCode::Enter);
  assert!(app.details);
  assert!(!app.handle_key(KeyEvent::from(KeyCode::Esc)));
  assert!(!app.details);
  let result = Integration::Ready {
    config: "system".into(),
    snapshots: vec![snapshot(7, false), snapshot(8, false)],
  };
  app.job = Some(app.jobs.spawn(move |_| Ok(Ok(Reply::Inspected(result)))));
  let deadline = std::time::Instant::now() + Duration::from_secs(2);
  while app.busy() && std::time::Instant::now() < deadline {
    app.poll();
    std::thread::yield_now();
  }
  assert!(!app.busy());
  assert_eq!(app.selected().unwrap().number, 7);
  assert!(app.handle_key(KeyEvent::from(KeyCode::Esc)));
}

#[test]
fn loading_and_operation_phases_render_without_blocking_input() {
  for phase in [Phase::Detecting, Phase::Loading, Phase::Changing] {
    let mut app = app();
    app.phase = phase;
    let mut terminal = Terminal::new(TestBackend::new(80, 20)).unwrap();
    terminal.draw(|frame| app.draw(frame)).unwrap();
    assert!(!app.rows().join(" ").contains("control_center.snapshot."));
  }
}

#[test]
fn unsupported_and_permission_help_never_advertise_mutations() {
  let mut app = app();
  for state in [
    Integration::NonBtrfs("ext4".into()),
    Integration::MissingSnapper,
    Integration::NoConfig,
  ] {
    app.state = Some(state);
    assert!(!app.help_text().contains("create"));
    assert!(!app.help_text().contains("delete"));
  }
  app.error = Some(SnapshotError::new(ErrorKind::Permission, "denied"));
  assert!(app.help_text().contains("Authorize"));
  assert!(!app.help_text().contains("create"));
}

#[test]
fn failed_read_of_replaced_configuration_can_be_discovered_again() {
  let mut app = app();
  app.config = Some("old".into());
  app.phase = Phase::Loading;
  app.job = Some(app.jobs.spawn(|_| {
    Ok(Err(SnapshotError::new(
      ErrorKind::Invalid,
      "configuration changed",
    )))
  }));
  let deadline = std::time::Instant::now() + Duration::from_secs(2);
  while app.busy() && std::time::Instant::now() < deadline {
    app.poll();
    std::thread::yield_now();
  }
  assert!(!app.busy());
  assert!(app.config.is_none());
  assert!(app.error.is_some());
}

#[test]
fn selective_restore_review_requires_a_separate_confirmation_and_cancel_is_read_only() {
  let mut app = app();
  app.config = Some("system".into());
  app.comparison = Some(Comparison {
    unsupported: 0,
    truncated: false,
    total_seen: 1,
    origin: 1,
    destination: 2,
    changes: vec![],
  });
  app.restore_review = Some(RestoreReview {
    source_snapshot: 2,
    path: "/etc/app.conf".into(),
    destination: None,
    diff: FileDiff::Text("+snapshot contents".into()),
  });
  press(&mut app, KeyCode::Enter);
  assert!(app.confirmation.is_some());
  assert!(app.job.is_none());
  press(&mut app, KeyCode::Esc);
  assert!(app.confirmation.is_none());
  assert!(app.job.is_none());
  assert!(app.restore_review.is_some());
}

#[test]
fn selective_restore_review_renders_path_diff_and_limits_in_both_languages() {
  for locale in ["en-US", "pt-BR"] {
    let mut app = app();
    app.lang = Lang::for_locale(locale);
    app.state = Some(Integration::Ready {
      config: "system".into(),
      snapshots: vec![snapshot(2, false)],
    });
    app.restore_review = Some(RestoreReview {
      source_snapshot: 2,
      path: "/etc/app.conf".into(),
      destination: None,
      diff: FileDiff::Text("+snapshot contents".into()),
    });
    let text = app.rows().join("\n");
    assert!(text.contains("/etc/app.conf"));
    assert!(text.contains("snapshot contents"));
    assert!(!text.contains("control_center.snapshot."));
  }
}

#[test]
fn retention_view_opens_editor_and_cleanup_cancel_has_no_job() {
  let mut app = app();
  app.config = Some("system".into());
  app.retention_view = true;
  app.retention = Some(parse_retention_config(br#"{"NUMBER_LIMIT":"50"}"#).unwrap());
  press(&mut app, KeyCode::Char('e'));
  assert!(app.editor.is_some());
  press(&mut app, KeyCode::Esc);
  assert!(app.editor.is_none());
  press(&mut app, KeyCode::Char('c'));
  press(&mut app, KeyCode::Enter);
  assert!(app.confirmation.is_some());
  press(&mut app, KeyCode::Esc);
  assert!(app.confirmation.is_none());
  assert!(app.job.is_none());
}

#[test]
fn truncated_comparison_warning_is_translated_in_both_languages() {
  for locale in ["en-US", "pt-BR"] {
    let mut app = app();
    app.lang = Lang::for_locale(locale);
    app.state = Some(Integration::Ready {
      config: "system".into(),
      snapshots: vec![],
    });
    app.comparison = Some(Comparison {
      unsupported: 0,
      origin: 1,
      destination: 2,
      changes: vec![],
      truncated: true,
      total_seen: 50000,
    });
    let rows = app.rows();
    assert!(rows.iter().any(|row| row.contains("0/50000") && row.contains(label(app.lang, "comparison_truncated"))));
    assert!(
      !error_text(app.lang, &SnapshotError::new(ErrorKind::OutputLimit, ""))
        .contains("control_center.")
    );
  }
}

#[test]
fn restore_review_render_does_not_follow_comparison_selection() {
  let mut app = app();
  app.state = Some(Integration::Ready {
    config: "system".into(),
    snapshots: vec![snapshot(2, false)],
  });
  app.comparison = Some(Comparison {
    unsupported: 0,
    origin: 1,
    destination: 2,
    changes: vec![],
    truncated: false,
    total_seen: 0,
  });
  app.restore_review = Some(RestoreReview {
    source_snapshot: 2,
    path: "/etc/app.conf".into(),
    destination: None,
    diff: FileDiff::Text((0..100).map(|index| format!("+line {index}\n")).collect()),
  });
  let mut terminal = Terminal::new(TestBackend::new(100, 24)).unwrap();
  terminal.draw(|frame| app.draw(frame)).unwrap();
  let initial = terminal.backend().buffer().clone();
  app.change_selection.index = 9000;
  terminal.draw(|frame| app.draw(frame)).unwrap();
  assert_eq!(terminal.backend().buffer(), &initial);
}

#[test]
fn diff_scroll_is_bounded_by_content() {
  let mut app = app();
  app.diff = Some(FileDiff::Text("short\nlast".into()));
  for _ in 0..20 {
    press(&mut app, KeyCode::PageDown);
    press(&mut app, KeyCode::Right);
  }
  assert_eq!(app.diff_scroll, 1);
  assert_eq!(app.diff_horizontal, 4);
}

#[test]
fn unsupported_paths_are_reported_without_action_rows() {
  for locale in ["en-US", "pt-BR"] {
    let mut app = app();
    app.lang = Lang::for_locale(locale);
    app.state = Some(Integration::Ready {
      config: "system".into(),
      snapshots: vec![snapshot(2, false)],
    });
    app.comparison = Some(Comparison {
      unsupported: 3,
      origin: 1,
      destination: 2,
      changes: vec![],
      truncated: false,
      total_seen: 3,
    });
    let text = app.rows().join("\n");
    assert!(text.contains(label(app.lang, "unsupported_paths")));
    assert!(!text.contains("control_center.snapshot."));
    press(&mut app, KeyCode::Enter);
    assert!(app.job.is_none());
  }
}

#[test]
fn lock_contention_messages_are_distinct_and_translated() {
  for locale in ["en-US", "pt-BR"] {
    let lang = Lang::for_locale(locale);
    for (kind, key) in [
      (ErrorKind::PackageUpdateBusy, "package_update_busy"),
      (ErrorKind::RestoreBusy, "restore_busy"),
    ] {
      let error = SnapshotError::new(kind, "technical lock detail");
      let wire: SnapshotResult<()> = Err(error.clone());
      let decoded: SnapshotResult<()> =
        serde_json::from_slice(&serde_json::to_vec(&wire).unwrap()).unwrap();
      assert_eq!(decoded.unwrap_err(), error);
      assert_eq!(error_text(lang, &error), label(lang, key));
      assert!(!error_text(lang, &error).contains("control_center.snapshot."));
    }
    assert_ne!(
      label(lang, "package_update_busy"),
      label(lang, "restore_busy")
    );
  }
}

#[test]
fn absent_retention_settings_are_labeled_as_argvus_fallbacks() {
  for locale in ["en-US", "pt-BR"] {
    let mut app = app();
    app.lang = Lang::for_locale(locale);
    app.state = Some(Integration::Ready {
      config: "system".into(),
      snapshots: vec![],
    });
    app.retention_view = true;
    app.retention = Some(parse_retention_config(b"{}").unwrap());
    let text = app.rows().join("\n");
    assert!(text.contains(label(app.lang, "retention_fallback")));
    assert!(!text.contains("control_center.snapshot."));
  }
}

fn finish_one_reply(app: &mut SnapshotApp) {
  let deadline = std::time::Instant::now() + Duration::from_secs(2);
  while !app.poll() {
    assert!(std::time::Instant::now() < deadline);
    std::thread::yield_now();
  }
}

#[test]
fn s11_config_choice_resets_previous_snapshot_index() {
  let mut app = app();
  app.selection.index = 90;
  app.job = Some(app.jobs.spawn(|_| {
    Ok(Ok(Reply::Inspected(Integration::ChooseConfig(vec![
      "system".into(),
      "root".into(),
    ]))))
  }));
  finish_one_reply(&mut app);
  assert_eq!(app.selection.index, 0);
  press(&mut app, KeyCode::Enter);
  assert_eq!(app.config.as_deref(), Some("system"));
}

#[test]
fn s11_failed_retention_save_invalidates_and_rereads_values() {
  let mut app = app();
  app.config = Some("system".into());
  app.retention_view = true;
  app.retention = Some(parse_retention_config(br#"{"NUMBER_LIMIT":"20"}"#).unwrap());
  app.phase = Phase::Changing;
  app.job = Some(
    app
      .jobs
      .spawn(|_| Ok(Err(SnapshotError::new(ErrorKind::Snapper, "lost reply")))),
  );
  finish_one_reply(&mut app);
  assert!(app.retention.is_none());
  assert!(app.retention_view);
  assert!(matches!(app.phase, Phase::Loading));
  assert!(app.busy());
  app.job = Some(app.jobs.spawn(|_| {
    Ok(Ok(Reply::Retention(
      parse_retention_config(br#"{"NUMBER_LIMIT":"30"}"#).unwrap(),
      SnapperTimers {
        timeline: TimerState::Missing,
        cleanup: TimerState::Missing,
      },
    )))
  }));
  finish_one_reply(&mut app);
  assert_eq!(
    app.retention.as_ref().unwrap().values["NUMBER_LIMIT"].value,
    "30"
  );
}

#[test]
fn s11_unchanged_rows_reuse_the_same_cache() {
  let app = app();
  let first = app.rows();
  let second = app.rows();
  assert_eq!(first.as_ptr(), second.as_ptr());
}

#[test]
fn s11_row_cache_is_invalidated_by_completed_reply() {
  let mut app = app();
  let previous = app.rows();
  assert!(!app.poll());
  assert_eq!(app.rows().as_ptr(), previous.as_ptr());
  app.job = Some(app.jobs.spawn(|_| {
    Ok(Ok(Reply::Inspected(Integration::ChooseConfig(vec![
      "new-config".into(),
    ]))))
  }));
  finish_one_reply(&mut app);
  let current = app.rows();
  assert_ne!(current.as_ptr(), previous.as_ptr());
  assert!(current.join("\n").contains("new-config"));
}

#[test]
fn s11_row_cache_tracks_public_language_changes() {
  let mut app = app();
  let english = app.rows();
  app.lang = Lang::for_locale("pt-BR");
  let portuguese = app.rows();
  assert_ne!(english.as_ptr(), portuguese.as_ptr());
  assert_ne!(*english, *portuguese);
}
