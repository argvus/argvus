//! ARGVUS Clock: clock, world clocks, stopwatch, timer and alarms.
//!
//! One instance runs per session (`sh.argvus.Clock` on the session bus); a
//! second launch hands its arguments to the running instance and exits.

mod alarms;
mod app;
mod autostart;
mod cli;
mod ringing;
mod settings;
mod timers;
mod ui;

fn main() -> gtk::glib::ExitCode {
  env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
  // Parse here first, so `--help` and errors are printed by this process
  // even when another instance already runs and receives the arguments.
  use clap::ValueEnum as _;

  let cli =
    cli::Cli::parse_arguments(std::env::args_os().collect()).unwrap_or_else(|error| error.exit());
  let page = cli.page.and_then(|page| page.to_possible_value());
  if !cli.morphing_session
    && !cli.background
    && !cli.tooltip
    && argvus_taskbar_popup::morphing::present(
      "clock",
      cli.toggle,
      cli.anchor(),
      page.as_ref().map_or("", |page| page.get_name()),
    )
  {
    return gtk::glib::ExitCode::SUCCESS;
  }
  app::run()
}
