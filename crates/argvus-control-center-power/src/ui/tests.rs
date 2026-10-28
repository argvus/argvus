use super::*;

fn laptop_state() -> PowerState {
  PowerState {
    lid: [PowerBehavior::Suspend, PowerBehavior::Hibernate],
    power_button: PowerButtonBehavior::Poweroff,
    screen_off_minutes: Some(15),
    lock_minutes: Some(15),
    can_suspend: true,
    can_hibernate: true,
    screen_off_supported: true,
    lock_supported: true,
    keep_awake: false,
    is_laptop: true,
  }
}

fn desktop_state() -> PowerState {
  PowerState {
    lid: [PowerBehavior::Suspend, PowerBehavior::Hibernate],
    power_button: PowerButtonBehavior::Poweroff,
    screen_off_minutes: Some(15),
    lock_minutes: Some(15),
    can_suspend: true,
    can_hibernate: true,
    screen_off_supported: true,
    lock_supported: true,
    keep_awake: false,
    is_laptop: false,
  }
}

#[test]
fn energy_app_loads_in_the_background() {
  let app = PowerApp::new(Lang::for_locale("en-US"), Theme::load());
  assert!(app.job.is_some());
  assert!(app.status.is_some());
}

#[test]
fn row_count_laptop() {
  let mut app = PowerApp::new(Lang::for_locale("en-US"), Theme::load());
  assert_eq!(app.row_count(), 0);
  app.state = Some(laptop_state());
  assert_eq!(app.row_count(), 7);
}

#[test]
fn row_count_desktop() {
  let mut app = PowerApp::new(Lang::for_locale("en-US"), Theme::load());
  assert_eq!(app.row_count(), 0);
  app.state = Some(desktop_state());
  assert_eq!(app.row_count(), 3);
}

#[test]
fn picker_opens_closes_and_applies_screen_off() {
  let mut app = PowerApp::new(Lang::for_locale("en-US"), Theme::load());
  app.job = None;
  app.action = None;
  app.state = Some(desktop_state());
  app.selected = 0;
  app.handle(KeyCode::Enter);
  assert!(app.picker.is_some());
  assert_eq!(app.picker.unwrap().target, PickerTarget::ScreenOff);
  app.handle(KeyCode::Down);
  app.handle(KeyCode::Down);
  app.handle(KeyCode::Enter);
  assert!(app.picker.is_none());
  assert_eq!(app.state.as_ref().unwrap().screen_off_minutes, Some(60));
  assert!(app.action.is_some());
}

#[test]
fn picker_escape_cancels_without_change() {
  let mut app = PowerApp::new(Lang::for_locale("en-US"), Theme::load());
  app.job = None;
  app.action = None;
  app.state = Some(desktop_state());
  app.selected = 0;
  app.handle(KeyCode::Enter);
  app.handle(KeyCode::Down);
  assert_eq!(app.picker.unwrap().selected, 3);
  app.handle(KeyCode::Esc);
  assert!(app.picker.is_none());
  assert_eq!(app.state.as_ref().unwrap().screen_off_minutes, Some(15));
  assert!(app.action.is_none());
}

#[test]
fn picker_nunca_clears_screen_off() {
  let mut app = PowerApp::new(Lang::for_locale("en-US"), Theme::load());
  app.job = None;
  app.action = None;
  app.state = Some(desktop_state());
  app.selected = 0;
  app.handle(KeyCode::Enter);
  app.handle(KeyCode::End);
  assert_eq!(app.picker.unwrap().selected, 6);
  app.handle(KeyCode::Enter);
  assert!(app.picker.is_none());
  assert_eq!(app.state.as_ref().unwrap().screen_off_minutes, None);
  assert!(app.action.is_some());
}

#[test]
fn picker_navigation_is_bounded() {
  let mut app = PowerApp::new(Lang::for_locale("en-US"), Theme::load());
  app.job = None;
  app.action = None;
  app.state = Some(desktop_state());
  app.selected = 0;
  app.handle(KeyCode::Enter);
  for _ in 0..6 {
    app.handle(KeyCode::Down);
  }
  assert_eq!(app.picker.unwrap().selected, 6);
  app.handle(KeyCode::Home);
  assert_eq!(app.picker.unwrap().selected, 0);
}

#[test]
fn lid_picker_applies_selected_behavior() {
  let mut app = PowerApp::new(Lang::for_locale("en-US"), Theme::load());
  app.job = None;
  app.action = None;
  app.state = Some(laptop_state());
  app.selected = 0;
  app.handle(KeyCode::Enter);
  assert_eq!(app.picker.unwrap().target, PickerTarget::LidBattery);
  app.handle(KeyCode::Down);
  app.handle(KeyCode::Enter);
  assert!(app.picker.is_none());
  assert_eq!(app.state.as_ref().unwrap().lid[0], PowerBehavior::Hibernate);
  assert!(app.action.is_some());
}

#[test]
fn picker_defaults_match_current_state() {
  let state = laptop_state();
  assert_eq!(picker_default(PickerTarget::LidBattery, &state), 0);
  assert_eq!(picker_default(PickerTarget::LidAc, &state), 1);
  assert_eq!(picker_default(PickerTarget::PowerButton, &state), 0);
  assert_eq!(picker_target_len(PickerTarget::ScreenOff), 7);
  assert_eq!(picker_target_len(PickerTarget::Lock), 6);
  let desktop = desktop_state();
  assert_eq!(picker_default(PickerTarget::ScreenOff, &desktop), 2);
  assert_eq!(picker_default(PickerTarget::Lock, &desktop), 3);
}

