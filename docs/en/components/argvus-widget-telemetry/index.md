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
`network` and `dev_dashboard`. The taskbar also uses the `cpu-temp` and `gpu` probes.
