use argvus_tui::menu::{Row, RowKind};
use crossterm::event::{Event, KeyCode, KeyEvent};

use super::*;
use crate::item::{Item, RatbagRow};
use crate::system::ssh_keys::{AgentState, SshKeyEntry, SshKeys};

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

#[test]
fn language_page_shows_status_lines_and_selectable_choices() {
  let app = app(Page::Language);
  let rows = app.rows();
  assert_eq!(rows.len(), 6);
  assert_eq!(rows[0].label(), "Current language");
  assert_eq!(rows[0].kind(), RowKind::Info);
  assert_eq!(rows[1].label(), "Regional locale");
  assert_eq!(rows[2].label(), "Encoding");
  assert_eq!(rows[3].kind(), RowKind::Separator);
  assert_eq!(rows[4].id(), Some(&Item::Language(0)));
  assert_eq!(rows[5].id(), Some(&Item::Language(1)));
  assert!(
    rows.iter().all(|row| !row.label().contains('🇺')),
    "no flag emoji"
  );
  assert_eq!(
    rows[4].kind(),
    RowKind::Choice { current: true },
    "English is current"
  );
  assert_eq!(app.selected_item(), Some(Item::Language(0)));
}

#[test]
fn fonts_dashboard_has_sections_icons_status_and_danger_zone() {
  let app = app(Page::Fonts);
  let rows = app.rows();
  let targets: Vec<&Row<Item>> = rows
    .iter()
    .filter(|row| matches!(row.id(), Some(Item::FontTarget(_))))
    .collect();
  assert_eq!(targets.len(), FontTarget::ALL.len());
  assert_eq!(targets[0].icon_glyph(), Some(argvus_tui::icons::TASKBAR));
  assert!(targets.iter().all(|row| row.detail_text().is_some()));
  let settings = rows
    .iter()
    .filter(|row| matches!(row.id(), Some(Item::FontSetting(_))))
    .count();
  assert_eq!(settings, SettingKind::ALL.len());
  assert_eq!(rows.iter().filter(|row| row.is_section()).count(), 3);
  let last = rows.last().unwrap();
  assert_eq!(last.id(), Some(&Item::ResetDefaults));
  assert_eq!(last.kind(), RowKind::Destructive);
}

#[test]
fn fonts_dashboard_icons_are_distinct() {
  let mut seen = std::collections::HashSet::new();
  for target in FontTarget::ALL {
    let icon = font_target_icon(target);
    assert!(seen.insert(icon), "duplicate icon {icon} for {target:?}");
  }
  for setting in SettingKind::ALL {
    let icon = setting_icon(setting);
    assert!(seen.insert(icon), "duplicate icon {icon} for {setting:?}");
  }
}

#[test]
fn default_apps_dashboard_uses_icons_and_status() {
  let app = app(Page::DefaultApps);
  let rows = app.rows();
  assert_eq!(rows.len(), Category::ORDER.len() + 2);
  assert!(rows[0].label().contains("Terminal"), "{}", rows[0].label());
  assert_eq!(rows[0].icon_glyph(), Some(argvus_tui::icons::TERMINAL));
  assert_eq!(rows[0].kind(), RowKind::Submenu);
  for row in &rows[..Category::ORDER.len()] {
    assert!(row.detail_text().is_some());
  }
  assert_eq!(rows.last().unwrap().id(), Some(&Item::ResetDefaults));
}

#[test]
fn default_apps_dashboard_icons_are_distinct() {
  let mut seen = std::collections::HashSet::new();
  for category in Category::ORDER {
    let icon = category_icon(category);
    assert!(seen.insert(icon), "duplicate icon {icon} for {category:?}");
  }
}

#[test]
fn reset_row_and_r_open_the_same_confirmation() {
  let mut app = app(Page::DefaultApps);
  press(&mut app, KeyCode::End);
  assert_eq!(app.selected_item(), Some(Item::ResetDefaults));
  press(&mut app, KeyCode::Enter);
  assert!(matches!(app.confirm, Some(PendingAction::ResetApps)));
  press(&mut app, KeyCode::Esc);
  assert!(app.confirm.is_none());
  press(&mut app, KeyCode::Char('r'));
  assert!(matches!(app.confirm, Some(PendingAction::ResetApps)));
}

#[test]
fn users_groups_hub_uses_typed_submenus_with_counts() {
  let app = app(Page::UsersGroups);
  let rows = app.rows();
  assert_eq!(rows.len(), 2);
  assert_eq!(rows[0].id(), Some(&Item::Users));
  assert_eq!(rows[0].label(), "Users");
  assert_eq!(rows[0].detail_text(), Some("N/A"));
  assert_eq!(rows[1].id(), Some(&Item::Groups));
  assert!(!rows[1].label().ends_with(':'));
  assert_eq!(rows[1].detail_text(), Some("N/A"));
}

#[test]
fn notifications_page_holds_only_do_not_disturb() {
  let app = app(Page::Notifications);
  let rows = app.rows();
  assert_eq!(rows.len(), 1);
  assert_eq!(rows[0].id(), Some(&Item::DoNotDisturb));
  assert!(matches!(rows[0].kind(), RowKind::Toggle { .. }));
  assert!(rows[0].label().contains("Do Not Disturb"));
}

#[test]
fn hostname_row_opens_the_editor_directly() {
  let mut app = app(Page::Hostname);
  app.hostname = "current-machine".into();
  press(&mut app, KeyCode::Enter);
  assert!(app.hostname_editing);
  assert_eq!(app.hostname_input, "current-machine");
  assert_eq!(app.page(), Page::Hostname);
}

