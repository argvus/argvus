---
title: Official Apps
description: ARGVUS-maintained commands and applications.
---

Official ARGVUS applications and commands use the `argvus-` prefix. They are maintained as part of the ARGVUS desktop rather than treated as generic third-party utilities.

## Desktop commands

The `argvus` package ships `argvus-setup`:

| Command | Role |
| --- | --- |
| `argvus-setup` | Optional helper that copies packaged defaults from `/usr/share/argvus` to `$XDG_CONFIG_HOME/<app>` only when complete user customization is desired. |

The session launchers are owned by the separate [`argvus-session`](https://github.com/argvus/argvus-session) package:

| Command | Role |
| --- | --- |
| `argvus-session` | Display-manager wrapper that prepares the environment and starts the session. |
| `argvus-start` | Starts the ARGVUS runtime and Hyprland session. |
| `argvus-tty` | Manual TTY launcher for systems without a display manager. |

## ARGVUS Calendar

`argvus-calendar` is an official Rust application maintained in [`argvus/argvus-calendar`](https://github.com/argvus/argvus-calendar). It renders a native GTK4 calendar popup as a layer-shell surface, opened from the Waybar date module, with SQLite storage, ICS import/export and configurable reminders.

Installed layout:

```text
/usr/bin/argvus-calendar
/etc/argvus-calendar/config.toml
/etc/argvus-calendar/style.css
/etc/argvus-calendar/theme.css
/etc/argvus-calendar/themes/
/usr/lib/systemd/user/argvus-calendar.service
```

The `argvus` package depends on `argvus-calendar`, and the calendar package owns its binary, config and themes.

See [Calendar](./calendar/) for usage, configuration and commands.

## ARGVUS Storage

`argvus-storage` is an official Rust application maintained in [`argvus/argvus-storage`](https://github.com/argvus/argvus-storage). It watches removable storage through UDisks2 and feeds the Waybar `custom/storage` module.

Installed layout:

```text
/usr/bin/argvus-storage
/etc/argvus-storage/config.json
/etc/argvus-storage/theme.css
/etc/argvus-storage/themes/
/usr/share/licenses/argvus-storage/LICENSE
```

User overrides live in:

```text
$XDG_CONFIG_HOME/argvus-storage/config.json
$XDG_CONFIG_HOME/argvus-storage/theme.css
$XDG_CONFIG_HOME/argvus-storage/themes/
```

The ARGVUS environment does not vendor these files anymore. The `argvus` package depends on `argvus-storage`, and the storage package owns its binary, default config and themes.

See [Removable Storage](./storage/) for command usage and configuration keys.

## ARGVUS Appearance

[`argvus-appearance`](https://github.com/argvus/argvus-appearance) ships the shared visual assets used by the desktop:

```text
/usr/share/backgrounds/argvus/
/usr/share/fonts/
```

Keeping these assets in a separate package avoids duplicating wallpapers and fonts inside the `argvus` configuration package.

## Internal shell helpers

The environment also includes shell helpers under `config/argvus/sh/`. They are part of the desktop implementation and are called by keybindings, Waybar modules and the sidebar.

Common helpers include:

- `theme-switch.sh` for applying a theme family across desktop components.
- `accent-switch.sh` for changing the accent color.
- `spaces-switch.sh` for workspace gaps and spacing modes.
- `brightness-switch.sh` for brightness actions.
- `weather-location.sh` for the weather module.
- `toggle-mode.sh` for mode switching.

These helpers are not packaged as public `/usr/bin/argvus-*` commands. They are installed as configuration support files under `/usr/share/argvus` and run from the packaged defaults during a normal session. Use `argvus-setup --copy argvus` only when you explicitly want a complete user-owned copy for customization.
