---
title: Removable devices
description: Mount and manage removable storage from ARGVUS.
---

`argvus-removable-devices` watches UDisks2 and provides the ARGVUS Taskbar storage module and its device menu. The menu is also available as a separate Rofi-style interface, so the same device actions can be used from the taskbar or from a launcher workflow.

The available actions are `open`, `mount`, `unmount`, `eject`, `poweroff`, `lock`, `unlock` and `copy`. The `list`, `watch` and `menu` commands expose the corresponding status, taskbar stream and menu interfaces for integrations.

The default file-manager action follows the file manager selected in ARGVUS Control Center. This keeps removable-device actions consistent with the user's application defaults.

The system configuration is `/etc/argvus/removable-devices/config.json`. The ARGVUS-managed per-user projection is `~/.config/argvus/data/removable-devices/config.json`, with a matching `theme.css` next to it. That stylesheet is also projected to `~/.config/argvus/data/generated/removable-devices/theme.css` from the canonical theme, accent and mode, with a family/variant fallback for themes that ship no dedicated removable-devices asset, so a theme change keeps the module consistent without any manual step. Legacy component paths are migration inputs; later user configuration overrides system defaults.

Useful settings include whether the module is hidden when no device is present (`hide_when_empty`), whether names and capacities are shown, the device sort order, notification behavior, menu backend and status icons. Packaged themes are under `/usr/share/argvus/removable-devices/config/themes/`; these are implementation assets, not the same as the seven ARGVUS appearance theme families.
