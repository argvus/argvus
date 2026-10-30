---
title: System monitor
description: Open the ARGVUS system monitor.
---

Open the configured monitor with:

```sh
argvus --system-monitor
```

The `argvus system-monitor` subcommand provides the native entry point and btop integration. Telemetry shown in the taskbar is a separate feature provided by `argvus widget-telemetry`. Clicking its CPU, Memory, and GPU modules opens btop inside an ARGVUS TUI panel with a branded header and an `Esc Close` footer. GPU mode shows up to six GPU boxes and no CPU, memory, network, or process boxes. Press `q` or `Esc` to close the panel. The bottom Kitty tab shows the localized `Press q to close` hint. Standalone `argvus system-monitor --cpu`, `--mem`, and `--gpu` calls launch btop directly with the selected boxes.

The GPU panel depends on the capabilities of the installed btop build and the system's device providers. NVIDIA monitoring needs GPU support in btop, the NVIDIA Management Library from the installed NVIDIA driver, and access to the device. AMD monitoring uses ROCm SMI, which is optional and may not be installed. Intel GPU data is read from sysfs and depends on the user's access to those interfaces; btop 1.4.7 reports Intel utilization, power, and clock, but does not provide Intel temperature. NVIDIA nouveau and other drivers may expose a GPU temperature to the taskbar through device-linked hwmon, while btop provider support depends on the driver and libraries available. ARGVUS does not change device permissions or run btop with elevated privileges. If no GPU is exposed, or btop does not render a GPU box within three seconds because support or a provider is unavailable, the panel shows a localized unavailable message.

The monitor reads the selected theme and accent from
`$XDG_CONFIG_HOME/argvus/config.json` first. Its generated btop files remain
derived cache state; `.active-theme` and `.accent-color` are compatibility
fallbacks. Because btop keeps its own theme file, the monitor is reconciled by
the theme transaction (`argvus theme-switch`) as an external adapter after the canonical change is
committed; it is not part of the generated tree.

## Runtime contract

- The theme and the accent come from the canonical configuration (`argvus config get
  /appearance/theme` and `/appearance/accent`). The files `~/.config/argvus/data/.active-theme`
  and `.accent-color` are fallbacks.
- The generated btop configuration and themes are written to
  `$XDG_CACHE_HOME/argvus/argvus-system-monitor`. They are derived state; do not edit them.
- The defaults are installed under `/usr/share/argvus-system-monitor/btop`.
- The module shortcuts (`--mem`, `--cpu`, `--net`, `--proc`) are read only as the first argument.
  Passing btop's own `-c`/`--config` or `--themes-dir` runs btop with those paths and writes no
  configuration.
- `--panel [--cpu|--mem|--gpu]` embeds btop in the ARGVUS TUI shell. This mode is used by the taskbar;
  without `--panel`, the monitor continues to replace itself with btop.
