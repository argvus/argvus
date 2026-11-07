//! The top Search bar is the only editable package query.
use super::*;

#[test]
fn slash_focuses_search_on_every_filtered_page_without_opening_an_overlay() {
  for page in [
    OFFICIAL_INSTALL,
    AUR_INSTALL,
    OFFICIAL_REMOVE,
    AUR_REMOVE,
    OFFICIAL_UPDATE,
    AUR_UPDATE,
    PackagesPage::Orphans,
  ] {
    let mut app = app();
    app.page = page;
    app.query = "linux".into();
    app.handle_key(KeyCode::Char('/').into());
    assert_eq!(app.query, "linux");
    assert!(app.search_focused);
    assert!(app.input.is_none());
    assert!(app.input_mode.is_none());
    let text = screen(&mut app, 100, 30);
    assert!(text.contains("Search: linux▌"), "{text}");
    assert!(
      !text.contains("Enter Apply"),
      "the old search overlay is gone: {text}"
    );
  }
}

#[test]
fn enter_in_search_submits_even_when_old_results_exist_or_the_query_was_cached() {
  for page in [OFFICIAL_INSTALL, AUR_INSTALL] {
    let mut app = app();
    app.page = page;
    app.query = "firefox".into();
    app.packages = vec![package("firefox", false)];
    app.aur = vec![AurPackage {
      name: "firefox".into(),
      ..Default::default()
    }];
    app.last_loaded = Some((app.reload_key(), std::time::Instant::now()));
    app.handle_key(KeyCode::Char('/').into());
    app.handle_key(KeyCode::Enter.into());
    assert!(app.job.is_some());
    assert_eq!(app.loading_key.as_ref(), Some(&app.reload_key()));
    assert!(app.packages.is_empty() && app.aur.is_empty());
    assert!(!app.search_focused);
    assert!(app.pending.is_none() && app.plan.is_none());
  }
}

#[test]
fn editing_cancels_old_remote_results_and_waits_for_another_enter() {
  for page in [OFFICIAL_INSTALL, AUR_INSTALL] {
    let mut app = app();
    app.page = page;
    app.query = "old".into();
    app.loading_key = Some(app.reload_key());
    let (release, wait) = std::sync::mpsc::channel();
    app.job = Some(app.jobs.spawn(move |_| {
      wait.recv().unwrap();
      Ok(Ok(Loaded::Packages(vec![package("old-result", false)])))
    }));
    app.handle_key(KeyCode::Char('/').into());
    app.paste("-new");
    assert_eq!(app.query, "old-new");
    assert!(app.job.is_none() && app.loading_key.is_none());
    release.send(()).unwrap();
    app.poll();
    assert!(app.packages.is_empty());
    assert!(app.job.is_none(), "edits must wait for explicit submission");
    app.handle_key(KeyCode::Enter.into());
    assert!(app.job.is_some());
  }
}

#[test]
fn local_filters_keep_their_pending_load_and_do_not_fetch_again() {
  for page in [
    OFFICIAL_REMOVE,
    AUR_REMOVE,
    OFFICIAL_UPDATE,
    AUR_UPDATE,
    PackagesPage::Orphans,
  ] {
    let mut app = app();
    app.page = page;
    let loading_key = app.reload_key();
    app.loading_key = Some(loading_key.clone());
    let (release, wait) = std::sync::mpsc::channel();
    app.job = Some(app.jobs.spawn(move |_| {
      wait.recv().unwrap();
      Ok(Ok(Loaded::Packages(vec![
        package("linux", true),
        package("mesa", true),
      ])))
    }));
    app.handle_key(KeyCode::Char('/').into());
    app.paste("linux");
    assert!(app.job.is_some());
    assert_eq!(app.reload_key(), loading_key);
    release.send(()).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    while !app.poll() {
      assert!(std::time::Instant::now() < deadline);
      std::thread::yield_now();
    }
    assert!(app.job.is_none());
    assert_eq!(
      app.last_loaded.as_ref().map(|(key, _)| key),
      Some(&loading_key)
    );
    assert_eq!(
      app
        .visible_packages()
        .iter()
        .map(|(_, package)| package.name.as_str())
        .collect::<Vec<_>>(),
      ["linux"]
    );
  }
}

#[test]
fn search_navigation_separates_submission_from_package_actions() {
  let mut app = app();
  app.page = OFFICIAL_UPDATE;
  app.updates = vec![update("linux")];
  app.handle(KeyCode::Tab);
  assert!(app.actions.cursor().is_some());
  app.handle(KeyCode::Char('/'));
  assert!(app.search_focused && app.actions.cursor().is_none());
  app.handle(KeyCode::Enter);
  assert!(app.pending.is_none() && app.plan.is_none());
  assert!(!app.search_focused);
  app.handle(KeyCode::Up);
  assert!(app.search_focused);
  app.handle(KeyCode::Down);
  assert!(!app.search_focused);
  app.handle(KeyCode::Enter);
  assert_eq!(app.pending, Some(Action::UpgradePackage("linux".into())));
}

#[test]
fn search_edits_and_pastes_utf8_at_the_cursor_without_exceeding_the_byte_limit() {
  let mut app = app();
  app.page = OFFICIAL_REMOVE;
  app.handle(KeyCode::Char('/'));
  app.paste("café");
  app.handle(KeyCode::Left);
  app.handle(KeyCode::Backspace);
  assert_eq!(app.query, "caé");
  app.handle(KeyCode::Delete);
  assert_eq!(app.query, "ca");
  app.handle(KeyCode::Home);
  app.paste("火");
  assert_eq!(app.query, "火ca");
  app.handle(KeyCode::Right);
  app.handle(KeyCode::Char('r'));
  assert_eq!(app.query, "火cra");
  assert_eq!(
    app.page, OFFICIAL_REMOVE,
    "arrows edit Search rather than opening details"
  );
  app.query = "x".repeat(QUERY_MAX - 1);
  app.handle(KeyCode::Char('/'));
  app.paste("é");
  assert_eq!(app.query.len(), QUERY_MAX - 1);
  app.paste("a");
  assert_eq!(app.query.len(), QUERY_MAX);
  app.paste("z");
  assert_eq!(app.query.len(), QUERY_MAX);
  let text = screen(&mut app, 60, 18);
  assert!(text.contains("Search:") && text.contains('▌'), "{text}");
}

#[test]
fn search_focus_and_help_render_without_raw_keys_in_both_languages() {
  for locale in ["en-US", "pt-BR"] {
    let mut app = app();
    app.lang = Lang::for_locale(locale);
    app.page = OFFICIAL_INSTALL;
    app.handle(KeyCode::Char('/'));
    app.paste("firefox");
    let text = screen(&mut app, 100, 30);
    assert!(
      text.contains(&format!(
        "{}: firefox▌",
        tr(app.lang, "control_center.search_2c43ee")
      )),
      "{text}"
    );
    assert!(text.contains("Enter"), "{text}");
    assert!(!text.contains("control_center."), "{text}");
    assert!(app.input.is_none());
  }
}
