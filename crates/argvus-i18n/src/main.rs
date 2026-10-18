//! The `argvus-i18n` binary. The `argvus` dispatcher runs the same code in-process.

use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
  let arguments: Vec<String> = env::args().skip(1).collect();
  argvus_i18n_cli::run(&arguments)
}
