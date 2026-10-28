//! Package lists: filters, search, details, orphans, updates, cache, downgrades and history.

use super::*;

#[test]
fn updates_show_the_filter_on_top_and_the_action_bar_below() {
  let mut app = app();
  app.page = OFFICIAL_UPDATE;
  app.updates.push(update("linux"));
  let text = screen(&mut app, 90, 25);
  assert!(text.contains("Search:"), "{text}");
  assert!(text.contains("Actions"), "{text}");
  assert!(text.contains("Upgrade all"), "{text}");
  assert!(text.contains("Refresh database"), "{text}");
  assert!(text.contains("linux"), "{text}");
  assert!(
    !app
      .rows()
      .iter()
      .any(|row| row.id() == Some(&Item::UpgradeAll))
  );
}

#[test]
fn filtered_lists_show_the_search_box_on_top_and_the_results_block() {
  for page in [
    OFFICIAL_INSTALL,
    AUR_INSTALL,
    OFFICIAL_REMOVE,
    AUR_REMOVE,
    PackagesPage::Orphans,
  ] {
    let mut app = app();
    app.page = page;
    app.packages = vec![package("linux", true)];
    app.aur = vec![AurPackage {
      name: "linux".into(),
      version: "1.0".into(),
      ..Default::default()
    }];
    app.handle(KeyCode::Char('l'));
    let text = screen(&mut app, 90, 25);
    assert!(text.contains("Search: l▌"), "{page:?}\n{text}");
    assert!(text.contains("┌Results"), "{page:?}\n{text}");
    assert!(text.contains("linux"), "{page:?}\n{text}");
    assert!(!text.contains("Actions"), "{page:?}\n{text}");
    // The filter lives in the box; the list has no read-only search row.
    assert!(!text.contains("l_"), "{page:?}\n{text}");
  }
}

#[test]
fn empty_filtered_lists_say_why() {
  let mut app = app();
  app.page = OFFICIAL_REMOVE;
  app.packages = vec![package("linux", true)];
  app.handle(KeyCode::Char('z'));
  assert!(screen(&mut app, 90, 25).contains("No matching items"));
  app.page = OFFICIAL_INSTALL;
  app.packages.clear();
  app.query.clear();
  assert!(screen(&mut app, 90, 25).contains("Press / to search packages"));
}

#[test]
fn tab_moves_between_the_update_list_and_its_action_bar() {
  let mut app = app();
  app.page = OFFICIAL_UPDATE;
  app.updates.push(update("linux"));
  assert_eq!(app.actions.cursor(), None);
  app.handle(KeyCode::Tab);
  assert_eq!(app.actions.cursor(), Some(0));
  app.handle(KeyCode::Right);
  assert_eq!(app.actions.cursor(), Some(1));
  app.handle(KeyCode::Left);
  assert_eq!(app.actions.cursor(), Some(0));
  app.handle(KeyCode::Up);
  assert_eq!(app.actions.cursor(), None);
  app.handle(KeyCode::Tab);
  assert_eq!(app.actions.cursor(), Some(0));
  app.handle(KeyCode::BackTab);
  assert_eq!(app.actions.cursor(), None);
  // On the list, `→` opens the update's package instead of the bar.
  app.handle(KeyCode::Right);
  assert_eq!(app.page, PackagesPage::Details);
  assert_eq!(app.details_target().as_deref(), Some("linux"));
  assert_eq!(app.actions.cursor(), None);
  app.handle(KeyCode::Esc);
  assert_eq!(app.page, OFFICIAL_UPDATE);
  assert_eq!(app.menu.selected_id(&app.rows()), Some(Item::Update(0)));
}

#[test]
fn tab_skips_the_disabled_upgrade_all_without_updates() {
  let mut app = app();
  app.page = OFFICIAL_UPDATE;
  app.handle(KeyCode::Tab);
  assert_eq!(app.actions.cursor(), Some(1));
  app.handle(KeyCode::Enter);
  assert_eq!(app.pending, Some(Action::RefreshDatabase));
}

