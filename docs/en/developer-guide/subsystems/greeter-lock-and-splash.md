---
title: Greeter, lock and splash
description: Separate startup and authentication visuals.
---

`argvus-greeter` provides the greetd interface. The session-loading overlay is the `argvus loading-theme` helper shipped by `argvus`, started by `argvus-session-loading.service`. `argvus-boot-splash` provides the separate boot splash. `argvus-lock` provides Hyprlock configuration.

Theme state is handed across the greeter and session transition, but these packages have different lifecycles and installed payloads.

The greeter runs before the user's authenticated session and uses greetd's IPC/PAM boundary. The session-loading overlay starts when the new Wayland socket becomes available and ends when Hyprland reports readiness. The Plymouth theme belongs to the boot path and is not the login screen. The lock component is used after login and does not replace either greeter.

The login handoff has a DRM surface of its own. Once authentication succeeds, the greeter starts `/usr/lib/argvus/greeter/handoff` (`argvus-greeter-handoff`) before it asks greetd to start the session. The helper takes DRM master as soon as the greeter compositor releases it and keeps a frame on screen, so the display does not go blank between the greeter and the user compositor. It keeps running after the greeter exits. `argvus-start` sends `release` to its socket (`/run/argvus-greeter/handoff.sock`) just before it launches Hyprland, and the helper acknowledges only after it has dropped DRM master. A session without a greeter sends the same command and finds no surface, so nothing happens. A failure to start the surface is logged and does not stop the login. While it runs, send it commands with `/usr/lib/argvus/greeter/handoff --send theme=<name>` or `--send release`. The socket and the greeter's pre-login `session.log` live in `/run/argvus-greeter`, which `systemd-tmpfiles` creates owned by the `greeter` user with mode 0755: only the greeter writes there, and the session that logs in only reads the log and connects to the socket.

This separation matters when diagnosing a visual problem: a missing boot splash is a Plymouth/package issue, a blank transition is a session-loading or compositor-readiness issue, and an authentication failure belongs to greetd/PAM rather than the desktop session.

The interactive theme transition, direct session reload and post-login session handoff pass the selected theme explicitly to `/usr/lib/argvus/loading-theme/splash`. Direct reloads use the instant mode so the first mapped frame is opaque before configuration work begins; interactive transitions retain their visual fade. The interactive path also passes generated background, foreground and accent values; the splash palette is only a fallback. Verify the installed binary and package payload when source changes are not visible in the running session.
