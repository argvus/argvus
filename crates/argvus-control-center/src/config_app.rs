//! Implements `config app` responsibilities in crate `argvus control center`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use std::sync::Arc;

use argvus_control_center_core::{
  config::AppConfig,
  jobs::{JobHandle, JobManager, JobState},
  privileged::{PrivilegedOperation, PrivilegedRequest, SystemSettingsOperation},
  process::{ProcessRequest, ProcessRunner, SystemProcessRunner},
  sanitize::terminal_text,
};
use argvus_i18n::{Lang, tr};
use argvus_theme::Theme;
use argvus_tui::{
  components::{StatusKind, StatusMessage},
  page::{list, readonly, shell, status},
};
use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::text::Line;

/// Names the type `ConfigOperation`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
type ConfigOperation = Arc<dyn PrivilegedOperation>;

/// The ARGVUS Control Center configuration screen. Hosts the option that
/// enables/disables the decorative icons across the whole layout plus the
/// transparency control for the Control Center window itself. Blur is owned by
/// Hyprland > Blur and is only carried through unchanged.
pub struct ConfigApp {
  lang: Lang,
  theme: Theme,
  pub icons: bool,
  pub transparency: bool,
  pub transparency_value: i32,
  /// Blur flag read from argvus-config and sent back unchanged, so saving the
  /// transparency does not overwrite it.
  blur_kept: bool,
  selected: usize,
  pub status: Option<StatusMessage>,
  operation: ConfigOperation,
  runner: Arc<dyn ProcessRunner + Send + Sync>,
  jobs: JobManager,
  save_job: Option<JobHandle<()>>,
  effects_job: Option<JobHandle<()>>,
}

impl ConfigApp {
  /// Replaces the semantic theme used by this page.
  pub fn set_theme(&mut self, theme: &Theme) {
    self.theme = theme.clone();
  }

