---
title: Repository roles
description: Responsibilities of the ARGVUS source repositories after the workspace consolidation.
---

The repositories are grouped by responsibility rather than by navigation page:

- workspace: `argvus` (dispatcher, in-process commands, GTK 4 helpers, shared crates, the greeter in `greeter/` and the data of the commands);
- session and integration: `argvus-session`, `argvus-hyprland`, `argvus-portal`, `argvus-lock`, `argvus-boot-splash`, and the greeter (`argvus-greeter`, built from `argvus/greeter`);
- UI outside the workspace: Control Panel, taskbar (`argvus-taskbar`), launcher and Waybar;
- providers: display, power, firewall, notifications;
- assets and packaging: appearance, wallpapers, fonts, icons, profiles.

## Consolidated into the workspace

These repositories were merged into `de/argvus`. Their documentation now lives under `docs/` of the workspace, and their repositories are kept only for history:

| Former repository | Now | Documentation |
| --- | --- | --- |
| `argvus-config` | `argvus config` | [command line](/docs/reference/command-line/), [configuration files](/docs/reference/configuration-files/) |
| `argvus-i18n` | `argvus i18n` | [Internationalization component](/docs/components/argvus-i18n/) |
| `argvus-keys`, `argvus-projects` | `argvus keys`, `argvus projects` | command line |
| `argvus-terminal` | `argvus terminal` | [Terminal component](/docs/components/argvus-terminal/) |
| `argvus-system-monitor` | `argvus system-monitor` | [System monitor component](/docs/components/argvus-system-monitor/) |
| `argvus-network` | `argvus network`, `argvus bluetooth` | [Network component](/docs/components/argvus-network/) |
| `argvus-widget-telemetry` | `argvus widget-telemetry` | command line |
| `argvus-control-center` | `argvus control-center` (terminal application) | [Control Center component](/docs/components/argvus-control-center/) |
| `argvus-taskbar-calendar` | `argvus taskbar-calendar` (GTK 4 helper) | [Taskbar calendar component](/docs/components/argvus-taskbar-calendar/) |
| `argvus-removable-devices` | `argvus removable-devices` (GTK 4 helper) | [Removable devices component](/docs/components/argvus-removable-devices/) |
| `argvus-loading-theme` | `argvus loading-theme` (`splash` helper) | command line |
| `argvus-tui-terminal` | `argvus tui-terminal` (crate in the workspace) | command line |
| `argvus-tui`, `argvus-theme` | Shared crates `crates/argvus-tui` and `crates/argvus-theme` | developer notes |
| `argvus-greeter` | Source in `greeter/`, package `argvus-greeter` | [greeter component](/docs/components/argvus-greeter/) |
| `argvus-themes` | Retired. Each official theme is its own `argvus-theme-*` package. | [installing themes](/docs/user-guide/appearance/) |

The developer notes of the consolidated components are in `docs/internal/<repository>/`.

Language is not the ownership boundary: each component keeps its runtime helpers with its package. The one hard rule that cuts across packages: only `argvus-config` writes below `data/generated/`. A component that needs to reach an application that cannot read `config.json` adds an adapter in its own helper and lets the orchestrator call it after the canonical commit; it never writes a generated file. Public CLIs belong in `/usr/bin`; internal helpers belong under `/usr/lib/argvus/<component>` or in the commands directory when they are needed at runtime; architecture-independent data remains under `/usr/share/argvus/<component>`.

See the [package matrix](/docs/reference/package-matrix/) for the installed package boundary.
