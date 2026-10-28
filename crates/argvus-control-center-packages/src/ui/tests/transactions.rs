//! Transaction window, confirmations, removal choices and the privileged commands they build.

use super::*;

#[test]
fn package_confirmation_cancel_does_not_start_an_operation() {
  let mut app = app();
  app.pending = Some(Action::Upgrade);
  app.handle(KeyCode::Enter);
  assert!(app.pending.is_none());
  assert!(app.action.is_none());
}

#[test]
fn transaction_window_scrolls_closes_and_generates_action_plans() {
  let mut app = app();
  let live = LiveProcess::new();
  for _ in 0..40 {
    live.push_line("line");
  }
  app.transaction_live = Some(live.clone());
  app.transaction_open = true;
  let total = transaction_wrapped_lines(&live.output());
  assert_eq!(total, 40);
  app.handle(KeyCode::Down);
  assert_eq!(app.transaction_scroll, 1);
  app.handle(KeyCode::Up);
  assert_eq!(app.transaction_scroll, 0);
  app.handle(KeyCode::PageDown);
  assert_eq!(app.transaction_scroll, 10);
  app.handle(KeyCode::Home);
  assert_eq!(app.transaction_scroll, 0);
  app.handle(KeyCode::End);
  assert_eq!(
    app.transaction_scroll as usize,
    40usize.saturating_sub(TRANSACTION_CONTENT_HEIGHT)
  );
  app.handle(KeyCode::Esc);
  assert!(!app.transaction_open);
  assert_eq!(app.transaction_scroll, 0);
}

#[test]
fn start_pending_opens_the_process_window_immediately() {
  let mut app = app();
  // The operation is only checked as built; test builds start no process
  // (the core `testing` feature), so the job below runs nothing.
  let (_, process) = privileged_process(Action::RefreshDatabase);
  assert_eq!(process.args, ["system-settings", "package", "refresh-db"]);
  app.pending = Some(Action::RefreshDatabase);
  app.start_pending();
  assert!(
    app.transaction_open,
    "window must appear when the process starts"
  );
  assert!(app.transaction_live.is_some());
  assert!(app.transaction_follow);
  assert!(app.action.is_some());
  app.handle(KeyCode::Esc);
  assert!(!app.transaction_open);
}

#[test]
fn transaction_bottom_offset_is_zero_for_short_output() {
  let mut app = app();
  let live = LiveProcess::new();
  live.push_line("done.");
  app.transaction_live = Some(live);
  app.transaction_open = true;
  app.handle(KeyCode::End);
  assert_eq!(app.transaction_scroll, 0);
  assert_eq!(transaction_bottom_offset("done."), 0);
}

#[test]
fn transaction_window_uses_theme_background_and_border() {
  let theme = Theme::load();
  let mut app = PackagesApp::new(
    Lang::for_locale("en-US"),
    theme.clone(),
    Capabilities::default(),
  );
  let live = LiveProcess::new();
  live.push_line("ok");
  app.transaction_live = Some(live);
  app.transaction_open = true;
  let mut terminal = Terminal::new(TestBackend::new(120, 25)).unwrap();
  terminal.draw(|frame| app.draw(frame)).unwrap();
  let buffer = terminal.backend().buffer();
  let styled = buffer.content.iter().any(|cell| {
    cell.symbol() == "│" && cell.bg == theme.background && cell.fg == theme.border_active
  });
  assert!(
    styled,
    "border should contrast with the background behind it"
  );
}

#[test]
fn confirmation_starts_on_cancel_and_answers_y_and_n() {
  let mut app = app();
  app.request(Action::RefreshDatabase);
  assert!(!app.confirmation.is_confirm_focused());
  assert_eq!(app.footer_hints(&app.rows()), confirm_hints(app.lang));
  app.handle(KeyCode::Char('n'));
  assert!(app.pending.is_none());
  assert!(app.action.is_none());
  app.request(Action::RefreshDatabase);
  // y starts the job, which runs no process in test builds (core
  // `testing` feature); the command itself is checked as built in
  // `package_operations_build_the_privileged_system_settings_command`.
  app.handle(KeyCode::Char('y'));
  assert!(app.pending.is_none());
  assert!(app.action.is_some(), "y runs the confirmed operation");
  assert!(app.transaction_open);
}

