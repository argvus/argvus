use super::*;
use crate::app::{InitialRoute, Route};
#[cfg(feature = "about")]
use argvus_control_center_about::Tab;
use crossterm::event::{KeyEventState, KeyModifiers};

fn press(code: KeyCode) -> Event {
  Event::Key(KeyEvent::new_with_kind_and_state(
    code,
    KeyModifiers::NONE,
    KeyEventKind::Press,
    KeyEventState::NONE,
  ))
}

#[cfg(feature = "apps")]
#[test]
fn escape_cancels_settings_confirmation_without_leaving_page() {
  let mut app = App::new(InitialRoute::Apps);
  app.settings.confirm = Some(argvus_control_center_settings::app::PendingAction::ResetApps);
  handle(&mut app, press(KeyCode::Esc));
  assert_eq!(app.route, Route::Settings);
  assert_eq!(
    app.settings.page(),
    argvus_control_center_settings::Page::DefaultApps
  );
  assert!(app.settings.confirm.is_none());
}

#[cfg(feature = "apps")]
#[test]
fn q_still_quits_and_question_mark_opens_help_outside_text_fields() {
  let mut app = App::new(InitialRoute::Apps);
  assert!(!app.captures_text());
  handle(&mut app, press(KeyCode::Char('?')));
  assert!(app.help);
  handle(&mut app, press(KeyCode::Esc));
  handle(&mut app, press(KeyCode::Char('q')));
  assert!(app.quit);
}

#[cfg(feature = "packages")]
#[test]
fn q_is_part_of_the_filter_on_package_lists_that_filter_while_typing() {
  let mut app = App::new(InitialRoute::Packages(
    argvus_control_center_packages::PackagesPage::Task(
      argvus_control_center_packages::PackageSource::Official,
      argvus_control_center_packages::PackageTask::Install,
    ),
  ));
  assert!(app.captures_text());
  handle(&mut app, press(KeyCode::Char('q')));
  assert!(!app.quit);
}

#[cfg(feature = "apps")]
#[test]
fn ctrl_c_quits_even_while_typing() {
  let mut app = App::new(InitialRoute::Apps);
  // fc-list does not run in tests, so the font discovery error modal is
  // open; the test is about typing, not about that error.
  app.settings.error_modal = None;
  app.settings.searching = true;
  assert!(app.captures_text());
  handle(
    &mut app,
    Event::Key(KeyEvent::new_with_kind_and_state(
      KeyCode::Char('c'),
      KeyModifiers::CONTROL,
      KeyEventKind::Press,
      KeyEventState::NONE,
    )),
  );
  assert!(app.quit);
}

#[cfg(feature = "about")]
#[test]
fn q_quits_from_about() {
  let mut app = App::new(InitialRoute::About(Tab::System));
  handle(&mut app, press(KeyCode::Char('q')));
  assert!(app.quit);
}

#[test]
fn account_editor_receives_quit_and_help_characters() {
  let mut app = App::new(InitialRoute::Home);
  app.route = Route::Settings;
  app.settings.error_modal = None;
  app
    .settings
    .navigation
    .push(argvus_control_center_settings::Page::CreateUser);
  app.settings.normalize_selection();
  app.settings.open_or_apply();
  app.settings.normalize_selection();
  app.settings.open_or_apply();
  handle(&mut app, press(KeyCode::Char('q')));
  handle(&mut app, press(KeyCode::Char('?')));
  assert!(!app.quit);
  assert!(!app.help);
  assert!(app.settings.admin.editor.is_some());
  handle(&mut app, press(KeyCode::Esc));
  assert!(app.settings.admin.editor.is_none());
  assert_eq!(
    app.settings.page(),
    argvus_control_center_settings::Page::UserAvatar
  );
}

#[test]
fn home_global_search_updates_live_and_clears_before_canceling() {
  let mut app = App::new(InitialRoute::Home);
  handle(&mut app, press(KeyCode::Char('/')));
  handle(&mut app, press(KeyCode::Char('w')));
  handle(&mut app, press(KeyCode::Char('i')));
  handle(&mut app, press(KeyCode::Char('f')));
  assert!(app.search_active);
  assert_eq!(app.search_query, "wif");
  assert!(app.search_result_ids.iter().any(|id| id == "network.wifi"));
  handle(&mut app, press(KeyCode::Esc));
  assert!(app.search_active);
  assert!(app.search_query.is_empty());
  handle(&mut app, press(KeyCode::Esc));
  assert!(!app.search_active);
}

