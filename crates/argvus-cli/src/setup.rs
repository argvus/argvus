//! `argvus --setup`: copies packaged defaults into the user configuration so they
//! can be customized. Packaged files are only read, never changed.

use crate::copy::copy_tree;
use crate::exec::{Failure, Outcome, env_nonempty};
use crate::usage;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};

const DEFAULT_CONFIG_SOURCE: &str = "/usr/share/argvus";
/// The app whose packaged files are merged into the configuration root itself.
const ROOT_APP: &str = "argvus";

/// The parsed `--setup` options.
#[derive(Default)]
struct Options {
  apps: Vec<String>,
  copy_all: bool,
  force: bool,
  dry_run: bool,
}

/// Runs `argvus --setup`. Validation runs in the same order as the shell, so the
/// same error is reported for the same input.
pub fn run(arguments: &[OsString]) -> Outcome {
  let Some(options) = parse(arguments)? else {
    print!("{}", usage::TEXT);
    return Ok(ExitCode::SUCCESS);
  };
  let home = env_nonempty("HOME").ok_or_else(|| Failure::setup("HOME is not set."))?;
  let source = env_nonempty("ARGVUS_CONFIG_SRC")
    .map(PathBuf::from)
    .unwrap_or_else(|| PathBuf::from(DEFAULT_CONFIG_SOURCE));
  if !source.is_dir() {
    return Err(Failure::setup(format!(
      "config source not found: {}",
      source.display()
    )));
  }
  if !options.copy_all && options.apps.is_empty() {
    print!("{}", usage::TEXT);
    return Ok(ExitCode::SUCCESS);
  }

  let apps = if options.copy_all {
    packaged_apps(&source)?
  } else {
    options.apps.clone()
  };
  let destination = configuration_destination(&home);
  let setup = Setup {
    source,
    destination,
    dry_run: options.dry_run,
    force: options.force,
    timestamp: backup_timestamp(),
  };

  setup.perform(format!("mkdir -p {}", setup.destination.display()), || {
    fs::create_dir_all(&setup.destination)
  })?;
  for app in &apps {
    setup.copy_app(app)?;
  }
  println!("Done. Copied files are now user overrides.");
  Ok(ExitCode::SUCCESS)
}

/// Parses the options. `Ok(None)` means `--help` was requested.
fn parse(arguments: &[OsString]) -> Result<Option<Options>, Failure> {
  let mut options = Options::default();
  let mut index = 0;
  while index < arguments.len() {
    let argument = arguments[index].to_string_lossy().into_owned();
    match argument.as_str() {
      "--copy" => {
        let app = arguments
          .get(index + 1)
          .ok_or_else(|| Failure::setup("--copy requires an app name."))?;
        push_app(&mut options, &app.to_string_lossy())?;
        index += 1;
      }
      copy if copy.starts_with("--copy=") => {
        push_app(&mut options, &copy["--copy=".len()..])?;
      }
      "--copy-all" => options.copy_all = true,
      "--force" => options.force = true,
      "--repair" => push_app(&mut options, ROOT_APP)?,
      "--dry-run" => options.dry_run = true,
      "--help" | "-h" => return Ok(None),
      unknown => return Err(Failure::setup(format!("unknown option: {unknown}"))),
    }
    index += 1;
  }
  Ok(Some(options))
}

/// Accepts an app name only when it is a single, visible path component.
fn push_app(options: &mut Options, app: &str) -> Result<(), Failure> {
  if app.is_empty() || app.starts_with('.') || app.contains('/') {
    return Err(Failure::setup(format!("invalid app name: {app}")));
  }
  options.apps.push(app.to_owned());
  Ok(())
}

/// The user configuration root that setup writes to.
fn configuration_destination(home: &OsString) -> PathBuf {
  let config_home = env_nonempty("ARGVUS_CONFIG_HOME")
    .or_else(|| env_nonempty("XDG_CONFIG_HOME"))
    .map(PathBuf::from)
    .unwrap_or_else(|| {
      let mut default_home = home.clone();
      default_home.push("/.config");
      PathBuf::from(default_home)
    });
  config_home.join("argvus")
}