#[test]
fn lock_picker_opens_on_desktop_row_after_screen_off() {
  let mut app = PowerApp::new(Lang::for_locale("en-US"), Theme::load());
  app.job = None;
  app.action = None;
  app.state = Some(desktop_state());
  app.selected = 0;
  app.handle(KeyCode::Enter);
  assert_eq!(app.picker.unwrap().target, PickerTarget::ScreenOff);
  app.handle(KeyCode::Esc);
  app.selected = 1;
  app.handle(KeyCode::Enter);
  assert_eq!(app.picker.unwrap().target, PickerTarget::Lock);
  app.handle(KeyCode::Home);
  app.handle(KeyCode::Enter);
  assert!(app.picker.is_none());
  assert_eq!(app.state.as_ref().unwrap().lock_minutes, Some(1));
  assert!(app.action.is_some());
}

#[test]
fn lock_picker_nunca_clears_lock() {
  let mut app = PowerApp::new(Lang::for_locale("en-US"), Theme::load());
  app.job = None;
  app.action = None;
  app.state = Some(desktop_state());
  app.selected = 1;
  app.handle(KeyCode::Enter);
  assert_eq!(app.picker.unwrap().target, PickerTarget::Lock);
  app.handle(KeyCode::End);
  assert_eq!(app.picker.unwrap().selected, 5);
  app.handle(KeyCode::Enter);
  assert!(app.picker.is_none());
  assert_eq!(app.state.as_ref().unwrap().lock_minutes, None);
  assert!(app.action.is_some());
}

#[test]
fn power_app_navigates_and_toggles_action_focus() {
  let mut app = PowerApp::new(Lang::for_locale("en-US"), Theme::load());
  app.job = None;
  app.action = None;
  app.state = Some(laptop_state());
  app.handle(KeyCode::Down);
  assert_eq!(app.selected, 1);
  app.handle(KeyCode::Tab);
  assert_eq!(app.actions.cursor(), Some(0));
  app.handle(KeyCode::Tab);
  assert_eq!(app.actions.cursor(), None);
  app.handle(KeyCode::BackTab);
  assert_eq!(
    app.actions.cursor(),
    Some(0),
    "Tab and BackTab both focus the first button, as in Time Zone"
  );
}

#[test]
fn power_arrows_on_the_list_reach_the_action_bar_and_up_gives_it_back() {
  let mut app = PowerApp::new(Lang::for_locale("en-US"), Theme::load());
  app.job = None;
  app.action = None;
  app.state = Some(laptop_state());
  app.handle(KeyCode::Down);
  let before = app.selected;
  app.handle(KeyCode::Right);
  assert_eq!(app.actions.cursor(), Some(0), "→ focuses the bar");
  assert_eq!(app.selected, before, "the list did not move");
  app.handle(KeyCode::Right);
  assert_eq!(app.actions.cursor(), Some(1), "→ steps to the next button");
  app.handle(KeyCode::Up);
  assert_eq!(
    app.actions.cursor(),
    None,
    "↑ gives the focus back to the list"
  );
  assert_eq!(app.selected, before - 1, "and moves the list");
}

#[test]
fn power_esc_goes_back_and_left_does_not_leave_the_page() {
  let mut app = PowerApp::new(Lang::for_locale("en-US"), Theme::load());
  app.job = None;
  app.action = None;
  app.state = Some(laptop_state());
  assert!(!app.handle(KeyCode::Left), "← never leaves the page");
  assert!(app.handle(KeyCode::Esc), "Esc on Home leaves the page");
}

#[test]
fn idle_label_handles_hours_and_minutes() {
  assert_eq!(idle_label(0), "Nunca");
  assert_eq!(idle_label(15), "15 min");
  assert_eq!(idle_label(60), "1 h");
  assert_eq!(idle_label(120), "2 h");
}

#[test]
fn power_app_renders_without_panic() {
  let mut app = PowerApp::new(Lang::for_locale("en-US"), Theme::load());
  app.state = Some(laptop_state());
  let mut terminal = ratatui::Terminal::new(ratatui::backend::TestBackend::new(100, 20)).unwrap();
  terminal.draw(|frame| app.draw(frame)).unwrap();
}

#[test]
fn power_app_desktop_renders_without_panic() {
  let mut app = PowerApp::new(Lang::for_locale("en-US"), Theme::load());
  app.state = Some(desktop_state());
  let mut terminal = ratatui::Terminal::new(ratatui::backend::TestBackend::new(100, 20)).unwrap();
  terminal.draw(|frame| app.draw(frame)).unwrap();
}

#[test]
fn power_app_renders_with_picker_open() {
  let mut app = PowerApp::new(Lang::for_locale("en-US"), Theme::load());
  app.state = Some(desktop_state());
  app.picker = Some(Picker {
    target: PickerTarget::ScreenOff,
    selected: 6,
  });
  let mut terminal = ratatui::Terminal::new(ratatui::backend::TestBackend::new(100, 20)).unwrap();
  terminal.draw(|frame| app.draw(frame)).unwrap();
}
