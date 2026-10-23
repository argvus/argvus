---
title: Loading overlay
description: The overlay shown while a theme is applied or the session starts.
---

The loading overlay is a GTK 4 layer-shell surface shown while ARGVUS applies a theme or starts
the desktop session. It is a helper shipped in the `argvus` package, started by the session at
`/usr/lib/argvus/loading-theme/splash`, and it is not placed in `/usr/bin`.

During a session start the overlay shows only the non-interactive spinner and exits on `SIGTERM`.
The spinner is the one the greeter's DRM handoff surface also draws, so the login reads as one
splash (see [Greeter, lock and splash](/docs/developer-guide/subsystems/greeter-lock-and-splash/)).
It reads the selected theme from the ARGVUS data directory and falls back to the packaged ARGVUS
Dark palette. It reports readiness only after the first frame of every surface it maps, not when
the process starts.
