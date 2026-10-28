use super::*;

#[test]
fn pid_must_be_a_positive_number() {
  let dir = std::env::temp_dir().join(format!("argvus-splash-pid-{}", std::process::id()));
  let _ = fs::create_dir_all(&dir);
  let path = dir.join("pid");
  for (content, expected) in [
    ("1234\n", Some(1234)),
    ("12x\n", None),
    ("\n", None),
    ("0\n", None),
    ("", None),
  ] {
    fs::write(&path, content).expect("pid file");
    assert_eq!(read_pid(&path), expected, "{content:?}");
  }
  let _ = fs::remove_dir_all(dir);
}

fn environment_under(runtime: &Path) -> SplashEnvironment {
  SplashEnvironment {
    values: BTreeMap::from([("XDG_RUNTIME_DIR".to_string(), runtime.display().to_string())]),
  }
}

#[test]
fn missing_binary_is_unavailable_and_writes_nothing() {
  let runtime = std::env::temp_dir().join(format!("argvus-splash-missing-{}", std::process::id()));
  let _ = fs::create_dir_all(&runtime);
  let environment = environment_under(&runtime);
  let outcome = start_with(
    &environment,
    &runtime.join("no-such-splash"),
    &SplashRequest {
      theme: "github-light",
      colors: None,
      await_compositor: false,
      spinner_only: true,
      detached: true,
      owner: Owner::Session,
    },
  );
  assert_eq!(outcome, StartOutcome::Unavailable);
  assert!(!pid_file(&environment, Owner::Session).exists());
  let _ = fs::remove_dir_all(runtime);
}

#[test]
fn a_running_splash_is_left_alone() {
  let runtime = std::env::temp_dir().join(format!("argvus-splash-running-{}", std::process::id()));
  let _ = fs::create_dir_all(&runtime);
  let environment = environment_under(&runtime);
  // This test process is alive, so the pid file names a running splash.
  fs::write(
    pid_file(&environment, Owner::Session),
    format!("{}\n", std::process::id()),
  )
  .expect("pid file");
  let outcome = start_with(
    &environment,
    &runtime.join("no-such-splash"),
    &SplashRequest::session("github-light"),
  );
  assert_eq!(outcome, StartOutcome::AlreadyRunning);
  let _ = fs::remove_dir_all(runtime);
}

#[test]
fn session_splash_is_opaque_and_detached_while_the_overlay_is_not() {
  let ready = Path::new("/run/ready");
  let session = splash_arguments(&SplashRequest::session("github-light"), ready);
  assert!(session.contains(&OsString::from("--spinner-only")));
  assert!(session.contains(&OsString::from("--detached")));

  let overlay = SplashRequest {
    theme: "github-light",
    colors: None,
    await_compositor: false,
    spinner_only: false,
    detached: false,
    owner: Owner::Transition,
  };
  let arguments = splash_arguments(&overlay, ready);
  assert!(!arguments.contains(&OsString::from("--spinner-only")));
  assert!(!arguments.contains(&OsString::from("--detached")));
}

#[test]
fn stale_pid_file_is_removed_and_not_active() {
  let runtime = std::env::temp_dir().join(format!("argvus-splash-stale-{}", std::process::id()));
  let _ = fs::create_dir_all(&runtime);
  let environment = SplashEnvironment {
    values: BTreeMap::from([("XDG_RUNTIME_DIR".to_string(), runtime.display().to_string())]),
  };
  // A pid far above any real process: it cannot be alive.
  fs::write(pid_file(&environment, Owner::Session), "2147483646\n").expect("pid file");
  assert!(!active(&environment));
  assert!(!pid_file(&environment, Owner::Session).exists());
  let _ = fs::remove_dir_all(runtime);
}

#[test]
fn wayland_socket_must_be_a_socket() {
  let runtime = std::env::temp_dir().join(format!("argvus-splash-socket-{}", std::process::id()));
  let _ = fs::create_dir_all(&runtime);
  let environment = SplashEnvironment {
    values: BTreeMap::from([
      ("XDG_RUNTIME_DIR".to_string(), runtime.display().to_string()),
      ("WAYLAND_DISPLAY".to_string(), "wayland-test".to_string()),
    ]),
  };
  assert!(!wayland_socket_ready(&environment), "no socket yet");
  fs::write(runtime.join("wayland-test"), "").expect("regular file");
  assert!(
    !wayland_socket_ready(&environment),
    "a regular file is not a socket"
  );
  let _ = fs::remove_dir_all(runtime);
}

