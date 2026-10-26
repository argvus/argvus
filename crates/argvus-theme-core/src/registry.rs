//! The consumer registry: one `<system>/theme/consumers.d/<id>.toml` per
//! themed file a component owns. The component ships the template and says how
//! its running program picks up a new build.
//!
//! ```toml
//! id = "taskbar"
//! template = "taskbar/theme-template/theme.css"
//! output = "taskbar/theme.css"
//!
//! [reload]
//! kind = "rewrite"
//! path = "argvus/data/waybar/argvus-taskbar.css"
//! ```
//!
//! `template` is relative to the system root, `output` to the build directory.
//! `reload_stage = "build"` reloads the consumer as soon as the build is active,
//! because it reads the build directly; the default, `"files"`, waits until
//! the switch has written the files the consumer reads.
//!
//! `[palette]` gives a default to the tokens the template reads, as palette
//! expressions (`btop_cpu_mid = "@fg_dim"`). A theme that defines the token
//! wins, so a theme only needs the central tokens and a new consumer needs no
//! change in the themes. Two consumers may default the same token only to the
//! same expression.

use crate::error::{ThemeError, read_text};
use crate::manifest::validate_id;
use crate::paths::relative_inside;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

/// How a running consumer picks up a new build. The theme switch carries it out.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
pub enum Reload {
  /// The consumer reads the build the next time it starts.
  None,
  /// Rewrite a watched file so the consumer reloads it (Waybar's
  /// `reload_style_on_change`). `path` is relative to the config home.
  Rewrite { path: String },
  /// Send a signal to every process with this name.
  Signal { process: String, signal: String },
  /// Run a command; the first word is the program.
  Command { argv: Vec<String> },
}

/// When a running consumer reloads during a theme switch.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ReloadStage {
  /// As soon as `current` points at the new build.
  Build,
  /// After the switch has written the consumer files.
  #[default]
  Files,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawConsumer {
  id: String,
  template: String,
  output: String,
  #[serde(default = "no_reload")]
  reload: Reload,
  #[serde(default)]
  reload_stage: ReloadStage,
  #[serde(default)]
  palette: BTreeMap<String, String>,
}

