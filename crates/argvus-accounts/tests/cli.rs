//! Read-only CLI integration tests.
//!
//! Deliberately avoids any command that would mutate the running user's real
//! home directory or system accounts. Mutation paths are covered hermetically
//! by `argvus-accounts-core` tests with mock backends and temp homes.

use assert_cmd::Command;
use predicates::prelude::*;

fn bin() -> Command {
  Command::cargo_bin("argvus-accounts").expect("binary builds")
}

#[test]
fn help_is_informative() {
  bin()
    .arg("--help")
    .assert()
    .success()
    .stdout(predicate::str::contains("Usage:"))
    .stdout(predicate::str::contains("avatar"))
    .stdout(predicate::str::contains("self"));
}

#[test]
fn version_matches_semver() {
  let version = env!("CARGO_PKG_VERSION");
  bin()
    .arg("--version")
    .assert()
    .success()
    .stdout(predicate::str::contains(version));
}

#[test]
fn list_succeeds_with_header() {
  bin()
    .arg("list")
    .assert()
    .success()
    .stdout(predicate::str::starts_with("USER"))
    .stdout(predicate::str::contains("NAME"));
}

#[test]
fn frontend_snapshot_is_read_only_and_structured() {
  bin()
    .arg("manage")
    .write_stdin(r#"{"action":"snapshot"}"#)
    .assert()
    .success()
    .stdout(predicate::str::contains("\"users\":"))
    .stdout(predicate::str::contains("\"groups\":"))
    .stdout(predicate::str::contains("\"actor_uid\":"));
}

#[test]
fn malformed_frontend_request_does_not_echo_secrets() {
  bin()
    .arg("manage")
    .write_stdin(r#"{"action":"password","new":"secret-marker","unexpected":true}"#)
    .assert()
    .failure()
    .stderr(predicate::str::contains("secret-marker").not());
}

#[test]
fn show_missing_user_fails_cleanly() {
  bin()
    .args(["show", "definitely_missing_user_xyz"])
    .assert()
    .failure()
    .code(1)
    .stderr(predicate::str::contains("user not found"));
}

#[test]
fn traversal_username_is_rejected() {
  bin()
    .args(["show", "../etc/passwd"])
    .assert()
    .failure()
    .code(1)
    .stderr(predicate::str::contains("invalid username"));
}

#[test]
fn empty_display_name_is_rejected_before_any_system_call() {
  // Validation happens before authorization/NSS mutation, so this is safe.
  let me = whoami();
  bin()
    .args(["name", &me, "   "])
    .assert()
    .failure()
    .code(1)
    .stderr(predicate::str::contains("invalid display name"));

  bin()
    .args(["self", "name", "has,comma"])
    .assert()
    .failure()
    .code(1)
    .stderr(predicate::str::contains("invalid display name"));
}

#[test]
fn groups_of_missing_user_fails_cleanly() {
  bin()
    .args(["groups", "definitely_missing_user_xyz"])
    .assert()
    .failure()
    .code(1)
    .stderr(predicate::str::contains("user not found"));
}

#[test]
fn contradictory_group_request_is_rejected() {
  // Unknown target would fail later; use an invalid one to prove validation
  // ordering without touching privileges at all.
  bin()
    .args(["groups", "../evil", "--add", "wheel", "--remove", "wheel"])
    .assert()
    .failure()
    .code(1);
}

#[test]
fn avatar_requires_image_or_remove_flag() {
  bin().args(["self", "avatar"]).assert().failure().code(2);
}

#[test]
fn verbose_flag_is_accepted_globally() {
  bin()
    .args(["--verbose", "list"])
    .assert()
    .success()
    .stdout(predicate::str::starts_with("USER"));
}

#[test]
fn passwd_mismatched_confirmation_fails_before_any_prompt() {
  let me = whoami();
  bin()
    .args(["passwd", &me])
    .write_stdin("old-secret-1\nnew-secret-1\ndifferent-2\n")
    .env_remove(elevation_guard())
    .assert()
    .failure()
    .code(1)
    .stderr(predicate::str::contains("do not match"));
}

#[test]
fn passwd_short_passwords_are_accepted_policy_free() {
  // Complexity policy belongs to frontends/PAM, not this CLI: a single
  // character passes local validation and the flow stops at proof of the
  // current password (wrong here), before any elevation attempt.
  let me = whoami();
  if !has_chkpwd_helper() {
    return;
  }
  bin()
    .args(["passwd", &me])
    .write_stdin("definitely-not-my-password\nx\nx\n")
    .env_remove(elevation_guard())
    .assert()
    .failure()
    .code(1)
    .stderr(predicate::str::contains("current password does not match"));
}

#[test]
fn passwd_wrong_current_password_is_denied_without_elevation() {
  // Self-service flow: the wrong proof is caught before pkexec would ever
  // run, so this test stays non-interactive and hermetic.
  if !has_chkpwd_helper() {
    return;
  }
  let me = whoami();
  bin()
    .args(["passwd", &me])
    .write_stdin("definitely-not-my-password\nanother-pass-1\nanother-pass-1\n")
    .env_remove(elevation_guard())
    .assert()
    .failure()
    .code(1)
    .stderr(predicate::str::contains("current password does not match"));
}

#[test]
fn passwd_unknown_user_fails_cleanly() {
  bin()
    .args(["passwd", "definitely_missing_user_xyz"])
    .write_stdin("old-secret-1\nnew-secret-1\nnew-secret-1\n")
    .env_remove(elevation_guard())
    .assert()
    .failure()
    .code(1)
    .stderr(predicate::str::contains("user not found"));
}

#[test]
fn passwd_rejects_passwords_given_as_arguments() {
  // The secrets must never reach argv: the old positional form is a usage error.
  let me = whoami();
  bin()
    .args([
      "passwd",
      &me,
      "old-secret-1",
      "new-secret-1",
      "new-secret-1",
    ])
    .env_remove(elevation_guard())
    .assert()
    .failure()
    .code(2);
}

#[test]
fn passwd_forwards_input_through_elevation_without_argv() {
  // A stand-in `pkexec` on PATH runs the command without authentication, so the
  // elevated child must receive the passwords through the pipe. The target is
  // another account and the process is not root, so the child refuses before
  // changing anything. Skipped as root: there the real change would run.
  let uid = std::process::Command::new("id")
    .arg("-u")
    .output()
    .expect("id runs");
  if String::from_utf8_lossy(&uid.stdout).trim() == "0" {
    return;
  }
  let fake_dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("fake-pkexec");
  std::fs::create_dir_all(&fake_dir).expect("fake pkexec dir");
  let fake = fake_dir.join("pkexec");
  std::fs::write(&fake, "#!/bin/sh\nexec \"$@\"\n").expect("fake pkexec written");
  std::fs::set_permissions(&fake, std::os::unix::fs::PermissionsExt::from_mode(0o755))
    .expect("fake pkexec executable");

  let path = format!(
    "{}:{}",
    fake_dir.display(),
    std::env::var("PATH").unwrap_or_default()
  );
  bin()
    .args(["passwd", "root"])
    .write_stdin("unused-current\nnew-secret-1\nnew-secret-1\n")
    .env_remove(elevation_guard())
    .env("PATH", path)
    .assert()
    .failure()
    .code(1)
    .stderr(predicate::str::contains(
      "requesting administrator privileges",
    ))
    .stderr(predicate::str::contains("one per line").not());
}

#[test]
fn passwd_requires_exactly_three_lines_of_input() {
  let me = whoami();
  bin()
    .args(["passwd", &me])
    .write_stdin("only-one-line\n")
    .env_remove(elevation_guard())
    .assert()
    .failure()
    .code(1)
    .stderr(predicate::str::contains("one per line"));
}

fn elevation_guard() -> &'static str {
  "ARGVUS_ACCOUNTS_ELEVATED"
}

fn has_chkpwd_helper() -> bool {
  [
    "/usr/bin/unix_chkpwd",
    "/usr/sbin/unix_chkpwd",
    "/sbin/unix_chkpwd",
  ]
  .iter()
  .any(|p| std::path::Path::new(p).exists())
}

/// Resolves the *effective* username so self-service tests always target the
/// identity the helper would verify against (unlike `$USER`, which can drift
/// under su/sudo/packaging sandboxes).
fn whoami() -> String {
  let output = std::process::Command::new("id")
    .arg("-un")
    .output()
    .expect("'id' is available on unix");
  String::from_utf8_lossy(&output.stdout).trim().to_string()
}
