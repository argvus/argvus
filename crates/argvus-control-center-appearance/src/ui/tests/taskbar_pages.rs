//! Taskbar pages: height, border, icons, date and time, and the launcher icon.

use super::*;

#[test]
fn taskbar_page_lists_height_and_border_after_the_spaces() {
  let application = draft_app(AppearancePage::Taskbar);
  let labels = labels(&application);
  let spaces = labels.iter().position(|label| label == "Spaces");
  let height = labels.iter().position(|label| label == "Height");
  let border = labels.iter().position(|label| label == "Border");
  assert!(spaces.is_some() && height > spaces && border > height);
  assert_eq!(
    application.surface_draft().unwrap().height,
    28,
    "the default height is 28"
  );
  assert_eq!(application.state.waybar_bottom, 1);
}

#[test]
fn height_and_border_pages_extend_the_taskbar_breadcrumb() {
  let application = app(AppearancePage::Taskbar);
  let taskbar = application.breadcrumb_of(AppearancePage::Taskbar);
  assert_eq!(
    application.breadcrumb_of(AppearancePage::TaskbarHeight),
    format!("{taskbar} > Height")
  );
  assert_eq!(
    application.breadcrumb_of(AppearancePage::TaskbarBorder),
    format!("{taskbar} > Border")
  );
  assert_eq!(
    application.breadcrumb_of(AppearancePage::TaskbarBorderThickness),
    format!("{taskbar} > Border > Thickness")
  );
  let icons = application.breadcrumb_of(AppearancePage::TaskbarIcons);
  assert_eq!(
    application.breadcrumb_of(AppearancePage::SurfaceSection {
      surface: EffectSurface::Taskbar,
      section: SurfaceSection::Focus,
    }),
    format!("{icons} > Focus")
  );
}

#[test]
fn height_option_edits_the_draft_until_apply() {
  let mut application = draft_app(AppearancePage::TaskbarHeight);
  assert!(!application.has_pending_changes());
  application.activate(Item::TaskbarHeightOption(36));
  assert_eq!(application.surface_draft().unwrap().height, 36);
  assert!(application.has_pending_changes());
}

#[test]
fn border_thickness_is_disabled_while_the_border_is_off() {
  let mut application = draft_app(AppearancePage::TaskbarBorder);
  assert!(application.surface_draft().unwrap().border_enabled);
  application.activate(Item::BorderEnabled);
  assert!(!application.surface_draft().unwrap().border_enabled);
  assert!(application.has_pending_changes());

  let rows = application.rows();
  let mut menu = application.menu;
  menu.normalize(&rows);
  assert!(
    !menu.select(&rows, &Item::BorderThickness),
    "Thickness is skipped while Enable is off"
  );
}

#[test]
fn border_thickness_option_is_chosen_when_the_border_is_on() {
  let mut application = draft_app(AppearancePage::TaskbarBorderThickness);
  application.activate(Item::BorderThicknessOption(3));
  assert_eq!(application.surface_draft().unwrap().border_thickness, 3);
  assert!(application.has_pending_changes());
}

#[test]
fn taskbar_icons_lists_checks_first_and_submenus_last_with_icons() {
  let rows = app(AppearancePage::TaskbarIcons).page_rows();
  let first_submenu = rows
    .iter()
    .position(|row| row.kind() == RowKind::Submenu)
    .expect("Taskbar icons ends with submenus");
  assert_eq!(first_submenu, rows.len() - 3);
  assert!(
    rows[..first_submenu]
      .iter()
      .all(|row| matches!(row.kind(), RowKind::Toggle { .. }))
  );
  assert!(
    rows[first_submenu..]
      .iter()
      .all(|row| row.kind() == RowKind::Submenu)
  );
  assert!(rows.iter().all(|row| row.icon_glyph().is_some()));
}

#[test]
fn esc_from_taskbar_launcher_returns_to_taskbar_icons() {
  let mut icons_page = draft_app(AppearancePage::TaskbarIcons);
  select(&mut icons_page, Item::LauncherIcon);
  icons_page.handle(KeyCode::Enter);
  assert_eq!(
    icons_page.page,
    AppearancePage::SurfaceSection {
      surface: EffectSurface::Taskbar,
      section: SurfaceSection::Launcher,
    }
  );
  icons_page.job = None;
  icons_page.handle(KeyCode::Esc);
  assert_eq!(icons_page.page, AppearancePage::TaskbarIcons);
}

