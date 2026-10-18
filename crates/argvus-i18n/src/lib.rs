//! `argvus i18n`: prints translations, the detected locale and catalog checks.
//! The `argvus` dispatcher calls [`run`] in-process; the `argvus-i18n` binary is a
//! thin wrapper over the same function, kept for callers not yet moved.

use argvus_i18n::{
  I18n, SYSTEM_CATALOG_DIR, detect_locale_from, discover_locales, validate_catalogs,
};
use std::path::PathBuf;
use std::process::ExitCode;

/// Runs one `i18n` command. `arguments` are the words after the command name.
pub fn run(arguments: &[String]) -> ExitCode {
  let mut args = arguments.iter().map(String::as_str);
  let root = std::env::var_os("ARGVUS_I18N_DIR")
    .map(PathBuf::from)
    .unwrap_or_else(|| SYSTEM_CATALOG_DIR.into());
  match args.next() {
    Some("get") => {
      let domain = args.next().unwrap_or_default();
      let key = args.next().unwrap_or_default();
      let values = args.collect::<Vec<_>>();
      let pairs = values.iter().filter_map(|value| value.split_once('='));
      match I18n::from_root(&root, domain) {
        Ok(i18n) => {
          println!("{}", i18n.tr_args(key, pairs));
          ExitCode::SUCCESS
        }
        Err(error) => {
          eprintln!("argvus-i18n: {error}");
          ExitCode::FAILURE
        }
      }
    }
    Some("locale") => {
      println!("{}", detect_locale_from(&root));
      ExitCode::SUCCESS
    }
    Some("list") => {
      for locale in discover_locales(&root) {
        println!("{}\t{}\t{}", locale.locale, locale.name, locale.native_name);
      }
      ExitCode::SUCCESS
    }
    Some("validate") => match validate_catalogs(&root, args.next()) {
      Ok(()) => {
        println!("catalogs valid");
        ExitCode::SUCCESS
      }
      Err(errors) => {
        for error in errors {
          eprintln!("{error}");
        }
        ExitCode::FAILURE
      }
    },
    _ => {
      eprintln!(
        "usage: argvus-i18n <get DOMAIN KEY [name=value ...]|locale|list|validate [LOCALE]>"
      );
      ExitCode::from(2)
    }
  }
}
