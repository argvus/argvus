use super::*;
use crate::model::Mode;
use argvus_tui::menu::RowKind;

/// Executes the `monitor` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn monitor(name: &str) -> Monitor {
  Monitor {
    id: 0,
    name: name.into(),
    x: 0,
    y: 0,
    width: 1920,
    height: 1080,
    refresh_rate: 60.0,
    scale: 1.0,
    transform: 0,
    focused: false,
    vrr: 0,
    dpms_status: "on".into(),
    disabled: false,
    modes: vec![
      Mode {
        id: 1,
        width: 1920,
        height: 1080,
        refresh_rate: 60.0,
        bit_depth: 8,
      },
      Mode {
        id: 2,
        width: 1280,
        height: 720,
        refresh_rate: 59.94,
        bit_depth: 10,
      },
    ],
    connected: true,
    info: crate::model::MonitorInfo::default(),
    active_workspace: Some(1),
  }
}

/// Executes the `app_with` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn app_with(job: bool) -> DisplaysApp {
  let mut app = DisplaysApp::new(Lang::for_locale("en-US"), Theme::load());
  if !job {
    app.job = None;
    app.action = None;
    app.hotplug = None;
  }
  app.monitors = vec![monitor("eDP-1")];
  app.status = None;
  app
}

fn ids(app: &DisplaysApp) -> Vec<Option<Item>> {
  app.rows().iter().map(|row| row.id().copied()).collect()
}

fn selected(app: &DisplaysApp) -> Option<Item> {
  let rows = app.rows();
  let mut menu = app.menu;
  menu.normalize(&rows);
  menu.selected_id(&rows)
}

fn select(app: &mut DisplaysApp, item: Item) {
  let rows = app.rows();
  assert!(app.menu.select(&rows, &item), "{item:?} is not selectable");
}

fn profile(name: &str) -> MonitorProfile {
  MonitorProfile {
    name: name.into(),
    apply_wallpapers: false,
    config: PersistedConfig::default(),
  }
}

#[test]
/// Executes the `versions_gate_fancy_features` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn versions_gate_fancy_features() {
  let mut app = app_with(true);
  assert!(!app.vrr_supported() && !app.hdr_supported());
  app.version = (0, 42);
  assert!(app.vrr_supported() && app.hdr_supported());
}

#[test]
fn home_lists_monitors_profiles_and_refresh_as_rows() {
  let mut app = app_with(false);
  app.state.profiles.push(profile("Trabalho"));
  let rows = app.rows();
  assert_eq!(
    ids(&app),
    vec![
      Some(Item::Monitor(0)),
      Some(Item::Profiles),
      None,
      Some(Item::Refresh)
    ]
  );
  assert_eq!(rows[0].label(), "eDP-1");
  assert_eq!(rows[0].icon_glyph(), Some(icons::MONITOR));
  assert!(!rows[0].detail_text().unwrap().contains("Trabalho"));
  assert!(
    !rows[0].detail_text().unwrap().contains("escala"),
    "no fixed Portuguese text"
  );
  assert_eq!(rows[1].detail_text(), Some("1"));
}

#[test]
fn profiles_row_is_visible_without_profiles() {
  let app = app_with(false);
  assert!(app.state.profiles.is_empty());
  assert!(ids(&app).contains(&Some(Item::Profiles)));
}

#[test]
fn each_saved_configuration_opens_its_own_detail() {
  let mut app = app_with(false);
  app
    .config
    .set_monitor("HDMI-A-1", PersistedMonitor::default());
  app.config.set_monitor("DP-2", PersistedMonitor::default());
  let names = app.stale_names();
  assert_eq!(names.len(), 2);
  select(&mut app, Item::Stale(1));
  app.handle(KeyCode::Enter);
  assert_eq!(app.page, DisplayPage::Detail(2));
  assert_eq!(app.stale_name(2).as_deref(), Some(names[1].as_str()));
  assert_eq!(
    ids(&app),
    vec![None, None, Some(Item::RemoveConfig)],
    "Info, Danger zone, Remove configuration"
  );
  assert_eq!(app.rows()[2].kind(), RowKind::Destructive);
}

