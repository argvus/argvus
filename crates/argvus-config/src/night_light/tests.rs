use super::*;
use chrono::FixedOffset;
use std::cell::{Cell, RefCell};

struct FakeBackend {
  state: RefCell<(bool, u32)>,
  calls: Cell<usize>,
  fail: bool,
}

impl Backend for FakeBackend {
  fn state(&self) -> ConfigResult<(bool, u32)> {
    Ok(*self.state.borrow())
  }
  fn apply(&self, active: bool, temperature: u32) -> ConfigResult<()> {
    self.calls.set(self.calls.get() + 1);
    if self.fail {
      return Err(invalid("simulated IPC failure"));
    }
    *self.state.borrow_mut() = (active, temperature);
    Ok(())
  }
}

#[test]
fn validates_types_times_and_temperature_before_storage() {
  for value in [
    serde_json::json!({"enabled": "true"}),
    serde_json::json!({"temperature": 999}),
    serde_json::json!({"temperature": 20001}),
    serde_json::json!({"temperature": 4500.5}),
    serde_json::json!({"start":"24:00"}),
    serde_json::json!({"end":"7:00"}),
    serde_json::json!({"start":"07:00", "end":"07:00"}),
    Value::Null,
  ] {
    assert!(Settings::from_value(&value).is_err(), "{value}");
    let mut document = crate::ConfigDocument::default();
    assert!(document.set(POINTER, value).is_err());
  }
  for temperature in [MIN_TEMPERATURE, MAX_TEMPERATURE] {
    assert!(Settings::from_value(&serde_json::json!({"temperature":temperature})).is_ok());
  }
  assert!(!Settings::default().enabled);
  assert!(!Settings::default().schedule_enabled);
}

#[test]
fn windows_include_start_and_exclude_end_across_midnight() {
  let mut settings = Settings::default();
  for (minute, active) in [
    (0, true),
    (419, true),
    (420, false),
    (1199, false),
    (1200, true),
    (1439, true),
  ] {
    assert_eq!(settings.active_at(minute), active);
  }
  settings.start = "09:00".into();
  settings.end = "17:00".into();
  for (minute, active) in [(539, false), (540, true), (1019, true), (1020, false)] {
    assert_eq!(settings.active_at(minute), active);
  }
}

#[test]
fn transition_follows_local_zone_and_expires_override_at_boundary() {
  let settings = Settings {
    schedule_enabled: true,
    ..Settings::default()
  };
  let utc = Utc.with_ymd_and_hms(2026, 10, 29, 21, 0, 0).unwrap();
  let brazil = utc.with_timezone(&FixedOffset::west_opt(3 * 3600).unwrap());
  let next_utc = next_transition(&settings, &utc).unwrap();
  let next_brazil = next_transition(&settings, &brazil).unwrap();
  assert_ne!(next_utc, next_brazil);
  let overlay = Override {
    active: true,
    expires_at: next_brazil,
    settings: settings.clone(),
    instance: "session".into(),
  };
  assert!(overlay.valid(&settings, Some(next_brazil), "session"));
  assert!(!overlay.valid(&settings, Some(next_utc), "session"));
  assert!(!overlay.valid(&settings, Some(next_brazil), "new-session"));
  let boundary = brazil.timezone().timestamp_opt(next_brazil, 0).unwrap();
  assert!(!overlay.valid(&settings, next_transition(&settings, &boundary), "session"));
}

#[test]
fn application_is_idempotent_and_propagates_ipc_failure() {
  let backend = FakeBackend {
    state: RefCell::new((false, 6000)),
    calls: Cell::new(0),
    fail: false,
  };
  apply_if_needed(&backend, false, 4500).unwrap();
  assert_eq!(backend.calls.get(), 0);
  apply_if_needed(&backend, true, 4500).unwrap();
  apply_if_needed(&backend, true, 4500).unwrap();
  assert_eq!(backend.calls.get(), 1);
  apply_if_needed(&backend, false, 4500).unwrap();
  assert_eq!(backend.calls.get(), 2);
  let failed = FakeBackend {
    fail: true,
    ..backend
  };
  assert!(apply_if_needed(&failed, true, 4500).is_err());
  assert!(!failed.state.borrow().0);
}

