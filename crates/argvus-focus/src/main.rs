//! ARGVUS Focus: Pomodoro and focus sessions.
//!
//! One instance runs per session (`sh.argvus.Focus` on the session bus); a
//! second launch hands its arguments to the running instance and exits.

mod app;
mod cli;
mod engine;
mod history;
mod settings;
mod ui;

fn main() -> gtk::glib::ExitCode {
  env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
  // Parse here first, so `--help` and errors are printed by this process
  // even when another instance already runs and receives the arguments.
  if let Err(error) = cli::Cli::parse_arguments(std::env::args_os().collect()) {
    error.exit();
  }
  app::run()
}