#[test]
/// Executes the `risky_persist_changes_arm_revert` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn risky_persist_changes_arm_revert() {
  assert!(option_is_risky(&PersistChange::Mode("x".into())));
  assert!(option_is_risky(&PersistChange::Position(1, 2)));
  assert!(option_is_risky(&PersistChange::Disabled(true)));
  assert!(option_is_risky(&PersistChange::Mirror(String::new())));
  assert!(option_is_risky(&PersistChange::BitDepth(10)));
  assert!(!option_is_risky(&PersistChange::Scale(1.0)));
  assert!(!option_is_risky(&PersistChange::Vrr(1)));
  assert!(!option_is_risky(&PersistChange::Hdr(1)));
  assert!(!option_is_risky(&PersistChange::Workspace(3, None)));
}

#[test]
/// Executes the `prompt_validation_rejects_bad_positions` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn prompt_validation_rejects_bad_positions() {
  assert_eq!(parse_position("1920,0"), Some((1920, 0)));
  assert_eq!(parse_position("0x0"), None);
  assert_eq!(parse_position("abc"), None);
  assert_eq!(parse_number("1.5"), Some(1.5));
  assert_eq!(parse_number("x"), None);
}

#[test]
/// Executes the `workspace_binding_persist_moves_workspace` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
/// Executes the `profile_rule_replays_persisted_fields` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn profile_rule_replays_persisted_fields() {
  let persisted = PersistedMonitor {
    mode: Some("1920x1080@144".into()),
    position: Some("0x0".into()),
    scale: Some(1.25),
    transform: Some(1),
    vrr: Some(1),
    hdr: Some(1),
    bitdepth: Some(10),
    disabled: Some(false),
    ..Default::default()
  };
  let rule = rule_from_persisted("eDP-1", &persisted);
  assert!(
    rule.starts_with("eDP-1, 1920x1080@144, 0x0, 1.25, transform, 1"),
    "{rule}"
  );
  assert!(rule.contains("vrr, 1"), "{rule}");
  assert!(rule.contains("bitdepth, 10"), "{rule}");
  assert!(rule.contains("supports_hdr, 1"), "{rule}");
  let disabled = PersistedMonitor {
    disabled: Some(true),
    ..Default::default()
  };
  assert_eq!(rule_from_persisted("eDP-1", &disabled), "eDP-1, disabled");
}

#[test]
/// Executes the `detail_settings_include_supported_and_gated` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn detail_settings_include_supported_and_gated() {
  let mut app = app_with(false);
  app.version = (0, 42);
  app.page = DisplayPage::Detail(0);
  let settings = app.detail_settings();
  assert!(settings.contains(&MonitorSetting::Workspaces));
  assert!(settings.contains(&MonitorSetting::SdrBrightness));
  assert!(settings.contains(&MonitorSetting::SdrSaturation));
  assert!(settings.contains(&MonitorSetting::Enabled));
  assert!(settings.contains(&MonitorSetting::Hdr));
  app.version = (0, 30);
  let settings = app.detail_settings();
  assert!(!settings.contains(&MonitorSetting::Vrr));
  assert!(!settings.contains(&MonitorSetting::Hdr));
  assert!(!settings.contains(&MonitorSetting::SdrBrightness));
}

#[test]
fn detail_has_settings_section_and_actions_without_buttons() {
  let mut app = app_with(false);
  app.page = DisplayPage::Detail(0);
  let rows = app.rows();
  let settings: Vec<&Row<Item>> = rows
    .iter()
    .filter(|row| matches!(row.id(), Some(Item::Setting(_))))
    .collect();
  assert_eq!(settings.len(), app.detail_settings().len());
  assert!(settings.iter().all(|row| row.kind() == RowKind::Submenu));
  let mut icons_seen = std::collections::HashSet::new();
  assert!(
    settings
      .iter()
      .all(|row| icons_seen.insert(row.icon_glyph())),
    "sibling settings have distinct icons"
  );
  let actions: Vec<Option<Item>> = rows
    .iter()
    .rev()
    .take(2)
    .map(|row| row.id().copied())
    .collect();
  assert_eq!(actions, vec![Some(Item::Reset), Some(Item::Apply)]);
  assert_eq!(
    selected(&app),
    Some(Item::Setting(MonitorSetting::Resolution))
  );
}

