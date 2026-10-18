//! Implements terminal UI rendering and interaction in crate `argvus control center dev services`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use crate::{backend, model::DevServicesSnapshot};
use argvus_control_center_core::navigation::HomeCardId;
use argvus_control_center_core::{
  capabilities::Capabilities,
  config::AppConfig,
  jobs::{JobHandle, JobManager, JobState},
};
use argvus_i18n::{Lang, tr};
use argvus_theme::Theme;
use argvus_tui::components::{StatusKind, StatusMessage};
use argvus_tui::hints::{HintContext, hints};
use argvus_tui::icons;
use argvus_tui::page::{list, shell, status};
use crossterm::event::KeyCode;
use ratatui::Frame;

/// Represents `DevServicesApp`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct DevServicesApp {
  capabilities: Capabilities,
  snapshot: DevServicesSnapshot,
  manager: JobManager,
  job: Option<JobHandle<DevServicesSnapshot>>,
  selected: usize,
  pub lang: Lang,
  pub theme: Theme,
  pub status: Option<StatusMessage>,
}

impl DevServicesApp {
  /// Constructs `new` with this module's expected initial state. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn new(lang: Lang, theme: Theme, capabilities: Capabilities) -> Self {
    Self {
      capabilities,
      snapshot: DevServicesSnapshot::default(),
      manager: JobManager::default(),
      job: None,
      selected: 0,
      lang,
      theme,
      status: None,
    }
  }

  /// Replaces the semantic theme used by this page.
  pub fn set_theme(&mut self, theme: &Theme) {
    self.theme = theme.clone();
  }

  /// Executes the `reload` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn reload(&mut self) {
    if self.job.is_some() {
      return;
    }
    let capabilities = self.capabilities.clone();
    self.job = Some(self.manager.spawn(move |_| {
      let (services_result, (ports_result, containers_result)) =
        rayon::join(backend::list_user_services, || {
          rayon::join(backend::listening_ports, || {
            backend::containers(&capabilities)
          })
        });
      let (user_services, services_error) = match services_result {
        Ok(units) => (units, None),
        Err(error) => (Vec::new(), Some(error)),
      };
      let (ports, ports_error) = match ports_result {
        Ok(ports) => (
          ports
            .into_iter()
            .map(|port| crate::model::ListeningPort { port })
            .collect(),
          None,
        ),
        Err(error) => (Vec::new(), Some(error)),
      };
      let (containers, containers_error) = containers_result;
      Ok(DevServicesSnapshot {
        user_services,
        ports,
        containers,
        services_error,
        ports_error,
        containers_error,
      })
    }));
    self.status = Some(StatusMessage {
      kind: StatusKind::Info,
      text: tr(self.lang, "control_center.loading_dev_services").into(),
    });
  }

  /// Whether typed characters currently go to a search field. This page has
  /// none, so the global shortcuts always apply.
  pub fn captures_text(&self) -> bool {
    false
  }

  /// Executes the `poll` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn poll(&mut self) -> bool {
    let Some(job) = &self.job else {
      return false;
    };
    if let JobState::Finished(result) = job.try_state() {
      self.job = None;
      match result {
        Ok(snapshot) => {
          self.snapshot = snapshot;
          self.status = None;
        }
        Err(error) => {
          self.status = Some(StatusMessage {
            kind: StatusKind::Error,
            text: error,
          })
        }
      }
      return true;
    }
    false
  }

  /// Processes `handle` in this module's event flow. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn handle(&mut self, key: KeyCode) -> bool {
    if self.job.is_some() {
      return false;
    }
    match key {
      KeyCode::Esc | KeyCode::Left => return true,
      KeyCode::Char('r') => self.reload(),
      KeyCode::Up | KeyCode::Char('k') => self.selected = self.selected.saturating_sub(1),
      KeyCode::Down | KeyCode::Char('j') => self.selected = self.selected.saturating_add(1),
      KeyCode::Home => self.selected = 0,
      KeyCode::End => self.selected = self.rows().len().saturating_sub(1),
      _ => {}
    }
    self.selected = self.selected.min(self.rows().len().saturating_sub(1));
    false
  }

  /// Executes the `breadcrumb` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn breadcrumb(&self) -> String {
    format!(
      "{} > {}",
      HomeCardId::Tools.label(self.lang),
      tr(self.lang, "control_center.developer_services")
    )
  }

  /// Executes the `footer_hints` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn footer_hints(&self) -> String {
    hints(
      self.lang,
      &HintContext {
        can_go_back: true,
        refresh: true,
        ..HintContext::default()
      },
    )
  }

  /// Builds the read-only rows of the three sections this page reports.
  fn rows(&self) -> Vec<String> {
    let mut rows = Vec::new();
    rows.push(format!(
      "{} {} ({})",
      AppConfig::icon(icons::SERVICES),
      tr(self.lang, "control_center.systemd_user_services"),
      self.snapshot.user_services.len(),
    ));
    if let Some(error) = &self.snapshot.services_error {
      rows.push(format!("  {error}"));
    } else if self.snapshot.user_services.is_empty() {
      rows.push(format!(
        "  {}",
        tr(self.lang, "control_center.no_user_services")
      ));
    } else {
      let mut units = self.snapshot.user_services.clone();
      units.sort_by(|left, right| left.name.cmp(&right.name));
      rows.extend(
        units
          .iter()
          .map(|unit| format!("  {:<40} {} / {}", unit.name, unit.active, unit.sub)),
      );
    }
    rows.push(String::new());
    rows.push(format!(
      "{} {} ({})",
      AppConfig::icon(icons::NETWORK),
      tr(self.lang, "control_center.listening_ports"),
      self.snapshot.ports.len(),
    ));
    if let Some(error) = &self.snapshot.ports_error {
      rows.push(format!("  {error}"));
    } else if self.snapshot.ports.is_empty() {
      rows.push(format!(
        "  {}",
        tr(self.lang, "control_center.no_listening_ports")
      ));
    } else {
      rows.extend(
        self
          .snapshot
          .ports
          .iter()
          .map(|port| format!("  {}", port.port)),
      );
    }
    rows.push(String::new());
    rows.push(format!(
      "{} {} ({})",
      AppConfig::icon(icons::PACKAGES),
      tr(self.lang, "control_center.containers"),
      self.snapshot.containers.len(),
    ));
    if let Some(error) = &self.snapshot.containers_error {
      rows.push(format!("  {error}"));
    } else if self.snapshot.containers.is_empty() {
      rows.push(format!(
        "  {}",
        tr(self.lang, "control_center.no_containers")
      ));
    } else {
      rows.extend(self.snapshot.containers.iter().map(|container| {
        format!(
          "  {:<24} {:<24} {} [{}]",
          container.name, container.image, container.status, container.runtime
        )
      }));
    }
    rows
  }

  /// Renders `draw` while respecting the current domain state and semantic theme. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn draw(&mut self, frame: &mut Frame) {
    let area = frame.area();
    let body = shell(
      frame,
      area,
      &self.theme,
      self.lang,
      &self.breadcrumb(),
      &self.footer_hints(),
    );
    let rows = self.rows();
    let selection = if self.job.is_some() {
      usize::MAX
    } else {
      self.selected.min(rows.len().saturating_sub(1))
    };
    list(frame, body, &self.theme, &rows, selection);
    if let Some(s) = &self.status {
      status(frame, area, &self.theme, s)
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::model::{ContainerSummary, ListeningPort};
  use argvus_control_center_services::Unit;

  #[test]
  /// Executes the `rows_render_each_section_with_its_count` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn rows_render_each_section_with_its_count() {
    let mut app = DevServicesApp::new(
      Lang::for_locale("en-US"),
      Theme::load(),
      Capabilities::default(),
    );
    app.snapshot.user_services = vec![Unit {
      name: "argvus-session.service".into(),
      active: "active".into(),
      sub: "running".into(),
      ..Default::default()
    }];
    app.snapshot.ports = vec![ListeningPort { port: 22 }];
    app.snapshot.containers = vec![ContainerSummary {
      runtime: "podman".into(),
      name: "web".into(),
      image: "nginx:latest".into(),
      status: "Up 2 hours".into(),
    }];
    let rows = app.rows().join("\n");
    assert!(rows.contains("argvus-session.service"));
    assert!(rows.contains("22"));
    assert!(rows.contains("web") && rows.contains("nginx:latest"));
  }

  #[test]
  /// Executes the `empty_sections_show_their_fallback_message` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn empty_sections_show_their_fallback_message() {
    let app = DevServicesApp::new(
      Lang::for_locale("en-US"),
      Theme::load(),
      Capabilities::default(),
    );
    let rows = app.rows().join("\n");
    assert!(
      rows.contains("No user services")
        || rows.contains(tr(app.lang, "control_center.no_user_services"))
    );
  }

  #[test]
  /// Executes the `esc_requests_leaving_the_page` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn esc_requests_leaving_the_page() {
    let mut app = DevServicesApp::new(
      Lang::for_locale("en-US"),
      Theme::load(),
      Capabilities::default(),
    );
    assert!(app.handle(KeyCode::Esc));
  }
}
