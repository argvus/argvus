---
title: Greeter, lock and splash
description: Separate startup and authentication visuals.
---

`argvus-greeter` provides the greetd interface. `argvus-theme-splash` provides the GTK4 session-loading overlay started by `argvus-session-loading.service`. `argvus-splash` provides the separate Plymouth boot theme. `argvus-lock` provides Hyprlock configuration.

Theme state is handed across the greeter and session transition, but these packages have different lifecycles and installed payloads.

The greeter runs before the user's authenticated session and uses greetd's IPC/PAM boundary. The session-loading overlay starts when the new Wayland socket becomes available and ends when Hyprland reports readiness. The Plymouth theme belongs to the boot path and is not the login screen. The lock component is used after login and does not replace either greeter.

This separation matters when diagnosing a visual problem: a missing boot splash is a Plymouth/package issue, a blank transition is a session-loading or compositor-readiness issue, and an authentication failure belongs to greetd/PAM rather than the desktop session.

The interactive theme transition, direct session reload and post-login session handoff pass the selected theme explicitly to `/usr/lib/argvus/theme-splash/splash`. Direct reloads use the instant mode so the first mapped frame is opaque before configuration work begins; interactive transitions retain their visual fade. The interactive path also passes generated background, foreground and accent values; the splash palette is only a fallback. Verify the installed binary and package payload when source changes are not visible in the running session.
