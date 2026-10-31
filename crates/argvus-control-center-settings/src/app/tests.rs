use argvus_tui::menu::{Row, RowKind};
use crossterm::event::{Event, KeyCode, KeyEvent};

use super::*;
use crate::item::{Item, RatbagRow};
use crate::system::ssh_keys::{AgentState, SshKeyEntry, SshKeys};

mod dashboards;
mod gpg_key_pages;
mod keybinding_pages;
mod keyboard_layout_page;
mod snippets_page;
mod ssh_key_page;
mod system_locales_page;
mod timezone_page;

fn app(page: Page) -> App {
  let mut app = App::with_context(page, Lang::for_locale("en-US"), Theme::load());
  app.error_modal = None;
  app
}

fn press(app: &mut App, code: KeyCode) {
  crate::event::handle(app, Event::Key(KeyEvent::from(code)));
}

fn row_of(rows: &[Row<Item>], item: Item) -> &Row<Item> {
  rows
    .iter()
    .find(|row| row.id() == Some(&item))
    .unwrap_or_else(|| panic!("missing row {item:?}"))
}

#[test]
fn search_is_case_insensitive() {
  assert!(search_matches("nOtO", &["Noto Sans", "Regular"]));
  assert!(!search_matches("Roboto", &["Noto Sans"]));
}

#[test]
fn hardware_rows_follow_ratbag_capabilities() {
  let mut app = app(Page::MouseTouchpad);
  app.input_loading = false;
  app.input.devices.mouse = true;
  app.input.ratbag = vec![crate::system::ratbag::Device {
    path: "/device/test".into(),
    name: "Test mouse".into(),
    profiles: vec![crate::system::ratbag::Profile {
      path: "/profile/test".into(),
      name: String::new(),
      index: 3,
      active: true,
      disabled: false,
      report_rate: Some(500),
      report_rates: vec![125, 500],
      resolutions: vec![crate::system::ratbag::Resolution {
        path: "/resolution/test".into(),
        index: 2,
        dpi_x: 800,
        dpi_y: 800,
        active: true,
        default: true,
        supported: vec![400, 800],
      }],
    }],
  }];
  let rows = app.rows();
  let hardware = rows
    .iter()
    .find(|row| row.label() == tr(app.lang, "control_center.input_hardware_mouse"))
    .expect("hardware section");
  assert!(hardware.is_section());
  assert_eq!(hardware.detail_text(), Some("Test mouse"));
  let dpi = row_of(&rows, Item::Ratbag(RatbagRow::Dpi));
  assert!(dpi.is_selectable());
  assert_eq!(dpi.kind(), RowKind::Value { step: Some(1) });
  assert!(row_of(&rows, Item::Ratbag(RatbagRow::ReportRate)).is_selectable());
  assert!(
    !row_of(&rows, Item::Ratbag(RatbagRow::Profile)).is_selectable(),
    "a single profile cannot be switched"
  );
  assert!(
    rows
      .iter()
      .all(|row| row.id() != Some(&Item::Ratbag(RatbagRow::Device)))
  );
}

#[test]
fn mouse_rows_are_typed_and_disabled_without_a_mouse() {
  let mut app = app(Page::MouseTouchpad);
  app.input_loading = false;
  app.input.devices.mouse = false;
  app.input.devices.touchpad = true;
  let rows = app.rows();
  assert_eq!(
    row_of(&rows, Item::Input(1)).kind(),
    RowKind::Value { step: Some(1) }
  );
  assert!(matches!(
    row_of(&rows, Item::Input(3)).kind(),
    RowKind::Toggle { .. }
  ));
  assert!(!row_of(&rows, Item::Input(1)).is_selectable());
  assert!(row_of(&rows, Item::Input(7)).is_selectable());
  assert_eq!(app.selected_item(), Some(Item::Input(7)));
}

#[test]
fn labels_cover_every_backend_category() {
  for category in Category::ORDER {
    assert!(!category_label(Lang::for_locale("en-US"), category).is_empty());
    assert!(!category_label(Lang::for_locale("pt-BR"), category).is_empty());
  }
}

/// System Locales with `en_US` enabled and `pt_BR` available but not enabled.
/// The tests never confirm `Apply`, so no `locale-gen` command runs.
fn system_locales_app() -> App {
  let mut app = app(Page::SystemLocales);
  app.locale_gen_entries = vec![
    locale::LocaleEntry {
      locale: "en_US.UTF-8".into(),
      encoding: "UTF-8".into(),
      enabled: true,
      line_index: 0,
    },
    locale::LocaleEntry {
      locale: "pt_BR.UTF-8".into(),
      encoding: "UTF-8".into(),
      enabled: false,
      line_index: 1,
    },
  ];
  app.selected_locales = ["en_US.UTF-8 UTF-8".to_string()].into();
  app.navigation.current_mut().menu = argvus_tui::menu::MenuState::default();
  app.select_current();
  app
}

