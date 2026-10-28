//! Official repositories pages: adding, reviewing, listing and toggling repositories.

use super::*;

#[test]
fn official_repositories_lead_to_add_and_list() {
  let mut app = app();
  app.dashboard.repository_count = 3;
  app.page = PackagesPage::Source(PackageSource::Official);
  let rows = app.rows();
  let repositories = row(&rows, Item::Open(PackagesPage::Repositories));
  assert_eq!(repositories.label(), "Repositories");
  assert_eq!(repositories.detail_text(), Some("3 repositories"));
  app.page = PackagesPage::Source(PackageSource::Aur);
  assert!(
    !app
      .rows()
      .iter()
      .any(|row| row.id() == Some(&Item::Open(PackagesPage::Repositories))),
    "the AUR has no repositories"
  );

  app.go(PackagesPage::Repositories);
  let labels: Vec<_> = app
    .rows()
    .iter()
    .map(|row| row.label().to_owned())
    .collect();
  assert_eq!(labels, ["Add", "List"]);
  for (page, path) in [
    (
      PackagesPage::Repositories,
      "Official Repositories > Repositories",
    ),
    (
      PackagesPage::RepositoryAdd,
      "Official Repositories > Repositories > Add",
    ),
    (
      PackagesPage::RepositoryList,
      "Official Repositories > Repositories > List",
    ),
  ] {
    app.page = page;
    assert_eq!(
      app.breadcrumb(),
      format!("System Administration > Packages > {path}")
    );
  }
  app.repositories = vec![repository("argvus", true, None)];
  app.page = PackagesPage::RepositoryDetails(0);
  assert!(app.breadcrumb().ends_with("Repositories > List > argvus"));
  app.handle(KeyCode::Esc);
  assert_eq!(app.page, PackagesPage::RepositoryList);
  assert_eq!(app.menu.selected_id(&app.rows()), Some(Item::Repository(0)));
  app.handle(KeyCode::Esc);
  assert_eq!(app.page, PackagesPage::Repositories);
  app.handle(KeyCode::Esc);
  assert_eq!(app.page, PackagesPage::Source(PackageSource::Official));
  assert_eq!(
    app.menu.selected_id(&app.rows()),
    Some(Item::Open(PackagesPage::Repositories))
  );
}

#[test]
fn the_add_page_shows_the_fields_of_each_source() {
  let mut app = app();
  app.page = PackagesPage::RepositoryAdd;
  let ids =
    |app: &PackagesApp| -> Vec<Item> { app.rows().iter().filter_map(Row::id).copied().collect() };
  assert_eq!(
    ids(&app),
    [
      Item::RepoField(DraftField::Source),
      Item::RepoField(DraftField::ConfUrl),
      Item::RepoField(DraftField::KeySource),
      Item::RepoReview,
    ]
  );
  assert!(
    !row(&app.rows(), Item::RepoReview).is_selectable(),
    "nothing to review yet"
  );
  // Enter moves a choice forward, ←/→ both ways.
  press(&mut app, Item::RepoField(DraftField::Source));
  assert!(ids(&app).contains(&Item::RepoField(DraftField::ConfText)));
  app.handle(KeyCode::Right);
  assert!(ids(&app).contains(&Item::RepoField(DraftField::Server)));
  assert!(ids(&app).contains(&Item::RepoField(DraftField::SigLevel)));
  app.handle(KeyCode::Left);
  app.handle(KeyCode::Left);
  assert!(ids(&app).contains(&Item::RepoField(DraftField::ConfUrl)));
  press(&mut app, Item::RepoField(DraftField::KeySource));
  assert!(ids(&app).contains(&Item::RepoField(DraftField::KeyUrl)));
  let text = screen(&mut app, 80, 24);
  for expected in [
    "Repository",
    "Signing key",
    "Configuration URL",
    "Key URL",
    "Review and add",
  ] {
    assert!(text.contains(expected), "{expected}\n{text}");
  }
}

