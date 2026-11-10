---
title: 集成图
description: ARGVUS 运行时的跨项目关系。
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
功能页面描述用户行为；此页面记录更改代码时使用的边界。
