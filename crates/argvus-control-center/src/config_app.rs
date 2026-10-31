//! Control Center options page: decorative icons and the transparency of the Control Center window.
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

  /// The `runner` handles the user-owned effects hops so the Control Center effects can be persisted without privilege elevation.
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

  pub fn breadcrumb(&self) -> String {
    format!(
      "{} > {}",
      tr(self.lang, "control_center.settings"),
      tr(self.lang, "control_center.style")
    )
  }

  pub fn footer_hints(&self) -> &'static str {
    tr(
      self.lang,
      "control_center.navigate_enter_space_toggle_left_right_value_esc_back_help",
    )
  }

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

  fn persist_failed(&mut self, text: String) {
    self.status = Some(StatusMessage {
      kind: StatusKind::Warning,
      text,
    });
  }

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
    let body = argvus_tui::page::admin_warning(frame, body, &self.theme, self.lang, true);
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

/// Renders the checkbox glyph for `enabled`.
fn checkbox(enabled: bool) -> &'static str {
  // Same `[x]` / `[ ]` convention used by every selection list in the Control Center.
  if enabled { "x" } else { " " }
}

/// Reads the Control Center effect keys from argvus-config in a best-effort
/// way. When the tool is unavailable or the keys are missing, the session
/// defaults are used so the screen still reflects what the effect path applies.
fn load_effects(runner: &Arc<dyn ProcessRunner + Send + Sync>) -> (bool, i32, bool) {
  let transparency = read_effect_flag(runner, "transparency_control-center_enabled", true);
  let transparency_value = read_effect_value(runner, "transparency_control-center_value", 15);
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
mod tests;