#[test]
fn text_fields_open_an_editor_that_saves_and_pastes() {
  let mut app = app();
  app.page = PackagesPage::RepositoryAdd;
  press(&mut app, Item::RepoField(DraftField::ConfUrl));
  assert!(app.captures_text(), "q and ? are text in the editor");
  for character in "https://argvus.github.io/x.conf".chars() {
    app.handle(KeyCode::Char(character));
  }
  app.handle(KeyCode::Enter);
  assert!(app.editor.is_none());
  assert_eq!(app.repo_draft.conf_url, "https://argvus.github.io/x.conf");
  assert!(row(&app.rows(), Item::RepoReview).is_selectable());

  // Text: a multi-line editor; Enter breaks lines and Ctrl+S saves.
  app.repo_draft.source = repositories::ConfSource::Text;
  press(&mut app, Item::RepoField(DraftField::ConfText));
  app.paste("[argvus]\r\nServer = https://a/$arch\r\n");
  app.handle(KeyCode::Enter);
  let text = screen(&mut app, 80, 24);
  assert!(text.contains("Ctrl+S save"), "{text}");
  app.handle_key(ctrl('s'));
  assert_eq!(
    app.repo_draft.conf_text,
    "[argvus]\nServer = https://a/$arch\n\n"
  );
  assert_eq!(
    row(&app.rows(), Item::RepoField(DraftField::ConfText)).detail_text(),
    Some("3 lines")
  );
  // Esc cancels the editor without touching the field.
  press(&mut app, Item::RepoField(DraftField::ConfText));
  app.handle(KeyCode::Char('x'));
  app.handle(KeyCode::Esc);
  assert!(app.editor.is_none());
  assert!(!app.repo_draft.conf_text.contains('x'));
}

#[test]
fn leaving_the_add_page_with_a_draft_asks_first() {
  let mut app = app();
  app.page = PackagesPage::RepositoryAdd;
  app.handle(KeyCode::Esc);
  assert_eq!(
    app.page,
    PackagesPage::Repositories,
    "an empty draft leaves at once"
  );

  app.go(PackagesPage::RepositoryAdd);
  app.repo_draft.conf_url = "https://a/x.conf".into();
  app.handle(KeyCode::Esc);
  assert_eq!(app.page, PackagesPage::RepositoryAdd);
  assert!(app.discard.is_some());
  assert!(screen(&mut app, 80, 24).contains("Discard the repository being added?"));
  app.handle(KeyCode::Enter);
  assert!(app.discard.is_none(), "Enter on Cancel keeps the draft");
  assert_eq!(app.repo_draft.conf_url, "https://a/x.conf");
  app.handle(KeyCode::Esc);
  app.handle(KeyCode::Char('y'));
  assert_eq!(app.page, PackagesPage::Repositories);
  assert_eq!(app.repo_draft, RepositoryDraft::default());
}

#[test]
fn a_reviewed_repository_is_confirmed_then_offers_the_upgrade() {
  let mut app = app();
  app.page = PackagesPage::RepositoryAdd;
  app.repo_draft.conf_url = "https://a/x.conf".into();
  app.apply(Loaded::RepositoryReview(Box::new(review(true))));
  let Some(Action::RepoAdd(pending)) = &app.pending else {
    panic!("the review opens the confirmation");
  };
  assert_eq!(pending.name(), "argvus");
  assert!(app.pending.as_ref().unwrap().is_destructive());
  let text = screen(&mut app, 100, 30);
  assert!(text.contains("0123 4567 89AB CDEF"), "{text}");
  assert!(
    !app.confirmation.is_confirm_focused(),
    "focus starts on Cancel"
  );

  // Confirmed: it runs (no process starts in test builds); on success the
  // draft is cleared, the List opens and, once the output is closed, the
  // full upgrade is offered.
  app.handle(KeyCode::Char('y'));
  assert!(matches!(app.running, Some(Action::RepoAdd(_))));
  app.action = None;
  app.after_success();
  assert_eq!(app.page, PackagesPage::RepositoryList);
  assert_eq!(app.repo_draft, RepositoryDraft::default());
  assert!(app.pending.is_none(), "the output window is still open");
  app.handle(KeyCode::Esc);
  assert_eq!(app.pending, Some(Action::UpgradeAfterRepoAdd));
  assert!(screen(&mut app, 100, 30).contains("Synchronize the databases"));
}

