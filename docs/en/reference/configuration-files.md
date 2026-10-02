---
title: Configuration files
description: User, system, package-default and generated ARGVUS configuration.
---

| Layer | Location | Meaning |
| --- | --- | --- |
| System | `/etc/argvus/` | Administrator configuration for services and providers. |
| Package defaults | `/usr/share/argvus/` | Read-only component defaults and runtime assets. |
| User | `$XDG_CONFIG_HOME/argvus/config.json` | Canonical portable preferences. |
| Managed data | `$XDG_CONFIG_HOME/argvus/data/` | Component data, compatibility files and internal metadata. |
| Generated | `$XDG_CONFIG_HOME/argvus/data/generated/` | Projections of `config.json`, written only by `argvus-config`. Never edit, never copy elsewhere as configuration, and never read back as a source. |
| Runtime | `$XDG_RUNTIME_DIR/` and `$XDG_CACHE_HOME/` | Sockets, locks and transient state. |

The active theme, accent, custom wallpaper and effects state are logical user state. The managed logical model is documented in [Canonical configuration](./argvus-config/). Generated Hyprland, GTK, Qt6ct, Waybar, Quickshell, Rofi, Dunst, Hyprlock, Yazi, Superfile, terminal, font and removable-devices files should not be treated as independent sources of truth.

`argvus-config` also mirrors the theme layer into `data/waybar/argvus-taskbar.{jsonc,css}` and `data/waybar/argvus-widget-telemetry.{jsonc,css}`. Those copies are replaced from the packaged defaults on an appearance change, then have their delimited managed blocks rewritten, so treat them as derived rather than as hand-maintained files. Native application configuration under `$XDG_CONFIG_HOME/<app>` is different: it is an adapter target written by the appearance orchestrators, never by the projector.
