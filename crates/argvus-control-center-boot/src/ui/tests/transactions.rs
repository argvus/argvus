//! Boot actions, confirmations and the process window.

use super::*;

#[test]
fn systemd_entry_default_goes_through_the_confirmation() {
  let mut app = systemd_app();
  app.page = BootPage::Bootloader;
  press(&mut app, Item::Entry(0));
  assert_eq!(app.page, BootPage::BootloaderDetail(0));
  press(&mut app, Item::SetDefault);
  assert!(app.pending.is_some());
  app.handle(KeyCode::Tab);
  assert!(app.confirmation.is_confirm_focused());
  app.handle(KeyCode::BackTab);
  assert!(!app.confirmation.is_confirm_focused());
  app.handle(KeyCode::Enter);
  assert!(app.pending.is_none());
  assert!(app.action.is_none());
  app.handle(KeyCode::Enter);
  assert!(app.pending.is_some());
  app.handle(KeyCode::Esc);
  assert!(app.pending.is_none());
}

#[test]
fn confirmation_starts_on_cancel_and_n_cancels() {
  let mut app = app();
  app.page = BootPage::Initramfs;
  press(&mut app, Item::RegenerateInitramfs);
  assert!(app.pending.is_some());
  assert!(!app.confirmation.is_confirm_focused());
  assert_eq!(app.footer_hints(&app.rows()), confirm_hints(app.lang));
  app.handle(KeyCode::Char('n'));
  assert!(app.pending.is_none());
  assert!(app.action.is_none());
  press(&mut app, Item::RegenerateInitramfs);
  app.handle(KeyCode::Char('k'));
  assert!(app.confirmation.is_confirm_focused());
  app.handle(KeyCode::Esc);
  assert!(app.pending.is_none());
  // A new confirmation starts on Cancel again.
  press(&mut app, Item::RegenerateInitramfs);
  assert!(!app.confirmation.is_confirm_focused());
}

#[test]
fn only_the_regenerations_use_the_danger_style() {
  assert!(BootAction::GrubRegenerate.is_destructive());
  assert!(BootAction::Initramfs.is_destructive());
  assert!(!BootAction::SystemdDefault("arch.conf".into()).is_destructive());
  assert!(!BootAction::SystemdTimeout(3).is_destructive());
  assert!(!BootAction::GrubTimeout(3).is_destructive());
  assert!(!BootAction::GrubCmdline("quiet".into()).is_destructive());
  assert!(!BootAction::Plymouth("argvus".into()).is_destructive());
}

#[test]
fn confirmation_renders_the_boot_change() {
  let mut app = app();
  app.snapshot.plymouth.themes = vec!["argvus".into()];
  app.page = BootPage::Plymouth;
  press(&mut app, Item::Theme(0));
  let text = screen(&mut app, 90, 25);
  assert!(text.contains(tr(app.lang, "control_center.confirm_boot_operation")));
  assert!(text.contains("argvus"));
  assert!(text.contains(tr(app.lang, "control_center.continue")));
}

#[test]
fn boot_changes_wait_for_a_running_snapshot_but_pages_open() {
  let mut app = app();
  app.snapshot.initramfs.presets = vec!["linux.preset".into()];
  app.page = BootPage::Initramfs;
  running_job(&mut app);
  press(&mut app, Item::RegenerateInitramfs);
  assert!(app.pending.is_none());
  press(&mut app, Item::Preset(0));
  assert_eq!(app.page, BootPage::InitramfsDetail(0));
  assert!(!app.handle(KeyCode::Esc));
  assert!(!app.handle(KeyCode::Esc));
  assert_eq!(app.page, BootPage::Home);
  press(&mut app, Item::Kernels);
  assert_eq!(app.page, BootPage::Kernel);
}

