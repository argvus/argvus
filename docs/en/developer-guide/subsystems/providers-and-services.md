---
title: Providers and services
description: Provider packages and their system interfaces.
---

Provider projects expose focused interfaces to UI consumers:

- `argvus-network` integrates NetworkManager and Bluetooth helpers;
- `argvus-display` integrates monitor state with Hyprland;
- `argvus-power` integrates idle, lock and power actions;
- `argvus-firewall` manages `argvus-firewall.service`;
- `argvus-removable-devices` consumes UDisks2;
- `argvus-control-center` consumes provider APIs and system D-Bus interfaces.

The Control Center is a client of these providers, not their replacement.