#[test]
/// Executes the `primary_option_rows_list_every_connected_monitor` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
/// Executes the `workspace_editor_marks_this_monitor_and_others` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
/// Executes the `app_navigates_home_to_detail_picker_and_back` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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

#[test]
fn tab_does_nothing_and_end_reaches_refresh() {
  let mut app = app_with(false);
  app.handle(KeyCode::Tab);
  assert_eq!(selected(&app), Some(Item::Monitor(0)));
  app.handle(KeyCode::End);
  assert_eq!(selected(&app), Some(Item::Refresh));
}

#[test]
fn profiles_open_a_profile_page_with_its_actions() {
  let mut app = app_with(false);
  app.state.profiles = vec![profile("Casa"), profile("Trabalho")];
  app.state.active_profile = Some("Trabalho".into());
  app.page = DisplayPage::Profiles;
  assert_eq!(
    ids(&app),
    vec![
      Some(Item::Profile(0)),
      Some(Item::Profile(1)),
      None,
      Some(Item::NewProfile)
    ]
  );
  assert_eq!(app.rows()[1].detail_text(), Some("Active"));
  select(&mut app, Item::Profile(1));
  app.handle(KeyCode::Enter);
  assert_eq!(app.page, DisplayPage::Profile(1));
  let rows = app.rows();
  let last = rows.last().unwrap();
  assert_eq!(last.id(), Some(&Item::DeleteProfile));
  assert_eq!(last.kind(), RowKind::Destructive);
  assert_eq!(selected(&app), Some(Item::ApplyProfile));
  app.handle(KeyCode::Down);
  app.handle(KeyCode::Enter);
  assert_eq!(
    app.page,
    DisplayPage::Prompt {
      goal: PromptGoal::ProfileRename(1)
    }
  );
  assert_eq!(app.prompt_buffer, "Trabalho");
  app.handle(KeyCode::Esc);
  assert_eq!(
    app.page,
    DisplayPage::Profile(1),
    "rename returns to the profile"
  );
  app.handle(KeyCode::Esc);
  assert_eq!(app.page, DisplayPage::Profiles);
}

#[test]
fn new_profile_row_opens_the_name_prompt() {
  let mut app = app_with(false);
  app.page = DisplayPage::Profiles;
  assert_eq!(selected(&app), Some(Item::NewProfile));
  app.handle(KeyCode::Enter);
  assert_eq!(
    app.page,
    DisplayPage::Prompt {
      goal: PromptGoal::ProfileCreate
    }
  );
}

#[test]
fn rendered_pages_have_no_button_labels() {
  use ratatui::{Terminal, backend::TestBackend};
  let mut app = app_with(false);
  app.state.profiles.push(profile("Casa"));
  for page in [
    DisplayPage::Home,
    DisplayPage::Detail(0),
    DisplayPage::Profiles,
    DisplayPage::Profile(0),
    DisplayPage::Picker {
      monitor: 0,
      setting: MonitorSetting::Workspaces,
    },
  ] {
    app.page = page;
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    terminal.draw(|frame| app.draw(frame)).unwrap();
    let rendered: String = terminal
      .backend()
      .buffer()
      .content
      .iter()
      .map(|cell| cell.symbol())
      .collect();
    assert!(
      !rendered.replace("[ ]", "").contains("[ "),
      "{page:?}: {rendered}"
    );
  }
}

/// A config that differs from the default, to tell the revert target apart.
fn previous_config() -> PersistedConfig {
  let mut config = PersistedConfig::default();
  config.set_monitor(
    "eDP-1",
    PersistedMonitor {
      mode: Some("1920x1080@60".into()),
      ..Default::default()
    },
  );
  config
}

fn armed(now: Instant) -> DisplaysApp {
  let mut app = app_with(false);
  app.arm_revert("eDP-1".into(), monitor("eDP-1"), previous_config(), now);
  app
}

#[test]
fn revert_window_is_fifteen_seconds_from_the_change() {
  assert_eq!(REVERT_SECONDS, 15);
  let now = Instant::now();
  let app = armed(now);
  assert_eq!(
    app.revert.as_ref().unwrap().deadline,
    now + Duration::from_secs(15)
  );
}

