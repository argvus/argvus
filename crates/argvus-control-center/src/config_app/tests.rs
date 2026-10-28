use super::*;
use argvus_control_center_core::process::{ProcessError, ProcessOutput};
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;
use std::time::Duration;

static ENV_TEST_LOCK: Mutex<()> = Mutex::new(());
static ENV_TEST_SEQUENCE: AtomicUsize = AtomicUsize::new(0);

#[derive(Default)]
struct RecordingOperation(Mutex<Option<PrivilegedRequest>>);

impl PrivilegedOperation for RecordingOperation {
  fn execute(&self, request: &PrivilegedRequest) -> Result<ProcessOutput, String> {
    *self.0.lock().unwrap() = Some(request.clone());
    Ok(ProcessOutput {
      stdout: Vec::new(),
      stderr: Vec::new(),
      status: Some(0),
      timed_out: false,
    })
  }
}

#[derive(Default)]
struct FailingOperation;

impl PrivilegedOperation for FailingOperation {
  fn execute(&self, _request: &PrivilegedRequest) -> Result<ProcessOutput, String> {
    Err("simulated denial".into())
  }
}

#[derive(Default)]
struct RecordingRunner(Mutex<Option<ProcessRequest>>);

impl ProcessRunner for RecordingRunner {
  fn run(&self, request: &ProcessRequest) -> Result<ProcessOutput, ProcessError> {
    *self.0.lock().unwrap() = Some(request.clone());
    Ok(ProcessOutput {
      stdout: Vec::new(),
      stderr: Vec::new(),
      status: Some(0),
      timed_out: false,
    })
  }
}

/// Uses a hermetic runner so effect reads never touch the real system.
fn with_operation(operation: ConfigOperation) -> ConfigApp {
  ConfigApp::from_inputs(
    Lang::for_locale("pt-BR"),
    Theme::load(),
    operation,
    Arc::new(RecordingRunner::default()),
  )
}

/// Acquires `ENV_TEST_LOCK`, recovering from a poisoned state left behind by
/// a previously failed test; the env var each test guards is reset between
/// runs, so a stale poison is safe to ignore.
fn env_test_guard() -> std::sync::MutexGuard<'static, ()> {
  ENV_TEST_LOCK
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn with_config(on_icons: bool) -> (PathBuf, PathBuf) {
  let sequence = ENV_TEST_SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
  let dir = std::env::temp_dir().join(format!(
    "argvus-cc-config-app-{}-{sequence}",
    std::process::id()
  ));
  std::fs::create_dir_all(&dir).unwrap();
  let path = dir.join("config.toml");
  std::fs::write(
    &path,
    if on_icons {
      "[Appearance]\nicons = true\n"
    } else {
      "[Appearance]\nicons = false\n"
    },
  )
  .unwrap();
  (dir, path)
}

fn drain_poll(app: &mut ConfigApp) {
  for _ in 0..200 {
    if app.poll() {
      return;
    }
    std::thread::sleep(Duration::from_millis(2));
  }
  panic!("config save job did not finish");
}

