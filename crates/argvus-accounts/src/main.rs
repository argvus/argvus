//! Entry point: parsing, dispatch and error presentation.

mod cli;
mod commands;
mod elevate;

use clap::Parser;
use cli::Cli;

fn main() {
  let args = Cli::parse();

  if let Err(err) = commands::execute(&args) {
    eprintln!("error: {err}");
    if args.verbose {
      let mut source = std::error::Error::source(&err);
      while let Some(current) = source {
        eprintln!("  caused by: {current}");
        source = current.source();
      }
    }
    std::process::exit(1);
  }
}
