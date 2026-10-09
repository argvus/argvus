---
title: File locations
description: Important installed ARGVUS files and directories.
---

- `/usr/bin/argvus*` — public commands.
- `/usr/share/argvus/hyprland/` — Hyprland configuration and helpers.
- `/usr/share/argvus/control-panel/` — Control Panel QML and scripts.
- `/usr/share/argvus/taskbar/` — taskbar configuration and scripts.
- `/usr/share/argvus/lock/` — Hyprlock configuration.
- `/usr/share/argvus/portal/` — portal integration files.
- `/usr/share/backgrounds/argvus/` — wallpaper assets.
- `/etc/argvus/` — system provider configuration.
- `/etc/argvus/greeter.toml` — greeter configuration.
- `/etc/argvus/removable-devices/config.json` — system defaults for removable storage.
- `$XDG_CONFIG_HOME/argvus/config.json` — canonical portable preferences.
- `$XDG_CONFIG_HOME/argvus/data/` — managed component data, backups, locks and generated output.
- `$XDG_CONFIG_HOME/waybar/argvus-taskbar.{jsonc,css}` — optional complete native Waybar taskbar overrides.
- `$XDG_CONFIG_HOME/argvus/data/taskbar/argvus-taskbar.{jsonc,css}` — managed ARGVUS taskbar copies.
- `$XDG_CONFIG_HOME/argvus/data/generated/waybar/` — generated Waybar profiles, including `argvus-taskbar.css` and `argvus-widget-telemetry.{jsonc,css}`; these take precedence over the copies above.
- `$XDG_CONFIG_HOME/argvus/data/generated/waybar/argvus-widget-telemetry.{jsonc,css}` — also mirrored under `data/waybar/`, which is replaced from the packaged default on an appearance change and then has its `ARGVUS_TELEMETRY_*` and font blocks rewritten.
- `$XDG_CONFIG_HOME/argvus/data/generated/yazi/` — projected Yazi configuration tree, with a variant-aware fallback for themes that ship no Yazi flavor.
- `$XDG_CONFIG_HOME/argvus/data/generated/superfile/` — projected Superfile theme set.
- `$XDG_CONFIG_HOME/argvus/data/generated/terminal/` — projected terminal profiles.
- `$XDG_CONFIG_HOME/argvus/data/generated/qt6ct/` — projected Qt6ct palette.
- `$XDG_CONFIG_HOME/argvus/data/generated/gtk/` — projected GTK theme files.
- `$XDG_CONFIG_HOME/argvus/data/rofi/` — user copies of the Rofi configuration, themes and mode. ARGVUS creates each copy when it first changes the file; unchanged files are read from `/usr/share/argvus/launcher/config/`.
- `$XDG_CONFIG_HOME/argvus/data/generated/removable-devices/theme.css` — projected removable-devices stylesheet.
- `$XDG_CONFIG_HOME/argvus/data/hypr/` — user Hyprland Lua overrides and native Hyprland projections.
- `$XDG_CONFIG_HOME/hypr/hyprland.lua` — optional complete native Hyprland configuration selected by `argvus-start`; it is not an incremental overlay.
- `$XDG_STATE_HOME/argvus/config-projection.json` — projection manifest: generation, effective-config hash, changed sections and pending runtime work. Read by `argvus-sessionctl reload`; never a source of truth.
- `$XDG_STATE_HOME/argvus/widget-telemetry-blocks` — legacy telemetry block preferences. Migration input only; canonical values live in `config.json` and this file is no longer written.
- `$XDG_STATE_HOME/argvus/session.log` — graphical session log (normally `~/.local/state/argvus/session.log`).
- `/usr/share/wayland-sessions/argvus.desktop` — graphical session entry.
- `/usr/share/plymouth/themes/argvus/` — packaged boot splash assets.

Exact payloads are package-specific; see the developer component reference and the relevant package manifest.
