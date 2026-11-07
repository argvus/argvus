use super::*;
use std::time::{Duration, Instant};

fn start(script: &str) -> (PackageTerminal, thread::JoinHandle<Result<String, String>>) {
  let (terminal, io) = PackageTerminal::new(10, 80);
  let mut command = CommandBuilder::new("/bin/sh");
  command.args(["-c", script]);
  let worker = thread::spawn(move || io.run_command(command));
  (terminal, worker)
}

fn wait_for(terminal: &mut PackageTerminal, condition: impl Fn(&PackageTerminal) -> bool) {
  let deadline = Instant::now() + Duration::from_secs(5);
  loop {
    terminal.poll();
    if condition(terminal) {
      return;
    }
    assert!(
      Instant::now() < deadline,
      "terminal did not reach the expected state"
    );
    thread::sleep(Duration::from_millis(5));
  }
}

#[test]
fn real_pty_displays_unterminated_prompts_and_accepts_yes_or_no() {
  for answer in ["y", "n"] {
    let (mut terminal, worker) = start(
      r#"printf 'Proceed? [Y/n] '; IFS= read -r answer; printf '\033[32manswer=%s\033[0m\n' "$answer"; test "$answer" = y"#,
    );
    wait_for(&mut terminal, |terminal| {
      terminal.view.contains_text("Proceed? [Y/n]")
    });
    assert!(!worker.is_finished(), "the prompt must wait for real input");
    // Terminal line editing must reach the line discipline, not the list filter.
    terminal
      .write(format!("{answer}x\x7f\r").into_bytes())
      .unwrap();
    wait_for(&mut terminal, |terminal| {
      terminal.view.contains_text(&format!("answer={answer}"))
    });
    wait_for(&mut terminal, |_| worker.is_finished());
    assert_eq!(worker.join().unwrap().is_ok(), answer == "y");
  }
}

#[test]
fn resize_changes_the_child_terminal_and_does_not_lose_the_prompt() {
  let (mut terminal, worker) =
    start("printf 'resize-ready'; read answer; stty size; printf 'resize-done'");
  wait_for(&mut terminal, |terminal| {
    terminal.view.contains_text("resize-ready")
  });
  terminal.resize(12, 55).unwrap();
  assert!(terminal.view.contains_text("resize-ready"));
  terminal.write(b"\r".to_vec()).unwrap();
  wait_for(&mut terminal, |terminal| {
    terminal.view.contains_text("12 55")
  });
  wait_for(&mut terminal, |_| worker.is_finished());
  assert!(worker.join().unwrap().is_ok());
}

#[test]
fn ctrl_c_interrupts_the_child_without_closing_the_view() {
  let (mut terminal, worker) = start("printf 'interrupt-ready'; read answer");
  wait_for(&mut terminal, |terminal| {
    terminal.view.contains_text("interrupt-ready")
  });
  terminal.write(vec![3]).unwrap();
  wait_for(&mut terminal, |_| worker.is_finished());
  assert!(worker.join().unwrap().is_err());
  assert!(terminal.view.contains_text("interrupt-ready"));
}

#[test]
fn missing_executable_fails_without_waiting_for_input() {
  let (mut terminal, io) = PackageTerminal::new(10, 80);
  let result = io.run_command(CommandBuilder::new(
    "/nonexistent/argvus-package-terminal-test",
  ));
  assert!(result.unwrap_err().contains("terminal command"));
  assert!(!terminal.poll());
}

#[test]
fn history_is_bounded_and_scrolling_does_not_discard_the_live_screen() {
  let (mut terminal, _io) = PackageTerminal::new(5, 40);
  for index in 0..12_000 {
    terminal
      .view
      .process(format!("line {index}\r\n").as_bytes());
  }
  terminal.view.set_scrollback(usize::MAX);
  assert_eq!(terminal.view.scrollback(), HISTORY_LINES);
  terminal.view.set_scrollback(0);
  assert!(terminal.view.contains_text("line 11999"));
}

