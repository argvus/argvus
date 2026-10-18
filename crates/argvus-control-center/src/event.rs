//! Implements input event normalization and dispatch in crate `argvus control center`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use crossterm::event::{
  Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind,
};

use crate::app::{App, Route};

/// Processes `handle` in this module's event flow. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn handle(app: &mut App, event: Event) {
  if let Event::Paste(text) = event {
    if app.route == Route::Settings {
      app.settings.admin_paste(&text);
    }
    #[cfg(feature = "appearance")]
    if app.route == Route::Appearance {
      app.appearance.paste(&text);
    }
    return;
  }
  if let Event::Resize(width, height) = event {
    app.resize(width, height);
    return;
  }
  if let Event::Mouse(mouse) = event {
    handle_mouse(app, mouse);
    return;
  }
  let Event::Key(key) = event else {
    return;
  };
  if key.kind != KeyEventKind::Press {
    return;
  }
  if app.route == Route::Settings && app.settings.admin.editor.is_some() {
    argvus_control_center_settings::event::handle(&mut app.settings, Event::Key(key));
    return;
  }
  if app.route == Route::Settings && app.settings.admin.busy() {
    return;
  }
  if app.route == Route::Home && app.search_active {
    match key.code {
      KeyCode::Esc if app.search_query.is_empty() => app.clear_global_search(),
      KeyCode::Esc => app.update_global_search(String::new()),
      KeyCode::Backspace => app.pop_global_search(),
      KeyCode::Up => app.move_global_search(-1),
      KeyCode::Down => app.move_global_search(1),
      KeyCode::Enter => app.open_global_search_result(),
      KeyCode::Char(character) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
        let mut query = app.search_query.clone();
        query.push(character);
        app.update_global_search(query);
      }
      _ => {}
    }
    return;
  }
  if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
    app.quit = true;
    return;
  }
  if app.help {
    if matches!(key.code, KeyCode::Char('?') | KeyCode::Esc | KeyCode::Enter) {
      app.help = false;
    }
    return;
  }
  // While a page is typing, `?` and `q` are text for that page (D10).
  let typing = app.captures_text();
  if key.code == KeyCode::Char('?') && !typing {
    app.help = true;
    return;
  }
  if key.code == KeyCode::Char('q') && !typing {
    app.quit = true;
    return;
  }

  match app.route {
    Route::Home => handle_home(app, key),
    Route::AppSettings => match app.settings_hub.handle(key.code) {
      crate::settings_hub::HubAction::Back => app.route = Route::Home,
      crate::settings_hub::HubAction::Language => {
        // Esc on the Language page returns here instead of to the Home.
        app.return_route = Route::AppSettings;
        app.open_settings(argvus_control_center_settings::Page::Language);
      }
      crate::settings_hub::HubAction::Style => app.route = Route::Config,
      crate::settings_hub::HubAction::None => {}
    },
    Route::Config => {
      if app.config.handle(key.code) {
        app.route = Route::AppSettings;
      }
    }
    Route::Settings => {
      if key.code == KeyCode::Esc
        && !app.settings.searching
        && app.settings.error_modal.is_none()
        && app.settings.confirm.is_none()
        && !app.settings.has_keybinding_conflict()
        && !app.settings.hostname_editing
        && !app.settings.task_open
      {
        app.back();
      } else {
        argvus_control_center_settings::event::handle(&mut app.settings, Event::Key(key));
        app.lang = app.settings.lang;
        if app.settings.page() == argvus_control_center_settings::Page::Main {
          app.route = std::mem::replace(&mut app.return_route, Route::Home);
        }
      }
    }
    #[cfg(feature = "about")]
    Route::About => handle_about(app, key),
    #[cfg(feature = "hardware")]
    Route::Hardware => {
      if app.hardware.handle(domain_key(key.code)) {
        app.route = Route::Home;
      }
    }
    #[cfg(feature = "services")]
    Route::Services => {
      if app.services.handle(domain_key(key.code)) {
        app.route = Route::Home;
      }
    }
    #[cfg(feature = "dev-services")]
    Route::DevServices => {
      if app.dev_services.handle(domain_key(key.code)) {
        app.route = Route::Home;
      }
    }
    #[cfg(feature = "network")]
    Route::Network => {
      if app.network.handle(domain_key(key.code)) {
        app.route = Route::Home;
      }
    }
    #[cfg(feature = "audio")]
    Route::Audio => {
      if app.audio.handle(domain_key(key.code)) {
        app.route = Route::Home;
      }
    }
    #[cfg(feature = "bluetooth")]
    Route::Bluetooth => {
      if app.bluetooth.handle(domain_key(key.code)) {
        app.route = Route::Home;
      }
    }
    #[cfg(feature = "boot")]
    Route::Boot => {
      if app.boot.handle(domain_key(key.code)) {
        app.route = Route::Home;
      }
    }
    #[cfg(feature = "packages")]
    Route::Packages => {
      if app.packages.handle(domain_key(key.code)) {
        app.route = Route::Home;
      }
    }
    #[cfg(feature = "storage")]
    Route::Storage => {
      if app.storage.handle(domain_key(key.code)) {
        app.route = Route::Home;
      }
    }
    #[cfg(feature = "diagnostics")]
    Route::Diagnostics => {
      if app.diagnostics.handle(domain_key(key.code)) {
        app.route = Route::Home;
      }
    }
    #[cfg(feature = "power")]
    Route::Power => {
      if app.power.handle(domain_key(key.code)) {
        app.route = Route::Home;
      }
    }
    #[cfg(feature = "session")]
    Route::Session => {
      if app.session.handle(domain_key(key.code)) {
        app.route = Route::Home;
      }
    }
    #[cfg(feature = "displays")]
    Route::Displays => {
      if app.displays.handle(domain_key(key.code)) {
        app.route = Route::Home;
      }
    }
    #[cfg(feature = "appearance")]
    Route::Appearance => {
      let shift_vertical = key.modifiers.contains(KeyModifiers::SHIFT)
        && matches!(key.code, KeyCode::Up | KeyCode::Down);
      let moved = shift_vertical
        && (app
          .appearance
          .move_focused_telemetry_block(key.code == KeyCode::Down)
          || app
            .appearance
            .move_focused_control_panel_card(key.code == KeyCode::Down));
      if moved {
        return;
      }
      if app.appearance.handle(domain_key(key.code)) {
        app.route = std::mem::replace(&mut app.return_route, Route::Home);
      } else if app.appearance.take_fonts_request() {
        // Fonts belongs to the settings crate. Esc on its main page returns
        // here, to Appearance > Desktop Style.
        app.return_route = Route::Appearance;
        app.open_settings(argvus_control_center_settings::Page::Fonts);
      }
    }
    Route::Hyprland => {
      if crate::hyprland::handle(app, key.code) {
        app.route = Route::Home;
      }
    }
  }
}

