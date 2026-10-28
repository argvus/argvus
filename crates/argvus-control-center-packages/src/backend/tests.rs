use super::*;
use argvus_control_center_core::process::{ProcessOutput, ProcessRequest};
use std::sync::Mutex;

#[derive(Default)]
struct InstalledRunner(Mutex<Vec<ProcessRequest>>);
impl ProcessRunner for InstalledRunner {
  fn run(&self, request: &ProcessRequest) -> Result<ProcessOutput, ProcessError> {
    self.0.lock().unwrap().push(request.clone());
    let stdout = if request.args == ["-Q"] {
      "linux 6.17.1.arch1-1\nargvus-control-center 0.1.0-1\n"
    } else if request.args == ["-Qmq"] {
      "argvus-control-center\n"
    } else {
      ""
    };
    Ok(ProcessOutput {
      stdout: stdout.as_bytes().to_vec(),
      stderr: Vec::new(),
      status: Some(0),
      timed_out: false,
    })
  }
}
#[test]
fn dashboard_reports_pacman_unavailable_instead_of_silent_zeros() {
  let backend = PackageBackend::new(InstalledRunner::default(), Capabilities::default());
  let dashboard = backend.dashboard();
  assert_eq!(dashboard.installed_count, 0);
  assert!(
    dashboard
      .error
      .as_deref()
      .unwrap_or_default()
      .contains("pacman"),
    "{}",
    dashboard.error.as_deref().unwrap_or_default()
  );
}
#[test]
fn parses_installed() {
  let x = parse_installed("linux 6.1\nfoo 1-2\nrust 1:1.95.0-1");
  assert_eq!(x[0].name, "linux");
  assert_eq!(x[2].version, "1:1.95.0-1");
}
#[test]
fn dashboard_reports_query_failures_instead_of_silent_zeros() {
  struct FailingRunner;
  impl ProcessRunner for FailingRunner {
    fn run(&self, _: &ProcessRequest) -> Result<ProcessOutput, ProcessError> {
      Err(ProcessError::EmptyProgram)
    }
  }
  let backend = PackageBackend::new(
    FailingRunner,
    Capabilities {
      has_pacman: true,
      ..Default::default()
    },
  );
  let dashboard = backend.dashboard();
  assert!(dashboard.error.is_some());
}
#[test]
fn installed_uses_two_compatible_queries_without_n_plus_one() {
  let capabilities = Capabilities {
    has_pacman: true,
    ..Default::default()
  };
  let backend = PackageBackend::new(InstalledRunner::default(), capabilities);
  let packages = backend.installed().unwrap();
  assert_eq!(packages.len(), 2);
  assert!(packages[1].foreign);
  let requests = backend.runner.0.lock().unwrap();
  assert_eq!(requests.len(), 2);
  let mut args = requests
    .iter()
    .map(|request| request.args.clone())
    .collect::<Vec<_>>();
  args.sort();
  assert_eq!(args, [vec![String::from("-Q")], vec![String::from("-Qmq")]]);
}
#[test]
fn parses_updates() {
  let x = parse_updates("linux 6.1 -> 6.2");
  assert_eq!(x[0].available, "6.2");
}

/// Answers every request with the same output and records the requests.
/// `failing` splits pacman's refusal as pacman does: the `error:` line on
/// stderr and the details on stdout.
struct ScriptedRunner {
  stdout: &'static str,
  stderr: &'static str,
  status: i32,
  requests: Mutex<Vec<ProcessRequest>>,
}
impl ScriptedRunner {
  fn new(stdout: &'static str, status: i32) -> Self {
    Self {
      stdout,
      stderr: "",
      status,
      requests: Mutex::default(),
    }
  }
  /// Fails with status 1, `stderr` and the details in `stdout`, as pacman
  /// does when it refuses a transaction.
  fn failing(stderr: &'static str, stdout: &'static str) -> Self {
    Self {
      stderr,
      ..Self::new(stdout, 1)
    }
  }
}
impl ProcessRunner for ScriptedRunner {
  fn run(&self, request: &ProcessRequest) -> Result<ProcessOutput, ProcessError> {
    self.requests.lock().unwrap().push(request.clone());
    Ok(ProcessOutput {
      stdout: self.stdout.as_bytes().to_vec(),
      stderr: self.stderr.as_bytes().to_vec(),
      status: Some(self.status),
      timed_out: false,
    })
  }
}
fn pacman_capabilities() -> Capabilities {
  Capabilities {
    has_pacman: true,
    ..Default::default()
  }
}

#[test]
fn foreign_lists_the_packages_outside_the_repositories() {
  let backend = PackageBackend::new(
    ScriptedRunner::new("yay-bin 12.4.2-1\n", 0),
    pacman_capabilities(),
  );
  let packages = backend.foreign().unwrap();
  assert_eq!(packages.len(), 1);
  assert!(packages[0].foreign && packages[0].installed);
  assert_eq!(backend.runner.requests.lock().unwrap()[0].args, ["-Qm"]);
  let none = PackageBackend::new(ScriptedRunner::new("", 1), pacman_capabilities());
  assert!(none.foreign().unwrap().is_empty());
}