fn no_reload() -> Reload {
  Reload::None
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Consumer {
  pub id: String,
  /// The packaged template, an absolute path under the system root.
  pub template: PathBuf,
  /// The rendered file, relative to the build directory.
  pub output: PathBuf,
  pub reload: Reload,
  pub reload_stage: ReloadStage,
  /// Default palette expressions for the tokens of the template.
  pub palette: BTreeMap<String, String>,
}

impl Consumer {
  pub fn parse(system_root: &Path, path: &Path, text: &str) -> Result<Self, ThemeError> {
    let invalid = |message: String| ThemeError::invalid(path, message);
    let raw: RawConsumer = toml::from_str(text).map_err(|error| invalid(error.to_string()))?;
    validate_id(&raw.id).map_err(|message| invalid(message.replace("id ", "consumer id ")))?;
    let template = relative_inside(&raw.template)
      .map(|relative| system_root.join(relative))
      .map_err(|message| invalid(format!("template: {message}")))?;
    let output =
      relative_inside(&raw.output).map_err(|message| invalid(format!("output: {message}")))?;
    validate_reload(&raw.reload).map_err(invalid)?;
    crate::manifest::validate_palette_entries(&raw.palette).map_err(invalid)?;
    Ok(Self {
      id: raw.id,
      template,
      output,
      reload: raw.reload,
      reload_stage: raw.reload_stage,
      palette: raw.palette,
    })
  }
}

fn validate_reload(reload: &Reload) -> Result<(), String> {
  match reload {
    Reload::None => Ok(()),
    Reload::Rewrite { path } => relative_inside(path)
      .map(|_| ())
      .map_err(|message| format!("reload.path: {message}")),
    Reload::Signal { process, signal } => {
      if process.is_empty() || process.contains('/') {
        return Err(format!("reload.process `{process}` must be a program name"));
      }
      let valid_signal = signal
        .strip_prefix("SIG")
        .unwrap_or(signal)
        .chars()
        .all(|character| {
          character.is_ascii_uppercase() || character.is_ascii_digit() || character == '+'
        });
      if signal.is_empty() || !valid_signal {
        return Err(format!("reload.signal `{signal}` is not a signal name"));
      }
      Ok(())
    }
    Reload::Command { argv } => {
      if argv.first().is_none_or(String::is_empty) {
        return Err("reload.argv must name a program".into());
      }
      Ok(())
    }
  }
}

/// Loads every `*.toml` of the registry directory, sorted by file name. A
/// missing directory is an empty registry. Ids and outputs must be unique.
pub fn load(system_root: &Path, directory: &Path) -> Result<Vec<Consumer>, ThemeError> {
  let entries = match fs::read_dir(directory) {
    Ok(entries) => entries,
    Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
    Err(error) => return Err(ThemeError::io(directory, error)),
  };
  let mut files = entries
    .map(|entry| entry.map(|entry| entry.path()))
    .collect::<Result<Vec<_>, _>>()
    .map_err(|error| ThemeError::io(directory, error))?;
  files.retain(|path| {
    path
      .extension()
      .is_some_and(|extension| extension == "toml")
  });
  files.sort();

  let mut consumers = Vec::with_capacity(files.len());
  let mut ids = BTreeSet::new();
  let mut outputs = BTreeSet::new();
  let mut defaults: BTreeMap<String, (String, String)> = BTreeMap::new();
  for path in files {
    let consumer = Consumer::parse(system_root, &path, &read_text(&path)?)?;
    if !ids.insert(consumer.id.clone()) {
      return Err(ThemeError::invalid(
        &path,
        format!("consumer `{}` is registered twice", consumer.id),
      ));
    }
    if !outputs.insert(consumer.output.clone()) {
      return Err(ThemeError::invalid(
        &path,
        format!(
          "output `{}` is already written by another consumer",
          consumer.output.display()
        ),
      ));
    }
    for (token, expression) in &consumer.palette {
      match defaults.get(token) {
        Some((owner, existing)) if existing != expression => {
          return Err(ThemeError::invalid(
            &path,
            format!("palette.{token} is already defaulted to `{existing}` by `{owner}`"),
          ));
        }
        Some(_) => {}
        None => {
          defaults.insert(token.clone(), (consumer.id.clone(), expression.clone()));
        }
      }
    }
    consumers.push(consumer);
  }
  Ok(consumers)
}

/// The palette defaults of `consumers`, merged (`load` refuses conflicts).
pub fn palette_defaults(consumers: &[Consumer]) -> BTreeMap<String, String> {
  consumers
    .iter()
    .flat_map(|consumer| consumer.palette.iter())
    .map(|(token, expression)| (token.clone(), expression.clone()))
    .collect()
}

#[cfg(test)]
mod tests {
  use super::*;

  fn parse(text: &str) -> Result<Consumer, ThemeError> {
    Consumer::parse(
      Path::new("/usr/share/argvus"),
      Path::new("taskbar.toml"),
      text,
    )
  }

  #[test]
  fn parses_each_reload_kind() {
    let consumer = parse(
      "id = \"taskbar\"\ntemplate = \"taskbar/theme-template/theme.css\"\noutput = \"taskbar/theme.css\"\n\
       [reload]\nkind = \"rewrite\"\npath = \"argvus/data/waybar/argvus-taskbar.css\"\n",
    )
    .unwrap();
    assert_eq!(
      consumer.template,
      Path::new("/usr/share/argvus/taskbar/theme-template/theme.css")
    );
    assert!(matches!(consumer.reload, Reload::Rewrite { .. }));

    let base = "id = \"x\"\ntemplate = \"t\"\noutput = \"o\"\n";
    assert_eq!(parse(base).unwrap().reload, Reload::None);
    assert_eq!(parse(base).unwrap().reload_stage, ReloadStage::Files);
    assert_eq!(
      parse(&format!("{base}reload_stage = \"build\"\n"))
        .unwrap()
        .reload_stage,
      ReloadStage::Build
    );
    assert!(matches!(
      parse(&format!(
        "{base}[reload]\nkind = \"signal\"\nprocess = \"btop\"\nsignal = \"SIGUSR2\"\n"
      ))
      .unwrap()
      .reload,
      Reload::Signal { .. }
    ));
    assert!(matches!(
      parse(&format!(
        "{base}[reload]\nkind = \"command\"\nargv = [\"dunstctl\", \"reload\"]\n"
      ))
      .unwrap()
      .reload,
      Reload::Command { .. }
    ));
  }

  #[test]
  fn rejects_unsafe_or_incomplete_entries() {
    let base = "id = \"x\"\ntemplate = \"t\"\noutput = \"o\"\n";
    for (text, reason) in [
      (base.replace("\"o\"", "\"../o\""), "output"),
      (base.replace("\"t\"", "\"/etc/t\""), "template"),
      (base.replace("\"x\"", "\"X\""), "consumer id"),
      (
        format!("{base}[reload]\nkind = \"signal\"\nprocess = \"/bin/sh\"\nsignal = \"USR2\"\n"),
        "program name",
      ),
      (
        format!("{base}[reload]\nkind = \"signal\"\nprocess = \"btop\"\nsignal = \"usr2;rm\"\n"),
        "signal name",
      ),
      (
        format!("{base}[reload]\nkind = \"command\"\nargv = []\n"),
        "name a program",
      ),
      (
        format!("{base}[reload]\nkind = \"rewrite\"\npath = \"/etc/x\"\n"),
        "reload.path",
      ),
      (format!("{base}extra = 1\n"), "unknown field"),
    ] {
      let error = parse(&text).unwrap_err().to_string();
      assert!(error.contains(reason), "{reason}: {error}");
    }
  }

  #[test]
  fn loads_sorted_and_refuses_duplicates() {
    let root = tempfile::tempdir().unwrap();
    let directory = root.path().join("consumers.d");
    assert!(load(root.path(), &directory).unwrap().is_empty());

    fs::create_dir(&directory).unwrap();
    fs::write(
      directory.join("b.toml"),
      "id = \"b\"\ntemplate = \"t\"\noutput = \"b.css\"\n",
    )
    .unwrap();
    fs::write(
      directory.join("a.toml"),
      "id = \"a\"\ntemplate = \"t\"\noutput = \"a.css\"\n",
    )
    .unwrap();
    fs::write(directory.join("notes.txt"), "ignored").unwrap();
    let ids: Vec<_> = load(root.path(), &directory)
      .unwrap()
      .into_iter()
      .map(|consumer| consumer.id)
      .collect();
    assert_eq!(ids, ["a", "b"]);

    fs::write(
      directory.join("c.toml"),
      "id = \"c\"\ntemplate = \"t\"\noutput = \"a.css\"\n",
    )
    .unwrap();
    assert!(
      load(root.path(), &directory)
        .unwrap_err()
        .to_string()
        .contains("already written")
    );
  }

  #[test]
  fn palette_defaults_merge_and_refuse_conflicts() {
    let root = tempfile::tempdir().unwrap();
    let directory = root.path().join("consumers.d");
    fs::create_dir(&directory).unwrap();
    let entry = |id: &str, palette: &str| {
      fs::write(
        directory.join(format!("{id}.toml")),
        format!("id = \"{id}\"\ntemplate = \"t\"\noutput = \"{id}.css\"\n[palette]\n{palette}\n"),
      )
      .unwrap();
    };
    entry("a", "card = \"@bg\"\nline = \"@fg | alpha(0.2)\"");
    entry("b", "card = \"@bg\"");
    let consumers = load(root.path(), &directory).unwrap();
    let defaults = palette_defaults(&consumers);
    assert_eq!(defaults["card"], "@bg");
    assert_eq!(defaults["line"], "@fg | alpha(0.2)");

    entry("c", "card = \"@fg\"");
    let error = load(root.path(), &directory).unwrap_err().to_string();
    assert!(error.contains("already defaulted"), "{error}");

    entry("c", "Card = \"@fg\"");
    let error = load(root.path(), &directory).unwrap_err().to_string();
    assert!(error.contains("not a valid entry name"), "{error}");
  }
}
