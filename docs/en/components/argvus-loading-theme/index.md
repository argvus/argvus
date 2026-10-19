---
title: Loading overlay
description: The overlay shown while a theme is applied or the session starts.
---

The loading overlay is a GTK 4 layer-shell surface shown while ARGVUS applies a theme or starts
the desktop session. It is a helper of the `argvus-gtk` package, started by the session at
`/usr/lib/argvus/loading-theme/splash`, and it is not placed in `/usr/bin`.

During a session start the overlay shows only the non-interactive spinner and exits on `SIGTERM`.
It reads the selected theme from the ARGVUS data directory and falls back to the packaged ARGVUS
Dark palette. It reports readiness only after the first frame of every surface it maps, not when
the process starts.
