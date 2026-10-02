---
title: Runtime lifecycle
description: Session startup and user-service ownership.
---

`argvus-session` imports the environment, starts Hyprland through `argvus-start` and starts `argvus-session.target` after the compositor is ready. The target pulls in shell, notification, wallpaper, idle and clipboard services. `argvus-sessionctl` coordinates start, stop, reload, restart, status and logs.

The session stops the target when Hyprland exits. Individual projects remain responsible for their own binaries and configuration.

## Startup

The graphical entry point is `/usr/share/wayland-sessions/argvus.desktop`, which invokes `/usr/bin/argvus-session`. The session imports the ARGVUS and XDG configuration environment, synchronizes it with `systemd --user` and D-Bus activation, and delegates compositor startup to `argvus-start`.

`argvus-start` validates the Hyprland configuration, applies the packaged fallback when a stock generated file is found or a user configuration is invalid, detects virtualization compatibility, and launches Hyprland. The compositor's readiness bridge then calls `argvus-sessionctl ready`, which starts the ARGVUS target with the current Wayland and Hyprland environment.

The target uses a one-shot preparation unit before starting long-running components. Preparation applies generated configuration, theme and accent state, monitor state, application integration and session environment. Persistent services are owned by `systemd --user`, not by ad-hoc background processes.

`argvus-sessionctl ready` projects the canonical configuration before starting the target. A fatal projection or target-start failure is logged and stops the independent loading overlay, preventing an apparently endless spinner. Optional visual consumers must resolve a fallback or report a degraded failure without replacing the session target.

Path resolution and the Hyprland preparation plan are Rust-owned. The legacy
`paths.sh` and `hypr-init.sh` files are compatibility entrypoints; they no
longer contain path precedence, startup loops or projection orchestration.

## Services and shutdown

The target commonly owns `argvus-control-panel.service`, `argvus-taskbar.service`, `argvus-widget-telemetry.service` when enabled, `argvus-wallpaper.service`, `argvus-dunst.service`, `argvus-hypridle.service`, clipboard watchers, keyboard-layout notifications, PolicyKit integration and the session-loading overlay. A failure in an optional service does not replace the whole session.

The services that need a Wayland client connection, including the Control Panel and idle manager, wait for a usable Hyprland runtime before launching their client. This avoids starting Quickshell or hypridle against a stale socket during compositor handoff; if Hyprland is replaced, systemd can start the component again after the new runtime is available.

Theme transitions serialize their projection and service fan-out as one lifecycle operation. The transition reload reuses the existing projection and does not re-enter it while its theme lock is held, preventing concurrent reloads from blocking session preparation.

When Hyprland exits, `argvus-start` stops `argvus-session.target` and the services tied to it. This prevents taskbar, shell, notification, idle and clipboard processes from surviving logout.

## Reloading

```sh
argvus-sessionctl reload
```

Reload re-imports the graphical environment, then **applies** a committed generation when a preceding domain command such as `apply-theme`, `accent`, `set`/`patch` or a widget-telemetry block change has already projected the change. In that case it restores the consumers listed in the manifest plan and does not call `argvus-config project` again, because reprojecting would rebuild an identical tree and discard the plan. Projection, migration and `ensure` are performed only when no valid pending plan exists — at session startup, or after `argvus-config recover` has to restore the previous document. A clean no-op is allowed only when the projections and managed runtime are both current. If the canonical state is unchanged but a compositor socket, service or consumer is stale, reload reconciles it instead of returning early. The normal fan-out covers Hyprland, taskbar, widget telemetry, Control Panel, notifications, wallpaper and idle according to the projection manifest, while unchanged optional consumers remain untouched. Every file under `data/generated/` is written by `argvus-config` at this point, so the session never re-renders a theme itself. The reload then runs `theme-adapters.sh`, which reconciles the consumers that cannot read the canonical document or the published projection on their own. Those adapters only read the committed generation: they write their own consumer files and caches, and they never write the canonical document or `data/generated/`, so a single user action still costs one mutation, one projection, one publish and one reload. Use `argvus-sessionctl restart <component>` only when a single owned service needs an explicitly isolated restart. Inspect the [services reference](../../reference/services/) for exact unit names.

Projection and reload operations are serialized with a user-runtime lock. For a temporary timing trace, set `ARGVUS_SESSION_PERF=1` before invoking the reload and inspect `~/.local/state/argvus/session.log`; the trace records stage boundaries without changing the normal reload behavior.
