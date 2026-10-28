use super::*;
use ratatui::{Terminal, backend::TestBackend};

fn app() -> DiagnosticsApp {
  app_in(Lang::for_locale("en-US"))
}

/// The diagnostics page in `lang`, with fixed checks.
fn app_in(lang: Lang) -> DiagnosticsApp {
  let mut app = DiagnosticsApp::new(lang, Theme::load(), Capabilities::default());
  app.job = None;
  app.checks = vec![
    DiagnosticCheck {
      id: "system.failed_units".into(),
      category: "services".into(),
      severity: Severity::Warning,
      title: "Failed system units".into(),
      summary: "2 system unit(s) failed".into(),
      details: "The system manager reports failed units.".into(),
      remediation_hint: Some("Open Services > Failed".into()),
      ..Default::default()
    },
    DiagnosticCheck {
      id: "graphics.opengl".into(),
      category: "graphics".into(),
      severity: Severity::Ok,
      title: "OpenGL renderer".into(),
      summary: "Mesa hardware renderer".into(),
      details: "OpenGL reports a hardware renderer.".into(),
      ..Default::default()
    },
    DiagnosticCheck {
      id: "graphics.environment".into(),
      category: "system".into(),
      severity: Severity::Info,
      title: "Graphics environment".into(),
      summary: "Hardware graphics checks are provided by Hardware".into(),
      ..Default::default()
    },
  ];
  app
}

#[test]
fn diagnostics_home_dashboard_lists_categories_with_previews() {
  let app = app();
  let rows = app.dashboard_rows();
  assert_eq!(rows.len(), 10);
  assert!(
    rows[0].contains("Resumo") || rows[0].contains("Summary"),
    "first row must summarize"
  );
  assert!(rows[1].contains("Serviços") || rows[1].contains("Services"));
  assert!(rows[1].contains("warning"));
  assert!(rows[3].contains("Gráficos") || rows[3].contains("Graphics"));
  assert!(rows[3].contains("OK"));
  assert!(
    rows[9].contains("ARGVUS"),
    "last row must cover the ARGVUS category"
  );
}

#[test]
fn diagnostics_category_rows_render_severity_glyphs() {
  let mut app = app();
  app.page = DiagnosticPage::Services;
  let rows = app.category_rows();
  assert_eq!(rows.len(), 1);
  assert!(rows[0].starts_with("▲"));
  assert!(rows[0].contains("Failed system units"));
  assert!(rows[0].contains("2 system unit(s) failed"));
}

#[test]
fn diagnostics_detail_rows_render_aligned_sections() {
  let app = app();
  let rows = app.detail_rows(0);
  assert!(
    rows[0].contains("SERVICES"),
    "first row is a section header"
  );
  assert!(rows.iter().any(|row| row.contains("Severity:")));
  assert!(
    rows
      .iter()
      .any(|row| row.contains("Open Services > Failed"))
  );
}

#[test]
fn diagnostics_summary_rows_include_counters_and_system_checks() {
  let app = app();
  let rows = app.summary_rows();
  assert!(rows[0].contains("DIAGNOSTICS"));
  assert!(rows.iter().any(|row| row.contains("Checks:")));
  assert!(rows.iter().any(|row| row.contains("SYSTEM")));
  assert!(rows.iter().any(|row| row.contains("Graphics environment")));
}

#[test]
fn diagnostics_home_opens_summary_and_escapes_back() {
  let mut app = app();
  app.handle(KeyCode::Enter);
  assert_eq!(app.page, DiagnosticPage::Summary);
  app.handle(KeyCode::Esc);
  assert_eq!(app.page, DiagnosticPage::Home);
  assert!(
    app.handle(KeyCode::Esc),
    "second back returns to global home"
  );
}

#[test]
fn diagnostics_category_details_back_out_one_level() {
  let mut app = app();
  app.page = DiagnosticPage::Services;
  app.handle(KeyCode::Enter);
  assert_eq!(app.page, DiagnosticPage::Detail(0));
  app.handle(KeyCode::Esc);
  assert_eq!(app.page, DiagnosticPage::Services);
  app.handle(KeyCode::Esc);
  assert_eq!(app.page, DiagnosticPage::Home);
}

#[test]
fn diagnostics_uses_shared_chrome_and_contextual_footer() {
  let mut app = app();
  let mut terminal = Terminal::new(TestBackend::new(90, 25)).unwrap();
  terminal.draw(|frame| app.draw(frame)).unwrap();
  let text = terminal
    .backend()
    .buffer()
    .content
    .iter()
    .map(|cell| cell.symbol())
    .collect::<String>();
  assert!(text.contains("ARGVUS"));
  assert!(text.contains("Diagnostics"));
  assert!(text.contains("Enter") && text.contains("Back"));

  app.page = DiagnosticPage::Services;
  let mut terminal = Terminal::new(TestBackend::new(90, 25)).unwrap();
  terminal.draw(|frame| app.draw(frame)).unwrap();
  let text = terminal
    .backend()
    .buffer()
    .content
    .iter()
    .map(|cell| cell.symbol())
    .collect::<String>();
  assert!(text.contains("Services"));
  assert!(text.contains("Details"));
}

#[test]
/// The header is drawn from the translated root and card: in pt-BR the rendered
/// screen shows the full path in Portuguese and no English navigation segment.
fn diagnostics_header_is_translated_in_pt_br() {
  let app = app_in(Lang::for_locale("pt-BR"));
  let mut terminal = Terminal::new(TestBackend::new(120, 30)).unwrap();
  let mut app = app;
  terminal.draw(|frame| app.draw(frame)).unwrap();
  let rendered = terminal
    .backend()
    .buffer()
    .content
    .iter()
    .map(|cell| cell.symbol())
    .collect::<String>();
  assert!(
    rendered.contains("Central de Controle > Ferramentas e diagnóstico > Diagnóstico"),
    "{rendered}"
  );
  assert!(!rendered.contains("Tools & Diagnostics"), "{rendered}");
}

#[test]
fn diagnostics_detail_footer_and_breadcrumb_follow_the_parent_category() {
  let app = app();
  assert_eq!(app.breadcrumb(), "Tools & Diagnostics > Diagnostics");
  let mut detail = app;
  detail.page = DiagnosticPage::Detail(2);
  assert!(detail.breadcrumb().ends_with("System"));
  assert!(detail.footer_hints().contains("Back"));
}

#[test]
fn completion_status_reflects_worst_severity() {
  assert_eq!(
    completion_status(Lang::for_locale("en-US"), &[]).kind,
    StatusKind::Success
  );
  let mut checks = app();
  checks.checks = vec![DiagnosticCheck {
    id: "storage.usage./".into(),
    category: "storage".into(),
    severity: Severity::Error,
    ..Default::default()
  }];
  assert_eq!(
    completion_status(Lang::for_locale("en-US"), &checks.checks).kind,
    StatusKind::Error
  );
  let mut warnings = app();
  warnings.checks = vec![DiagnosticCheck {
    id: "packages.lock".into(),
    category: "packages".into(),
    severity: Severity::Warning,
    ..Default::default()
  }];
  assert_eq!(
    completion_status(Lang::for_locale("en-US"), &warnings.checks).kind,
    StatusKind::Warning
  );
}
