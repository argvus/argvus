---
title: Bản đồ hội nhập
description: Mối quan hệ giữa các dự án trong thời gian chạy ARGVUS.
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

Các trang tính năng mô tả hành vi của người dùng; trang này ghi lại các ranh giới được sử dụng khi thay đổi mã.
