---
title: Removable Storage
description: The Rust-powered Waybar removable storage module.
---

The `argvus-storage` package ships a Rust program that watches removable storage over UDisks2 (D-Bus, zero polling) and feeds a compact Waybar module.

## Usage

The module comes pre-configured in the recording/microphone/audio group. The bar shows a single Font Awesome icon when removable devices exist; when no device is present the module hides. The Waybar tray is reserved for apps that publish tray icons, such as Telegram and Steam.

| Button | Action |
|---|---|
| Left | Choose a device and open it in the file manager |
| Right | Context menu (Open/Mount/Unmount/Eject/Power Off/Unlock/Lock/Copy) |

## Commands

```sh
argvus-storage once      # print one JSON line and exit
argvus-storage list      # list removable volumes
argvus-storage devices   # choose a device and open it in the file manager
argvus-storage menu      # context menu (rofi/wofi/dmenu)
argvus-storage unmount   # actions: open mount unmount eject poweroff lock unlock copy
```

## Configuration

Defaults live in `/etc/argvus-storage/config.json` and `/etc/argvus-storage/theme.css`. To customize per user, copy them to `$XDG_CONFIG_HOME/argvus-storage/`. Options: `show_name`, `show_capacity`, `hide_when_empty`, `show_hidden`, `max_devices`, `sort` (`mount_time` | `insertion` | `name` | `size`), `separator`, `format`, `tooltip_format`, `file_manager_command`, `open_command` (compatibility alias), `copy_command`, `unlock_command`, `menu` (`rofi` | `wofi` | `dmenu`), `menu_flags` and `icons`.

For LUKS volumes, `unlock_command` (default `kitty -e`) opens a terminal for the passphrase. Runtime dependencies: `udisks2`, `glib2`, `rofi`/`wofi`/`dmenu`, `wl-clipboard` and `libnotify`.

## Themes

The system package owns:

```text
/etc/argvus-storage/theme.css
/etc/argvus-storage/themes/argvus-dark-aether.css
/etc/argvus-storage/themes/argvus-dark-silver.css
/etc/argvus-storage/themes/argvus-light-veil.css
/etc/argvus-storage/themes/argvus-dark-slate.css
```

When the user changes the desktop theme, ARGVUS prepares the matching storage theme in its generated runtime config. User files under `$XDG_CONFIG_HOME/argvus-storage/` still take precedence over system defaults.

## Integration with ARGVUS

The Waybar module is configured in `argvus` as `custom/storage`:

```json
{
  "exec": "argvus-storage watch",
  "return-type": "json",
  "format": "{}",
  "tooltip": true,
  "hide-empty-text": true,
  "on-click": "/usr/share/argvus/hypr/scripts/storage-menu.sh"
}
```

`storage-menu.sh` anchors the GTK menu to the pointer position when possible. The `argvus` package depends on `argvus-storage`, but the storage package owns its binary, config, themes and license.