#[test]
/// Ensures printable navigation letters remain usable as global search input while arrow keys move results.
fn home_global_search_accepts_j_and_k_and_uses_arrows_for_navigation() {
  let mut app = App::new(InitialRoute::Home);
  handle(&mut app, press(KeyCode::Char('/')));
  handle(&mut app, press(KeyCode::Char('k')));
  assert_eq!(app.search_query, "k");
  handle(&mut app, press(KeyCode::Backspace));
  handle(&mut app, press(KeyCode::Char('j')));
  assert_eq!(app.search_query, "j");

  app
    .search_registry
    .register(argvus_control_center_core::search::SearchEntry {
      id: "test.search.first".into(),
      category: "test".into(),
      title: "First result".into(),
      keywords: vec!["zzzz".into()],
      route: "test/first".into(),
    })
    .unwrap();
  app
    .search_registry
    .register(argvus_control_center_core::search::SearchEntry {
      id: "test.search.second".into(),
      category: "test".into(),
      title: "Second result".into(),
      keywords: vec!["zzzz".into()],
      route: "test/second".into(),
    })
    .unwrap();
  app.update_global_search("zzzz".into());
  assert_eq!(app.search_selected, 0);
  handle(&mut app, press(KeyCode::Down));
  assert_eq!(app.search_selected, 1);
  assert_eq!(app.search_query, "zzzz");
  handle(&mut app, press(KeyCode::Up));
  assert_eq!(app.search_selected, 0);
  assert_eq!(app.search_query, "zzzz");
}

#[cfg(feature = "language")]
#[test]
fn global_language_search_opens_the_preferences_language_page() {
  let mut app = App::new(InitialRoute::Home);
  let entry = app
    .search_registry
    .entries()
    .iter()
    .find(|entry| entry.id == "settings.language")
    .expect("interface language is searchable");
  assert_eq!(entry.category, "preferences");
  assert_eq!(entry.route, "settings/language");
  let route = entry.route.clone();

  app.open_search_route(&route);
  assert_eq!(app.route, Route::Settings);
  assert_eq!(
    app.settings.page(),
    argvus_control_center_settings::Page::Language
  );
  assert_eq!(
    app.settings.breadcrumb(),
    "Control Center > Preferences > Language & Region > Language"
  );
}

#[test]
fn clicking_home_search_field_focuses_global_search() {
  let mut app = App::new(InitialRoute::Home);
  handle(
    &mut app,
    Event::Mouse(crossterm::event::MouseEvent {
      kind: MouseEventKind::Down(MouseButton::Left),
      column: 10,
      row: 3,
      modifiers: KeyModifiers::NONE,
    }),
  );
  assert!(app.search_active);
}

#[cfg(feature = "network")]
#[test]
fn global_search_opens_a_real_deep_link() {
  let mut app = App::new(InitialRoute::Home);
  app.begin_global_search();
  app.update_global_search("wifi".into());
  app.open_global_search_result();
  assert_eq!(app.route, Route::Network);
  assert_eq!(
    app.network.page,
    argvus_control_center_network::NetworkPage::Wifi
  );
}

#[cfg(feature = "network")]
#[test]
/// The connection pages are found by what they manage and open with
/// their data: Add lists the kinds, Connections loads the profiles.
fn global_search_opens_the_connection_pages() {
  use argvus_control_center_network::NetworkPage;
  for (query, id, page) in [
    (
      "bridge",
      "network.add_connection",
      NetworkPage::AddConnection,
    ),
    ("nmtui", "network.connections", NetworkPage::Connections),
    ("activate", "network.activate", NetworkPage::Activate),
  ] {
    let mut app = App::new(InitialRoute::Home);
    app.begin_global_search();
    app.update_global_search(query.into());
    assert_eq!(app.search_result_ids.first().map(String::as_str), Some(id));
    app.open_global_search_result();
    assert_eq!(app.route, Route::Network);
    assert_eq!(app.network.page, page);
  }
}

#[cfg(feature = "fonts")]
#[test]
/// An active settings search is a text field: `q` and `?` are typed into
/// it instead of quitting or opening the help (D10 in `docs/ux-audit.md`;
/// before D10 this test asserted that `q` quit).
fn q_and_question_mark_are_typed_into_an_active_settings_search() {
  let mut app = App::new(InitialRoute::Fonts);
  // fc-list does not run in tests, so the font discovery error modal is
  // open; the test is about typing, not about that error.
  app.settings.error_modal = None;
  app.settings.searching = true;
  handle(&mut app, press(KeyCode::Char('q')));
  handle(&mut app, press(KeyCode::Char('?')));
  assert!(!app.quit, "q must not quit while typing");
  assert!(!app.help, "? must not open the help while typing");
  assert_eq!(app.settings.search, "q?");
}

