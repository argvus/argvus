//! The `argvus-config` binary. The commands are in the library, so the
//! `argvus` dispatcher can run them in-process.

use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
  let arguments: Vec<String> = env::args().skip(1).collect();
  argvus_config_core::cli::run_command(arguments)
}