#[test]
fn repository_operations_build_the_privileged_commands() {
  let (echo, process) = privileged_process(Action::RepoAdd(Box::new(review(true))));
  assert!(
    echo.contains("pacman-key --lsign-key 0123456789ABCDEF"),
    "{echo}"
  );
  assert_eq!(
    &process.args[..5],
    [
      "system-settings",
      "package",
      "repo-add",
      "argvus",
      review(true).conf.as_str()
    ]
  );
  assert_eq!(
    &process.args[5..],
    ["0123456789ABCDEF0123456789ABCDEF01234567", "99010d"]
  );
  let (_, unsigned) = privileged_process(Action::RepoAdd(Box::new(review(false))));
  assert_eq!(unsigned.args.len(), 5, "no key arguments without a key");
  for (action, expected) in [
    (
      Action::RepoEnable("argvus".into()),
      vec!["repo-enable", "argvus"],
    ),
    (
      Action::RepoDisable("argvus".into()),
      vec!["repo-disable", "argvus"],
    ),
    (
      Action::RepoRemove {
        name: "argvus".into(),
        key: Some("0123456789ABCDEF0123456789ABCDEF01234567".into()),
        delete_key: true,
      },
      vec!["repo-remove", "argvus", "delete-key"],
    ),
    (
      Action::RepoRemove {
        name: "argvus".into(),
        key: Some("0123456789ABCDEF0123456789ABCDEF01234567".into()),
        delete_key: false,
      },
      vec!["repo-remove", "argvus"],
    ),
    (Action::UpgradeAfterRepoAdd, vec!["upgrade"]),
  ] {
    let (_, process) = privileged_process(action);
    assert_eq!(&process.args[2..], expected);
  }
}

#[test]
fn the_list_opens_a_repository_that_toggles_and_removes() {
  let mut app = app();
  app.page = PackagesPage::RepositoryList;
  assert!(screen(&mut app, 80, 24).contains("No unofficial repositories"));
  app.repositories = vec![
    repository(
      "argvus",
      false,
      Some("0123456789ABCDEF0123456789ABCDEF01234567"),
    ),
    repository("sublime-text", true, None),
  ];
  let rows = app.rows();
  assert_eq!(
    row(&rows, Item::Repository(0)).detail_text(),
    Some("Disabled · argvus.conf")
  );
  press(&mut app, Item::Repository(1));
  assert_eq!(app.page, PackagesPage::RepositoryDetails(1));
  let rows = app.rows();
  assert_eq!(
    row(&rows, Item::RepoToggle).kind(),
    RowKind::Toggle { on: true }
  );
  assert_eq!(row(&rows, Item::RepoRemove).kind(), RowKind::Destructive);
  press(&mut app, Item::RepoToggle);
  assert_eq!(
    app.pending,
    Some(Action::RepoDisable("sublime-text".into()))
  );
  app.handle(KeyCode::Esc);
  // Without a recorded key, Remove is a plain confirmation.
  press(&mut app, Item::RepoRemove);
  assert!(app.pending.as_ref().unwrap().choices().is_none());
  app.handle(KeyCode::Esc);

  // With a key, Remove offers removing the key too.
  app.go(PackagesPage::RepositoryDetails(0));
  press(&mut app, Item::RepoToggle);
  assert_eq!(app.pending, Some(Action::RepoEnable("argvus".into())));
  app.handle(KeyCode::Esc);
  press(&mut app, Item::RepoRemove);
  let text = screen(&mut app, 100, 30);
  assert!(text.contains("Remove the repository and its key"), "{text}");
  app.handle(KeyCode::Up);
  app.handle(KeyCode::Enter);
  assert!(matches!(
    app.running,
    Some(Action::RepoRemove {
      delete_key: true,
      ..
    })
  ));
}