#[test]
fn taskbar_icons_date_time_pages_toggle_and_end_with_apply() {
  let mut icons_page = draft_app(AppearancePage::TaskbarIcons);
  let draft = |application: &AppearanceApp| application.surface_draft().unwrap().clone();

  let before = draft(&icons_page);
  assert!(before.focus_enabled, "Focus is enabled by default");
  select(&mut icons_page, Item::AudioPlayer);
  icons_page.handle(KeyCode::Enter);
  select(&mut icons_page, Item::FocusIcon);
  icons_page.handle(KeyCode::Enter);
  assert_eq!(
    icons_page.page,
    AppearancePage::SurfaceSection {
      surface: EffectSurface::Taskbar,
      section: SurfaceSection::Focus,
    }
  );
  icons_page.job = None;
  select(&mut icons_page, Item::FocusEnabled);
  icons_page.handle(KeyCode::Enter);
  assert_eq!(
    icons_page.surface_draft().unwrap().focus_enabled,
    !before.focus_enabled
  );
  icons_page.handle(KeyCode::Esc);
  assert_eq!(icons_page.page, AppearancePage::TaskbarIcons);
  icons_page.job = None;

  let widget = TaskbarUtilityWidget::GpuTemperature;
  select(&mut icons_page, Item::UtilityWidget(widget));
  icons_page.handle(KeyCode::Enter);
  let after = draft(&icons_page);
  assert_eq!(after.audio_player_enabled, !before.audio_player_enabled);
  assert_eq!(after.focus_enabled, !before.focus_enabled);
  assert_eq!(
    after.utility_widgets.enabled(widget),
    !before.utility_widgets.enabled(widget)
  );

  // "Launcher ›" opens the shared `SurfaceSection{Taskbar, Launcher}` page
  // (Enable + Custom), instead of toggling inline like the other rows.
  select(&mut icons_page, Item::LauncherIcon);
  icons_page.handle(KeyCode::Enter);
  assert_eq!(
    icons_page.page,
    AppearancePage::SurfaceSection {
      surface: EffectSurface::Taskbar,
      section: SurfaceSection::Launcher,
    }
  );

  let mut launcher = draft_app(AppearancePage::SurfaceSection {
    surface: EffectSurface::Taskbar,
    section: SurfaceSection::Launcher,
  });
  let before_enabled = launcher.surface_draft().unwrap().launcher_enabled;
  select(&mut launcher, Item::LauncherEnabled);
  launcher.handle(KeyCode::Enter);
  assert_eq!(
    launcher.surface_draft().unwrap().launcher_enabled,
    !before_enabled
  );

  // Back to the Icons page before exercising the next submenu, since
  // navigating into Launcher moved `icons_page.page` away from TaskbarIcons.
  // `go()` schedules a refresh job that nothing polls in this test, so clear
  // it or the next Enter would be swallowed by the "busy" guard in `handle`.
  icons_page.go(AppearancePage::TaskbarIcons);
  icons_page.job = None;

  // "Utilities ›" opens the shared `SurfaceSection{Taskbar, UtilityIcons}`
  // page (Always expanded / Expand on hover).
  select(&mut icons_page, Item::Utilities);
  icons_page.handle(KeyCode::Enter);
  assert_eq!(
    icons_page.page,
    AppearancePage::SurfaceSection {
      surface: EffectSurface::Taskbar,
      section: SurfaceSection::UtilityIcons,
    }
  );

  let mut utility_group = draft_app(AppearancePage::SurfaceSection {
    surface: EffectSurface::Taskbar,
    section: SurfaceSection::UtilityIcons,
  });
  for mode in [
    TaskbarUtilityGroupMode::AlwaysExpanded,
    TaskbarUtilityGroupMode::Auto,
  ] {
    select(&mut utility_group, Item::UtilityGroup(mode));
    utility_group.handle(KeyCode::Enter);
    assert_eq!(utility_group.surface_draft().unwrap().utility_group, mode);
  }

  let mut date_format = draft_app(AppearancePage::TaskbarDateFormat);
  let format = TaskbarDateFormat::WeekdayDayMonthYear;
  select(&mut date_format, Item::DateFormat(format));
  date_format.handle(KeyCode::Enter);
  assert_eq!(date_format.surface_draft().unwrap().date_format, format);

  let mut time_format = draft_app(AppearancePage::TaskbarTimeFormat);
  select(
    &mut time_format,
    Item::TimeFormat(TaskbarTimeFormat::TwelveHour),
  );
  time_format.handle(KeyCode::Enter);
  assert_eq!(
    time_format.surface_draft().unwrap().time_format,
    TaskbarTimeFormat::TwelveHour
  );

  // Apply is the last row and runs the Taskbar apply.
  icons_page.go(AppearancePage::TaskbarIcons);
  icons_page.job = None;
  icons_page.handle(KeyCode::End);
  assert_eq!(selected(&icons_page), Some(Item::Apply));
  icons_page.handle(KeyCode::Enter);
  assert!(icons_page.action.is_some());
}

#[test]
fn launcher_custom_row_shows_the_current_icon_path_as_its_detail() {
  let mut launcher = draft_app(AppearancePage::SurfaceSection {
    surface: EffectSurface::Taskbar,
    section: SurfaceSection::Launcher,
  });
  let rows = launcher.rows();
  let custom_row = rows
    .iter()
    .find(|row| row.label() == tr(launcher.lang, "control_center.custom"))
    .expect("Custom row must be present");
  assert_eq!(
    custom_row.detail_text(),
    Some(tr(launcher.lang, "control_center.none"))
  );

  launcher.state.taskbar_launcher_custom_icon_path = Some("/home/user/icon.svg".into());
  let rows = launcher.rows();
  let custom_row = rows
    .iter()
    .find(|row| row.label() == tr(launcher.lang, "control_center.custom"))
    .expect("Custom row must be present");
  assert_eq!(custom_row.detail_text(), Some("/home/user/icon.svg"));
}