/// pacman's stderr when a removal would break installed packages.
const DEPENDENCIES_ERROR: &str =
  "error: failed to prepare transaction (could not satisfy dependencies)\n";
/// The stdout of the same refusal, which names the dependents.
const BREAKS_DEPENDENCIES: &str = ":: removing 7zip breaks dependency '7zip' required by julia
:: removing 7zip breaks dependency '7zip' required by file-roller
:: removing p7zip breaks dependency 'p7zip' required by julia
";

#[test]
fn breaking_dependents_are_listed_once_and_in_order() {
  let targets = ["7zip".to_owned(), "p7zip".to_owned()];
  assert_eq!(
    parse_breaking_dependents(BREAKS_DEPENDENCIES, &targets),
    ["julia", "file-roller"]
  );
  let own = ":: removing a breaks dependency 'a' required by b\n:: removing b breaks dependency 'b' required by a";
  assert_eq!(
    parse_breaking_dependents(own, &["a".into(), "b".into()]),
    Vec::<String>::new(),
    "targets removed together do not block each other"
  );
  assert!(parse_breaking_dependents("error: target not found: zz", &[]).is_empty());
}

#[test]
fn removal_plans_report_the_dependents_and_plan_the_cascade() {
  let backend = PackageBackend::new(
    ScriptedRunner::failing(DEPENDENCIES_ERROR, BREAKS_DEPENDENCIES),
    pacman_capabilities(),
  );
  let targets = ["7zip".to_owned()];
  match backend.plan_remove(&targets) {
    Err(PackageError::BreaksDependencies(dependents)) => {
      assert_eq!(dependents, ["julia", "file-roller"]);
    }
    other => panic!("unexpected plan: {other:?}"),
  }
  let request = backend.runner.requests.lock().unwrap()[0].clone();
  assert_eq!(
    request.args,
    ["-Rs", "--print", "--print-format", PLAN_FORMAT, "7zip"]
  );
  assert!(request.env.contains(&("LC_ALL".into(), "C".into())));

  let other = PackageBackend::new(
    ScriptedRunner::failing("error: target not found: zz\n", ""),
    pacman_capabilities(),
  );
  assert!(matches!(
    other.plan_remove(&["zz".into()]),
    Err(PackageError::Command(_))
  ));

  let cascade = PackageBackend::new(
    ScriptedRunner::new("julia\t2:1.13.2-1\t598270576\n7zip\t26.05-1\t6783359\n", 0),
    pacman_capabilities(),
  );
  let plan = cascade.plan_remove_cascade(&targets).unwrap();
  assert_eq!(
    plan
      .remove
      .iter()
      .map(|p| p.name.as_str())
      .collect::<Vec<_>>(),
    ["julia", "7zip"]
  );
  assert_eq!(plan.remove[0].size, Some(598_270_576));
  assert_eq!(cascade.runner.requests.lock().unwrap()[0].args[0], "-Rcs");
}

