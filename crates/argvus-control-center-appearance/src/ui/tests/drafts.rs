//! Draft pages: values stay in the draft until Apply, and leaving with changes asks first.

use super::*;

#[test]
fn value_rows_adjust_with_arrows_and_footer_says_esc_goes_back() {
  let mut editor = app(AppearancePage::Blur);
  select(&mut editor, Item::BlurField(BlurField::Size));
  let rows = editor.rows();
  let footer = editor.hints(&rows);
  assert!(footer.contains("←/→ "), "{footer}");
  assert!(footer.contains("Esc "), "{footer}");
  assert!(!footer.contains("←/Esc"), "← adjusts here: {footer}");

  editor.handle(KeyCode::Right);
  assert_eq!(blur_value(&editor, BlurField::Size), 7.0);
  editor.handle(KeyCode::Left);
  editor.handle(KeyCode::Left);
  assert_eq!(blur_value(&editor, BlurField::Size), 5.0);
  assert_eq!(
    editor.page,
    AppearancePage::Blur,
    "← adjusted, did not leave"
  );
  editor.handle(KeyCode::Char('h'));
  editor.handle(KeyCode::Char('l'));
  editor.handle(KeyCode::Char('l'));
  assert_eq!(blur_value(&editor, BlurField::Size), 6.0);
  assert!(!editor.blur_draft_has_changes(), "back to the stored value");
}

#[test]
fn blur_switch_and_values_stay_in_the_draft_until_apply() {
  let mut blur = app(AppearancePage::Blur);
  select(&mut blur, Item::BlurEnabled);
  blur.handle(KeyCode::Enter);
  assert_eq!(
    blur.blur_draft.map(|draft| draft.enabled),
    Some(!blur.state.blur)
  );
  assert!(blur.has_pending_changes(), "Apply is enabled");
  assert!(blur.action.is_none(), "the switch is not written yet");

  select(&mut blur, Item::BlurField(BlurField::Passes));
  blur.handle(KeyCode::Right);
  assert_eq!(blur_value(&blur, BlurField::Passes), 3.0);
  assert_eq!(
    blur.state.blur_values.get(BlurField::Passes),
    2.0,
    "stored value unchanged"
  );
  assert!(blur.action.is_none(), "nothing applied");
}

#[test]
fn draft_pages_end_with_an_apply_row_enabled_only_with_changes() {
  let pages = [
    AppearancePage::Taskbar,
    AppearancePage::TaskbarIcons,
    AppearancePage::TaskbarDate,
    AppearancePage::TaskbarDateFormat,
    AppearancePage::TaskbarTime,
    AppearancePage::TaskbarTimeFormat,
    AppearancePage::WidgetTelemetry,
    AppearancePage::ControlPanel,
    AppearancePage::SurfaceSection {
      surface: EffectSurface::Taskbar,
      section: SurfaceSection::UtilityIcons,
    },
    AppearancePage::SurfaceSection {
      surface: EffectSurface::WidgetTelemetry,
      section: SurfaceSection::Sessions,
    },
    AppearancePage::SurfaceSection {
      surface: EffectSurface::ControlPanel,
      section: SurfaceSection::Transparency,
    },
  ];
  for page in pages {
    let mut application = draft_app(page);
    let rows = application.rows();
    let (apply, separator) = (&rows[rows.len() - 1], &rows[rows.len() - 2]);
    assert_eq!(apply.id(), Some(&Item::Apply), "{page:?}");
    assert_eq!(separator.kind(), RowKind::Separator, "{page:?}");
    assert_eq!(apply.icon_glyph(), Some(icons::APPLY));
    assert!(
      !apply.is_selectable(),
      "clean draft: Apply is skipped ({page:?})"
    );

    application
      .surface_draft
      .as_mut()
      .unwrap()
      .transparency_enabled ^= true;
    let rows = application.rows();
    let apply = rows.last().unwrap();
    assert!(
      apply.is_selectable(),
      "dirty draft: Apply is enabled ({page:?})"
    );
    assert!(
      apply.detail_text().is_some(),
      "pending indicator ({page:?})"
    );
  }

  for page in [
    AppearancePage::Blur,
    AppearancePage::TerminalTransparency,
    AppearancePage::TransparencySurface {
      surface: EffectSurface::Launchers,
    },
  ] {
    let mut application = app(page);
    if page == AppearancePage::Blur {
      select(&mut application, Item::BlurField(BlurField::Size));
    }
    assert!(!application.rows().last().unwrap().is_selectable());
    application.handle(KeyCode::Right);
    assert!(
      application.rows().last().unwrap().is_selectable(),
      "{page:?}"
    );
  }
}

