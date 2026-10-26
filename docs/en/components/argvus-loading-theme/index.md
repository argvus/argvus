---
title: Loading overlay
description: The overlay shown while the session reloads directly.
---

The loading overlay is a GTK 4 layer-shell surface shown while ARGVUS reloads the session directly
(`argvus sessionctl reload`). A theme switch does not use it: the desktop changes in place. It is a helper shipped in the `argvus` package at
`/usr/lib/argvus/loading-theme/splash`, and it is not placed in `/usr/bin`.

The login does not use it: the greeter splash and the session-start spinner are the Wayland mode of
the handoff helper, which draws the same spinner without GTK and shows its first frame within
milliseconds (see [Greeter, lock and splash](/docs/developer-guide/subsystems/greeter-lock-and-splash/)).
With `--spinner-only` the overlay shows only that spinner and exits on `SIGTERM`. With `--detached` it does not exit when the process that started it exits; a session reload uses both flags and stops it through its pid file.
It reads the selected theme from the ARGVUS data directory and falls back to the packaged ARGVUS
Dark palette. It reports readiness only after the first frame of every surface it maps, not when
the process starts.
