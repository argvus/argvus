//! Kernel command line field: text input, length limit and rejected characters.

use super::*;

#[test]
fn kernel_command_line_field_takes_text() {
  let mut app = BootApp::new(
    Lang::for_locale("en-US"),
    Theme::load(),
    Capabilities::default(),
  );
  app.snapshot.bootloader = BootloaderKind::Grub;
  app.snapshot.bootloader_info.grub_values =
    vec![("GRUB_CMDLINE_LINUX_DEFAULT".into(), "loglevel=3".into())];
  app.timeout_input = Some(app.grub_value("GRUB_CMDLINE_LINUX_DEFAULT"));
  app.input_mode = Some(InputMode::GrubCmdline);
  for character in " quiet splash".chars() {
    app.handle(KeyCode::Char(character));
  }
  assert_eq!(
    app.timeout_input.as_deref(),
    Some("loglevel=3 quiet splash")
  );
  app.handle(KeyCode::Backspace);
  app.handle(KeyCode::Char('h'));
  app.handle(KeyCode::Enter);
  assert!(matches!(
    &app.pending,
    Some(Pending::Action(BootAction::GrubCmdline(value))) if value == "loglevel=3 quiet splash"
  ));
}

#[test]
fn kernel_command_line_field_stops_at_the_backend_limit() {
  let full = "a".repeat(KERNEL_CMDLINE_MAX);
  assert!(!accepts_input_char(
    Some(InputMode::GrubCmdline),
    &full,
    'b'
  ));
  assert!(accepts_input_char(
    Some(InputMode::GrubCmdline),
    &full[1..],
    'b'
  ));
  assert!(!accepts_input_char(Some(InputMode::Timeout), "12", '3'));
  assert!(!accepts_input_char(Some(InputMode::Timeout), "", 'a'));
}

#[test]
fn kernel_command_line_rejects_quotes_when_applied() {
  let mut app = BootApp::new(
    Lang::for_locale("en-US"),
    Theme::load(),
    Capabilities::default(),
  );
  app.timeout_input = Some(String::new());
  app.input_mode = Some(InputMode::GrubCmdline);
  app.handle(KeyCode::Char('"'));
  app.handle(KeyCode::Enter);
  assert!(app.pending.is_none());
  assert!(
    app
      .status
      .as_ref()
      .is_some_and(|status| status.kind == StatusKind::Error)
  );
}

#[test]
fn input_tail_keeps_the_cursor_visible() {
  assert_eq!(input_tail("quiet", 10), "quiet_");
  assert_eq!(input_tail("loglevel=3 quiet splash", 7), "splash_");
}
