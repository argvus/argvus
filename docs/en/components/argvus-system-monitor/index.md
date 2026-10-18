---
title: System monitor
description: Open the ARGVUS system monitor.
---

Open the configured monitor with:

```sh
argvus --system-monitor
```

The `argvus system-monitor` subcommand provides the native entry point and btop integration. Telemetry shown in the taskbar is a separate feature provided by `argvus widget-telemetry`.

The monitor reads the selected theme and accent from
`$XDG_CONFIG_HOME/argvus/config.json` first. Its generated btop files remain
derived cache state; `.active-theme` and `.accent-color` are compatibility
fallbacks. Because btop keeps its own theme file, the monitor is reconciled by
the theme transaction (`argvus theme-switch`) as an external adapter after the canonical change is
committed; it is not part of the generated tree.
