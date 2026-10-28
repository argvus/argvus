//! Mirror pages: the reflector editor, its bounded fields and the preview.

use super::*;

#[test]
fn configure_mirrors_is_disabled_without_reflector() {
  let mut app = app();
  app.page = PackagesPage::Mirrors;
  app.mirrors = vec![Mirror {
    server: "https://mirror.example/$repo/os/$arch".into(),
    enabled: true,
    ..Default::default()
  }];
  let rows = app.rows();
  let configure = row(&rows, Item::ConfigureMirrors);
  assert!(!configure.is_selectable());
  assert_eq!(
    configure.detail_text(),
    Some(tr(app.lang, "control_center.unavailable_no_reflector"))
  );
  assert_eq!(
    rows.last().unwrap().detail_text(),
    Some(tr(app.lang, "control_center.mirror_enabled"))
  );
}

#[test]
fn mirror_editor_is_a_page_with_adjustable_options() {
  let caps = Capabilities {
    has_reflector: true,
    ..Default::default()
  };
  let mut app = PackagesApp::new(Lang::for_locale("en-US"), Theme::load(), caps);
  app.mirror_countries = Some(vec!["Brazil".into(), "Argentina".into()]);
  app.page = PackagesPage::Mirrors;
  app.handle(KeyCode::Tab);
  app.handle(KeyCode::Enter);
  assert_eq!(app.page, PackagesPage::MirrorEditor);
  let countries = |app: &PackagesApp| app.mirror_options.as_ref().unwrap().countries.clone();
  assert_eq!(countries(&app), ["Brazil"]);
  app.handle(KeyCode::Right);
  assert_eq!(countries(&app), ["Argentina"]);
  app.handle(KeyCode::Left);
  assert_eq!(countries(&app), ["Brazil"], "← moves back");
  app.handle(KeyCode::Char(' '));
  assert_eq!(countries(&app), ["Argentina"], "Space moves forward");

  app.handle(KeyCode::Down);
  app.handle(KeyCode::Down);
  app.handle(KeyCode::Left);
  assert_eq!(app.mirror_options.as_ref().unwrap().age_hours, 11);
  app.handle(KeyCode::Down);
  app.handle(KeyCode::Right);
  assert_eq!(app.mirror_options.as_ref().unwrap().count, 11);
  app.handle(KeyCode::Down);
  app.handle(KeyCode::Enter);
  assert_eq!(app.mirror_options.as_ref().unwrap().sort, "age");

  app.handle(KeyCode::Esc);
  assert_eq!(app.page, PackagesPage::Mirrors);
  assert!(app.mirror_options.is_none());
  assert_eq!(
    app.menu.selected_id(&app.rows()),
    Some(Item::ConfigureMirrors)
  );
}

#[test]
fn mirror_numbers_open_a_bounded_field() {
  let mut app = app();
  app.page = PackagesPage::MirrorEditor;
  app.mirror_options = Some(ReflectorOptions::default());
  press(&mut app, Item::MirrorCount);
  assert!(app.captures_text());
  for key in ['1', 'x', '5', '0', '0'] {
    app.handle(KeyCode::Char(key));
  }
  assert_eq!(app.input.as_deref(), Some("150"));
  app.handle(KeyCode::Enter);
  assert_eq!(app.mirror_options.as_ref().unwrap().count, 10);
  assert_eq!(
    app.status.as_ref().map(|status| status.kind),
    Some(StatusKind::Error),
    "values out of 1–100 are rejected"
  );
  press(&mut app, Item::MirrorAge);
  for key in ['4', '8'] {
    app.handle(KeyCode::Char(key));
  }
  app.handle(KeyCode::Enter);
  assert_eq!(app.mirror_options.as_ref().unwrap().age_hours, 48);
}

#[test]
fn mirror_preview_returns_to_mirrors_and_asks_for_confirmation() {
  let mut app = app();
  app.page = PackagesPage::MirrorEditor;
  app.mirror_options = Some(ReflectorOptions::default());
  app.apply(Loaded::MirrorPreview(
    "Server = https://a/$repo/os/$arch\n".into(),
  ));
  assert_eq!(app.page, PackagesPage::Mirrors);
  assert!(matches!(app.pending, Some(Action::ApplyMirrors(_))));
}

#[test]
fn country_cycling_falls_back_to_a_small_static_list_until_reflector_loads() {
  let mut options = ReflectorOptions {
    countries: vec!["Brazil".into()],
    ..Default::default()
  };
  cycle_country(&mut options, &[], 1);
  assert_eq!(options.countries, ["United States"]);
  cycle_country(&mut options, &["Brazil".into(), "Argentina".into()], 1);
  assert_eq!(options.countries, ["Brazil"]);
  cycle_country(&mut options, &["Brazil".into(), "Argentina".into()], 1);
  assert_eq!(options.countries, ["Argentina"]);
  cycle_country(&mut options, &["Brazil".into(), "Argentina".into()], 1);
  assert!(options.countries.is_empty(), "wraps to every country");
  cycle_country(&mut options, &["Brazil".into(), "Argentina".into()], -1);
  assert_eq!(options.countries, ["Argentina"]);
}
