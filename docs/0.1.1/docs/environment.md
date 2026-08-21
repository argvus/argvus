---
title: Environment
description: How the ARGVUS desktop environment is organized.
slug: 0.1.1/docs/environment
---

ARGVUS is a complete desktop, ready to use, focused on Hyprland. The desktop configuration lives in the [`argvus`](https://github.com/argvus/argvus) repository and is packaged as the `argvus` Arch Linux package.

The package installs read-only defaults under `/usr/share/argvus` and the optional `argvus-setup` copy helper under `/usr/bin`. It never writes to `$HOME` during package installation.

## Component packages

The desktop is split into focused packages, all built by `argvus-pkgbuild` and published to the same repository:

| Package | Owns |
| --- | --- |
| `argvus` | Desktop configuration under `/usr/share/argvus` and the optional `/usr/bin/argvus-setup` helper. |
| `argvus-session` | Session launchers `/usr/bin/argvus-session`, `/usr/bin/argvus-start`, `/usr/bin/argvus-tty` and `/usr/share/wayland-sessions/argvus.desktop`. |
| `argvus-appearance` | Shared wallpapers under `/usr/share/backgrounds/argvus` and bundled fonts under `/usr/share/fonts`. |
| `argvus-storage` | The removable-storage Waybar module (binary, config and themes). |
| `argvus-calendar` | The native calendar popup for Waybar (binary, config and themes). |
| `argvus-greeter` | The greetd greeter integration. |

Installing `argvus` pulls in the other component packages as dependencies.

## Desktop composition

| Area | Files | Role |
| --- | --- | --- |
| Hyprland | `config/hypr/hyprland.lua` | Main compositor configuration, keybindings, workspaces, rules and startup. |
| Setup | `bin/argvus-setup` | Optional helper that copies packaged defaults to the user's configuration directory. |
| Waybar | `config/waybar/config.jsonc`, `config/waybar/style.css` | Top bar with workspaces, media, window context, tray, system state and ARGVUS modules. |
| Sidebar | `config/quickshell/sidebar-right/` | Quickshell panel for appearance, system, calendar, network, volume, brightness and power controls. |
| Launchers | `config/rofi/`, `config/wofi/` | Themed launchers, menus and command surfaces. |
| Terminal and TUI | `config/kitty/`, `config/yazi/`, `config/superfile/`, `config/btop/`, `config/bottom/` | Terminal, file managers and system monitors. |
| Notifications and power | `config/dunst/` | Notification styling and power controls. |

## Session flow

`argvus-session` is the display-manager entrypoint. It prepares the environment and executes `argvus-start`; it does not copy configuration to `$HOME`.

`argvus-start` starts Hyprland with the ARGVUS configuration. It uses `$XDG_CONFIG_HOME/hypr/hyprland.lua` when that complete user override exists; otherwise it uses the packaged `/usr/share/argvus/hypr/hyprland.lua`. The packaged Lua configuration also loads small optional user override files from `$XDG_CONFIG_HOME/argvus/hypr/`.

`argvus-tty` provides a manual path for starting ARGVUS on systems without a display manager. It registers a logind session, starts the D-Bus session bus, defines the Wayland/XDG environment and starts the same session.

## Configuration model

ARGVUS does not copy dotfiles to `$HOME` before the desktop can start. Runtime entrypoints use this priority:

```text
$XDG_CONFIG_HOME/<app>  ->  $XDG_STATE_HOME/argvus/config/<app>  ->  /usr/share/argvus/<app>  ->  upstream defaults
```

If `XDG_STATE_HOME` is unset, ARGVUS uses the standard fallback `~/.local/state`, so generated runtime configuration lives under `~/.local/state/argvus/config`.

The responsibility of each location is:

| Location | Responsibility |
| --- | --- |
| `/usr/share/argvus` | Immutable packaged defaults owned by pacman. Package upgrades may replace these files. |
| `$XDG_STATE_HOME/argvus/config` | Generated runtime configuration rebuilt from the current packaged defaults plus ARGVUS preferences. |
| `$XDG_CONFIG_HOME/argvus` | Small user preferences such as `.active-theme`, `.accent-color`, `.spaces`, `.weather-location` and optional Hyprland Lua overrides. |
| `$XDG_CONFIG_HOME/<app>` | Explicit complete-application user overrides created manually or with `argvus-setup`. |
| `$XDG_CACHE_HOME/argvus` | Runtime cache such as the generated Hyprlock wallpaper. |

`argvus-setup` is optional. Use it only when you want to copy packaged defaults into `$XDG_CONFIG_HOME` for customization:

| Mode | Behavior |
| --- | --- |
| `--copy <app>` | Copies `/usr/share/argvus/<app>` to `$XDG_CONFIG_HOME/<app>`. |
| `--copy-all` | Copies all packaged defaults. |
| `--force` | Backs up the existing destination as `$XDG_CONFIG_HOME/<app>.bak-<timestamp>` before copying. |
| `--dry-run` | Shows actions without changing files. |
| `--repair` | Compatibility alias for `--copy argvus`. |

Once copied, files in `$XDG_CONFIG_HOME` are user-owned overrides and package upgrades never replace them.

## Hyprland

The main compositor file is `config/hypr/hyprland.lua`. It loads the active theme, accent color and spacing before defining the session.

Important defaults:

* `SUPER` is the main modifier.
* The layout is `dwindle`, with groups enabled.
* Keyboard layout is `br,us` with `grp:alt_shift_toggle`.
* Firefox runs natively on Wayland through `MOZ_ENABLE_WAYLAND=1`.
* Qt apps use the configured theme and Hyprland Qt style when available.
* XWayland remains enabled for compatibility.

Optional Hyprland user overrides live under `$XDG_CONFIG_HOME/argvus/hypr/` and are loaded after packaged defaults in this order:

```text
monitors.lua
rules.lua
bindings.lua
user.lua
```

Missing override files are ignored.

Hyprland helper scripts live in `config/hypr/scripts/` and cover screenshots, power menu, wallpaper selection, cheatsheets, Waybar startup and the ARGVUS Storage menu anchor.

## Waybar

ARGVUS uses Waybar as the main top bar. The default layout groups workspaces and media on the left, window context in the center and system modules on the right.

Important modules:

* `hyprland/workspaces` for workspace navigation.
* `mpris` for media.
* `network`, `memory`, `cpu`, CPU/GPU temperature scripts and power profile.
* `custom/storage`, fed by `argvus-storage watch`.
* `clock#date`, which toggles the native calendar popup (`argvus-calendar`) on click.
* `pulseaudio#input` and `pulseaudio#output` with click and scroll actions.
* `tray` for applications with status icons.
* `custom/settings`, which toggles the Quickshell sidebar.

System detail scripts live in `config/waybar/scripts/sysinfo/`; the auxiliary view is configured by `config/waybar/sysinfo.jsonc` and `config/waybar/sysinfo.css`.

## Quickshell sidebar

The right sidebar lives in `config/quickshell/sidebar-right`. It is a QML interface for repeated desktop actions.

It includes cards for:

* appearance and themes;
* brightness and volume;
* calendar;
* keyboard layout;
* network;
* notifications;
* power;
* workspace spacing;
* system information;
* weather.

The Waybar settings button toggles this sidebar through the ARGVUS taskbar script.

## Themes

ARGVUS ships eight theme families:

* `argvus-dark-aether`
* `argvus-dark-aether-aether-float`
* `argvus-dark-silver`
* `argvus-dark-silver-float`
* `argvus-light-veil`
* `argvus-light-veil-float`
* `argvus-dark-aether-slate`
* `argvus-dark-aether-slate-float`

Themes are mirrored across Hyprland, Waybar, Rofi, Dunst, Kitty, Btop, Bottom, Wlogout, Yazi, Superfile, Snappy Switcher and Qt color schemes. The active theme is applied by `config/argvus/sh/theme-switch.sh`.

Accent color is controlled separately by `config/argvus/sh/accent-switch.sh`, so the user can change the accent without replacing the theme family.

Shared shell modules live in `config/argvus/sh/` and are loaded by `bootstrap.sh`, which exposes logging, path, notification, JSON and Hyprland helpers to scripts.

Wallpapers and bundled fonts are owned by the `argvus-appearance` package and referenced from system paths such as `/usr/share/backgrounds/argvus/`.

## Packaged paths

The `argvus` package installs:

```text
/usr/bin/argvus-setup
/usr/share/argvus/
```

The `argvus-session` package installs session entrypoints:

```text
/usr/bin/argvus-session
/usr/bin/argvus-start
/usr/bin/argvus-tty
/usr/share/wayland-sessions/argvus.desktop
```

`argvus-storage` and `argvus-calendar` are packaged separately and install their own binaries and system defaults. See [Official Apps](./official-apps/), [Calendar](./calendar/) and [Removable Storage](./storage/).