#[test]
fn apply_row_runs_the_same_apply_and_keeps_the_page() {
  let mut application = draft_app(AppearancePage::TaskbarTime);
  select(&mut application, Item::Seconds);
  application.handle(KeyCode::Enter);
  application.handle(KeyCode::End);
  assert_eq!(selected(&application), Some(Item::Apply));
  application.handle(KeyCode::Enter);
  assert!(
    application.action.is_some(),
    "apply_surface_changes started"
  );
  assert_eq!(application.page, AppearancePage::TaskbarTime);

  // Effect editors apply and return where they did before.
  let mut terminal = app(AppearancePage::TerminalTransparency);
  terminal.handle(KeyCode::Left);
  select(&mut terminal, Item::Apply);
  terminal.handle(KeyCode::Enter);
  assert!(terminal.action.is_some(), "apply_effect_changes started");
  assert_eq!(terminal.page, AppearancePage::Terminal);
  assert_eq!(terminal.effect_draft, None);
}

#[test]
fn draft_pages_render_apply_as_a_row_without_button_labels() {
  let mut application = draft_app(AppearancePage::TaskbarTime);
  application
    .surface_draft
    .as_mut()
    .unwrap()
    .time_seconds_enabled ^= true;
  let screen = render(&mut application, 80, 24);
  let apply = line_with(&screen, "Apply");
  assert!(apply.contains(icons::APPLY), "{apply:?}");
  assert!(
    !screen.iter().any(|line| line.contains("[ Apply ]")),
    "{screen:#?}"
  );
}

#[test]
fn esc_with_a_pending_draft_asks_before_discarding() {
  // Clean draft: Esc leaves without asking. Taskbar is an entry point
  // reached from the Control Center Home (crate principal), so leaving
  // signals the host instead of navigating to a parent page here.
  let mut clean = draft_app(AppearancePage::Taskbar);
  assert!(clean.handle(KeyCode::Esc), "signals leaving the crate");
  assert_eq!(clean.page, AppearancePage::Taskbar);
  assert!(clean.confirm.is_none());

  // Pending draft inside the Taskbar pages: going up keeps the draft.
  let mut nested = draft_app(AppearancePage::TaskbarTime);
  nested.surface_draft.as_mut().unwrap().time_seconds_enabled ^= true;
  nested.handle(KeyCode::Esc);
  assert_eq!(nested.page, AppearancePage::Taskbar);
  assert!(nested.confirm.is_none());
  assert!(nested.surface_draft_has_changes(), "draft survives");

  // Leaving the Taskbar would drop it: confirmation, focus on Cancel.
  nested.handle(KeyCode::Esc);
  assert_eq!(nested.page, AppearancePage::Taskbar);
  let (confirmation, state) = nested.confirm.unwrap();
  assert_eq!(confirmation, Confirmation::DiscardDraft);
  assert!(!state.is_confirm_focused());
  let rows = nested.rows();
  assert_eq!(nested.hints(&rows), confirm_hints(nested.lang));
  nested.handle(KeyCode::Enter);
  assert!(nested.confirm.is_none(), "Enter on Cancel closes");
  assert_eq!(nested.page, AppearancePage::Taskbar);
  assert!(nested.surface_draft_has_changes());

  nested.handle(KeyCode::Left);
  nested.handle(KeyCode::Char('n'));
  assert_eq!(nested.page, AppearancePage::Taskbar);
  nested.handle(KeyCode::Esc);
  assert!(
    nested.handle(KeyCode::Char('y')),
    "discarding on an exit point signals leaving the crate"
  );
  assert_eq!(nested.page, AppearancePage::Taskbar);
  assert!(nested.surface_draft.is_none(), "discarded");

  // Effect editors drop their value on every Esc. Blur is now an entry
  // point reached from the Hyprland category (crate principal), so
  // confirming here signals the host to leave instead of navigating to a
  // parent page inside this crate.
  let mut editor = app(AppearancePage::Blur);
  select(&mut editor, Item::BlurField(BlurField::Size));
  editor.handle(KeyCode::Right);
  editor.handle(KeyCode::Esc);
  assert!(editor.confirm.is_some());
  editor.handle(KeyCode::Up);
  assert!(editor.handle(KeyCode::Enter), "signals leaving the crate");
  assert_eq!(editor.page, AppearancePage::Blur);
  assert_eq!(editor.blur_draft, None);
}

#[test]
fn enter_on_a_draft_value_types_it_without_applying() {
  let mut editor = app(AppearancePage::Blur);
  select(&mut editor, Item::BlurField(BlurField::Contrast));
  editor.handle(KeyCode::Enter);
  assert_eq!(
    editor.page,
    AppearancePage::Prompt {
      goal: PromptGoal::BlurValue(BlurField::Contrast)
    }
  );
  assert!(editor.captures_text());
  for key in ['0', '.', '5'] {
    editor.handle(KeyCode::Char(key));
  }
  editor.handle(KeyCode::Enter);
  assert_eq!(editor.page, AppearancePage::Blur);
  assert_eq!(blur_value(&editor, BlurField::Contrast), 0.5);
  assert_eq!(
    selected(&editor),
    Some(Item::BlurField(BlurField::Contrast))
  );
  assert!(editor.action.is_none(), "nothing applied");

  let mut section = draft_app(AppearancePage::SurfaceSection {
    surface: EffectSurface::Taskbar,
    section: SurfaceSection::Transparency,
  });
  select(&mut section, Item::SectionValue);
  section.handle(KeyCode::Enter);
  section.handle(KeyCode::Char('0'));
  section.handle(KeyCode::Enter);
  assert_eq!(section.surface_draft().unwrap().transparency, 0);
  assert_eq!(selected(&section), Some(Item::SectionValue));
  assert!(section.action.is_none());

  // Out of range keeps the prompt open with an error.
  // Size accepts 1–64: 200 keeps the prompt open and leaves the draft alone.
  let mut invalid = app(AppearancePage::Blur);
  select(&mut invalid, Item::BlurField(BlurField::Size));
  invalid.handle(KeyCode::Enter);
  for key in ['2', '0', '0'] {
    invalid.handle(KeyCode::Char(key));
  }
  invalid.handle(KeyCode::Enter);
  assert!(invalid.prompt_error.is_some());
  assert_eq!(invalid.blur_draft, None);
}

