---
title: Removable devices
description: Mount and manage removable storage from ARGVUS.
---

`argvus-removable-devices` watches UDisks2 and provides the ARGVUS Taskbar storage module and its device menu. The menu is also available as a separate Rofi-style interface, so the same device actions can be used from the taskbar or from a launcher workflow.

The available actions are `open`, `mount`, `unmount`, `eject`, `poweroff`, `lock`, `unlock` and `copy`. The `list`, `watch` and `menu` commands expose the corresponding status, taskbar stream and menu interfaces for integrations.

## Taskbar menu

With `"mode": "gui"` (the system default), a click on the taskbar storage icon opens a GTK menu attached to the ARGVUS Taskbar, like the calendar popup: below the bar when the taskbar is at the top of the screen, and above it when the taskbar is at the bottom. Each device row has an action submenu with only the actions its state allows.

- **Pointer:** hovering or clicking a device row opens its submenu. The submenu stays open while the pointer crosses the menu edges or leaves the menus, and it follows another row only after the pointer rests there briefly, so a diagonal path to the submenu does not close it. A click outside both menus closes the popup.
- **Keyboard:** `Up`/`Down` select a row, `Right`, `Enter` or `Space` open the submenu (or run **Refresh**/**Quit**), `Enter` or `Space` run the selected action, `Left` or `Escape` return to the device list, and `Escape` there closes the popup.

**Copy path** puts the mount point on the clipboard through `copy_command` (default `wl-copy`, from `wl-clipboard`), so the path is still there after the menu closes; a notification confirms it when `show_notification` is on. **Lock** is offered for every unlocked encrypted volume and unmounts it first when needed.

The default file-manager action follows the file manager selected in ARGVUS Control Center. This keeps removable-device actions consistent with the user's application defaults.

The system configuration is `/etc/argvus/removable-devices/config.json`. The ARGVUS-managed per-user projection is `~/.config/argvus/data/removable-devices/config.json`, with a matching `theme.css` next to it. That stylesheet is also projected to `~/.config/argvus/data/generated/removable-devices/theme.css` from the canonical theme, accent and mode, with a family/variant fallback for themes that ship no dedicated removable-devices asset, so a theme change keeps the module consistent without any manual step. Legacy component paths are migration inputs; later user configuration overrides system defaults.

Useful settings include whether the module is hidden when no device is present (`hide_when_empty`), whether names and capacities are shown, the device sort order, notification behavior, menu backend and status icons. Its stylesheet is rendered from the active ARGVUS theme (`removable-devices/theme.css` of the theme build) and copied to the panel's `theme.css` on every theme or accent change.
