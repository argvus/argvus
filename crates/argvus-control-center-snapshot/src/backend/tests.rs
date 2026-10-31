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
  });
  assert_eq!(
    backend(vec![failure]).inspect(None).unwrap_err().kind,
    ErrorKind::Detection
  );
}
