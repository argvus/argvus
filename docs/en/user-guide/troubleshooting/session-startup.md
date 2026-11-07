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

If `greeter.handoff-takeover-failed` appears without `handoff.master-acquired`,
inspect `handoff.master-unavailable card=<name> errno=<n>` for the acquisition
error. An observer opened while another compositor was master can return errno
13 (`EACCES`) even after that compositor releases master. The helper reopens the
same GPU to acquire master implicitly; `handoff.master-reopened` records this
recovery and a new framebuffer is prepared on that descriptor. The greeter cancels the failed helper (`greeter.handoff-cancelled`), so it
cannot add another wait before the user compositor starts. A session sending
`release` while acquisition is pending gets `handoff.release-before-master`:
the helper closes its DRM descriptors before acknowledging. A greeter compositor
crash in Hyprland/Aquamarine is a separate finding; cancelling the helper does
not establish that the driver or compositor problem has been fixed.

`greeter.vt-switch-returned active=<tty>` records the active VT when logind's
switch call returns. A drawing timeout also records `active-vt=<tty>` in its
error. These are snapshots, not proof of a completed DRM handover.

For a before/after comparison, save work and use five logins per version on the
same machine, theme and monitor setup. Before each following logout, copy
`/run/argvus-greeter/timing.log`, the session log and the boot journal to a separate
sample directory under `/tmp/argvus/`; the next greeter run replaces its timing
log. Record the boot ID with each sample and compare timestamps only within that
boot. Measure authentication to handoff first frame, session-wrapper entry,
final Hyprland launch and loading-spinner stop. Compare the median and worst
authentication-to-spinner-stop time and check visually for black frames,
flashes or a stuck spinner. The desktop-surfaces duration alone is not the
end-to-end login time. Repeat failure and recovery tests in a disposable session,
without terminating a session containing unsaved work.

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

`timing.log` holds the last login only: the greeter, the DRM handoff surface (`handoff.master-acquired`, `handoff.output-found`, `handoff.first-frame`, `handoff.release-received`, `handoff.replaced-by-compositor`) and `greeter.*` marks. `handoff.crtc` lines record every CRTC change while the handoff waits for DRM master; a line with `fb=0` or `mode_valid=false`, or `handoff.output-found ... framebuffer=0`, means the greeter compositor turned the output off before the handoff took it, so the monitor had to sync again. `greeter.watchdog-stop-requested` means the greeter session ended without waiting for its compositor, and `greeter.compositor-exit-deferred` that the compositor is asked to exit once the handoff surface is gone; `greeter.compositor-exit-requested` alone means it was asked to exit at once. If the greetd and logind lines in the journal show a gap of about half a second between the greeter session closing and the user session opening, the greeter compositor's teardown ran during the login. That is expected while the mark shows `compositor-in-foreground=true`: the hangup of the greeter's terminal ends the compositor before the deferred exit. That happens when the greeter could not leave its VT before its compositor exited: look for `greeter.vt-switch-requested` followed by `greeter.handoff-drawing`; when the second is missing, `greeter.handoff-takeover-failed error=<reason>` says why. `handoff.page-flip` means the handoff replaced the greeter's last frame without a modeset; `handoff.page-flip-failed errno=<n>` followed by `handoff.modeset` means the kernel refused the page flip. `handoff.overlay plane=<id>` and then `handoff.retired` mean the overlay hid Hyprland's first frames until the session spinner was on screen; `handoff.overlay-unavailable` means the CRTC had no free overlay plane, and `handoff.retire-timeout` means the session spinner never sent `retire`. The session spinner (`argvus-greeter-handoff`) writes `first layer-shell frame callback` and `stop requested` to the journal.

A `project.theme-switch` or nonzero `prepare.startup-theme` entry is expected after an appearance change, changed theme files, a new accent or migration of an older startup stamp. Reinstalling identical files preserves the content-based startup cache. If it appears on every login, the materialized theme files are missing; run `argvus theme-switch <theme>` once.

The session log is kept under `$XDG_STATE_HOME/argvus/session.log` (normally `~/.local/state/argvus/session.log`). After changing a user Hyprland override, log out and in so the session entry point validates and selects it again. See [Graphical session](/docs/argvus-session/graphical-session/) for the normal startup flow.
