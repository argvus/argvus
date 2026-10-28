use super::*;
use argvus_tui::menu::RowKind;
#[test]
fn failed_filter_is_explicit() {
  let mut a = ServicesApp::new(Lang::for_locale("en-US"), Theme::load());
  a.page = ServicePage::Failed;
  a.units = vec![
    Unit {
      name: "a.service".into(),
      active: "failed".into(),
      ..Default::default()
    },
    Unit {
      name: "b.service".into(),
      active: "active".into(),
      ..Default::default()
    },
  ];
  assert_eq!(a.filtered().len(), 1);
}

#[test]
fn actions_are_structured_and_user_actions_are_not_elevated() {
  assert_eq!(
    backend::action_args("enable-now", "NetworkManager.service").unwrap(),
    ["enable", "--now", "NetworkManager.service"]
  );
  assert!(backend::action_args("restart", "bad unit.service").is_err());
}

#[test]
fn service_list_filter_and_selection_stay_bounded() {
  let mut app = ServicesApp::new(Lang::for_locale("en-US"), Theme::load());
  app.page = ServicePage::System;
  app.units = vec![Unit {
    name: "demo.service".into(),
    description: "Demo".into(),
    active: "active".into(),
    file_state: "enabled".into(),
    ..Default::default()
  }];
  app.handle(KeyCode::Down);
  assert_eq!(app.selected, 0);
  app.handle(KeyCode::Char('f'));
  assert_eq!(app.filter, UnitFilter::Running);
  assert_eq!(app.filtered().len(), 1);
  app.handle(KeyCode::Char('/'));
  app.handle(KeyCode::Char('z'));
  assert!(app.filtered().is_empty());
  assert_eq!(app.selected, 0);
}

#[test]
fn service_details_expose_actions_and_confirmation_defaults_to_cancel() {
  let mut app = ServicesApp::new(Lang::for_locale("en-US"), Theme::load());
  app.page = ServicePage::Detail;
  app.detail_parent = ServicePage::System;
  app.detail_unit = Some("demo.service".into());
  app.units = vec![Unit {
    name: "demo.service".into(),
    active: "active".into(),
    file_state: "enabled".into(),
    ..Default::default()
  }];
  assert!(
    app
      .detail_actions()
      .iter()
      .any(|(action, _)| *action == "restart")
  );
  app.request("restart");
  app.handle(KeyCode::Enter);
  assert!(app.pending.is_none());
  assert!(app.action.is_none());
  app.request("restart");
  app.handle(KeyCode::Tab);
  assert!(app.confirmation.confirm_selected);
}

#[test]
fn polkit_exit_codes_are_not_collapsed_into_a_generic_error() {
  let output = argvus_control_center_core::process::ProcessOutput {
    stdout: Vec::new(),
    stderr: Vec::new(),
    status: Some(126),
    timed_out: false,
  };
  assert!(privileged_failure(&output).contains("cancelled"));
}

#[test]
/// The Logs filters are keys, not buttons: the filter line shows them.
fn logs_filter_summary_names_boot_service_and_priority() {
  let mut logs = ServicesApp::new(Lang::for_locale("en-US"), Theme::load());
  logs.page = ServicePage::Logs;
  assert_eq!(
    logs.logs_filter_summary(),
    "Boot: Current · Service: — · Priority: 0-7"
  );
  logs.previous_boot = true;
  logs.logs_unit = Some("sshd.service".into());
  logs.priority = Some("3".into());
  assert_eq!(
    logs.logs_filter_summary(),
    "Boot: Previous · Service: sshd.service · Priority: 3"
  );
}

#[test]
/// The unit list shows the filter line, the search box and the `Services:`
/// block, without the old `Name:` header.
fn services_list_uses_filter_search_and_block_layout() {
  let mut app = ServicesApp::new(Lang::for_locale("en-US"), Theme::load());
  app.page = ServicePage::System;
  app.units = vec![Unit {
    name: "alsa-restore.service".into(),
    active: "active".into(),
    file_state: "enabled".into(),
    ..Default::default()
  }];
  let text = render_text(&mut app, 90, 25);
  assert!(text.contains("Filter:") && text.contains("All"));
  assert!(text.contains("Search:"));
  assert!(text.contains("Services:"));
  assert!(text.contains("alsa-restore.service"));
  assert!(text.contains("Active") && text.contains("Enabled"));
  assert!(!text.contains("Name:"));
  assert!(!text.contains("Start") && !text.contains("Iniciar"));
}

#[test]
/// The Logs page uses the same layout, with the `Logs:` block as its list.
fn logs_page_uses_filter_search_and_block_layout() {
  let mut app = ServicesApp::new(Lang::for_locale("en-US"), Theme::load());
  app.page = ServicePage::Logs;
  let text = render_text(&mut app, 90, 25);
  assert!(text.contains("Filter:") && text.contains("Priority: 0-7"));
  assert!(text.contains("Search:"));
  assert!(text.contains("Logs:"));
}

/// Draws `app` into a test terminal of `width` x `height` and returns the
/// rendered cells as text.
fn render_text(app: &mut ServicesApp, width: u16, height: u16) -> String {
  let mut terminal =
    ratatui::Terminal::new(ratatui::backend::TestBackend::new(width, height)).unwrap();
  terminal.draw(|frame| app.draw(frame)).unwrap();
  terminal
    .backend()
    .buffer()
    .content
    .iter()
    .map(|cell| cell.symbol())
    .collect()
}

