//! The binary of this command. The `argvus` dispatcher runs the same code in-process.

use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
  argvus_tui_terminal::run_command(env::args().skip(1).collect())
}