/// Copies `source` to `directory/name`, so the kernel reports `name` as the
/// process name, and starts it with `arguments`.
fn spawn_named(
  directory: &Path,
  source: &str,
  name: &str,
  arguments: &[&str],
) -> std::process::Child {
  let program = directory.join(name);
  fs::copy(source, &program).expect("copy test program");
  // Another test may fork while this copy is still open for writing; the
  // child then holds the descriptor, and exec fails with ETXTBSY until it closes.
  for _ in 0..50 {
    match Command::new(&program)
      .args(arguments)
      .stdin(Stdio::null())
      .stdout(Stdio::null())
      .stderr(Stdio::null())
      .spawn()
    {
      Ok(child) => {
        wait_for_exec(child.id(), name);
        return child;
      }
      Err(error) if error.raw_os_error() == Some(libc::ETXTBSY) => {
        sleep(Duration::from_millis(20));
      }
      Err(error) => panic!("spawn test program: {error}"),
    }
  }
  panic!("spawn test program: still busy after retries");
}

/// Waits until `pid` runs a program named `name`. Identifying a splash by its
/// name is only meaningful once the exec has replaced the test-binary image.
fn wait_for_exec(pid: u32, name: &str) {
  let expected: String = name.chars().take(COMM_LENGTH).collect();
  for _ in 0..400 {
    let comm = fs::read_to_string(format!("/proc/{pid}/comm")).unwrap_or_default();
    if comm.trim_end() == expected {
      return;
    }
    sleep(Duration::from_millis(5));
  }
  panic!("child {pid} did not exec {name} in time");
}

/// Records `pid` the way the session does: the pid, then its start time, which
/// identifies the process even after an exec changes its name.
fn write_recorded_pid(environment: &SplashEnvironment, owner: Owner, pid: u32) {
  let start_time = process_start_time(pid as i32).expect("start time of the child");
  fs::write(
    pid_file(environment, owner),
    format!("{pid}\n{start_time}\n"),
  )
  .expect("pid file");
}

fn test_runtime(label: &str) -> PathBuf {
  let runtime = std::env::temp_dir().join(format!("argvus-splash-{label}-{}", std::process::id()));
  let _ = fs::remove_dir_all(&runtime);
  fs::create_dir_all(&runtime).expect("runtime dir");
  runtime
}

#[test]
fn stop_reaps_the_splash_instead_of_waiting_for_a_zombie() {
  let runtime = test_runtime("reap");
  let environment = environment_under(&runtime);
  let mut child = spawn_named(&runtime, "/bin/sleep", "splash-test", &["60"]);
  write_recorded_pid(&environment, Owner::Transition, child.id());

  let started = std::time::Instant::now();
  stop_named(&environment, Owner::Transition, "splash-test");
  // Without reaping, `process_alive` keeps seeing the zombie and the wait runs for five seconds.
  assert!(
    // The splash sleeps for 60 s, so this bound only has to rule out a
    // five-second wait; it leaves room for a loaded machine.
    started.elapsed() < Duration::from_secs(10),
    "{:?}",
    started.elapsed()
  );
  assert!(!pid_file(&environment, Owner::Transition).exists());
  assert!(
    child.try_wait().is_err(),
    "stop did not reap the splash child"
  );
  let _ = fs::remove_dir_all(runtime);
}

#[test]
fn legacy_pid_file_is_identified_by_process_name() {
  let runtime = test_runtime("legacy");
  let environment = environment_under(&runtime);
  let mut child = spawn_named(&runtime, "/bin/sleep", "splash-test", &["60"]);
  // A pid file written before the start time was recorded holds only the pid.
  fs::write(
    pid_file(&environment, Owner::Transition),
    format!("{}\n", child.id()),
  )
  .expect("pid file");

  stop_named(&environment, Owner::Transition, "splash-test");
  assert!(
    child.try_wait().is_err(),
    "stop did not reap the legacy splash child"
  );
  let _ = fs::remove_dir_all(runtime);
}

#[test]
fn stop_does_not_signal_a_pid_that_is_not_the_splash() {
  let runtime = test_runtime("identity");
  let environment = environment_under(&runtime);
  let mut other = spawn_named(&runtime, "/bin/sleep", "other-process", &["60"]);
  fs::write(
    pid_file(&environment, Owner::Transition),
    format!("{}\n", other.id()),
  )
  .expect("pid file");

  stop_named(&environment, Owner::Transition, "splash-test");
  assert!(
    process_alive(other.id() as i32),
    "a recycled pid must survive"
  );
  assert!(!pid_file(&environment, Owner::Transition).exists());
  let _ = other.kill();
  let _ = other.wait();
  let _ = fs::remove_dir_all(runtime);
}

