---
title: انٹیگریشن کا نقشہ
description: ARGVUS رن ٹائم میں کراس پروجیکٹ تعلقات۔
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

فیچر کے صفحات صارف کے رویے کو بیان کرتے ہیں۔ یہ صفحہ کوڈ کو تبدیل کرتے وقت استعمال ہونے والی حدود کو ریکارڈ کرتا ہے۔
