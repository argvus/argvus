//! Setting pickers and typed prompts: current values, custom values and validation.

use super::*;

#[test]
fn prompt_validation_rejects_bad_positions() {
  assert_eq!(parse_position("1920,0"), Some((1920, 0)));
  assert_eq!(parse_position("0x0"), None);
  assert_eq!(parse_position("abc"), None);
  assert_eq!(parse_number("1.5"), Some(1.5));
  assert_eq!(parse_number("x"), None);
}

#[test]
fn workspace_binding_persist_moves_workspace() {
  let mut config = PersistedConfig::default();
  config.set_workspaces("eDP-1", vec![1, 2]);
  config.set_workspaces("DP-1", vec![3]);
  DisplaysApp::apply_persist(
    &mut config,
    "DP-1",
    &PersistChange::Workspace(2, Some("DP-1".into())),
  );
  assert_eq!(config.workspaces_of("eDP-1"), vec![1]);
  assert_eq!(config.workspaces_of("DP-1"), vec![2, 3]);
  DisplaysApp::apply_persist(&mut config, "DP-1", &PersistChange::Workspace(2, None));
  assert_eq!(config.workspaces_of("DP-1"), vec![3]);
}

#[test]
fn primary_option_rows_list_every_connected_monitor() {
  let mut app = app_with(false);
  app.monitors.push(Monitor {
    name: "DP-1".into(),
    x: 1920,
    y: 0,
    ..monitor("DP-1")
  });
  let options = app.primary_options();
  assert_eq!(options.len(), 2);
  assert!(
    options
      .iter()
      .all(|option| option.persist == PersistChange::Primary)
  );
  app.config.primary_monitor = Some("DP-1".into());
  let rows = app.picker_rows(0, MonitorSetting::Primary);
  assert_eq!(rows[1].kind(), RowKind::Choice { current: true });
  assert_eq!(rows[1].label(), "DP-1", "no text suffix: ● marks it");
}

#[test]
fn workspace_editor_marks_this_monitor_and_others() {
  let mut app = app_with(false);
  app.monitors.push(Monitor {
    name: "DP-1".into(),
    x: 1920,
    y: 0,
    ..monitor("DP-1")
  });
  app.config.set_workspaces("eDP-1", vec![1, 2]);
  app.config.set_workspaces("DP-1", vec![3]);
  let options = app.workspace_options(0);
  assert!(options[0].label.contains("this monitor"));
  assert!(options[2].label.contains("another monitor"));
  assert!(options[4].label.contains("unbound"));
  let rows = app.picker_rows(0, MonitorSetting::Workspaces);
  assert_eq!(rows[0].kind(), RowKind::Toggle { on: true });
  assert_eq!(rows[0].label(), "1");
  assert!(rows[0].detail_text().unwrap().contains("this monitor"));
  assert_eq!(rows[2].kind(), RowKind::Toggle { on: false });
}

#[test]
fn picker_marks_the_current_value_and_offers_a_custom_value_row() {
  let mut app = app_with(false);
  app.page = DisplayPage::Picker {
    monitor: 0,
    setting: MonitorSetting::Scale,
  };
  let rows = app.rows();
  let current: Vec<&str> = rows
    .iter()
    .filter(|row| row.kind() == RowKind::Choice { current: true })
    .map(|row| row.label())
    .collect();
  assert_eq!(current, vec!["1"]);
  let custom = rows.last().unwrap();
  assert_eq!(custom.id(), Some(&Item::Custom));
  assert_eq!(custom.kind(), RowKind::Value { step: None });
  assert!(!custom.label().contains('✎'));

  let resolution = app.picker_rows(0, MonitorSetting::Resolution);
  assert_eq!(
    resolution[0].kind(),
    RowKind::Choice { current: true },
    "1920x1080 is current"
  );
  assert_eq!(resolution[1].kind(), RowKind::Choice { current: false });

  let dpms = app.picker_rows(0, MonitorSetting::Dpms);
  assert_eq!(dpms[0].kind(), RowKind::Choice { current: true });
  assert!(dpms.iter().all(|row| !row.label().contains("current")));
}

#[test]
fn unsupported_settings_show_an_info_row() {
  let mut app = app_with(false);
  app.version = (0, 30);
  let rows = app.picker_rows(0, MonitorSetting::Vrr);
  assert_eq!(rows.len(), 1);
  assert_eq!(rows[0].kind(), RowKind::Info);
}

#[test]
fn app_navigates_home_to_detail_picker_and_back() {
  let mut app = app_with(false);
  app.handle(KeyCode::Enter);
  assert!(matches!(app.page, DisplayPage::Detail(0)));
  app.handle(KeyCode::Enter);
  assert!(matches!(
    app.page,
    DisplayPage::Picker {
      setting: MonitorSetting::Resolution,
      ..
    }
  ));
  app.handle(KeyCode::Char('r'));
  assert!(
    matches!(app.page, DisplayPage::Detail(0)),
    "r cancels the picker"
  );
  app.handle(KeyCode::Enter);
  app.handle(KeyCode::Esc);
  assert!(matches!(app.page, DisplayPage::Detail(0)));
  app.handle(KeyCode::Esc);
  assert_eq!(app.page, DisplayPage::Home);
  assert!(app.handle(KeyCode::Esc), "Esc on Home leaves Displays");
}
