//! Boot pages: home, kernels, bootloader, timeout, initramfs, plymouth and navigation.

use super::*;

#[test]
fn boot_home_rows_are_submenus_with_their_state() {
  let mut app = app();
  app.snapshot.firmware = "UEFI".into();
  app.snapshot.current_kernel = "linux-lts 6.18".into();
  app.snapshot.secure_boot = Some(false);
  app.snapshot.bootloader = BootloaderKind::SystemdBoot;
  app.snapshot.bootloader_info.timeout = Some(3);
  app.snapshot.initramfs.available = true;
  app.snapshot.initramfs.presets = vec!["linux-lts.preset".into()];
  let rows = app.rows();
  assert_eq!(
    ids(&rows),
    [
      Item::Summary,
      Item::Kernels,
      Item::Bootloader,
      Item::Initramfs,
      Item::Plymouth
    ]
  );
  assert!(rows.iter().all(|row| row.kind() == RowKind::Submenu));
  let detail = |item| {
    row(&rows, item)
      .detail_text()
      .unwrap_or_default()
      .to_owned()
  };
  assert!(detail(Item::Summary).contains("UEFI"));
  assert!(detail(Item::Kernels).contains("linux-lts 6.18"));
  assert!(
    detail(Item::Bootloader).contains("systemd-boot") && detail(Item::Bootloader).contains("3 s")
  );
  assert!(detail(Item::Initramfs).contains('1'));
  assert!(
    detail(Item::Plymouth).contains("Not installed")
      || detail(Item::Plymouth).contains("Não instalado")
  );
  assert_eq!(row(&rows, Item::Bootloader).icon_glyph(), Some(icons::BOOT));
}

#[test]
fn boot_home_dashboard_uses_theme_and_falls_back_to_installed_flag() {
  let mut app = app();
  app.snapshot.plymouth.installed = true;
  let rows = app.rows();
  let plymouth = row(&rows, Item::Plymouth).detail_text().unwrap_or_default();
  assert!(plymouth.contains("Installed") || plymouth.contains("Instalado"));
  app.snapshot.plymouth.current_theme = Some("argvus".into());
  let rows = app.rows();
  assert_eq!(row(&rows, Item::Plymouth).detail_text(), Some("argvus"));
}

#[test]
fn boot_kernel_rows_carry_current_and_default_badges() {
  let mut app = app();
  app.snapshot.kernels = vec![
    KernelInfo {
      package: "linux".into(),
      version: "6.1".into(),
      current: true,
      ..Default::default()
    },
    KernelInfo {
      package: "linux-lts".into(),
      version: "6.18".into(),
      default: true,
      ..Default::default()
    },
    KernelInfo {
      package: "linux-zen".into(),
      version: "6.19".into(),
      ..Default::default()
    },
  ];
  app.page = BootPage::Kernel;
  let rows = app.rows();
  assert_eq!(
    ids(&rows),
    [Item::Kernel(0), Item::Kernel(1), Item::Kernel(2)]
  );
  assert_eq!(rows[0].label(), "linux 6.1");
  assert_eq!(
    rows[0].detail_text(),
    Some(tr(app.lang, "control_center.current"))
  );
  assert_eq!(
    rows[1].detail_text(),
    Some(tr(app.lang, "control_center.default"))
  );
  assert_eq!(rows[2].detail_text(), None);
  assert!(rows.iter().all(|row| row.icon_glyph().is_none()));
}

#[test]
fn kernel_detail_shows_info_and_sets_the_mapped_entry_as_default() {
  let mut app = systemd_app();
  app.page = BootPage::KernelDetail(0);
  let rows = app.rows();
  assert!(
    rows
      .iter()
      .any(|row| row.detail_text() == Some("/boot/EFI/Linux/linux.efi"))
  );
  assert!(
    rows
      .iter()
      .filter(|row| row.id().is_none())
      .all(|row| !row.is_selectable())
  );
  let last = rows.last().unwrap();
  assert_eq!(last.id(), Some(&Item::SetDefault));
  assert_eq!(last.kind(), RowKind::Action);
  assert_eq!(last.icon_glyph(), Some(icons::STAR));
  assert!(last.is_enabled());
  press(&mut app, Item::SetDefault);
  assert!(matches!(
    &app.pending,
    Some(Pending::Action(BootAction::SystemdDefault(id))) if id == "arch.conf"
  ));
}

