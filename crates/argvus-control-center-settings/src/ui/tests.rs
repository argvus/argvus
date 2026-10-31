use super::*;
use crate::navigation::Page;
use crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::{Terminal, backend::TestBackend};
use serde_json::json;

#[test]
fn ssh_keys_search_opens_on_slash_and_filters_rows() {
  let mut app = App::with_context(
    Page::SshKeys,
    crate::i18n::Lang::for_locale("en-US"),
    crate::theme::Theme::load(),
  );
  app.ssh_keys = crate::system::ssh_keys::SshKeys {
    available: true,
    entries: vec![crate::system::ssh_keys::SshKeyEntry {
      name: "alpha".into(),
      key_type: "ed25519".into(),
      fingerprint: "SHA256:alpha".into(),
      comment: "alpha@host".into(),
      has_passphrase: false,
      agent: crate::system::ssh_keys::AgentState::NotLoaded,
    }],
  };
  app.begin_search();
  assert!(app.searching, "/ must open the search on the SSH keys page");
  app.push_search('z');
  assert!(
    app
      .rows()
      .iter()
      .all(|row| row.id() != Some(&Item::LoadSshKey(0))),
    "a query matching no key hides its block"
  );
}

fn user_app() -> App {
  let mut app = App::with_context(
    Page::Main,
    crate::i18n::Lang::for_locale("en-US"),
    crate::theme::Theme::load(),
  );
  app.error_modal = None;
  app.navigation.push(Page::User);
  app.admin.accounts = json!({"actor_uid":1000, "shells":["/bin/bash", "/bin/zsh"], "groups":["users", "wheel"],
  "group_details":[
    {"name":"users","gid":1000,"members":[]},
    {"name":"wheel","gid":998,"members":["alice"]}
  ],
  "users":[
    {"user":"root", "uid":0, "gid":0, "name":"root", "shell":"/bin/bash", "primary_group":"root", "groups":[]},
    {"user":"alice", "uid":1000, "gid":1000, "name":"Alice", "shell":"/bin/bash", "primary_group":"users", "groups":["wheel"]}
  ]});
  app.admin.user = app.admin.accounts["users"][1].clone();
  app
}

/// Background of the first cell of the line that shows `label`.
fn line_bg(terminal: &Terminal<TestBackend>, label: &str) -> Option<ratatui::style::Color> {
  let width = terminal.backend().buffer().area.width as usize;
  terminal
    .backend()
    .buffer()
    .content
    .chunks(width)
    .find(|line| {
      line
        .iter()
        .map(|cell| cell.symbol())
        .collect::<String>()
        .contains(label)
    })
    .map(|line| line[3].bg)
}

#[test]
fn actions_are_rows_and_only_the_cursor_row_is_highlighted() {
  let mut terminal = Terminal::new(TestBackend::new(80, 40)).unwrap();
  let mut app = user_app();
  let selected_bg = app.theme.selected_background;
  terminal.draw(|frame| draw(&mut app, frame)).unwrap();
  // The list starts on its first option; the buttons are not highlighted.
  assert_eq!(line_bg(&terminal, "Avatar image"), Some(selected_bg));
  assert_ne!(line_bg(&terminal, "Save"), Some(selected_bg));

  // End moves the list to its last option, the info block's end.
  crate::event::handle(&mut app, Event::Key(KeyEvent::from(KeyCode::End)));
  terminal.draw(|frame| draw(&mut app, frame)).unwrap();
  assert_eq!(line_bg(&terminal, "Password"), Some(selected_bg));
  assert_ne!(line_bg(&terminal, "Avatar image"), Some(selected_bg));
}

