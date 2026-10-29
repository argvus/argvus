//! Background loading: dropped loads, refresh windows, card availability and surface recreation.

use super::*;

#[test]
fn open_drops_a_load_left_running_for_another_page() {
  // The host returned to its Home while the Control Panel was still loading;
  // opening Widget Telemetry must not wait for that load.
  let mut application = app(AppearancePage::ControlPanel);
  application.job = Some(application.manager.spawn(|_| {
    std::thread::sleep(std::time::Duration::from_secs(2));
    Ok(JobData::Action(String::new()))
  }));
  application.job_page = Some(AppearancePage::ControlPanel);
  let stale = application.job.as_ref().map(JobHandle::id);

  application.open(AppearancePage::WidgetTelemetry);

  assert_eq!(application.page, AppearancePage::WidgetTelemetry);
  assert_eq!(application.job_page, Some(AppearancePage::WidgetTelemetry));
  assert_ne!(application.job.as_ref().map(JobHandle::id), stale);
}

#[test]
fn reopening_a_surface_within_its_refresh_window_shows_its_values() {
  // Leaving the page clears its draft; reopening it while the loaded state is
  // still fresh starts no load, so the draft must come from that state.
  let mut application = app(AppearancePage::Home);
  application.state.widget_telemetry = true;
  application
    .refreshed
    .push((AppearancePage::WidgetTelemetry, std::time::Instant::now()));

  application.open(AppearancePage::WidgetTelemetry);

  assert!(application.job.is_none(), "no reload inside the window");
  assert!(
    application
      .surface_draft()
      .is_some_and(|draft| draft.widget_enabled)
  );
  assert!(!application.has_pending_changes());
}

#[test]
fn only_structural_surface_changes_recreate_the_surface() {
  let application = app(AppearancePage::Desktop);
  let taskbar = application.make_surface_draft(EffectSurface::Taskbar);
  let styled = SurfaceDraft {
    border_enabled: !taskbar.border_enabled,
    border_thickness: 3,
    transparency: 40,
    blur: 70,
    ..taskbar.clone()
  };
  assert!(!recreates_surface(&styled, &taskbar));
  let taller = SurfaceDraft {
    height: taskbar.height + 4,
    ..taskbar.clone()
  };
  assert!(recreates_surface(&taller, &taskbar));
  let no_seconds = SurfaceDraft {
    time_seconds_enabled: !taskbar.time_seconds_enabled,
    ..taskbar.clone()
  };
  assert!(recreates_surface(&no_seconds, &taskbar));
  let focus_icon = SurfaceDraft {
    focus_enabled: !taskbar.focus_enabled,
    ..taskbar.clone()
  };
  assert!(recreates_surface(&focus_icon, &taskbar));

  let widget = application.make_surface_draft(EffectSurface::WidgetTelemetry);
  let reordered = SurfaceDraft {
    widget_order: widget.widget_order.iter().rev().copied().collect(),
    ..widget.clone()
  };
  assert!(recreates_surface(&reordered, &widget));
  let translucent = SurfaceDraft {
    transparency_enabled: !widget.transparency_enabled,
    ..widget.clone()
  };
  assert!(!recreates_surface(&translucent, &widget));

  let terminal = application.make_surface_draft(EffectSurface::Terminal);
  let clear = SurfaceDraft {
    transparency: 10,
    ..terminal.clone()
  };
  assert!(!recreates_surface(&clear, &terminal));
}

#[test]
fn a_hidden_control_panel_is_not_recreated_by_its_cards() {
  let application = app(AppearancePage::Desktop);
  let mut current = application.make_surface_draft(EffectSurface::ControlPanel);
  current.control_panel_enabled = false;
  let mut cards = current.clone();
  cards.control_panel_cards.set(
    ControlPanelCard::ALL[0],
    !current
      .control_panel_cards
      .enabled(ControlPanelCard::ALL[0]),
  );
  assert!(!recreates_surface(&cards, &current));
  let shown = SurfaceDraft {
    control_panel_enabled: true,
    ..current.clone()
  };
  assert!(recreates_surface(&shown, &current));
  current.control_panel_enabled = true;
  cards.control_panel_enabled = true;
  assert!(recreates_surface(&cards, &current));
}
