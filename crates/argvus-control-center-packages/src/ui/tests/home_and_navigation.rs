//! Packages home, source menus, breadcrumbs, footers and layout.

use super::*;

#[test]
fn package_home_rows_are_submenus_with_their_counters() {
  let mut app = app();
  app.dashboard = PackageDashboard {
    installed_count: 1200,
    foreign_count: 20,
    update_count: 7,
    orphan_count: 3,
    cache_count: 12,
    cache_bytes: 240_057_409_536,
    history_count: 40,
    mirrors_total: 6,
    mirrors_enabled: 4,
    available_count: 18_500,
    aur_helper: None,
    ..Default::default()
  };
  let rows = app.home_rows();
  assert_eq!(rows.len(), 5);
  assert!(rows.iter().all(|row| row.kind() == RowKind::Submenu));
  assert_eq!(
    rows.iter().map(Row::icon_glyph).collect::<Vec<_>>(),
    [
      icons::PACKAGES,
      icons::DATABASE,
      icons::HISTORY,
      icons::DOWNGRADE,
      icons::NETWORK
    ]
    .map(Some)
  );
  assert_eq!(rows[0].label(), "Official Repositories");
  // Without the AUR menu, its packages stay on the Official Remove list.
  assert_eq!(rows[0].detail_text(), Some("1200 packages · 7 pending"));
  assert_eq!(rows[1].detail_text(), Some("12 files · 223.6 GiB"));
  assert_eq!(rows[4].detail_text(), Some("4/6 active"));
}

#[test]
fn aur_appears_on_the_home_only_with_a_helper() {
  let mut app = aur_app();
  app.dashboard = PackageDashboard {
    installed_count: 1200,
    foreign_count: 20,
    aur_helper: Some("paru".into()),
    ..Default::default()
  };
  let rows = app.home_rows();
  assert_eq!(
    rows[1].id(),
    Some(&Item::Open(PackagesPage::Source(PackageSource::Aur)))
  );
  assert_eq!(rows[1].label(), "AUR (Arch User Repository)");
  assert_eq!(rows[1].icon_glyph(), Some(icons::USERS));
  assert_eq!(rows[1].detail_text(), Some("paru · 20 packages"));
  assert_eq!(rows[0].detail_text(), Some("1180 packages · 0 pending"));
}

#[test]
fn source_menus_list_install_remove_and_update() {
  for source in [PackageSource::Official, PackageSource::Aur] {
    let mut app = aur_app();
    app.page = PackagesPage::Source(source);
    let mut rows = app.rows();
    // Official Repositories also lists Repositories, after the tasks.
    if source == PackageSource::Official {
      assert_eq!(
        rows.pop().and_then(|row| row.id().copied()),
        Some(Item::Open(PackagesPage::Repositories))
      );
    }
    assert_eq!(
      rows.iter().filter_map(Row::id).copied().collect::<Vec<_>>(),
      [
        PackageTask::Install,
        PackageTask::Remove,
        PackageTask::Update
      ]
      .map(|task| Item::Open(PackagesPage::Task(source, task)))
    );
    assert_eq!(
      rows.iter().map(Row::label).collect::<Vec<_>>(),
      ["Install", "Remove", "Update"]
    );
    assert_eq!(
      rows.iter().map(Row::icon_glyph).collect::<Vec<_>>(),
      [icons::ADD, icons::DELETE, icons::UPDATE].map(Some)
    );
  }
}

#[test]
fn package_home_is_navigable_and_back_returns_to_the_origin() {
  let mut app = app();
  app.handle(KeyCode::Enter);
  assert_eq!(app.page, PackagesPage::Source(PackageSource::Official));
  app.handle(KeyCode::Down);
  app.handle(KeyCode::Enter);
  assert_eq!(app.page, OFFICIAL_REMOVE);
  assert!(!app.handle(KeyCode::Esc));
  assert_eq!(app.page, PackagesPage::Source(PackageSource::Official));
  assert_eq!(
    app.menu.selected_id(&app.rows()),
    Some(Item::Open(OFFICIAL_REMOVE))
  );
  assert!(!app.handle(KeyCode::Left));
  assert_eq!(app.page, PackagesPage::Home);
  assert_eq!(
    app.menu.selected_id(&app.rows()),
    Some(Item::Open(PackagesPage::Source(PackageSource::Official)))
  );
  assert!(app.handle(KeyCode::Esc));

  let text = screen(&mut app, 90, 25);
  assert!(text.contains("ARGVUS"));
  assert!(text.contains("Packages"));
  assert!(text.contains("Enter"));
}

#[test]
fn packages_pages_have_no_button_bar_and_tab_does_nothing() {
  let mut app = app();
  app.page = OFFICIAL_REMOVE;
  app.packages.push(package("linux", true));
  let before = app.menu;
  app.handle(KeyCode::Tab);
  app.handle(KeyCode::BackTab);
  assert_eq!(app.menu, before);
  let text = screen(&mut app, 90, 25);
  assert!(!text.contains("[ "), "{text}");
  assert!(!text.contains("Actions"), "{text}");
  assert!(text.contains("linux"));
}

#[test]
fn packages_breadcrumbs_follow_the_source_and_the_task() {
  let mut app = app();
  for (page, path) in [
    (
      PackagesPage::Source(PackageSource::Official),
      "Official Repositories",
    ),
    (OFFICIAL_INSTALL, "Official Repositories > Install"),
    (OFFICIAL_REMOVE, "Official Repositories > Remove"),
    (OFFICIAL_UPDATE, "Official Repositories > Update"),
    (
      PackagesPage::Orphans,
      "Official Repositories > Remove > Orphans",
    ),
    (
      PackagesPage::Source(PackageSource::Aur),
      "AUR (Arch User Repository)",
    ),
    (AUR_INSTALL, "AUR (Arch User Repository) > Install"),
    (AUR_REMOVE, "AUR (Arch User Repository) > Remove"),
    (AUR_UPDATE, "AUR (Arch User Repository) > Update"),
  ] {
    app.page = page;
    let expected = format!("System Administration > Packages > {path}");
    assert_eq!(app.breadcrumb(), expected);
  }
  // A package page opened from a list keeps the path of that list.
  app.details_name = Some("linux".into());
  for (parent, path) in [
    (OFFICIAL_INSTALL, "Official Repositories > Install"),
    (AUR_UPDATE, "AUR (Arch User Repository) > Update"),
    (
      PackagesPage::Orphans,
      "Official Repositories > Remove > Orphans",
    ),
  ] {
    app.details_parent = parent;
    app.page = PackagesPage::Details;
    let expected = format!("System Administration > Packages > {path} > linux");
    assert_eq!(app.breadcrumb(), expected);
  }
}

#[test]
fn packages_render_at_80x24() {
  let mut app = orphans_app();
  let text = screen(&mut app, 80, 24);
  assert!(text.contains("libfoo"));
  assert!(text.contains("Danger zone"));
  let mut app = aur_app();
  app.page = AUR_UPDATE;
  app.updates = vec![update("yay-bin")];
  let text = screen(&mut app, 80, 24);
  assert!(text.contains("yay-bin"), "{text}");
  assert!(text.contains("Upgrade all (AUR)"), "{text}");
}
