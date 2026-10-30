use super::*;
use serde_json::json;
use std::collections::VecDeque;
use std::sync::Mutex;

struct Fake {
  replies: Mutex<VecDeque<Result<ProcessOutput, ProcessError>>>,
  requests: Mutex<Vec<ProcessRequest>>,
}
impl ProcessRunner for Fake {
  fn run(&self, request: &ProcessRequest) -> Result<ProcessOutput, ProcessError> {
    self.requests.lock().unwrap().push(request.clone());
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
