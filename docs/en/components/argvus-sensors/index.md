---
title: Temperature sensors
description: Live temperature overview backed by lm_sensors JSON.
---

`argvus sensors` opens the in-process Ratatui panel used by the taskbar CPU temperature popup and its GPU temperature fallback. It runs `sensors -j`, groups readings by the chip names and labels reported by the hardware, and refreshes every two seconds.

Readings use Celsius. The panel shows `max`, `crit`, and alarm state when lm_sensors provides them. A gauge is shown only when a positive limit is available. Missing `lm_sensors`, command failures, invalid JSON, and empty readings remain visible as recoverable states; polling retries on the next cycle.

Press `q` or `Esc` to close. The taskbar opens the panel in Kitty through `argvus tui-terminal`; its localized bottom tab title shows the close hint. The GPU path backed by `nvidia-smi` is unchanged. The taskbar package already depends on `lm_sensors`, so no separate package or dependency is needed.