#[test]
fn typing_on_the_updates_action_bar_goes_back_to_the_filter() {
  let mut app = app();
  app.page = OFFICIAL_UPDATE;
  app.updates = vec![update("linux"), update("mesa")];
  app.handle(KeyCode::Tab);
  assert!(app.actions.cursor().is_some());
  app.handle(KeyCode::Char('m'));
  assert_eq!(app.actions.cursor(), None);
  assert_eq!(app.query, "m");
  let names: Vec<_> = app
    .visible_updates()
    .iter()
    .map(|(_, u)| u.name.clone())
    .collect();
  assert_eq!(names, ["mesa"]);
  app.handle(KeyCode::Tab);
  app.handle(KeyCode::Backspace);
  assert_eq!(app.actions.cursor(), None);
  assert!(app.query.is_empty());
}

#[test]
fn task_footers_name_the_task_the_details_and_the_action_bar() {
  let mut app = app();
  app.page = OFFICIAL_UPDATE;
  app.updates.push(update("linux"));
  let footer = app.footer_hints(&app.rows());
  assert!(footer.contains("Enter Update"), "{footer}");
  assert!(footer.contains("→ Details"), "{footer}");
  assert!(footer.contains("Tab Actions"), "{footer}");
  assert!(footer.contains("←/Esc Back"), "{footer}");
  assert!(!footer.contains("→/Enter"), "{footer}");
  app.handle(KeyCode::Tab);
  let footer = app.footer_hints(&app.rows());
  assert!(footer.contains("Enter Run"), "{footer}");
  assert!(!footer.contains("Tab Actions"), "{footer}");

  app.go(OFFICIAL_REMOVE);
  app.packages.push(package("zsh", true));
  let footer = app.footer_hints(&app.rows());
  assert!(footer.contains("→/Enter Open"), "the Orphans row: {footer}");
  app.handle(KeyCode::Down);
  let footer = app.footer_hints(&app.rows());
  assert!(footer.contains("Enter Remove"), "{footer}");
  assert!(footer.contains("→ Details"), "{footer}");
  assert!(!footer.contains("Tab"), "{footer}");
}

#[test]
fn empty_updates_say_why_and_keep_upgrade_all_disabled() {
  let mut app = app();
  app.page = OFFICIAL_UPDATE;
  let text = screen(&mut app, 90, 25);
  assert!(text.contains("No pending updates"), "{text}");
  assert!(!row(&app.update_action_rows(), Item::UpgradeAll).is_selectable());
  app.updates.push(update("linux"));
  app.handle(KeyCode::Char('z'));
  let text = screen(&mut app, 90, 25);
  assert!(text.contains("No matching items"), "{text}");
  assert!(!row(&app.update_action_rows(), Item::UpgradeAll).is_selectable());
}

#[test]
fn typing_filters_lists_and_keeps_letters_as_text() {
  let mut app = app();
  app.page = OFFICIAL_REMOVE;
  app.packages = vec![package("firefox", true), package("jq", true)];
  for key in ['j', 'q'] {
    app.handle(KeyCode::Char(key));
  }
  assert_eq!(app.query, "jq");
  assert!(app.captures_text());
  let rows = app.rows();
  let entries = rows
    .iter()
    .filter(|row| row.id().is_some_and(|item| item.is_list_entry()))
    .count();
  assert_eq!(entries, 1);
  assert_eq!(app.menu.selected_id(&rows), Some(Item::Package(1)));
  app.handle(KeyCode::Backspace);
  app.handle(KeyCode::Char('r'));
  assert_eq!(app.query, "jr");
  assert!(app.job.is_none(), "r is text on the filtered lists");
}

#[test]
fn empty_search_does_not_create_package_action() {
  let mut app = app();
  app.page = OFFICIAL_INSTALL;
  app.handle(KeyCode::Tab);
  app.handle(KeyCode::Enter);
  assert!(app.pending.is_none());
  app.handle(KeyCode::Right);
  app.handle(KeyCode::Enter);
  assert!(app.pending.is_none());
  assert!(app.plan.is_none());
}