#[test]
fn task_window_scrolls_and_closes() {
  let mut app = app(Page::Main);
  let live = LiveProcess::new();
  for _ in 0..40 {
    live.push_line("line");
  }
  app.task_live = Some(live.clone());
  app.task_open = true;
  app.task_scroll = 0;
  press(&mut app, KeyCode::Down);
  assert_eq!(app.task_scroll, 1);
  press(&mut app, KeyCode::Up);
  assert_eq!(app.task_scroll, 0);
  press(&mut app, KeyCode::PageDown);
  assert_eq!(app.task_scroll, 10);
  press(&mut app, KeyCode::Home);
  assert_eq!(app.task_scroll, 0);
  press(&mut app, KeyCode::End);
  assert_eq!(
    app.task_scroll as usize,
    40usize.saturating_sub(TASK_CONTENT_HEIGHT)
  );
  press(&mut app, KeyCode::Esc);
  assert!(!app.task_open);
  assert_eq!(app.task_scroll, 0);
}

#[test]
fn task_bottom_offset_is_zero_for_short_output() {
  let mut app = app(Page::Main);
  let live = LiveProcess::new();
  live.push_line("done.");
  app.task_live = Some(live);
  app.task_open = true;
  press(&mut app, KeyCode::End);
  assert_eq!(app.task_scroll, 0);
  assert_eq!(task_bottom_offset("done."), 0);
}

#[test]
fn empty_poll_returns_false() {
  let mut app = app(Page::Main);
  assert!(!app.poll());
}

fn ssh_key_entry(name: &str, agent: AgentState) -> SshKeyEntry {
  SshKeyEntry {
    name: name.into(),
    key_type: "ed25519".into(),
    fingerprint: format!("SHA256:{name}"),
    comment: format!("{name}@host"),
    has_passphrase: false,
    agent,
  }
}

/// Keyboard > Layout with `br` as the configured default and `us` available.
/// The tests never call `Apply`, so no keyboard command runs.
fn keyboard_layout_app() -> App {
  let mut app = app(Page::KeyboardLayout);
  app.keyboard_layouts = vec![
    keyboard::Layout {
      code: "br".into(),
      description: "Portuguese (Brazil)".into(),
    },
    keyboard::Layout {
      code: "us".into(),
      description: "English (US)".into(),
    },
  ];
  app.keyboard_info.hypr_layout = "br".into();
  app.keyboard_info.default_layout = "br".into();
  app.keyboard_info.x11_layout = "br".into();
  app.keyboard_draft = keyboard_draft::LayoutDraft::loaded(&app.keyboard_info);
  app.navigation.current_mut().menu = argvus_tui::menu::MenuState::default();
  app
}

/// Time Zone with the system on `Europe/Lisbon` and two zones offered.
/// The tests never call `Apply`, so no `timedatectl` command runs.
fn timezone_app() -> App {
  let mut app = app(Page::TimeZone);
  app.timezones = vec!["America/Sao_Paulo".into(), "Europe/Lisbon".into()];
  app.datetime.time_zone = "Europe/Lisbon".into();
  app.load_timezone_draft();
  app.navigation.current_mut().menu = argvus_tui::menu::MenuState::default();
  app.select_current();
  app
}

#[test]
fn list_footer_names_the_arrows_only_when_apply_is_enabled() {
  let mut app = timezone_app();
  assert!(!app.footer().contains("←/→"), "Apply is dimmed");
  press(&mut app, KeyCode::Up);
  press(&mut app, KeyCode::Enter);
  assert!(
    app.footer().contains("←/→") && app.footer().contains("Actions"),
    "Time Zone shows ←/→ Actions"
  );

  let mut keyboard = keyboard_layout_app();
  press(&mut keyboard, KeyCode::Down);
  press(&mut keyboard, KeyCode::Char(' '));
  assert!(
    keyboard.footer().contains("←/→") && keyboard.footer().contains("Actions"),
    "Keyboard > Layout shows ←/→ Actions too"
  );
}

fn gpg_key_entry(user_id: &str, fingerprint: &str) -> crate::system::gpg_keys::GpgKeyEntry {
  crate::system::gpg_keys::GpgKeyEntry {
    fingerprint: fingerprint.into(),
    key_id: fingerprint.chars().take(16).collect(),
    key_type: "ed25519".into(),
    expires: "never".into(),
    secret: true,
    validity: "u".into(),
    user_id: user_id.into(),
  }
}