#[test]
fn hostname_page_row_keeps_current_value_while_editing() {
  let mut app = app(Page::Hostname);
  app.hostname = "current-machine".into();
  app.hostname_editing = true;
  app.hostname_input = "draft-name".into();
  let rows = app.rows();
  assert_eq!(rows.len(), 1);
  assert!(rows[0].label().contains("Current hostname"));
  assert_eq!(rows[0].detail_text(), Some("current-machine"));
}

#[test]
fn users_groups_counts_match_loaded_accounts() {
  let mut app = app(Page::UsersGroups);
  app.admin.accounts = serde_json::json!({
    "actor_uid":1000,
    "shells":["/bin/bash", "/bin/zsh"],
    "groups":["users", "wheel"],
    "group_details":[
      {"name":"users","gid":1000,"members":[]},
      {"name":"wheel","gid":998,"members":["alice"]}
    ],
    "users":[
      {"user":"root","uid":0,"gid":0,"name":"root","shell":"/bin/bash","primary_group":"root","groups":[]},
      {"user":"alice","uid":1000,"gid":1000,"name":"Alice","shell":"/bin/bash","primary_group":"users","groups":["wheel"]}
    ]
  });
  let rows = app.rows();
  assert_eq!(rows[0].detail_text(), Some("1"));
  assert_eq!(rows[1].detail_text(), Some("1"));
  assert!(rows.iter().all(Row::is_selectable));
}

#[test]
fn font_size_is_a_value_row_with_arrows_and_shortcuts() {
  let mut app = app(Page::Fonts);
  app.navigation.push(Page::FontSelector(FontTarget::Apps));
  app.pending_size = 13;
  let rows = app.rows();
  assert_eq!(rows[0].id(), Some(&Item::FontSize));
  assert_eq!(rows[0].kind(), RowKind::Value { step: Some(1) });
  assert!(
    app
      .navigation
      .current_mut()
      .menu
      .select(&rows, &Item::FontSize)
  );
  press(&mut app, KeyCode::Right);
  assert_eq!(app.pending_size, 14);
  press(&mut app, KeyCode::Left);
  press(&mut app, KeyCode::Char('-'));
  assert_eq!(app.pending_size, 12);
  assert_eq!(app.page(), Page::FontSelector(FontTarget::Apps));
  press(&mut app, KeyCode::Enter);
  assert!(app.admin.editor.is_some(), "Enter opens the size prompt");
}

#[test]
fn system_locales_toggle_with_enter_and_apply_from_the_last_row() {
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
  let apply = |app: &App| {
    app
      .rows()
      .into_iter()
      .find(|row| row.id() == Some(&Item::ApplyLocales))
      .unwrap()
      .is_selectable()
  };
  assert!(!apply(&app), "Apply is dimmed without changes");
  press(&mut app, KeyCode::Down);
  assert_eq!(app.selected_item(), Some(Item::LocaleGen(1)));
  press(&mut app, KeyCode::Enter);
  assert!(app.selected_locales.contains("pt_BR.UTF-8 UTF-8"));
  assert!(app.confirm.is_none(), "Enter toggles instead of applying");
  press(&mut app, KeyCode::End);
  assert_eq!(
    app.selected_item(),
    Some(Item::ApplyLocales),
    "Apply is enabled"
  );
  press(&mut app, KeyCode::Enter);
  assert!(matches!(
    app.confirm,
    Some(PendingAction::ApplySystemLocales)
  ));
}

#[test]
fn r_on_shortcuts_restores_the_selected_shortcut_after_confirmation() {
  let mut app = app(Page::Keybindings);
  let Some(first) = app.visible_keybindings().first().copied() else {
    return; // no keybinding catalog in this environment
  };
  let rows = app.rows();
  assert!(
    app
      .navigation
      .current_mut()
      .menu
      .select(&rows, &Item::Keybinding(first))
  );
  press(&mut app, KeyCode::Char('r'));
  let id = app.keybindings[first].id.clone();
  assert!(matches!(
    &app.confirm,
    Some(PendingAction::RestoreKeybinding { id: pending, leave: false }) if *pending == id
  ));
  assert_eq!(
    app.rows().last().and_then(|row| row.id().copied()),
    Some(Item::RestoreAllShortcuts)
  );
}

#[test]
fn keyboard_layout_footer_lists_enter_and_space() {
  let mut app = app(Page::KeyboardLayout);
  app.keyboard_layouts = vec![keyboard::Layout {
    code: "us".into(),
    description: "English (US)".into(),
  }];
  app.navigation.current_mut().menu = argvus_tui::menu::MenuState::default();
  let footer = app.footer();
  assert!(footer.contains("Enter "), "{footer}");
  assert!(footer.contains("Space "), "{footer}");
  assert!(!footer.contains("Enter/Space"), "{footer}");
}