/// Processes `handle_mouse` in this module's event flow. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn handle_mouse(app: &mut App, mouse: crossterm::event::MouseEvent) {
  if app.route != Route::Home || !matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left)) {
    return;
  }
  // Home layout: outer border + one header row, then the three-row search field.
  let search_top = 2u16;
  let search_bottom = search_top.saturating_add(3);
  if mouse.row >= search_top && mouse.row < search_bottom {
    app.begin_global_search();
  }
}

#[cfg(any(
  feature = "hardware",
  feature = "services",
  feature = "network",
  feature = "audio",
  feature = "bluetooth",
  feature = "boot",
  feature = "packages",
  feature = "storage",
  feature = "diagnostics",
  feature = "power",
  feature = "session",
  feature = "displays",
  feature = "appearance"
))]
/// Executes the `domain_key` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn domain_key(key: KeyCode) -> KeyCode {
  key
}

/// Processes `handle_home` in this module's event flow. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn handle_home(app: &mut App, key: KeyEvent) {
  match key.code {
    KeyCode::Char('/') => app.begin_global_search(),
    KeyCode::Up | KeyCode::Char('k') => app.move_home(-1),
    KeyCode::Down | KeyCode::Char('j') => app.move_home(1),
    KeyCode::Home => app.home_selected = 0,
    KeyCode::End => app.home_selected = app.home_item_count().saturating_sub(1),
    KeyCode::PageUp => app.move_home(-5),
    KeyCode::PageDown => app.move_home(5),
    KeyCode::Tab => app.move_home(1),
    KeyCode::BackTab => app.move_home(-1),
    KeyCode::Left | KeyCode::Char('h') => app.move_home_category(-1),
    KeyCode::Right | KeyCode::Char('l') => app.move_home_category(1),
    KeyCode::Enter => app.open_home(),
    KeyCode::Char('s') => app.route = Route::AppSettings,
    _ => {}
  }
}

