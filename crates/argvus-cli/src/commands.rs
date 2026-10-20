//! The closed tables of the dispatcher. Only the subcommands listed here reach a
//! private executable or a packaged script, so no path is ever built from user
//! input.

use crate::exec::{Failure, env_nonempty, find_on_path, is_executable};
use std::path::PathBuf;

/// Directory of the private executables, unless `ARGVUS_COMMANDS_DIR` overrides it.
const DEFAULT_COMMANDS_DIR: &str = "/usr/lib/argvus/commands";

/// Private executables that routes start by name, outside the generic table.
pub const CONTROL_CENTER: &str = "control-center";
pub const GAME_SNAKE: &str = "game-snake";
/// The loading overlay. It stays at this path: the greeter and the session config
/// launch it from here, so the route does not move it into the commands directory.
pub const LOADING_THEME_BINARY: &str = "/usr/lib/argvus/loading-theme/splash";
pub const TASKBAR_CALENDAR: &str = "taskbar-calendar";
pub const REMOVABLE_DEVICES: &str = "removable-devices";

macro_rules! private_commands {
  ($($variant:ident => $subcommand:literal => $executable:literal,)+) => {
    /// A private executable reached through `argvus <subcommand>`.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum PrivateCommand {
      $($variant,)+
    }

    impl PrivateCommand {
      /// The private command for a subcommand name, or `None` when the name is
      /// not part of the table.
      pub fn from_subcommand(subcommand: &str) -> Option<Self> {
        match subcommand {
          $($subcommand => Some(Self::$variant),)+
          _ => None,
        }
      }

      /// The file name of the executable, as installed in the commands directory.
      pub fn executable_name(self) -> &'static str {
        match self {
          $(Self::$variant => $executable,)+
        }
      }
    }
  };
}

private_commands! {
  Session => "session" => "session",
  Start => "start" => "start",
  Tty => "tty" => "tty",
  Sessionctl => "sessionctl" => "sessionctl",
  Greeter => "greeter" => "greeter",
  GreeterSession => "greeter-session" => "greeter-session",
  GreeterTui => "greeter-tui" => "greeter-tui",
  GreeterSetup => "greeter-setup" => "greeter-setup",
  Accounts => "accounts" => "accounts",
  Firewall => "firewall" => "firewall",
  TaskbarCalendar => "taskbar-calendar" => "taskbar-calendar",
  RemovableDevices => "removable-devices" => "removable-devices",
  Notifications => "notifications" => "notifications",
  Displayctl => "displayctl" => "displayctl",
  Launcher => "launcher" => "launcher",
  Snippets => "snippets" => "snippets",
  GameSnake => "game-snake" => "game-snake",
  TaskbarActions => "taskbar-actions" => "taskbar-actions",
  Ssh => "ssh" => "ssh",
  Git => "git" => "git",
  AppProfilesYaziKeymap => "app-profiles-yazi-keymap" => "app-profiles-yazi-keymap",
}

macro_rules! script_routes {
  ($($variant:ident => $name:literal => $relative:literal,)+) => {
    /// A Hyprland script command, routed to a script under the ARGVUS data root.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum ScriptRoute {
      $($variant,)+
    }

    impl ScriptRoute {
      /// The route for a script command name, or `None` when the name is not routed.
      pub fn from_name(name: &str) -> Option<Self> {
        match name {
          $($name => Some(Self::$variant),)+
          _ => None,
        }
      }

      /// The script path relative to the ARGVUS data root (`<domain>/sh/<script>.sh`).
      pub fn relative_path(self) -> &'static str {
        match self {
          $(Self::$variant => $relative,)+
        }
      }
    }
  };
}

script_routes! {
  CheatsheetsHypr => "cheatsheets-hypr" => "hyprland/sh/cheatsheets.sh",
  CheatsheetsLauncher => "cheatsheets-launcher" => "launcher/sh/cheatsheets.sh",
  Scratchpad => "scratchpad" => "hyprland/sh/scratchpad.sh",
  HyprScreenshot => "hypr-screenshot" => "hyprland/sh/hypr-screenshot.sh",
  SpacesSwitch => "spaces-switch" => "hyprland/sh/spaces-switch.sh",
  EmojiPicker => "emoji-picker" => "launcher/sh/emoji-picker.sh",
  ToggleSidebar => "toggle-sidebar" => "control-panel/sh/toggle-sidebar.sh",
  WeatherLocation => "weather-location" => "control-panel/sh/weather-location.sh",
  LayoutModeMenu => "layout-mode-menu" => "appearance/sh/layout-mode-menu.sh",
  BrightnessSwitch => "brightness-switch" => "appearance/sh/brightness-switch.sh",
  ToggleMode => "toggle-mode" => "appearance/sh/toggle-mode.sh",
  IdleTimeout => "idle-timeout" => "power/sh/idle-timeout.sh",
  KeepAwake => "keep-awake" => "power/sh/keep-awake.sh",
  HyprPowerMenu => "hypr-power-menu" => "power/sh/hypr-power-menu.sh",
}

/// The directory of the private executables. `ARGVUS_COMMANDS_DIR` overrides it
/// for tests and DESTDIR checks.
pub fn commands_dir() -> PathBuf {
  env_nonempty("ARGVUS_COMMANDS_DIR")
    .map(PathBuf::from)
    .unwrap_or_else(|| PathBuf::from(DEFAULT_COMMANDS_DIR))
}

/// Finds an executable for `name`. The commands directory wins. The PATH fallback
/// only looks for the `argvus-<name>` file that older packages installed in
/// `/usr/bin`: a bare name such as `git` or `ssh` must never run an unrelated
/// program from PATH.
pub fn resolve_executable(name: &str) -> Option<PathBuf> {
  let installed = commands_dir().join(name);
  if is_executable(&installed) {
    return Some(installed);
  }
  find_on_path(&format!("argvus-{name}"))
}

/// Like `resolve_executable`, but a missing program is a failure with a message.
pub fn required_executable(name: &str) -> Result<PathBuf, Failure> {
  resolve_executable(name).ok_or_else(|| Failure::new(format!("{name} is not installed")))
}