#[test]
fn shortcut_conflict_is_a_confirmation_that_esc_cancels() {
  let mut app = app(Page::Keybindings);
  app.keybinding_conflict = Some(("a".into(), "SUPER + Q".into(), vec!["b".into()]));
  let (title, message, confirm, danger) = app.confirm_dialog().unwrap();
  assert_eq!(
    title,
    tr(app.lang, "control_center.keybindings_conflict_title")
  );
  assert!(message.contains("SUPER + Q"));
  assert_eq!(confirm, tr(app.lang, "control_center.replace"));
  assert!(!danger);
  assert!(app.footer().contains("r "), "{}", app.footer());
  press(&mut app, KeyCode::Enter);
  assert!(!app.has_keybinding_conflict(), "Enter on Cancel cancels");
  app.keybinding_conflict = Some(("a".into(), "SUPER + Q".into(), vec!["b".into()]));
  press(&mut app, KeyCode::Esc);
  assert!(!app.has_keybinding_conflict());
  assert_eq!(app.page(), Page::Keybindings);
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

#[test]
fn snippets_page_offers_type_and_remove_for_each_entry() {
  let mut app = app(Page::Snippets);
  app.snippets = crate::system::snippets::Snippets {
    available: true,
    entries: vec![crate::system::snippets::SnippetEntry {
      name: "email".into(),
      content: "you@example.com".into(),
    }],
  };
  let rows = app.rows();
  assert!(row_of(&rows, Item::AddSnippet).is_selectable());
  assert!(row_of(&rows, Item::OpenSnippetPicker).is_selectable());
  assert!(row_of(&rows, Item::SnippetContent(0)).is_selectable());
  assert!(row_of(&rows, Item::TypeSnippet(0)).is_selectable());
  assert!(row_of(&rows, Item::RemoveSnippet(0)).is_selectable());
}

#[test]
fn snippets_page_without_the_launcher_only_informs() {
  let mut app = app(Page::Snippets);
  app.snippets = crate::system::snippets::Snippets::default();
  let rows = app.rows();
  assert!(rows.iter().all(|row| row.id().is_none()));
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

#[test]
fn ssh_keys_page_offers_unload_only_for_loaded_keys() {
  let mut app = app(Page::SshKeys);
  app.ssh_keys = SshKeys {
    available: true,
    entries: vec![
      ssh_key_entry("alpha", AgentState::Loaded),
      ssh_key_entry("beta", AgentState::NotLoaded),
    ],
  };
  let rows = app.rows();
  assert!(row_of(&rows, Item::UnloadSshKey(0)).is_selectable());
  assert!(row_of(&rows, Item::LoadSshKey(1)).is_selectable());
  assert!(
    rows
      .iter()
      .all(|row| row.id() != Some(&Item::LoadSshKey(0)))
  );
  assert!(
    rows
      .iter()
      .all(|row| row.id() != Some(&Item::UnloadSshKey(1)))
  );
}

#[test]
fn ssh_keys_delete_is_a_destructive_row_that_only_asks_for_confirmation() {
  let mut app = app(Page::SshKeys);
  app.ssh_keys = SshKeys {
    available: true,
    entries: vec![ssh_key_entry("alpha", AgentState::NotLoaded)],
  };
  let rows = app.rows();
  assert_eq!(
    row_of(&rows, Item::DeleteSshKey(0)).kind(),
    RowKind::Destructive
  );
  // The delete row is the last selectable row, inside the Danger zone.
  press(&mut app, KeyCode::End);
  assert_eq!(app.selected_item(), Some(Item::DeleteSshKey(0)));
  press(&mut app, KeyCode::Enter);
  assert!(matches!(
    app.confirm,
    Some(PendingAction::DeleteSshKey(ref name)) if name == "alpha"
  ));
}

#[test]
fn ssh_keys_block_copies_the_public_key_before_load_and_both_rows_have_icons() {
  let mut app = app(Page::SshKeys);
  app.ssh_keys = SshKeys {
    available: true,
    entries: vec![ssh_key_entry("alpha", AgentState::NotLoaded)],
  };
  let rows = app.rows();
  let copy = row_of(&rows, Item::CopySshPublicKey(0));
  assert_eq!(copy.kind(), RowKind::Action);
  assert_eq!(copy.icon_glyph(), Some(argvus_tui::icons::COPY));
  assert_eq!(
    row_of(&rows, Item::DeleteSshKey(0)).icon_glyph(),
    Some(argvus_tui::icons::DELETE)
  );
  let position = |item: Item| rows.iter().position(|row| row.id() == Some(&item));
  assert!(position(Item::CopySshPublicKey(0)) < position(Item::LoadSshKey(0)));
}

#[test]
fn ssh_keys_form_rows_respond_to_enter() {
  let mut app = app(Page::SshKeys);
  app.ssh_keys = SshKeys {
    available: true,
    entries: vec![ssh_key_entry("alpha", AgentState::NotLoaded)],
  };
  assert_eq!(app.selected_item(), Some(Item::NewSshKeyName));
  press(&mut app, KeyCode::Enter);
  assert!(app.admin.editor.is_some(), "Enter on Name opens the editor");
  press(&mut app, KeyCode::Esc);
  press(&mut app, KeyCode::Down);
  assert_eq!(app.selected_item(), Some(Item::SshKeyType));
  press(&mut app, KeyCode::Enter);
  assert_eq!(app.new_ssh_key_type, crate::system::ssh_keys::KeyType::Rsa);
}

#[test]
fn ssh_keys_search_matches_name_or_comment_and_keeps_the_listing_index() {
  let mut app = app(Page::SshKeys);
  app.ssh_keys = SshKeys {
    available: true,
    entries: vec![
      ssh_key_entry("alpha", AgentState::NotLoaded),
      ssh_key_entry("beta", AgentState::NotLoaded),
    ],
  };
  // "beta@host" is the comment of the second pair, so only its block remains,
  // and its Delete row still points at index 1 of the full listing.
  app.search = "beta".into();
  let rows = app.rows();
  assert!(
    rows
      .iter()
      .any(|row| row.id() == Some(&Item::DeleteSshKey(1)))
  );
  assert!(
    rows
      .iter()
      .all(|row| row.id() != Some(&Item::DeleteSshKey(0)))
  );
}

#[test]
fn ssh_keys_action_bar_holds_refresh_and_unload_all() {
  let mut app = app(Page::SshKeys);
  app.ssh_keys = SshKeys {
    available: true,
    entries: vec![ssh_key_entry("alpha", AgentState::Loaded)],
  };
  assert!(app.has_action_bar());
  let actions: Vec<Option<Item>> = app
    .action_rows()
    .iter()
    .map(|row| row.id().copied())
    .collect();
  assert_eq!(
    actions,
    vec![Some(Item::RefreshSshKeys), Some(Item::UnloadAllSshKeys)]
  );
}

#[test]
fn ssh_keys_page_without_the_launcher_only_informs() {
  let mut app = app(Page::SshKeys);
  app.ssh_keys = SshKeys::default();
  let rows = app.rows();
  assert!(rows.iter().all(|row| row.id().is_none()));
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
  app.keyboard_draft = keyboard_draft::LayoutDraft::loaded(&app.keyboard_info);
  app.navigation.current_mut().menu = argvus_tui::menu::MenuState::default();
  app
}

#[test]
fn keyboard_space_and_enter_only_edit_the_draft_until_apply() {
  let mut app = keyboard_layout_app();
  press(&mut app, KeyCode::Down);
  assert_eq!(app.selected_item(), Some(Item::Layout(1)));
  press(&mut app, KeyCode::Char(' '));
  assert!(
    app.keyboard_draft.is_enabled("us"),
    "Space enables the layout"
  );
  press(&mut app, KeyCode::Enter);
  assert!(
    app.keyboard_draft.is_default("us"),
    "Enter makes it the default"
  );
  assert_eq!(
    app.keyboard_info.hypr_layout, "br",
    "the system state is untouched before Apply"
  );
  assert!(app.keyboard_layout_changed());
}

#[test]
fn keyboard_default_is_marked_and_apply_is_dimmed_until_a_change() {
  let mut app = keyboard_layout_app();
  let rows = app.rows();
  assert!(row_of(&rows, Item::Layout(0)).is_marked());
  assert!(!row_of(&rows, Item::Layout(1)).is_marked());
  let apply = |app: &App| app.action_rows()[0].clone();
  assert_eq!(apply(&app).id(), Some(&Item::ApplyKeyboardLayout));
  assert!(!apply(&app).is_selectable());
  press(&mut app, KeyCode::Down);
  press(&mut app, KeyCode::Char(' '));
  assert!(apply(&app).is_selectable());
}

#[test]
fn keyboard_apply_lives_in_the_action_bar_not_in_the_list() {
  let app = keyboard_layout_app();
  assert!(
    app
      .rows()
      .iter()
      .all(|row| row.id() != Some(&Item::ApplyKeyboardLayout)),
    "the list cursor never walks into Apply"
  );
  assert_eq!(app.action_rows().len(), 1);
}

#[test]
fn keyboard_tab_focuses_the_action_bar_only_when_apply_is_enabled() {
  let mut app = keyboard_layout_app();
  press(&mut app, KeyCode::Tab);
  assert_eq!(
    app.action_cursor(),
    None,
    "Apply is dimmed: nothing to focus"
  );
  press(&mut app, KeyCode::Down);
  press(&mut app, KeyCode::Char(' '));
  press(&mut app, KeyCode::Tab);
  assert_eq!(app.action_cursor(), Some(0), "Tab focuses Apply");
  press(&mut app, KeyCode::Up);
  assert_eq!(
    app.action_cursor(),
    None,
    "Up gives the focus back to the list"
  );
  assert_eq!(
    app.selected_item(),
    Some(Item::Layout(0)),
    "and moves the list"
  );
}

#[test]
fn keyboard_arrows_on_the_action_bar_do_not_move_the_list() {
  let mut app = keyboard_layout_app();
  press(&mut app, KeyCode::Down);
  press(&mut app, KeyCode::Char(' '));
  press(&mut app, KeyCode::Tab);
  let before = app.selected_item();
  press(&mut app, KeyCode::Right);
  assert_eq!(app.action_cursor(), Some(0), "one button: it stays focused");
  assert_eq!(app.selected_item(), before, "the list did not move");
}

#[test]
fn keyboard_arrows_do_not_toggle_layouts_on_the_list() {
  let mut app = keyboard_layout_app();
  press(&mut app, KeyCode::Down);
  let before = app.keyboard_draft.clone();
  press(&mut app, KeyCode::Right);
  press(&mut app, KeyCode::Left);
  assert_eq!(app.keyboard_draft, before, "arrows never change a layout");
  assert_eq!(app.action_cursor(), None, "the list keeps the focus");
}

#[test]
fn keyboard_slash_focuses_the_top_search_and_filters_the_list() {
  let mut app = keyboard_layout_app();
  assert!(app.search_on_top());
  press(&mut app, KeyCode::Char('/'));
  assert!(app.searching, "/ focuses the search box");
  press(&mut app, KeyCode::Char('u'));
  press(&mut app, KeyCode::Char('s'));
  assert_eq!(app.search, "us");
  let ids: Vec<Item> = app
    .rows()
    .iter()
    .filter_map(|row| row.id().copied())
    .collect();
  assert_eq!(ids, [Item::Layout(1)], "only the matching layout is listed");
  press(&mut app, KeyCode::Esc);
  assert!(!app.searching);
  assert!(app.search.is_empty(), "Esc clears the query");
}

#[test]
fn keyboard_last_enabled_layout_cannot_be_switched_off_in_the_draft() {
  let mut app = keyboard_layout_app();
  // `br` is the only enabled layout, so Space on it is refused.
  press(&mut app, KeyCode::Char(' '));
  assert!(app.keyboard_draft.is_enabled("br"));
  assert!(!app.keyboard_layout_changed());
}

#[test]
fn keyboard_draft_asks_before_leaving_with_pending_edits() {
  let mut app = keyboard_layout_app();
  press(&mut app, KeyCode::Down);
  press(&mut app, KeyCode::Char(' '));
  press(&mut app, KeyCode::Esc);
  assert!(matches!(app.confirm, Some(PendingAction::DiscardDraft)));
  assert_eq!(app.page(), Page::KeyboardLayout);
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
fn timezone_enter_only_edits_the_draft_until_apply() {
  let mut app = timezone_app();
  assert_eq!(
    app.selected_item(),
    Some(Item::Zone(1)),
    "the cursor starts on the system zone"
  );
  press(&mut app, KeyCode::Up);
  press(&mut app, KeyCode::Enter);
  assert_eq!(app.timezone_draft, "America/Sao_Paulo");
  assert_eq!(
    app.datetime.time_zone, "Europe/Lisbon",
    "the system zone is untouched before Apply"
  );
  assert!(app.timezone_changed());
}

#[test]
fn timezone_apply_is_dimmed_until_a_change() {
  let mut app = timezone_app();
  let apply = |app: &App| app.action_rows()[0].clone();
  assert_eq!(apply(&app).id(), Some(&Item::ApplyTimezone));
  assert!(!apply(&app).is_selectable());
  press(&mut app, KeyCode::Up);
  press(&mut app, KeyCode::Enter);
  assert!(apply(&app).is_selectable());
}

#[test]
fn timezone_apply_lives_in_the_action_bar_not_in_the_list() {
  let app = timezone_app();
  assert!(
    app
      .rows()
      .iter()
      .all(|row| row.id() != Some(&Item::ApplyTimezone)),
    "the list cursor never walks into Apply"
  );
  assert_eq!(app.action_rows().len(), 1);
}

#[test]
fn timezone_tab_focuses_apply_only_when_it_is_enabled() {
  let mut app = timezone_app();
  press(&mut app, KeyCode::Tab);
  assert_eq!(
    app.action_cursor(),
    None,
    "Apply is dimmed: nothing to focus"
  );
  press(&mut app, KeyCode::Up);
  press(&mut app, KeyCode::Enter);
  press(&mut app, KeyCode::Tab);
  assert_eq!(app.action_cursor(), Some(0), "Tab focuses Apply");
}

#[test]
fn timezone_slash_focuses_the_top_search_and_filters_the_list() {
  let mut app = timezone_app();
  assert!(app.search_on_top());
  press(&mut app, KeyCode::Char('/'));
  assert!(app.searching, "/ focuses the search box");
  for character in "sao".chars() {
    press(&mut app, KeyCode::Char(character));
  }
  let ids: Vec<Item> = app
    .rows()
    .iter()
    .filter_map(|row| row.id().copied())
    .collect();
  assert_eq!(ids, [Item::Zone(0)], "only the matching zone is listed");
}

#[test]
fn timezone_draft_asks_before_leaving_with_pending_edits() {
  let mut app = timezone_app();
  press(&mut app, KeyCode::Up);
  press(&mut app, KeyCode::Enter);
  press(&mut app, KeyCode::Esc);
  assert!(matches!(app.confirm, Some(PendingAction::DiscardDraft)));
  assert_eq!(app.page(), Page::TimeZone);
}

#[test]
fn timezone_discard_restores_the_system_zone_in_the_draft() {
  let mut app = timezone_app();
  press(&mut app, KeyCode::Up);
  press(&mut app, KeyCode::Enter);
  app.discard_and_leave();
  assert_eq!(app.timezone_draft, "Europe/Lisbon");
  assert!(!app.timezone_changed());
}

#[test]
fn timezone_arrows_on_the_list_move_the_focus_to_apply() {
  let mut app = timezone_app();
  press(&mut app, KeyCode::Up);
  press(&mut app, KeyCode::Enter);
  let before = app.selected_item();
  press(&mut app, KeyCode::Right);
  assert_eq!(app.action_cursor(), Some(0), "→ focuses Apply, as Tab does");
  assert_eq!(app.selected_item(), before, "the list did not move");
  press(&mut app, KeyCode::Up);
  assert_eq!(app.action_cursor(), None, "Up gives the focus back");
  press(&mut app, KeyCode::Left);
  assert_eq!(app.action_cursor(), Some(0), "← focuses Apply too");
}

#[test]
fn timezone_arrows_do_nothing_while_apply_is_dimmed() {
  let mut app = timezone_app();
  let before = app.selected_item();
  press(&mut app, KeyCode::Left);
  press(&mut app, KeyCode::Right);
  assert_eq!(app.action_cursor(), None, "nothing to focus");
  assert_eq!(app.selected_item(), before, "the list did not move");
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

#[test]
fn gpg_sections_start_expanded_and_collapse_on_enter() {
  use crate::item::GpgSection;

  let mut app = app(Page::GpgKeys);
  app.gpg_keys = crate::system::gpg_keys::GpgKeys {
    available: true,
    ..Default::default()
  };
  assert!(
    app
      .rows()
      .iter()
      .any(|row| row.id() == Some(&Item::GpgFullName))
  );
  assert!(
    app
      .rows()
      .iter()
      .any(|row| row.id() == Some(&Item::ImportGpgKey))
  );
  select_gpg(&mut app, Item::ToggleGpgSection(GpgSection::Generate));
  press(&mut app, KeyCode::Enter);
  assert!(
    app
      .rows()
      .iter()
      .all(|row| row.id() != Some(&Item::GpgFullName))
  );
  press(&mut app, KeyCode::Enter);
  assert!(
    app
      .rows()
      .iter()
      .any(|row| row.id() == Some(&Item::GpgFullName))
  );
}

#[test]
fn gpg_keys_page_offers_the_form_and_a_link_to_the_keyring_only() {
  let mut app = app(Page::GpgKeys);
  app.gpg_keys = crate::system::gpg_keys::GpgKeys {
    available: true,
    entries: vec![gpg_key_entry(
      "Alice",
      "AAAA111122223333AAAA111122223333AAAAAAAA",
    )],
    ..Default::default()
  };
  let rows = app.rows();
  assert!(rows.iter().any(|row| row.id() == Some(&Item::OpenGpgKeys)));
  assert!(
    rows
      .iter()
      .all(|row| row.id() != Some(&Item::ExportGpgKey(0))),
    "keys are listed on their own page"
  );
}

#[test]
fn gpg_keys_generate_stays_disabled_until_name_and_email_are_typed() {
  let mut app = app(Page::GpgKeys);
  open_gpg_section(&mut app, crate::item::GpgSection::Generate);
  app.gpg_keys = crate::system::gpg_keys::GpgKeys {
    available: true,
    ..Default::default()
  };
  let rows = app.rows();
  assert!(!row_of(&rows, Item::GenerateGpgKey).is_selectable());
  app.gpg_form.full_name = "Alice".into();
  let rows = app.rows();
  assert!(!row_of(&rows, Item::GenerateGpgKey).is_selectable());
  app.gpg_form.email = "alice@example.com".into();
  let rows = app.rows();
  assert!(row_of(&rows, Item::GenerateGpgKey).is_selectable());
}

#[test]
fn gpg_keys_generate_reports_a_bad_form_without_running() {
  let mut app = app(Page::GpgKeys);
  open_gpg_section(&mut app, crate::item::GpgSection::Generate);
  app.gpg_keys = crate::system::gpg_keys::GpgKeys {
    available: true,
    ..Default::default()
  };
  app.gpg_form.full_name = "Alice".into();
  app.gpg_form.email = "not-an-email".into();
  select_gpg(&mut app, Item::GenerateGpgKey);
  press(&mut app, KeyCode::Enter);
  assert!(
    app.confirm.is_none(),
    "a bad form never asks for confirmation"
  );
  assert!(app.gpg_job.is_none(), "a bad form never starts gpg");
}

#[test]
fn gpg_keys_generate_without_passphrase_asks_before_running() {
  let mut app = app(Page::GpgKeys);
  open_gpg_section(&mut app, crate::item::GpgSection::Generate);
  app.gpg_keys = crate::system::gpg_keys::GpgKeys {
    available: true,
    ..Default::default()
  };
  app.gpg_form.full_name = "Alice".into();
  app.gpg_form.email = "alice@example.com".into();
  app.gpg_form.protect_with_passphrase = false;
  select_gpg(&mut app, Item::GenerateGpgKey);
  press(&mut app, KeyCode::Enter);
  assert!(matches!(
    app.confirm,
    Some(PendingAction::GenerateGpgNoPassphrase)
  ));
  assert!(
    app.gpg_job.is_none(),
    "nothing runs before the confirmation"
  );
}

#[test]
fn gpg_keys_cancel_clears_every_field_of_the_form() {
  let mut app = app(Page::GpgKeys);
  open_gpg_section(&mut app, crate::item::GpgSection::Generate);
  app.gpg_keys = crate::system::gpg_keys::GpgKeys {
    available: true,
    ..Default::default()
  };
  app.gpg_form.full_name = "Alice".into();
  app.gpg_form.email = "alice@example.com".into();
  app.gpg_form.comment = "work".into();
  select_gpg(&mut app, Item::CancelGpgForm);
  press(&mut app, KeyCode::Enter);
  let order = crate::system::date::DateOrder::for_locale(&app.lang.locale());
  assert_eq!(
    app.gpg_form,
    crate::system::gpg_keys::GpgForm::new(order),
    "Cancel returns the form to its defaults"
  );
}

#[test]
fn gpg_keys_algorithm_and_size_rows_change_on_enter() {
  let mut app = app(Page::GpgKeys);
  open_gpg_section(&mut app, crate::item::GpgSection::Generate);
  app.gpg_keys = crate::system::gpg_keys::GpgKeys {
    available: true,
    ..Default::default()
  };
  select_gpg(&mut app, Item::GpgAlgorithm);
  press(&mut app, KeyCode::Enter);
  assert_eq!(
    app.gpg_form.algorithm,
    crate::system::gpg_keys::KeyAlgorithm::Dsa
  );
  assert_eq!(app.gpg_form.key_size, 2048, "DSA starts at 2048 bits");
  select_gpg(&mut app, Item::GpgKeySize);
  press(&mut app, KeyCode::Enter);
  assert_eq!(
    app.gpg_form.key_size, 3072,
    "DSA offers 3072 instead of 4096"
  );
}

#[test]
fn gpg_keys_protect_toggle_clears_the_passphrases_when_turned_off() {
  let mut app = app(Page::GpgFormPassphrase);
  app.gpg_keys = crate::system::gpg_keys::GpgKeys {
    available: true,
    ..Default::default()
  };
  app.gpg_form.passphrase = "correct horse".into();
  app.gpg_form.passphrase_confirm = "correct horse".into();
  select_gpg(&mut app, Item::GpgProtectPassphrase);
  press(&mut app, KeyCode::Enter);
  assert!(!app.gpg_form.protect_with_passphrase);
  assert!(app.gpg_form.passphrase.is_empty());
  assert!(app.gpg_form.passphrase_confirm.is_empty());
}

#[test]
fn gpg_keys_never_expires_turns_the_date_row_off() {
  let mut app = app(Page::GpgFormExpiration);
  app.gpg_keys = crate::system::gpg_keys::GpgKeys {
    available: true,
    ..Default::default()
  };
  assert!(row_of(&app.rows(), Item::GpgExpiry).is_selectable());
  select_gpg(&mut app, Item::GpgNeverExpires);
  press(&mut app, KeyCode::Enter);
  assert!(app.gpg_form.never_expires);
  assert!(!row_of(&app.rows(), Item::GpgExpiry).is_selectable());
}

#[test]
fn gpg_keys_export_opens_the_path_editor_for_the_selected_key() {
  let mut app = app(Page::GpgKeyList);
  app.gpg_keys = crate::system::gpg_keys::GpgKeys {
    available: true,
    entries: vec![gpg_key_entry(
      "Alice",
      "AAAA111122223333AAAA111122223333AAAAAAAA",
    )],
    ..Default::default()
  };
  select_gpg(&mut app, Item::ExportGpgKey(0));
  press(&mut app, KeyCode::Enter);
  assert!(
    app.admin.editor.is_some(),
    "Enter on Export opens the path editor"
  );
}

#[test]
fn gpg_keys_list_shows_only_information_without_focus() {
  let mut app = app(Page::GpgKeyList);
  app.gpg_keys = crate::system::gpg_keys::GpgKeys {
    available: true,
    entries: vec![gpg_key_entry(
      "Alice",
      "AAAA111122223333AAAA111122223333AAAAAAAA",
    )],
    ..Default::default()
  };
  let rows = app.rows();
  let info: Vec<_> = rows
    .iter()
    .filter(|row| row.kind() == RowKind::Info)
    .collect();
  assert!(!info.is_empty());
  assert!(info.iter().all(|row| !row.is_selectable()));
}

#[test]
fn gpg_keys_search_matches_the_identity_and_keeps_the_listing_index() {
  let mut app = app(Page::GpgKeyList);
  app.gpg_keys = crate::system::gpg_keys::GpgKeys {
    available: true,
    entries: vec![
      gpg_key_entry(
        "Alice <alice@example.com>",
        "AAAA111122223333AAAA111122223333AAAAAAAA",
      ),
      gpg_key_entry(
        "Bob <bob@example.com>",
        "CCCC111122223333CCCC111122223333CCCCCCCC",
      ),
    ],
    ..Default::default()
  };
  app.search = "bob".into();
  let rows = app.rows();
  assert!(
    rows
      .iter()
      .any(|row| row.id() == Some(&Item::ExportGpgKey(1)))
  );
  assert!(
    rows
      .iter()
      .all(|row| row.id() != Some(&Item::ExportGpgKey(0)))
  );
}

#[test]
fn gpg_keys_without_the_launcher_only_informs() {
  let app = app(Page::GpgKeys);
  let rows = app.rows();
  assert!(rows.iter().all(|row| row.id().is_none()));
}

#[test]
fn gpg_key_list_keeps_the_search_on_top_without_an_action_bar() {
  let mut app = app(Page::GpgKeyList);
  app.gpg_keys = crate::system::gpg_keys::GpgKeys {
    available: true,
    entries: vec![
      gpg_key_entry(
        "Alice <alice@example.com>",
        "AAAA111122223333AAAA111122223333AAAAAAAA",
      ),
      gpg_key_entry(
        "Bob <bob@example.com>",
        "CCCC111122223333CCCC111122223333CCCCCCCC",
      ),
    ],
    ..Default::default()
  };
  assert!(app.search_on_top());
  assert!(!app.has_action_bar());
  app.begin_search();
  for character in "bob".chars() {
    app.push_search(character);
  }
  let rows = app.rows();
  assert!(
    rows
      .iter()
      .any(|row| row.id() == Some(&Item::ExportGpgKey(1)))
  );
  assert!(
    rows
      .iter()
      .all(|row| row.id() != Some(&Item::ExportGpgKey(0)))
  );
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
fn gpg_key_block_offers_edit_copy_export_and_delete() {
  let mut app = app(Page::GpgKeyList);
  gpg_keys_with_two_secrets(&mut app);
  let rows = app.rows();
  for item in [
    Item::OpenGpgKeyEdit(0),
    Item::ExportGpgPrivate(0),
    Item::CopyGpgPublic(0),
    Item::CopyGpgFingerprint(0),
    Item::DeleteGpgKey(0),
  ] {
    assert!(
      rows.iter().any(|row| row.id() == Some(&item)),
      "missing {item:?}"
    );
  }
  assert_eq!(
    row_of(&rows, Item::DeleteGpgKey(0)).kind(),
    RowKind::Destructive
  );
}

#[test]
fn a_public_only_key_offers_no_private_export() {
  let mut app = app(Page::GpgKeyList);
  app.gpg_keys = crate::system::gpg_keys::GpgKeys {
    available: true,
    entries: vec![crate::system::gpg_keys::GpgKeyEntry {
      secret: false,
      ..gpg_key_entry("Carol <carol@example.com>", FP_FIRST)
    }],
    ..Default::default()
  };
  let rows = app.rows();
  assert!(
    rows
      .iter()
      .all(|row| row.id() != Some(&Item::ExportGpgPrivate(0)))
  );
}

#[test]
fn deleting_a_key_asks_before_it_runs() {
  let mut app = app(Page::GpgKeyList);
  gpg_keys_with_two_secrets(&mut app);
  select_gpg(&mut app, Item::DeleteGpgKey(0));
  press(&mut app, KeyCode::Enter);
  assert!(matches!(
    app.confirm,
    Some(PendingAction::DeleteGpgKey(ref fingerprint)) if fingerprint == FP_FIRST
  ));
  assert!(
    app.gpg_action.is_none(),
    "nothing runs before the confirmation"
  );
}

#[test]
fn the_edit_page_follows_the_fingerprint_of_the_key() {
  let mut app = app(Page::GpgKeyList);
  gpg_keys_with_two_secrets(&mut app);
  select_gpg(&mut app, Item::OpenGpgKeyEdit(1));
  press(&mut app, KeyCode::Enter);
  assert_eq!(app.page(), Page::GpgKeyEdit);
  assert_eq!(app.gpg_edit_fingerprint.as_deref(), Some(FP_SECOND));
  assert!(
    app
      .rows()
      .iter()
      .any(|row| row.id() == Some(&Item::GpgEditPasswd))
  );
}

#[test]
fn the_edit_page_of_a_vanished_key_offers_no_action() {
  let mut app = app(Page::GpgKeyEdit);
  gpg_keys_with_two_secrets(&mut app);
  app.gpg_edit_fingerprint = Some("FFFF000000000000FFFF000000000000FFFF0000".into());
  let rows = app.rows();
  assert!(rows.iter().all(|row| row.id().is_none()));
}

#[test]
fn a_public_only_key_cannot_change_its_passphrase() {
  let mut app = app(Page::GpgKeyEdit);
  app.gpg_keys = crate::system::gpg_keys::GpgKeys {
    available: true,
    entries: vec![crate::system::gpg_keys::GpgKeyEntry {
      secret: false,
      ..gpg_key_entry("Carol <carol@example.com>", FP_FIRST)
    }],
    ..Default::default()
  };
  app.gpg_edit_fingerprint = Some(FP_FIRST.into());
  assert!(!row_of(&app.rows(), Item::GpgEditPasswd).is_selectable());
}

#[test]
fn sign_choices_are_the_other_keys_with_a_private_part() {
  let mut app = app(Page::GpgEditSign);
  gpg_keys_with_two_secrets(&mut app);
  app.gpg_edit_fingerprint = Some(FP_FIRST.into());
  let rows = app.rows();
  assert!(
    rows
      .iter()
      .any(|row| row.id() == Some(&Item::GpgSignWith(1)))
  );
  assert!(
    rows
      .iter()
      .all(|row| row.id() != Some(&Item::GpgSignWith(0)))
  );
}

#[test]
fn revoking_asks_for_a_reason_and_confirms_it() {
  let mut app = app(Page::GpgEditRevoke);
  gpg_keys_with_two_secrets(&mut app);
  app.gpg_edit_fingerprint = Some(FP_FIRST.into());
  select_gpg(
    &mut app,
    Item::GpgRevokeWith(crate::system::gpg_keys::RevokeReason::Unused),
  );
  press(&mut app, KeyCode::Enter);
  assert!(matches!(
    app.confirm,
    Some(PendingAction::RevokeGpgKey(ref fingerprint, crate::system::gpg_keys::RevokeReason::Unused))
      if fingerprint == FP_FIRST
  ));
  assert!(app.gpg_action.is_none());
}

#[test]
fn an_expiry_in_the_past_is_refused_before_gpg_runs() {
  let mut app = app(Page::GpgEditExpiration);
  gpg_keys_with_two_secrets(&mut app);
  app.gpg_edit_fingerprint = Some(FP_FIRST.into());
  app.gpg_edit_never = false;
  app.gpg_edit_expiry = "2001/01/01".into();
  select_gpg(&mut app, Item::ApplyGpgExpiry);
  press(&mut app, KeyCode::Enter);
  assert!(app.gpg_action.is_none(), "a past date never starts gpg");
}

#[test]
fn the_main_page_offers_the_private_key_import_next_to_the_public_one() {
  let mut app = app(Page::GpgKeys);
  open_gpg_section(&mut app, crate::item::GpgSection::Import);
  app.gpg_keys = crate::system::gpg_keys::GpgKeys {
    available: true,
    ..Default::default()
  };
  select_gpg(&mut app, Item::ImportGpgPrivate);
  press(&mut app, KeyCode::Enter);
  assert!(matches!(app.confirm, Some(PendingAction::ImportGpgPrivate)));
}

#[test]
fn a_launcher_older_than_the_page_offers_no_key_action() {
  let mut edit = app(Page::GpgKeyEdit);
  let mut app = app(Page::GpgKeyList);
  app.gpg_keys = crate::system::gpg_keys::GpgKeys {
    available: true,
    outdated: true,
    entries: vec![gpg_key_entry("Alice", FP_FIRST)],
    ..Default::default()
  };
  let rows = app.rows();
  assert!(rows.iter().all(|row| row.id().is_none()));
  edit.gpg_keys = app.gpg_keys.clone();
  edit.gpg_edit_fingerprint = Some(FP_FIRST.into());
  assert!(edit.rows().iter().all(|row| row.id().is_none()));
}

#[test]
fn exports_propose_the_fingerprint_as_the_file_name() {
  let mut app = app(Page::GpgKeyList);
  gpg_keys_with_two_secrets(&mut app);
  select_gpg(&mut app, Item::ExportGpgKey(0));
  press(&mut app, KeyCode::Enter);
  let public = app
    .admin
    .editor
    .take()
    .expect("public export opens the path editor");
  assert!(
    public.text().ends_with(&format!("/{FP_FIRST}.asc")),
    "{}",
    public.text()
  );

  app.open_confirm(PendingAction::ExportGpgPrivate(FP_FIRST.into()));
  press(&mut app, KeyCode::Char('y'));
  let private = app
    .admin
    .editor
    .take()
    .expect("private export opens the path editor");
  assert!(
    private
      .text()
      .ends_with(&format!("/{FP_FIRST}-private.asc")),
    "{}",
    private.text()
  );
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
      "Central de Controle > Administração do sistema > Hostname",
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