#[cfg(feature = "about")]
#[test]
fn escape_returns_from_about_to_home() {
  let mut app = App::new(InitialRoute::About(Tab::System));
  handle(&mut app, press(KeyCode::Esc));
  assert_eq!(app.route, Route::Home);
}

#[cfg(feature = "about")]
#[test]
fn arrows_switch_about_tabs() {
  let mut app = App::new(InitialRoute::About(Tab::System));
  handle(&mut app, press(KeyCode::Right));
  assert_eq!(app.about.active_tab, Tab::About);
}

#[test]
fn all_domain_subpages_return_one_level_with_escape_and_left() {
  let cases = [
    #[cfg(feature = "network")]
    (
      InitialRoute::Network(argvus_control_center_network::NetworkPage::Wifi),
      Route::Network,
    ),
    #[cfg(feature = "audio")]
    (
      InitialRoute::Audio(argvus_control_center_audio::AudioPage::Output),
      Route::Audio,
    ),
    #[cfg(feature = "bluetooth")]
    (
      InitialRoute::Bluetooth(argvus_control_center_bluetooth::BluetoothPage::Devices),
      Route::Bluetooth,
    ),
    #[cfg(feature = "boot")]
    (
      InitialRoute::Boot(argvus_control_center_boot::BootPage::Kernel),
      Route::Boot,
    ),
    #[cfg(feature = "packages")]
    (
      InitialRoute::Packages(argvus_control_center_packages::PackagesPage::Source(
        argvus_control_center_packages::PackageSource::Official,
      )),
      Route::Packages,
    ),
    #[cfg(feature = "hardware")]
    (
      InitialRoute::Hardware(argvus_control_center_hardware::HardwarePage::Gpu),
      Route::Hardware,
    ),
    #[cfg(feature = "services")]
    (
      InitialRoute::Services(argvus_control_center_services::ServicePage::Logs),
      Route::Services,
    ),
  ];
  for (initial, route) in cases {
    let mut app = App::new(initial);
    assert_eq!(app.route, route);
    handle(&mut app, press(KeyCode::Esc));
    assert_eq!(app.route, route, "first back must stay in the domain");
    handle(&mut app, press(KeyCode::Left));
    assert_eq!(app.route, Route::Home, "second back must reach global home");
  }
}

#[cfg(feature = "network")]
#[test]
fn home_opens_network_and_back_returns_home() {
  let mut app = App::new(InitialRoute::Home);
  app.home_selected = app
    .home_destinations()
    .iter()
    .position(|destination| *destination == crate::home::HomeDestination::Network)
    .expect("Network row must be listed");
  handle(&mut app, press(KeyCode::Enter));
  assert_eq!(app.route, Route::Network);
  handle(&mut app, press(KeyCode::Esc));
  assert_eq!(app.route, Route::Home);
}

#[test]
fn home_tab_moves_between_grid_options() {
  let mut app = App::new(InitialRoute::Home);
  handle(&mut app, press(KeyCode::Tab));
  assert_eq!(app.home_selected, 1);
  handle(&mut app, press(KeyCode::BackTab));
  assert_eq!(app.home_selected, 0);
}

#[cfg(feature = "boot")]
#[test]
fn boot_home_enter_uses_the_selected_item() {
  let mut app = App::new(InitialRoute::Boot(
    argvus_control_center_boot::BootPage::Home,
  ));
  handle(&mut app, press(KeyCode::Down));
  handle(&mut app, press(KeyCode::Enter));
  assert_eq!(app.boot.page, argvus_control_center_boot::BootPage::Kernel);
}

#[test]
fn s_opens_the_settings_hub_and_escape_returns_home() {
  let mut app = App::new(InitialRoute::Home);
  handle(&mut app, press(KeyCode::Char('s')));
  assert_eq!(app.route, Route::AppSettings);
  handle(&mut app, press(KeyCode::Esc));
  assert_eq!(app.route, Route::Home);
}