#[test]
fn only_removals_cleanups_and_downgrades_use_the_danger_style() {
  for action in [
    Action::Remove(vec!["zsh".into()]),
    Action::RemoveOnly(vec!["7zip".into()]),
    Action::RemoveCascade {
      packages: vec!["7zip".into()],
      dependents: vec!["julia".into()],
    },
    Action::CleanCache("keep-one"),
    Action::Downgrade("/tmp/zsh.pkg.tar.zst".into()),
  ] {
    assert!(action.is_destructive(), "{action:?}");
  }
  for action in [
    Action::Install(vec!["zsh".into()]),
    Action::Reinstall("zsh".into()),
    Action::UpgradePackage("zsh".into()),
    Action::Upgrade,
    Action::RefreshDatabase,
    Action::AurInstall("paru-bin".into()),
    Action::AurUpgradePackage("paru-bin".into()),
    Action::AurUpgrade,
    Action::ApplyMirrors(String::new()),
  ] {
    assert!(!action.is_destructive(), "{action:?}");
  }
}

#[test]
fn removal_choice_names_the_dependents() {
  let lang = Lang::for_locale("en-US");
  let dependents: Vec<String> = (1..=10).map(|n| format!("dep{n}")).collect();
  let names: Vec<&str> = dependents.iter().map(String::as_str).collect();
  let message = action_message(lang, &cascade(&names));
  assert!(message.starts_with("7zip is required by"), "{message}");
  assert!(message.contains("dep1, dep2"), "{message}");
  assert!(message.contains("dep8, …"), "{message}");
  assert!(!message.contains("dep9"), "{message}");
  assert!(!message.contains('{'), "placeholders are filled: {message}");
}

#[test]
fn removal_choice_offers_only_the_package_or_the_cascade() {
  let mut app = removal_choice_app();
  assert_eq!(app.choice.focused(), None, "focus starts on Cancel");
  let text = screen(&mut app, 100, 30);
  assert!(text.contains("julia"), "{text}");
  assert!(text.contains("Remove only 7zip (-Rdd)"), "{text}");
  assert!(
    text.contains("Remove with dependents: 2 packages (-Rcns)"),
    "{text}"
  );
  assert!(text.contains("> Cancel"), "{text}");
  let footer = app.footer_hints(&app.rows());
  assert!(footer.contains("Enter"), "{footer}");
  assert!(
    !footer.contains("y "),
    "no y with several options: {footer}"
  );

  // Enter on Cancel, `y` and Esc never remove anything.
  app.handle(KeyCode::Char('y'));
  assert!(app.pending.is_some() && app.action.is_none());
  app.handle(KeyCode::Enter);
  assert!(app.pending.is_none() && app.action.is_none());
}

#[test]
fn removal_choice_runs_the_chosen_way() {
  // Up from Cancel reaches the cascade (the row above), then only the package.
  for (presses, expected) in [
    (2, Action::RemoveOnly(vec!["7zip".into()])),
    (1, cascade(&["julia"])),
  ] {
    let mut app = removal_choice_app();
    for _ in 0..presses {
      app.handle(KeyCode::Up);
    }
    let chosen = app.pending.as_ref().and_then(Action::choices).unwrap()
      [app.choice.focused().unwrap()]
    .clone();
    assert_eq!(chosen, expected);
    // The choice confirms: the job starts (no process runs in test builds).
    app.handle(KeyCode::Enter);
    assert!(app.pending.is_none());
    assert!(app.action.is_some(), "{expected:?} starts");
    assert!(app.transaction_open);
  }
}

#[test]
fn plan_counters_show_a_readable_download_size() {
  let lang = Lang::for_locale("en-US");
  let message = preview_message(
    lang,
    &TransactionPlan {
      install: vec![package("zsh", false)],
      download_bytes: 5 * 1024 * 1024,
      ..Default::default()
    },
  );
  assert!(message.contains(": 1\n"), "{message}");
  assert!(message.ends_with(": 5 MiB"), "{message}");
}

