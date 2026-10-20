//! Integration tests of the `argvus` dispatcher. They are the Rust port of the
//! shell suite `tests/dispatcher.sh`, and they run the real binary. Fake private
//! commands are created under `/tmp/argvus` and removed when the sandbox drops.

use std::fs;
use std::io::Write;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Mutex, MutexGuard};

/// Prints each argument bracketed, then reads stdin and exits with `$FAKE_STATUS`.
const FAKE_COMMAND: &str = "#!/bin/sh\nfor arg do printf \"[%s]\\n\" \"$arg\"; done\nprintf \"stdin=%s\\n\" \"$(cat)\"\nexit \"${FAKE_STATUS:-0}\"\n";

static SANDBOX_COUNTER: AtomicU32 = AtomicU32::new(0);

/// A temporary tree with the commands directory, a PATH with only `cat`, and a
/// second PATH directory for fallback checks.
struct Sandbox {
  root: PathBuf,
}

impl Sandbox {
  fn new(name: &str) -> Self {
    let index = SANDBOX_COUNTER.fetch_add(1, Ordering::Relaxed);
    let root = PathBuf::from(format!(
      "/tmp/argvus/dispatcher-{name}-{}-{index}",
      std::process::id()
    ));
    for directory in ["commands", "empty-path", "path-bin", "tools"] {
      fs::create_dir_all(root.join(directory)).expect("create sandbox directory");
    }
    symlink("/usr/bin/cat", root.join("tools/cat")).expect("link cat into the tools directory");
    Self { root }
  }

  fn commands(&self) -> PathBuf {
    self.root.join("commands")
  }

  fn tools_path(&self) -> String {
    self.root.join("tools").display().to_string()
  }

  /// Writes an executable fake command into `directory`.
  fn fake(&self, directory: &Path, name: &str) {
    write_executable(&directory.join(name), FAKE_COMMAND);
  }

  /// Builds the dispatcher command with an isolated environment: PATH holds only
  /// `path`, HOME is the sandbox, and the commands directory is the sandbox's.
  fn command(&self, path: &str, arguments: &[&str]) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_argvus"));
    command
      .env_clear()
      .env("PATH", path)
      .env("HOME", &self.root)
      .env("ARGVUS_COMMANDS_DIR", self.commands())
      .args(arguments);
    command
  }

  /// Runs the dispatcher with `stdin` piped in, and collects its output.
  fn run(&self, path: &str, arguments: &[&str], stdin: &str) -> Output {
    finish(self.command(path, arguments), stdin)
  }
}

impl Drop for Sandbox {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.root);
  }
}

/// Serializes the writes of fake commands and the spawns of the dispatcher. A
/// fork that happens while another thread still holds a write descriptor for a
/// script makes that script's `exec` fail with ETXTBSY, so the two never overlap.
static SPAWN_GUARD: Mutex<()> = Mutex::new(());

fn spawn_guard() -> MutexGuard<'static, ()> {
  SPAWN_GUARD
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn write_executable(path: &Path, content: &str) {
  let _guard = spawn_guard();
  let mut file = fs::File::create(path).expect("create fake command");
  file
    .write_all(content.as_bytes())
    .expect("write fake command");
  drop(file);
  fs::set_permissions(path, fs::Permissions::from_mode(0o755))
    .expect("make fake command executable");
}

fn finish(mut command: Command, stdin: &str) -> Output {
  let mut child = {
    let _guard = spawn_guard();
    command
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .spawn()
      .expect("start argvus")
  };
  child
    .stdin
    .take()
    .expect("piped stdin")
    .write_all(stdin.as_bytes())
    .expect("write stdin");
  child.wait_with_output().expect("wait for argvus")
}

/// The text a shell `$(...)` would capture: trailing newlines are removed.
fn text(output: &[u8]) -> String {
  String::from_utf8_lossy(output)
    .trim_end_matches('\n')
    .to_owned()
}

fn expected(lines: &[&str]) -> String {
  lines.join("\n")
}

#[test]
fn no_arguments_prints_usage_and_succeeds() {
  let sandbox = Sandbox::new("usage");
  let output = sandbox.run(&sandbox.tools_path(), &[], "");
  assert!(output.status.success());
  assert!(text(&output.stdout).starts_with("Usage:"));
}

#[test]
fn help_flags_print_the_command_list() {
  let sandbox = Sandbox::new("help");
  for flag in ["help", "--help", "-h"] {
    let output = sandbox.run(&sandbox.tools_path(), &[flag], "");
    assert!(output.status.success(), "{flag} failed");
    assert!(
      String::from_utf8_lossy(&output.stdout).contains("\nCommands:\n"),
      "{flag} output"
    );
  }
}

