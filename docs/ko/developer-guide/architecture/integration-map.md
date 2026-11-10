---
title: 통합 지도
description: ARGVUS 런타임에서의 프로젝트 간 관계.
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
기능 페이지는 사용자 행동을 설명합니다. 이 페이지는 코드를 변경할 때 사용되는 경계를 기록합니다.