/// The packaged apps for `--copy-all`: each visible directory, sorted by name.
fn packaged_apps(source: &Path) -> Result<Vec<String>, Failure> {
  let entries = fs::read_dir(source)
    .map_err(|error| Failure::setup(format!("{}: {error}", source.display())))?;
  let mut apps: Vec<String> = entries
    .filter_map(Result::ok)
    .filter(|entry| entry.path().is_dir())
    .map(|entry| entry.file_name().to_string_lossy().into_owned())
    .filter(|name| !name.starts_with('.'))
    .collect();
  apps.sort();
  Ok(apps)
}

/// The timestamp that names backups, from `date +%Y%m%d%H%M%S`. It falls back to
/// the Unix time when `date` cannot run.
fn backup_timestamp() -> String {
  let output = Command::new("date")
    .arg("+%Y%m%d%H%M%S")
    .stderr(Stdio::null())
    .output();
  match output {
    Ok(output) if output.status.success() => {
      String::from_utf8_lossy(&output.stdout).trim().to_owned()
    }
    _ => std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .map_or(0, |elapsed| elapsed.as_secs())
      .to_string(),
  }
}

/// The state of one setup run: where to copy from and to, and how to report it.
struct Setup {
  source: PathBuf,
  destination: PathBuf,
  dry_run: bool,
  force: bool,
  timestamp: String,
}

impl Setup {
  fn copy_app(&self, app: &str) -> Result<(), Failure> {
    let app_source = self.source.join(app);
    if !app_source.is_dir() {
      return Err(Failure::setup(format!(
        "packaged config not found for app '{app}': {}",
        app_source.display()
      )));
    }
    // The root app is merged into the configuration root, so its contents are
    // copied (`src/.`) rather than the directory itself.
    let (app_destination, copy_source) = if app == ROOT_APP {
      (self.destination.clone(), app_source.join("."))
    } else {
      (self.destination.join(app), app_source.clone())
    };

    if app_destination.symlink_metadata().is_ok() {
      if !self.force {
        return Err(Failure::setup(format!(
          "destination already exists: {}\nUse --force to replace it after a backup.",
          app_destination.display()
        )));
      }
      let backup = self.backup_path(&app_destination);
      self.perform(
        format!("mv {} {}", app_destination.display(), backup.display()),
        || fs::rename(&app_destination, &backup),
      )?;
      println!(
        "Backup: {} -> {}",
        app_destination.display(),
        backup.display()
      );
    }

    let parent = app_destination.parent().unwrap_or(Path::new("."));
    self.perform(format!("mkdir -p {}", parent.display()), || {
      fs::create_dir_all(parent)
    })?;
    self.perform(
      format!(
        "cp -R {} {}",
        copy_source.display(),
        app_destination.display()
      ),
      || copy_tree(&copy_source, &app_destination),
    )?;
    println!(
      "Copied: {} -> {}",
      app_source.display(),
      app_destination.display()
    );
    Ok(())
  }

  /// The first free backup name for `destination`: `<name>.bak-<timestamp>`, with
  /// a numeric suffix when that name is already taken.
  fn backup_path(&self, destination: &Path) -> PathBuf {
    let base = format!("{}.bak-{}", destination.display(), self.timestamp);
    let mut candidate = PathBuf::from(&base);
    let mut suffix = 1;
    while candidate.symlink_metadata().is_ok() {
      candidate = PathBuf::from(format!("{base}-{suffix}"));
      suffix += 1;
    }
    candidate
  }

  /// Runs `action`, or only prints it in dry-run mode. The description is the
  /// command line the action stands for, so the dry-run output reads like the
  /// shell's.
  fn perform(
    &self,
    description: String,
    action: impl FnOnce() -> io::Result<()>,
  ) -> Result<(), Failure> {
    if self.dry_run {
      println!("[dry-run] {description}");
      return Ok(());
    }
    action().map_err(|error| Failure::setup(format!("{description}: {error}")))
  }
}