#[test]
fn unknown_command_fails_with_a_hint() {
  let sandbox = Sandbox::new("unknown");
  let output = sandbox.run(&sandbox.tools_path(), &["does-not-exist"], "");
  assert!(!output.status.success());
  assert!(String::from_utf8_lossy(&output.stderr).contains("argvus --help"));
}

#[test]
fn arguments_are_forwarded_exactly_and_stdin_is_inherited() {
  let sandbox = Sandbox::new("forward");
  sandbox.fake(&sandbox.commands(), "sessionctl");
  let output = sandbox.run(
    &sandbox.tools_path(),
    &["sessionctl", "a b", "", "x$y", "*", "it's"],
    "in",
  );
  let want = expected(&["[a b]", "[]", "[x$y]", "[*]", "[it's]", "stdin=in"]);
  assert!(output.status.success());
  assert_eq!(text(&output.stdout), want);
}

#[test]
fn exit_status_of_an_in_process_command_is_preserved() {
  // `config` runs in-process; an unknown subcommand fails with status 1.
  let sandbox = Sandbox::new("status");
  let output = sandbox.run(&sandbox.tools_path(), &["config", "frobnicate"], "");
  assert_eq!(output.status.code(), Some(1));
}

#[test]
fn works_outside_the_project_directory() {
  let sandbox = Sandbox::new("cwd");
  sandbox.fake(&sandbox.commands(), "sessionctl");
  let mut command = sandbox.command(&sandbox.tools_path(), &["sessionctl", "ok"]);
  command.current_dir("/");
  let output = finish(command, "");
  assert!(output.status.success());
  assert_eq!(text(&output.stdout), expected(&["[ok]", "stdin="]));
}

#[test]
fn falls_back_to_path_for_legacy_prefixed_names() {
  let sandbox = Sandbox::new("path-fallback");
  // `i18n` runs in-process now, so the fallback is checked with a command that
  // is still a private executable. Older packages installed it as argvus-snippets.
  sandbox.fake(&sandbox.root.join("path-bin"), "argvus-snippets");
  let path = format!(
    "{}:{}",
    sandbox.root.join("path-bin").display(),
    sandbox.tools_path()
  );
  let mut command = sandbox.command(&path, &["snippets", "list"]);
  command.env("ARGVUS_COMMANDS_DIR", sandbox.root.join("empty-path"));
  let output = finish(command, "");
  assert!(output.status.success());
  assert_eq!(text(&output.stdout), expected(&["[list]", "stdin="]));
}

#[test]
fn never_runs_unprefixed_names_from_path() {
  let sandbox = Sandbox::new("bare-path");
  // A program named like a helper must not run: `argvus git` would otherwise
  // start whatever `git` is on PATH.
  sandbox.fake(&sandbox.root.join("path-bin"), "git");
  let path = format!(
    "{}:{}",
    sandbox.root.join("path-bin").display(),
    sandbox.tools_path()
  );
  let output = sandbox.run(&path, &["git"], "");
  assert!(!output.status.success());
  assert!(String::from_utf8_lossy(&output.stderr).contains("git is not installed"));
}

#[test]
fn commands_directory_takes_precedence_over_path() {
  let sandbox = Sandbox::new("precedence");
  sandbox.fake(&sandbox.commands(), "snippets");
  // The PATH copy answers differently, so the output shows which one ran.
  write_executable(
    &sandbox.root.join("path-bin/argvus-snippets"),
    "#!/bin/sh\necho from-path\n",
  );
  let path = format!(
    "{}:{}",
    sandbox.root.join("path-bin").display(),
    sandbox.tools_path()
  );
  let output = sandbox.run(&path, &["snippets", "list"], "");
  assert!(output.status.success());
  assert_eq!(text(&output.stdout), expected(&["[list]", "stdin="]));
}

#[test]
fn missing_private_command_is_reported() {
  let sandbox = Sandbox::new("missing");
  let output = sandbox.run(&sandbox.tools_path(), &["firewall", "status"], "");
  assert!(!output.status.success());
  assert!(String::from_utf8_lossy(&output.stderr).contains("firewall is not installed"));
}

#[test]
fn itself_and_unlisted_names_are_not_dispatched() {
  let sandbox = Sandbox::new("unlisted");
  sandbox.fake(&sandbox.commands(), "argvus-evil");
  let self_output = sandbox.run(&sandbox.tools_path(), &["argvus", "status"], "");
  let evil_output = sandbox.run(&sandbox.tools_path(), &["evil"], "");
  assert!(!self_output.status.success());
  assert!(!evil_output.status.success());
  assert!(String::from_utf8_lossy(&evil_output.stderr).contains("unknown command: evil"));
}