#[test]
fn right_opens_the_details_and_back_returns_to_the_package() {
  let mut app = app();
  app.page = OFFICIAL_INSTALL;
  app.packages = vec![package("firefox", false), package("zsh", false)];
  app.handle(KeyCode::Down);
  app.handle(KeyCode::Right);
  assert_eq!(app.page, PackagesPage::Details);
  assert_eq!(app.details_target().as_deref(), Some("zsh"));
  app.handle(KeyCode::Esc);
  assert_eq!(app.page, OFFICIAL_INSTALL);
  assert_eq!(app.menu.selected_id(&app.rows()), Some(Item::Package(1)));
}

#[test]
fn enter_runs_the_task_of_each_list() {
  let mut app = app();
  app.page = OFFICIAL_INSTALL;
  app.packages = vec![package("zsh", false)];
  press(&mut app, Item::Package(0));
  assert!(app.plan.is_some(), "installing plans the transaction first");
  assert_eq!(
    app.page, OFFICIAL_INSTALL,
    "Enter does not open the details"
  );

  for page in [OFFICIAL_REMOVE, AUR_REMOVE] {
    let mut app = aur_app();
    app.page = page;
    app.packages = vec![package("zsh", true)];
    press(&mut app, Item::Package(0));
    assert!(
      app.plan.is_some(),
      "{page:?}: removing plans the transaction"
    );
  }

  for (page, action) in [
    (OFFICIAL_UPDATE, Action::UpgradePackage("linux".into())),
    (AUR_UPDATE, Action::AurUpgradePackage("linux".into())),
  ] {
    let mut app = aur_app();
    app.page = page;
    app.updates = vec![update("linux")];
    press(&mut app, Item::Update(0));
    assert_eq!(app.pending, Some(action), "{page:?}");
  }
}

#[test]
fn official_remove_starts_with_the_orphans_page() {
  let mut app = app();
  app.page = OFFICIAL_REMOVE;
  app.packages = vec![package("zsh", true)];
  app.normalize();
  let rows = app.rows();
  assert_eq!(rows[0].id(), Some(&Item::Open(PackagesPage::Orphans)));
  assert_eq!(rows[0].kind(), RowKind::Submenu);
  assert_eq!(rows[0].icon_glyph(), Some(icons::CLEAN));
  assert_eq!(
    app.menu.selected_id(&rows),
    Some(Item::Open(PackagesPage::Orphans))
  );
  // The installed list is still loading: its result must not fill Orphans.
  app.reload_force();
  assert!(app.job.is_some());
  app.handle(KeyCode::Enter);
  assert_eq!(app.page, PackagesPage::Orphans);
  assert!(app.packages.is_empty(), "the orphans load from scratch");
  assert_eq!(
    app.reload_key(),
    "Orphans||",
    "the Orphans load replaced it"
  );
  app.job = None;
  app.handle(KeyCode::Esc);
  assert_eq!(app.page, OFFICIAL_REMOVE);
  assert_eq!(
    app.menu.selected_id(&app.rows()),
    Some(Item::Open(PackagesPage::Orphans))
  );
  // AUR Remove has no Orphans row.
  app.page = AUR_REMOVE;
  assert!(
    !app
      .rows()
      .iter()
      .any(|row| row.id() == Some(&Item::Open(PackagesPage::Orphans)))
  );
}

#[test]
fn aur_update_bar_upgrades_every_aur_package_with_the_helper() {
  let mut app = aur_app();
  app.page = AUR_UPDATE;
  app.updates = vec![update("yay-bin")];
  let rows = app.update_action_rows();
  assert_eq!(rows.len(), 1, "the helper refreshes its own metadata");
  assert_eq!(rows[0].label(), "Upgrade all (AUR)");
  press_action(&mut app, Item::UpgradeAll);
  assert_eq!(app.pending, Some(Action::AurUpgrade));
  assert!(app.plan.is_none());
}

