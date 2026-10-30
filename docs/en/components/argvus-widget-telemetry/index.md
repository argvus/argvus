---
title: Widget telemetry
description: The Waybar telemetry blocks and their toggle, through argvus widget-telemetry.
---

The telemetry widget is the system information shown in the taskbar (header, keys, system, CPU
and GPU, memory, storage, processes, network and the developer dashboard). It is in-process in
the `argvus` package: there are no shell collectors.

## Commands

```text
argvus widget-telemetry block BLOCK ...   prints the text of one block (used by the bar)
argvus widget-telemetry toggle ...        switches the blocks on and off, and their order
```

`block` is the hot path of the bar: it is called on every refresh. `toggle` changes which blocks
the bar shows and moves the scroll window.

The blocks are `header`, `keys`, `system`, `cpu_gpu`, `memory`, `storage`, `processes`,
`network` and `dev_dashboard`. The taskbar also uses the `cpu-temp` and `gpu` probes. The GPU probe accepts `--present` (a supported GPU is visible in DRM/sysfs), `--available` (a GPU and the btop command are available for the panel), `--available-temp` (an attributable temperature can be read), and `--temp` (the highest valid temperature, or `N/A`).

GPU temperatures are matched to a GPU's PCI device. NVIDIA proprietary readings use `nvidia-smi`/NVML bus IDs; NVIDIA nouveau, AMD, and Intel readings use hwmon only when its sysfs device link uniquely matches that GPU. The taskbar temperature is the maximum valid value across detected GPUs. Missing, invalid, or ambiguous readings stay unavailable. NVIDIA proprietary temperature requires the driver provider; AMD and Intel temperature depends on the kernel exposing a linked hwmon sensor. Intel temperature is not supplied by btop 1.4.7's GPU panel.

## Starting with the session

The `argvus-widget-telemetry.service` user unit starts with the session only when the widget is
enabled. The master switch is `effects.widget_telemetry_enabled` in the configuration, which is
enabled by default, so a new account starts the widget at login.

Once the toggle has written `$XDG_CACHE_HOME/argvus/waybar/widget-telemetry-state`, that file
decides instead. Only a first line of `enabled` starts the widget. The older
`desktop-telemetry-state` and `sysinfo-state` files are still read when the current file is
missing.