#[test]
fn aur_updates_run_the_helper_and_treat_status_one_as_none() {
  let backend = PackageBackend::new(
    ScriptedRunner::new("yay-bin 12.4.1-1 -> 12.4.2-1\n", 0),
    Capabilities::default(),
  );
  let updates = backend.aur_updates("yay").unwrap();
  assert_eq!(updates[0].name, "yay-bin");
  assert_eq!(updates[0].available, "12.4.2-1");
  let request = backend.runner.requests.lock().unwrap()[0].clone();
  assert_eq!(request.program, "yay");
  assert_eq!(request.args, ["-Qua"]);
  let current = PackageBackend::new(ScriptedRunner::new("", 1), Capabilities::default());
  assert!(current.aur_updates("paru").unwrap().is_empty());
  assert!(current.aur_updates("sh").is_err());
}
#[test]
fn parses_real_pacman_search_shape() {
  let packages = parse_search(
    "extra/firefox 155.0.1-1 [installed]\n    Fast, Private & Safe Web Browser\nextra/firefox-developer-edition 156.0b1-1\n    Developer Edition of the popular browser\n",
  );
  assert_eq!(packages.len(), 2);
  assert_eq!(packages[0].name, "firefox");
  assert_eq!(packages[0].repository.as_deref(), Some("extra"));
  assert!(packages[0].installed);
  assert_eq!(packages[0].description, "Fast, Private & Safe Web Browser");
}
#[test]
fn parses_localized_pacman_details_with_variable_spacing() {
  let details = parse_details(
    "Nome                 : pacman\nVersão               : 7.1.0-1\nDescrição            : Package manager\nArquitetura           : x86_64\n",
  )
  .expect("metadata should be parsed");
  assert_eq!(details.package.name, "pacman");
  assert_eq!(details.package.version, "7.1.0-1");
  assert_eq!(details.architecture.as_deref(), Some("x86_64"));
}
#[test]
fn detects_installed_via_localized_normalized_detail_field() {
  let installed = parse_details(
    "Nome                 : pacman\nVersão               : 7.1.0-1\nRepositório           : core\nData de instalação    : ter 01 jan 2020 00:00:00\n",
  )
  .expect("metadata should be parsed");
  assert!(installed.package.installed);
  let not_installed = parse_details(
    "Nome                 : pacman\nVersão               : 7.1.0-1\nRepositório           : core\n",
  )
  .expect("metadata should be parsed");
  assert!(!not_installed.package.installed);
}
#[test]
fn rejects_shell_input() {
  assert!(validate_package_name("foo;bar").is_err());
}
#[test]
fn history_is_structured() {
  let x = parse_history("[2026-09-10T10:00] [ALPM] upgraded linux (1 -> 2)");
  assert_eq!(x[0].action, "ALPM");
  assert_eq!(x[0].old_version.as_deref(), Some("1"));
  assert_eq!(x[0].new_version.as_deref(), Some("2"));
}
#[test]
fn mirror_parser_keeps_order_and_enabled_state() {
  let mirrors = parse_mirrors(
    "#Server = https://old.example/$repo/os/$arch\nServer = https://new.example/$repo/os/$arch\n",
  );
  assert_eq!(mirrors.len(), 2);
  assert!(!mirrors[0].enabled);
  assert_eq!(mirrors[1].protocol, "https");
  assert_eq!(mirrors[1].order, 2);
}
#[test]
fn parses_real_reflector_country_table_with_multi_word_names() {
  let countries = parse_reflector_countries(
    "Country              Code Count\n-------------------- ---- -----\nAlbania                AL     1\nHong Kong              HK    11\nNew Zealand            NZ     8\nUnited States          US   196\n",
  );
  assert_eq!(
    countries,
    ["Albania", "Hong Kong", "New Zealand", "United States"]
  );
}
#[test]
fn rejects_code_and_header_lines_from_country_table() {
  assert!(parse_reflector_countries("Country  Code  Count\n---- ---- -----\n").is_empty());
  assert!(parse_reflector_countries("Albania AL\n").is_empty());
}
#[test]
fn package_names_allow_arch_names_but_reject_shell_syntax() {
  assert!(validate_package_name("foo-bar@1").is_ok());
  assert!(validate_package_name("foo/bar").is_err());
  assert!(validate_search("mesa graphics").is_ok());
  assert!(validate_search("foo\nbar").is_err());
}
#[test]
fn transaction_plan_preserves_targets_and_sizes_without_running_a_transaction() {
  let plan = parse_transaction_plan("foo-bar\t2.0-1\t1024\nlibfoo\t1.0-2\t2048\n", true);
  assert_eq!(plan.install.len(), 2);
  assert_eq!(plan.download_bytes, 3072);
  assert!(plan.requires_full_upgrade);
  let removal = parse_transaction_plan("foo-bar\t2.0-1\t0\n", false);
  assert_eq!(removal.remove[0].name, "foo-bar");
  assert!(!removal.requires_full_upgrade);
}

#[test]
fn classifies_conflicts_and_signature_failures_without_bypassing_them() {
  assert!(matches!(
    classify_pacman_failure("error: conflicting files: /usr/bin/foo exists in filesystem"),
    Some(TransactionDecision::Conflict {
      kind: ConflictKind::File,
      ..
    })
  ));
  assert!(matches!(
    classify_pacman_failure("error: signature from key is unknown trust"),
    Some(TransactionDecision::SignatureError {
      kind: SignatureErrorKind::UnknownKey,
      ..
    })
  ));
}

#[test]
fn reflector_options_are_typed_and_shell_safe() {
  let options = ReflectorOptions {
    countries: vec!["Brazil".into(), "United States".into()],
    protocols: vec!["https".into()],
    age_hours: 12,
    count: 10,
    sort: "rate".into(),
  };
  let args = reflector_args(&options).unwrap();
  assert!(
    args
      .windows(2)
      .any(|pair| pair == ["--country", "Brazil,United States"])
  );
  assert!(!args.iter().any(|arg| arg.contains(';')));
  assert!(
    validate_reflector_options(&ReflectorOptions {
      sort: "arbitrary-command".into(),
      ..options
    })
    .is_err()
  );
}

#[test]
fn cache_preview_matches_policy_and_counts_real_bytes() {
  let entries = vec![
    CachePackage {
      name: "foo".into(),
      version: "3".into(),
      bytes: 30,
      installed: true,
      ..Default::default()
    },
    CachePackage {
      name: "foo".into(),
      version: "2".into(),
      bytes: 20,
      ..Default::default()
    },
    CachePackage {
      name: "foo".into(),
      version: "1".into(),
      bytes: 10,
      ..Default::default()
    },
  ];
  let preview = cache_preview(entries, CachePolicy::KeepOne);
  assert_eq!(preview.candidates.len(), 2);
  assert_eq!(preview.bytes, 30);
}

#[test]
fn cached_metadata_does_not_depend_on_hyphen_splitting() {
  assert_eq!(
    parse_cached_metadata("foo-bar-baz\t2:1.4-3\tx86_64"),
    Some(("foo-bar-baz".into(), "2:1.4-3".into(), "x86_64".into()))
  );
  assert!(parse_cached_metadata("foo-bar\t\tx86_64").is_none());
}