/// Moves the cursor to `item` from the top of the page.
fn select_gpg(app: &mut App, item: Item) {
  press(app, KeyCode::Home);
  for _ in 0..200 {
    if app.selected_item() == Some(item) {
      return;
    }
    press(app, KeyCode::Down);
  }
  panic!("could not select {item:?}");
}

/// Expands one collapsible section of the GPG keys page, as a user would.
fn open_gpg_section(app: &mut App, section: crate::item::GpgSection) {
  app.gpg_sections_open.insert(section);
}

const FP_FIRST: &str = "AAAA111122223333AAAA111122223333AAAAAAAA";
const FP_SECOND: &str = "CCCC111122223333CCCC111122223333CCCCCCCC";

/// A keyring with two keys that have a private part, the first one being edited.
fn gpg_keys_with_two_secrets(app: &mut App) {
  app.gpg_keys = crate::system::gpg_keys::GpgKeys {
    available: true,
    entries: vec![
      gpg_key_entry("Alice <alice@example.com>", FP_FIRST),
      gpg_key_entry("Bob <bob@example.com>", FP_SECOND),
    ],
    ..Default::default()
  };
}

#[test]
/// Every page reached from the Home shows the card it is listed under, so the
/// breadcrumb keeps the path the user followed (see the Control Center Home).
fn breadcrumbs_show_the_home_card_of_each_page() {
  let cases = [
    (
      Page::Hostname,
      "Control Center > System Administration > Hostname",
    ),
    (
      Page::UsersGroups,
      "Control Center > System Administration > Users & Groups",
    ),
    (
      Page::Notifications,
      "Control Center > System Administration > Notifications",
    ),
    (
      Page::Firewall,
      "Control Center > System Administration > Firewall",
    ),
    (
      Page::DefaultApps,
      "Control Center > Preferences > Default Apps",
    ),
    (Page::Language, "Control Center > Settings > Language"),
    (
      Page::LocaleRegion,
      "Control Center > Preferences > Language & Region",
    ),
    (Page::DateTime, "Control Center > Preferences > Date & Time"),
    (
      Page::Keybindings,
      "Control Center > Desktop & Input > Keyboard Shortcuts",
    ),
    (
      Page::WindowRules,
      "Control Center > Desktop & Input > Hyprland > Window rules",
    ),
    (
      Page::MouseTouchpad,
      "Control Center > Desktop & Input > Mouse & Touchpad",
    ),
    (
      Page::Projects,
      "Control Center > Tools & Diagnostics > Projects",
    ),
    (
      Page::SshKeys,
      "Control Center > Tools & Diagnostics > SSH keys",
    ),
    (
      Page::Fonts,
      "Control Center > Appearance > Desktop Style > Fonts",
    ),
  ];
  for (page, expected) in cases {
    assert_eq!(app(page).breadcrumb(), expected, "{page:?}");
  }
}

fn app_in(page: Page, locale: &str) -> App {
  let mut app = App::with_context(page, Lang::for_locale(locale), Theme::load());
  app.error_modal = None;
  app
}

#[test]
/// The same breadcrumbs in pt-BR: the root and every card are translated, and
/// no English card name leaks into the path.
fn breadcrumbs_are_translated_in_pt_br() {
  let cases = [
    (
      Page::Hostname,
      "Central de Controle > Administração do sistema > Nome do host",
    ),
    (
      Page::UsersGroups,
      "Central de Controle > Administração do sistema > Usuários e grupos",
    ),
    (
      Page::DefaultApps,
      "Central de Controle > Preferências > Aplicativos padrão",
    ),
    (
      Page::LocaleRegion,
      "Central de Controle > Preferências > Idioma e Região",
    ),
    (
      Page::Fonts,
      "Central de Controle > Aparência > Estilo do desktop > Fontes",
    ),
    (
      Page::Keybindings,
      "Central de Controle > Desktop e entrada > Atalhos de Teclado",
    ),
    (
      Page::MouseTouchpad,
      "Central de Controle > Desktop e entrada > Mouse e Touchpad",
    ),
    (
      Page::Projects,
      "Central de Controle > Ferramentas e diagnóstico > Projetos",
    ),
    (
      Page::SshKeys,
      "Central de Controle > Ferramentas e diagnóstico > Chaves SSH",
    ),
    (
      Page::Language,
      "Central de Controle > Configurações > Idioma",
    ),
  ];
  for (page, expected) in cases {
    let breadcrumb = app_in(page, "pt-BR").breadcrumb();
    assert_eq!(breadcrumb, expected, "{page:?}");
    assert!(
      !breadcrumb.contains("System Administration"),
      "{breadcrumb}"
    );
    assert!(!breadcrumb.contains("Control Center"), "{breadcrumb}");
  }
}
