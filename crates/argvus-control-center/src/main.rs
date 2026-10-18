//! Implements executable entry point and terminal lifecycle in crate `argvus control center`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use std::time::Duration;

use anyhow::Result;
use argvus_control_center::app::App;
use argvus_control_center::{cli, event, ui};
use argvus_tui::terminal::{TerminalGuard, install_panic_hook};

/// Polls one domain when its feature is compiled in and folds its change flag
/// into `$dirty`. The `cfg` is evaluated where the macro is expanded.
macro_rules! poll_domain {
  ($dirty:ident, $feature:literal, $domain:expr) => {
    #[cfg(feature = $feature)]
    {
      $dirty |= $domain.poll();
    }
  };
}

/// Executes the `main` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn main() -> Result<()> {
  let args: Vec<String> = std::env::args().skip(1).collect();
  if args.first().is_some_and(|arg| arg == "system-settings") {
    if let Err(error) = argvus_control_center_settings::system::command::run(&args[1..]) {
      eprintln!("argvus-control-center: {error}");
      std::process::exit(1);
    }
    return Ok(());
  }

  let Some(initial) = cli::parse_or_print()? else {
    return Ok(());
  };

  install_panic_hook();
  let mut app = App::new(initial);
  let mut terminal = TerminalGuard::new()?;
  let mut dirty = true;
  while !app.quit {
    if dirty {
      terminal
        .terminal_mut()
        .draw(|frame| ui::draw(&mut app, frame))?;
      dirty = false;
    }
    if crossterm::event::poll(Duration::from_millis(100))? {
      event::handle(&mut app, crossterm::event::read()?);
      dirty = true;
    }
    if app.settings.expire_status() {
      dirty = true;
    }
    if app.settings.poll() {
      dirty = true;
    }
    if app.config.poll() {
      dirty = true;
    }
    if app.poll_search_cursor() {
      dirty = true;
    }
    #[cfg(any(
      feature = "hardware",
      feature = "services",
      feature = "dev-services",
      feature = "network",
      feature = "audio",
      feature = "bluetooth",
      feature = "boot",
      feature = "packages",
      feature = "storage",
      feature = "diagnostics",
      feature = "power",
      feature = "session",
      feature = "displays",
      feature = "appearance"
    ))]
    {
      let mut domain_dirty = false;
      poll_domain!(domain_dirty, "hardware", app.hardware);
      poll_domain!(domain_dirty, "services", app.services);
      poll_domain!(domain_dirty, "dev-services", app.dev_services);
      poll_domain!(domain_dirty, "network", app.network);
      poll_domain!(domain_dirty, "audio", app.audio);
      poll_domain!(domain_dirty, "bluetooth", app.bluetooth);
      poll_domain!(domain_dirty, "boot", app.boot);
      poll_domain!(domain_dirty, "packages", app.packages);
      poll_domain!(domain_dirty, "storage", app.storage);
      poll_domain!(domain_dirty, "diagnostics", app.diagnostics);
      poll_domain!(domain_dirty, "power", app.power);
      poll_domain!(domain_dirty, "session", app.session);
      poll_domain!(domain_dirty, "displays", app.displays);
      #[cfg(feature = "appearance")]
      {
        domain_dirty |= app.appearance.poll();
        if app.appearance.take_theme_dirty() {
          app.refresh_theme();
          domain_dirty = true;
        }
      }
      if domain_dirty {
        dirty = true;
      }
    }
  }
  Ok(())
}