#[cfg(feature = "about")]
/// Processes `handle_about` in this module's event flow. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn handle_about(app: &mut App, key: KeyEvent) {
  match key.code {
    KeyCode::Esc => app.back(),
    KeyCode::Char('h') | KeyCode::Left | KeyCode::BackTab => app.about.prev_tab(),
    KeyCode::Char('l') | KeyCode::Right | KeyCode::Tab => app.about.next_tab(),
    KeyCode::Char('j') | KeyCode::Down => app.about.move_cursor(1),
    KeyCode::Char('k') | KeyCode::Up => app.about.move_cursor(-1),
    KeyCode::Enter => app.about.enter(),
    KeyCode::PageDown => app.about.page(1),
    KeyCode::PageUp => app.about.page(-1),
    KeyCode::Home => app.about.home(),
    KeyCode::End => app.about.end(),
    _ => {}
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::app::{InitialRoute, Route};
  #[cfg(feature = "about")]
  use argvus_control_center_about::Tab;
  use crossterm::event::{KeyEventState, KeyModifiers};

  /// Executes the `press` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
  /// Executes the `escape_cancels_settings_confirmation_without_leaving_page` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
      argvus_control_center_packages::PackagesPage::Search,
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
  /// Executes the `q_quits_from_about` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn q_quits_from_about() {
    let mut app = App::new(InitialRoute::About(Tab::System));
    handle(&mut app, press(KeyCode::Char('q')));
    assert!(app.quit);
  }

  #[test]
  /// Executes the `account_editor_receives_quit_and_help_characters` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
  /// Executes the `home_global_search_updates_live_and_clears_before_canceling` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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

  #[test]
  /// Executes the `clicking_home_search_field_focuses_global_search` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
  /// Executes the `global_search_opens_a_real_deep_link` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
  /// Executes the `escape_returns_from_about_to_home` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn escape_returns_from_about_to_home() {
    let mut app = App::new(InitialRoute::About(Tab::System));
    handle(&mut app, press(KeyCode::Esc));
    assert_eq!(app.route, Route::Home);
  }

  #[cfg(feature = "about")]
  #[test]
  /// Executes the `arrows_switch_about_tabs` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn arrows_switch_about_tabs() {
    let mut app = App::new(InitialRoute::About(Tab::System));
    handle(&mut app, press(KeyCode::Right));
    assert_eq!(app.about.active_tab, Tab::About);
  }

  #[test]
  /// Executes the `all_domain_subpages_return_one_level_with_escape_and_left` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
        InitialRoute::Packages(argvus_control_center_packages::PackagesPage::Updates),
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
  /// Executes the `home_opens_network_and_back_returns_home` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
  /// Executes the `boot_home_enter_uses_the_selected_item` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn boot_home_enter_uses_the_selected_item() {
    let mut app = App::new(InitialRoute::Boot(
      argvus_control_center_boot::BootPage::Home,
    ));
    handle(&mut app, press(KeyCode::Down));
    handle(&mut app, press(KeyCode::Enter));
    assert_eq!(app.boot.page, argvus_control_center_boot::BootPage::Kernel);
  }

  #[test]
  /// Executes the `s_opens_config_and_escape_returns_home` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
  /// Executes the `domain_home_lists_open_the_selected_page` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn domain_home_lists_open_the_selected_page() {
    #[cfg(feature = "network")]
    {
      let mut app = App::new(InitialRoute::Home);
      app.route = Route::Network;
      handle(&mut app, press(KeyCode::Down));
      handle(&mut app, press(KeyCode::Enter));
      assert_eq!(
        app.network.page,
        argvus_control_center_network::NetworkPage::Interfaces
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
      let mut app = App::new(InitialRoute::Home);
      app.route = Route::Packages;
      handle(&mut app, press(KeyCode::Down));
      handle(&mut app, press(KeyCode::Down));
      handle(&mut app, press(KeyCode::Enter));
      assert_eq!(
        app.packages.page,
        argvus_control_center_packages::PackagesPage::Installed
      );
    }
  }

  #[test]
  /// Executes the `hardware_and_services_nested_pages_back_out_one_level` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
  /// Executes the `hardware_route_initialization_is_reachable` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn hardware_route_initialization_is_reachable() {
    let app = App::new(InitialRoute::Hardware(
      argvus_control_center_hardware::HardwarePage::Cpu,
    ));
    assert_eq!(
      app.hardware.page,
      argvus_control_center_hardware::HardwarePage::Cpu
    );
  }
}
