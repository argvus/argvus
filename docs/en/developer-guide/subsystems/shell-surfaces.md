---
title: Shell surfaces
description: Taskbar, control panel and telemetry ownership.
---

`argvus-shell` owns taskbar actions and UX; the taskbar JSONC and CSS it ships are projections of `config.json`, and `argvus-config` writes the active layer under `data/generated/taskbar/` plus a mirrored copy under `data/taskbar/`. `argvus-shell` owns Quickshell cards. `argvus-shell` provides the optional Telemetry surface, while the `argvus widget-telemetry` command in `argvus` retains block-control semantics; its JSONC and CSS are written to `data/taskbar/` from the packaged defaults on an appearance change, after which `argvus-config` rewrites only the delimited `ARGVUS_TELEMETRY_*` and font blocks. `argvus-session` owns their user services.

There is no current `argvus-shell` package or `argvus-shell.service` in this runtime model.
