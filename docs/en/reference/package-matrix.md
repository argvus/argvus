---
title: Package matrix
description: ARGVUS package responsibilities.
---

| Package | Responsibility |
| --- | --- |
| `argvus` | Dispatcher and package entry point. |
| `argvus-session` | Session entry points and user lifecycle. |
| `argvus-config` | Canonical logical user configuration, the only writer of `data/generated/`, and profile import/export. |
| `argvus-hyprland` | Hyprland configuration. |
| `argvus-control-center` | TUI settings application. |
| `argvus-control-panel` | Quickshell control panel. |
| `argvus-taskbar`, `argvus-waybar` | Waybar configuration and patched binary. |
| `argvus-appearance`, `argvus-wallpapers`, `argvus-fonts`, `argvus-icons` | Appearance state and visual assets. |
| `argvus-network`, `argvus-display`, `argvus-power`, `argvus-notifications` | Desktop providers. |
| `argvus-greeter`, `argvus-lock`, `argvus-splash`, `argvus-theme-splash` | Login, lock and startup visuals. |
| `argvus-i18n`, `argvus-tui` | Shared libraries and catalogs. |

## Additional package boundaries

| Package | Responsibility |
| --- | --- |
| `argvus-accounts` | Local account metadata and avatars. |
| `argvus-taskbar-calendar` | Calendar popup and event integration. |
| `argvus-games` | Meta-package for the official ARGVUS games; depends on `argvus-game-snake`. |
| `argvus-game-snake` | Retro terminal Snake game and local ranking. |
| `argvus-removable-devices` | UDisks2 storage actions and taskbar/menu integration. |
| `argvus-launcher` | Launcher configuration and entry points. |
| `argvus-terminal`, `argvus-system-monitor` | Terminal and system-monitor profiles. |
| `argvus-firewall` | System firewall service and Control Center integration. |

The `argvus` package depends on the component packages for a complete desktop. A package's presence in this table does not mean it owns the whole feature: for example, the taskbar combines `argvus-waybar`, `argvus-taskbar`, `argvus-session` and domain providers.
