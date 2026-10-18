//! The `argvus-networkctl` binary. The `argvus` dispatcher runs the same code in-process.

use std::env;
use std::path::Path;
use std::process::ExitCode;

fn main() -> ExitCode {
  let program = env::args().next().unwrap_or_default();
  let program_name = Path::new(&program)
    .file_name()
    .map(|name| name.to_string_lossy().into_owned())
    .unwrap_or_default();
  argvus_network_commands::networkctl::run_command(&program_name, env::args().skip(1).collect())
}
