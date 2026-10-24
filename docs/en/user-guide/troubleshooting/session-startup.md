---
title: Session startup troubleshooting
description: Diagnose greeter, Hyprland and systemd user-session failures.
---

Run:

```sh
argvus sessionctl status
argvus sessionctl logs
systemctl --user status argvus-session.target
```

Check the greeter, `argvus start`, Hyprland and the relevant user unit separately. A successful package build does not prove that the installed payload or active user override is current.

Use the checks in this order:

1. Confirm that the greeter is using the installed ARGVUS greetd setup.
2. Read the session log and look for a missing or invalid Hyprland Lua configuration.
3. Check readiness and target state with `systemctl --user status argvus-session.target`.
4. Inspect the failing component rather than restarting every service at once.

If `argvus-session.target` is inactive while the loading overlay is active, inspect
the projection error before restarting services:

```sh
argvus config project
journalctl --user -u argvus-session.target -u argvus-session-prepare.service -b --no-pager
```

`argvus-config` is the only writer below `data/generated/`, so `project` is the
authoritative way to reproduce a generation while diagnosing. Session startup
runs it because no committed plan exists yet; the plan it writes to
`${XDG_STATE_HOME:-$HOME/.local/state}/argvus/config-projection.json` is what a
later `argvus sessionctl reload` applies, without projecting again.

The session stops the loading overlay when fatal preparation or target startup
fails. A visual consumer failure should either use its documented fallback or be
reported explicitly; it must not be hidden as a successful session.

## Slow login

Each startup step records its duration in the session log. To see the steps of the last login, slowest first:

```sh
grep timing ~/.local/state/argvus/session.log | tail -30 | sort -k4 -rn
systemd-analyze --user critical-chain argvus-session.target
```

To time the switch from the greeter to the desktop, read the greeter and handoff marks together with the journal. All of them use `CLOCK_MONOTONIC`, so they line up:

```sh
cat /run/argvus-greeter/timing.log
journalctl -b -o short-monotonic --since -2min | grep -E 'greetd|Hyprland|coredump|argvus-greeter-handoff|session-loading'
grep monotonic_ns ~/.local/state/argvus/session.log | tail -5
```

`timing.log` holds the last login only: the greeter, the DRM handoff surface (`handoff.master-acquired`, `handoff.output-found`, `handoff.first-frame`, `handoff.release-received`, `handoff.replaced-by-compositor`) and `greeter.*` marks. `handoff.crtc` lines record every CRTC change while the handoff waits for DRM master; a line with `fb=0` or `mode_valid=false`, or `handoff.output-found ... framebuffer=0`, means the greeter compositor turned the output off before the handoff took it, so the monitor had to sync again. That happens when the installed Hyprland does not support `HYPRLAND_RETAIN_FRAME_ON_EXIT`. `handoff.page-flip` means the handoff replaced the greeter's last frame without a modeset; `handoff.page-flip-failed errno=<n>` followed by `handoff.modeset` means the kernel refused the page flip. The session spinner (`argvus-greeter-handoff`) writes `first layer-shell frame callback` and `stop requested` to the journal.

A `project.theme-switch` or `prepare.startup-theme` entry is expected only after an appearance change, a package update or a new accent. If it appears on every login, the materialized theme files are missing; run `argvus theme-switch <theme>` once.

The session log is kept under `$XDG_STATE_HOME/argvus/session.log` (normally `~/.local/state/argvus/session.log`). After changing a user Hyprland override, log out and in so the session entry point validates and selects it again. See [Graphical session](/docs/argvus-session/graphical-session/) for the normal startup flow.
