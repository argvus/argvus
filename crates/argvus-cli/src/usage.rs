//! The help text of `argvus`. It lists the public subcommands and their aliases,
//! and it is printed by `--help` and by `--setup --help`.

/// Printed for `help`, `--help`, `-h` and when no subcommand is given.
pub const TEXT: &str = r#"Usage:
  argvus --start-desktop
  argvus --setup [setup options]
  argvus --system-monitor [btop options]
  argvus --control-center [page]
  argvus --terminal [terminal options]
  argvus --spf [superfile options]
  argvus --yazi [yazi options]
  argvus --about [about tab]
  argvus --calendar [argvus taskbar-calendar options]
  argvus --game-snake
  argvus --default-apps [control center Apps options]
  argvus --removable-devices [argvus removable-devices options]
  argvus --displays [control center Displays page]
  argvus --network-interfaces
  argvus --version
  argvus help | --help | -h
  argvus <command> [arguments...]

Commands:
  argvus sessionctl | session | start | tty
  argvus greeter | greeter-session | greeter-tui | greeter-setup
  argvus config | i18n | accounts | firewall
  argvus taskbar-calendar | removable-devices | notifications
  argvus networkctl | bluetoothctl | displayctl | widget-telemetry-toggle
  argvus launcher | projects | snippets | ssh | keys | gpg | git
  argvus tui-terminal | app-profiles-yazi-keymap | game-snake | terminal

Hyprland script commands (used by the Hyprland configuration):
  argvus cheatsheets-hypr | cheatsheets-launcher | scratchpad | hypr-screenshot
  argvus spaces-switch | emoji-picker | toggle-sidebar | weather-location
  argvus theme-switch | layout-mode-menu | brightness-switch | toggle-mode
  argvus idle-timeout | keep-awake | hypr-power-menu | effects-toggle

Aliases:
  argvus setup | system-monitor | control-center | terminal | spf | yazi | about | calendar | game-snake | default-apps | removable-devices | displays | version

Setup options:
  --copy <app>   Copy /usr/share/argvus/<app> to ~/.config/argvus/data/<app>.
  --copy-all     Copy all packaged defaults for explicit customization.
  --force        Replace an existing destination after backing it up.
  --repair       Backward-compatible alias for --copy argvus.
  --dry-run      Print actions without changing files.
"#;
