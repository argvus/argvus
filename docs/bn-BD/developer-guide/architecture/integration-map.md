---
title: ইন্টিগ্রেশন মানচিত্র
description: ARGVUS রানটাইমে ক্রস-প্রকল্প সম্পর্ক।
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

বৈশিষ্ট্য পৃষ্ঠাগুলি ব্যবহারকারীর আচরণ বর্ণনা করে; এই পৃষ্ঠা কোড পরিবর্তন করার সময় ব্যবহৃত সীমানা রেকর্ড করে।