#[test]
fn search_pages_do_not_spawn_until_a_valid_query_is_confirmed() {
  let mut app = app();
  app.page = OFFICIAL_INSTALL;
  app.reload();
  assert!(app.job.is_none());
  app.page = AUR_INSTALL;
  app.reload();
  assert!(app.job.is_none());
  app.handle(KeyCode::Char('/'));
  app.handle(KeyCode::Char('a'));
  app.handle(KeyCode::Enter);
  assert!(app.job.is_none());
  assert!(app.status.as_ref().unwrap().text.contains("at least 2"));
  app.handle(KeyCode::Char('a'));
  app.handle(KeyCode::Enter);
  assert!(app.job.is_none());
  assert!(app.status.as_ref().unwrap().text.contains("at least 2"));
}

#[test]
fn cancelling_package_search_clears_the_query() {
  let mut app = app();
  app.page = OFFICIAL_INSTALL;
  app.query = "firefox".into();
  app.input = Some(app.query.clone());
  app.input_mode = Some(InputMode::Search);
  app.handle(KeyCode::Esc);
  assert!(app.query.is_empty());
  assert!(app.input.is_none());
}

#[test]
fn opening_package_details_keeps_the_selected_name_until_metadata_arrives() {
  let mut app = app();
  app.page = OFFICIAL_INSTALL;
  app.packages.push(package("firefox", false));
  open_details_of(&mut app, Item::Package(0));
  assert!(app.details.is_none());
  assert_eq!(app.details_target().as_deref(), Some("firefox"));
  assert_eq!(app.reload_key(), "Details||firefox");
}

#[test]
fn package_details_render_actions_sections_and_danger_zone() {
  let mut app = app();
  app.page = PackagesPage::Details;
  app.details = Some(PackageDetails {
    package: Package {
      name: "firefox".into(),
      version: "155.0.1-1".into(),
      repository: Some("extra".into()),
      installed: true,
      ..Default::default()
    },
    architecture: Some("x86_64".into()),
    url: Some("https://www.mozilla.org".into()),
    installed_size: Some(87_654_321),
    dependencies: vec!["gtk4".into(), "libx11".into()],
    ..Default::default()
  });
  let rows = app.rows();
  let titles = rows
    .iter()
    .filter(|row| row.is_section())
    .map(Row::label)
    .collect::<Vec<_>>();
  assert_eq!(
    titles,
    [
      "control_center.section_actions",
      "control_center.section_package",
      "control_center.section_source",
      "control_center.section_dependencies",
      "control_center.danger_zone",
    ]
    .map(|key| tr(app.lang, key))
  );
  assert_eq!(row(&rows, Item::Install).label(), "Reinstall");
  assert_eq!(row(&rows, Item::Install).icon_glyph(), Some(icons::RESTART));
  let remove = rows.last().unwrap();
  assert_eq!(remove.kind(), RowKind::Destructive);
  assert!(remove.is_selectable());
  let info = |label: &str| {
    rows
      .iter()
      .find(|row| row.kind() == RowKind::Info && row.label() == label)
      .and_then(Row::detail_text)
      .map(str::to_owned)
  };
  assert_eq!(info("Status").as_deref(), Some("Installed"));
  assert_eq!(info("Architecture").as_deref(), Some("x86_64"));
  assert_eq!(info("Depends on").as_deref(), Some("gtk4 libx11"));
  assert!(rows.iter().all(|row| !row.label().contains("Tab")));
}

