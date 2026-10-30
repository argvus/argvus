---
title: Night Light
description: Adjust color temperature and fixed local schedules on Hyprland.
---

Open **Control Center > Desktop & Input > Displays > Night Light**, search for **Night Light**, or run `argvus control-center displays night-light`.

Night Light and its schedule are off initially. Choose a temperature from **1000 to 20000 Kelvin** (default **4500 K**). Lower values give warmer colors. The page shows saved manual activation, the effective state and the next transition. Enter edits a value; Esc cancels it or returns to Displays. Invalid values are refused before saving. Changes save and apply immediately. If applying fails after a save, the saved preference remains visible alongside the actual effective state and the error.

## Schedule and quick actions

The fixed schedule uses the current local time zone. The start is included and the end is excluded; **20:00–07:00** crosses midnight. Start and end must differ and use `HH:MM`. No location or solar calculation is used.

Use the **Display** card of the Control Panel or **SUPER + Shift + B** for quick toggles. The brightness selector keeps all its brightness actions and adds Night Light when the backend is available, including on a display without a brightness backend. There is no permanent taskbar module.

With scheduling off, manual activation is persistent. With scheduling on, a quick toggle creates a runtime override until the next scheduled transition. Changing the schedule or temperature clears that override. Logout, a new compositor instance or a changed next transition (including a time-zone change) also invalidates it.

## Commands and state

All interfaces use the same public command:

```sh
argvus config display night-light status
argvus config display night-light enable
argvus config display night-light disable
argvus config display night-light toggle
argvus config display night-light temperature 4500
argvus config display night-light schedule on 20:00 07:00
argvus config display night-light schedule off 20:00 07:00
argvus config display night-light apply
argvus config display night-light apply-schedule
```

`status` returns JSON schema version 1: `available`, `unavailable_reason`, `settings` (enabled, temperature, schedule_enabled, start, end), `active`, `effective_temperature`, `mode` (`manual` or `scheduled`), `manual_override`, `next_transition` (ISO 8601 with offset), `minimum_temperature` and `maximum_temperature`. Effective fields are null when unavailable; effective temperature is null when identity is active. Availability failures are reported in status; action failures return nonzero with a diagnostic.

The canonical preference is `/displays/night_light`, stored atomically by `argvus config` in `$XDG_CONFIG_HOME/argvus/config/displays.json`. Volatile overrides and the action lock live under `$XDG_RUNTIME_DIR/argvus/night-light/`. Generated monitor files are independent. Do not edit generated files to configure this feature.

## Runtime

Requires Hyprland and **hyprsunset >= 0.4.0**. The session starts the upstream `hyprsunset.service` once, applies saved state through `argvus-night-light-init.service`, and evaluates schedules every minute through `argvus-night-light.timer` and `argvus-night-light.service`. Clock and time-zone changes trigger evaluation too. With scheduling off, the periodic operation leaves manual state alone. Repeated applications query effective identity/temperature and skip unchanged writes. No compositor reload is needed.

ARGVUS ships a systemd drop-in for the upstream service: it starts with `--identity` and isolates its config discovery to `/usr/share/argvus/session/config/night-light`. This avoids competing native profiles and leaves personal `hyprsunset.conf` files untouched. The drop-in applies to that upstream service while the ARGVUS session package is installed, including starts outside ARGVUS. Existing user systemd overrides can take precedence; inspect `systemctl --user cat hyprsunset.service` when diagnosing that situation. Hyprsunset 0.4.0 rejects the separate path argument of `--config`, so config discovery is isolated with service-local XDG variables instead.

The backend uses the official Hyprsunset IPC (`temperature`, `identity`, `identity get`), with bounded I/O and read-back after changes. No shader, alternative compositor backend, system configuration write or privilege escalation is used. Unavailable controls are disabled in Control Center and hidden in the Display card.
