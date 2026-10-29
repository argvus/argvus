---
title: Package matrix
description: ARGVUS package responsibilities.
---

| Package | Responsibility |
| --- | --- |
| `argvus` | Master package: dispatcher, in-process commands (configuration, translations, keys, projects, terminal, system monitor, network, Bluetooth, telemetry, TUI terminal, setup), the shared data and the i18n catalogs. Every component package depends on it. |
| `argvus` (helpers) | Helpers in the same package: the Control Center (a terminal application), and the GTK 4 applications for the taskbar calendar, removable devices, the loading overlay, ARGVUS Focus and ARGVUS Clock. |
| `argvus` (greeter) | Login greeter for greetd, shipped in the same package and built from `crates/argvus-greeter`. |
| `argvus-session` | Session entry points and user lifecycle. |
| `argvus-hyprland` | Hyprland configuration. |
| `argvus-control-panel` | Quickshell control panel. |
| `argvus-taskbar`, `argvus-waybar` | Waybar configuration and patched binary. |
| `argvus-branding` | Logo and wordmark assets. |
| `argvus-appearance`, `argvus-wallpapers`, `argvus-fonts`, `argvus-icons` | Appearance state and visual assets. |
| `argvus-display`, `argvus-power`, `argvus-notifications` | Desktop providers. |
| `argvus-lock`, `argvus-boot-splash` | Lock screen and boot splash. |
| `argvus-firewall` | Firewall, with its own privileges. |
| `argvus-launcher` | Launcher configuration and entry points. |
| `argvus-app-profiles` | Data-only profiles for bundled terminal applications. |
| `argvus-portal` | XDG portal and Wayland/DBus defaults. |
| `argvus-theme-*` | One optional package per official theme family. |
| `argvus-game-snake`, `argvus-games` | Optional. Snake is an independent project; `argvus-games` groups the optional games. |

## Replaced packages

`argvus` declares `replaces`, `conflicts` and `provides` for the packages it absorbed, so an upgrade removes them and their files do not remain as orphans:

`argvus-config`, `argvus-i18n` (with `argvus-language-en-us` and `argvus-language-pt-br`), `argvus-keys`, `argvus-projects`, `argvus-terminal`, `argvus-system-monitor`, `argvus-network`, `argvus-widget-telemetry`. `argvus` also replaces `argvus-control-center`, `argvus-taskbar-calendar`, `argvus-loading-theme`, `argvus-removable-devices`, and the former split packages `argvus-gtk` and `argvus-greeter`, which it still provides so that packages depending on them keep resolving. The account manager (`argvus accounts`) is built from `crates/argvus-accounts` and installed as `/usr/lib/argvus/commands/accounts`, with its polkit rule.

`/usr/bin/argvus-terminal` stays installed as a compatibility launcher for `argvus terminal`. Hyprland's default terminal, the Control Center catalog and saved default applications still name that command.

The removed packages `argvus-pwvucontrol` and `argvus-snappy-switcher` are not provided by any package.

No install order is required: `argvus` is the master package, and every other package depends on it. A package's presence in this table does not mean it owns the whole feature: for example, the taskbar combines `argvus-waybar`, `argvus-taskbar`, `argvus-session` and domain providers.