#[test]
fn toggling_icons_flips_the_session_and_requests_a_privileged_save() {
  let _guard = env_test_guard();
  let (dir, path) = with_config(true);
  unsafe { std::env::set_var("ARGVUS_CONFIG_PATH", &path) };
  let recorder = Arc::new(RecordingOperation::default());
  let mut app = with_operation(Arc::clone(&recorder) as ConfigOperation);
  assert!(app.icons);
  app.toggle();
  assert!(!app.icons);
  // The live icon flag is process-wide, and every `App::new` in the tests
  // running in parallel reloads it from disk, so reading it here races with
  // them. `set_icons_matches_disk_and_session_state` in the core crate
  // covers it under that crate's own lock.
  drain_poll(&mut app);
  let requested = recorder.0.lock().unwrap().clone().unwrap();
  assert_eq!(requested.domain, "config");
  assert_eq!(requested.action, "icons");
  assert_eq!(requested.arguments, vec![String::from("false")]);
  assert!(matches!(
    app.status,
    Some(StatusMessage {
      kind: StatusKind::Success,
      ..
    })
  ));
  unsafe { std::env::remove_var("ARGVUS_CONFIG_PATH") };
  let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn failed_save_reports_applied_this_session_warning() {
  let _guard = env_test_guard();
  let (dir, path) = with_config(true);
  unsafe { std::env::set_var("ARGVUS_CONFIG_PATH", &path) };
  let mut app = with_operation(Arc::new(FailingOperation));
  app.toggle();
  assert!(!app.icons);
  drain_poll(&mut app);
  let Some(message) = app.status else {
    panic!("expected a status message");
  };
  assert_eq!(message.kind, StatusKind::Warning);
  assert!(message.text.contains("simulated denial"), "{message:?}?");
  unsafe { std::env::remove_var("ARGVUS_CONFIG_PATH") };
  let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn checkbox_label_tracks_icon_state() {
  let _guard = env_test_guard();
  let (dir, path) = with_config(true);
  unsafe { std::env::set_var("ARGVUS_CONFIG_PATH", &path) };
  let app = with_operation(Arc::new(RecordingOperation::default()));
  let row = app.rows().into_iter().next().unwrap();
  assert!(row.starts_with("[x] Ícones"), "{row}");
  unsafe { std::env::remove_var("ARGVUS_CONFIG_PATH") };
  let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn escape_leaves_and_enter_toggles() {
  let _guard = env_test_guard();
  let (dir, path) = with_config(true);
  unsafe { std::env::set_var("ARGVUS_CONFIG_PATH", &path) };
  let mut app = with_operation(Arc::new(RecordingOperation::default()));
  assert!(app.handle(KeyCode::Esc), "Esc must signal going back");
  assert!(!app.handle(KeyCode::Enter));
  assert!(!app.icons);
  unsafe { std::env::remove_var("ARGVUS_CONFIG_PATH") };
  let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn transparency_toggle_applies_the_control_center_surface() {
  let _guard = env_test_guard();
  let (dir, path) = with_config(true);
  unsafe { std::env::set_var("ARGVUS_CONFIG_PATH", &path) };
  let recorder = Arc::new(RecordingRunner::default());
  let mut app = ConfigApp::from_inputs(
    Lang::for_locale("pt-BR"),
    Theme::load(),
    Arc::new(RecordingOperation::default()),
    Arc::clone(&recorder) as Arc<dyn ProcessRunner + Send + Sync>,
  );
  assert!(app.transparency);
  assert!(!app.handle(KeyCode::Down));
  assert!(!app.handle(KeyCode::Enter));
  assert!(!app.transparency);
  drain_poll(&mut app);
  let requested = recorder.0.lock().unwrap().clone().unwrap();
  assert_eq!(requested.program, "argvus");
  assert_eq!(requested.args[0], "effects-toggle");
  let tail = requested.args[requested.args.len() - 6..].to_vec();
  assert_eq!(
    tail,
    vec![
      "surface-apply",
      "control-center",
      "disabled",
      "15",
      "enabled",
      "50"
    ]
  );
  assert!(matches!(
    app.status,
    Some(StatusMessage {
      kind: StatusKind::Success,
      ..
    })
  ));
  unsafe { std::env::remove_var("ARGVUS_CONFIG_PATH") };
  let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn left_and_right_step_the_transparency_value() {
  let _guard = env_test_guard();
  let (dir, path) = with_config(true);
  unsafe { std::env::set_var("ARGVUS_CONFIG_PATH", &path) };
  let recorder = Arc::new(RecordingRunner::default());
  let mut app = ConfigApp::from_inputs(
    Lang::for_locale("pt-BR"),
    Theme::load(),
    Arc::new(RecordingOperation::default()),
    Arc::clone(&recorder) as Arc<dyn ProcessRunner + Send + Sync>,
  );
  assert!(!app.handle(KeyCode::Down));
  assert!(!app.handle(KeyCode::Right));
  assert_eq!(app.transparency_value, 20);
  drain_poll(&mut app);
  let requested = recorder.0.lock().unwrap().clone().unwrap();
  let tail = requested.args[requested.args.len() - 6..].to_vec();
  assert_eq!(
    tail,
    vec![
      "surface-apply",
      "control-center",
      "enabled",
      "20",
      "enabled",
      "50"
    ]
  );
  assert!(!app.handle(KeyCode::Left));
  assert_eq!(app.transparency_value, 15);
  drain_poll(&mut app);
  let requested = recorder.0.lock().unwrap().clone().unwrap();
  let tail = requested.args[requested.args.len() - 6..].to_vec();
  assert_eq!(
    tail,
    vec![
      "surface-apply",
      "control-center",
      "enabled",
      "15",
      "enabled",
      "50"
    ]
  );
  assert!(!app.handle(KeyCode::Left));
  assert_eq!(app.transparency_value, 10);
  drain_poll(&mut app);
  let requested = recorder.0.lock().unwrap().clone().unwrap();
  let tail = requested.args[requested.args.len() - 6..].to_vec();
  assert_eq!(
    tail,
    vec![
      "surface-apply",
      "control-center",
      "enabled",
      "10",
      "enabled",
      "50"
    ]
  );
  unsafe { std::env::remove_var("ARGVUS_CONFIG_PATH") };
  let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn draws_the_checkbox_screen() {
  let _guard = env_test_guard();
  let (dir, path) = with_config(true);
  unsafe { std::env::set_var("ARGVUS_CONFIG_PATH", &path) };
  let app = with_operation(Arc::new(RecordingOperation::default()));
  let mut terminal = ratatui::Terminal::new(ratatui::backend::TestBackend::new(90, 24)).unwrap();
  terminal.draw(|frame| app.draw(frame)).unwrap();
  let text = terminal
    .backend()
    .buffer()
    .content
    .iter()
    .map(|cell| cell.symbol())
    .collect::<String>();
  assert!(text.contains("ARGVUS"));
  assert!(text.contains("Estilo") || text.contains("Style"));
  assert!(text.contains("[x]") || text.contains("[ ]"));
  assert!(text.contains("Ícones") || text.contains("Icons"));
  unsafe { std::env::remove_var("ARGVUS_CONFIG_PATH") };
  let _ = std::fs::remove_dir_all(&dir);
}