#[test]
fn reload_keeps_a_changed_draft_and_rebuilds_a_clean_one() {
  let mut application = draft_app(AppearancePage::TaskbarTime);
  let mut loaded = application.state.clone();
  loaded.taskbar_time_seconds_enabled = !loaded.taskbar_time_seconds_enabled;

  // Clean: the draft follows the reloaded state.
  application.on_loaded(AppearancePage::TaskbarTime, loaded.clone());
  assert_eq!(
    application.surface_draft().unwrap().time_seconds_enabled,
    loaded.taskbar_time_seconds_enabled
  );

  // Changed: the user's edit survives the reload.
  application.activate(Item::DateFormat(TaskbarDateFormat::WeekdayDayMonthYear));
  application.on_loaded(AppearancePage::TaskbarTime, loaded);
  assert_eq!(
    application.surface_draft().unwrap().date_format,
    TaskbarDateFormat::WeekdayDayMonthYear
  );
}

#[test]
fn surface_value_navigation_keeps_vertical_focus_and_ends_with_apply() {
  for surface in EffectSurface::ALL {
    let mut application = draft_app(AppearancePage::SurfaceSection {
      surface,
      section: SurfaceSection::Transparency,
    });

    application.handle(KeyCode::Down);
    assert_eq!(selected(&application), Some(Item::SectionValue));
    application.handle(KeyCode::Up);
    assert_eq!(selected(&application), Some(Item::SectionEnabled));

    select(&mut application, Item::SectionValue);
    let value = |application: &AppearanceApp| application.surface_draft().unwrap().transparency;
    let initial = value(&application);
    application.handle(KeyCode::Char('+'));
    assert_eq!(selected(&application), Some(Item::SectionValue));
    assert_eq!(value(&application), (initial + 5).min(100));

    application.handle(KeyCode::End);
    assert_eq!(selected(&application), Some(Item::Apply));
    application.handle(KeyCode::Enter);
    assert!(application.action.is_some());
  }
}

#[test]
fn apply_keeps_rendering_the_just_applied_draft_until_refresh_replaces_it() {
  // Regression test: pressing Apply used to clear `surface_draft`
  // immediately, which made every toggle/radio on the page flash back to
  // its pre-edit (unchecked/default) state for the whole apply+refresh
  // round trip, since the rows all render from `surface_draft()`.
  let mut application = draft_app(AppearancePage::TaskbarIcons);
  let draft = application.surface_draft.as_mut().unwrap();
  draft.audio_player_enabled = false;
  draft.launcher_enabled = false;

  application.apply_surface_changes();

  let draft = application
    .surface_draft()
    .expect("the just-edited draft must still be present right after Apply");
  assert!(!draft.audio_player_enabled);
  assert!(!draft.launcher_enabled);
  assert!(application.action.is_some());
}

#[test]
fn card_availability_arrives_without_blocking_or_dirtying_the_draft() {
  let mut application = draft_app(AppearancePage::ControlPanel);
  application.control_panel_draft = Some(application.state.control_panel_cards.clone());
  application.availability = Some(application.manager.spawn(|_| {
    Ok(JobData::Availability(vec![(
      ControlPanelCard::Bluetooth,
      false,
    )]))
  }));
  assert!(
    application.job.is_none(),
    "the probe does not make the page busy"
  );

  poll_until(&mut application, |application| {
    application.availability.is_none()
  });

  assert!(
    !application
      .state
      .control_panel_cards
      .available(ControlPanelCard::Bluetooth)
  );
  assert!(application.surface_draft().is_some_and(|draft| {
    !draft
      .control_panel_cards
      .available(ControlPanelCard::Bluetooth)
  }));
  assert!(
    application
      .control_panel_draft
      .as_ref()
      .is_some_and(|draft| !draft.available(ControlPanelCard::Bluetooth))
  );
  assert!(!application.has_pending_changes());

  // A page load that copied the state before the probe finished must not
  // bring the old availability back.
  application.on_loaded(AppearancePage::ControlPanel, AppearanceState::default());
  assert!(
    !application
      .state
      .control_panel_cards
      .available(ControlPanelCard::Bluetooth)
  );
  assert!(!application.has_pending_changes());
}