#[test]
fn persistent_manual_changes_and_volatile_scheduled_changes_use_one_store() {
  let root = PathBuf::from(format!(
    "/tmp/argvus/night-light-test-{}-{}",
    std::process::id(),
    Utc::now().timestamp_nanos_opt().unwrap()
  ));
  fs::create_dir_all(&root).unwrap();
  let store = ConfigStore::at(&root);
  let backend = FakeBackend {
    state: RefCell::new((false, 6000)),
    calls: Cell::new(0),
    fail: false,
  };
  execute_locked(&store, "enable", &[], &root, &backend).unwrap();
  assert!(load_settings(&store).unwrap().enabled);
  execute_locked(&store, "apply-schedule", &[], &root, &backend).unwrap();
  assert!(backend.state.borrow().0);
  assert_eq!(backend.calls.get(), 1);
  execute_locked(
    &store,
    "schedule",
    &["on".into(), "20:00".into(), "07:00".into()],
    &root,
    &backend,
  )
  .unwrap();
  let before = load_settings(&store).unwrap();
  let toggled = execute_locked(&store, "toggle", &[], &root, &backend)
    .unwrap()
    .unwrap();
  assert!(toggled.manual_override);
  assert_eq!(load_settings(&store).unwrap(), before);
  assert!(root.join("override.json").exists());
  let calls = backend.calls.get();
  execute_locked(&store, "apply-schedule", &[], &root, &backend).unwrap();
  assert_eq!(backend.calls.get(), calls);
  // Configuration changes invalidate the volatile override.
  execute_locked(&store, "temperature", &["5000".into()], &root, &backend).unwrap();
  assert!(!root.join("override.json").exists());
  assert_eq!(load_settings(&store).unwrap().temperature, 5000);
  assert!(execute_locked(&store, "temperature", &["999".into()], &root, &backend).is_err());
  assert_eq!(load_settings(&store).unwrap().temperature, 5000);
  fs::remove_dir_all(root).unwrap();
}

#[test]
fn failed_application_keeps_saved_preference_and_reports_effective_state() {
  let root = PathBuf::from(format!(
    "/tmp/argvus/night-light-failure-{}-{}",
    std::process::id(),
    Utc::now().timestamp_nanos_opt().unwrap()
  ));
  fs::create_dir_all(&root).unwrap();
  let store = ConfigStore::at(&root);
  let backend = FakeBackend {
    state: RefCell::new((false, 6000)),
    calls: Cell::new(0),
    fail: true,
  };
  assert!(execute_locked(&store, "enable", &[], &root, &backend).is_err());
  let status = execute_locked(&store, "status", &[], &root, &backend)
    .unwrap()
    .unwrap();
  assert!(status.settings.enabled);
  assert_eq!(status.active, Some(false));
  assert_eq!(status.effective_temperature, None);
  fs::remove_dir_all(root).unwrap();
}

#[test]
fn schedule_handles_missing_and_repeated_dst_wall_times() {
  let zone = chrono_tz::America::New_York;
  let settings = Settings {
    schedule_enabled: true,
    start: "02:30".into(),
    end: "07:00".into(),
    ..Settings::default()
  };
  let before_gap = zone.with_ymd_and_hms(2026, 3, 8, 1, 59, 30).unwrap();
  let transition = zone
    .timestamp_opt(next_transition(&settings, &before_gap).unwrap(), 0)
    .unwrap();
  assert_eq!((transition.hour(), transition.minute()), (3, 0));
  let settings = Settings {
    start: "01:30".into(),
    ..settings
  };
  let before_repeat = zone.with_ymd_and_hms(2026, 11, 1, 0, 59, 0).unwrap();
  let first_start = zone
    .timestamp_opt(next_transition(&settings, &before_repeat).unwrap(), 0)
    .unwrap();
  assert_eq!((first_start.hour(), first_start.minute()), (1, 30));
  let fallback = zone
    .timestamp_opt(next_transition(&settings, &first_start).unwrap(), 0)
    .unwrap();
  assert_eq!((fallback.hour(), fallback.minute()), (1, 0));
}
