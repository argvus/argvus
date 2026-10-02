---
title: Command line
description: Public ARGVUS commands and their owning packages.
---

The `/usr/bin/argvus` dispatcher provides `--setup`, `--system-monitor`, `--control-center`, `--terminal`, `--spf`, `--yazi`, `--about`, `--calendar`, `--game-snake`, `--default-apps`, `--removable-devices`, `--displays`, `--network-interfaces` and `--version`. The same names are available without the leading `--`.

Other public entry points include `argvus-session`, `argvus-start`, `argvus-sessionctl`, `argvus-tty`, `argvus-control-center`, `argvus-taskbar-calendar`, `argvus-accounts`, `argvus-displayctl`, `argvus-networkctl`, `argvus-bluetoothctl`, `argvus-notifications`, `argvus-firewall` and `argvus-widget-telemetry-toggle`.

Use each command's `--help` for options. There is no separate `argvus-about`, `argvus-setup` or `argvus-default-apps` binary.

## Dispatcher examples

The dispatcher is the stable user-facing entry point for integrated applications:

| Command | Action |
| --- | --- |
| `argvus --control-center` | Open the ARGVUS Control Center. |
| `argvus --default-apps` | Open the Control Center Apps page (compatibility route). |
| `argvus --calendar` | Toggle the taskbar calendar. |
| `argvus --game-snake` | Open the retro terminal Snake game. |
| `argvus --removable-devices` | Open the removable-device menu. |
| `argvus --system-monitor` | Open the configured system monitor. |
| `argvus --terminal` | Open the ARGVUS terminal profile. |
| `argvus --spf` / `argvus --yazi` | Open the corresponding file-manager profile, reading `data/generated/superfile/` and `data/generated/yazi/`. |
| `argvus --displays` | Open the Control Center Displays page. |
| `argvus --network-interfaces` | Open network-interface controls in the Control Center. |

The dispatcher also accepts command names without the leading `--`, such as `argvus control-center` or `argvus calendar`. Run `argvus --help` for the installed version's complete list.

## Session and domain commands

```sh
argvus-sessionctl status
argvus-sessionctl reload
argvus-sessionctl logs
argvus-displayctl --apply
argvus-networkctl status
argvus-bluetoothctl available
argvus-widget-telemetry-toggle status
```

`argvus-sessionctl` controls the ARGVUS user session and can restart owned components. The display, network and Bluetooth commands delegate to their respective providers. They are not replacements for the underlying system services.

## Setup and customization

`SUPER + SHIFT + R` runs `argvus-sessionctl reload`. When a preceding command has already committed a generation, reload applies it from the projection manifest and does not project again. If the ARGVUS profile or
`config.json` is missing, reload materializes a fresh canonical profile,
projects its generated files and applies the runtime changes. It does not copy
`/usr/share/argvus` into the home directory, and it does not replace a
malformed existing JSON file.

`argvus --setup` is a separate, optional override tool. It never serves as
profile recovery. Its manifest currently supports the advanced TUI overrides
`foot-tui` and `kitty-tui`:

```sh
argvus --setup --copy foot-tui
argvus --setup --copy kitty-tui
argvus --setup --copy-all
argvus --setup --copy foot-tui --force
argvus --setup --copy-all --dry-run
```

`--copy-all` copies only those explicit manifest entries; it does not enumerate
all first-level directories under `/usr/share/argvus`, and it never copies
scripts, services, assets, generated files or `config.json`. Setup resolves all
sources and destinations before mutating anything. `--force` backs up only the
affected destination, while `--dry-run` performs no filesystem mutation. There
is no separate `argvus-setup` binary.