#[test]
fn set_default_is_disabled_with_the_reason_when_no_entry_maps() {
  let mut app = systemd_app();
  app.snapshot.bootloader_info.entries[0].linux = Some("/vmlinuz-linux-zen".into());
  app.page = BootPage::KernelDetail(0);
  let rows = app.rows();
  let set_default = row(&rows, Item::SetDefault);
  assert!(!set_default.is_enabled());
  assert_eq!(
    set_default.detail_text(),
    Some(tr(
      app.lang,
      "control_center.unavailable_no_systemd_boot_entry"
    ))
  );

  app.snapshot.bootloader = BootloaderKind::Unknown;
  let rows = app.rows();
  assert_eq!(
    row(&rows, Item::SetDefault).detail_text(),
    Some(tr(
      app.lang,
      "control_center.unavailable_unknown_bootloader"
    ))
  );
  // Disabled rows are skipped: the page has no cursor.
  app.handle(KeyCode::Enter);
  assert!(app.pending.is_none());
}

#[test]
fn boot_pages_are_keyboard_navigable_and_back_returns_to_the_origin() {
  let mut app = systemd_app();
  app.handle(KeyCode::Down);
  app.handle(KeyCode::Enter);
  assert_eq!(app.page, BootPage::Kernel);
  app.job = None;
  app.handle(KeyCode::Right);
  assert_eq!(app.page, BootPage::KernelDetail(0));
  app.handle(KeyCode::Esc);
  assert_eq!(app.page, BootPage::Kernel);
  assert_eq!(app.menu.selected_id(&app.rows()), Some(Item::Kernel(0)));
  app.handle(KeyCode::Left);
  assert_eq!(app.page, BootPage::Home);
  assert_eq!(app.menu.selected_id(&app.rows()), Some(Item::Kernels));
  assert!(app.handle(KeyCode::Esc));
}

#[test]
fn timeout_input_is_bounded_and_cancelable() {
  let mut app = app();
  app.snapshot.bootloader = BootloaderKind::SystemdBoot;
  app.page = BootPage::Bootloader;
  press(&mut app, Item::Timeout);
  assert_eq!(app.timeout_input.as_deref(), Some(""));
  app.handle(KeyCode::Char('6'));
  app.handle(KeyCode::Char('1'));
  app.handle(KeyCode::Enter);
  assert!(app.pending.is_none());
  assert!(
    app
      .status
      .as_ref()
      .is_some_and(|status| status.kind == StatusKind::Error)
  );
  press(&mut app, Item::Timeout);
  app.handle(KeyCode::Char('5'));
  app.handle(KeyCode::Enter);
  assert!(matches!(
    app.pending,
    Some(Pending::Action(BootAction::SystemdTimeout(5)))
  ));
  app.handle(KeyCode::Esc);
  press(&mut app, Item::Timeout);
  app.handle(KeyCode::Esc);
  assert!(app.timeout_input.is_none());
  assert_eq!(app.page, BootPage::Bootloader);
}

#[test]
fn bootloader_rows_follow_the_detected_bootloader() {
  let mut app = systemd_app();
  app.snapshot.bootloader_info.timeout = Some(4);
  app.page = BootPage::Bootloader;
  let rows = app.rows();
  assert_eq!(ids(&rows), [Item::Entry(0), Item::Timeout]);
  let timeout = row(&rows, Item::Timeout);
  assert_eq!(timeout.kind(), RowKind::Value { step: None });
  assert_eq!(timeout.detail_text(), Some("4 s"));
  assert_eq!(timeout.icon_glyph(), Some(icons::TIMER));

  let mut grub = super::tests::app();
  grub.snapshot.bootloader = BootloaderKind::Grub;
  grub.snapshot.bootloader_info.grub_values =
    vec![("GRUB_CMDLINE_LINUX_DEFAULT".into(), "quiet".into())];
  grub.page = BootPage::Bootloader;
  let rows = grub.rows();
  assert_eq!(
    ids(&rows),
    [Item::Timeout, Item::KernelCmdline, Item::RegenerateGrub]
  );
  assert_eq!(row(&rows, Item::KernelCmdline).detail_text(), Some("quiet"));
  assert_eq!(
    row(&rows, Item::KernelCmdline).icon_glyph(),
    Some(icons::TERMINAL)
  );
  // The Danger zone closes the page.
  let danger = &rows[rows.len() - 2];
  assert!(danger.is_section());
  assert_eq!(danger.label(), tr(grub.lang, "control_center.danger_zone"));
  let regenerate = rows.last().unwrap();
  assert_eq!(regenerate.kind(), RowKind::Destructive);
  assert_eq!(regenerate.icon_glyph(), Some(icons::SYNC));
  press(&mut grub, Item::RegenerateGrub);
  assert!(matches!(
    grub.pending,
    Some(Pending::Action(BootAction::GrubRegenerate))
  ));
  grub.handle(KeyCode::Esc);
  press(&mut grub, Item::KernelCmdline);
  assert_eq!(grub.timeout_input.as_deref(), Some("quiet"));
  assert_eq!(grub.input_mode, Some(InputMode::GrubCmdline));
}