#[test]
fn input_encoding_preserves_control_keys_unicode_and_application_cursors() {
  for (code, modifiers, expected) in [
    (KeyCode::Char('c'), KeyModifiers::CONTROL, vec![3]),
    (KeyCode::Char('q'), KeyModifiers::NONE, b"q".to_vec()),
    (KeyCode::Char('?'), KeyModifiers::NONE, b"?".to_vec()),
    (
      KeyCode::Char('é'),
      KeyModifiers::NONE,
      "é".as_bytes().to_vec(),
    ),
    (KeyCode::Enter, KeyModifiers::NONE, vec![b'\r']),
    (KeyCode::Backspace, KeyModifiers::NONE, vec![0x7f]),
    (KeyCode::Left, KeyModifiers::NONE, b"\x1b[D".to_vec()),
    (KeyCode::Char('x'), KeyModifiers::ALT, b"\x1bx".to_vec()),
  ] {
    assert_eq!(
      encode_key(KeyEvent::new(code, modifiers), false),
      Some(expected)
    );
  }
  assert_eq!(
    encode_key(KeyCode::Up.into(), true),
    Some(b"\x1bOA".to_vec())
  );
  assert_eq!(
    encode_key(KeyCode::Esc.into(), false),
    None,
    "Esc belongs to the frame"
  );
}

#[test]
fn paste_respects_bracketed_mode_and_input_is_bounded() {
  let (mut terminal, io) = PackageTerminal::new(5, 40);
  terminal.view.process(b"\x1b[?2004h");
  terminal.paste("hello\nworld").unwrap();
  assert_eq!(io.next_input().unwrap(), b"\x1b[200~hello\nworld\x1b[201~");
  terminal.view.process(b"\x1b[?2004l");
  terminal.paste("y\r\nn").unwrap();
  assert_eq!(io.next_input().unwrap(), b"y\rn");
  assert!(terminal.write(vec![b'x'; MAX_INPUT_BYTES + 1]).is_err());
  for _ in 0..INPUT_MESSAGES {
    terminal.write(vec![b'y']).unwrap();
  }
  assert!(terminal.write(vec![b'n']).is_err());
  assert_eq!(io.next_input(), Some(vec![b'y']));
}

#[test]
fn output_backpressure_does_not_block_ui_polling() {
  let (mut terminal, io) = PackageTerminal::new(5, 40);
  for _ in 0..OUTPUT_CHUNKS {
    io.emit(b"line\r\n");
  }
  assert!(terminal.poll());
  assert!(terminal.view.contains_text("line"));
  assert!(terminal.poll());
}

#[test]
fn denied_or_cancelled_authentication_exit_codes_are_failures() {
  // pkexec uses these statuses for denial/cancellation. No authentication runs.
  for status in [126, 127] {
    let (mut terminal, worker) = start(&format!(
      "printf 'authorization unavailable'; exit {status}"
    ));
    wait_for(&mut terminal, |terminal| {
      terminal.view.contains_text("authorization unavailable")
    });
    wait_for(&mut terminal, |_| worker.is_finished());
    assert_eq!(
      worker.join().unwrap(),
      Err("control_center.package_transaction_failed".into())
    );
    assert!(terminal.view.contains_text("authorization unavailable"));
  }
}

#[test]
fn success_message_follows_all_pending_output_and_is_appended_once() {
  let (mut terminal, io) = PackageTerminal::new(8, 60);
  for _ in 0..32 {
    io.emit(b"progress\r\n");
  }
  terminal.finish_success("Done!");
  assert!(!terminal.view.contains_text("Done!"));
  // Even after process completion, the reader may still have a final chunk.
  io.emit(b"last output\r\n");
  drop(io);
  terminal.poll();
  assert!(!terminal.view.contains_text("Done!"));
  assert!(terminal.poll());
  assert!(terminal.view.contains_text("last output"));
  assert!(terminal.view.contains_text("Done!"));
  assert!(!terminal.poll());
  terminal.finish_success("Done!");
  assert!(!terminal.poll());
}

#[test]
fn real_pty_success_appends_done_after_the_reader_finishes() {
  let (mut terminal, worker) = start("printf 'final command output'");
  wait_for(&mut terminal, |_| worker.is_finished());
  assert!(worker.join().unwrap().is_ok());
  terminal.finish_success("Done!");
  wait_for(&mut terminal, |terminal| {
    terminal.view.contains_text("Done!")
  });
  assert!(terminal.view.contains_text("final command output"));
}