#[test]
fn main_page_renders_brand_menu_and_footer() {
  let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
  let mut app = App::new(Page::Main);
  app.error_modal = None;
  terminal.draw(|frame| draw(&mut app, frame)).unwrap();
  let rendered = terminal
    .backend()
    .buffer()
    .content
    .iter()
    .map(|cell| cell.symbol())
    .collect::<String>();
  assert!(rendered.contains("ARGVUS"));
  assert!(rendered.contains("Default Apps") || rendered.contains("Aplicativos padrão"));
  let quit = format!(
    "q {}",
    crate::i18n::tr(app.lang, "control_center.hint.quit")
  );
  assert!(app.footer().ends_with(&quit), "{}", app.footer());
}

#[test]
fn mouse_touchpad_page_renders_integrated_input_controls() {
  let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();
  let mut app = App::new(Page::MouseTouchpad);
  app.error_modal = None;
  terminal.draw(|frame| draw(&mut app, frame)).unwrap();
  let rendered = terminal
    .backend()
    .buffer()
    .content
    .iter()
    .map(|cell| cell.symbol())
    .collect::<String>();
  let rows = app.rows();
  for row in rows
    .iter()
    .filter(|row| row.detail_text().is_some())
    .take(3)
  {
    assert!(
      rendered.contains(row.label()),
      "missing rendered row: {}",
      row.label()
    );
  }
}

#[test]
fn reset_pages_render_a_restore_defaults_row_without_brackets() {
  let mut terminal = Terminal::new(TestBackend::new(80, 40)).unwrap();
  for page in [Page::DefaultApps, Page::Fonts] {
    let mut app = App::new(page);
    app.error_modal = None;
    terminal.draw(|frame| draw(&mut app, frame)).unwrap();
    let rendered = terminal
      .backend()
      .buffer()
      .content
      .iter()
      .map(|cell| cell.symbol())
      .collect::<String>();
    assert!(rendered.contains(crate::i18n::tr(app.lang, "control_center.reset_defaults")));
    // Default Apps shows Restore Defaults as an action button, which is
    // drawn with brackets; the list rows of Fonts still have none.
    if page == Page::Fonts {
      assert!(!rendered.contains("[ "), "no button labels on {page:?}");
    }
  }
}

#[test]
fn hostname_editing_renders_popup_with_typed_value() {
  let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
  let mut app = App::new(Page::Hostname);
  app.error_modal = None;
  app.hostname_editing = true;
  app.hostname_input = "mybox".into();
  terminal.draw(|frame| draw(&mut app, frame)).unwrap();
  let rendered = terminal
    .backend()
    .buffer()
    .content
    .iter()
    .map(|cell| cell.symbol())
    .collect::<String>();
  assert!(rendered.contains("mybox_"), "typed value must render");
  assert!(
    rendered.contains("Enter apply") || rendered.contains("Enter aplicar"),
    "popup hint must render"
  );
  assert!(
    rendered.contains("Current hostname")
      || rendered.contains("Hostname atual")
      || rendered.contains("Nome do host atual"),
    "list row must still show the current hostname"
  );
}

#[test]
fn tab_focuses_restore_defaults_on_the_action_bar() {
  let mut app = App::new(Page::DefaultApps);
  app.error_modal = None;
  let before = app.selected_item();
  crate::event::handle(&mut app, Event::Key(KeyEvent::from(KeyCode::Tab)));
  assert_eq!(app.selected_item(), before, "the list did not move");
  assert_eq!(app.action_cursor(), Some(0));
  crate::event::handle(&mut app, Event::Key(KeyEvent::from(KeyCode::Enter)));
  assert!(matches!(
    app.confirm,
    Some(crate::app::PendingAction::ResetApps)
  ));
  crate::event::handle(&mut app, Event::Key(KeyEvent::from(KeyCode::Esc)));
  assert!(app.confirm.is_none());
}

