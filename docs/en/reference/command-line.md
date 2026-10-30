---
title: Command line
description: The argvus command, its subcommands, the flags kept for keybindings, and where each one runs.
---

`/usr/bin/argvus` is the public entry point of ARGVUS. It is one Rust program built from the `argvus-cli` crate. Each subcommand either runs inside that program, runs a helper executable from the commands directory, or runs a packaged script.

Run `argvus --help` for the list installed with your version. The subcommands are written without the leading `--`. The `--` forms are kept only where a keybinding or a desktop entry uses them.

## Where each command runs

- **In-process:** the command is a library of the workspace and runs inside `argvus`. No separate executable exists. These are `config`, `theme`, `i18n`, `keys`, `gpg`, `projects`, `terminal`, `system-monitor`, `network`, `bluetooth`, `focus status`, `widget-telemetry`, `tui-terminal`, `sensors` and `setup`.
- **Helper:** a separate executable in `/usr/lib/argvus/commands/`, installed by the `argvus` package. These are `control-center` (a terminal application) and the GTK 4 applications `taskbar-calendar`, `removable-devices`, `focus` and `clock`.
- **Private executable:** an executable that another package installs in `/usr/lib/argvus/commands/`. `argvus` only starts it. These include `session`, `start`, `tty`, `sessionctl`, `launcher`, `snippets`, `ssh`, `git`, `app-profiles-yazi-keymap`, `notifications`, `displayctl`, `firewall`, `accounts`, `taskbar-actions` and the `greeter-*` entry points.
- **Script:** a packaged shell script run with `/bin/sh`. These are the Hyprland script commands listed below.

`ARGVUS_COMMANDS_DIR` changes the commands directory, for tests and `DESTDIR` checks. A private executable that is not found there is looked up on `PATH`.

## Subcommands

| Command | Runs | Notes |
| --- | --- | --- |
| `argvus config ...` | In-process | Configuration model and tools. Also reached by `theme-switch` and `effects-toggle`. |
| `argvus theme ...` | In-process | `apply ID` is the theme switch (the same as `theme-switch`). Theme builds: `list` the installed themes, `check DIRECTORY` a theme against the installed templates, `render ID [--variant sticky\|float] [--accent #RRGGBB] [--output DIRECTORY]`. See [Appearance and themes](/docs/developer-guide/subsystems/appearance-and-themes/). |
| `argvus i18n ...` | In-process | Translations: `get DOMAIN KEY [name=value...]`, `locale`, `list`, `validate [LOCALE]`. |
| `argvus keys ...` | In-process | SSH key manager. `gpg` is an alias for the GPG subcommands. |
| `argvus gpg ...` | In-process | GPG keys: `list`, `status`, `generate`, `export-public`, `export-private`, `import`, `import-private`, `delete`, `edit`, `keyserver`. |
| `argvus projects ...` | In-process | Project picker. |
| `argvus terminal ...` | In-process | The ARGVUS terminal profile. |
| `argvus system-monitor ...` | In-process | The btop system monitor; `--panel [--cpu|--mem]` embeds it in the ARGVUS TUI shell. |
| `argvus network ...` | In-process | NetworkManager status and controls (`networkctl` is kept as an alias). |
| `argvus bluetooth ...` | In-process | Bluetooth status and controls (`bluetoothctl` is kept as an alias). |
| `argvus widget-telemetry block ...` | In-process | The Waybar telemetry block. This is the hot path of the bar. |
| `argvus widget-telemetry toggle ...` | In-process | Switches the telemetry blocks on and off, and their order. |
| `argvus game-snake` | Private executable | The terminal Snake game, from the independent `argvus-game-snake` project, an optional package. Outside a terminal it opens one. |
| `argvus control-center [page]` | Helper (terminal application) | Opens the Control Center, or focuses the window that is already open. |
| `argvus taskbar-calendar [args]` | GTK 4 application | Defaults to `toggle`. |
| `argvus removable-devices [args]` | GTK 4 application | Defaults to `menu`. |
| `argvus focus [--toggle] [--start\|--pause] [--x X --y Y] [--bar-bottom PX --bar-right PX]` | GTK 4 application | ARGVUS Focus popup. One instance per session. |
| `argvus focus status --waybar [--follow]` | In-process | Emits the Focus taskbar indicator as Waybar JSON; `--follow` streams changes without loading GTK. |
| `argvus clock [--toggle] [--background] [--page PAGE] [--x X --y Y] [--bar-bottom PX --bar-right PX]` | GTK 4 application | ARGVUS Clock popup. One instance per session; `--background` keeps alarms active without a popup. |
| `argvus loading-theme ...` | Packaged `splash` at `/usr/lib/argvus/loading-theme/splash` | The overlay of direct session reloads; a theme switch does not use it. |
| `argvus session`, `start`, `tty`, `sessionctl` | Private executables | Session entry points and the session controller. |
| `argvus greeter`, `greeter-session`, `greeter-tui`, `greeter-setup` | Private executables | Greeter entry points. |
| `argvus accounts`, `firewall` | Private executables | Accounts and firewall tools. |
| `argvus launcher`, `snippets`, `ssh`, `git` | Private executables | Launcher and picker tools. |
| `argvus tui-terminal` | In-process | Terminal wrapper for TUI applications. |
| `argvus sensors` | In-process | Live temperature overview from `sensors -j`. |
| `app-profiles-yazi-keymap` | Private executable | Keymap helper. |
| `argvus notifications`, `displayctl`, `taskbar-actions` | Private executables | Notifications, display and taskbar actions. |
| `argvus setup [options]` | In-process | Copies packaged defaults into the user configuration. See below. |