#[test]
fn install_and_remove_follow_the_open_package() {
  let mut app = app();
  app.page = OFFICIAL_INSTALL;
  app.packages.push(package("firefox", false));
  open_details_of(&mut app, Item::Package(0));
  let rows = app.rows();
  assert_eq!(row(&rows, Item::Install).label(), "Install");
  assert!(
    !row(&rows, Item::Remove).is_selectable(),
    "an uninstalled package cannot be removed"
  );
  // The details load runs; package operations wait for it, as before.
  press(&mut app, Item::Install);
  assert!(app.plan.is_none());
  app.job = None;
  press(&mut app, Item::Install);
  assert!(app.plan.is_some(), "installing plans the transaction first");

  let mut installed = PackagesApp::new(
    Lang::for_locale("en-US"),
    Theme::load(),
    Capabilities::default(),
  );
  installed.page = OFFICIAL_REMOVE;
  installed.packages.push(package("zsh", true));
  open_details_of(&mut installed, Item::Package(0));
  installed.job = None;
  let rows = installed.rows();
  assert_eq!(row(&rows, Item::Install).label(), "Reinstall");
  press(&mut installed, Item::Remove);
  assert!(installed.plan.is_some());
}

#[test]
fn aur_rows_open_and_install_the_filtered_package() {
  let mut app = app();
  app.page = AUR_INSTALL;
  app.aur = vec![
    AurPackage {
      name: "yay-bin".into(),
      ..Default::default()
    },
    AurPackage {
      name: "paru-bin".into(),
      installed: true,
      ..Default::default()
    },
  ];
  for key in "paru".chars() {
    app.handle(KeyCode::Char(key));
  }
  app.handle(KeyCode::Right);
  assert_eq!(app.page, PackagesPage::Details);
  assert_eq!(app.details_target().as_deref(), Some("paru-bin"));
  assert_eq!(app.details_parent, AUR_INSTALL);
  assert!(
    app.job.is_none(),
    "AUR results are not in the local database"
  );
  app.handle(KeyCode::Esc);
  assert_eq!(app.menu.selected_id(&app.rows()), Some(Item::Aur(1)));
  // An installed AUR package is rebuilt by the helper (D58).
  app.handle(KeyCode::Enter);
  assert_eq!(app.pending, Some(Action::AurInstall("paru-bin".into())));
}

#[test]
fn space_marks_orphans_and_the_marks_follow_the_filter() {
  let mut app = orphans_app();
  let rows = app.rows();
  assert!(!row(&rows, Item::RemoveMarked).is_selectable());
  for key in "lib".chars() {
    app.handle(KeyCode::Char(key));
  }
  app.handle(KeyCode::Down);
  app.handle(KeyCode::Char(' '));
  assert_eq!(app.marked, ["libbaz"]);
  assert_eq!(app.query, "lib", "Space marks instead of typing");
  let rows = app.rows();
  assert_eq!(
    row(&rows, Item::Orphan(2)).kind(),
    RowKind::Toggle { on: true }
  );
  assert_eq!(
    row(&rows, Item::RemoveMarked).label(),
    format!("{} (1)", tr(app.lang, "control_center.remove_marked"))
  );
  let text = screen(&mut app, 90, 25);
  assert!(text.contains("[x]"), "{text}");
  let mark = format!("Space {}", tr(app.lang, "control_center.mark"));
  assert!(text.contains(&mark), "{text}");
  press(&mut app, Item::RemoveMarked);
  assert!(app.plan.is_some(), "the marked set is planned for removal");
}

#[test]
fn enter_on_an_orphan_opens_its_details() {
  let mut app = orphans_app();
  app.handle(KeyCode::Enter);
  assert_eq!(app.page, PackagesPage::Details);
  assert!(app.marked.is_empty());
  app.handle(KeyCode::Esc);
  assert_eq!(app.menu.selected_id(&app.rows()), Some(Item::Orphan(0)));
}

#[test]
fn reloading_orphans_keeps_only_the_marks_still_listed() {
  let mut app = orphans_app();
  app.marked = vec!["libfoo".into(), "gone".into()];
  app.apply(Loaded::Packages(vec![package("libfoo", true)]));
  assert_eq!(app.marked, ["libfoo"]);
}