#[test]
fn boot_actions_build_the_privileged_system_settings_command() {
  let cases = [
    (
      BootAction::SystemdDefault("arch.conf".into()),
      "$ bootctl set-default arch.conf",
      vec!["systemd-default", "arch.conf"],
    ),
    (
      BootAction::SystemdTimeout(5),
      "$ boot timeout = 5s",
      vec!["systemd-timeout", "5"],
    ),
    (
      BootAction::GrubTimeout(3),
      "$ GRUB_TIMEOUT = 3s",
      vec!["grub-timeout", "3"],
    ),
    (
      BootAction::GrubCmdline("quiet splash".into()),
      "$ GRUB_CMDLINE_LINUX_DEFAULT",
      vec!["grub-cmdline", "quiet splash"],
    ),
    (
      BootAction::GrubRegenerate,
      "$ grub-mkconfig",
      vec!["grub-regenerate"],
    ),
    (
      BootAction::Initramfs,
      "$ mkinitcpio -P",
      vec!["initramfs-regenerate"],
    ),
    (
      BootAction::Plymouth("bgrt".into()),
      "$ plymouth-set-default-theme -R bgrt",
      vec!["plymouth-theme", "bgrt"],
    ),
  ];
  for (action, expected_echo, expected_args) in cases {
    let (echo, request) = boot_request(action).unwrap();
    assert_eq!(echo, expected_echo);
    let process = SystemSettingsOperation::new(
      SystemProcessRunner,
      "/usr/lib/argvus/commands/control-center",
    )
    .process_for(&request)
    .unwrap();
    assert_eq!(process.program, "/usr/lib/argvus/commands/control-center");
    let mut args = vec!["system-settings", "boot"];
    args.extend(expected_args);
    assert_eq!(process.args, args);
  }
}

#[test]
fn boot_start_pending_opens_the_process_window_immediately() {
  let mut app = BootApp::new(
    Lang::for_locale("en-US"),
    Theme::load(),
    Capabilities::default(),
  );
  // The operation is only checked as built; test builds start no process
  // (the core `testing` feature), so the job below runs nothing.
  let (_, request) = boot_request(BootAction::Initramfs).unwrap();
  let process = SystemSettingsOperation::new(
    SystemProcessRunner,
    "/usr/lib/argvus/commands/control-center",
  )
  .process_for(&request)
  .unwrap();
  assert_eq!(
    process.args,
    ["system-settings", "boot", "initramfs-regenerate"]
  );
  app.pending = Some(Pending::Action(BootAction::Initramfs));
  app.start_pending();
  assert!(
    app.transaction_open,
    "window must appear when the process starts"
  );
  assert!(app.transaction_live.is_some(), "live sink must be shared");
  assert!(app.transaction_follow);
  assert!(app.action.is_some());
  app.handle(KeyCode::Esc);
  assert!(!app.transaction_open);
}

#[test]
fn boot_transaction_window_scrolls_and_closes() {
  let mut app = BootApp::new(
    Lang::for_locale("en-US"),
    Theme::load(),
    Capabilities::default(),
  );
  let live = LiveProcess::new();
  for _ in 0..40 {
    live.push_line("line");
  }
  app.transaction_live = Some(live.clone());
  app.transaction_open = true;
  assert!(app.transaction_open);
  app.handle(KeyCode::Down);
  assert_eq!(app.transaction_scroll, 1);
  app.handle(KeyCode::Up);
  assert_eq!(app.transaction_scroll, 0);
  app.handle(KeyCode::PageDown);
  assert_eq!(app.transaction_scroll, 10);
  app.handle(KeyCode::PageUp);
  assert_eq!(app.transaction_scroll, 0);
  app.handle(KeyCode::End);
  assert_eq!(
    app.transaction_scroll as usize,
    transaction_bottom_offset(&live.output()) as usize
  );
  app.handle(KeyCode::Esc);
  assert!(!app.transaction_open);
  assert_eq!(app.transaction_scroll, 0);
}

#[test]
fn boot_transaction_window_renders_process_title_and_output() {
  let mut app = BootApp::new(
    Lang::for_locale("en-US"),
    Theme::load(),
    Capabilities::default(),
  );
  let live = LiveProcess::new();
  live.push_line("$ plymouth-set-default-theme -R argvus");
  live.push_line("==> Building image");
  app.transaction_live = Some(live);
  app.transaction_open = true;
  let mut terminal = Terminal::new(TestBackend::new(110, 25)).unwrap();
  terminal.draw(|frame| app.draw(frame)).unwrap();
  let text = terminal
    .backend()
    .buffer()
    .content
    .iter()
    .map(|cell| cell.symbol())
    .collect::<String>();
  assert!(text.contains("Process") || text.contains("Processo"));
  assert!(text.contains("$ plymouth-set-default-theme -R argvus"));
  assert!(text.contains("scr") || text.contains("rol"));
}