#[test]
fn services_home_rows_act_as_a_status_dashboard() {
  let mut app = ServicesApp::new(Lang::for_locale("en-US"), Theme::load());
  app.units = vec![
    Unit {
      name: "a.service".into(),
      active: "active".into(),
      file_state: "enabled".into(),
      scope: "system".into(),
      ..Default::default()
    },
    Unit {
      name: "b.service".into(),
      active: "inactive".into(),
      file_state: "disabled".into(),
      scope: "system".into(),
      ..Default::default()
    },
    Unit {
      name: "c.service".into(),
      active: "failed".into(),
      file_state: "enabled".into(),
      scope: "user".into(),
      ..Default::default()
    },
  ];
  let rows = app.home_rows();
  assert_eq!(rows.len(), 4);
  assert!(rows[0].contains("System") && rows[0].contains("1 active"));
  assert!(rows[1].contains("User") && rows[1].contains("0 active"));
  assert!(rows[2].contains("Failed") && rows[2].contains("1 failed"));
  assert!(rows[3].contains("Logs"));
}

#[test]
fn services_list_rows_render_status_columns() {
  let mut app = ServicesApp::new(Lang::for_locale("en-US"), Theme::load());
  app.page = ServicePage::System;
  app.units = vec![
    Unit {
      name: "NetworkManager.service".into(),
      active: "active".into(),
      file_state: "enabled".into(),
      ..Default::default()
    },
    Unit {
      name: "cups.service".into(),
      active: "inactive".into(),
      file_state: "disabled".into(),
      ..Default::default()
    },
    Unit {
      name: "sshd.service".into(),
      active: "failed".into(),
      file_state: "enabled".into(),
      ..Default::default()
    },
  ];
  let rows = app.list_rows();
  // One row per unit: the filter is cycled with `f`, not listed.
  assert_eq!(rows.len(), 3);
  assert_eq!(rows[0].label(), "NetworkManager.service");
  assert_eq!(rows[0].detail_text(), Some("● Active      ● Enabled"));
  assert_eq!(rows[1].detail_text(), Some("○ Active      ○ Enabled"));
  assert_eq!(rows[2].detail_text(), Some("○ Active      ● Enabled"));
}

#[test]
fn services_detail_rows_show_info_then_actions() {
  let mut app = ServicesApp::new(Lang::for_locale("en-US"), Theme::load());
  app.page = ServicePage::Detail;
  app.detail_parent = ServicePage::System;
  app.detail_unit = Some("demo.service".into());
  app.units = vec![Unit {
    name: "demo.service".into(),
    description: "Demo service".into(),
    active: "active".into(),
    sub: "running".into(),
    file_state: "enabled".into(),
    main_pid: Some(1234),
    fragment: Some("/usr/lib/systemd/system/demo.service".into()),
    ..Default::default()
  }];
  let rows = app.detail_rows();
  let text = rows
    .iter()
    .map(|row| format!("{} {}", row.label(), row.detail_text().unwrap_or_default()))
    .collect::<Vec<_>>()
    .join("\n");
  assert!(text.contains("demo.service"));
  assert!(text.contains("1234"));
  assert!(text.contains("Enabled"));
  assert!(text.contains("Unit file"));
  // Info rows are read-only; the actions follow them as menu rows.
  assert!(rows.iter().any(|row| row.is_section()));
  assert!(rows.iter().any(|row| row.kind() == RowKind::Action));
}

#[test]
/// Leaving a unit list clears the filter, so each visit starts on All.
fn leaving_a_list_clears_the_filter() {
  let mut app = ServicesApp::new(Lang::for_locale("en-US"), Theme::load());
  app.page = ServicePage::Failed;
  app.filter = UnitFilter::Enabled;
  app.handle(KeyCode::Esc);
  assert_eq!(app.page, ServicePage::Home);
  assert_eq!(app.filter, UnitFilter::All);
}

#[test]
fn loading_status_is_cleared_once_units_arrive() {
  let mut app = ServicesApp::new(Lang::for_locale("en-US"), Theme::load());
  app.page = ServicePage::System;
  app.status = Some(StatusMessage {
    kind: StatusKind::Info,
    text: tr(app.lang, "control_center.loading_services").into(),
  });
  app.job = Some(
    app
      .manager
      .spawn(|_| Ok(JobData::Units(vec![Unit::default()]))),
  );
  for _ in 0..2000 {
    if app.job.is_none() {
      break;
    }
    app.poll();
    std::thread::sleep(std::time::Duration::from_millis(1));
  }
  assert!(app.job.is_none());
  assert!(app.status.is_none());
  assert_eq!(app.units.len(), 1);
}

#[test]
fn arrows_are_blocked_while_services_are_loading() {
  let mut app = ServicesApp::new(Lang::for_locale("en-US"), Theme::load());
  app.page = ServicePage::System;
  app.selected = 3;
  app.job = Some(app.manager.spawn(|_| {
    std::thread::sleep(std::time::Duration::from_millis(150));
    Ok(JobData::Units(vec![]))
  }));
  app.handle(KeyCode::Down);
  app.handle(KeyCode::Up);
  app.handle(KeyCode::Home);
  app.handle(KeyCode::Char('f'));
  assert_eq!(app.selected, 3);
  assert_eq!(app.filter, UnitFilter::All);
}