  /// Constructs `new` with this module's expected initial state. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn new(lang: Lang, theme: Theme) -> Self {
    let executable = std::env::current_exe()
      .map(|path| path.to_string_lossy().into_owned())
      .unwrap_or_else(|_| "argvus-control-center".into());
    Self::from_inputs(
      lang,
      theme,
      Arc::new(SystemSettingsOperation::new(
        SystemProcessRunner,
        executable,
      )),
      Arc::new(SystemProcessRunner),
    )
  }

  /// Constructs `from_inputs` with this module's expected initial state. The `runner` handles the user-owned effects hops so the Control Center effects can be persisted without privilege elevation. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn from_inputs(
    lang: Lang,
    theme: Theme,
    operation: ConfigOperation,
    runner: Arc<dyn ProcessRunner + Send + Sync>,
  ) -> Self {
    let config = AppConfig::load();
    let (transparency, transparency_value, blur_kept) = load_effects(&runner);
    Self {
      lang,
      theme,
      icons: config.icons(),
      transparency,
      transparency_value,
      blur_kept,
      selected: 0,
      status: None,
      operation,
      runner,
      jobs: JobManager::default(),
      save_job: None,
      effects_job: None,
    }
  }

  /// Executes the `rows` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn rows(&self) -> Vec<String> {
    vec![
      format!(
        "[{}] {}",
        checkbox(self.icons),
        tr(self.lang, "control_center.icons")
      ),
      format!(
        "[{}] {} {}%",
        checkbox(self.transparency),
        tr(self.lang, "control_center.control_center_transparency"),
        self.transparency_value
      ),
    ]
  }

  /// Executes the `breadcrumb` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn breadcrumb(&self) -> String {
    format!(
      "{} > {}",
      tr(self.lang, "control_center.settings"),
      tr(self.lang, "control_center.style")
    )
  }

  /// Executes the `footer_hints` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn footer_hints(&self) -> &'static str {
    tr(
      self.lang,
      "control_center.navigate_enter_space_toggle_left_right_value_esc_back_help",
    )
  }

  /// Processes `handle` in this module's event flow. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn handle(&mut self, key: KeyCode) -> bool {
    match key {
      KeyCode::Esc => true,
      KeyCode::Left => {
        if self.selected == 1 {
          self.adjust_value(-5);
          false
        } else {
          true
        }
      }
      KeyCode::Right => {
        if self.selected == 1 {
          self.adjust_value(5);
        }
        false
      }
      KeyCode::Enter | KeyCode::Char(' ') => {
        match self.selected {
          0 => self.toggle(),
          1 => {
            self.transparency = !self.transparency;
            self.save_effects();
          }
          _ => {}
        }
        false
      }
      KeyCode::Up | KeyCode::Char('k') | KeyCode::Home | KeyCode::PageUp => {
        self.selected = self.selected.saturating_sub(1);
        self.normalize();
        false
      }
      KeyCode::Down | KeyCode::Char('j') | KeyCode::End | KeyCode::PageDown => {
        self.selected = self.selected.saturating_add(1);
        self.normalize();
        false
      }
      _ => false,
    }
  }

  /// Executes the `normalize` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn normalize(&mut self) {
    self.selected = self.selected.min(self.rows().len().saturating_sub(1));
  }

  /// Applies the icons toggle in-session immediately and persists it through the
  /// privileged system-settings flow (polkit). Result is reported on poll().
  pub fn toggle(&mut self) {
    if self.save_job.is_some() {
      return;
    }
    self.icons = !self.icons;
    AppConfig::set_session_icons(self.icons);
    self.status = Some(StatusMessage {
      kind: StatusKind::Info,
      text: tr(self.lang, "control_center.saving_configuration").into(),
    });
    let operation = Arc::clone(&self.operation);
    let enabled = self.icons.to_string();
    let request = match PrivilegedRequest::new("config", "icons", vec![enabled]) {
      Ok(request) => request,
      Err(error) => {
        self.persist_failed(format!(
          "{}: {error}",
          tr(
            self.lang,
            "control_center.applied_this_session_but_the_configuration_could_not_be_saved"
          )
        ));
        return;
      }
    };
    self.save_job = Some(self.jobs.spawn(move |_| {
      let output = operation.execute(&request)?;
      if output.status == Some(0) {
        Ok(())
      } else {
        Err(terminal_text(
          String::from_utf8_lossy(&output.stderr).trim(),
        ))
      }
    }));
  }

  /// Persists the Control Center transparency through the
  /// `argvus effects-toggle` surface-apply route. The command writes the
  /// argvus-config keys, rematerializes the tab-less Kitty profile and asks
  /// running instances to reload, so the running window updates live.
  fn save_effects(&mut self) {
    if self.effects_job.is_some() {
      return;
    }
    self.status = Some(StatusMessage {
      kind: StatusKind::Info,
      text: tr(self.lang, "control_center.saving_configuration").into(),
    });
    let transparency_enabled = if self.transparency {
      "enabled"
    } else {
      "disabled"
    };
    let transparency_value = self.transparency_value.to_string();
    let blur_enabled = if self.blur_kept {
      "enabled"
    } else {
      "disabled"
    };
    // The Control Center surface reads the blur enable flag only; the blur
    // value argument is still required to keep the shared surface contract.
    let blur_value = "50";
    let runner = Arc::clone(&self.runner);
    self.effects_job = Some(self.jobs.spawn(move |_| {
      let request = ProcessRequest::new("argvus")
        .arg("effects-toggle")
        .arg("surface-apply")
        .arg("control-center")
        .arg(transparency_enabled)
        .arg(transparency_value)
        .arg(blur_enabled)
        .arg(blur_value);
      let output = runner.run(&request).map_err(|error| error.to_string())?;
      if output.status == Some(0) {
        Ok(())
      } else {
        Err(terminal_text(
          String::from_utf8_lossy(&output.stderr).trim(),
        ))
      }
    }));
  }

  /// Steps `transparency_value` by `delta` while keeping it inside its range and
  /// persists the new value immediately.
  fn adjust_value(&mut self, delta: i32) {
    self.transparency_value = (self.transparency_value + delta).clamp(0, 100);
    self.save_effects();
  }

  /// Executes the `persist_failed` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn persist_failed(&mut self, text: String) {
    self.status = Some(StatusMessage {
      kind: StatusKind::Warning,
      text,
    });
  }

  /// Executes the `poll` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn poll(&mut self) -> bool {
    let mut changed = false;
    if let Some(job) = &self.save_job
      && let JobState::Finished(result) = job.try_state()
    {
      self.save_job = None;
      match result {
        Ok(()) => {
          self.status = Some(StatusMessage {
            kind: StatusKind::Success,
            text: if self.icons {
              tr(self.lang, "control_center.icons_enabled").into()
            } else {
              tr(self.lang, "control_center.icons_disabled").into()
            },
          });
        }
        Err(error) => {
          self.persist_failed(format!(
            "{}: {error}",
            tr(
              self.lang,
              "control_center.applied_this_session_but_the_configuration_could_not_be_saved"
            )
          ));
        }
      }
      changed = true;
    }
    if let Some(job) = &self.effects_job
      && let JobState::Finished(result) = job.try_state()
    {
      self.effects_job = None;
      match result {
        Ok(()) => {
          self.status = Some(StatusMessage {
            kind: StatusKind::Success,
            text: tr(self.lang, "control_center.effects_applied").into(),
          });
        }
        Err(error) => {
          self.persist_failed(format!(
            "{}: {error}",
            tr(
              self.lang,
              "control_center.applied_this_session_but_the_configuration_could_not_be_saved"
            )
          ));
        }
      }
      changed = true;
    }
    changed
  }

  /// Renders `draw` while respecting the current domain state and semantic theme. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn draw(&self, frame: &mut Frame) {
    let area = frame.area();
    let body = shell(
      frame,
      area,
      &self.theme,
      self.lang,
      &self.breadcrumb(),
      self.footer_hints(),
    );
    let split = Layout::vertical([Constraint::Length(1), Constraint::Min(1)]).split(body);
    readonly(
      frame,
      split[0],
      &self.theme,
      &[Line::from(format!(
        "  {}",
        tr(self.lang, "control_center.style")
      ))],
    );
    let rows = self.rows();
    list(
      frame,
      split[1],
      &self.theme,
      &rows,
      self.selected.min(rows.len().saturating_sub(1)),
    );
    if let Some(status_message) = &self.status {
      status(frame, body, &self.theme, status_message);
    }
  }
}

