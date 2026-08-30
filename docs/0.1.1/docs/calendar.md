---
title: Calendar
description: The native ARGVUS calendar popup for Waybar.
slug: 0.1.1/docs/calendar
---

`argvus-calendar` is the native ARGVUS calendar popup for Wayland/Hyprland, written in Rust with Relm4 and GTK4 and positioned with `gtk4-layer-shell`. It opens from the Waybar date module and provides a compact month view, agenda, event editor and reminders.

## Opening

Click the date in the Waybar top bar. The module is configured as `clock#date` with `"on-click"` bound to the ARGVUS taskbar calendar action, which runs:

```sh
argvus-calendar toggle
```

The popup is a single-instance layer-shell surface: it stays running after hiding, so `toggle`, `show` and `hide` go through a UNIX socket IPC at `$XDG_CACHE_HOME/argvus-calendar/argvus-calendar.sock`. It closes when you click anywhere outside it, click the date again, press Escape, or when it loses focus.

## Commands

```sh
argvus-calendar                 # toggle the popup
argvus-calendar show            # show the popup
argvus-calendar hide            # hide the popup
argvus-calendar config          # open the config file in a terminal with sudo
argvus-calendar config-path     # print the config path
argvus-calendar reload          # reload the external CSS/theme on the running popup
argvus-calendar import file.ics # import an ICS file
argvus-calendar export --output calendar.ics
argvus-calendar sync            # run a sync with the configured CalDAV/WebDAV account
argvus-calendar status          # print database/config/style/theme status
argvus-calendar service         # run the reminder scheduler
```

## Configuration

All configuration lives in `/etc/argvus-calendar/config.toml`. The application uses built-in defaults when the file is missing. Open it for editing with `argvus-calendar config`, which launches a terminal with `sudo` (terminal from `$TERMINAL` or the `[terminal]` setting, editor from `$VISUAL`/`$EDITOR` or the `[editor]` setting, default `nano`).

```toml
[appearance]
font_family = "monospace"
font_size = 12 # valid range: 8-32

[locale]
language = "en-US" # en-US | pt-BR

[calendar]
week_start = "monday" # monday | sunday
default_event_duration_minutes = 60
default_reminder_minutes = 10
sync_interval_minutes = 15

[popup]
enable = false # false opens below the Waybar click/cursor
anchor = "top-right" # used only when enable = true
margin_top = 8
margin_right = 10
margin_bottom = 8
margin_left = 8

[editor]
command = "" # falls back to $VISUAL, then $EDITOR, then nano
args = []

[terminal]
command = "" # falls back to $TERMINAL, then kitty
args = []
```

The gear button in the popup opens the same config file. The popup reloads the config on each restart; `argvus-calendar reload` refreshes the external CSS/theme on the running popup.

## Paths

```text
/etc/argvus-calendar/config.toml    system-wide config (the package ships a default)
/etc/argvus-calendar/style.css      popup structure styles
/etc/argvus-calendar/theme.css      packaged default theme
/etc/argvus-calendar/themes/        ARGVUS theme files
$XDG_DATA_HOME/argvus-calendar/     SQLite database
$XDG_STATE_HOME/argvus-calendar/    runtime state
$XDG_CACHE_HOME/argvus-calendar/    IPC socket, events toggle and active theme
```

## Events and reminders

Events are stored in a SQLite database. Event reminders can be disabled or configured in hours and minutes before the start; all-day events can additionally repeat their notification at a chosen interval during the day. Events that ended before the current local day are permanently removed, and new events cannot be created on past dates.

Reminders are reliable while `argvus-calendar service` is running, or when the provided user systemd service is enabled:

```sh
systemctl --user enable --now argvus-calendar.service
```

The events section is controlled by the in-window toggle and persisted in `$XDG_CACHE_HOME/argvus-calendar/events-enabled`; it is not configured in `config.toml`.

ICS import and export are supported through `argvus-calendar import` and `argvus-calendar export`. CalDAV support is implemented as a maintained internal HTTP/XML client foundation; account management and full database reconciliation are the next integration step.

## Styling

Styling is external: `/etc/argvus-calendar/style.css` provides structure, `/etc/argvus-calendar/theme.css` provides the packaged default, `/etc/argvus-calendar/themes/` contains the ARGVUS themes, and `$XDG_CACHE_HOME/argvus-calendar/theme.css` is the user-level active theme written by the ARGVUS theme switcher.

Supported ARGVUS themes: Dark, Dark Float, Dark Silver, Dark Silver Float, Slate and Slate Float. The popup follows the desktop theme family and reads the active theme from `$XDG_CONFIG_HOME/argvus/.active-theme`.

## Package

`argvus-calendar` is packaged from [`argvus-pkgbuild`](https://github.com/argvus/argvus-pkgbuild) at `arch/argvus-calendar/PKGBUILD`. It installs `/usr/bin/argvus-calendar`, the defaults under `/etc/argvus-calendar/` and the user systemd unit at `/usr/lib/systemd/user/argvus-calendar.service`.
