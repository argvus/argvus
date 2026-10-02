---
title: Integration map
description: Cross-project relationships in the ARGVUS runtime.
---

```text
greeter or TTY
  -> argvus-session -> argvus-start -> Hyprland
  -> argvus-session.target
     -> taskbar / control panel / notifications / wallpaper / idle / clipboard

Control Center -> argvus-config (canonical state) -> argvus-sessionctl reload
argvus-config   -> the only writer of data/generated/
                   Hyprland / GTK / Qt6ct / Waybar / Quickshell / Rofi / Dunst /
                   Hyprlock / Yazi / Superfile / terminal / fonts / removable-devices
theme-switch / accent-switch -> argvus-config (commit) -> external adapters
                   Qt6ct file / GTK settings.ini + gsettings / terminals /
                   system-monitor / snappy-switcher / foot / superfile / greeter
Waybar          -> popups via immutable root coordinates; calendar resolves real horizontal Taskbar geometry
```

The feature pages describe the user behavior; this page records the boundaries used when changing code.
