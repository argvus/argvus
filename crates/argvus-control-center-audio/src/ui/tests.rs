use super::*;
use ratatui::{Terminal, backend::TestBackend};

/// Builds an app on an output page with one device, without touching the audio server.
fn app_with_output(volume: Option<u8>) -> AudioApp {
  let mut app = AudioApp::new(
    Lang::for_locale("en-US"),
    Theme::load(),
    Capabilities::default(),
  );
  app.snapshot.available = true;
  app.snapshot.outputs = vec![AudioDevice {
    id: 1,
    description: "Speakers".into(),
    direction: "output".into(),
    volume,
    ..Default::default()
  }];
  app.page = AudioPage::Output;
  app
}

#[test]
fn audio_home_rows_act_as_a_status_dashboard() {
  let mut app = AudioApp::new(
    Lang::for_locale("en-US"),
    Theme::load(),
    Capabilities::default(),
  );
  app.snapshot.available = true;
  app.snapshot.backend = "PipeWire".into();
  app.snapshot.outputs = vec![AudioDevice {
    id: 42,
    description: "Speakers".into(),
    direction: "output".into(),
    volume: Some(80),
    ..Default::default()
  }];
  app.snapshot.inputs = vec![AudioDevice {
    id: 43,
    description: "Mic".into(),
    direction: "input".into(),
    volume: Some(50),
    ..Default::default()
  }];
  app.snapshot.default_output = Some(42);
  app.snapshot.default_input = Some(43);

  let rows = app.rows();
  assert_eq!(rows.len(), 4);
  assert!(rows[0].contains("Summary") || rows[0].contains("Resumo"));
  assert!(rows[0].contains("PipeWire"));
  assert!(rows[1].contains("Output") || rows[1].contains("Saída"));
  assert!(rows[1].contains("Speakers"));
  assert!(rows[2].contains("Input") || rows[2].contains("Entrada"));
  assert!(rows[2].contains("Mic"));
  assert!(rows[3].contains("2"));
}

#[test]
fn audio_device_rows_render_clean_badges() {
  let mut app = AudioApp::new(
    Lang::for_locale("en-US"),
    Theme::load(),
    Capabilities::default(),
  );
  app.snapshot.available = true;
  app.snapshot.outputs = vec![
    AudioDevice {
      id: 1,
      description: "Built-in Audio".into(),
      direction: "output".into(),
      volume: Some(70),
      muted: false,
      ..Default::default()
    },
    AudioDevice {
      id: 2,
      description: "USB Headset".into(),
      direction: "output".into(),
      volume: Some(50),
      muted: true,
      ..Default::default()
    },
  ];
  app.snapshot.default_output = Some(1);
  app.page = AudioPage::Output;

  let rows = app.rows();
  assert_eq!(rows.len(), 2);
  assert!(rows[0].contains("Built-in Audio · 70%"));
  assert!(rows[0].contains("Default") || rows[0].contains("Padrão"));
  assert!(rows[1].contains("USB Headset · 50%"));
  assert!(rows[1].contains("Muted") || rows[1].contains("Mudo"));
}

#[test]
fn audio_home_and_details_are_keyboard_navigable() {
  let mut app = AudioApp::new(
    Lang::for_locale("en-US"),
    Theme::load(),
    Capabilities::default(),
  );
  app.snapshot.outputs = vec![AudioDevice {
    id: 1,
    description: "Speakers".into(),
    direction: "output".into(),
    ..Default::default()
  }];
  app.handle(KeyCode::Down); // select Output
  app.handle(KeyCode::Enter);
  assert_eq!(app.page, AudioPage::Output);
  app.job = None;
  app.handle(KeyCode::Esc);
  assert_eq!(app.page, AudioPage::Home);
}

#[test]
/// Enter stages the default device; nothing reaches the system until `s`, and leaving asks first.
fn audio_default_stays_in_draft_until_save() {
  let mut app = AudioApp::new(
    Lang::for_locale("en-US"),
    Theme::load(),
    Capabilities::default(),
  );
  app.snapshot.outputs = vec![AudioDevice {
    id: 10,
    description: "Headphones".into(),
    direction: "output".into(),
    ..Default::default()
  }];
  app.page = AudioPage::Output;
  app.handle(KeyCode::Enter);
  assert_eq!(app.draft.default_output, Some(10));
  assert!(app.action.is_none());
  assert!(app.has_changes());
  app.handle(KeyCode::Esc); // asks before dropping the draft
  assert!(app.confirm_discard);
  app.handle(KeyCode::Esc); // cancel keeps the page and the draft
  assert!(!app.confirm_discard);
  assert_eq!(app.page, AudioPage::Output);
  assert_eq!(app.draft.default_output, Some(10));
}

#[test]
/// Volume and mute keys change only the draft and show it in the rows.
fn audio_volume_and_mute_stay_in_draft_until_save() {
  let mut app = app_with_output(Some(70));
  app.handle(KeyCode::Right);
  assert_eq!(app.draft.volumes.get(&1), Some(&75));
  app.handle(KeyCode::Char(' '));
  assert_eq!(app.draft.mutes.get(&1), Some(&true));
  assert!(app.action.is_none());
  let rows = app.rows();
  assert!(rows[0].contains("75%"));
  assert!(rows[0].contains("Muted") || rows[0].contains("Mudo"));
}

#[test]
/// Saving with no pending change reports it instead of touching the audio server.
fn audio_save_without_changes_reports_nothing_to_save() {
  let mut app = app_with_output(Some(70));
  app.handle(KeyCode::Char('s'));
  assert!(app.action.is_none());
  assert_eq!(
    app.status.as_ref().map(|s| s.text.as_str()),
    Some(tr(app.lang, "control_center.nothing_to_save"))
  );
}

#[test]
/// Device pages expose their actions and the save key in the footer and render no button bar.
fn audio_device_footer_lists_volume_default_and_save_keys() {
  let app = app_with_output(Some(70));
  assert_eq!(
    app.footer_hints(),
    tr(
      app.lang,
      "control_center.navigate_volume_enter_set_default_r_refresh_s_save_esc_back_help"
    )
  );
}

#[test]
fn audio_uses_shared_chrome_and_contextual_footer() {
  let app = AudioApp::new(
    Lang::for_locale("en-US"),
    Theme::load(),
    Capabilities::default(),
  );
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
  assert!(text.contains("Audio") || text.contains("Áudio"));
  assert!(text.contains("Enter") && text.contains("Back"));
}