#[test]
fn font_selector_page_has_reset_row_and_search_still_works() {
  let mut app = App::new(Page::Fonts);
  app.error_modal = None;
  app
    .navigation
    .push(Page::FontSelector(crate::config::fonts::FontTarget::Apps));
  assert_eq!(
    app.rows().last().and_then(|row| row.id().copied()),
    Some(crate::item::Item::ResetDefaults)
  );
  crate::event::handle(&mut app, Event::Key(KeyEvent::from(KeyCode::Char('/'))));
  assert!(app.searching);
  crate::event::handle(&mut app, Event::Key(KeyEvent::from(KeyCode::Esc)));
  assert!(!app.searching);
  crate::event::handle(&mut app, Event::Key(KeyEvent::from(KeyCode::Char('r'))));
  assert!(app.confirm.is_some());
}

#[test]
fn minimum_size_message_renders() {
  let mut terminal = Terminal::new(TestBackend::new(42, 10)).unwrap();
  let mut app = App::new(Page::Main);
  app.error_modal = None;
  terminal.draw(|frame| draw(&mut app, frame)).unwrap();
  let rendered = terminal
    .backend()
    .buffer()
    .content
    .iter()
    .map(|cell| cell.symbol())
    .collect::<String>();
  assert!(rendered.contains("60x15"));
  assert!(rendered.contains("42x10"));
}

#[test]
fn task_window_renders_while_open() {
  let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
  let mut app = App::new(Page::Main);
  app.error_modal = None;
  let live = argvus_control_center_core::process::LiveProcess::new();
  live.push_line("locale-gen");
  app.task_live = Some(live);
  app.task_open = true;
  terminal.draw(|frame| draw(&mut app, frame)).unwrap();
  let rendered = terminal
    .backend()
    .buffer()
    .content
    .iter()
    .map(|cell| cell.symbol())
    .collect::<String>();
  assert!(rendered.contains("locale-gen"), "live line must render");
  assert!(
    rendered.contains("Process") || rendered.contains("Processo"),
    "process window title must render"
  );
}

#[test]
fn administrative_pages_show_warning_without_hiding_actions_or_footer() {
  for locale in ["en-US", "pt-BR"] {
    for (width, height) in [(60, 18), (100, 30)] {
      for page in [
        Page::SystemLocales,
        Page::DateTime,
        Page::Hostname,
        Page::ConsoleKeymap,
        Page::Firewall,
        Page::User,
        Page::CreateUser,
        Page::Group,
      ] {
        let mut app = App::with_context(
          page,
          crate::i18n::Lang::for_locale(locale),
          crate::theme::Theme::load(),
        );
        app.error_modal = None;
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal.draw(|frame| draw(&mut app, frame)).unwrap();
        let lines: Vec<String> = terminal
          .backend()
          .buffer()
          .content
          .chunks(width as usize)
          .map(|row| row.iter().map(|cell| cell.symbol()).collect())
          .collect();
        let warning = lines
          .iter()
          .position(|row| row.contains("[WARN]"))
          .unwrap_or_else(|| panic!("missing warning: {page:?} {locale}"));
        assert!(warning < height as usize - 2);
        assert!(
          lines[height as usize - 2].contains("Esc"),
          "footer lost: {page:?} {lines:?}"
        );
        assert!(!lines.join("").contains("control_center."));
      }
    }
  }
}

#[test]
fn personal_language_and_keyboard_pages_do_not_require_root() {
  for page in [
    Page::TimeZone,
    Page::RegionalLocale,
    Page::Language,
    Page::KeyboardLayout,
    Page::Fonts,
    Page::Notifications,
  ] {
    assert!(!page.requires_admin());
    let mut app = App::with_context(
      page,
      crate::i18n::Lang::for_locale("en-US"),
      crate::theme::Theme::load(),
    );
    app.error_modal = None;
    let mut terminal = Terminal::new(TestBackend::new(60, 18)).unwrap();
    terminal.draw(|frame| draw(&mut app, frame)).unwrap();
    let rendered: String = terminal
      .backend()
      .buffer()
      .content
      .iter()
      .map(|cell| cell.symbol())
      .collect();
    assert!(!rendered.contains("[WARN]"), "{page:?}");
  }
}
