---
title: Architecture overview
description: How ARGVUS forms an integrated desktop environment.
---

ARGVUS has five runtime layers:

1. entry points and dispatch in `argvus`;
2. session lifecycle in `argvus-session`;
3. compositor and shell surfaces in `argvus-hyprland`, `argvus-waybar`, `argvus-taskbar` and `argvus-control-panel`;
4. user applications such as Control Center, calendar and system monitor;
5. providers and assets for display, network, power, appearance, accounts and notifications.

Shared state connects these layers. `argvus-config` holds the canonical document and is the only writer of `data/generated/`, so a theme or effects change regenerates the files consumed by several packages in one transaction; session services provide lifecycle ownership without taking ownership of every component's implementation.

## Responsibility boundaries

| Layer | Primary responsibility | Examples |
| --- | --- | --- |
| Entry points and dispatch | Start desktop applications and resolve ARGVUS profiles. | `argvus`, `argvus-app-profiles` |
| Session lifecycle | Prepare the environment, launch Hyprland, synchronize the user service environment and own session shutdown. | `argvus-session`, `argvus-sessionctl` |
| Compositor and shell | Define window behavior and visible desktop surfaces. | `argvus-hyprland`, `argvus-waybar`, `argvus-taskbar`, `argvus-control-panel` |
| Configuration | Own `config.json`, the path model and every generated consumer file. | `argvus-config` |
| Applications | Provide settings and focused desktop workflows. | Control Center, calendar, launcher, system monitor and removable devices |
| Providers and assets | Implement domain operations and shared visual resources. | display, network, power, notifications, appearance, fonts and wallpapers |

The `argvus` package is the installation target and coordinator, but it is not a monolithic runtime. Focused packages keep ownership of their binaries, configuration and services. The session manager starts them at the appropriate point in the graphical session without absorbing their implementation.

This distinction matters when debugging: a session failure belongs first to `argvus-session`; a taskbar failure belongs to `argvus-taskbar`/`argvus-waybar`; and a generated theme or wallpaper failure belongs to the appearance and session configuration path.