#[test]
/// Esc on the Style screen returns to the Settings hub, as in the path Home > Settings > Style.
fn escape_on_style_returns_to_the_settings_hub() {
  let mut app = App::new(InitialRoute::Home);
  handle(&mut app, press(KeyCode::Char('s')));
  handle(&mut app, press(KeyCode::Down));
  handle(&mut app, press(KeyCode::Enter));
  assert_eq!(app.route, Route::Config);
  handle(&mut app, press(KeyCode::Esc));
  assert_eq!(app.route, Route::AppSettings);
}

#[test]
fn domain_home_lists_open_the_selected_page() {
  #[cfg(feature = "network")]
  {
    let mut app = App::new(InitialRoute::Home);
    app.route = Route::Network;
    // Status, then Connections.
    handle(&mut app, press(KeyCode::Down));
    handle(&mut app, press(KeyCode::Enter));
    assert_eq!(
      app.network.page,
      argvus_control_center_network::NetworkPage::Connections
    );
  }

  #[cfg(feature = "audio")]
  {
    let mut app = App::new(InitialRoute::Home);
    app.route = Route::Audio;
    handle(&mut app, press(KeyCode::Down));
    handle(&mut app, press(KeyCode::Enter));
    assert_eq!(
      app.audio.page,
      argvus_control_center_audio::AudioPage::Output
    );
  }

  #[cfg(feature = "bluetooth")]
  {
    let mut app = App::new(InitialRoute::Home);
    app.route = Route::Bluetooth;
    handle(&mut app, press(KeyCode::Down));
    handle(&mut app, press(KeyCode::Enter));
    assert_eq!(
      app.bluetooth.page,
      argvus_control_center_bluetooth::BluetoothPage::Devices
    );
  }

  #[cfg(feature = "packages")]
  {
    use argvus_control_center_packages::{PackageSource, PackageTask, PackagesPage};
    let mut app = App::new(InitialRoute::Home);
    app.route = Route::Packages;
    handle(&mut app, press(KeyCode::Enter));
    assert_eq!(
      app.packages.page,
      PackagesPage::Source(PackageSource::Official)
    );
    handle(&mut app, press(KeyCode::Down));
    handle(&mut app, press(KeyCode::Enter));
    assert_eq!(
      app.packages.page,
      PackagesPage::Task(PackageSource::Official, PackageTask::Remove)
    );
  }
}

#[test]
fn hardware_and_services_nested_pages_back_out_one_level() {
  #[cfg(feature = "hardware")]
  {
    let mut app = App::new(InitialRoute::Hardware(
      argvus_control_center_hardware::HardwarePage::Cpu,
    ));
    handle(&mut app, press(KeyCode::Esc));
    assert_eq!(
      app.hardware.page,
      argvus_control_center_hardware::HardwarePage::Home
    );
    handle(&mut app, press(KeyCode::Esc));
    assert_eq!(app.route, Route::Home);
  }

  #[cfg(feature = "services")]
  {
    let mut app = App::new(InitialRoute::Services(
      argvus_control_center_services::ServicePage::Detail,
    ));
    handle(&mut app, press(KeyCode::Esc));
    assert_eq!(
      app.services.page,
      argvus_control_center_services::ServicePage::System
    );
    handle(&mut app, press(KeyCode::Esc));
    assert_eq!(
      app.services.page,
      argvus_control_center_services::ServicePage::Home
    );
    handle(&mut app, press(KeyCode::Esc));
    assert_eq!(app.route, Route::Home);
  }
}

#[cfg(feature = "hardware")]
#[test]
fn hardware_route_initialization_is_reachable() {
  let app = App::new(InitialRoute::Hardware(
    argvus_control_center_hardware::HardwarePage::Cpu,
  ));
  assert_eq!(
    app.hardware.page,
    argvus_control_center_hardware::HardwarePage::Cpu
  );
}

#[cfg(feature = "bluetooth")]
#[test]
fn quit_waits_for_an_action_even_after_leaving_its_page() {
  for key in [
    KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE),
    KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
  ] {
    let mut app = App::new(InitialRoute::Home);
    app.bluetooth.page = argvus_control_center_bluetooth::BluetoothPage::State;
    app.bluetooth.handle(KeyCode::Tab);
    app.bluetooth.handle(KeyCode::Enter);
    assert!(app.bluetooth.has_pending_action());
    handle(&mut app, Event::Key(key));
    assert!(
      !app.quit,
      "must retain the application while an action is pending"
    );
    assert!(app.quit_blocked);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    while app.has_pending_action() {
      app.bluetooth.poll();
      app.settings.poll();
      app.settings.expire_status();
      app.config.poll();
      assert!(std::time::Instant::now() < deadline);
      std::thread::yield_now();
    }
    assert!(!app.quit, "completion must preserve the result for review");
    handle(&mut app, Event::Key(key));
    assert!(app.quit);
  }
}

