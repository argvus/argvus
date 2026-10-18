mod app;
mod caldav;
mod calendar;
mod config;
mod error;
mod i18n;
mod ical;
mod ipc;
mod reminders;
mod service;
mod storage;
mod ui;

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use error::Result;
use ipc::PopupAnchor;

#[derive(Debug, Parser)]
#[command(
  name = "argvus-taskbar-calendar",
  version,
  about = "Native Argvus calendar popup for Wayland"
)]
struct Cli {
  /// Horizontal click position supplied by the Waybar launcher.
  #[arg(
    long,
    global = true,
    value_name = "PX",
    allow_hyphen_values = true,
    requires = "y"
  )]
  x: Option<i32>,

  /// Vertical click position supplied by the Waybar launcher.
  #[arg(
    long,
    global = true,
    value_name = "PX",
    allow_hyphen_values = true,
    requires = "x"
  )]
  y: Option<i32>,

  /// Bottom edge of the Waybar surface in desktop coordinates.
  #[arg(long, global = true, value_name = "PX", allow_hyphen_values = true)]
  bar_bottom: Option<i32>,

  /// Right edge of the Waybar surface in desktop coordinates.
  #[arg(long, global = true, value_name = "PX", allow_hyphen_values = true)]
  bar_right: Option<i32>,

  #[command(subcommand)]
  command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
  Toggle,
  Show,
  Hide,
  Config,
  ConfigPath,
  Reload,
  Import {
    file: PathBuf,
  },
  Export {
    #[arg(short, long)]
    output: PathBuf,
  },
  Sync,
  Status,
  Service,
}

fn main() -> Result<()> {
  env_logger::init();
  let cli = Cli::parse();
  let opens_popup = matches!(
    &cli.command,
    None | Some(Command::Toggle) | Some(Command::Show)
  );
  // The Waybar launcher supplies an immutable click position. Direct CLI
  // calls retain the live-pointer fallback for backwards compatibility.
  let click_position = opens_popup
    .then(|| {
      cli
        .x
        .zip(cli.y)
        .map(|(x, y)| PopupAnchor {
          x,
          y,
          bar_bottom: cli.bar_bottom,
          bar_right: cli.bar_right,
        })
        .or_else(|| {
          ui::app::current_pointer_position().map(|(x, y)| PopupAnchor {
            x,
            y,
            bar_bottom: cli.bar_bottom,
            bar_right: cli.bar_right,
          })
        })
    })
    .flatten();
  let paths = config::resolve_paths()?;
  let settings = match config::Settings::load(&paths) {
    Ok(settings) => settings,
    Err(err) => {
      eprintln!("{err}\nUsing default configuration.");
      config::Settings::default()
    }
  };
  storage::Database::open(&paths.database)?.purge_expired_local_single_events(chrono::Utc::now())?;

  match cli.command {
    None | Some(Command::Toggle) => {
      if !ipc::notify(
        &paths,
        ipc::IpcMessage {
          command: ipc::IpcCommand::Toggle,
          position: click_position,
        },
      ) {
        ui::run_ui(
          paths,
          settings,
          ui::app::PopupCommand::Toggle,
          click_position,
        );
      }
    }
    Some(Command::Show) => {
      if !ipc::notify(
        &paths,
        ipc::IpcMessage {
          command: ipc::IpcCommand::Show,
          position: click_position,
        },
      ) {
        ui::run_ui(paths, settings, ui::app::PopupCommand::Show, click_position);
      }
    }
    Some(Command::Hide) => {
      if !ipc::notify(
        &paths,
        ipc::IpcMessage {
          command: ipc::IpcCommand::Hide,
          position: None,
        },
      ) {
        eprintln!("argvus-taskbar-calendar: no running instance to hide");
      }
    }
    Some(Command::Config) => config::open_config(&paths, &settings)?,
    Some(Command::ConfigPath) => {
      println!("{}", config::effective_config_file(&paths).display())
    }
    Some(Command::Reload) => {
      if !ipc::notify(
        &paths,
        ipc::IpcMessage {
          command: ipc::IpcCommand::ReloadTheme,
          position: None,
        },
      ) {
        eprintln!("argvus-taskbar-calendar: no running instance to reload");
      }
    }
    Some(Command::Import { file }) => app::import_file(&paths, file)?,
    Some(Command::Export { output }) => app::export_file(&paths, output)?,
    Some(Command::Sync) => app::sync_once(&paths, &settings)?,
    Some(Command::Status) => app::status(&paths)?,
    Some(Command::Service) => {
      service::run_service(paths)?;
    }
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn accepts_fixed_position_before_or_after_popup_command() {
    for args in [
      [
        "argvus-taskbar-calendar",
        "--x",
        "120",
        "--y",
        "28",
        "toggle",
      ],
      [
        "argvus-taskbar-calendar",
        "show",
        "--x",
        "-120",
        "--y",
        "28",
      ],
    ] {
      let cli = Cli::try_parse_from(args).expect("valid fixed popup position");
      assert!(matches!(cli.command, Some(Command::Toggle | Command::Show)));
      assert_eq!(
        cli
          .x
          .zip(cli.y)
          .map(|(x, y)| (x.abs(), y, cli.bar_bottom, cli.bar_right)),
        Some((120, 28, None, None))
      );
    }
  }

  #[test]
  fn rejects_incomplete_fixed_position() {
    assert!(Cli::try_parse_from(["argvus-taskbar-calendar", "toggle", "--x", "120"]).is_err());
    assert!(Cli::try_parse_from(["argvus-taskbar-calendar", "toggle", "--y", "28"]).is_err());
  }
}