/// Renders the checkbox glyph for `enabled`. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn checkbox(enabled: bool) -> &'static str {
  // Same `[x]` / `[ ]` convention used by every selection list in the Control Center.
  if enabled { "x" } else { " " }
}

/// Reads the Control Center effect keys from argvus-config in a best-effort
/// way. When the tool is unavailable or the keys are missing, the session
/// defaults are used so the screen still reflects what the effect path applies.
fn load_effects(runner: &Arc<dyn ProcessRunner + Send + Sync>) -> (bool, i32, bool) {
  let transparency = read_effect_flag(runner, "transparency_control-center_enabled", true);
  let transparency_value = read_effect_value(runner, "transparency_control-center_value", 50);
  let blur = read_effect_flag(runner, "blur_control-center_enabled", true);
  (transparency, transparency_value, blur)
}

/// Executes `argvus config get` for a single effects key and returns the raw
/// effective value when the process succeeds.
fn read_effect(runner: &Arc<dyn ProcessRunner + Send + Sync>, key: &str) -> Option<String> {
  let request = ProcessRequest::new("argvus")
    .arg("config")
    .arg("get")
    .arg(format!("/effects/{key}"))
    .arg("--effective")
    .arg("--raw");
  match runner.run(&request) {
    Ok(output) if output.status == Some(0) => {
      let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
      if value.is_empty() { None } else { Some(value) }
    }
    _ => None,
  }
}

/// Parses `read_effect` as an enabled/disabled flag with a `default` fallback.
fn read_effect_flag(
  runner: &Arc<dyn ProcessRunner + Send + Sync>,
  key: &str,
  default: bool,
) -> bool {
  match read_effect(runner, key).as_deref() {
    Some("true") | Some("enabled") => true,
    Some("false") | Some("disabled") => false,
    _ => default,
  }
}

