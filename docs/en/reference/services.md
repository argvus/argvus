---
title: Services
description: ARGVUS systemd and user services.
---

The main user target is `argvus-session.target`. Common user services include `argvus-taskbar.service`, `argvus-control-panel.service`, `argvus-widget-telemetry.service`, `argvus-dunst.service`, `argvus-wallpaper.service`, `argvus-hypridle.service`, clipboard services and `argvus-session-loading.service`.

The firewall is a system service named `argvus-firewall.service`. The calendar package installs `argvus-taskbar-calendar.service` as a user unit.

Inspect active units with:

```sh
systemctl --user status argvus-session.target
systemctl --user list-units 'argvus-*.service'
```

## Session-owned units

The session package installs these user units under `/usr/lib/systemd/user` (the source payload is under the session package's shared configuration tree):

| Unit | Role |
| --- | --- |
| `argvus-session.target` | Lifetime boundary for the ARGVUS graphical session. |
| `argvus-session-prepare.service` | One-shot startup preparation and generated configuration. |
| `argvus-control-panel.service` | Quickshell Control Panel. |
| `argvus-taskbar.service` | Main ARGVUS taskbar. |
| `argvus-widget-telemetry.service` | Optional telemetry/sysinfo surface. |
| `argvus-wallpaper.service` | Active wallpaper backend. |
| `argvus-dunst.service` | Dunst notification daemon. |
| `argvus-hypridle.service` | Idle and lock policy. |
| `argvus-clipboard-text.service` / `argvus-clipboard-image.service` | Clipboard history watchers. |
| `argvus-keyboard-layout.service` | Keyboard-layout notifications. |
| `argvus-polkit.service` | PolicyKit agent integration. |
| `argvus-session-loading.service` | Session-loading overlay during compositor startup. |

`argvus-blueman-applet.service` and `argvus-snappy-switcher.service` are also shipped by the session package. Some units are optional or started conditionally; they are not all active in every session.

To inspect a unit's effective definition rather than only its source checkout:

```sh
systemctl --user cat argvus-session.target
systemctl --user status argvus-session.target
journalctl --user -u argvus-session.target -u 'argvus-*.service'
```

The system-level `argvus-firewall.service` belongs to `argvus-firewall`. The calendar package separately ships `argvus-taskbar-calendar.service` as a user unit.

## Session management commands

The `argvus-sessionctl` command manages the session lifecycle and can restart specific components without logging out:

### Session lifecycle

```sh
argvus-sessionctl start               # Start a new session
argvus-sessionctl stop                # Stop the current session
argvus-sessionctl restart             # Restart the entire session
argvus-sessionctl reload              # Reload configuration and active services
argvus-sessionctl status              # Show session status
argvus-sessionctl logs                # Display session logs
```

### Component-specific restarts

Restart individual services without affecting the entire session:

```sh
# Desktop surfaces
argvus-sessionctl restart waybar
argvus-sessionctl restart wallpaper
argvus-sessionctl restart shell

# Notifications and switcher
argvus-sessionctl restart dunst snappy-switcher

# Clipboard and input
argvus-sessionctl restart clipboard keyboard-layout

# Bluetooth applet (optional)
argvus-sessionctl restart blueman-applet
```

### Environment synchronization

Import environment variables into the session:

```sh
argvus-sessionctl import-environment
```

This synchronizes Wayland, Hyprland, Qt, cursor, and DBus variables between the shell and systemd user services.

## Service units reference

Additional user units may be managed directly with `systemctl --user`:

```sh
systemctl --user status argvus-session.target
systemctl --user start argvus-session.target
systemctl --user stop argvus-session.target
systemctl --user restart <unit>

# View logs for a specific service
journalctl --user -u argvus-wallpaper.service -b -f
```

Logs for the initial session bootstrap are stored at:

```
${XDG_STATE_HOME:-$HOME/.local/state}/argvus/session.log
```
