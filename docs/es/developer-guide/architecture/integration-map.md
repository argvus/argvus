---
title: Mapa de integración
description: Relaciones entre proyectos en el tiempo de ejecución de ARGVUS.
---

```text
greeter or TTY
  -> argvus-session -> argvus start -> Hyprland
  -> argvus-session.target
     -> taskbar / control panel / notifications / wallpaper / idle / clipboard

Control Center -> argvus config (canonical state) -> argvus sessionctl reload
argvus config   -> the only writer of data/generated/
                   Hyprland / GTK / Qt6ct / Waybar / Quickshell / Dunst /
                   Hyprlock / Yazi / Superfile / terminal / fonts / removable-devices
theme-switch / config accent -> argvus config (commit) -> external adapters
                   Qt6ct file / GTK settings.ini + gsettings / terminals /
                   system-monitor / foot / superfile / greeter
Waybar          -> popups via immutable root coordinates; calendar resolves real horizontal Taskbar geometry
```
Las páginas de funciones describen el comportamiento del usuario; esta página registra los límites utilizados al cambiar el código.
