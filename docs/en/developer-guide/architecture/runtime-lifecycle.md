---
title: Runtime lifecycle
description: Session startup and user-service ownership.
---

`argvus-session` imports the environment, starts Hyprland through `argvus start` and starts `argvus-session.target` after the compositor is ready. The target pulls in shell, notification, wallpaper, idle and clipboard services. `argvus sessionctl` coordinates start, stop, reload, restart, status and logs.

The session stops the target when Hyprland exits. Individual projects remain responsible for their own binaries and configuration.

## Startup

The graphical entry point is `/usr/share/wayland-sessions/argvus.desktop`, which invokes `argvus session`. The session imports the ARGVUS and XDG configuration environment, synchronizes it with `systemd --user` and D-Bus activation, and delegates compositor startup to `argvus start`.

`argvus start` validates the Hyprland configuration, applies the packaged fallback when a stock generated file is found or a user configuration is invalid, detects virtualization compatibility, and launches Hyprland. The compositor's readiness bridge then calls `argvus sessionctl ready`, which starts the ARGVUS target with the current Wayland and Hyprland environment.

The target uses a one-shot preparation unit before starting long-running components. Preparation applies generated configuration, theme and accent state, monitor state, application integration and session environment. Persistent services are owned by `systemd --user`, not by ad-hoc background processes.

`argvus start` queues `argvus-config.service` (`--no-block`) right before it launches Hyprland: the projection needs no compositor, so it overlaps the compositor startup. `argvus sessionctl ready` then starts the target, which finds that unit active or waits for its job, so the canonical configuration is still projected before preparation runs. Once the target has started, `sessionctl ready` waits up to 1.5 seconds for the wallpaper (a background layer surface) and the taskbar to be mapped before it removes the loading overlay, because a simple service counts as started before it draws; a surface whose unit is not running is not waited for, and the wait is logged as `session.desktop-surfaces`, with `session.wallpaper-surface` and `session.taskbar-surface` for the time each surface took. A fatal projection or target-start failure is logged and stops the independent loading overlay, preventing an apparently endless spinner. Optional visual consumers must resolve a fallback or report a degraded failure without replacing the session target.

### Login cost

A login with no configuration change repeats no expensive work:

- The projection runs the theme switch and the effects adapter only when the appearance or effects sections changed, or when their real outputs (the materialized user copies and `data/state/effects/<theme>.conf`) are missing.
- Preparation applies the startup theme, and the accent on top of it, only when the stamp in `$XDG_CACHE_HOME/argvus/session/theme-startup.stamp` no longer matches the packaged theme files and the accent marker. A package update or a new accent invalidates it once. When the theme is reapplied, preparation also reloads Hyprland, which started on the previous files.
- The GTK settings (`gsettings`) are written only when the startup theme was reapplied. Between logins dconf keeps them, so a stable login does not run them again.
- `argvus-dunst.service` is `Type=dbus`: it counts as started once Dunst owns `org.freedesktop.Notifications`, so the session target does not wait for the pause state to be restored.
- Preparation runs `spaces-switch.sh` and `borders-switch.sh` (`--apply-static`) side by side: they write disjoint files (the bars' JSONC margins and effective geometry, and the bars' CSS and the Rofi theme).
- Projection rewrites a generated file only when its contents changed. Unchanged files are left alone, so a stable login does not fsync them again.
- The display step of preparation (`argvus config display session-prepare`) applies only DPMS and disabled outputs: it neither projects again nor reloads the compositor.
- `argvus-session-reload.service` runs `argvus sessionctl reload-desktop --login`, which reloads Hyprland only when the projection manifest lists `hyprland` among the affected consumers. The desktop components do not wait for it.

The session spinner (the handoff helper in Wayland mode) is mapped from the compositor handoff until `argvus sessionctl ready` has started the target, so a single spinner covers the whole startup. It has no `PartOf=`, because recreating the target at a new login must not take it down; `sessionctl` stops it explicitly, and `RuntimeMaxSec=30` ends it if no stop arrives.

Each preparation, reload and projection-adapter step appends its duration to the session log as `<epoch-ms> timing <step> <N>ms`; see [session startup troubleshooting](/docs/user-guide/troubleshooting/session-startup/).

Path resolution, session preparation and the Hyprland reload plan are Rust-owned.
`argvus config session-prepare` and `argvus config session-reload-config` run the
startup and reload steps; the legacy `hypr-init.sh` script has been removed.

## Services and shutdown

The target commonly owns `argvus-control-panel.service`, `argvus-taskbar.service`, `argvus-widget-telemetry.service` when enabled, `argvus-wallpaper.service`, `argvus-dunst.service`, `argvus-hypridle.service`, clipboard watchers, keyboard-layout notifications and PolicyKit integration. The session-loading overlay is started by `argvus start`, not by the target. A failure in an optional service does not replace the whole session.

The services that need a Wayland client connection, including the Control Panel and idle manager, wait for a usable Hyprland runtime before launching their client. This avoids starting Quickshell or hypridle against a stale socket during compositor handoff; if Hyprland is replaced, systemd can start the component again after the new runtime is available.

A theme switch serializes its commit, build, projection and consumer reloads as one lifecycle operation under the theme lock. It does not reload `argvus-config.service` nor restart services: the projection it runs only writes files and leaves the theme to the switch while the lock is held, and the running consumers reload in place (see [Appearance and themes](/docs/developer-guide/subsystems/appearance-and-themes/)).

When Hyprland exits, `argvus start` stops `argvus-session.target` and the services tied to it. This prevents taskbar, shell, notification, idle and clipboard processes from surviving logout.

## Reloading

```sh
argvus sessionctl reload
```

Reload re-imports the graphical environment, then **applies** a committed generation when a preceding domain command such as `apply-theme`, `accent`, `set`/`patch` or a widget-telemetry block change has already projected the change. In that case it restores the consumers listed in the manifest plan and does not call `argvus config project` again, because reprojecting would rebuild an identical tree and discard the plan. Projection, migration and `ensure` are performed only when no valid pending plan exists — at session startup, or after `argvus config recover` has to restore the previous document. A clean no-op is allowed only when the projections and managed runtime are both current. If the canonical state is unchanged but a compositor socket, service or consumer is stale, reload reconciles it instead of returning early. The normal fan-out covers Hyprland, taskbar, widget telemetry, Control Panel, notifications, wallpaper and idle according to the projection manifest, while unchanged optional consumers remain untouched. Every file under `data/generated/` is written by `argvus-config` at this point, so the session never re-renders a theme itself. The reload then runs `theme-adapters.sh`, which reconciles the consumers that cannot read the canonical document or the published projection on their own. Those adapters only read the committed generation: they write their own consumer files and caches, and they never write the canonical document or `data/generated/`, so a single user action still costs one mutation, one projection, one publish and one reload. Use `argvus sessionctl restart <component>` only when a single owned service needs an explicitly isolated restart. Inspect the [services reference](/docs/reference/services/) for exact unit names.

Projection and reload operations are serialized with a user-runtime lock. For a temporary timing trace, set `ARGVUS_SESSION_PERF=1` before invoking the reload and inspect `~/.local/state/argvus/session.log`; the trace records stage boundaries without changing the normal reload behavior.
