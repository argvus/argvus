use super::*;
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