#[test]
fn timeout_is_disabled_with_the_reason_for_an_unknown_bootloader() {
  let mut app = app();
  app.page = BootPage::Bootloader;
  let rows = app.rows();
  let timeout = row(&rows, Item::Timeout);
  assert!(!timeout.is_enabled());
  assert_eq!(
    timeout.detail_text(),
    Some(tr(
      app.lang,
      "control_center.unavailable_unknown_bootloader"
    ))
  );
  app.handle(KeyCode::Enter);
  assert!(app.timeout_input.is_none());
}

#[test]
fn initramfs_regenerates_from_the_danger_zone() {
  let mut app = app();
  app.snapshot.initramfs.presets = vec!["linux.preset".into()];
  app.page = BootPage::Initramfs;
  let rows = app.rows();
  assert_eq!(ids(&rows), [Item::Preset(0), Item::RegenerateInitramfs]);
  assert!(rows[rows.len() - 2].is_section());
  assert_eq!(rows.last().unwrap().kind(), RowKind::Destructive);
  press(&mut app, Item::RegenerateInitramfs);
  assert!(matches!(
    app.pending,
    Some(Pending::Action(BootAction::Initramfs))
  ));
  app.handle(KeyCode::Esc);
  press(&mut app, Item::Preset(0));
  assert_eq!(app.page, BootPage::InitramfsDetail(0));
  assert!(app.rows().iter().all(|row| !row.is_selectable()));
  app.handle(KeyCode::Esc);
  assert_eq!(app.menu.selected_id(&app.rows()), Some(Item::Preset(0)));
}

#[test]
fn plymouth_themes_are_confirmed_choices() {
  let mut app = app();
  app.snapshot.plymouth.themes = vec!["argvus".into(), "spinner".into()];
  app.snapshot.plymouth.current_theme = Some("argvus".into());
  app.page = BootPage::Plymouth;
  let rows = app.rows();
  assert_eq!(rows[0].kind(), RowKind::Choice { current: true });
  assert_eq!(rows[1].kind(), RowKind::Choice { current: false });
  press(&mut app, Item::Theme(1));
  assert!(matches!(
    &app.pending,
    Some(Pending::Action(BootAction::Plymouth(theme))) if theme == "spinner"
  ));
}

#[test]
fn summary_is_read_only_and_has_no_cursor() {
  let mut app = systemd_app();
  app.page = BootPage::Summary;
  let rows = app.rows();
  assert!(rows.iter().all(|row| !row.is_selectable()));
  assert!(
    rows
      .iter()
      .any(|row| row.detail_text() == Some("systemd-boot"))
  );
  app.normalize();
  assert!(app.menu.is_scroll_only());
}

#[test]
fn tab_does_nothing_without_tabs() {
  let mut app = systemd_app();
  app.page = BootPage::Bootloader;
  app.handle(KeyCode::Down);
  let before = app.menu;
  app.handle(KeyCode::Tab);
  app.handle(KeyCode::BackTab);
  assert_eq!(app.menu, before);
  assert_eq!(app.page, BootPage::Bootloader);
}

#[test]
fn boot_uses_shared_chrome_and_contextual_footer() {
  let mut app = app();
  let text = screen(&mut app, 90, 25);
  assert!(text.contains("ARGVUS"));
  assert!(text.contains("Boot"));
  assert!(text.contains(tr(app.lang, "control_center.hint.open")));
  assert!(text.contains(tr(app.lang, "control_center.hint.refresh")));
  assert!(text.contains("Summary") || text.contains("Resumo"));
}

#[test]
fn boot_pages_have_no_button_bar() {
  let mut app = app();
  app.snapshot.bootloader = BootloaderKind::Grub;
  app.page = BootPage::Bootloader;
  let text = screen(&mut app, 90, 25);
  assert!(!text.contains("[ "), "{text}");
  assert!(text.contains(tr(app.lang, "control_center.kernel_command_line")));
}
