---
title: Providers and services
description: Provider packages and their system interfaces.
---

Provider projects expose focused interfaces to UI consumers:

- `argvus network` and `argvus bluetooth` integrate NetworkManager and Bluetooth helpers;
- `argvus-display` integrates monitor state with Hyprland;
- `argvus-power` integrates idle, lock and power actions;
- `argvus-firewall` manages `argvus-firewall.service`;
- the `argvus removable-devices` helper in `argvus-gtk` consumes UDisks2;
- the `argvus control-center` helper in `argvus-gtk` consumes provider APIs and system D-Bus interfaces.

The Control Center is a client of these providers, not their replacement.
