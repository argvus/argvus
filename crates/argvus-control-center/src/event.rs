//! Input event normalization and dispatch.
use crossterm::event::{
  Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind,
};

use crate::app::{App, Route};

pub fn handle(app: &mut App, event: Event) {
  if app.welcome.active
    && app.route != Route::Welcome
    && let Event::Key(key) = &event
  {
    if key.code == KeyCode::BackTab && !app.captures_text() {
      app.welcome.done_focus = !app.welcome.done_focus;
      return;
    }
    if app.welcome.done_focus && matches!(key.code, KeyCode::Tab | KeyCode::Esc) {
      app.welcome.done_focus = false;
      return;
    }
  }
  let done = match &event {
    Event::Key(key) => {
      key.kind == KeyEventKind::Press
        && (key.code == KeyCode::F(6) || (app.welcome.done_focus && key.code == KeyCode::Enter))
    }
    Event::Mouse(mouse) => {
      matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left))
        && mouse.row == app.height.saturating_sub(1)
    }
    _ => false,
  };
  if done && app.welcome.active && app.route != Route::Welcome && !app.help && !app.captures_text()
  {
    if app.route == Route::Settings
      && (app.settings.confirm.is_some()
        || app.settings.error_modal.is_some()
        || app.settings.admin.busy()
        || app.settings.admin.editor.is_some())
    {
      return;
    }
    let mut ready = app.route != Route::Settings || app.settings.onboarding_ready();
    #[cfg(feature = "displays")]
    if app.route == Route::Displays {
      ready = app.displays.onboarding_ready();
    }
    #[cfg(feature = "appearance")]
    if app.route == Route::Appearance {
      ready = app.appearance.onboarding_ready();
    }
    #[cfg(feature = "network")]
    if app.route == Route::Network {
      ready = app.network.onboarding_ready();
    }
    #[cfg(feature = "audio")]
    if app.route == Route::Audio {
      ready = app.audio.onboarding_ready();
    }
    #[cfg(feature = "bluetooth")]
    if app.route == Route::Bluetooth {
      ready = app.bluetooth.onboarding_ready();
    }
    #[cfg(feature = "power")]
    if app.route == Route::Power {
      ready = app.power.onboarding_ready();
    }
    #[cfg(feature = "session")]
    if app.route == Route::Session {
      ready = app.session.onboarding_ready();
    }
    if !ready {
      app.welcome.done_blocked = true;
      return;
    }
    app.finish_welcome_step(argvus_welcome::state::StepStatus::Reviewed);
    return;
  }
  handle_inner(app, event);
  if app.welcome.active && app.route == Route::Home {
    app.route = Route::Welcome;
    app.welcome.reviewing = true;
  }
}

fn handle_inner(app: &mut App, event: Event) {
  if let Event::Paste(text) = event {
    #[cfg(feature = "snapshot")]
    if app.route == Route::Snapshot {
      app.snapshot.paste(&text);
    }
    if app.route == Route::Settings {
      app.settings.admin_paste(&text);
    }
    #[cfg(feature = "appearance")]
    if app.route == Route::Appearance {
      app.appearance.paste(&text);
    }
    #[cfg(feature = "packages")]
    if app.route == Route::Packages {
      app.packages.paste(&text);
    }
    #[cfg(feature = "network")]
    if app.route == Route::Network {
      app.network.paste(&text);
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
    Route::Welcome => app.handle_welcome(key.code),
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
      // The whole event: the connection editor fields take modifiers.
      if app.network.handle_key(key) {
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
      // The whole event: the repository editor saves with Ctrl+S.
      if app.packages.handle_key(key) {
        app.route = Route::Home;
      }
    }
    #[cfg(feature = "snapshot")]
    Route::Snapshot => {
      if app.snapshot.handle_key(key) {
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
      } else if let Some(route) = app.diagnostics.take_requested_route() {
        app.open_search_route(&route);
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
fn domain_key(key: KeyCode) -> KeyCode {
  key
}

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
fn handle_about(app: &mut App, key: KeyEvent) {
  match key.code {
    KeyCode::Esc => app.back(),
    KeyCode::Char('h') | KeyCode::Left | KeyCode::BackTab => app.about.prev_tab(),
    KeyCode::Char('l') | KeyCode::Right | KeyCode::Tab => app.about.next_tab(),
    KeyCode::Char('j') | KeyCode::Down => app.about.move_cursor(1),
    KeyCode::Char('k') | KeyCode::Up => app.about.move_cursor(-1),
    KeyCode::Enter => app.about.enter(),
    KeyCode::Char('c') => app.about.copy_report(),
    KeyCode::PageDown => app.about.page(1),
    KeyCode::PageUp => app.about.page(-1),
    KeyCode::Home => app.about.home(),
    KeyCode::End => app.about.end(),
    _ => {}
  }
}

#[cfg(test)]
mod tests;
