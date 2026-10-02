---
title: Shell surfaces
description: Taskbar, control panel and telemetry ownership.
---

`argvus-waybar` provides the patched `/usr/bin/waybar`. `argvus-taskbar` owns taskbar actions and UX; the taskbar JSONC and CSS it ships are projections of `config.json`, and `argvus-config` writes the active layer under `data/generated/waybar/` plus a mirrored copy under `data/waybar/`. `argvus-control-panel` owns Quickshell cards. `argvus-widget-telemetry` owns the optional telemetry bar and its block semantics; its JSONC and CSS are written to `data/waybar/` from the packaged defaults on an appearance change, after which `argvus-config` rewrites only the delimited `ARGVUS_TELEMETRY_*` and font blocks. `argvus-session` owns their user services.

There is no current `argvus-shell` package or `argvus-shell.service` in this runtime model.