/// Parses `read_effect` as a percentage value with a `default` fallback.
fn read_effect_value(
  runner: &Arc<dyn ProcessRunner + Send + Sync>,
  key: &str,
  default: i32,
) -> i32 {
  match read_effect(runner, key) {
    Some(value) => value
      .parse::<i32>()
      .ok()
      .filter(|parsed| (0..=100).contains(parsed))
      .unwrap_or(default),
    None => default,
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use argvus_control_center_core::process::{ProcessError, ProcessOutput};
  use std::path::PathBuf;
  use std::sync::Mutex;
  use std::sync::atomic::AtomicUsize;
  use std::time::Duration;

  /// Maintains the static state `ENV_TEST_LOCK`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
  static ENV_TEST_LOCK: Mutex<()> = Mutex::new(());
  /// Maintains the static state `ENV_TEST_SEQUENCE`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
  static ENV_TEST_SEQUENCE: AtomicUsize = AtomicUsize::new(0);

  #[derive(Default)]
  /// Represents `RecordingOperation`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
  struct RecordingOperation(Mutex<Option<PrivilegedRequest>>);

  impl PrivilegedOperation for RecordingOperation {
    /// Executes the `execute` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
  /// Represents `FailingOperation`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
  struct FailingOperation;

  impl PrivilegedOperation for FailingOperation {
    /// Executes the `execute` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
    fn execute(&self, _request: &PrivilegedRequest) -> Result<ProcessOutput, String> {
      Err("simulated denial".into())
    }
  }

  #[derive(Default)]
  /// Represents `RecordingRunner`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
  struct RecordingRunner(Mutex<Option<ProcessRequest>>);

  impl ProcessRunner for RecordingRunner {
    /// Executes the `run` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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

  /// Constructs `with_operation` with this module's expected initial state. Uses a hermetic runner so effect reads never touch the real system. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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

  /// Constructs `with_config` with this module's expected initial state. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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

  /// Executes the `drain_poll` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
  /// Executes the `toggling_icons_flips_the_session_and_requests_a_privileged_save` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn toggling_icons_flips_the_session_and_requests_a_privileged_save() {
    let _guard = env_test_guard();
    let (dir, path) = with_config(true);
    unsafe { std::env::set_var("ARGVUS_CONFIG_PATH", &path) };
    let recorder = Arc::new(RecordingOperation::default());
    let mut app = with_operation(Arc::clone(&recorder) as ConfigOperation);
    assert!(app.icons);
    app.toggle();
    assert!(!app.icons);
    assert!(!AppConfig::icons_enabled());
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
  /// Executes the `failed_save_reports_applied_this_session_warning` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
  /// Executes the `checkbox_label_tracks_icon_state` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
  /// Executes the `escape_leaves_and_enter_toggles` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
  /// Executes the `transparency_toggle_applies_the_control_center_surface` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
        "50",
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
  /// Executes the `left_and_right_step_the_transparency_value` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
    assert_eq!(app.transparency_value, 55);
    drain_poll(&mut app);
    let requested = recorder.0.lock().unwrap().clone().unwrap();
    let tail = requested.args[requested.args.len() - 6..].to_vec();
    assert_eq!(
      tail,
      vec![
        "surface-apply",
        "control-center",
        "enabled",
        "55",
        "enabled",
        "50"
      ]
    );
    assert!(!app.handle(KeyCode::Left));
    assert_eq!(app.transparency_value, 50);
    drain_poll(&mut app);
    let requested = recorder.0.lock().unwrap().clone().unwrap();
    let tail = requested.args[requested.args.len() - 6..].to_vec();
    assert_eq!(
      tail,
      vec![
        "surface-apply",
        "control-center",
        "enabled",
        "50",
        "enabled",
        "50"
      ]
    );
    assert!(!app.handle(KeyCode::Left));
    assert_eq!(app.transparency_value, 45);
    drain_poll(&mut app);
    let requested = recorder.0.lock().unwrap().clone().unwrap();
    let tail = requested.args[requested.args.len() - 6..].to_vec();
    assert_eq!(
      tail,
      vec![
        "surface-apply",
        "control-center",
        "enabled",
        "45",
        "enabled",
        "50"
      ]
    );
    unsafe { std::env::remove_var("ARGVUS_CONFIG_PATH") };
    let _ = std::fs::remove_dir_all(&dir);
  }

  #[test]
  /// Renders `draws_the_checkbox_screen` while respecting the current domain state and semantic theme. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
}
