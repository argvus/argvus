//! Revert countdown after a risky change: deadline, keys, footer and dialog.

use super::*;

#[test]
fn risky_persist_changes_arm_revert() {
  assert!(option_is_risky(&PersistChange::Mode("x".into())));
  assert!(option_is_risky(&PersistChange::Position(1, 2)));
  assert!(option_is_risky(&PersistChange::Disabled(true)));
  assert!(option_is_risky(&PersistChange::Mirror(String::new())));
  assert!(option_is_risky(&PersistChange::BitDepth(10)));
  assert!(!option_is_risky(&PersistChange::Scale(1.0)));
  assert!(!option_is_risky(&PersistChange::Vrr(1)));
  assert!(!option_is_risky(&PersistChange::Hdr(1)));
  assert!(!option_is_risky(&PersistChange::Workspace(3, None)));
}

#[test]
fn revert_window_is_fifteen_seconds_from_the_change() {
  assert_eq!(REVERT_SECONDS, 15);
  let now = Instant::now();
  let app = armed(now);
  assert_eq!(
    app.revert.as_ref().unwrap().deadline,
    now + Duration::from_secs(15)
  );
}

#[test]
fn revert_keys_follow_the_confirmation_component() {
  let now = Instant::now();
  let reverts = |key: KeyCode| {
    let mut app = armed(now);
    matches!(app.answer_revert(key), RevertAnswer::Revert(revert)
      if revert.previous_config == previous_config())
  };
  assert!(reverts(KeyCode::Enter), "Enter starts on Revert");
  assert!(reverts(KeyCode::Char('n')));
  assert!(reverts(KeyCode::Esc));

  let mut app = armed(now);
  assert!(matches!(
    app.answer_revert(KeyCode::Char('y')),
    RevertAnswer::Keep(name) if name == "eDP-1"
  ));
  assert!(app.revert.is_none());

  let mut app = armed(now);
  assert!(matches!(
    app.answer_revert(KeyCode::Char(' ')),
    RevertAnswer::Pending
  ));
  assert!(app.revert.is_some(), "Space has no effect");
  assert!(matches!(
    app.answer_revert(KeyCode::Up),
    RevertAnswer::Pending
  ));
  assert!(matches!(
    app.answer_revert(KeyCode::Enter),
    RevertAnswer::Keep(_)
  ));
}

#[test]
fn revert_footer_puts_y_keep_first() {
  let app = armed(Instant::now());
  let footer = app.footer_hints(&app.rows());
  assert!(
    footer.starts_with(&format!("y {}", tr(app.lang, "control_center.keep"))),
    "{footer}"
  );
}

#[test]
fn revert_dialog_renders_with_the_countdown() {
  use ratatui::{Terminal, backend::TestBackend};
  let mut app = armed(Instant::now());
  let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
  terminal.draw(|frame| app.draw(frame)).unwrap();
  let rendered: String = terminal
    .backend()
    .buffer()
    .content
    .iter()
    .map(|cell| cell.symbol())
    .collect();
  assert!(rendered.contains(tr(app.lang, "control_center.keep")));
  assert!(rendered.contains(tr(app.lang, "control_center.revert")));
  assert!(rendered.contains(" s"), "remaining seconds are shown");
}
