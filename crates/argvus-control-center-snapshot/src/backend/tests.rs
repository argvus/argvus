use super::*;
use serde_json::json;
use std::collections::VecDeque;
use std::sync::Mutex;

struct Fake {
  replies: Mutex<VecDeque<Result<ProcessOutput, ProcessError>>>,
  requests: Mutex<Vec<ProcessRequest>>,
  events: Option<std::sync::Arc<Mutex<Vec<String>>>>,
}
impl ProcessRunner for Fake {
  fn run(&self, request: &ProcessRequest) -> Result<ProcessOutput, ProcessError> {
    self.requests.lock().unwrap().push(request.clone());
    if let Some(events) = &self.events {
      events
        .lock()
        .unwrap()
        .push(format!("{} {:?}", request.program, request.args));
    }
    self
      .replies
      .lock()
      .unwrap()
      .pop_front()
      .expect("unexpected process request")
  }
}
fn output(text: &str) -> Result<ProcessOutput, ProcessError> {
  Ok(ProcessOutput {
    stdout: text.as_bytes().into(),
    stderr: vec![],
    status: Some(0),
    timed_out: false,
    cancelled: false,
  })
}
fn backend(replies: Vec<Result<ProcessOutput, ProcessError>>) -> SnapshotBackend<Fake> {
  SnapshotBackend::new(Fake {
    replies: Mutex::new(replies.into()),
    requests: Mutex::new(vec![]),
    events: None,
  })
}
fn backend_with_events(
  replies: Vec<Result<ProcessOutput, ProcessError>>,
  events: std::sync::Arc<Mutex<Vec<String>>>,
) -> SnapshotBackend<Fake> {
  SnapshotBackend::new(Fake {
    replies: Mutex::new(replies.into()),
    requests: Mutex::new(vec![]),
    events: Some(events),
  })
}
fn root(filesystem: &str) -> String {
  json!({"filesystems":[{"fstype":filesystem,"source":"/dev/test","target":"/","options":"rw"}]})
    .to_string()
}
fn row(number: u32, active: bool, default: bool) -> Value {
  json!({"number":number,"type":"single","date":"2026-10-30 12:00:00","description":"manual","cleanup":"","active":active,"default":default})
}
fn ready(rows: Vec<Value>) -> Vec<Result<ProcessOutput, ProcessError>> {
  vec![
    output(&root("btrfs")),
    output("--jsonout"),
    output(r#"{"configs":[{"config":"system","subvolume":"/"}]}"#),
    output(r#"{"SUBVOLUME":"/","FSTYPE":"btrfs"}"#),
    output(&json!({"system":rows}).to_string()),
  ]
}

#[test]
fn parses_status_indicators_and_paths_without_losing_spaces() {
  let changes = parse_status(b"+..... /new file\n-..... /gone\ncpu gxa /broken\n").unwrap_err();
  assert_eq!(changes.kind, ErrorKind::Invalid);
  let changes =
    parse_status("+..... /new file\n-..... /gone\ncpugxa /módulo\nt..... /type\n".as_bytes())
      .unwrap();
  assert_eq!(changes.len(), 4);
  assert_eq!(changes[0].kind, ChangeKind::Created);
  assert_eq!(changes[1].kind, ChangeKind::Removed);
  assert_eq!(changes[2].path, "/módulo");
  assert!(
    changes[2].content
      && changes[2].permissions
      && changes[2].owner
      && changes[2].group
      && changes[2].xattrs
      && changes[2].acl
  );
  assert!(changes[3].file_type);
  for bad in [
    "...... /same",
    "z..... /bad",
    "c..... relative",
    "c..... /a/../b",
    "c..... /a\u{1b}",
  ] {
    assert!(parse_status(bad.as_bytes()).is_err(), "{bad}");
  }
}

#[test]
fn parses_retention_config_with_explicit_defaults_and_unknown_keys() {
  let config = parse_retention_config(
    br#"{"NUMBER_CLEANUP":"yes","NUMBER_LIMIT":"4-8","VENDOR_OPTION":"kept"}"#,
  )
  .unwrap();
  assert_eq!(config.values["NUMBER_CLEANUP"].value, "yes");
  assert_eq!(
    config.values["NUMBER_CLEANUP"].origin,
    SettingOrigin::Explicit
  );
  assert_eq!(config.values["NUMBER_LIMIT"].value, "4-8");
  assert_eq!(config.values["TIMELINE_MIN_AGE"].value, "3600");
  assert_eq!(
    config.values["TIMELINE_MIN_AGE"].origin,
    SettingOrigin::SnapperDefault
  );
  assert_eq!(config.unknown["VENDOR_OPTION"], "kept");
  for invalid in [
    br#""#.as_slice(),
    br#"[]"#,
    br#"{"NUMBER_CLEANUP":"maybe"}"#,
    br#"{"NUMBER_LIMIT":"8-4"}"#,
    br#"{"NUMBER_MIN_AGE":"-1"}"#,
  ] {
    assert!(parse_retention_config(invalid).is_err());
  }
}

#[test]
fn retention_configdata_and_helper_accept_only_validated_keys() {
  assert_eq!(
    build_retention_configdata(&[
      ("NUMBER_LIMIT".into(), "5-12".into()),
      ("NUMBER_CLEANUP".into(), "yes".into())
    ])
    .unwrap(),
    "NUMBER_LIMIT=5-12 NUMBER_CLEANUP=yes"
  );
  for pair in [
    ("SUBVOLUME", "/"),
    ("FSTYPE", "btrfs"),
    ("QGROUP", "1/0"),
    ("UNKNOWN", "yes"),
    ("TIMELINE_CREATE", "true"),
  ] {
    assert!(build_retention_configdata(&[(pair.0.into(), pair.1.into())]).is_err());
  }
  assert!(validate_helper("cleanup", &["system".into(), "oops".into()]).is_err());
  assert!(validate_helper("set-retention", &["system".into(), "SUBVOLUME=/".into()]).is_err());
}

#[test]
fn reads_systemd_timer_states_without_mutating_units() {
  let backend = backend(vec![
    output("LoadState=loaded\nUnitFileState=enabled\nActiveState=active\n"),
    output("LoadState=loaded\nUnitFileState=enabled\nActiveState=inactive\n"),
  ]);
  assert_eq!(
    backend.timers(),
    SnapperTimers {
      timeline: TimerState::Active,
      cleanup: TimerState::EnabledInactive
    }
  );
  let requests = backend.runner.requests.lock().unwrap();
  assert_eq!(requests.len(), 2);
  assert!(
    requests
      .iter()
      .all(|request| request.program == "/usr/bin/systemctl" && request.args[0] == "show")
  );
}

#[test]
fn reports_missing_and_unavailable_timer_states() {
  let backend = backend(vec![
    output("LoadState=not-found\n"),
    Err(ProcessError::OutputLimit),
  ]);
  let timers = backend.timers();
  assert_eq!(timers.timeline, TimerState::Missing);
  assert!(matches!(timers.cleanup, TimerState::Unavailable(_)));
}

#[test]
fn cleanup_runs_official_algorithm_and_reports_observed_removed_ids() {
  let mut replies = ready(vec![row(1, false, false), row(2, false, false)]);
  replies.push(output(
    &json!({"system":[row(1, false, false),row(2, false, false)]}).to_string(),
  ));
  replies.push(output(""));
  replies.push(output(
    &json!({"system":[row(2, false, false)]}).to_string(),
  ));
  let backend = backend(replies);
  let (removed, remaining) = backend
    .cleanup("system", &[CleanupAlgorithm::Number])
    .unwrap();
  assert_eq!(removed, [1]);
  assert_eq!(remaining, [2]);
  let requests = backend.runner.requests.lock().unwrap();
  assert_eq!(requests[6].args, ["-c", "system", "cleanup", "number"]);
  assert!(requests.len() > 7);
}

#[test]
fn cleanup_failure_is_not_reported_as_success() {
  let mut replies = ready(vec![row(4, false, false)]);
  replies.push(output(
    &json!({"system":[row(4, false, false)]}).to_string(),
  ));
  replies.push(Ok(ProcessOutput {
    stdout: vec![],
    stderr: b"cleanup failed".to_vec(),
    status: Some(1),
    timed_out: false,
    cancelled: false,
  }));
  let backend = backend(replies);
  assert!(
    backend
      .cleanup("system", &[CleanupAlgorithm::Timeline])
      .is_err()
  );
}

#[test]
fn retention_save_uses_one_configdata_argument_and_verifies_readback() {
  let mut replies = ready(vec![]);
  replies.push(output(""));
  replies.extend(ready(vec![]));
  replies.push(output("{\"NUMBER_LIMIT\":\"20\"}"));
  let backend = backend(replies);
  let saved = backend
    .set_retention("system", &[("NUMBER_LIMIT".into(), "20".into())])
    .unwrap();
  assert_eq!(saved.values["NUMBER_LIMIT"].value, "20");
  let requests = backend.runner.requests.lock().unwrap();
  let set = requests
    .iter()
    .find(|request| {
      request
        .args
        .get(2)
        .is_some_and(|argument| argument == "set-config")
    })
    .unwrap();
  assert_eq!(set.args, ["-c", "system", "set-config", "NUMBER_LIMIT=20"]);
}

#[test]
fn pre_upgrade_snapshot_is_confirmed_before_transaction_and_failure_keeps_it() {
  let events = std::sync::Arc::new(Mutex::new(Vec::new()));
  let mut replies = ready(vec![row(1, false, false)]);
  replies.extend(ready(vec![row(1, false, false)]));
  replies.push(output("42\n"));
  let snapshot_backend = backend_with_events(replies, events.clone());
  let id = run_pre_upgrade_transaction(&snapshot_backend, |snapshot_number| {
    assert_eq!(snapshot_number, 42);
    events.lock().unwrap().push("pacman starts".into());
    Ok(())
  })
  .unwrap();
  assert_eq!(id, 42);
  let events = events.lock().unwrap();
  let snapshot = events
    .iter()
    .position(|event| event.contains("create") && event.contains("--print-number"))
    .unwrap();
  let pacman = events
    .iter()
    .position(|event| event == "pacman starts")
    .unwrap();
  assert!(snapshot < pacman);

  let mut replies = ready(vec![row(1, false, false)]);
  replies.extend(ready(vec![row(1, false, false)]));
  replies.push(output("43\n"));
  let backend = backend(replies);
  let failure = run_pre_upgrade_transaction(&backend, |_| Err("pacman failed".into())).unwrap_err();
  assert!(failure.contains("ARGVUS_PRE_UPDATE_FAILED:43"));
}

#[test]
fn pre_upgrade_blocked_filesystem_never_starts_the_transaction() {
  let backend = backend(vec![output(&root("ext4"))]);
  let mut started = false;
  let error = run_pre_upgrade_transaction(&backend, |_| {
    started = true;
    Ok(())
  })
  .unwrap_err();
  assert!(error.contains("ARGVUS_PRE_UPDATE_BLOCKED"));
  assert!(!started);
  assert_eq!(backend.runner.requests.lock().unwrap().len(), 1);
}

#[test]
fn pre_upgrade_without_snapper_or_root_configuration_never_starts_the_transaction() {
  let missing_snapper = Err(ProcessError::Io(std::io::Error::from(
    IoErrorKind::NotFound,
  )));
  let missing_backend = backend(vec![output(&root("btrfs")), missing_snapper]);
  let mut started = false;
  assert!(
    run_pre_upgrade_transaction(&missing_backend, |_| {
      started = true;
      Ok(())
    })
    .is_err()
  );
  assert!(!started);

  let backend = backend(vec![
    output(&root("btrfs")),
    output("--jsonout"),
    output(r#"{"configs":[]}"#),
  ]);
  let mut started = false;
  assert!(
    run_pre_upgrade_transaction(&backend, |_| {
      started = true;
      Ok(())
    })
    .is_err()
  );
  assert!(!started);
}

#[test]
fn pre_upgrade_rejects_missing_and_malformed_snapshot_numbers() {
  for number in ["", "0", "not-a-number"] {
    let mut replies = ready(vec![row(1, false, false)]);
    replies.extend(ready(vec![row(1, false, false)]));
    replies.push(output(number));
    let backend = backend(replies);
    let mut started = false;
    assert!(
      run_pre_upgrade_transaction(&backend, |_| {
        started = true;
        Ok(())
      })
      .is_err()
    );
    assert!(!started);
  }
}

#[test]
fn comparison_rechecks_root_and_uses_structured_arguments() {
  let mut replies = ready(vec![row(1, false, false), row(2, false, false)]);
  replies.push(output("c..... /etc/a file\n"));
  let backend = backend(replies);
  let comparison = backend.compare("system", 1, 2).unwrap();
  assert_eq!(comparison.changes[0].path, "/etc/a file");
  let requests = backend.runner.requests.lock().unwrap();
  let status = requests.last().unwrap();
  assert_eq!(status.program, "/usr/bin/snapper");
  assert_eq!(status.args, ["-c", "system", "status", "1..2"]);
}

#[test]
fn diff_rechecks_status_and_limits_output() {
  let mut replies = ready(vec![row(1, false, false), row(2, false, false)]);
  replies.push(output("c..... /etc/a file\n"));
  replies.push(Err(ProcessError::OutputLimit));
  let backend = backend(replies);
  assert_eq!(
    backend.diff("system", 1, 2, "/etc/a file").unwrap(),
    FileDiff::Truncated
  );
  let requests = backend.runner.requests.lock().unwrap();
  let diff = requests.last().unwrap();
  assert_eq!(diff.args, ["-c", "system", "diff", "1..2", "/etc/a file"]);
  assert_eq!(diff.output_limit, Some(DIFF_LIMIT));
}

#[test]
fn diff_reports_non_utf8_and_rejects_unlisted_paths() {
  let mut replies = ready(vec![row(1, false, false), row(2, false, false)]);
  replies.push(output("c..... /etc/a\n"));
  let comparison_backend = backend(replies);
  assert_eq!(
    comparison_backend
      .diff("system", 1, 2, "/etc/b")
      .unwrap_err()
      .kind,
    ErrorKind::Invalid
  );
  assert_eq!(comparison_backend.runner.requests.lock().unwrap().len(), 6);

  let mut replies = ready(vec![row(1, false, false), row(2, false, false)]);
  replies.push(output("c..... /etc/a\n"));
  replies.push(Ok(ProcessOutput {
    stdout: vec![0xff],
    stderr: vec![],
    status: Some(0),
    timed_out: false,
    cancelled: false,
  }));
  assert_eq!(
    backend(replies).diff("system", 1, 2, "/etc/a").unwrap(),
    FileDiff::Binary
  );
}

#[test]
fn comparison_rejects_equal_ids_before_invoking_processes() {
  let backend = backend(vec![]);
  assert_eq!(
    backend.compare("system", 1, 1).unwrap_err().kind,
    ErrorKind::Invalid
  );
  assert!(backend.runner.requests.lock().unwrap().is_empty());
}

#[test]
fn identifies_only_the_root_mount() {
  for filesystem in ["btrfs", "ext4", "xfs", "overlay", "tmpfs"] {
    assert_eq!(parse_root(root(filesystem).as_bytes()).unwrap(), filesystem);
  }
  for text in [
    "",
    "{}",
    "not json",
    r#"{"filesystems":[]}"#,
    r#"{"filesystems":[{"fstype":"btrfs","target":"/home"}]}"#,
  ] {
    assert_eq!(
      parse_root(text.as_bytes()).unwrap_err().kind,
      ErrorKind::Detection
    );
  }
  for filesystem in ["", "unknown", "none", "auto"] {
    assert!(parse_root(root(filesystem).as_bytes()).is_err());
  }
}
#[test]
fn incompatible_filesystems_never_invoke_snapper() {
  for filesystem in ["ext4", "xfs", "overlay"] {
    let backend = backend(vec![output(&root(filesystem))]);
    assert_eq!(
      backend.inspect(None).unwrap(),
      Integration::NonBtrfs(filesystem.into())
    );
    assert_eq!(backend.runner.requests.lock().unwrap().len(), 1);
  }
}
#[test]
fn detection_failures_are_not_filesystem_incompatibility() {
  let missing = Err(ProcessError::Io(std::io::Error::from(
    IoErrorKind::NotFound,
  )));
  for result in [missing, output(""), output("{bad")] {
    assert_eq!(
      backend(vec![result]).inspect(None).unwrap_err().kind,
      ErrorKind::Detection
    );
  }
}
#[test]
fn distinguishes_missing_snapper_and_no_configuration() {
  let missing = Err(ProcessError::Io(std::io::Error::from(
    IoErrorKind::NotFound,
  )));
  assert_eq!(
    backend(vec![output(&root("btrfs")), missing])
      .inspect(None)
      .unwrap(),
    Integration::MissingSnapper
  );
  assert_eq!(
    backend(vec![
      output(&root("btrfs")),
      output("--jsonout"),
      output(r#"{"configs":[]}"#)
    ])
    .inspect(None)
    .unwrap(),
    Integration::NoConfig
  );
}
#[test]
fn rejects_root_named_configuration_pointing_elsewhere() {
  assert_eq!(
    backend(vec![
      output(&root("btrfs")),
      output("--jsonout"),
      output(r#"{"configs":[{"config":"root","subvolume":"/home"}]}"#)
    ])
    .inspect(None)
    .unwrap(),
    Integration::NoConfig
  );
  assert_eq!(
    backend(vec![
      output(&root("btrfs")),
      output("--jsonout"),
      output(r#"{"configs":[{"config":"root","subvolume":"/"}]}"#),
      output(r#"{"SUBVOLUME":"/home","FSTYPE":"btrfs"}"#)
    ])
    .inspect(None)
    .unwrap(),
    Integration::NoConfig
  );
}
#[test]
fn parses_empty_and_populated_lists_and_rejects_missing_protection() {
  assert!(
    parse_snapshots(br#"{"system":[]}"#, "system")
      .unwrap()
      .is_empty()
  );
  let snapshots = parse_snapshots(
    json!({"system":[row(0,false,false),row(2,true,false),row(3,false,true),row(4,false,false)]})
      .to_string()
      .as_bytes(),
    "system",
  )
  .unwrap();
  assert_eq!(
    snapshots
      .iter()
      .map(Snapshot::deletable)
      .collect::<Vec<_>>(),
    [false, false, false, true]
  );
  for text in ["", "[]", "{}", r#"{"system":[{"number":1}]}"#] {
    assert!(parse_snapshots(text.as_bytes(), "system").is_err());
  }
  let mut malformed = row(1, false, false);
  malformed.as_object_mut().unwrap().remove("active");
  assert!(
    parse_snapshots(
      json!({"system":[malformed]}).to_string().as_bytes(),
      "system"
    )
    .is_err()
  );
}
#[test]
fn create_keeps_special_description_in_one_argument() {
  let description = "backup $HOME; `id` ' \\\" á with spaces";
  let mut replies = ready(vec![]);
  replies.push(output("42\n"));
  let backend = backend(replies);
  assert_eq!(
    backend.change("create", "system", description).unwrap(),
    Some(42)
  );
  let requests = backend.runner.requests.lock().unwrap();
  let request = requests.last().unwrap();
  assert_eq!(request.program, SNAPPER);
  assert_eq!(
    request.args,
    [
      "-c",
      "system",
      "create",
      "--type",
      "single",
      "--description",
      description,
      "--print-number"
    ]
  );
  assert_eq!(request.timeout, Some(Duration::from_secs(120)));
  assert_eq!(request.output_limit, Some(LIMIT));
}
#[test]
fn deletion_requires_one_existing_unprotected_positive_number() {
  for number in ["0", "-1", "1-4", "1 2", "1;id", "+1", "4294967296"] {
    assert!(parse_number(number).is_err());
  }
  for (number, rows) in [
    ("2", vec![row(2, true, false)]),
    ("3", vec![row(3, false, true)]),
    ("7", vec![row(4, false, false)]),
  ] {
    let backend = backend(ready(rows));
    assert!(backend.change("delete", "system", number).is_err());
    assert_eq!(backend.runner.requests.lock().unwrap().len(), 5);
  }
  let mut replies = ready(vec![row(4, false, false)]);
  replies.push(output(""));
  let backend = backend(replies);
  assert_eq!(backend.change("delete", "system", "4").unwrap(), None);
  assert_eq!(
    backend.runner.requests.lock().unwrap().last().unwrap().args,
    ["-c", "system", "delete", "4"]
  );
}
#[test]
fn helper_grammar_and_description_are_restricted() {
  for (action, values) in [
    ("delete", vec!["system", "0"]),
    ("delete", vec!["system", "1-4"]),
    ("create-config", vec!["/"]),
    ("inspect", vec!["../etc"]),
    ("create", vec!["system", "bad\ntext"]),
  ] {
    assert!(
      validate_helper(
        action,
        &values.into_iter().map(String::from).collect::<Vec<_>>()
      )
      .is_err()
    );
  }
  assert!(validate_description(&"á".repeat(256)).is_ok());
  assert!(validate_description(&"a".repeat(257)).is_err());
}
#[test]
fn distinguishes_permission_authentication_timeout_and_failure() {
  for (status, timed_out, stderr, kind) in [
    (1, false, "Permission denied", ErrorKind::Permission),
    (126, false, "cancelled", ErrorKind::Authentication),
    (
      127,
      false,
      "authorization failed",
      ErrorKind::Authentication,
    ),
    (1, true, "", ErrorKind::Timeout),
    (1, false, "snapper error", ErrorKind::Snapper),
  ] {
    let error = check_output(ProcessOutput {
      stdout: vec![],
      stderr: stderr.as_bytes().into(),
      status: Some(status),
      timed_out,
      cancelled: false,
    })
    .unwrap_err();
    assert_eq!(error.kind, kind);
  }
}

#[test]
fn multiple_root_configs_require_an_explicit_choice() {
  let configs =
    r#"{"configs":[{"config":"one","subvolume":"/"},{"config":"two","subvolume":"/"}]}"#;
  let settings = r#"{"SUBVOLUME":"/","FSTYPE":"btrfs"}"#;
  let replies = || {
    vec![
      output(&root("btrfs")),
      output("--jsonout"),
      output(configs),
      output(settings),
      output(settings),
    ]
  };
  assert_eq!(
    backend(replies()).inspect(None).unwrap(),
    Integration::ChooseConfig(vec!["one".into(), "two".into()])
  );
  let mut selected = replies();
  selected.push(output(r#"{"two":[]}"#));
  assert_eq!(
    backend(selected).inspect(Some("two")).unwrap(),
    Integration::Ready {
      config: "two".into(),
      snapshots: vec![]
    }
  );
  assert!(backend(replies()).inspect(Some("other")).is_err());
}

#[test]
fn refuses_mutation_when_root_changed_or_config_disappeared() {
  let non_btrfs = backend(vec![output(&root("ext4"))]);
  assert!(non_btrfs.change("create", "system", "manual").is_err());
  assert_eq!(non_btrfs.runner.requests.lock().unwrap().len(), 1);
  let backend = backend(vec![
    output(&root("btrfs")),
    output("--jsonout"),
    output(r#"{"configs":[]}"#),
  ]);
  assert!(backend.change("delete", "system", "4").is_err());
}

#[test]
fn all_queries_are_bounded_and_never_use_a_shell() {
  let backend = backend(ready(vec![]));
  backend.inspect(None).unwrap();
  for request in backend.runner.requests.lock().unwrap().iter() {
    assert!(matches!(request.program.as_str(), FINDMNT | SNAPPER));
    assert_eq!(request.output_limit, Some(LIMIT));
    assert_eq!(request.timeout, Some(Duration::from_secs(15)));
    assert!(request.stdin.is_none());
  }
}

#[test]
fn sanitizes_failure_output_and_distinguishes_bad_versions() {
  let error = check_output(ProcessOutput {
    stdout: vec![],
    stderr: b"\x1b[31mpermission denied\x1b[0m\x07".to_vec(),
    status: Some(1),
    timed_out: false,
    cancelled: false,
  })
  .unwrap_err();
  assert_eq!(error.kind, ErrorKind::Permission);
  assert!(!error.detail.contains('\x1b'));
  assert_eq!(
    backend(vec![output(&root("btrfs")), output("old help")])
      .inspect(None)
      .unwrap_err()
      .kind,
    ErrorKind::Incompatible
  );
  let failure = Ok(ProcessOutput {
    stdout: vec![],
    stderr: b"error".to_vec(),
    status: Some(1),
    timed_out: false,
    cancelled: false,
  });
  assert_eq!(
    backend(vec![failure]).inspect(None).unwrap_err().kind,
    ErrorKind::Detection
  );
}

fn restore_fixture(
  replies: Vec<Result<ProcessOutput, ProcessError>>,
) -> (SnapshotBackend<Fake>, tempfile::TempDir) {
  use std::os::unix::fs::PermissionsExt;

  let directory = tempfile::tempdir().unwrap();
  let root_path = directory.path().join("root");
  let snapshots_path = root_path.join(".snapshots");
  std::fs::create_dir_all(root_path.join("etc")).unwrap();
  let source_path = snapshots_path.join("2/snapshot/etc/app.conf");
  std::fs::create_dir_all(source_path.parent().unwrap()).unwrap();
  std::fs::write(&source_path, "snapshot version\n").unwrap();
  std::fs::set_permissions(&source_path, std::fs::Permissions::from_mode(0o644)).unwrap();
  let mountinfo = directory.path().join("mountinfo");
  std::fs::write(&mountinfo, "1 0 0:1 / / rw - btrfs /dev/test rw\n").unwrap();
  let mut backend = backend(replies);
  let lock_path = root_path.join("restore.lock");
  let package_lock = root_path.join("package.lock");
  backend.restore_paths = RestorePaths {
    filesystem_uuid: |_| Ok([1; 16]),
    root: root_path,
    snapshots: snapshots_path,
    lock: lock_path,
    package_lock,
    mountinfo,
  };
  (backend, directory)
}

fn restore_preview_replies() -> Vec<Result<ProcessOutput, ProcessError>> {
  let mut replies = ready(vec![row(1, false, false), row(2, false, false)]);
  replies.push(output("c..... /etc/app.conf\n"));
  replies.push(output("--- old\n+++ snapshot\n@@ -1 +1 @@\n-old\n+new\n"));
  replies
}

fn restore_success_replies() -> Vec<Result<ProcessOutput, ProcessError>> {
  let mut replies = restore_preview_replies();
  replies.extend(restore_preview_replies());
  replies.extend(ready(vec![row(1, false, false), row(2, false, false)]));
  replies.push(output("9\n"));
  replies.extend(ready(vec![
    row(1, false, false),
    row(2, false, false),
    row(9, false, false),
  ]));
  replies.push(output(""));
  replies.push(output(""));
  replies
}

#[test]
fn selective_restore_reviews_then_uses_safety_snapshot_and_official_undochange() {
  let (backend, _directory) = restore_fixture(restore_success_replies());
  std::fs::write(
    backend.restore_paths.root.join("etc/app.conf"),
    "snapshot version\n",
  )
  .unwrap();
  let review = backend
    .restore_preview("system", 1, 2, "/etc/app.conf")
    .unwrap();
  assert_eq!(review.source_snapshot, 2);
  assert!(review.destination.is_some());
  assert!(matches!(review.diff, FileDiff::Text(_)));
  let result = backend
    .restore_file("system", 1, 2, "/etc/app.conf", review.destination.as_ref())
    .unwrap();
  assert_eq!(result.safety_snapshot, 9);
  assert_eq!(result.source_snapshot, 2);
  let requests = backend.runner.requests.lock().unwrap();
  let undo = requests
    .iter()
    .find(|request| {
      request
        .args
        .get(2)
        .is_some_and(|argument| argument == "undochange")
    })
    .unwrap();
  assert_eq!(
    undo.args,
    ["-c", "system", "undochange", "2..9", "/etc/app.conf"]
  );
  assert!(
    requests
      .iter()
      .any(|request| request.args.contains(&"--print-number".into()))
  );
}

#[test]
fn selective_restore_can_create_a_missing_regular_file_from_the_snapshot() {
  let (backend, _directory) = restore_fixture(restore_preview_replies());
  let review = backend
    .restore_preview("system", 1, 2, "/etc/app.conf")
    .unwrap();
  assert!(review.destination.is_none());
}

#[test]
fn selective_restore_rejects_unlisted_paths_and_internal_snapshot_paths() {
  assert!(validate_restore_path("").is_err());
  assert!(validate_restore_path("relative/file").is_err());
  assert!(validate_restore_path("/etc/../passwd").is_err());
  assert!(validate_restore_path("/etc/bad\0name").is_err());
  assert!(validate_restore_path("/.snapshots/2/snapshot/etc/passwd").is_err());
  assert!(
    validate_helper(
      "restore-file",
      &[
        "system".into(),
        "1".into(),
        "2".into(),
        "/etc/passwd".into(),
        "{}".into()
      ]
    )
    .is_err()
  );
  assert!(validate_helper("undochange", &["system".into(), "1".into()]).is_err());
}

#[test]
fn selective_restore_rejects_symlinks_hardlinks_and_mount_boundaries() {
  let (backend, directory) = restore_fixture(restore_preview_replies());
  let source_parent = backend.restore_paths.snapshots.join("2/snapshot/etc");
  std::fs::remove_file(source_parent.join("app.conf")).unwrap();
  std::os::unix::fs::symlink("/etc/passwd", source_parent.join("app.conf")).unwrap();
  assert!(
    backend
      .restore_preview("system", 1, 2, "/etc/app.conf")
      .is_err()
  );

  let target = backend.restore_paths.root.join("etc/app.conf");
  std::fs::write(&target, "target\n").unwrap();
  let alias = directory.path().join("hardlink");
  std::fs::hard_link(&target, alias).unwrap();
  assert!(
    secure_destination(
      &target,
      &backend.restore_paths.root,
      std::fs::metadata(&backend.restore_paths.root)
        .unwrap()
        .uid()
    )
    .is_err()
  );

  let mountinfo = directory.path().join("mountinfo");
  let mountpoint = backend.restore_paths.root.join("etc");
  std::fs::write(
    &mountinfo,
    format!(
      "1 0 0:1 / / rw - btrfs /dev/test rw\n2 1 0:2 / {} rw - ext4 /dev/other rw\n",
      mountpoint.display()
    ),
  )
  .unwrap();
  assert!(ensure_not_mountpoint(&target, &mountinfo).is_err());
}

#[test]
fn restore_confirmation_token_detects_destination_changes_before_snapshot_creation() {
  let mut replies = restore_preview_replies();
  replies.extend(restore_preview_replies());
  let (backend, _directory) = restore_fixture(replies);
  let target = backend.restore_paths.root.join("etc/app.conf");
  std::fs::write(&target, "current version\n").unwrap();
  let review = backend
    .restore_preview("system", 1, 2, "/etc/app.conf")
    .unwrap();
  std::fs::write(&target, "changed after review\n").unwrap();
  let error = backend
    .restore_file("system", 1, 2, "/etc/app.conf", review.destination.as_ref())
    .unwrap_err();
  assert!(error.detail.contains("changed since review"));
  assert!(
    !backend
      .runner
      .requests
      .lock()
      .unwrap()
      .iter()
      .any(|request| request.args.contains(&"create".into()))
  );
}

#[test]
fn restore_failure_reports_the_safety_snapshot_id() {
  let mut replies = restore_success_replies();
  let undo_result = replies.len() - 2;
  replies[undo_result] = Ok(ProcessOutput {
    stdout: vec![],
    stderr: b"undochange failed".to_vec(),
    status: Some(1),
    timed_out: false,
    cancelled: false,
  });
  let (backend, _directory) = restore_fixture(replies);
  std::fs::write(
    backend.restore_paths.root.join("etc/app.conf"),
    "snapshot version\n",
  )
  .unwrap();
  let review = backend
    .restore_preview("system", 1, 2, "/etc/app.conf")
    .unwrap();
  let error = backend
    .restore_file("system", 1, 2, "/etc/app.conf", review.destination.as_ref())
    .unwrap_err();
  assert!(error.detail.contains("safety snapshot #9 remains"));
}

#[test]
fn selective_restore_lock_prevents_concurrent_mutations_and_releases_on_drop() {
  let directory = tempfile::tempdir().unwrap();
  let root = directory.path().join("root");
  std::fs::create_dir_all(&root).unwrap();
  let lock_path = root.join("restore.lock");
  let owner = std::fs::metadata(&root).unwrap().uid();

  let first =
    RestoreLock::acquire(&lock_path, &root, owner, LockPurpose::SelectiveRestore).unwrap();
  assert!(RestoreLock::acquire(&lock_path, &root, owner, LockPurpose::SelectiveRestore).is_err());
  drop(first);
  assert!(RestoreLock::acquire(&lock_path, &root, owner, LockPurpose::SelectiveRestore).is_ok());
}

#[test]
fn restore_rejects_content_mismatch_despite_empty_status() {
  let (backend, _directory) = restore_fixture(restore_success_replies());
  std::fs::write(
    backend.restore_paths.root.join("etc/app.conf"),
    "wrong content\n",
  )
  .unwrap();
  let review = backend
    .restore_preview("system", 1, 2, "/etc/app.conf")
    .unwrap();
  let error = backend
    .restore_file("system", 1, 2, "/etc/app.conf", review.destination.as_ref())
    .unwrap_err();
  assert!(error.detail.contains("safety snapshot #9 remains"));
}

#[test]
fn restore_does_not_request_root_status_even_when_its_output_would_overflow() {
  let mut replies = restore_success_replies();
  *replies.last_mut().unwrap() = output(&"c..... /etc/other\n".repeat(70000));
  let (backend, _directory) = restore_fixture(replies);
  std::fs::write(
    backend.restore_paths.root.join("etc/app.conf"),
    "snapshot version\n",
  )
  .unwrap();
  let review = backend
    .restore_preview("system", 1, 2, "/etc/app.conf")
    .unwrap();
  backend
    .restore_file("system", 1, 2, "/etc/app.conf", review.destination.as_ref())
    .unwrap();
  assert!(
    !backend
      .runner
      .requests
      .lock()
      .unwrap()
      .iter()
      .any(|request| request.args == ["-c", "system", "status", "2..0"])
  );
}

#[test]
fn restored_files_compare_absence_and_permissions() {
  use std::os::unix::fs::PermissionsExt;
  let (backend, _directory) = restore_fixture(vec![]);
  let root = &backend.restore_paths.root;
  let owner = std::fs::metadata(root).unwrap().uid();
  let source = root.join("etc/source");
  let target = root.join("etc/target");
  assert!(restored_files_match(&source, &target, root, owner).unwrap());
  std::fs::write(&source, "same").unwrap();
  assert!(!restored_files_match(&source, &target, root, owner).unwrap());
  std::fs::write(&target, "same").unwrap();
  std::fs::set_permissions(&source, std::fs::Permissions::from_mode(0o600)).unwrap();
  std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o644)).unwrap();
  assert!(!restored_files_match(&source, &target, root, owner).unwrap());
}

#[test]
fn large_comparison_has_bounded_compact_response() {
  let status: String = (0..50000)
    .map(|index| format!("c..... /etc/file-{index}\n"))
    .collect();
  let mut replies = ready(vec![row(1, false, false), row(2, false, false)]);
  replies.push(output(&status));
  let comparison = backend(replies).compare("system", 1, 2).unwrap();
  assert_eq!(comparison.changes.len(), 10000);
  assert!(comparison.truncated);
  assert_eq!(comparison.total_seen, 50000);
  let encoded = serde_json::to_vec(&comparison).unwrap();
  assert_eq!(
    serde_json::from_slice::<Comparison>(&encoded).unwrap(),
    comparison
  );
  assert!(serde_json::to_vec(&comparison).unwrap().len() < 1024 * 1024);
}

#[test]
fn comparison_output_overflow_is_a_typed_error() {
  let mut replies = ready(vec![row(1, false, false), row(2, false, false)]);
  replies.push(Err(ProcessError::OutputLimit));
  assert_eq!(
    backend(replies).compare("system", 1, 2).unwrap_err().kind,
    ErrorKind::OutputLimit
  );
}

#[test]
fn restore_preview_rejects_distinct_filesystem_uuids_even_with_same_device() {
  let (mut backend, _directory) = restore_fixture(restore_preview_replies());
  backend.restore_paths.filesystem_uuid = |path| {
    Ok(
      if path
        .components()
        .any(|part| part.as_os_str() == ".snapshots")
      {
        [2; 16]
      } else {
        [1; 16]
      },
    )
  };
  assert!(
    backend
      .restore_preview("system", 1, 2, "/etc/app.conf")
      .is_err()
  );
}

#[test]
fn restore_filesystem_identity_ignores_subvolume_device_numbers() {
  // Fake subvolumes have different device identities but the same filesystem UUID.
  assert!(
    verify_restore_filesystem(Path::new("/"), Path::new("/.snapshots/2/snapshot"), |_| Ok(
      [1; 16]
    ))
    .is_ok()
  );
}

#[test]
fn restore_preview_fails_closed_when_filesystem_probe_fails() {
  let (mut backend, _directory) = restore_fixture(restore_preview_replies());
  backend.restore_paths.filesystem_uuid = |_| Err("not Btrfs".into());
  assert!(
    backend
      .restore_preview("system", 1, 2, "/etc/app.conf")
      .is_err()
  );
}

#[test]
#[ignore = "requires disposable Btrfs mounts prepared in /audit-root and /audit-other"]
fn real_btrfs_subvolume_identity() {
  let root = Path::new("/audit-root");
  let source = root.join("snapshot/file");
  assert_ne!(
    fs::metadata(root).unwrap().dev(),
    fs::metadata(&source).unwrap().dev()
  );
  verify_restore_filesystem(root, &source, btrfs_filesystem_uuid).unwrap();
  assert!(
    verify_restore_filesystem(root, Path::new("/audit-other/file"), btrfs_filesystem_uuid).is_err()
  );
  assert!(btrfs_filesystem_uuid(Path::new("/proc")).is_err());
  let target = root.join("origin/file");
  assert!(restored_files_match(&source, &target, root, 0).unwrap());
  fs::write(&target, "changed\n").unwrap();
  assert!(!restored_files_match(&source, &target, root, 0).unwrap());
  fs::write(&target, fs::read(&source).unwrap()).unwrap();
  assert!(restored_files_match(&source, &target, root, 0).unwrap());
}

#[test]
fn automatic_pre_upgrade_snapshot_uses_number_cleanup() {
  let mut replies = ready(vec![]);
  replies.extend(ready(vec![]));
  replies.push(output("42\n"));
  let backend = backend(replies);
  assert_eq!(backend.create_pre_upgrade().unwrap(), 42);
  let requests = backend.runner.requests.lock().unwrap();
  assert!(
    requests
      .last()
      .unwrap()
      .args
      .windows(2)
      .any(|pair| pair == ["--cleanup-algorithm", "number"])
  );
}

#[test]
fn restore_safety_snapshot_uses_number_cleanup() {
  let (backend, _directory) = restore_fixture(restore_success_replies());
  fs::write(
    backend.restore_paths.root.join("etc/app.conf"),
    "snapshot version\n",
  )
  .unwrap();
  let review = backend
    .restore_preview("system", 1, 2, "/etc/app.conf")
    .unwrap();
  backend
    .restore_file("system", 1, 2, "/etc/app.conf", review.destination.as_ref())
    .unwrap();
  let requests = backend.runner.requests.lock().unwrap();
  let creation = requests
    .iter()
    .find(|request| request.args.get(2).is_some_and(|arg| arg == "create"))
    .unwrap();
  assert!(
    creation
      .args
      .windows(2)
      .any(|pair| pair == ["--cleanup-algorithm", "number"])
  );
}

#[test]
fn comparison_keeps_valid_paths_around_unsupported_names() {
  let mut replies = ready(vec![row(1, false, false), row(2, false, false)]);
  let bytes = b"c..... /etc/good\nc..... /bad/../name\nc..... /bad\xffname\nc..... /bad\tname\nc..... /etc/also-good\n";
  replies.push(Ok(ProcessOutput {
    stdout: bytes.to_vec(),
    stderr: vec![],
    status: Some(0),
    timed_out: false,
    cancelled: false,
  }));
  let comparison = backend(replies).compare("system", 1, 2).unwrap();
  assert_eq!(comparison.unsupported, 3);
  assert_eq!(comparison.total_seen, 5);
  assert_eq!(
    comparison
      .changes
      .iter()
      .map(|change| change.path.as_str())
      .collect::<Vec<_>>(),
    ["/etc/good", "/etc/also-good"]
  );
}

#[test]
fn newline_path_prefix_is_not_exposed_as_restorable() {
  let (changes, unsupported) = parse_supported_status(
    b"c..... /etc/good\nc..... /bad-prefix\ncontinuation\nc..... /etc/also-good\n",
  )
  .unwrap();
  assert_eq!(unsupported, 1);
  assert_eq!(
    changes
      .iter()
      .map(|change| change.path.as_str())
      .collect::<Vec<_>>(),
    ["/etc/good", "/etc/also-good"]
  );
}

#[test]
fn diff_revalidates_path_without_building_truncated_comparison() {
  let mut replies = ready(vec![row(1, false, false), row(2, false, false)]);
  let status: String = (0..10001)
    .map(|index| format!("c..... /etc/file-{index}\n"))
    .collect();
  replies.push(output(&status));
  replies.push(output("+selected contents\n"));
  let backend = backend(replies);
  assert!(matches!(
    backend.diff("system", 1, 2, "/etc/file-10000").unwrap(),
    FileDiff::Text(_)
  ));
  let requests = backend.runner.requests.lock().unwrap();
  assert_eq!(
    requests
      .iter()
      .filter(|request| request.args.get(2).is_some_and(|arg| arg == "status"))
      .count(),
    1
  );
  let diff = requests.last().unwrap();
  assert_eq!(
    diff.args,
    ["-c", "system", "diff", "1..2", "/etc/file-10000"]
  );
}

#[test]
fn restore_preview_inspects_and_compares_once() {
  let (backend, _directory) = restore_fixture(restore_preview_replies());
  backend
    .restore_preview("system", 1, 2, "/etc/app.conf")
    .unwrap();
  let requests = backend.runner.requests.lock().unwrap();
  assert_eq!(
    requests
      .iter()
      .filter(|request| request.program == FINDMNT)
      .count(),
    1
  );
  for command in ["status", "diff"] {
    assert_eq!(
      requests
        .iter()
        .filter(|request| request.args.get(2).is_some_and(|arg| arg == command))
        .count(),
      1
    );
  }
  assert_eq!(
    requests.last().unwrap().args,
    ["-c", "system", "diff", "2..0", "/etc/app.conf"]
  );
}

#[test]
fn mount_path_octal_escapes_preserve_utf8_bytes() {
  assert_eq!(
    decode_mount_path(br"/m\303\263dulo\040de\040dados"),
    Some(PathBuf::from("/módulo de dados"))
  );
}

#[test]
fn escaped_unicode_mountpoint_blocks_restore_boundary() {
  let (backend, _directory) = restore_fixture(vec![]);
  let escaped = format!("{}/m\\303\\263dulo", backend.restore_paths.root.display());
  fs::write(
    &backend.restore_paths.mountinfo,
    format!("1 0 0:1 / {escaped} rw - btrfs /dev/test rw\n"),
  )
  .unwrap();
  assert!(
    ensure_not_mountpoint(
      &backend.restore_paths.root.join("módulo/file"),
      &backend.restore_paths.mountinfo
    )
    .is_err()
  );
}

#[test]
fn mount_path_preserves_non_utf8_and_rejects_bad_escapes() {
  use std::os::unix::ffi::OsStrExt;
  assert_eq!(
    decode_mount_path(br"/raw\377")
      .unwrap()
      .as_os_str()
      .as_bytes(),
    b"/raw\xff"
  );
  assert_eq!(
    decode_mount_path(b"/raw\xff")
      .unwrap()
      .as_os_str()
      .as_bytes(),
    b"/raw\xff"
  );
  for path in [
    br"/bad\12".as_slice(),
    br"/bad\888",
    br"/bad\777",
    br"/bad\000",
  ] {
    assert!(decode_mount_path(path).is_none());
  }
}

#[test]
fn package_lock_contention_is_not_reported_as_another_restore() {
  let (backend, _directory) = restore_fixture(vec![]);
  let root = &backend.restore_paths.root;
  let owner = fs::metadata(root).unwrap().uid();
  let _held = RestoreLock::acquire(
    &backend.restore_paths.package_lock,
    root,
    owner,
    LockPurpose::PackageUpdate,
  )
  .unwrap();
  let error = backend
    .restore_file("system", 1, 2, "/etc/app.conf", None)
    .unwrap_err();
  assert_eq!(error.kind, ErrorKind::PackageUpdateBusy);
  assert!(error.detail.contains("package"), "{error}");
  assert!(backend.runner.requests.lock().unwrap().is_empty());
}

#[test]
fn restore_lock_contention_has_its_own_error_and_releases_package_lock() {
  let (backend, _directory) = restore_fixture(vec![]);
  let root = &backend.restore_paths.root;
  let owner = fs::metadata(root).unwrap().uid();
  let _held = RestoreLock::acquire(
    &backend.restore_paths.lock,
    root,
    owner,
    LockPurpose::SelectiveRestore,
  )
  .unwrap();
  let error = backend
    .restore_file("system", 1, 2, "/etc/app.conf", None)
    .unwrap_err();
  assert_eq!(error.kind, ErrorKind::RestoreBusy);
  assert!(
    RestoreLock::acquire(
      &backend.restore_paths.package_lock,
      root,
      owner,
      LockPurpose::PackageUpdate
    )
    .is_ok()
  );
  assert!(backend.runner.requests.lock().unwrap().is_empty());
}

#[test]
fn retention_multiple_settings_are_separate_snapper_arguments() {
  let mut replies = ready(vec![]);
  replies.push(output(""));
  replies.extend(ready(vec![]));
  replies.push(output(r#"{"NUMBER_LIMIT":"20","NUMBER_CLEANUP":"yes"}"#));
  let backend = backend(replies);
  backend
    .set_retention(
      "system",
      &[
        ("NUMBER_LIMIT".into(), "20".into()),
        ("NUMBER_CLEANUP".into(), "yes".into()),
      ],
    )
    .unwrap();
  let requests = backend.runner.requests.lock().unwrap();
  let set = requests
    .iter()
    .find(|request| request.args.get(2).is_some_and(|arg| arg == "set-config"))
    .unwrap();
  assert_eq!(
    set.args,
    [
      "-c",
      "system",
      "set-config",
      "NUMBER_LIMIT=20",
      "NUMBER_CLEANUP=yes"
    ]
  );
}

#[test]
fn shared_retention_parser_validates_every_pair() {
  let changes = parse_retention_configdata("NUMBER_LIMIT=20 NUMBER_CLEANUP=yes").unwrap();
  assert_eq!(
    build_retention_configdata(&changes).unwrap(),
    "NUMBER_LIMIT=20 NUMBER_CLEANUP=yes"
  );
  for invalid in [
    "",
    "NUMBER_LIMIT=20 NUMBER_LIMIT=30",
    "NUMBER_LIMIT=20 SUBVOLUME=/",
    "NUMBER_CLEANUP=maybe",
    "NUMBER_LIMIT=20  NUMBER_CLEANUP=yes",
  ] {
    assert!(parse_retention_configdata(invalid).is_err(), "{invalid}");
  }
}

#[test]
fn pre_upgrade_with_multiple_root_configs_uses_a_stable_choice() {
  for (names, expected) in [(["zeta", "root"], "root"), (["zeta", "alpha"], "alpha")] {
    let inspection = || {
      vec![
        output(&root("btrfs")),
        output("--jsonout"),
        output(
          &json!({"configs": names.map(|name| json!({"config":name,"subvolume":"/"}))}).to_string(),
        ),
        output(r#"{"SUBVOLUME":"/","FSTYPE":"btrfs"}"#),
        output(r#"{"SUBVOLUME":"/","FSTYPE":"btrfs"}"#),
      ]
    };
    let mut replies = inspection();
    replies.extend(inspection());
    replies.push(output(&json!({expected: []}).to_string()));
    replies.push(output("42\n"));
    let backend = backend(replies);
    assert_eq!(backend.create_pre_upgrade().unwrap(), 42);
    let requests = backend.runner.requests.lock().unwrap();
    let create = requests.last().unwrap();
    assert!(create.args.windows(2).any(|pair| pair == ["-c", expected]));
    assert!(create.args.iter().any(|arg| arg == "create"));
    assert!(
      create
        .args
        .windows(2)
        .any(|pair| pair == ["--cleanup-algorithm", "number"])
    );
  }
}