Arguments are forwarded unchanged, including spaces and empty values. Standard input, output, error, signals and the exit status belong to the command that runs. An unknown name fails with a hint to run `argvus --help`. A command that runs in-process refuses an argument that is not valid UTF-8 (exit status 2) instead of passing an altered value on; commands that run a separate program receive the bytes unchanged.

## Flags and aliases

These names exist so that keybindings, desktop entries and older scripts keep working. Prefer the subcommand form in new code.

| Flag or alias | Equivalent |
| --- | --- |
| `--control-center`, `--about`, `--default-apps` | `control-center` (with `about` and `apps` pages) |
| `--displays`, `--monitors` page | `control-center displays` |
| `--network-interfaces` | `control-center network interfaces` |
| `--system-monitor`, `--terminal`, `--game-snake` | `system-monitor`, `terminal`, `game-snake` |
| `--calendar`, `--removable-devices`, `--storage` | `taskbar-calendar`, `removable-devices` |
| `--focus`, `--clock` | `focus`, `clock` |
| `--spf`, `--superfile` | the file manager (`spf`) |
| `--yazi`, `--yazy` | the file browser (`yazi`) |
| `--setup` | `setup` |
| `--start-desktop` | starts `tty` (no arguments are forwarded) |
| `--version`, `-V`, `version` | prints `ARGVUS <version>` |
| `help`, `--help`, `-h` | prints the usage |
| `networkctl`, `bluetoothctl` | `network`, `bluetooth` |
| `widget-telemetry-toggle`, `widget-telemetry-block` | `widget-telemetry toggle`, `widget-telemetry block` |
| `theme-switch` | `config theme-apply` |
| `effects-toggle` | `config effects` |

The Control Center accepts legacy page flags as its first argument and maps them to page names. Examples: `--apps` becomes `apps`, `--keyboard-layout` becomes `locale`, and `--hostname` becomes `system`.

## Hyprland script commands

These run packaged scripts and are used by the Hyprland configuration:

`cheatsheets-hypr`, `cheatsheets-launcher`, `scratchpad`, `hypr-screenshot`, `spaces-switch`, `emoji-picker`, `toggle-sidebar`, `weather-location`, `layout-mode-menu`, `brightness-switch`, `toggle-mode`, `idle-timeout`, `keep-awake`, `hypr-power-menu`.

A script in the user's configuration overrides the packaged one. The lookup order is the same as `_config_path` in `hyprland.lua`: the user's config copy, then the ARGVUS data copy, then the state copy, then the generated copy, and only then the packaged file under `/usr/share/argvus`.

## Examples

```sh
argvus control-center displays
argvus taskbar-calendar toggle
argvus widget-telemetry block header system
argvus widget-telemetry toggle status
argvus i18n get widget-telemetry label.device
argvus config get /appearance/theme
argvus network status
argvus bluetooth status
argvus gpg list
```

## Setup

`argvus setup` copies packaged defaults from `/usr/share/argvus/<app>` into `~/.config/argvus/data/<app>`, so that you can customize them. It never edits the packaged files.

```sh
argvus setup --copy <app>          # copy one packaged app
argvus setup --copy-all            # copy every packaged app directory
argvus setup --copy <app> --force  # back up the existing destination first
argvus setup --copy <app> --dry-run
```

A destination that already exists is reported and left alone, unless `--force` is given. With `--force`, the existing directory is renamed to `<name>.bak-<timestamp>` before the copy.

The `argvus` app is special: it copies the contents of its packaged directory into the configuration root. Because the configuration root already exists when setup runs, `--copy argvus` needs `--force` in the current code.

Setup validates every option before it changes anything. Invalid app names (empty, starting with `.`, or containing `/`) are rejected.

## Session and reload

`SUPER + SHIFT + R` runs `argvus sessionctl reload`. The session controller applies the generated configuration and restarts only the components it owns. It does not copy `/usr/share/argvus` into the home directory, and it does not replace a malformed JSON file.

## Night Light

Open **Desktop & Input > Displays > Night Light** or run `argvus control-center displays night-light`. Configure temperature, manual activation and fixed local schedules through `argvus config display night-light`. See [Night Light](/docs/user-guide/night-light/) for commands, state and runtime requirements.
