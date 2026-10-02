---
title: Session startup troubleshooting
description: Diagnose greeter, Hyprland and systemd user-session failures.
---

Run:

```sh
argvus-sessionctl status
argvus-sessionctl logs
systemctl --user status argvus-session.target
```

Check the greeter, `argvus-start`, Hyprland and the relevant user unit separately. A successful package build does not prove that the installed payload or active user override is current.

Use the checks in this order:

1. Confirm that the greeter is using the installed ARGVUS greetd setup.
2. Read the session log and look for a missing or invalid Hyprland Lua configuration.
3. Check readiness and target state with `systemctl --user status argvus-session.target`.
4. Inspect the failing component rather than restarting every service at once.

If `argvus-session.target` is inactive while the loading overlay is active, inspect
the projection error before restarting services:

```sh
argvus-config project
journalctl --user -u argvus-session.target -u argvus-session-prepare.service -b --no-pager
```

`argvus-config` is the only writer below `data/generated/`, so `project` is the
authoritative way to reproduce a generation while diagnosing. Session startup
runs it because no committed plan exists yet; the plan it writes to
`${XDG_STATE_HOME:-$HOME/.local/state}/argvus/config-projection.json` is what a
later `argvus-sessionctl reload` applies, without projecting again.

The session stops the loading overlay when fatal preparation or target startup
fails. A visual consumer failure should either use its documented fallback or be
reported explicitly; it must not be hidden as a successful session.

The session log is kept under `$XDG_STATE_HOME/argvus/session.log` (normally `~/.local/state/argvus/session.log`). After changing a user Hyprland override, log out and in so the session entry point validates and selects it again. See [Graphical session](../sessions/graphical-session/) for the normal startup flow.
