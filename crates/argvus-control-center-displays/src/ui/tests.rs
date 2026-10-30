use super::jobs::revert_message;
use super::picker::option_is_risky;
use super::profiles::rule_from_persisted;
use super::prompts::{parse_number, parse_position};
use super::*;
use crate::model::Mode;
use argvus_tui::menu::RowKind;

mod pickers_and_prompts;
mod profiles;
mod revert;

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
fn versions_gate_fancy_features() {
  let mut app = app_with(true);
  assert!(!app.vrr_supported() && !app.hdr_supported());
  app.version = (0, 42);
  assert!(app.vrr_supported() && app.hdr_supported());
}

#[test]
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
fn tab_does_nothing_and_end_reaches_refresh() {
  let mut app = app_with(false);
  app.handle(KeyCode::Tab);
  assert_eq!(selected(&app), Some(Item::Monitor(0)));
  app.handle(KeyCode::End);
  assert_eq!(selected(&app), Some(Item::Refresh));
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
fn night_light_navigation_validation_and_resize() {
  use ratatui::{Terminal, backend::TestBackend};
  let mut app = app_with(false);
  app.page = DisplayPage::NightLight;
  assert_eq!(app.breadcrumb(), "Desktop & Input > Displays > Night Light");
  assert!(
    !app
      .rows()
      .iter()
      .any(|row| row.id() == Some(&Item::NightToggle) && row.is_selectable())
  );
  app.night_light = crate::model::NightLightStatus {
    available: true,
    settings: crate::model::NightLightSettings {
      temperature: 4500,
      start: "20:00".into(),
      end: "07:00".into(),
      ..Default::default()
    },
    ..Default::default()
  };
  select(&mut app, Item::NightStart);
  for (width, height) in [(80, 24), (120, 35), (30, 8), (10, 3)] {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal.draw(|frame| app.draw(frame)).unwrap();
    assert_eq!(selected(&app), Some(Item::NightStart));
  }
  app.open_prompt(PromptGoal::NightStart);
  app.prompt_buffer = "24:00".into();
  app.commit_prompt(PromptGoal::NightStart);
  assert!(app.prompt_error.is_some());
  app.handle(KeyCode::Esc);
  assert_eq!(app.page, DisplayPage::NightLight);
  assert!(!app.handle(KeyCode::Esc));
  assert_eq!(app.page, DisplayPage::Home);
}