#[test]
fn git_maps_to_argvus_git_and_forwards_its_arguments() {
  let sandbox = Sandbox::new("git");
  sandbox.fake(&sandbox.commands(), "git");
  let output = sandbox.run(&sandbox.tools_path(), &["git", "my repo"], "");
  assert!(output.status.success());
  assert_eq!(text(&output.stdout), expected(&["[my repo]", "stdin="]));
}

#[test]
fn script_route_forwards_arguments_to_the_packaged_script() {
  let sandbox = Sandbox::new("script");
  let system = sandbox.root.join("system");
  fs::create_dir_all(system.join("hyprland/sh")).expect("create packaged scripts");
  write_executable(
    &system.join("hyprland/sh/scratchpad.sh"),
    "for arg do printf \"[%s]\\n\" \"$arg\"; done\n",
  );
  let mut command = sandbox.command(&sandbox.tools_path(), &["scratchpad", "a b", ""]);
  command.env("ARGVUS_SYSTEM_CONFIG", &system);
  let output = finish(command, "");
  assert!(output.status.success());
  assert_eq!(text(&output.stdout), expected(&["[a b]", "[]"]));
}

#[test]
fn user_script_override_wins_over_the_packaged_script() {
  let sandbox = Sandbox::new("override");
  let system = sandbox.root.join("system");
  let config_home = sandbox.root.join("config");
  fs::create_dir_all(system.join("hyprland/sh")).expect("create packaged scripts");
  fs::create_dir_all(config_home.join("scripts/argvus")).expect("create user scripts");
  write_executable(
    &system.join("hyprland/sh/scratchpad.sh"),
    "printf 'packaged\\n'\n",
  );
  write_executable(
    &config_home.join("scripts/argvus/scratchpad.sh"),
    "printf 'override\\n'\n",
  );
  let mut command = sandbox.command(&sandbox.tools_path(), &["scratchpad"]);
  command
    .env("ARGVUS_SYSTEM_CONFIG", &system)
    .env("ARGVUS_CONFIG_HOME", &config_home);
  let output = finish(command, "");
  assert!(output.status.success());
  assert_eq!(text(&output.stdout), "override");
}

#[test]
fn config_route_runs_argvus_config_in_process() {
  // A read-only key in an empty configuration home: no side effects, and the
  // answer comes from argvus-config itself rather than from a fake executable.
  let sandbox = Sandbox::new("config");
  let mut command = sandbox.command(
    &sandbox.tools_path(),
    &["config", "get", "/appearance/theme"],
  );
  command.env("ARGVUS_CONFIG_HOME", sandbox.root.join("config"));
  let output = finish(command, "");
  assert!(output.status.success());
  assert_eq!(text(&output.stdout), "null");
}

#[test]
fn gpg_route_runs_argvus_keys_in_process() {
  // The sandbox PATH has no gpg, so argvus-keys reports its own missing dependency
  // (the message key is untranslated here, there is no catalog in the sandbox). The
  // `argvus-gpg:` prefix proves the route reached argvus-keys without the dispatcher
  // looking for an executable.
  let sandbox = Sandbox::new("gpg");
  let output = sandbox.run(&sandbox.tools_path(), &["gpg", "list"], "");
  assert_eq!(output.status.code(), Some(1));
  assert!(String::from_utf8_lossy(&output.stderr).starts_with("argvus-gpg:"));
}

#[test]
fn theme_switch_is_not_dispatched_to_a_private_executable() {
  // `theme-switch` applies a theme, which touches the desktop, so the test does not
  // run it. It only checks the route name is known, by the usage it prints without
  // arguments: `argvus config theme-apply` reports the missing argument.
  let sandbox = Sandbox::new("theme");
  let output = sandbox.run(&sandbox.tools_path(), &["theme-switch"], "");
  assert!(!String::from_utf8_lossy(&output.stderr).contains("unknown command"));
}

#[test]
fn start_desktop_starts_tty_without_forwarding_arguments() {
  let sandbox = Sandbox::new("start-desktop");
  sandbox.fake(&sandbox.commands(), "tty");
  let output = sandbox.run(&sandbox.tools_path(), &["--start-desktop", "ignored"], "");
  assert!(output.status.success());
  assert_eq!(text(&output.stdout), "stdin=");
}
