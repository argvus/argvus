---
title: Repository roles
description: Responsibilities of the ARGVUS source repositories.
---

The repositories are grouped by responsibility rather than by navigation page:

- infrastructure: `argvus`, `argvus-config`, `argvus-session`, `argvus-tui`, `argvus-i18n`;
- configuration: `argvus-config` owns `config.json`, the path and lock model, and is the only writer of `data/generated/`;
- UI: Control Center, Control Panel, taskbar, calendar, launcher and monitor;
- providers: network, display, power, firewall, notifications and removable devices;
- integration: Hyprland, portal, greeter, lock and splash;
- assets and packaging: appearance, wallpapers, fonts, icons, profiles and Waybar.

See the [package matrix](/docs/reference/package-matrix/) for the installed package boundary.

Language is not the ownership boundary: each component keeps its runtime
helpers with its package. The one hard rule that cuts across packages: only
`argvus-config` writes below `data/generated/`. A component that needs to reach
an application that cannot read `config.json` adds an adapter in its own helper
and lets the orchestrator call it after the canonical commit; it never writes a
generated file. Rust owns persistent/stateful runtime logic, Shell
provides minimal compatibility or process adapters, and Python is used for
tests and audit tooling. Public CLIs belong in `/usr/bin`; internal executable
helpers belong under `/usr/lib/argvus/<component>` when they are needed at
runtime; architecture-independent data remains under
`/usr/share/argvus/<component>`.