#[test]
fn stop_escalates_to_sigkill_when_sigterm_is_ignored() {
  let runtime = test_runtime("sigkill");
  let environment = environment_under(&runtime);
  let mut child = spawn_named(
    &runtime,
    "/bin/sh",
    "splash-test",
    &["-c", "trap '' TERM; sleep 60"],
  );
  write_recorded_pid(&environment, Owner::Transition, child.id());

  stop_named(&environment, Owner::Transition, "splash-test");
  assert!(!process_alive(child.id() as i32), "SIGKILL ends the splash");
  let _ = child.kill();
  let _ = child.wait();
  let _ = fs::remove_dir_all(runtime);
}

#[test]
fn a_recorded_start_time_identifies_a_script_splash() {
  // A script runs under its interpreter, so its process name is not the binary
  // name. The recorded start time is what identifies it.
  let runtime = test_runtime("start-time");
  let environment = environment_under(&runtime);
  let splash = spawn_named(&runtime, "/bin/sleep", "interpreter-name", &["60"]);
  let start_time = process_start_time(splash.id() as i32).expect("start time");
  fs::write(
    pid_file(&environment, Owner::Transition),
    format!("{}\n{start_time}\n", splash.id()),
  )
  .expect("pid file");

  stop_named(&environment, Owner::Transition, "splash-test");
  // `stop` reaped the child, so it no longer exists as a process at all.
  assert!(
    !process_alive(splash.id() as i32),
    "the recorded splash was stopped"
  );
  // `stop` already reaped the child; this only satisfies the Child handle.
  let mut splash = splash;
  let _ = splash.wait();
  let _ = fs::remove_dir_all(runtime);
}

#[test]
fn a_recorded_start_time_that_no_longer_matches_is_left_alone() {
  let runtime = test_runtime("start-mismatch");
  let environment = environment_under(&runtime);
  let mut other = spawn_named(&runtime, "/bin/sleep", "other-process", &["60"]);
  let start_time = process_start_time(other.id() as i32).expect("start time");
  // The same pid with a different start time is a recycled pid.
  fs::write(
    pid_file(&environment, Owner::Transition),
    format!("{}\n{}\n", other.id(), start_time + 1),
  )
  .expect("pid file");

  stop_named(&environment, Owner::Transition, "splash-test");
  assert!(
    process_alive(other.id() as i32),
    "a recycled pid must survive"
  );
  let _ = other.kill();
  let _ = other.wait();
  let _ = fs::remove_dir_all(runtime);
}

#[test]
fn each_owner_stops_only_its_own_splash() {
  let runtime = test_runtime("owners");
  let environment = environment_under(&runtime);
  let mut session = spawn_named(&runtime, "/bin/sleep", "splash-test", &["60"]);
  fs::write(
    pid_file(&environment, Owner::Session),
    format!("{}\n", session.id()),
  )
  .expect("pid file");

  stop_named(&environment, Owner::Transition, "splash-test");
  assert!(
    process_alive(session.id() as i32),
    "the session splash is not the transition's"
  );
  assert!(pid_file(&environment, Owner::Session).exists());
  let _ = session.kill();
  let _ = session.wait();
  let _ = fs::remove_dir_all(runtime);
}

#[test]
fn a_transition_overlay_never_starts_over_a_session_splash() {
  let runtime = test_runtime("shared-guard");
  let environment = environment_under(&runtime);
  let mut session = spawn_named(&runtime, "/bin/sleep", "splash-test", &["60"]);
  fs::write(
    pid_file(&environment, Owner::Session),
    format!("{}\n", session.id()),
  )
  .expect("pid file");

  let outcome = start_with(
    &environment,
    &runtime.join("no-such-splash"),
    &SplashRequest {
      theme: "github-light",
      colors: None,
      await_compositor: false,
      spinner_only: false,
      detached: false,
      owner: Owner::Transition,
    },
  );
  assert_eq!(outcome, StartOutcome::AlreadyRunning);
  let _ = session.kill();
  let _ = session.wait();
  let _ = fs::remove_dir_all(runtime);
}