#[test]
fn without_an_answer_the_previous_configuration_comes_back_at_the_deadline() {
  let now = Instant::now();
  let mut app = armed(now);
  assert!(
    app
      .take_expired_revert(now + Duration::from_millis(14_999))
      .is_none(),
    "nothing happens before the deadline"
  );
  assert!(app.revert.is_some());
  let expired = app
    .take_expired_revert(now + Duration::from_secs(15))
    .expect("reverts at the deadline");
  assert_eq!(
    expired.previous_config,
    previous_config(),
    "same destination"
  );
  assert_eq!(expired.previous.name, "eDP-1");
  assert!(app.revert.is_none());
  assert_eq!(
    revert_message(app.lang, &expired),
    format!("{} eDP-1", tr(app.lang, "control_center.reverted")),
    "same message as before"
  );
}

#[test]
fn revert_keys_follow_the_confirmation_component() {
  let now = Instant::now();
  let reverts = |key: KeyCode| {
    let mut app = armed(now);
    matches!(app.answer_revert(key), RevertAnswer::Revert(revert)
      if revert.previous_config == previous_config())
  };
  assert!(reverts(KeyCode::Enter), "Enter starts on Revert");
  assert!(reverts(KeyCode::Char('n')));
  assert!(reverts(KeyCode::Esc));

  let mut app = armed(now);
  assert!(matches!(
    app.answer_revert(KeyCode::Char('y')),
    RevertAnswer::Keep(name) if name == "eDP-1"
  ));
  assert!(app.revert.is_none());

  let mut app = armed(now);
  assert!(matches!(
    app.answer_revert(KeyCode::Char(' ')),
    RevertAnswer::Pending
  ));
  assert!(app.revert.is_some(), "Space has no effect");
  assert!(matches!(
    app.answer_revert(KeyCode::Up),
    RevertAnswer::Pending
  ));
  assert!(matches!(
    app.answer_revert(KeyCode::Enter),
    RevertAnswer::Keep(_)
  ));
}

#[test]
fn revert_footer_puts_y_keep_first() {
  let app = armed(Instant::now());
  let footer = app.footer_hints(&app.rows());
  assert!(
    footer.starts_with(&format!("y {}", tr(app.lang, "control_center.keep"))),
    "{footer}"
  );
}

#[test]
fn revert_dialog_renders_with_the_countdown() {
  use ratatui::{Terminal, backend::TestBackend};
  let mut app = armed(Instant::now());
  let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
  terminal.draw(|frame| app.draw(frame)).unwrap();
  let rendered: String = terminal
    .backend()
    .buffer()
    .content
    .iter()
    .map(|cell| cell.symbol())
    .collect();
  assert!(rendered.contains(tr(app.lang, "control_center.keep")));
  assert!(rendered.contains(tr(app.lang, "control_center.revert")));
  assert!(rendered.contains(" s"), "remaining seconds are shown");
}

#[test]
fn removing_a_configuration_and_deleting_a_profile_ask_first() {
  let mut app = app_with(false);
  app
    .config
    .set_monitor("HDMI-A-1", PersistedMonitor::default());
  app.page = DisplayPage::Detail(app.stale_start());
  app.handle(KeyCode::Enter);
  assert!(matches!(app.confirm, Some((Confirmation::RemoveConfig, _))));
  app.handle(KeyCode::Enter);
  assert!(app.confirm.is_none(), "Enter right after opening cancels");
  assert_eq!(app.config.monitors.len(), 1, "nothing removed");

  app.state.profiles.push(profile("Casa"));
  app.page = DisplayPage::Profile(0);
  select(&mut app, Item::DeleteProfile);
  app.handle(KeyCode::Enter);
  assert!(matches!(
    app.confirm,
    Some((Confirmation::DeleteProfile(0), _))
  ));
  assert!(app.footer_hints(&app.rows()).contains("y "));
  app.handle(KeyCode::Char('n'));
  assert!(app.confirm.is_none());
  assert_eq!(app.state.profiles.len(), 1, "nothing deleted");
}
