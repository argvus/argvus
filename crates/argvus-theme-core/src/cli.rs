//! `argvus theme`: list installed themes, check a theme directory and render a
//! theme into a build. `argvus theme apply` is the theme switch; the dispatcher
//! sends it to `argvus config theme-apply`, which activates the build.

use crate::build::{self, RenderRequest};
use crate::color::Color;
use crate::discovery::{self, Discovery};
use crate::error::ThemeError;
use crate::layout::{Layout, Variant};
use crate::manifest::ThemeManifest;
use crate::paths::ThemePaths;
use crate::registry;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "\
usage: argvus theme apply ID
       argvus theme list
       argvus theme check DIRECTORY [--variant sticky|float]
       argvus theme render ID [--variant sticky|float] [--accent #RRGGBB] [--output DIRECTORY]";

const USAGE_ERROR: u8 = 64;

/// Runs one `argvus theme` command. `arguments` are the words after `theme`.
pub fn run_command(arguments: Vec<String>) -> ExitCode {
  let Some((command, rest)) = arguments.split_first() else {
    eprintln!("{USAGE}");
    return ExitCode::from(USAGE_ERROR);
  };
  let options = match Options::parse(rest) {
    Ok(options) => options,
    Err(message) => return usage_error(&message),
  };
  let Some(paths) = ThemePaths::from_environment() else {
    eprintln!("argvus theme: cannot resolve the configuration directories (HOME is not set)");
    return ExitCode::FAILURE;
  };
  match command.as_str() {
    "list" if options.positional.is_empty() => list(&paths),
    "check" if options.positional.len() == 1 => {
      check(&paths, Path::new(&options.positional[0]), &options)
    }
    "render" if options.positional.len() == 1 => render(&paths, &options.positional[0], &options),
    "help" | "--help" | "-h" => {
      println!("{USAGE}");
      ExitCode::SUCCESS
    }
    _ => usage_error(&format!("unexpected arguments: {}", arguments.join(" "))),
  }
}

fn usage_error(message: &str) -> ExitCode {
  eprintln!("argvus theme: {message}\n{USAGE}");
  ExitCode::from(USAGE_ERROR)
}

#[derive(Debug, Default, PartialEq)]
struct Options {
  positional: Vec<String>,
  variant: Option<Variant>,
  accent: Option<Color>,
  output: Option<PathBuf>,
}

impl Options {
  fn parse(arguments: &[String]) -> Result<Self, String> {
    let mut options = Self::default();
    let mut words = arguments.iter();
    while let Some(word) = words.next() {
      let mut value = |flag: &str| {
        words
          .next()
          .cloned()
          .ok_or_else(|| format!("{flag} needs a value"))
      };
      match word.as_str() {
        "--variant" => {
          let text = value("--variant")?;
          options.variant =
            Some(Variant::parse(&text).ok_or_else(|| format!("unknown variant `{text}`"))?);
        }
        "--accent" => {
          let text = value("--accent")?;
          options.accent = Some(
            Color::parse(&text)
              .filter(|color| color.alpha == 255 && text.len() == 7)
              .ok_or_else(|| format!("accent `{text}` is not #RRGGBB"))?,
          );
        }
        "--output" => options.output = Some(PathBuf::from(value("--output")?)),
        flag if flag.starts_with("--") => return Err(format!("unknown option `{flag}`")),
        _ => options.positional.push(word.clone()),
      }
    }
    Ok(options)
  }
}

fn list(paths: &ThemePaths) -> ExitCode {
  let discovery = discovery::discover(paths);
  report_warnings(&discovery);
  for theme in &discovery.themes {
    println!(
      "{}\t{}\t{}\t{}",
      theme.manifest.id,
      theme.manifest.category.as_str(),
      theme.source.as_str(),
      theme.manifest.name
    );
  }
  ExitCode::SUCCESS
}

/// Validates a theme directory against the installed templates, in both
/// layouts unless one is named.
fn check(paths: &ThemePaths, theme_dir: &Path, options: &Options) -> ExitCode {
  let manifest = match ThemeManifest::load(theme_dir) {
    Ok(manifest) => manifest,
    Err(error) => return failure(&[error]),
  };
  let consumers = match registry::load(&paths.system_root, &paths.registry()) {
    Ok(consumers) => consumers,
    Err(error) => return failure(&[error]),
  };
  let variants = options.variant.map_or_else(
    || vec![Variant::Sticky, Variant::Float],
    |variant| vec![variant],
  );
  let mut failed = false;
  for variant in &variants {
    if let Err(errors) = render_variant(paths, &manifest, *variant, options.accent, &consumers) {
      failed = true;
      for error in errors {
        eprintln!("argvus theme: {}: {error}", variant.as_str());
      }
    }
  }
  if failed {
    return ExitCode::FAILURE;
  }
  let layouts: Vec<_> = variants.iter().map(|variant| variant.as_str()).collect();
  println!(
    "{}: ok, {} consumer(s) rendered for {}",
    manifest.id,
    consumers.len(),
    layouts.join(" and ")
  );
  ExitCode::SUCCESS
}

/// Renders an installed theme. Without `--output` it becomes a build under
/// the generated theme directory, ready for the switch to activate.
fn render(paths: &ThemePaths, id: &str, options: &Options) -> ExitCode {
  let discovery = discovery::discover(paths);
  let Some(theme) = discovery.find(id) else {
    report_warnings(&discovery);
    eprintln!("argvus theme: theme `{id}` is not installed");
    return ExitCode::FAILURE;
  };
  let consumers = match registry::load(&paths.system_root, &paths.registry()) {
    Ok(consumers) => consumers,
    Err(error) => return failure(&[error]),
  };
  let variant = options.variant.unwrap_or(Variant::Sticky);
  let rendered = match render_variant(paths, &theme.manifest, variant, options.accent, &consumers) {
    Ok(rendered) => rendered,
    Err(errors) => return failure(&errors),
  };
  let written = match &options.output {
    Some(directory) => rendered.write_files(directory).map(|()| directory.clone()),
    None => rendered.write_build(&paths.builds()),
  };
  match written {
    Ok(directory) => {
      println!("{}", directory.display());
      ExitCode::SUCCESS
    }
    Err(error) => failure(&[error]),
  }
}

fn render_variant(
  paths: &ThemePaths,
  manifest: &ThemeManifest,
  variant: Variant,
  accent: Option<Color>,
  consumers: &[registry::Consumer],
) -> Result<build::Rendered, Vec<ThemeError>> {
  let layout = Layout::load(&paths.layouts(), variant).map_err(|error| vec![error])?;
  build::render(&RenderRequest {
    manifest,
    variant,
    accent,
    layout: layout.as_ref(),
    consumers,
  })
}

fn report_warnings(discovery: &Discovery) {
  for warning in &discovery.warnings {
    eprintln!("argvus theme: warning: {warning}");
  }
}

fn failure(errors: &[ThemeError]) -> ExitCode {
  for error in errors {
    eprintln!("argvus theme: {error}");
  }
  ExitCode::FAILURE
}

#[cfg(test)]
mod tests {
  use super::*;

  fn words(text: &str) -> Vec<String> {
    text.split_whitespace().map(str::to_owned).collect()
  }

  #[test]
  fn parses_flags_in_any_order() {
    let options = Options::parse(&words(
      "sunset --accent #3590BD --variant float --output /tmp/x",
    ))
    .unwrap();
    assert_eq!(options.positional, ["sunset"]);
    assert_eq!(options.variant, Some(Variant::Float));
    assert_eq!(options.accent, Color::parse("#3590BD"));
    assert_eq!(options.output.as_deref(), Some(Path::new("/tmp/x")));
  }

  #[test]
  fn rejects_bad_flags() {
    for (text, reason) in [
      ("--variant", "needs a value"),
      ("--variant round", "unknown variant"),
      ("--accent #3590BD80", "#RRGGBB"),
      ("--accent #fff", "#RRGGBB"),
      ("--force", "unknown option"),
    ] {
      let error = Options::parse(&words(text)).unwrap_err();
      assert!(error.contains(reason), "{text}: {error}");
    }
  }
}