#[test]
fn update_rows_confirm_one_package_or_plan_all() {
  let mut app = app();
  app.page = OFFICIAL_UPDATE;
  app.updates.push(Update {
    name: "linux".into(),
    current: "1".into(),
    available: "2".into(),
    ..Default::default()
  });
  press(&mut app, Item::Update(0));
  assert_eq!(app.pending, Some(Action::UpgradePackage("linux".into())));
  app.handle(KeyCode::Esc);
  assert!(app.pending.is_none());
  press_action(&mut app, Item::RefreshDatabase);
  assert_eq!(app.pending, Some(Action::RefreshDatabase));
  app.handle(KeyCode::Esc);
  press_action(&mut app, Item::UpgradeAll);
  assert!(app.plan.is_some());
}

#[test]
fn upgrade_all_is_disabled_without_updates() {
  let mut app = app();
  app.page = OFFICIAL_UPDATE;
  let rows = app.update_action_rows();
  assert!(!row(&rows, Item::UpgradeAll).is_selectable());
  assert!(row(&rows, Item::RefreshDatabase).is_selectable());
}

#[test]
fn cache_page_summarizes_and_cleans_from_the_danger_zone() {
  let mut app = app();
  app.page = PackagesPage::Cache;
  app.cache = vec![CachePackage {
    name: "firefox-155.0.1-1-x86_64.pkg.tar.zst".into(),
    bytes: 2_621_440,
    ..Default::default()
  }];
  let rows = app.rows();
  assert_eq!(rows.last().unwrap().id(), Some(&Item::CleanUninstalled));
  for (item, policy) in [
    (Item::CleanKeepThree, "keep-three"),
    (Item::CleanKeepOne, "keep-one"),
    (Item::CleanUninstalled, "uninstalled"),
  ] {
    assert_eq!(row(&rows, item).kind(), RowKind::Destructive);
    press(&mut app, item);
    assert_eq!(app.pending, Some(Action::CleanCache(policy)));
    app.handle(KeyCode::Esc);
  }
  press(&mut app, Item::CacheFiles);
  assert_eq!(app.page, PackagesPage::CacheFiles);
  let files = app.rows();
  assert_eq!(files[0].label(), "firefox-155.0.1-1-x86_64");
  assert_eq!(files[0].detail_text(), Some("2 MiB"));
  assert!(files.iter().all(|row| !row.is_selectable()));
  app.handle(KeyCode::Esc);
  assert_eq!(app.page, PackagesPage::Cache);
  assert_eq!(app.menu.selected_id(&app.rows()), Some(Item::CacheFiles));
}

#[test]
fn downgrade_rows_confirm_the_cached_file() {
  let mut app = app();
  app.page = PackagesPage::Downgrade;
  app.cache = vec![CachePackage {
    name: "zsh".into(),
    version: "5.9-1".into(),
    path: "/var/cache/pacman/pkg/zsh-5.9-1-x86_64.pkg.tar.zst".into(),
    ..Default::default()
  }];
  let rows = app.rows();
  assert_eq!(rows[0].kind(), RowKind::Destructive);
  assert_eq!(rows[0].label(), "zsh 5.9-1");
  app.handle(KeyCode::Enter);
  assert_eq!(
    app.pending,
    Some(Action::Downgrade(
      "/var/cache/pacman/pkg/zsh-5.9-1-x86_64.pkg.tar.zst".into()
    ))
  );
}

#[test]
fn history_is_read_only_and_has_no_cursor() {
  let mut app = app();
  app.page = PackagesPage::History;
  app.history = vec![HistoryEntry {
    timestamp: "2026-10-05T10:00".into(),
    action: "upgraded".into(),
    package: "linux".into(),
    old_version: Some("1".into()),
    new_version: Some("2".into()),
  }];
  let rows = app.rows();
  assert!(rows.iter().all(|row| !row.is_selectable()));
  assert_eq!(rows[0].detail_text(), Some("1 → 2"));
  app.handle(KeyCode::Enter);
  assert_eq!(app.page, PackagesPage::History);
}
