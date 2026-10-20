//! Runs the launcher against a temporary ARGVUS tree: the generated kitty
//! configuration carries the chosen theme, font and transparency, and a launch
//! hands the generated file to kitty.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// A temporary directory that holds the fixture tree of one test.
struct Fixture {
  root: PathBuf,
}

impl Fixture {
  fn new(name: &str) -> Self {
    let root = std::env::temp_dir().join(format!("argvus-terminal-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("argvus/data")).expect("create the data directory");
    fs::create_dir_all(root.join("cache")).expect("create the cache directory");
    Self { root }
  }

  /// The data directory of the ARGVUS configuration home.
  fn data(&self) -> PathBuf {
    self.root.join("argvus/data")
  }

  /// Installs an `argvus` that answers the canonical pointers with `answers`.
  fn answer_configuration(&self, answers: &[(&str, &str)]) {
    let bin = self.root.join("bin");
    fs::create_dir_all(&bin).expect("create the bin directory");
    let mut script =
      String::from("#!/bin/sh\n[ \"$1 $2\" = \"config get\" ] || exit 1\ncase \"$3\" in\n");
    for (pointer, value) in answers {
      script.push_str(&format!("  {pointer}) printf '%s\\n' '{value}' ;;\n"));
    }
    script.push_str("  *) exit 1 ;;\nesac\n");
    let argvus = bin.join("argvus");
    fs::write(&argvus, script).expect("write the argvus stub");
    make_executable(&argvus);
  }

  fn command(&self) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_argvus-terminal"));
    command
      .env("ARGVUS_CONFIG_HOME", &self.root)
      .env("ARGVUS_CACHE_HOME", self.root.join("cache"))
      .env("ARGVUS_TERMINAL_SYSTEM_CONFIG", system_root())
      .env("ARGVUS_SYSTEM_CONFIG", &self.root)
      .env("ARGVUS_COMMANDS_DIR", "/nonexistent")
      .env("PATH", self.root.join("bin"));
    command
  }
}

impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.root);
  }
}

/// The terminal package's own files, as the repository ships them.
fn system_root() -> PathBuf {
  Path::new(env!("CARGO_MANIFEST_DIR"))
    .join("../../packaging/archlinux/src/usr/share/argvus/terminal")
}

#[test]
fn print_config_writes_the_theme_font_and_accent() {
  let fixture = Fixture::new("print-config");
  fs::write(fixture.data().join(".active-theme"), "argvus-dark\n").unwrap();
  fs::write(fixture.data().join(".accent-color"), "#112233\n").unwrap();
  fs::create_dir_all(fixture.data().join("generated")).unwrap();
  fs::write(
    fixture.data().join("generated/fonts.conf"),
    "terminal_family=Fira Code\nterminal_size=14\n",
  )
  .unwrap();

  let output = fixture
    .command()
    .arg("--print-config")
    .output()
    .expect("run the launcher");

  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  let path = String::from_utf8_lossy(&output.stdout)
    .trim_end()
    .to_string();
  assert!(path.ends_with("argvus-terminal/kitty.conf"), "{path}");
  let generated = fs::read_to_string(&path).unwrap();
  assert!(generated.contains("font_family      Fira Code\n"));
  assert!(generated.contains("font_size        14.0\n"));
  assert!(generated.contains("active_tab_foreground   #112233\n"));
}

#[test]
fn canonical_configuration_wins_over_the_legacy_files() {
  let fixture = Fixture::new("canonical");
  fixture.answer_configuration(&[
    ("/appearance/theme", "dracula"),
    ("/appearance/accent", "#D08770"),
  ]);
  fs::write(fixture.data().join(".active-theme"), "argvus-dark\n").unwrap();
  fs::write(fixture.data().join(".accent-color"), "#112233\n").unwrap();

  let output = fixture
    .command()
    .arg("--print-config")
    .output()
    .expect("run the launcher");

  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  let path = String::from_utf8_lossy(&output.stdout)
    .trim_end()
    .to_string();
  let generated = fs::read_to_string(&path).unwrap();
  assert!(
    generated.contains("/themes/dracula/theme.conf\n"),
    "{generated}"
  );
  assert!(
    generated.contains("active_tab_foreground   #D08770\n"),
    "{generated}"
  );
}

#[test]
fn launch_hands_the_generated_configuration_to_kitty() {
  let fixture = Fixture::new("launch");
  let bin = fixture.root.join("bin");
  fs::create_dir_all(&bin).unwrap();
  let arguments_file = fixture.root.join("kitty-arguments");
  let kitty = bin.join("kitty");
  fs::write(
    &kitty,
    format!(
      "#!/bin/sh\nprintf '%s\\n' \"$@\" > '{}'\n",
      arguments_file.display()
    ),
  )
  .unwrap();
  make_executable(&kitty);

  let status = fixture
    .command()
    .env("PATH", format!("{}:/usr/bin:/bin", bin.display()))
    .args(["--class", "argvus-keys", "--", "-e", "zsh"])
    .status()
    .expect("run the launcher");

  assert!(status.success());
  let arguments = fs::read_to_string(&arguments_file).unwrap();
  let lines: Vec<&str> = arguments.lines().collect();
  assert_eq!(&lines[..2], ["--class", "argvus-keys"]);
  assert_eq!(lines[2], "--config");
  assert!(
    lines[3].ends_with("argvus-terminal/kitty.conf"),
    "{}",
    lines[3]
  );
  assert_eq!(&lines[4..], ["-e", "zsh"]);
}

#[test]
fn unknown_font_profile_is_rejected() {
  let fixture = Fixture::new("profile");
  let output = fixture
    .command()
    .args(["--font-profile", "editor", "--print-config"])
    .output()
    .expect("run the launcher");
  assert_eq!(output.status.code(), Some(1));
  assert!(String::from_utf8_lossy(&output.stderr).starts_with("argvus-terminal: "));
}

#[cfg(unix)]
fn make_executable(path: &Path) {
  use std::os::unix::fs::PermissionsExt;
  fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}