#[cfg(feature = "network")]
#[test]
fn read_only_refresh_does_not_prevent_exit() {
  let mut app = App::new(InitialRoute::Home);
  app.network.reload();
  handle(&mut app, press(KeyCode::Char('q')));
  assert!(app.quit);
}

#[cfg(feature = "bluetooth")]
#[test]
fn blocked_exit_renders_a_translated_notice_in_both_languages() {
  use argvus_i18n::{Lang, tr};
  use ratatui::{Terminal, backend::TestBackend};
  for lang in [Lang::for_locale("en-US"), Lang::for_locale("pt-BR")] {
    let mut app = App::new(InitialRoute::Home);
    app.lang = lang;
    app.bluetooth.page = argvus_control_center_bluetooth::BluetoothPage::State;
    app.bluetooth.handle(KeyCode::Tab);
    app.bluetooth.handle(KeyCode::Enter);
    handle(&mut app, press(KeyCode::Char('q')));
    let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
    terminal
      .draw(|frame| crate::ui::draw(&mut app, frame))
      .unwrap();
    let text: String = terminal
      .backend()
      .buffer()
      .content()
      .iter()
      .map(|cell| cell.symbol())
      .collect();
    let notice = tr(lang, "control_center.quit_operation_pending");
    assert!(!notice.starts_with("control_center."));
    assert!(text.contains(notice), "{text}");
  }
}

#[test]
fn c11_altgr_c_does_not_trigger_global_ctrl_c() {
  let mut app = App::new(InitialRoute::Home);
  handle(
    &mut app,
    Event::Key(KeyEvent::new(
      KeyCode::Char('c'),
      KeyModifiers::CONTROL | KeyModifiers::ALT,
    )),
  );
  assert!(!app.quit);
}

#[cfg(feature = "packages")]
#[test]
fn package_terminal_receives_global_quit_help_and_interrupt_keys() {
  use argvus_control_center_packages::{PackageSource, PackageTask, PackagesPage};
  let mut app = App::new(InitialRoute::Packages(PackagesPage::Task(
    PackageSource::Official,
    PackageTask::Update,
  )));
  // Finish the initial read-only load; the core testing feature prevents tools.
  let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
  while !app.packages.poll() {
    assert!(std::time::Instant::now() < deadline);
    std::thread::yield_now();
  }
  handle(&mut app, press(KeyCode::Tab));
  handle(&mut app, press(KeyCode::Enter));
  handle(&mut app, press(KeyCode::Char('y')));
  assert!(app.packages.terminal_focused());
  for key in [
    KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE),
    KeyEvent::new(KeyCode::Char('?'), KeyModifiers::NONE),
    KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
  ] {
    handle(&mut app, Event::Key(key));
    assert!(!app.quit);
    assert!(!app.help);
    assert_eq!(app.route, Route::Packages);
  }
  handle(&mut app, press(KeyCode::Esc));
  assert!(!app.packages.terminal_focused());
  app.help = true;
  handle(&mut app, press(KeyCode::F(8)));
  assert!(app.packages.terminal_focused());
  assert!(!app.help);
}

#[cfg(feature = "packages")]
#[test]
fn package_search_shortcut_and_paste_edit_the_top_bar() {
  use argvus_control_center_packages::{PackageSource, PackageTask, PackagesPage};
  let mut app = App::new(InitialRoute::Packages(PackagesPage::Task(
    PackageSource::Official,
    PackageTask::Install,
  )));
  handle(&mut app, press(KeyCode::Char('/')));
  handle(&mut app, Event::Paste("firefox".into()));
  handle(&mut app, press(KeyCode::Esc));
  assert_eq!(app.route, Route::Packages);
  let mut terminal = ratatui::Terminal::new(ratatui::backend::TestBackend::new(100, 30)).unwrap();
  terminal.draw(|frame| app.packages.draw(frame)).unwrap();
  let text = terminal
    .backend()
    .buffer()
    .content
    .iter()
    .map(|cell| cell.symbol())
    .collect::<String>();
  assert!(
    text.contains("firefox"),
    "Esc must preserve the query in Search: {text}"
  );
  assert!(!app.help);
}