#[test]
fn aur_install_confirms_with_the_pkgbuild_warning_only() {
  let mut app = app();
  app.page = AUR_INSTALL;
  app.aur.push(AurPackage {
    name: "paru-bin".into(),
    ..Default::default()
  });
  app.handle(KeyCode::Right);
  press(&mut app, Item::Install);
  assert!(app.plan.is_none(), "the AUR helper has no transaction plan");
  assert_eq!(app.pending, Some(Action::AurInstall("paru-bin".into())));
  assert!(app.preview.is_none());
  let text = screen(&mut app, 100, 30);
  assert!(text.contains("paru-bin"), "{text}");
  assert!(!text.contains(": 0"), "{text}");
}

#[test]
fn package_operations_build_the_privileged_system_settings_command() {
  let cases = [
    (
      Action::Install(vec!["zsh".into(), "jq".into()]),
      "$ pacman -S --needed zsh jq",
      vec!["install", "zsh", "jq"],
    ),
    (
      Action::Remove(vec!["zsh".into()]),
      "$ pacman -Rns zsh",
      vec!["remove", "zsh"],
    ),
    (
      Action::RemoveOnly(vec!["7zip".into()]),
      "$ pacman -Rdd 7zip",
      vec!["remove-only", "7zip"],
    ),
    (
      Action::RemoveCascade {
        packages: vec!["7zip".into()],
        dependents: vec!["julia".into()],
      },
      "$ pacman -Rcns 7zip",
      vec!["remove-cascade", "7zip"],
    ),
    (
      Action::Reinstall("zsh".into()),
      "$ pacman -S zsh",
      vec!["reinstall", "zsh"],
    ),
    (Action::Upgrade, "$ pacman -Syu", vec!["upgrade"]),
    (
      Action::UpgradePackage("linux".into()),
      "$ pacman -S linux",
      vec!["upgrade-package", "linux"],
    ),
    (Action::RefreshDatabase, "$ pacman -Syy", vec!["refresh-db"]),
    (
      Action::CleanCache("keep-one"),
      "$ paccache -r",
      vec!["clean-cache", "keep-one"],
    ),
    (
      Action::Downgrade("/var/cache/pacman/pkg/zsh-5.9-1-x86_64.pkg.tar.zst".into()),
      "$ pacman -U /var/cache/pacman/pkg/zsh-5.9-1-x86_64.pkg.tar.zst",
      vec![
        "downgrade",
        "/var/cache/pacman/pkg/zsh-5.9-1-x86_64.pkg.tar.zst",
      ],
    ),
    (
      Action::ApplyMirrors("Server = https://a/$repo/os/$arch\n".into()),
      "$ update mirrorlist",
      vec!["mirror-apply", "Server = https://a/$repo/os/$arch\n"],
    ),
  ];
  for (action, expected_echo, expected_args) in cases {
    let (echo, process) = privileged_process(action);
    assert_eq!(echo, expected_echo);
    assert_eq!(process.program, "/usr/lib/argvus/commands/control-center");
    let mut args = vec!["system-settings", "package"];
    args.extend(expected_args);
    assert_eq!(process.args, args);
  }
}

#[test]
fn aur_install_runs_the_helper_as_the_normal_user() {
  let caps = |paru, yay| Capabilities {
    has_paru: paru,
    has_yay: yay,
    ..Default::default()
  };
  for (caps, helper) in [(caps(true, true), "paru"), (caps(false, true), "yay")] {
    let Ok(ActionCommand::User { echo, process }) =
      action_command(Action::AurInstall("paru-bin".into()), &caps)
    else {
      panic!("the AUR install is not privileged");
    };
    assert_eq!(echo, format!("$ {helper} -S paru-bin"));
    assert_eq!(process.program, helper);
    assert_eq!(process.args, ["-S", "paru-bin"]);
  }
  assert!(action_command(Action::AurInstall("paru-bin".into()), &caps(false, false)).is_err());
  for (action, args) in [
    (
      Action::AurUpgradePackage("yay-bin".into()),
      vec!["-S", "yay-bin"],
    ),
    (Action::AurUpgrade, vec!["-Sua"]),
  ] {
    let Ok(ActionCommand::User { echo, process }) = action_command(action, &caps(false, true))
    else {
      panic!("the AUR update is not privileged");
    };
    assert_eq!(echo, format!("$ yay {}", args.join(" ")));
    assert_eq!(process.program, "yay");
    assert_eq!(process.args, args);
  }
}
